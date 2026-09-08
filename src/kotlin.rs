//! Kotlin language support for the Zed editor.
//!
//! This extension integrates the official [Kotlin LSP](https://github.com/Kotlin/kotlin-lsp)
//! from JetBrains. The language server binary is downloaded from JetBrains'
//! servers on first use and cached for subsequent sessions. Workspace
//! configuration is forwarded from Zed's `lsp.kotlin-dev-lsp` settings.

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

mod lsp;
mod proxy;
mod utils;

use lsp::KotlinLSP;
use proxy::Proxy;

/// The Kotlin extension registered with Zed.
struct KotlinExtension {
    /// Lazily initialized manager for the Kotlin LSP binary.
    kotlin_lsp: Option<KotlinLSP>,
    /// Lazily initialized manager for the LSP proxy binary.
    proxy: Option<Proxy>,
}

impl zed::Extension for KotlinExtension {
    fn new() -> Self {
        Self {
            kotlin_lsp: None,
            proxy: None,
        }
    }

    /// Returns the command used to start the Kotlin language server.
    ///
    /// Downloads the server binary on first use; the resolved path is cached
    /// afterwards. The server is always launched with `--stdio`.
    ///
    /// # Errors
    ///
    /// Returns an error if the language server ID is unrecognized or the
    /// server binary cannot be downloaded or located.
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        match language_server_id.as_ref() {
            KotlinLSP::LANGUAGE_SERVER_ID => {
                let kotlin_lsp = self.kotlin_lsp.get_or_insert_with(KotlinLSP::new);
                let binary_path = kotlin_lsp.language_server_binary_path(language_server_id)?;

                // Run kotlin-lsp behind kotlin-lsp-proxy, which rewrites archive-internal
                // (`jar!/`, `zip!/`) source URIs into real files so Go-to-Definition
                // into library/JDK sources works. If the proxy can't be obtained, fall
                // back to launching kotlin-lsp directly so core language features keep
                // working (only library source navigation is affected).
                let configuration =
                    LspSettings::for_worktree(language_server_id.as_ref(), worktree)
                        .ok()
                        .and_then(|lsp_settings| lsp_settings.settings);

                let proxy = self.proxy.get_or_insert_with(Proxy::new);
                match proxy.language_server_binary_path(
                    language_server_id,
                    &configuration,
                    worktree,
                ) {
                    Ok(proxy_path) => {
                        // `binary_path` is relative to the extension's work directory.
                        // Zed resolves a relative `command` against that directory
                        // automatically, but does NOT resolve `args` -- so it must be
                        // made absolute here before being passed as an argument to the
                        // proxy, otherwise the proxy inherits the *worktree's* cwd and
                        // fails to find it.
                        let absolute_binary_path = std::env::current_dir()
                            .map(|dir| dir.join(&binary_path))
                            .map(|path| path.to_string_lossy().into_owned())
                            .unwrap_or(binary_path);

                        Ok(zed::Command {
                            command: proxy_path,
                            args: vec![absolute_binary_path, "--stdio".to_string()],
                            env: Default::default(),
                        })
                    }
                    Err(err) => {
                        eprintln!(
                            "kotlin-lsp-proxy unavailable ({err}); launching kotlin-lsp directly"
                        );
                        Ok(zed::Command {
                            command: binary_path,
                            args: vec!["--stdio".to_string()],
                            env: Default::default(),
                        })
                    }
                }
            }
            _ => Err(format!(
                "Unrecognized language server for Kotlin: {language_server_id}"
            )),
        }
    }

    /// Provides workspace configuration to the language server.
    ///
    /// Forwards the user's `lsp.kotlin-dev-lsp.settings` from Zed's settings
    /// under the `kotlin` configuration key. Defaults to an empty object when
    /// no settings are configured.
    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let mut settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings)
            .unwrap_or_default();

        // `proxy_path` is consumed client-side by the extension; don't forward it.
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("proxy_path");
        }

        Ok(Some(zed::serde_json::json!({
            "kotlin": settings
        })))
    }
}

zed::register_extension!(KotlinExtension);
