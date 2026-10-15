# Hangang Runtime

Shared server infrastructure for Hangang World. Identity, authorization and network decisions remain owned by Hopae, Pumgye and Sumunjang.

The default is a statically linked Rust library, pinned and compiled into each
server executable. The four servers remain independently deployable and scalable;
sharing this library adds no request-path network hop. Linux release binaries
default to musl and must pass static ELF verification. Updating the runtime requires
rebuilding and qualifying each consumer. Database-backed in-memory policy snapshots
and domain-owned provider extensions can still change at runtime.

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

`time::ReceiverClock` adds Linux `CLOCK_BOOTTIME` infrastructure for observed
contracts. Fixed-origin ticks include host suspend and ignore UTC clock steps;
unsupported platforms/read failures refuse without a different clock fallback.
Native startup supplies a fresh nonzero CSPRNG generation. An additional private
instance seal rejects tokens from another clock even with the same generation.
Exchange samples bind separate `LocalDeadline` and `ReplayRetention` values;
revalidation narrows admission and extends retention. Admission retirement does
not authorize nonce deletion. There is no request-path mutex, heap allocation,
wall-clock read or SQL. Test clock injection is private and test-only.

These types are numeric infrastructure, not authenticated source reports or
permission capabilities. They do not restore persisted monotonic coordinates,
fence a former distributed owner, qualify clock-rate assumptions or implement a
replay ledger. Native receipt/client/ledger adoption remains a consumer gate.

`time::ElapsedStamp` supplies a separate opaque BOOTTIME age for source proof
budgets, including time spent awaiting authentication. It has no numeric or
serialized constructor and no weaker-clock fallback. Callers must charge the
original budget at every relevant boundary; adding this primitive alone does not
make existing source adapters suspend-safe.


`ReceiverClock::sample_ticks` reads active positive process-local BOOTTIME ticks
using one backend read, without constructing an exchange or cloning its seal.
It rechecks retirement after the read and refuses read/overflow failures. This
avoids two clock reads when native adapters need their own source coordinate;
it is numeric infrastructure and cannot authenticate a remote clock or grant a
permission. Deterministic tests cover suspend, flooring, failures, overflow and
retirement overlapping the read.
