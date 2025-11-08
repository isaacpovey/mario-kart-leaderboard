# Multistage Dockerfile for building a Rust application.
# Usage:
#   docker build -t myapp --build-arg BINARY=your_binary_name .
# If your crate binary name is different from "backend", pass it with --build-arg BINARY=...

ARG RUST_IMAGE=rust:latest
ARG BINARY=mario-kart-leaderboard-backend

FROM ${RUST_IMAGE} AS builder
WORKDIR /usr/src/app

# Cache dependencies: copy manifest first and build a dummy to populate cargo cache
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

# Final slim runtime image
FROM debian:bookworm-slim
# Install CA certs for HTTPS clients (adjust if not needed)
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage. Configure BINARY at build time if needed.
ARG BINARY=mario-kart-leaderboard-backend
COPY --from=builder /usr/src/app/target/release/${BINARY} /usr/local/bin/${BINARY}

# Ensure binary is executable
RUN chmod +x /usr/local/bin/${BINARY}

# Use a non-root user for security
RUN useradd -u 1000 -m appuser && chown appuser:appuser /usr/local/bin/${BINARY}
USER appuser

WORKDIR /home/appuser
ENV RUST_LOG=info

# Set a sensible default; override with "docker run ... <args>" if your binary needs args
ENTRYPOINT ["/usr/local/bin/mario-kart-leaderboard-backend"]