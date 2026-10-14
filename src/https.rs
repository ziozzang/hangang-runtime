//! Bounded mandatory mTLS management server; authentication roles remain in handlers.
use anyhow::{Result, ensure};
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use rustls::{RootCertStore, ServerConfig, server::WebPkiClientVerifier};
use std::{future::Future, io::Read, path::Path, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Semaphore, task::JoinSet, time::timeout};
pub fn tls_config(cert: &Path, key: &Path, ca: &Path, alpn: &[&[u8]]) -> Result<Arc<ServerConfig>> {
    fn bytes(path: &Path) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= 1024 * 1024, "TLS material exceeds bound");
        Ok(bytes)
    }
    let cert_bytes = bytes(cert)?;
    let key_bytes = bytes(key)?;
    let ca_bytes = bytes(ca)?;
    let certs =
        rustls_pemfile::certs(&mut cert_bytes.as_slice()).collect::<std::io::Result<Vec<_>>>()?;
    let key = rustls_pemfile::private_key(&mut key_bytes.as_slice())?
        .ok_or_else(|| anyhow::anyhow!("missing TLS private key"))?;
    let mut roots = RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut ca_bytes.as_slice()) {
        roots.add(cert?)?;
    }
    ensure!(!roots.is_empty(), "client CA must not be empty");
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier =
        WebPkiClientVerifier::builder_with_provider(Arc::new(roots), provider.clone()).build()?;
    let mut config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs, key)?;
    config.alpn_protocols = alpn.iter().map(|p| p.to_vec()).collect();
    config.max_early_data_size = 0;
    Ok(Arc::new(config))
}

/// Server-injected proof of a successfully authenticated TLS client certificate.
#[derive(Clone, Debug)]
pub struct AuthenticatedTlsPeer;

pub async fn serve_https(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    router: axum::Router,
    shutdown: impl Future<Output = ()> + Send,
) -> Result<()> {
    let capacity = Arc::new(Semaphore::new(2048));
    let mut tasks = JoinSet::new();
    tokio::pin!(shutdown);
    loop {
        tokio::select! { biased;
            _=&mut shutdown=>break,
            Some(_)=tasks.join_next(),if !tasks.is_empty()=>{},
            accepted=listener.accept()=>{
                let (socket,peer)=accepted?;
                let Ok(permit)=capacity.clone().try_acquire_owned() else {continue;};
                socket.set_nodelay(true)?;
                let acceptor=tokio_rustls::TlsAcceptor::from(config.clone());
                let router=router.clone().layer(axum::Extension(axum::extract::ConnectInfo(peer)));
                tasks.spawn(async move {
                    let _permit=permit;
                    if let Ok(Ok(stream))=timeout(Duration::from_secs(3),acceptor.accept(socket)).await {
                        if stream.get_ref().1.peer_certificates().is_none_or(|certs| certs.is_empty()) { return; }
                        let router=router.layer(axum::Extension(AuthenticatedTlsPeer));
                        let mut builder=hyper::server::conn::http1::Builder::new();
                        builder.timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(5)).max_headers(64).max_buf_size(32*1024);
                        let _=timeout(Duration::from_secs(300),builder.serve_connection(TokioIo::new(stream),TowerToHyperService::new(router))).await;
                    }
                });
            }
        }
    }
    if timeout(Duration::from_secs(5), async {
        while tasks.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
    }
    Ok(())
}
