# SafeHold Docker Build
# Multi-stage build for minimal production image

FROM rust:1.83-bullseye as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --features cli
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY README.md CHANGELOG.md LICENSE ./

# Build the application
RUN cargo build --release --features cli

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 safehold

# Copy the binary
COPY --from=builder /app/target/release/safehold /usr/local/bin/safehold

# Set up data directory
RUN mkdir -p /app/data && chown safehold:safehold /app/data

# Switch to non-root user
USER safehold
WORKDIR /app

# Set environment variables
ENV SAFEHOLD_DATA_DIR=/app/data
ENV RUST_LOG=info

# Expose volume for persistent data
VOLUME ["/app/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD safehold version || exit 1

# Default command
CMD ["safehold", "--help"]