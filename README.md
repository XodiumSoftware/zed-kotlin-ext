# zed-kotlin-ext

Kotlin language support for [Zed](https://github.com/zed-industries/zed), published as the **Kotlin DEV** extension.

## Features

- **Language server** — powered by JetBrains' official [Kotlin LSP](https://github.com/Kotlin/kotlin-lsp): completion, diagnostics, quick fixes, hover, rename, formatting, semantic highlighting, call hierarchy, and folding. The server binary is downloaded automatically on first use and cached per version.
- **Syntax highlighting** — tree-sitter based, with KDoc (`/** */`) highlighted distinctly from ordinary comments.
- **Outline** — classes, objects, companion objects, type aliases, enum entries, functions, and properties in the outline panel and breadcrumbs.
- **Run & test from the gutter** — runnable buttons on `fun main()` and on test functions/classes annotated with `@Test`, `@ParameterizedTest`, or `@RepeatedTest`, including tests in `@Nested` inner classes:
  - Gradle projects run `./gradlew test --tests ...` / `./gradlew run`, automatically targeting `:module:test` in multi-module builds
  - Maven projects run `mvn test -Dtest=...` (or `./mvnw` when present)
- **Text objects (Vim mode)** — `if`/`af` for functions (including lambdas and constructors), `ic`/`ac` for classes, and comment objects, powered by tree-sitter queries.

## Language Server

### Kotlin LSP

This extension uses the official [Kotlin LSP](https://github.com/kotlin/kotlin-lsp) from JetBrains. This is the actively maintained, modern language server for Kotlin.

> **Note:** This extension temporarily pins build `263.4421.0` because JetBrains'
> `RELEASES.md` lags behind actual releases and points at expired builds
> (see [Kotlin/kotlin-lsp#271](https://github.com/Kotlin/kotlin-lsp/issues/271)).

#### Configuration

Workspace configuration options can be passed to the language server via lsp
settings in `settings.json`:

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

#### Manual Installation

Kotlin LSP will be downloaded and updated automatically. To use a manually installed version, set the path to the `intellij-server` executable in the release assets:

```json
{
  "lsp": {
    "kotlin-dev-lsp": {
      "binary": {
        "path": "path/to/bin/intellij-server",
        "arguments": [ "--stdio" ]
      }
    }
  }
}
```
