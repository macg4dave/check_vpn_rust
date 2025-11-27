# use Dockerfile syntax that supports BuildKit cache mounts
# (ensures RUN --mount=type=cache is recognized by older/bundled builders)
# syntax = docker/dockerfile:1.4

FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

# Install system packages required to build and run tests and setup toolchain
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential \
       ca-certificates \
       curl \
       git \
       pkg-config \
       libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install rustup and the stable toolchain so this image supports the lockfile
# and recent Cargo features used by CI/tests.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && /root/.cargo/bin/rustc --version

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/src/check_vpn

# Use a minimal build context pattern: copy Cargo manifest first so Docker
# caches dependencies and the build context doesn't need to include the
# whole repo for dependency resolution. We create a tiny dummy `src`
# file so `cargo build` can run and populate the cargo cache layer.
# This allows subsequent COPY . . to be much faster.
COPY Cargo.toml Cargo.lock ./

# Dummy build step to populate the cargo cache (safe if the real source
# is added later). Use BuildKit cache mounts so cargo index/registry and
# target are cached between builds and we don't repeatedly re-download
# crates for each image or run.
RUN --mount=type=cache,id=cargo-registry,target=/root/.cargo/registry \
    --mount=type=cache,id=cargo-git,target=/root/.cargo/git \
    --mount=type=cache,id=target,target=/usr/src/check_vpn/target \
    mkdir -p src && printf 'fn main() { println!("build-cache"); }\n' > src/main.rs \
    && cargo build --release || true

# Now copy the full repo and build the real release.
## Selectively copy only the source, tests and examples so the build
## context remains small and Docker layer cache is effective. Avoid
## sending unrelated top-level files that increase build context size.
COPY src ./src
COPY tests/support ./tests/support
COPY examples ./examples

# Build the real project (now that full source exists). Use the same cache
# mounts so the dependency cache persists between the earlier seeding step
# and the real build.
RUN --mount=type=cache,id=cargo-registry,target=/root/.cargo/registry \
    --mount=type=cache,id=cargo-git,target=/root/.cargo/git \
    --mount=type=cache,id=target,target=/usr/src/check_vpn/target \
    cargo build --release

# Run tests by default
CMD ["/bin/sh", "-c", "cargo test --verbose"]

# To run ignored (real-network) tests interactively, launch the container and
# call: cargo test -- --ignored
