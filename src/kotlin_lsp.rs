use std::fs;

use zed_extension_api::{self as zed, make_file_executable, Result};

use crate::util;

pub struct KotlinLSP {
    cached_binary_path: Option<String>,
}

impl KotlinLSP {
    pub const LANGUAGE_SERVER_ID: &'static str = "kotlin-dev-lsp";

    pub fn new() -> Self {
        KotlinLSP {
            cached_binary_path: None,
        }
    }

    pub fn language_server_binary_path(
        &mut self,
        language_server_id: &zed::LanguageServerId,
    ) -> Result<String> {
        if let Some(path) = self.cached_binary_path.as_ref() {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = get_version()?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        let binary_path = download_from_teamcity(version)?;

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

/// Return URL to the kotlin-lsp package on TeamCity servers
fn get_version() -> Result<String> {
    // JetBrains' RELEASES.md is stale (still points to expired 262.x builds).
    // Hardcode the latest known working build until they update it.
    // See: https://github.com/Kotlin/kotlin-lsp/issues/271
    Ok("263.4421.0".to_string())
}

fn download_from_teamcity(version: String) -> Result<String> {
    let (os, arch) = zed_extension_api::current_platform();

    // WIN https://download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/kotlin-server-262.9593.0.win.zip
    // WIN ARM https://download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/kotlin-server-262.9593.0-aarch64.win.zip
    // LINUX https://download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/kotlin-server-262.9593.0.tar.gz
    // LINUX ARM https://download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/kotlin-server-262.9593.0-aarch64.tar.gz
    // MAC https://download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/kotlin-server-262.9593.0.sit
    // MAC ARM https://download-cdn.jetbrains.com/language-server/kotlin-server/262.9593.0/kotlin-server-262.9593.0-aarch64.sit

    let arch_suffix = match arch {
        zed::Architecture::X8664 => "",
        zed::Architecture::Aarch64 => "-aarch64",
        _ => {
            return Err("Platform X86 is not supported by the Kotlin language server.".to_string())
        }
    };

    let asset_name = match os {
        zed::Os::Windows => format!("kotlin-server-{version}{arch_suffix}.win.zip"),
        zed::Os::Mac => format!("kotlin-server-{version}{arch_suffix}.sit"),
        zed::Os::Linux => format!("kotlin-server-{version}{arch_suffix}.tar.gz"),
    };

    // JetBrains moved to download.jetbrains.com for newer builds (263.x+)
    let url = format!(
        "https://download.jetbrains.com/language-server/kotlin-server/{version}/{asset_name}"
    );

    let extension_dir = format!(
        "{server_id}-{version}",
        server_id = KotlinLSP::LANGUAGE_SERVER_ID
    );

    let target_dir = match os {
        zed::Os::Windows => extension_dir.clone(),
        _ => format!("{extension_dir}/kotlin-server-{version}"),
    };

    let binary_path = format!(
        "{target_dir}/bin/intellij-server{exe_suffix}",
        exe_suffix = match os {
            zed::Os::Windows => ".exe",
            _ => "",
        }
    );

    if !fs::metadata(&extension_dir).is_ok_and(|metadata| metadata.is_dir()) {
        let downloaded_file_type = match os {
            // We don't ask questions as to why `sit` == `zip`. Let JetBrains keep their secrets there
            zed::Os::Windows | zed::Os::Mac => zed_extension_api::DownloadedFileType::Zip,
            zed::Os::Linux => zed_extension_api::DownloadedFileType::GzipTar,
        };

        zed::download_file(&url, &extension_dir, downloaded_file_type)?;
        make_file_executable(&binary_path)?;
        util::remove_outdated_versions(KotlinLSP::LANGUAGE_SERVER_ID, &extension_dir)?;
    }

    Ok(binary_path)
}
