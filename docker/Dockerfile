FROM rust:1.87.0-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && \
    apt-get install -y libpcap-dev libpcap0.8-dev
COPY . .
RUN cargo build --release

FROM scratch AS artifact
COPY --from=builder /app/target/release/tcpgraph artifacts/