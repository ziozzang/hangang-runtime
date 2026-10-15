//! Shared bounded transport; domain decisions stay in their owning modules.
pub mod time;
use anyhow::{Result, ensure};
use std::{
    io,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore},
};
use tokio_stream::{StreamExt, wrappers::TcpListenerStream};
use tonic::transport::{
    Certificate, Identity, Server, ServerTlsConfig,
    server::{Connected, TcpConnectInfo},
};

pub fn configuration(cert: &Path, key: &Path, ca: &Path) -> Result<ServerTlsConfig> {
    fn bounded(path: &Path) -> Result<Vec<u8>> {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= 1024 * 1024,
            "gRPC TLS material exceeds bound"
        );
        Ok(bytes)
    }
    Ok(ServerTlsConfig::new()
        .identity(Identity::from_pem(bounded(cert)?, bounded(key)?))
        .client_ca_root(Certificate::from_pem(bounded(ca)?))
        .client_auth_optional(false)
        .timeout(Duration::from_secs(3)))
}
pub struct Connection {
    socket: TcpStream,
    _permit: OwnedSemaphorePermit,
}
impl Connected for Connection {
    type ConnectInfo = TcpConnectInfo;
    fn connect_info(&self) -> Self::ConnectInfo {
        self.socket.connect_info()
    }
}
impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().socket).poll_read(cx, buffer)
    }
}
impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().socket).poll_write(cx, bytes)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().socket).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().socket).poll_shutdown(cx)
    }
}

pub fn incoming(listener: TcpListener) -> impl tokio_stream::Stream<Item = io::Result<Connection>> {
    let connections = Arc::new(Semaphore::new(2048));
    TcpListenerStream::new(listener).filter_map(move |entry| match entry {
        Ok(socket) => {
            let permit = connections.clone().try_acquire_owned().ok()?;
            if let Err(error) = socket.set_nodelay(true) {
                return Some(Err(error));
            }
            Some(Ok(Connection {
                socket,
                _permit: permit,
            }))
        }
        Err(error) => Some(Err(error)),
    })
}
pub fn server(tls: ServerTlsConfig) -> Result<Server> {
    Ok(Server::builder()
        .tls_config(tls)?
        .concurrency_limit_per_connection(64)
        .max_concurrent_streams(Some(64))
        .http2_max_header_list_size(Some(8192))
        .timeout(Duration::from_millis(1000)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tls_files_are_bounded_and_missing_files_fail() {
        use std::io::Write;
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing");
        assert!(configuration(&missing, &missing, &missing).is_err());
        let oversized = root.path().join("oversized");
        std::fs::File::create(&oversized)
            .unwrap()
            .write_all(&vec![b'x'; 1024 * 1024 + 1])
            .unwrap();
        let error = configuration(&oversized, &missing, &missing).unwrap_err();
        assert!(error.to_string().contains("exceeds bound"));
    }
}

mod https;
pub use https::{AuthenticatedTlsPeer, serve_https, tls_config};

mod connections;
pub use connections::serve_connections;
