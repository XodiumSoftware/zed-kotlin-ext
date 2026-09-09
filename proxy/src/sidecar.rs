//! Localhost HTTP sidecar that lets the (wasm) extension send ad-hoc LSP
//! requests to the running kotlin-lsp through this proxy.
//!
//! Zed extensions are wasm sandboxes: they cannot open sockets to the language
//! server, and the LSP connection is the proxy's stdin/stdout streams. To support
//! features that need server commands issued out-of-band — chiefly starting the
//! embedded JetBrains debug server via `workspace/executeCommand` →
//! `start_debug_server` — the proxy hosts a tiny HTTP endpoint on 127.0.0.1
//! (ephemeral port, published in a port file) accepting POST bodies of the form
//! `{"method": "...", "params": ...}`.
//!
//! For each request the sidecar injects a JSON-RPC request into the server's
//! stdin using a prefixed string id (`$proxy-<n>`) that cannot collide with the
//! client's numeric ids, blocks until the stdout-forwarding thread routes the
//! matching response back over an mpsc channel (the response is withheld from
//! Zed), and returns its `result`/`error` as the HTTP response body — the exact
//! shape the `zed-extensions/java` wasm side already parses.
//!
//! The sidecar degrades gracefully: if the port cannot be bound or the
//! worktree is unknown, the proxy simply runs without it (debugging unavailable,
//! everything else unaffected).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::ChildStdin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use std::{env, fs, thread};

use serde_json::{Value, json};

use crate::lsp::encode_lsp;

/// Tracks in-flight sidecar requests by their injected id; the stdout-forwarding
/// thread routes matching responses onto the stored channel.
pub type PendingSidecar = Arc<Mutex<HashMap<Value, mpsc::Sender<Value>>>>;

/// How long an HTTP handler waits for the server to answer before giving up.
const SIDECAR_TIMEOUT: Duration = Duration::from_secs(90);

/// Id prefix for injected requests. Zed's LSP client uses numeric ids, so a
/// string id can never collide with client traffic on the same connection.
const ID_PREFIX: &str = "$proxy-";

/// Env var carrying the worktree root path, set by the extension. Keys the port
/// file so several windows (each with their own proxy+LSP) don't clobber each
/// other's file.
const WORKTREE_ENV: &str = "KOTLIN_LSP_WORKTREE_ROOT";

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Starts the sidecar on a dedicated thread. Never fails the proxy itself: any
/// error is logged and the sidecar simply stays unavailable.
pub fn spawn(child_stdin: Arc<Mutex<ChildStdin>>, pending: PendingSidecar) {
    thread::spawn(move || run(child_stdin, pending));
}

fn run(child_stdin: Arc<Mutex<ChildStdin>>, pending: PendingSidecar) {
    let listener = match TcpListener::bind(("127.0.0.1", 0)) {
        Ok(l) => l,
        Err(err) => {
            crate::log::warn(&format!("LSP sidecar unavailable (bind failed: {err})"));
            return;
        }
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);

    match env::var(WORKTREE_ENV) {
        Ok(worktree) => write_port_file(&worktree, port),
        Err(_) => crate::log::warn(&format!(
            "LSP sidecar: {WORKTREE_ENV} not set; skipping port file (debugging needs the latest proxy+extension pair)"
        )),
    }
    crate::log::info(&format!("LSP sidecar listening on 127.0.0.1:{port}"));

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let stdin = Arc::clone(&child_stdin);
        let pending = Arc::clone(&pending);
        thread::spawn(move || handle_connection(stream, stdin, pending));
    }
}

/// Writes the listening port to `<cwd>/proxy/<hex(worktree)>`, digits only, no
/// trailing newline (parsed with a plain `.parse::<u16>()` on the wasm side).
fn write_port_file(worktree: &str, port: u16) {
    let path = port_file_path(worktree);
    if let Some(parent) = path.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        crate::log::warn(&format!(
            "LSP sidecar: failed to create port file dir {}: {err}",
            parent.display()
        ));
        return;
    }
    if let Err(err) = fs::write(&path, port.to_string()) {
        crate::log::warn(&format!(
            "LSP sidecar: failed to write port file {}: {err}",
            path.display()
        ));
    }
}

