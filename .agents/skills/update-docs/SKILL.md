---
name: update-docs
description: Keeps README.md, GUIDE.md, and AGENTS.md in sync with the current code after behavior, feature, or workflow changes.
---

# Update Docs

Use this skill after code or workflow changes that users or future agents would
notice, or when the user asks to "update the docs" / "keep docs in sync".

## When to Use

- After adding/removing user-facing settings (`lsp.kotlin-dev-lsp.settings`,
  `binary.path`, `proxy_path`)
- After changing the proxy behavior, LSP version pin, or grammar pin
- After changing CI/release flow
- Before cutting a release

## Steps

1. Inspect what changed: read `src/kotlin.rs`, `src/lsp.rs`, `src/proxy.rs`,
   `proxy/src/`, `extension.toml`, and `.github/workflows/` as relevant.

2. Update the affected files:
   - **README.md** — Features bullets, note blocks (e.g. the LSP pin note),
     Requirements. Keep the Xodium badge/link template intact; shield/link
     definitions live at the bottom of the file.
   - **GUIDE.md** — Configuration snippets (keep JSON valid), Features list,
     Known limitations, Troubleshooting entries. Update its Table of Contents
     if sections changed.
   - **AGENTS.md** — Project Structure tree, Quick Commands, Key Conventions,
     Versioning & Releases, Important Notes. This file is the primary context
     for future agent sessions; keep it accurate.

3. There is no generated documentation (no Dokka/rustdoc publishing) — the
   markdown files are the docs.

4. Cross-check version numbers: `extension.toml` version vs. anything quoted in
   docs; `src/lsp.rs` pinned LSP build vs. the README note.

After finishing, summarize what documentation was updated and ask the user if
they want to commit.
