use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

mod kotlin_lsp;
mod util;

use kotlin_lsp::KotlinLSP;

struct KotlinExtension {
    kotlin_lsp: Option<KotlinLSP>,
}

impl zed::Extension for KotlinExtension {
    fn new() -> Self {
        Self { kotlin_lsp: None }
    }

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

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();

        Ok(Some(zed::serde_json::json!({
            "kotlin": settings
        })))
    }
}

zed::register_extension!(KotlinExtension);
