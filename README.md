# zed-kotlin

Kotlin language support for [Zed](https://github.com/zed-industries/zed).

## Language Server

### Kotlin Language Server

This extension uses the [Kotlin Language Server](https://github.com/fwcd/kotlin-language-server), which is currently the most stable and actively maintained LSP for Kotlin.

#### Configuration

Workspace configuration options can be passed to the language server via lsp
settings in `settings.json`:

```json
{
  "lsp": {
    "kotlin-language-server": {
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

The full list of workspace configuration options can be found
[here](https://github.com/fwcd/kotlin-language-server/blob/main/server/src/main/kotlin/org/javacs/kt/Configuration.kt).

## Requirements

- JDK 11+ (JDK 17 or 21 recommended)
- Gradle or Maven project (optional, but recommended for dependency resolution)

## Notes

This extension was updated to use the community `fwcd/kotlin-language-server` because the JetBrains Kotlin LSP has issues with expiring builds that break functionality. The fwcd server is more reliable and doesn't require JDK 25 or expire after 30 days.
