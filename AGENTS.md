# zed-kotlin-ext — Agents Context

## Project at a Glance

- **Name:** zed-kotlin-ext (published as the **Kotlin DEV** extension, id `kotlin-dev`)
- **Type:** Zed editor extension — compiled to wasm32-wasip2, plus a native companion binary
- **Language:** Rust (edition 2024)
- **Build Tool:** Cargo
- **Upstream:** Forked from [zed-extensions/kotlin](https://github.com/zed-extensions/kotlin) (MIT — attribution kept in `LICENSE`; this fork's contributions are AGPL-3.0, see `LICENSE.md`)

## APIs & Tools

| Category              | Technology                                                    | Purpose                                    |
|-----------------------|---------------------------------------------------------------|--------------------------------------------|
| **Extension API**     | [zed_extension_api](https://crates.io/crates/zed_extension_api) 0.7.0 | Zed extension host API           |
| **Language server**   | [kotlin-lsp](https://github.com/Kotlin/kotlin-lsp) (JetBrains)        | Kotlin language features; downloaded at runtime from JetBrains' CDN |
| **Grammar**           | [tree-sitter-kotlin](https://github.com/fwcd/tree-sitter-kotlin)      | Syntax highlighting/queries (pinned by commit in `extension.toml`) |
| **Companion binary**  | `proxy/` crate (`kotlin-lsp-proxy`)                           | Native stdio proxy; rewrites archive-internal (`jar!/`/`zip!/`) LSP location URIs so library/JDK source navigation works |
| **Style**             | rustfmt (`cargo fmt`)                                         | Edition-2024 import ordering; enforce before committing |

## Quick Commands

```bash
# Check the extension (wasm target)
cargo check --target wasm32-wasip2

# Format everything (run from repo root AND proxy/)
cargo fmt

# Proxy: unit tests + native build
cd proxy
cargo test
cargo build --release   # binary: proxy/target/release/kotlin-lsp-proxy[.exe]
```

## Project Structure

```
zed-kotlin-ext/
├── extension.toml              # Extension manifest: id, version, language server, grammar pin
├── Cargo.toml                  # Extension wasm crate (no version field; versioning lives in extension.toml)
├── src/
│   ├── kotlin.rs               # Extension entry point: LSP command wiring, proxy wrap + fallback, workspace config
│   ├── lsp.rs                  # KotlinLSP: downloads/caches the JetBrains LSP binary (version pin in get_version)
│   ├── proxy.rs                # Proxy: downloads/caches kotlin-lsp-proxy from GitHub Releases, proxy_path escape hatch
│   └── utils.rs                # Shared helpers (outdated-install cleanup)
├── proxy/                      # kotlin-lsp-proxy: standalone native crate (own workspace, NEVER in the wasm build)
│   └── src/
│       ├── main.rs             # stdio proxy: spawns LSP, forwards traffic, intercepts definition-family responses
│       ├── lsp.rs              # LSP framing (Content-Length + JSON body)
│       ├── log.rs              # window/logMessage logging to Zed's server logs panel
│       ├── resolve.rs          # archive URI split/extract/rewrite (+ unit tests)
│       └── platform/           # parent-death monitor (unix.rs / windows.rs via #[path] from main.rs)
├── languages/kotlin/           # config.toml, tree-sitter queries (.scm), tasks.json
├── grammars/                   # Grammar checkout (gitignored, managed by Zed)
└── .github/workflows/
    └── release-proxy.yml       # Auto-tags v<extension.toml version>, creates the release, builds proxy for 6 platforms
```

## Key Conventions

- **Doc comments** (`//!` module docs, `///` on public items) — the codebase documents intent; keep them accurate when changing behavior.
- **Line endings/formatting** — run `cargo fmt` before committing; `.editorconfig` enforces LF.
- **Paths over configuration** — the only user-facing settings are `lsp.kotlin-dev-lsp.settings` (forwarded to the server under the `kotlin` key, minus `proxy_path`) and `binary.path`/`proxy_path` overrides.
- **Proxy isolation** — `proxy/` must stay its own `[workspace]` so it never links into the wasm build; it must build on Windows, macOS, and Linux (all code gates via `#[cfg]`).
- **License hygiene** — upstream-derived code keeps MIT attribution (`LICENSE`); this fork's files are AGPL-3.0 (`LICENSE.md`).

## Versioning & Releases

- There is **no automated version bump** (the upstream registry workflow was removed).
  Bump `version` in `extension.toml` manually in the release commit.
- Pushing to `main` runs `release-proxy.yml`: if tag `v<version>` doesn't exist yet, it
  creates the tag and a GitHub Release, builds the proxy for all 6 platforms, and attaches
  the archives. Existing tag → run is a no-op; `workflow_dispatch` only validates builds.
- The extension resolves the proxy at runtime via the **latest GitHub Release** of this
  repo, so every release must carry the full set of `kotlin-lsp-proxy-*` assets.

## Testing

- No automated tests for the wasm extension side.
- The proxy has unit tests: `cd proxy && cargo test` (archive URI splitting, scheme stripping, percent-decoding — Windows and Unix variants are cfg-gated).
- Manual verification: `zed: install dev extension` on this repo, open a Kotlin project, restart the language server, and ctrl-click into a JDK/library class — it should open extracted source, not an empty buffer.

## Important Notes

- The LSP version is pinned in `src/lsp.rs` (`get_version`) because JetBrains' `RELEASES.md` points at expired builds ([Kotlin/kotlin-lsp#271](https://github.com/Kotlin/kotlin-lsp/issues/271)) — re-check the pin when updating.
- `.kts` scripts get highlighting only; LSP support is tracked upstream at [Kotlin/kotlin-lsp#229](https://github.com/Kotlin/kotlin-lsp/issues/229) — do not add workaround code for it.
- When passing the LSP binary as an *argument* to the proxy, make it absolute first (`current_dir().join(...)`): Zed only resolves a relative `command`, not `args`.
