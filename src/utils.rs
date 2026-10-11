//! Internal helpers shared across the extension.

use std::fs;

use zed_extension_api::Result;

/// Downloads and extracts an archive to `version_dir` if `binary_path` is missing,
/// then marks it executable and removes outdated versions.
///
/// If `binary_path` already exists, this is a no-op. Otherwise, `url` is
/// downloaded (as `file_type`) into `version_dir`, `binary_path` is marked
/// executable, and stale directories matching `install_prefix` are removed.
///
/// # Errors
///
/// Returns an error if the download fails, or if the binary still does not
/// exist after extraction.
pub(super) fn download_and_install(
    url: &str,
    version_dir: &str,
    file_type: zed_extension_api::DownloadedFileType,
    binary_path: &str,
    install_prefix: &'static str,
) -> Result<()> {
    if fs::metadata(binary_path).is_ok_and(|m| m.is_file()) {
        return Ok(());
    }

    zed_extension_api::download_file(url, version_dir, file_type)?;
    zed_extension_api::make_file_executable(binary_path)?;
    remove_outdated_versions(install_prefix, version_dir)?;

    if !fs::metadata(binary_path).is_ok_and(|m| m.is_file()) {
        return Err(format!(
            "Binary not found at {binary_path} after downloading"
        ));
    }

    Ok(())
}

/// Removes previously downloaded language server versions from the working
/// directory.
///
/// Deletes every entry whose file name starts with `language_server_id`,
/// except `version_dir`, which is the version currently in use.
///
/// # Errors
///
/// Returns an error if the working directory cannot be read. Individual
/// directories that fail to delete are skipped silently.
pub(super) fn remove_outdated_versions(
    language_server_id: &'static str,
    version_dir: &str,
) -> Result<()> {
    let entries = fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
        if entry.file_name().to_str().is_none_or(|file_name| {
            file_name.starts_with(language_server_id) && file_name != version_dir
        }) {
            fs::remove_dir_all(entry.path()).ok();
        }
    }
    Ok(())
}
