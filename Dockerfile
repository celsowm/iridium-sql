# syntax=docker/dockerfile:1
FROM rust:bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN cargo build --locked --release -p iridium_server --bin iridium-server --bin iridium-healthcheck

FROM debian:bookworm-slim
RUN mkdir -p /var/lib/iridium && chown 10001:10001 /var/lib/iridium
COPY --from=builder /build/target/release/iridium-server /usr/local/bin/iridium-server
COPY --from=builder /build/target/release/iridium-healthcheck /usr/local/bin/iridium-healthcheck
ENV HOME=/var/lib/iridium RUST_LOG=info
WORKDIR /var/lib/iridium
USER 10001:10001
EXPOSE 1433
HEALTHCHECK --interval=3s --timeout=3s --start-period=5s --retries=20 CMD ["/usr/local/bin/iridium-healthcheck"]
ENTRYPOINT ["/usr/local/bin/iridium-server", "--host", "0.0.0.0", "--port", "1433", "--data-dir", "/var/lib/iridium", "--tls-gen"]
