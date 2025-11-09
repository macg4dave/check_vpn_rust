FROM rust:1.73-slim
WORKDIR /usr/src/check_vpn

# Install basic build deps (ssl, pkg-config) and ca certs for HTTPS tests
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Build the project once (will cache if using Docker build cache)
RUN cargo build --release

# Run tests by default
CMD ["/bin/sh", "-c", "cargo test --verbose"]

# To run ignored (real-network) tests interactively, launch the container and
# call: cargo test -- --ignored
