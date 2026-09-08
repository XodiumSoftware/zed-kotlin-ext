# zed-kotlin-ext task runner
# Install `just` once: https://github.com/casey/just

_default:
    @just --list

# Lint the extension for the wasm target
lint-ext:
    cargo clippy --target wasm32-wasip2 --all-targets

# Lint the proxy for the native target
lint-proxy:
    cd proxy && cargo clippy --all-targets

# Run all lints
lint: lint-ext lint-proxy

# Check formatting across both crates
fmt-check:
    cargo fmt -- --check
    cd proxy && cargo fmt -- --check

# Format both crates
fmt:
    cargo fmt
    cd proxy && cargo fmt

# Run the proxy test suite (the extension has no automated tests)
test:
    cd proxy && cargo test

# Find unused dependencies in both crates
machete:
    cargo machete
    cd proxy && cargo machete

# Build the extension wasm (release)
build-ext:
    cargo build --release --target wasm32-wasip2

# Build the proxy binary (release)
build-proxy:
    cd proxy && cargo build --release

# Run the full validation suite
validate: lint fmt-check test machete
