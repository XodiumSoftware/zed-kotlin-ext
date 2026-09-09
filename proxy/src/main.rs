//! kotlin-lsp-proxy — a thin stdio proxy in front of JetBrains `kotlin-lsp`.
//!
//! kotlin-lsp returns navigation targets that live inside source archives
//! (`…/foo-sources.jar!/pkg/Foo.kt`, `…/lib/src.zip!/…/Instant.java`). Zed cannot open
//! those archive-internal paths, so "Go to Definition" into any library/JDK class shows
//! an empty buffer.
//!
//! This proxy sits between Zed and kotlin-lsp, forwards every message untouched, and only
//! rewrites the location URIs in definition-family responses: it extracts the referenced
//! entry from the jar/zip to a real temp file and points Zed at that instead.
//!
//! Additionally it hosts a localhost HTTP "sidecar" (see `sidecar.rs`) so the wasm
//! extension can issue out-of-band LSP requests — e.g. `start_debug_server` to launch
//! kotlin-lsp's embedded debug adapter.
//!
//! Usage: `kotlin-lsp-proxy <kotlin-lsp-binary> [args...]`
//!
//! Environment: `KOTLIN_LSP_PORT_FILE` — absolute path where the sidecar writes
//! its listening port (the extension computes it; our cwd is the worktree root,
//! not the extension work directory, so we must not resolve relative paths).

#[macro_use]
mod log;
mod lsp;
mod resolve;
mod sidecar;
#[cfg(unix)]
#[path = "platform/unix.rs"]
mod unix;
#[cfg(windows)]
#[path = "platform/windows.rs"]
mod windows;

use lsp::{LspReader, parse_lsp_content, raw_has_id, write_raw, write_to_stdout};
use resolve::rewrite_archive_locations;
use serde_json::Value;
use sidecar::{PendingSidecar, fail_all, route_reply, spawn as spawn_sidecar};
use std::{
    collections::{HashMap, HashSet},
    env,
    io::{self, BufReader, Write},
    process::{self, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
#[cfg(unix)]
use unix::spawn_parent_monitor;
#[cfg(windows)]
use windows::spawn_parent_monitor;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: kotlin-lsp-proxy <bin> [args...]");
        process::exit(1);
    }

    let bin = &args[0];
    let child_args = &args[1..];

    lsp_info!("kotlin-lsp-proxy starting: bin={bin}");

    let mut child = Command::new(bin)
        .args(child_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("Failed to spawn {bin}: {e}");
            lsp_error!("Failed to spawn {bin}: {e}");
            process::exit(1);
        });

    lsp_info!("kotlin-lsp process spawned (pid={})", child.id());

    let child_stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
    let child_stdout = child.stdout.take().unwrap();
    let alive = Arc::new(AtomicBool::new(true));

    // Request ids whose responses may carry archive (`jar!/`) location URIs.
    let tracked: Arc<Mutex<HashSet<Value>>> = Arc::new(Mutex::new(HashSet::new()));

    // In-flight HTTP-sidecar LSP requests waiting for their response.
    let pending_sidecar: PendingSidecar = Arc::new(Mutex::new(HashMap::new()));

    // --- HTTP sidecar: extension (wasm) -> LSP stdin (injected requests) ---
    spawn_sidecar(Arc::clone(&child_stdin), Arc::clone(&pending_sidecar));

    // --- Thread 1: Zed stdin -> kotlin-lsp stdin (record definition-family ids) ---
    let stdin_writer = Arc::clone(&child_stdin);
    let alive_in = Arc::clone(&alive);
    let tracked_in = Arc::clone(&tracked);
    thread::spawn(move || {
        let stdin = io::stdin().lock();
        let mut reader = LspReader::new(BufReader::new(stdin));
        while alive_in.load(Ordering::Relaxed) {
            match reader.read_message() {
                Ok(Some(raw)) => {
                    // Only requests carry an `id`; skip parsing high-volume notifications.
                    if raw_has_id(&raw)
                        && let Some(msg) = parse_lsp_content(&raw)
                        && is_location_request(&msg)
                        && let Some(id) = msg.get("id").cloned()
                    {
                        tracked_in.lock().unwrap().insert(id);
                    }
                    let mut w = stdin_writer.lock().unwrap();
                    if w.write_all(&raw).is_err() || w.flush().is_err() {
                        break;
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
        alive_in.store(false, Ordering::Relaxed);
    });

    // --- Thread 2: kotlin-lsp stdout -> rewrite archive URIs -> Zed stdout ---
    let alive_out = Arc::clone(&alive);
    let tracked_out = Arc::clone(&tracked);
    let pending_out = Arc::clone(&pending_sidecar);
    thread::spawn(move || {
        let mut reader = LspReader::new(BufReader::new(child_stdout));
        while alive_out.load(Ordering::Relaxed) {
            match reader.read_message() {
                Ok(Some(raw)) => {
                    // Notifications can't be responses we track: forward untouched.
                    if !raw_has_id(&raw) {
                        write_raw(&mut io::stdout().lock(), &raw);
                        continue;
                    }

                    let Some(mut msg) = parse_lsp_content(&raw) else {
                        write_raw(&mut io::stdout().lock(), &raw);
                        continue;
                    };

                    // A response to an injected sidecar request: route to its HTTP
                    // handler; Zed never sees id `$proxy-*` traffic.
                    if route_reply(&msg, &pending_out) {
                        continue;
                    }

                    let is_tracked = msg
                        .get("id")
                        .map(|id| tracked_out.lock().unwrap().remove(id))
                        .unwrap_or(false);

                    if is_tracked && rewrite_archive_locations(&mut msg) {
                        write_to_stdout(&msg);
                    } else {
                        write_raw(&mut io::stdout().lock(), &raw);
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
        // Server died: release any HTTP handlers still waiting on a response.
        fail_all(&pending_out);
        alive_out.store(false, Ordering::Relaxed);
    });

    // --- Thread 3: terminate kotlin-lsp if Zed dies ---
    spawn_parent_monitor(Arc::clone(&alive), child.id());

    let status = child.wait();
    sidecar::cleanup_port_file();
    lsp_info!("kotlin-lsp process exited: {status:?}");
    alive.store(false, Ordering::Relaxed);
}

fn is_location_request(msg: &Value) -> bool {
    matches!(
        msg.get("method").and_then(|m| m.as_str()),
        Some(
            "textDocument/definition"
                | "textDocument/typeDefinition"
                | "textDocument/implementation"
                | "textDocument/declaration"
                | "textDocument/references"
        )
    )
}
