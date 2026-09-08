//! Kotlin language support for the Zed editor.
//!
//! This extension integrates the official [Kotlin LSP](https://github.com/Kotlin/kotlin-lsp)
//! from JetBrains. The language server binary is downloaded from JetBrains'
//! servers on first use and cached for subsequent sessions. Workspace
//! configuration is forwarded from Zed's `lsp.kotlin-dev-lsp` settings.

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

mod lsp;
mod util;

use lsp::KotlinLSP;

/// The Kotlin extension registered with Zed.
struct KotlinExtension {
    /// Lazily initialized manager for the Kotlin LSP binary.
    kotlin_lsp: Option<KotlinLSP>,
}

impl zed::Extension for KotlinExtension {
    fn new() -> Self {
        Self { kotlin_lsp: None }
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
        _: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        match language_server_id.as_ref() {
            KotlinLSP::LANGUAGE_SERVER_ID => {
                let kotlin_lsp = self.kotlin_lsp.get_or_insert_with(KotlinLSP::new);
                let binary_path = kotlin_lsp.language_server_binary_path(language_server_id)?;
                Ok(zed::Command {
                    command: binary_path,
                    args: vec!["--stdio".to_string()],
                    env: Default::default(),
                })
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
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings)
            .unwrap_or_default();

        Ok(Some(zed::serde_json::json!({
            "kotlin": settings
        })))
    }
}

zed::register_extension!(KotlinExtension);
