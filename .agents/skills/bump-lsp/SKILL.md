---
name: bump-lsp
description: Updates the pinned JetBrains kotlin-lsp build version in src/lsp.rs and verifies the download wiring still matches JetBrains' layout.
---

# Bump Kotlin LSP Version

Use this skill when updating the hardcoded Kotlin LSP build pin.

## Background

The server version is pinned in `get_version()` in `src/lsp.rs` because
JetBrains' `RELEASES.md` lags behind actual releases and points at expired
builds ([Kotlin/kotlin-lsp#271](https://github.com/Kotlin/kotlin-lsp/issues/271)).
Before bumping, quickly re-check that issue/discussion in case the pin can be
removed in favor of reading `RELEASES.md` again.

## Steps

1. Find available builds on
   `https://download.jetbrains.com/language-server/kotlin-server/` (check the
   version the server advertises, e.g. via the kotlin-lsp GitHub releases page).
2. Update the version string in `get_version()` in `src/lsp.rs`.
3. Verify the asset-name patterns in `download_from_teamcity()` still match
   JetBrains' published layout for every platform:
   - Windows: `kotlin-server-<version>.win.zip` / `-aarch64.win.zip`
   - macOS: `kotlin-server-<version>.sit` / `-aarch64.sit`
   - Linux: `kotlin-server-<version>.tar.gz` / `-aarch64.tar.gz`
   If the layout changed, adjust `asset_name`, `target_dir`, and `binary_path`
   together (`binary_path` points at `bin/intellij-server[.exe]`).
4. Run `cargo fmt` and `cargo check --target wasm32-wasip2`.

## Manual Verification

The install cache keys off the version (`kotlin-dev-lsp-<version>`), so a bump
forces a fresh download on next start. Ideally verify once with a dev-install
(`zed: install dev extension`, then restart the language server and check the
server logs for a successful start).

After finishing, summarize the change and ask the user if they want to commit.