/// Removes this instance's port file on shutdown so stale ports can't mislead
/// the extension after the proxy exits (e.g. on language-server restart).
pub fn cleanup_port_file() {
    if let Ok(worktree) = env::var(WORKTREE_ENV) {
        let _ = fs::remove_file(port_file_path(&worktree));
    }
}

fn port_file_path(worktree: &str) -> PathBuf {
    PathBuf::from("proxy").join(hex_encode(worktree))
}

/// If `msg` is the response to an injected sidecar request, routes it onto the
/// waiting channel and returns `true` (caller must not forward it to Zed).
pub fn route_reply(msg: &Value, pending: &PendingSidecar) -> bool {
    let Some(id) = msg.get("id") else {
        return false;
    };
    let Some(tx) = pending.lock().unwrap().remove(id) else {
        return false;
    };
    let _ = tx.send(msg.clone());
    true
}

/// Removes all pending sidecar entries, dropping the senders so waiting HTTP
/// handlers fail fast instead of blocking until their timeout. Called when the
/// server stdout reaches EOF (server died or client went away).
pub fn fail_all(pending: &PendingSidecar) {
    pending.lock().unwrap().clear();
}

fn handle_connection(
    mut stream: TcpStream,
    child_stdin: Arc<Mutex<ChildStdin>>,
    pending: PendingSidecar,
) {
    let (status, body) = match read_http_request(&mut stream) {
        Some((request_line, body)) if request_line.starts_with("POST") => {
            handle_lsp_request(&body, &child_stdin, &pending)
        }
        Some(_) => (
            405,
            json!({"error": {"code": -32601, "message": "POST only"}}),
        ),
        None => (
            400,
            json!({"error": {"code": -32600, "message": "malformed HTTP request"}}),
        ),
    };
    let payload = body.to_string();
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    let _ = stream.flush();
}

/// Executes one `{method, params}` LSP round-trip and maps it to an HTTP
/// status + JSON body in the `{"result"|"error": ...}` envelope.
fn handle_lsp_request(
    body: &[u8],
    child_stdin: &Arc<Mutex<ChildStdin>>,
    pending: &PendingSidecar,
) -> (u16, Value) {
    let Ok(req) = serde_json::from_slice::<Value>(body) else {
        return (
            400,
            json!({"error": {"code": -32700, "message": "malformed JSON body"}}),
        );
    };
    let Some(method) = req.get("method").and_then(Value::as_str) else {
        return (
            400,
            json!({"error": {"code": -32600, "message": "missing string \"method\" field"}}),
        );
    };
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    match roundtrip(method, params, child_stdin, pending) {
        Ok(resp) => envelope_for(&resp),
        Err(err) => (504, json!({"error": {"code": -32000, "message": err}})),
    }
}

/// Maps an LSP response to the HTTP `{"result"|"error": ...}` envelope the wasm
/// side (adapted from `zed-extensions/java`) parses. LSP errors already have the
/// `{code, message, data}` shape expected there, so they pass through untouched.
fn envelope_for(resp: &Value) -> (u16, Value) {
    if let Some(err) = resp.get("error") {
        (200, json!({"error": err.clone()}))
    } else {
        (
            200,
            json!({"result": resp.get("result").cloned().unwrap_or(Value::Null)}),
        )
    }
}

