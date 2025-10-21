# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY shared ./shared
COPY domain ./domain
COPY application ./application
COPY infrastructure ./infrastructure
COPY api ./api
COPY main ./main

# Copy proto files from root
COPY ../proto ../proto

# Build release binary
RUN cargo build --release -p hodei-verified-permissions

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/hodei-verified-permissions /usr/local/bin/hodei-server

# Create data directory for SQLite
RUN mkdir -p /app/data

# Expose gRPC port
EXPOSE 50051

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///app/data/hodei.db

# Run server
CMD ["hodei-server"]
