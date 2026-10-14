# Hangang Runtime

Shared server infrastructure for Hangang World. Identity, authorization and network decisions remain owned by Hopae, Pumgye and Sumunjang.

Initial scope: mandatory gRPC mTLS, bounded TLS files, connection admission (2,048), per-connection concurrency/streams (64), header limits (8 KiB), request timeout (1 second), and TCP_NODELAY. Domain handlers retain credential roles, global decision admission, message-size limits and snapshot evaluation.

No policy, SQL, provider or ontology ownership. Native codecs, supervision, listener handover and telemetry remain future extractions. This library preserves independent server deployment.

[한국어](README.ko.md)
