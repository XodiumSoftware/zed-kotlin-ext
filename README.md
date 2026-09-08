# zed-kotlin

Kotlin language support for [Zed](https://github.com/zed-industries/zed).

## Language Server

### Kotlin LSP

This extension uses the official [Kotlin LSP](https://github.com/kotlin/kotlin-lsp) from JetBrains. This is the actively maintained, modern language server for Kotlin.

#### Configuration

Workspace configuration options can be passed to the language server via lsp
settings in `settings.json`:

```json
{
  "lsp": {
    "kotlin-lsp": {
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

Kotlin LSP will be downloaded and updated automatically. To use a manually installed version, set the path to the `kotlin-lsp.sh` script in the release assets:

```json
{
  "lsp": {
    "kotlin-lsp": {
      "binary": {
        "path": "path/to/kotlin-lsp.sh",
        "arguments": [ "--stdio" ]
      }
    }
  }
}
```

Note that the `kotlin-lsp.sh` script expects to be run from within the unzipped release zip file, and should not be moved elsewhere.
