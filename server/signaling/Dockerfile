# GenXLink Signaling Server Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/genxlink-signaling /usr/local/bin/genxlink-signaling

EXPOSE 8080

CMD ["genxlink-signaling"]
