<div id="readme-top"></div>

<h1 align="center">
  <br />
    <a href="https://xodium.org/">
        <img src="logo.svg" alt="zed-kotlin-ext Logo" width="200">
    </a>
  <br /><br />
  zed-kotlin-ext
  <br />
  <br />
</h1>

<h4 align="center">Kotlin language support for Zed, published as the Kotlin DEV extension</h4><br />

<div align="center">

[![Contributors][contributors_shield_url]][contributors_url]
[![Issues][issues_shield_url]][issues_url]
</div>

## Table of Contents

- [Features](#features)
- [Guide](GUIDE.md)
- [Built With](#built-with)
- [Code of Conduct][code_of_conduct_url]
- [Contributing][contributing_url]
- [License][license_url]
- [Requirements](#requirements)
- [Security][security_url]

## Requirements

- [Zed](https://zed.dev/) (latest stable)

The Kotlin LSP binary and the `kotlin-lsp-proxy` companion binary are downloaded
automatically on first use.

<p align="right"><a href="#readme-top">▲</a></p>

## Features

- **Language server** — powered by JetBrains' official [Kotlin LSP](https://github.com/Kotlin/kotlin-lsp): completion, diagnostics, quick fixes, hover, rename, formatting, semantic highlighting, call hierarchy, and folding. The server binary is downloaded automatically on first use and cached per version.
- **Library/JDK source navigation** — a stdio proxy in front of the language server resolves archive-internal (`jar!/`, `zip!/`) source URIs into real files, so go-to-definition into any dependency or JDK class just works.
- **Syntax highlighting** — tree-sitter based, with KDoc (`/** */`) highlighted distinctly from ordinary comments.
- **Outline** — classes, objects, companion objects, type aliases, enum entries, functions, and properties in the outline panel and breadcrumbs.
- **Run & test from the gutter** — runnable buttons on `fun main()` and on test functions/classes annotated with `@Test`, `@ParameterizedTest`, or `@RepeatedTest`, including tests in `@Nested` inner classes:
  - Gradle projects run `./gradlew test --tests ...` / `./gradlew run`, automatically targeting `:module:test` in multi-module builds
  - Maven projects run `mvn test -Dtest=...` (or `./mvnw` when present)
- **Text objects (Vim mode)** — `if`/`af` for functions (including lambdas and constructors), `ic`/`ac` for classes, and comment objects, powered by tree-sitter queries.

## Language Server

This extension uses the official [Kotlin LSP](https://github.com/kotlin/kotlin-lsp) from JetBrains. This is the actively maintained, modern language server for Kotlin.

> **Note:** This extension temporarily pins build `263.4421.0` because JetBrains'
> `RELEASES.md` lags behind actual releases and points at expired builds
> (see [Kotlin/kotlin-lsp#271](https://github.com/Kotlin/kotlin-lsp/issues/271)).

Configuration and manual-installation options are documented in the [Guide](GUIDE.md#configuration).

## License

zed-kotlin-ext as a whole is distributed under [GPL-3.0][license_url].
Portions derived from the upstream [zed-extensions/kotlin](https://github.com/zed-extensions/kotlin)
repository remain under the [MIT License](LICENSE).

<p align="right"><a href="#readme-top">▲</a></p>

## Built With

<div align="center">

[![Built With][built_with_shield_url]][built_with_url]
</div>

<p align="right"><a href="#readme-top">▲</a></p>

[built_with_shield_url]: https://skillicons.dev/icons?i=rust,kotlin,github,githubactions

[built_with_url]: https://skillicons.dev

[code_of_conduct_url]: https://github.com/XodiumSoftware/zed-kotlin-ext?tab=coc-ov-file

[contributing_url]: https://github.com/XodiumSoftware/zed-kotlin-ext/blob/main/CONTRIBUTING.md

[contributors_shield_url]: https://img.shields.io/github/contributors/XodiumSoftware/zed-kotlin-ext?style=for-the-badge&color=blue

[contributors_url]: https://github.com/XodiumSoftware/zed-kotlin-ext/graphs/contributors

[issues_shield_url]: https://img.shields.io/github/issues/XodiumSoftware/zed-kotlin-ext?style=for-the-badge&color=yellow

[issues_url]: https://github.com/XodiumSoftware/zed-kotlin-ext/issues

[license_url]: https://github.com/XodiumSoftware/zed-kotlin-ext/blob/main/LICENSE.md

[security_url]: https://github.com/XodiumSoftware/zed-kotlin-ext?tab=security-ov-file
