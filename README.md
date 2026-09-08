# zed-kotlin

Kotlin language support for [Zed](https://github.com/zed-industries/zed).

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
