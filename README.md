# Hangang Runtime

Shared server infrastructure for Hangang World. Identity, authorization and network decisions remain owned by Hopae, Pumgye and Sumunjang.

Initial scope: mandatory gRPC mTLS, bounded TLS files, connection admission (2,048), per-connection concurrency/streams (64), header limits (8 KiB), request timeout (1 second), and TCP_NODELAY. Domain handlers retain credential roles, global decision admission, message-size limits and snapshot evaluation.

No policy, SQL, provider or ontology ownership. Native codecs, supervision, listener handover and telemetry remain future extractions. This library preserves independent server deployment.

[한국어](README.ko.md)

The shared TLS parser also builds native/management mTLS configurations. `serve_https` provides bounded HTTP/1.1 management connections (2,048), 3-second TLS handshake, 64 headers/32 KiB buffer, 5-second header read, 300-second connection lifetime and 5-second shutdown drain. Handler roles and body limits stay domain owned; this is not the native decision protocol.

`serve_connections` shares bounded TCP admission and connection-task ownership.
It preserves actual peer addresses, applies TCP_NODELAY, rejects excess sockets,
drains for a caller-selected duration and aborts/joins remaining tasks. Cancellation
of the owning server also aborts its tasks. Handlers own TLS, native framing,
credentials, per-connection lifetime and domain semantics. Management HTTPS now
uses this primitive; native listener adoption remains a separate consumer gate.
Actual socket tests cover capacity retention/reuse, shutdown and owner cancellation.
