//! Management of the `kotlin-lsp-proxy` companion binary.
//!
//! The proxy resolves archive-internal (`jar!/`, `zip!/`) source URIs that
//! kotlin-lsp returns for library/JDK navigation targets by extracting the
//! referenced entry to a real temp file Zed can open. The prebuilt binary is
//! downloaded from this repository's GitHub Releases on first use and cached
//! afterwards; a user-configured path can bypass the download entirely.

use zed_extension_api::{self as zed, Result, Worktree, serde_json::Value};

use crate::utils;

/// Base name of the proxy binary and of its release assets.
const PROXY_BINARY: &str = "kotlin-lsp-proxy";

/// Install dirs are named `kotlin-proxy-<version>`. The distinct `kotlin-proxy`
/// prefix keeps cleanup from clobbering the `kotlin-dev-lsp-*` LSP installs.
const INSTALL_PREFIX: &str = "kotlin-proxy";

/// GitHub repository whose releases host the prebuilt proxy binaries.
const GITHUB_REPO: &str = "XodiumSoftware/zed-kotlin-ext";

/// Downloads and caches the `kotlin-lsp-proxy` binary that resolves
/// archive-internal (`jar!/`, `zip!/`) source URIs returned by kotlin-lsp.
pub struct Proxy {
    /// Path to a previously resolved proxy binary, if any.
    cached_binary_path: Option<String>,
}

impl Proxy {
    /// Creates a new manager with no cached binary path.
    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    /// Returns the path to the proxy binary, downloading and extracting the
    /// release asset first if needed.
    ///
    /// The resolved path is cached for the lifetime of this manager, so later
    /// calls return immediately.
    ///
    /// Honors the `lsp.kotlin-dev-lsp.settings.proxy_path` escape hatch: a
    /// locally-built or manually-placed binary that bypasses the release
    /// download entirely.
    ///
    /// # Errors
    ///
    /// Returns an error if no release asset matches the current platform, or
    /// the download or extraction fails.
    pub fn language_server_binary_path(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        configuration: &Option<Value>,
        _worktree: &Worktree,
    ) -> Result<String> {
        if let Some(path) = self.cached_binary_path.as_ref() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );
            return Ok(path.clone());
        }

        if let Some(path) = user_configured_proxy_path(configuration) {
            self.cached_binary_path = Some(path.clone());
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );
            return Ok(path);
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = match zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) {
            Ok(r) => r,
            Err(e) => {
                let err = format!("Failed to fetch kotlin-lsp-proxy release: {e}");
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(err.clone()),
                );
                return Err(err);
            }
        };

        let (asset_name, file_type) = asset()?;
        let version_dir = format!("{INSTALL_PREFIX}-{}", release.version);
        let binary_path = format!("{version_dir}/{}", proxy_exec());

        let asset = match release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "No kotlin-lsp-proxy asset matching {asset_name:?} in release {}",
                    release.version
                )
            }) {
            Ok(a) => a,
            Err(e) => {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(e.clone()),
                );
                return Err(e);
            }
        };

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        if let Err(e) = utils::download_and_install(
            &asset.download_url,
            &version_dir,
            file_type,
            &binary_path,
            INSTALL_PREFIX,
        ) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Failed(e.clone()),
            );
            return Err(e);
        }

        self.cached_binary_path = Some(binary_path.clone());
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );
        Ok(binary_path)
    }
}

/// Returns the release asset name and archive type for the current platform.
fn asset() -> Result<(String, zed::DownloadedFileType)> {
    let (os, arch) = zed::current_platform();
    let (os_str, file_type) = match os {
        zed::Os::Mac => ("darwin", zed::DownloadedFileType::GzipTar),
        zed::Os::Linux => ("linux", zed::DownloadedFileType::GzipTar),
        zed::Os::Windows => ("windows", zed::DownloadedFileType::Zip),
    };
    let arch_str = match arch {
        zed::Architecture::Aarch64 => "aarch64",
        zed::Architecture::X8664 => "x86_64",
        _ => return Err("Unsupported architecture for kotlin-lsp-proxy".into()),
    };
    let ext = match file_type {
        zed::DownloadedFileType::Zip => "zip",
        _ => "tar.gz",
    };
    Ok((
        format!("{PROXY_BINARY}-{os_str}-{arch_str}.{ext}"),
        file_type,
    ))
}

/// Returns the proxy executable file name for the current platform.
fn proxy_exec() -> String {
    match zed::current_platform().0 {
        zed::Os::Windows => format!("{PROXY_BINARY}.exe"),
        _ => PROXY_BINARY.to_string(),
    }
}

/// Reads the `proxy_path` escape hatch from the language server settings, if set.
fn user_configured_proxy_path(configuration: &Option<Value>) -> Option<String> {
    let path = configuration
        .as_ref()?
        .pointer("/proxy_path")
        .and_then(|v| v.as_str())?;
    Some(path.to_string())
}
