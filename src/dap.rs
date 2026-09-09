//! Debug adapter wiring for the embedded JetBrains debug server in kotlin-lsp.
//!
//! kotlin-lsp (263.x) ships an experimental DAP server (the IntelliJ debugger,
//! exposed via VS Code's `"intellij_debugger"` adapter). It is started over LSP:
//! `workspace/executeCommand` with `start_debug_server` returns the TCP port of
//! a fresh debug adapter session; Zed then speaks DAP to that port directly.
//!
//! A wasm extension cannot issue LSP requests itself, so this module goes
//! through the proxy's HTTP sidecar (`proxy/src/sidecar.rs`): it binds
//! 127.0.0.1:<ephemeral>, publishes its port to `proxy/<hex(worktree)>` inside
//! the extension work directory, and accepts POST `{"method", "params"}` bodies
//! answering with `{"result"|"error": ...}`. This is the same mechanism the
//! Zed Java extension uses for jdt.ls (`vscode.java.startDebugSession`).
//!
//! Verified against kotlin-lsp 263.4421.0 (see `.spike/` probes):
//! - attach: `{ "request": "attach", "hostName", "port" }` — breakpoints hit,
//!   stacks/scopes/variables work.
//! - launch: `{ "request": "launch", "javaExec", "mainClass", "classPaths" }` —
//!   the adapter spawns the debuggee JVM itself.
//! - breakpoint resolution only works after project import has finished;
//!   starting the debug server itself does not require it.

use std::fs;
use std::path::Path;

use zed::http_client::{HttpMethod, HttpRequest, fetch};
use zed::serde_json::{self, Value, json};
use zed_extension_api as zed;

/// The debug adapter name, as registered in `extension.toml` under
/// `[debug_adapters]` (and used as `"adapter"` value in `debug.json`).
pub const DEBUG_ADAPTER_NAME: &str = "Kotlin";

/// The LSP command that starts kotlin-lsp's embedded DAP server. Takes no
/// arguments and returns the listening TCP port (a bare integer).
const START_DEBUG_SERVER_COMMAND: &str = "start_debug_server";

/// Issues an LSP request to the running kotlin-lsp through the proxy's HTTP
/// sidecar and returns the plain `result` value.
///
/// # Errors
///
/// Returns an error when the sidecar port file is missing/unreadable (proxy not
/// running, e.g. the extension fell back to launching kotlin-lsp directly), the
/// HTTP call fails, or the server answers with an LSP error.
pub fn lsp_request(worktree_root: &str, method: &str, params: Value) -> Result<Value, String> {
    let port = read_sidecar_port(worktree_root)?;
    let body = json!({"method": method, "params": params}).to_string();

    let resp = fetch(
        &HttpRequest::builder()
            .method(HttpMethod::Post)
            .url(format!("http://127.0.0.1:{port}"))
            .body(body)
            .build()?,
    )
    .map_err(|err| format!("LSP sidecar request {method} failed: {err}"))?;

    let value: Value = serde_json::from_slice(&resp.body)
        .map_err(|err| format!("LSP sidecar returned malformed JSON: {err}"))?;

    if let Some(error) = value.get("error") {
        let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32000);
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(format!("LSP request {method} failed: {code} {message}"));
    }
    match value.get("result") {
        Some(result) => Ok(result.clone()),
        None => Err(format!(
            "LSP request {method} returned neither result nor error"
        )),
    }
}

/// Starts kotlin-lsp's embedded DAP session and returns its TCP port.
pub fn start_debug_server(worktree_root: &str) -> Result<u16, String> {
    let result = lsp_request(
        worktree_root,
        "workspace/executeCommand",
        json!({"command": START_DEBUG_SERVER_COMMAND, "arguments": []}),
    )?;
    result
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())
        .filter(|port| *port > 0)
        .ok_or_else(|| {
            format!("start_debug_server returned unexpected value: {result} (expected a TCP port)")
        })
}

/// Reads the sidecar's port file. The proxy writes it relative to the
/// extension's work directory (which is our cwd) as `proxy/<hex(worktree)>`.
fn read_sidecar_port(worktree_root: &str) -> Result<u16, String> {
    let path = Path::new("proxy").join(hex_encode(worktree_root));
    let content = fs::read_to_string(&path).map_err(|_| {
        format!(
            "kotlin-lsp sidecar not found at {} — debugging requires the kotlin-lsp-proxy \
             (it is missing or was overridden via proxy_path)",
            path.display()
        )
    })?;
    content.trim().parse::<u16>().map_err(|err| {
        format!(
            "kotlin-lsp sidecar port file at {} is corrupted: {err}",
            path.display()
        )
    })
}

/// Lowercase hex encoding of `s`'s UTF-8 bytes; must match the proxy's
/// `sidecar::hex_encode`.
fn hex_encode(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_matches_proxy_encoding() {
        assert_eq!(hex_encode("C:/proj"), "433a2f70726f6a");
        assert_eq!(hex_encode(""), "");
    }

    #[test]
    fn detects_result_envelope() {
        let value: Value = serde_json::from_slice(br#"{"result": 60096}"#).unwrap();
        assert_eq!(value.get("result").and_then(Value::as_u64), Some(60096));
        assert!(value.get("error").is_none());
    }

    #[test]
    fn detects_error_envelope() {
        let value: Value =
            serde_json::from_slice(br#"{"error": {"code": -32601, "message": "nope"}}"#).unwrap();
        let error = value.get("error").unwrap();
        assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32601));
        assert_eq!(error.get("message").and_then(Value::as_str), Some("nope"));
    }
}
