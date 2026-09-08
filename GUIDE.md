# Installation

## Table of Contents

- [Prerequisites](#prerequisites)
- [Install from Zed](#install-from-zed)
- [Build from Source](#build-from-source)
- [Configuration](#configuration)
- [Features](#features)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

- [Zed](https://zed.dev/) (latest stable)

The language server ([JetBrains Kotlin LSP](https://github.com/Kotlin/kotlin-lsp)) and the
`kotlin-lsp-proxy` binary are downloaded automatically on first use — no manual setup needed.

## Install from Zed

1. Open Zed
2. Run `zed: extensions` from the command palette
3. Search for **Kotlin DEV** and click Install

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- The wasm target: `rustup target add wasm32-wasip2`
- [Git](https://git-scm.com/)

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/XodiumSoftware/zed-kotlin-ext.git
   cd zed-kotlin-ext
   ```

2. Open Zed, run `zed: install dev extension`, and select the cloned directory.
   Zed compiles the extension wasm and registers it automatically.

### Building the proxy binary manually

The `kotlin-lsp-proxy` companion binary is normally downloaded from GitHub Releases.
To build it locally (e.g. for testing unreleased changes):

```bash
cd proxy
cargo build --release
```

The binary lands at `proxy/target/release/kotlin-lsp-proxy` (`.exe` on Windows).
Point the extension at it via `proxy_path` (see [Configuration](#configuration)).

## Configuration

Workspace settings are forwarded to the language server from `lsp.kotlin-dev-lsp.settings`:

```json
{
  "lsp": {
    "kotlin-dev-lsp": {
      "settings": {
        "compiler": {
          "jvm": {
            "target": "17"
          }
        }
      }
    }
  }
}
```

### Extension-level settings

These keys live under the same `settings` object but are consumed by the extension itself
and are not forwarded to the server:

```json
{
  "lsp": {
    "kotlin-dev-lsp": {
      "settings": {
        "proxy_path": "/absolute/path/to/kotlin-lsp-proxy"
      }
    }
  }
}
```

- **`proxy_path`** — bypass the GitHub Releases download and use a locally-built or
  manually-placed `kotlin-lsp-proxy` binary. `~` is expanded on Unix.

To use a manually installed language server binary instead of the auto-downloaded one:

```json
{
  "lsp": {
    "kotlin-dev-lsp": {
      "binary": {
        "path": "path/to/bin/intellij-server",
        "arguments": ["--stdio"]
      }
    }
  }
}
```

## Features

- **Language server** — completion, diagnostics, quick fixes, hover, rename, formatting,
  semantic highlighting, call hierarchy, and folding via JetBrains' Kotlin LSP
- **Library/JDK source navigation** — go-to-definition into dependencies and JDK classes
  resolves archive-internal sources to real files via the bundled `kotlin-lsp-proxy`
- **Syntax highlighting** — tree-sitter based, with KDoc (`/** */`) highlighted distinctly
  from ordinary comments
- **Outline** — classes, objects, companion objects, type aliases, enum entries, functions,
  and properties in the outline panel and breadcrumbs
- **Run & test from the gutter** — runnable buttons on `fun main()` and tests annotated with
  `@Test`, `@ParameterizedTest`, or `@RepeatedTest` (Gradle and Maven aware)
- **Text objects (Vim mode)** — `if`/`af` for functions, `ic`/`ac` for classes, comment objects

### Known limitations

- **Kotlin scripts (`.kts`)** get syntax highlighting only. Language server features
  (navigation, refactoring) for scripts depend on
  [Kotlin/kotlin-lsp#229](https://github.com/Kotlin/kotlin-lsp/issues/229) upstream.

## Troubleshooting

### Language server doesn't start

- Check `zed: open language server logs` → `kotlin-dev-lsp` for the actual error
- The extension downloads the server from JetBrains on first use — verify network access
- A leftover incomplete install is cleaned up automatically on the next start;
  if problems persist, remove the `kotlin-dev-lsp-*` directories from the extension
  work directory and retry

### Go-to-definition into a library class opens an empty buffer

- Check the language server logs for `kotlin-lsp-proxy unavailable ...` — the extension
  fell back to launching kotlin-lsp without the proxy
- The proxy ships as a GitHub Release asset; confirm a release exists with the
  `kotlin-lsp-proxy-<os>-<arch>` binary for your platform, or set `proxy_path`
  to a locally-built binary

### Manual LSP binary not used

- The `binary.path` setting requires an absolute path to the `intellij-server`
  executable, and `"arguments": ["--stdio"]` must be present
- Note that with a manual binary, the proxy is still applied on top

---

<p align="right"><a href="#readme-top">▲</a></p>
