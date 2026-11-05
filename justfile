# zed-kotlin-ext task runner
# Install `just` once: https://github.com/casey/just
#
# Recipes are identical across platforms (no shell builtins, no `mkdir`/`rm`/
# `cp`), so they behave the same from PowerShell and bash. Only recipes that
# genuinely need OS-specific tooling carry `[unix]` / `[windows]` variants.

[windows]
set shell := ["powershell.exe", "-NoProfile", "-Command"]

_default:
    @just --list

# Lint the extension for the wasm target
lint-ext:
    cargo clippy --target wasm32-wasip2 --all-targets -- -D warnings

# Lint the proxy for the native target
lint-proxy:
    cargo clippy --manifest-path proxy/Cargo.toml --all-targets -- -D warnings

# Run all lints
lint: lint-ext lint-proxy

# Check formatting across both crates
fmt-check:
    cargo fmt -- --check
    cargo fmt --manifest-path proxy/Cargo.toml -- --check

# Format both crates
fmt:
    cargo fmt
    cargo fmt --manifest-path proxy/Cargo.toml

# Run the proxy test suite (the extension has no automated tests)
test:
    cargo test --manifest-path proxy/Cargo.toml

# Find unused dependencies in both crates
machete:
    cargo machete
    cargo machete proxy/Cargo.toml

# Build the extension wasm (release)
build-ext:
    cargo build --release --target wasm32-wasip2

# Build the proxy binary (release)
build-proxy:
    cargo build --release --manifest-path proxy/Cargo.toml

# Run the full validation suite
validate: lint fmt-check test machete
