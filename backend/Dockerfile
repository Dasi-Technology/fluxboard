# Multi-stage build for Fluxboard backend

# Builder stage
FROM rust:1.91 AS builder

# Create app directory
WORKDIR /usr/src/app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build for release
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 app

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/fluxboard-backend /app/fluxboard-backend

# Copy migrations
COPY --from=builder /usr/src/app/migrations /app/migrations

# Change ownership
RUN chown -R app:app /app

# Switch to app user
USER app

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

# Run the binary
CMD ["/app/fluxboard-backend"]