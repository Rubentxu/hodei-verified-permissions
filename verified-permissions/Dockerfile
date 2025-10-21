# Build stage
FROM rust:latest as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy proto files first (from root context)
COPY proto ./proto

# Copy workspace files
COPY verified-permissions/Cargo.toml ./
COPY verified-permissions/shared ./shared
COPY verified-permissions/domain ./domain
COPY verified-permissions/application ./application
COPY verified-permissions/infrastructure ./infrastructure
COPY verified-permissions/api ./api
COPY verified-permissions/main ./main

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
