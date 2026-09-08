---
name: cut-release
description: Cuts a new extension release by bumping extension.toml, and verifies the release-proxy workflow published all six proxy binaries.
---

# Cut Release

Use this skill when the user wants to publish a new version of the extension.

## How Releasing Works

There is no automated version bump. A release is:

1. Bump `version` in `extension.toml` (semver; current scheme is `0.x.y`).
2. Commit with a message like `Bump version to v<version>` and push to `main`.
3. `release-proxy.yml` fires on the push: if tag `v<version>` doesn't exist, it
   creates the tag at that commit, creates a GitHub Release ("Kotlin DEV
   v<version>"), builds `kotlin-lsp-proxy` for all 6 platforms, and attaches
   the archives. If the tag already exists, the run is a no-op (the tag job
   exits with "nothing to release").

## Preflight

1. Confirm the working tree is clean and `main` is pushed.
2. Confirm everything builds: `cargo check --target wasm32-wasip2` and
   `cd proxy && cargo test`.
3. Sanity-check that `languages/kotlin/`, `src/`, and `proxy/` have no pending
   uncommitted work meant for this release.

## After Push

Watch the `Release Proxy Binary` run, then verify on the release page that all
six assets are attached:

- `kotlin-lsp-proxy-darwin-aarch64.tar.gz`
- `kotlin-lsp-proxy-darwin-x86_64.tar.gz`
- `kotlin-lsp-proxy-linux-x86_64.tar.gz`
- `kotlin-lsp-proxy-linux-aarch64.tar.gz`
- `kotlin-lsp-proxy-windows-x86_64.zip`
- `kotlin-lsp-proxy-windows-aarch64.zip`

Every release must carry the full set — the extension resolves the proxy via
the latest release of this repo regardless of extension version.

## If the Tag/Release Already Exists

Don't force-move version tags. Bump `extension.toml` to the next patch version
instead and re-push.
