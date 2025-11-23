# Multi-stage build for smaller image
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace manifest (needed for shared crates)
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (needed for workspace resolution)
COPY client ./client
COPY server ./server
COPY shared ./shared

# Build only the server package
RUN cargo build --release --package genxlink-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/genxlink-server /usr/local/bin/

# Create non-root user
RUN useradd -m -u 1000 genxlink && \
    chown -R genxlink:genxlink /usr/local/bin/genxlink-server

USER genxlink

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run server
CMD ["genxlink-server"]
