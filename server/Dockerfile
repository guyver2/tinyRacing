# Build stage
# Using stable Rust to support edition2024 required by some dependencies
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libncurses-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY server/Cargo.toml server/Cargo.lock ./

# Copy source code
COPY server/src ./src
COPY server/migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies including postgresql-client for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libncurses6 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/tiny_racing /app/tiny_racing

# Copy migrations directory
COPY server/migrations ./migrations

# Copy entrypoint script
COPY server/docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Assets will be mounted as volume, but create directory
RUN mkdir -p /app/assets

# Expose ports
EXPOSE 3000 3030

# Use entrypoint script
ENTRYPOINT ["/app/docker-entrypoint.sh"]

