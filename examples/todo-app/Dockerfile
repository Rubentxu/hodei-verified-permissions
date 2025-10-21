# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace root files
COPY ../../Cargo.toml ../../Cargo.toml
COPY ../../sdk ../../sdk
COPY ../../cli ../../cli
COPY ../../hodei-macros ../../hodei-macros
COPY ../../proto ../../proto

# Copy TODO app
COPY . .

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/todo-app /usr/local/bin/todo-app

# Copy static files (policies, schema, etc.)
COPY policies ./policies
COPY v4.cedarschema.json ./v4.cedarschema.json
COPY openapi.json ./openapi.json

# Expose HTTP port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV AUTH_ENDPOINT=http://hodei-server:50051
ENV POLICY_STORE_ID=todo-policy-store
ENV IDENTITY_SOURCE_ID=todo-identity-source

# Run app
CMD ["todo-app"]