/// Injects a request into the server's stdin and waits for its response.
fn roundtrip(
    method: &str,
    params: Value,
    child_stdin: &Arc<Mutex<ChildStdin>>,
    pending: &PendingSidecar,
) -> Result<Value, String> {
    let id = Value::String(format!(
        "{ID_PREFIX}{}",
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let (tx, rx) = mpsc::channel();
    pending.lock().unwrap().insert(id.clone(), tx);

    let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
    let framed = encode_lsp(&msg);
    let write_result = {
        let mut w = child_stdin.lock().unwrap();
        w.write_all(framed.as_bytes()).and_then(|_| w.flush())
    };
    if let Err(err) = write_result {
        pending.lock().unwrap().remove(&id);
        return Err(format!("failed to write request to LSP stdin: {err}"));
    }

    match rx.recv_timeout(SIDECAR_TIMEOUT) {
        Ok(resp) => Ok(resp),
        Err(_) => {
            pending.lock().unwrap().remove(&id);
            Err(format!("timeout waiting for LSP response to {method}"))
        }
    }
}

/// Minimal HTTP/1.1 request reader: request line, headers up to `\r\n\r\n`,
/// then exactly Content-Length body bytes. Returns (request line, body).
fn read_http_request(r: &mut impl Read) -> Option<(String, Vec<u8>)> {
    let mut header = Vec::new();
    let mut byte = [0u8; 1];
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() > 16 * 1024 {
            return None;
        }
        match r.read(&mut byte) {
            Ok(0) => return None,
            Ok(_) => header.push(byte[0]),
            Err(_) => return None,
        }
    }
    let header_text = String::from_utf8_lossy(&header);
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next()?.to_string();
    let content_length = lines
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim().eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .next()
        .unwrap_or(0);
    let mut body = vec![0u8; content_length];
    r.read_exact(&mut body).ok()?;
    Some((request_line, body))
}

/// Plain lowercase hex encoding of `s`'s UTF-8 bytes; the wasm side re-implements
/// this one-liner to locate the port file.
fn hex_encode(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn hex_encodes_worktree_path() {
        assert_eq!(hex_encode("C:/proj"), "433a2f70726f6a");
        assert_eq!(hex_encode(""), "");
    }

    #[test]
    fn parses_minimal_post() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 26\r\nHost: x\r\n\r\n{\"method\":\"workspace/foo\"}";
        let (line, body) = read_http_request(&mut Cursor::new(raw.as_slice())).unwrap();
        assert_eq!(line, "POST / HTTP/1.1");
        assert_eq!(body, b"{\"method\":\"workspace/foo\"}");
    }

    #[test]
    fn content_length_is_case_insensitive() {
        let raw =
            b"POST / HTTP/1.1\r\ncontent-type: application/json\r\ncontent-length: 2\r\n\r\n{}";
        let (_, body) = read_http_request(&mut Cursor::new(raw.as_slice())).unwrap();
        assert_eq!(body, b"{}");
    }

    #[test]
    fn rejects_truncated_body() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 50\r\n\r\n{}";
        assert!(read_http_request(&mut Cursor::new(raw.as_slice())).is_none());
    }

    #[test]
    fn routes_only_pending_reply() {
        let pending: PendingSidecar = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel();
        pending.lock().unwrap().insert(json!("$proxy-7"), tx);

        let other = json!({"jsonrpc": "2.0", "id": 3, "result": null});
        assert!(!route_reply(&other, &pending));
        assert_eq!(pending.lock().unwrap().len(), 1);

        let mine = json!({"jsonrpc": "2.0", "id": "$proxy-7", "result": 60096});
        assert!(route_reply(&mine, &pending));
        assert!(pending.lock().unwrap().is_empty());
        assert_eq!(rx.recv().unwrap()["result"], json!(60096));
    }

    #[test]
    fn fail_all_releases_waiters() {
        let pending: PendingSidecar = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel::<Value>();
        pending.lock().unwrap().insert(json!("$proxy-1"), tx);
        fail_all(&pending);
        assert!(matches!(
            rx.recv_timeout(Duration::from_millis(250)),
            Err(_)
        ));
    }

    #[test]
    fn envelope_maps_result_and_error() {
        let (status, body) =
            envelope_for(&json!({"jsonrpc": "2.0", "id": "$proxy-0", "result": 60096}));
        assert_eq!(status, 200);
        assert_eq!(body, json!({"result": 60096}));

        let (status, body) = envelope_for(
            &json!({"jsonrpc": "2.0", "id": "$proxy-0", "error": {"code": -32601, "message": "no such method"}}),
        );
        assert_eq!(status, 200);
        assert_eq!(
            body,
            json!({"error": {"code": -32601, "message": "no such method"}})
        );

        // Missing result -> explicit null, so callers never parse an empty object.
        let (_, body) = envelope_for(&json!({"jsonrpc": "2.0", "id": "$proxy-0"}));
        assert_eq!(body, json!({"result": Value::Null}));
    }
}
