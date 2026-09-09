//! Kotlin language support for the Zed editor.
//!
//! This extension integrates the official [Kotlin LSP](https://github.com/Kotlin/kotlin-lsp)
//! from JetBrains. The language server binary is downloaded from JetBrains'
//! servers on first use and cached for subsequent sessions. Workspace
//! configuration is forwarded from Zed's `lsp.kotlin-dev-lsp` settings.
//!
//! Debugging is provided by the LSP's embedded JetBrains DAP server: it is
//! started via the `start_debug_server` LSP command (through the proxy's HTTP
//! sidecar) and then reached over TCP — no separate debug adapter download.

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

mod dap;
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
                            // Keys the sidecar port file per worktree so the debug
                            // adapter wiring (`dap.rs`) can find this instance.
                            env: vec![(dap::WORKTREE_ENV_VAR.to_string(), worktree.root_path())],
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

    /// Determines launch-vs-attach for a Kotlin debug scenario from its
    /// `"request"` field.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown adapter names or a missing/invalid
    /// `"request"` field.
    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: zed::serde_json::Value,
    ) -> Result<zed::StartDebuggingRequestArgumentsRequest, String> {
        if adapter_name != dap::DEBUG_ADAPTER_NAME {
            return Err(format!(
                "Cannot create binary for adapter \"{adapter_name}\""
            ));
        }
        use zed::StartDebuggingRequestArgumentsRequest::{Attach, Launch};
        match config.get("request").and_then(|v| v.as_str()) {
            Some("launch") => Ok(Launch),
            Some("attach") => Ok(Attach),
            Some(other) => Err(format!(
                "Unexpected value for `request` key in Kotlin debug adapter configuration: {other:?}"
            )),
            None => {
                Err("Missing required `request` field in Kotlin debug adapter configuration".into())
            }
        }
    }

    /// Returns how to connect Zed to the kotlin-lsp embedded debug server for a
    /// debug session.
    ///
    /// The server lives inside the already-running kotlin-lsp: nothing is
    /// launched (`command: None`), Zed connects over TCP. Each scenario start
    /// requests a fresh debug session via the proxy sidecar — unless the user
    /// pinned `tcp_connection` themselves, which is honored as-is.
    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: zed::DebugTaskDefinition,
        _user_provided_debug_adapter_path: Option<String>,
        worktree: &zed::Worktree,
    ) -> Result<zed::DebugAdapterBinary, String> {
        if adapter_name != dap::DEBUG_ADAPTER_NAME {
            return Err(format!(
                "Cannot create binary for adapter \"{adapter_name}\""
            ));
        }

        let workspace = worktree.root_path();
        let request = self.dap_request_kind(
            adapter_name,
            zed::serde_json::from_str(&config.config)
                .map_err(|err| format!("Invalid JSON configuration: {err}"))?,
        )?;

        let template = match config.tcp_connection {
            Some(t) => t,
            None => zed::TcpArgumentsTemplate {
                port: Some(dap::start_debug_server(&workspace)?),
                host: None,
                timeout: None,
            },
        };

        Ok(zed::DebugAdapterBinary {
            command: None,
            arguments: vec![],
            cwd: Some(workspace),
            envs: vec![],
            request_args: zed::StartDebuggingRequestArguments {
                request,
                configuration: config.config,
            },
            connection: Some(
                zed::resolve_tcp_template(template)
                    .map_err(|err| format!("Failed to resolve debug server connection: {err}"))?,
            ),
        })
    }

    /// Maps the generic "new debug session" UI inputs onto a Kotlin scenario.
    ///
    /// Attaching produces a JDWP-host/port scenario (localhost:5005 by
    /// default). Launch is not derivable from the generic UI (a JVM program has
    /// no program binary) — it requires a `debug.json` scenario with an
    /// explicit `mainClass` + `classPaths`.
    fn dap_config_to_scenario(
        &mut self,
        config: zed::DebugConfig,
    ) -> Result<zed::DebugScenario, String> {
        match config.request {
            zed::DebugRequest::Attach(attach) => {
                if attach.process_id.is_some() {
                    return Err(
                        "Kotlin attach needs a JDWP host/port, not a process id; use a debug.json scenario \
                         with `hostName`/`port` (e.g. after `./gradlew run --debug-jvm`)"
                            .into(),
                    );
                }
                let mut cfg = zed::serde_json::json!({
                    "request": "attach",
                    "hostName": "localhost",
                    "port": 5005,
                });
                if let Some(stop) = config.stop_on_entry {
                    cfg["stopOnEntry"] = zed::serde_json::Value::Bool(stop);
                }
                Ok(zed::DebugScenario {
                    adapter: config.adapter,
                    label: config.label,
                    build: None,
                    tcp_connection: None,
                    config: cfg.to_string(),
                })
            }
            zed::DebugRequest::Launch(_) => Err(
                "Kotlin launch needs a JVM main class and classpath; use a debug.json scenario with \
                 `request: \"launch\"`, `mainClass` and `classPaths` (see the extension README)"
                    .into(),
            ),
        }
    }
}

zed::register_extension!(KotlinExtension);
