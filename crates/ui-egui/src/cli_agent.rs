//! Local CLI AI agent driver for PhotoCraft's built-in multi-AI sidebar.
//!
//! Executes locally installed, account-authenticated CLI agents without requiring
//! API credits:
//! - Antigravity CLI (`agy`) — Google account sign-in
//! - Gemini CLI (`gemini`) — Google account sign-in
//! - Claude CLI (`claude`) — Claude subscription sign-in
//! - Codex CLI (`codex`) — ChatGPT subscription sign-in
//!
//! Process discovery and spawning are enabled on native targets only
//! (`#[cfg(not(target_arch = "wasm32"))]`); WebAssembly returns
//! [`AiBackendError::UnsupportedPlatform`]. Pure command builders, stream-JSON
//! parsers, error classifiers, and prompt formatters remain available on all
//! targets for testing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::ai_backend::{AiBackendError, AiProviderKind, AiRequestConfig, AiWorkerHandle, WakeCallback};
use crate::state::{ChatImageAttachment, ChatMessage, ChatRole};

/// Token budget reserved for multi-turn history when passing context via a CLI prompt.
pub const MAX_CLI_CONTEXT_TOKENS: usize = 16_000;

/// Maximum single-line prompt byte length passed directly as a command-line argument
/// before switching to standard input.
const MAX_INLINE_PROMPT_BYTES: usize = 1_500;

/// Maximum bytes retained from child process `stderr` to prevent unbounded memory growth.
#[cfg(not(target_arch = "wasm32"))]
const MAX_STDERR_BYTES: usize = 64 * 1024;

/// Maximum length (in Unicode scalar values) of the concise error summary shown in the UI.
const MAX_ERROR_SUMMARY_CHARS: usize = 320;

/// Windows process creation flag preventing a visible console window flash (`CREATE_NO_WINDOW`).
#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Polling cadence while waiting on a CLI child process so Stop reacts quickly.
#[cfg(not(target_arch = "wasm32"))]
const CANCEL_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

/// Timeout for `--version` connectivity checks.
#[cfg(not(target_arch = "wasm32"))]
const VERSION_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Detection result for a local CLI binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliDetectionStatus {
    /// Resolved executable path on the local system.
    Found(PathBuf),
    /// Binary was not found in `PATH` or common installation directories.
    NotFound,
    /// Local CLI execution is unavailable on this platform (e.g. WebAssembly).
    UnsupportedPlatform,
}

impl CliDetectionStatus {
    /// Returns the resolved path when detected.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Found(p) => Some(p.as_path()),
            Self::NotFound | Self::UnsupportedPlatform => None,
        }
    }

    /// Short display filename (e.g. `gemini.cmd` or `claude`) when detected.
    pub fn display_file_name(&self) -> Option<String> {
        self.path()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
    }
}

/// Prepared subprocess invocation produced by the provider adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliInvocation {
    /// Program passed to `std::process::Command::new`.
    pub program: PathBuf,
    /// Arguments passed to `program`. When `uses_windows_cmd_wrapper` is `true`,
    /// the last entry is the formatted command line for `cmd.exe /D /S /C`.
    pub args: Vec<String>,
    /// Optional prompt payload piped into child `stdin` (then closed with EOF).
    pub stdin_payload: Option<String>,
    /// Optional isolated working directory (used for temporary workspace MCP config files).
    pub working_dir: Option<PathBuf>,
    /// Whether `program` is `cmd.exe` wrapping a Windows `.cmd` / `.bat` script.
    pub uses_windows_cmd_wrapper: bool,
}

/// Options for mounting PhotoCraft's temporary MCP server into a CLI subprocess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliMcpOptions {
    /// Resolved path to the local `photocraft-cli` binary.
    pub photocraft_cli_path: PathBuf,
    /// Whether the user explicitly opted into headless tool auto-approval.
    pub auto_approve_tools: bool,
    /// Extra environment variables placed on the `photocraft` MCP server entry.
    pub server_env: Vec<(String, String)>,
}

/// Materialized temporary MCP workspace metadata prepared for a single CLI turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedMcpMount {
    /// Temporary workspace directory passed as child process `current_dir`.
    pub workspace_dir: PathBuf,
    /// Provider-specific temporary MCP configuration file path, when a file is written.
    pub config_file: Option<PathBuf>,
    /// Resolved path to the local `photocraft-cli` binary.
    pub photocraft_cli_path: PathBuf,
    /// Extra environment variables placed on the `photocraft` MCP server entry.
    pub server_env: Vec<(String, String)>,
    /// Whether headless tool auto-approval is enabled for this turn.
    pub auto_approve_tools: bool,
}

/// Metadata for a canvas image attachment written to a temporary file for CLI inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentNote {
    pub message_index: usize,
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub file_path: PathBuf,
}

/// Execution state of a CLI tool step rendered in sidebar live MCP tool cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliToolState {
    Active,
    Done,
    Failed,
    Unknown,
}

/// Structured tool invocation / completion update parsed from a CLI stream.
#[derive(Clone, Debug, PartialEq)]
pub struct CliToolUpdate {
    /// Provider-assigned invocation identifier (e.g. Claude `tool_use.id`, Gemini `tool_id`, Codex `item.id`).
    pub call_id: Option<String>,
    /// Provider-assigned step index (e.g. Antigravity `step_update.step_index`).
    pub step_index: Option<u64>,
    /// Tool name (e.g. `doc_inspect`, `mcp__photocraft__command_run`).
    pub tool_name: String,
    /// Current execution state of the tool call.
    pub state: CliToolState,
    /// Tool arguments / parameters payload (`Value::Null` if not present on this event).
    pub parameters: Value,
    /// Tool output / result payload once available.
    pub output: Option<Value>,
    /// Error summary if the tool invocation failed.
    pub error: Option<String>,
    /// Raw provider metadata object (e.g. Antigravity `tool_info`), preserved for diagnostics.
    pub tool_info: Value,
}

impl CliToolUpdate {
    /// Returns a stable correlation key identifying this tool invocation within a turn.
    pub fn correlation_key(&self) -> Option<String> {
        if let Some(id) = self.call_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            return Some(format!("call:{id}"));
        }
        self.step_index.map(|idx| format!("step:{idx}"))
    }
}

/// Result of parsing a single stdout line from a CLI streaming session.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedStreamLine {
    /// Incremental assistant text to append to the chat stream.
    pub delta_text: Option<String>,
    /// External CLI session / conversation / thread identifier, if announced on this line.
    pub session_id: Option<String>,
    /// Structured tool invocation state updates announced on this line.
    pub tool_updates: Vec<CliToolUpdate>,
    /// Fatal stream error reported by the CLI, if any.
    pub error: Option<String>,
}

impl ParsedStreamLine {
    /// Convenience accessor for the first structured tool update on this line, if any.
    pub fn first_tool_update(&self) -> Option<&CliToolUpdate> {
        self.tool_updates.first()
    }
}

/// Returns whether `path` refers to a Windows batch wrapper (`.cmd` or `.bat`).
pub fn is_windows_batch_script(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"))
}

/// Returns candidate filenames to check inside a search directory for `base_name`.
///
/// On Windows, `.exe` / `.cmd` / `.bat` / `.com` are checked in priority order so
/// npm's executable `foo.cmd` shim is selected instead of its extensionless POSIX
/// `foo` shell script.
pub fn candidate_executable_names(base_name: &str, is_windows: bool) -> Vec<String> {
    if !is_windows {
        return vec![base_name.to_string()];
    }

    let lower = base_name.to_ascii_lowercase();
    if lower.ends_with(".exe")
        || lower.ends_with(".cmd")
        || lower.ends_with(".bat")
        || lower.ends_with(".com")
    {
        return vec![base_name.to_string()];
    }

    vec![
        format!("{base_name}.exe"),
        format!("{base_name}.cmd"),
        format!("{base_name}.bat"),
        format!("{base_name}.com"),
    ]
}

/// Searches `dirs` in order for the first matching CLI binary.
pub fn resolve_cli_in_dirs(base_name: &str, dirs: &[PathBuf], is_windows: bool) -> Option<PathBuf> {
    if base_name.trim().is_empty() {
        return None;
    }
    let candidates = candidate_executable_names(base_name, is_windows);
    for dir in dirs {
        for name in &candidates {
            let path = dir.join(name);
            if is_usable_executable(&path, is_windows) {
                return Some(path);
            }
        }
    }
    None
}

fn is_usable_executable(path: &Path, is_windows: bool) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    {
        if !is_windows {
            use std::os::unix::fs::PermissionsExt;
            return std::fs::metadata(path)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);
        }
    }
    let _ = is_windows;
    true
}

/// Expands a leading `~/` or `~\` against `HOME` / `USERPROFILE` when present.
#[cfg(not(target_arch = "wasm32"))]
fn expand_home_prefix(raw: &str) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\"))
        && let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(raw)
}

#[cfg(target_arch = "wasm32")]
fn expand_home_prefix(raw: &str) -> PathBuf {
    PathBuf::from(raw)
}

/// Resolves a user-supplied custom executable path override (either a file path or a directory
/// containing `base_name`). Returns `None` if `override_path` is blank or does not point to a
/// usable executable.
pub fn resolve_custom_executable(override_path: &str, base_name: &str, is_windows: bool) -> Option<PathBuf> {
    let trimmed = override_path.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }

    let candidate = expand_home_prefix(trimmed);
    if is_usable_executable(&candidate, is_windows) {
        return Some(candidate);
    }

    if candidate.is_dir() {
        return resolve_cli_in_dirs(base_name, &[candidate], is_windows);
    }

    // On Windows, allow omitting the extension (e.g. `C:\tools\agy` -> `agy.exe` / `agy.cmd`).
    if is_windows && candidate.extension().is_none() {
        for ext in ["exe", "cmd", "bat", "com"] {
            let with_ext = candidate.with_extension(ext);
            if is_usable_executable(&with_ext, true) {
                return Some(with_ext);
            }
        }
    }

    None
}

/// Resolves a CLI binary giving priority to an explicit non-empty `custom_override`.
/// When `custom_override` is empty/whitespace, falls back to auto-detection in `search_dirs`.
pub fn resolve_cli_with_override(
    base_name: &str,
    custom_override: Option<&str>,
    search_dirs: &[PathBuf],
    is_windows: bool,
) -> Option<PathBuf> {
    if let Some(override_str) = custom_override.map(str::trim).filter(|s| !s.is_empty()) {
        return resolve_custom_executable(override_str, base_name, is_windows);
    }
    resolve_cli_in_dirs(base_name, search_dirs, is_windows)
}

/// Detects whether the local CLI binary for `provider` is installed on the host,
/// respecting an optional user-configured custom executable path override first.
pub fn detect_cli_with_override(provider: AiProviderKind, custom_override: Option<&str>) -> CliDetectionStatus {
    let Some(base_name) = provider.cli_executable() else {
        return CliDetectionStatus::NotFound;
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let dirs = collect_cli_search_dirs();
        match resolve_cli_with_override(base_name, custom_override, &dirs, cfg!(target_os = "windows")) {
            Some(path) => CliDetectionStatus::Found(path),
            None => CliDetectionStatus::NotFound,
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (base_name, custom_override);
        CliDetectionStatus::UnsupportedPlatform
    }
}

/// Detects whether the local CLI binary for `provider` is installed on the host.
pub fn detect_cli(provider: AiProviderKind) -> CliDetectionStatus {
    detect_cli_with_override(provider, None)
}

/// Pure resolver locating `photocraft-cli`: checks `current_exe`'s sibling directory
/// first (matching `dist/release` or `target/debug`), then falls back to `search_dirs` (`PATH`).
pub fn resolve_photocraft_cli(
    current_exe: Option<&Path>,
    search_dirs: &[PathBuf],
    is_windows: bool,
) -> Option<PathBuf> {
    resolve_photocraft_cli_with_override(None, current_exe, search_dirs, is_windows)
}

/// Pure resolver locating `photocraft-cli`, preferring an explicit non-empty `custom_override`
/// before falling back to sibling directory and `search_dirs` (`PATH`).
pub fn resolve_photocraft_cli_with_override(
    custom_override: Option<&str>,
    current_exe: Option<&Path>,
    search_dirs: &[PathBuf],
    is_windows: bool,
) -> Option<PathBuf> {
    if let Some(override_str) = custom_override.map(str::trim).filter(|s| !s.is_empty()) {
        return resolve_custom_executable(override_str, "photocraft-cli", is_windows);
    }

    let mut dirs = Vec::with_capacity(search_dirs.len().saturating_add(1));
    let mut push_unique = |dir: PathBuf| {
        if !dir.as_os_str().is_empty() && !dirs.iter().any(|existing| existing == &dir) {
            dirs.push(dir);
        }
    };

    if let Some(parent) = current_exe.and_then(Path::parent) {
        push_unique(parent.to_path_buf());
    }
    for dir in search_dirs {
        push_unique(dir.clone());
    }

    resolve_cli_in_dirs("photocraft-cli", &dirs, is_windows)
}

/// Detects the local `photocraft-cli` binary used to bridge CLI agents to PhotoCraft via MCP,
/// honoring an optional user-configured custom path override first.
pub fn detect_photocraft_cli_with_override(custom_override: Option<&str>) -> CliDetectionStatus {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let current_exe = std::env::current_exe().ok();
        let path_dirs: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|path_var| std::env::split_paths(&path_var).collect())
            .unwrap_or_default();
        match resolve_photocraft_cli_with_override(
            custom_override,
            current_exe.as_deref(),
            &path_dirs,
            cfg!(target_os = "windows"),
        ) {
            Some(path) => CliDetectionStatus::Found(path),
            None => CliDetectionStatus::NotFound,
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = custom_override;
        CliDetectionStatus::UnsupportedPlatform
    }
}

/// Detects the local `photocraft-cli` binary used to bridge CLI agents to PhotoCraft via MCP.
pub fn detect_photocraft_cli() -> CliDetectionStatus {
    detect_photocraft_cli_with_override(None)
}

/// Builds the standardized [`AiBackendError::CliNotFound`] error for `provider`.
pub fn cli_not_found_error(provider: AiProviderKind) -> AiBackendError {
    AiBackendError::CliNotFound {
        provider: provider.display_name(),
        executable: provider.cli_executable().unwrap_or("cli"),
        install_hint: provider.cli_install_hint().unwrap_or("Install the CLI and ensure it is on PATH."),
    }
}

/// Collects `PATH` entries followed by common local installation directories.
#[cfg(not(target_arch = "wasm32"))]
pub fn collect_cli_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut push_unique = |dir: PathBuf| {
        if !dir.as_os_str().is_empty() && !dirs.iter().any(|existing| existing == &dir) {
            dirs.push(dir);
        }
    };

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            push_unique(dir);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            let local = PathBuf::from(local_appdata);
            push_unique(local.join("agy").join("bin"));
            push_unique(local.join("pnpm"));
            push_unique(local.join("Yarn").join("bin"));
            push_unique(local.join("Programs").join("claude"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            push_unique(PathBuf::from(appdata).join("npm"));
        }
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let home = PathBuf::from(profile);
            push_unique(home.join(".local").join("bin"));
            push_unique(home.join(".npm-global").join("bin"));
            push_unique(home.join(".volta").join("bin"));
            push_unique(home.join("scoop").join("shims"));
            push_unique(home.join(".cargo").join("bin"));
        }
        if let Some(prog_files) = std::env::var_os("ProgramFiles") {
            push_unique(PathBuf::from(prog_files).join("nodejs"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            push_unique(home.join(".local").join("bin"));
            push_unique(home.join(".npm-global").join("bin"));
            push_unique(home.join(".volta").join("bin"));
            push_unique(home.join(".asdf").join("shims"));
            push_unique(home.join(".nvm").join("current").join("bin"));
            push_unique(home.join(".bun").join("bin"));
            push_unique(home.join(".cargo").join("bin"));
        }
        push_unique(PathBuf::from("/opt/homebrew/bin"));
        push_unique(PathBuf::from("/usr/local/bin"));
        push_unique(PathBuf::from("/usr/bin"));
    }

    dirs
}

/// Quotes one argument token for a Windows `cmd.exe /D /S /C` command line.
fn quote_windows_cmd_arg(raw: &str) -> String {
    let cleaned = raw.replace(['\r', '\n'], " ");
    let mut out = String::with_capacity(cleaned.len() + 2);
    out.push('"');
    let mut pending_backslashes: usize = 0;
    for ch in cleaned.chars() {
        match ch {
            '\\' => {
                pending_backslashes = pending_backslashes.saturating_add(1);
            }
            '"' => {
                for _ in 0..pending_backslashes.saturating_mul(2).saturating_add(1) {
                    out.push('\\');
                }
                out.push('"');
                pending_backslashes = 0;
            }
            other => {
                for _ in 0..pending_backslashes {
                    out.push('\\');
                }
                pending_backslashes = 0;
                out.push(other);
            }
        }
    }
    for _ in 0..pending_backslashes.saturating_mul(2) {
        out.push('\\');
    }
    out.push('"');
    out
}

/// Formats a batch script path and its arguments into an outer-quoted command string
/// safe to pass via `cmd.exe /D /S /C <cmd_line>`.
pub fn format_windows_cmd_line(program: &Path, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len().saturating_add(1));
    parts.push(quote_windows_cmd_arg(&program.to_string_lossy()));
    for arg in args {
        parts.push(quote_windows_cmd_arg(arg));
    }
    format!("\"{}\"", parts.join(" "))
}

/// Decides whether `prompt` should be sent over child `stdin` rather than argv.
pub fn should_use_stdin_for_prompt(prompt: &str, is_batch_script: bool) -> bool {
    if prompt.contains('\n') || prompt.contains('\r') || prompt.len() > MAX_INLINE_PROMPT_BYTES {
        return true;
    }
    if prompt.trim_start().starts_with('-') {
        return true;
    }
    if is_batch_script
        && prompt
            .bytes()
            .any(|b| matches!(b, b'%' | b'!' | b'&' | b'|' | b'<' | b'>' | b'^' | b'"'))
    {
        return true;
    }
    false
}

/// Wraps a resolved binary and argument list for the target platform, routing
/// Windows `.cmd`/`.bat` scripts through `cmd.exe /D /S /C`.
pub fn wrap_command_for_platform(
    binary_path: &Path,
    raw_args: Vec<String>,
    stdin_payload: Option<String>,
    is_windows: bool,
) -> CliInvocation {
    if is_windows && is_windows_batch_script(binary_path) {
        let cmd_line = format_windows_cmd_line(binary_path, &raw_args);
        CliInvocation {
            program: PathBuf::from("cmd.exe"),
            args: vec!["/D".to_string(), "/S".to_string(), "/C".to_string(), cmd_line],
            stdin_payload,
            working_dir: None,
            uses_windows_cmd_wrapper: true,
        }
    } else {
        CliInvocation {
            program: binary_path.to_path_buf(),
            args: raw_args,
            stdin_payload,
            working_dir: None,
            uses_windows_cmd_wrapper: false,
        }
    }
}

/// Computes the environment variables attached to the temporary `photocraft` MCP server
/// entry so `photocraft-cli mcp` routes calls to the listening desktop window while
/// preserving any unrelated flags already present in `CRAFT_FLAGS`.
pub fn build_photocraft_mcp_env(existing_craft_flags: Option<&str>) -> Vec<(String, String)> {
    let mut parts: Vec<String> = existing_craft_flags
        .unwrap_or_default()
        .split([',', ';', ' '])
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .filter(|token| {
            let id = token
                .strip_prefix('-')
                .unwrap_or(token)
                .split_once('=')
                .map_or(*token, |(k, _)| k);
            id != "AI-MCP"
        })
        .map(str::to_string)
        .collect();
    parts.push("AI-MCP".to_string());
    vec![("CRAFT_FLAGS".to_string(), parts.join(","))]
}

/// Formats `raw` as a TOML basic string (`"..."`) suitable for Codex CLI `-c` overrides.
pub fn format_toml_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().saturating_add(2));
    out.push('"');
    for ch in raw.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04X}", u32::from(c)));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Builds the standard `mcpServers.photocraft` JSON document written into isolated
/// temporary workspace configurations (never touching user global CLI config files).
pub fn build_photocraft_mcp_server_json(photocraft_cli_path: &Path, server_env: &[(String, String)]) -> Value {
    let mut server = serde_json::Map::new();
    server.insert("command".to_string(), Value::String(photocraft_cli_path.to_string_lossy().into_owned()));
    server.insert("args".to_string(), Value::Array(vec![Value::String("mcp".to_string())]));
    if !server_env.is_empty() {
        let mut env_obj = serde_json::Map::new();
        for (k, v) in server_env {
            env_obj.insert(k.clone(), Value::String(v.clone()));
        }
        server.insert("env".to_string(), Value::Object(env_obj));
    }

    let mut servers = serde_json::Map::new();
    servers.insert("photocraft".to_string(), Value::Object(server));

    let mut root = serde_json::Map::new();
    root.insert("mcpServers".to_string(), Value::Object(servers));
    Value::Object(root)
}

/// Writes the provider-appropriate temporary MCP config inside `workspace_dir` without
/// touching any global CLI configuration file on the host.
pub fn write_mcp_workspace_config(
    provider: AiProviderKind,
    workspace_dir: &Path,
    options: &CliMcpOptions,
) -> Result<PreparedMcpMount, String> {
    if !provider.is_cli() {
        return Err("MCP workspace configuration applies to local CLI providers only.".to_string());
    }

    std::fs::create_dir_all(workspace_dir)
        .map_err(|e| format!("Failed to create temporary MCP workspace: {e}"))?;

    let config_file = match provider {
        AiProviderKind::AntigravityCli => {
            let dir = workspace_dir.join(".agents");
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create temporary .agents folder: {e}"))?;
            let path = dir.join("mcp_config.json");
            let json = build_photocraft_mcp_server_json(&options.photocraft_cli_path, &options.server_env);
            let text = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to serialize Antigravity MCP config: {e}"))?;
            std::fs::write(&path, text)
                .map_err(|e| format!("Failed to write temporary Antigravity MCP config: {e}"))?;
            Some(path)
        }
        AiProviderKind::ClaudeCli => {
            let path = workspace_dir.join("mcp-config.json");
            let json = build_photocraft_mcp_server_json(&options.photocraft_cli_path, &options.server_env);
            let text = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to serialize Claude MCP config: {e}"))?;
            std::fs::write(&path, text)
                .map_err(|e| format!("Failed to write temporary Claude MCP config: {e}"))?;
            Some(path)
        }
        AiProviderKind::GeminiCli => {
            let dir = workspace_dir.join(".gemini");
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create temporary .gemini folder: {e}"))?;
            let path = dir.join("settings.json");
            let json = build_photocraft_mcp_server_json(&options.photocraft_cli_path, &options.server_env);
            let text = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to serialize Gemini MCP settings: {e}"))?;
            std::fs::write(&path, text)
                .map_err(|e| format!("Failed to write temporary Gemini MCP settings: {e}"))?;
            Some(path)
        }
        AiProviderKind::CodexCli => {
            // Codex CLI receives temporary MCP overrides via `-c` arguments so ~/.codex/config.toml stays untouched.
            None
        }
        AiProviderKind::GeminiApi => {
            return Err("MCP workspace configuration applies to local CLI providers only.".to_string());
        }
    };

    Ok(PreparedMcpMount {
        workspace_dir: workspace_dir.to_path_buf(),
        config_file,
        photocraft_cli_path: options.photocraft_cli_path.clone(),
        server_env: options.server_env.clone(),
        auto_approve_tools: options.auto_approve_tools,
    })
}

/// Builds the non-interactive streaming invocation for a supported CLI provider without MCP injection.
pub fn build_cli_invocation(
    provider: AiProviderKind,
    binary_path: &Path,
    prompt: &str,
    session_id: Option<&str>,
    is_windows: bool,
) -> Option<CliInvocation> {
    build_cli_invocation_with_mcp(provider, binary_path, prompt, session_id, None, is_windows)
}

/// Builds the non-interactive streaming invocation for a supported CLI provider,
/// optionally mounting an isolated temporary PhotoCraft MCP workspace and headless approval flags.
pub fn build_cli_invocation_with_mcp(
    provider: AiProviderKind,
    binary_path: &Path,
    prompt: &str,
    session_id: Option<&str>,
    mcp: Option<&PreparedMcpMount>,
    is_windows: bool,
) -> Option<CliInvocation> {
    let is_batch = is_windows && is_windows_batch_script(binary_path);
    let use_stdin = should_use_stdin_for_prompt(prompt, is_batch);
    let clean_session = session_id.map(str::trim).filter(|s| !s.is_empty());

    let (raw_args, stdin_payload) = match provider {
        AiProviderKind::AntigravityCli => {
            let safe_prompt = if prompt.starts_with('-') {
                format!(" {prompt}")
            } else {
                prompt.to_string()
            };
            let mut args = vec![
                "-p".to_string(),
                safe_prompt,
                "--output-format".to_string(),
                "stream-json".to_string(),
            ];
            if let Some(id) = clean_session {
                args.push("--conversation".to_string());
                args.push(id.to_string());
            }
            if let Some(mount) = mcp
                && mount.auto_approve_tools
            {
                args.push("--dangerously-skip-permissions".to_string());
            }
            (args, None)
        }
        AiProviderKind::GeminiCli => {
            let (mut args, stdin) = if use_stdin {
                (
                    vec![
                        "-p".to_string(),
                        "Respond to the PhotoCraft assistant prompt provided on standard input.".to_string(),
                        "--output-format".to_string(),
                        "stream-json".to_string(),
                    ],
                    Some(prompt.to_string()),
                )
            } else {
                (
                    vec![
                        "-p".to_string(),
                        prompt.to_string(),
                        "--output-format".to_string(),
                        "stream-json".to_string(),
                    ],
                    None,
                )
            };
            if let Some(mount) = mcp
                && mount.auto_approve_tools
            {
                args.push("--approval-mode".to_string());
                args.push("yolo".to_string());
            }
            (args, stdin)
        }
        AiProviderKind::ClaudeCli => {
            let mut args = if use_stdin {
                vec![
                    "--output-format".to_string(),
                    "stream-json".to_string(),
                    "--verbose".to_string(),
                ]
            } else {
                vec![
                    "-p".to_string(),
                    prompt.to_string(),
                    "--output-format".to_string(),
                    "stream-json".to_string(),
                    "--verbose".to_string(),
                ]
            };
            if let Some(id) = clean_session {
                args.push("--resume".to_string());
                args.push(id.to_string());
            }
            if let Some(mount) = mcp {
                if let Some(ref cfg_path) = mount.config_file {
                    args.push("--mcp-config".to_string());
                    args.push(cfg_path.to_string_lossy().into_owned());
                }
                if mount.auto_approve_tools {
                    args.push("--allowedTools".to_string());
                    args.push("mcp__photocraft__*".to_string());
                }
            }
            if use_stdin {
                args.push("-p".to_string());
                (args, Some(prompt.to_string()))
            } else {
                (args, None)
            }
        }
        AiProviderKind::CodexCli => {
            let mut args = vec![
                "exec".to_string(),
                "--json".to_string(),
                "--skip-git-repo-check".to_string(),
            ];
            if let Some(mount) = mcp {
                let cmd_val = format_toml_string(&mount.photocraft_cli_path.to_string_lossy());
                args.push("-c".to_string());
                args.push(format!("mcp_servers.photocraft.command={cmd_val}"));
                args.push("-c".to_string());
                args.push("mcp_servers.photocraft.args=[\"mcp\"]".to_string());
                for (key, val) in &mount.server_env {
                    args.push("-c".to_string());
                    args.push(format!("mcp_servers.photocraft.env.{key}={}", format_toml_string(val)));
                }
                if mount.auto_approve_tools {
                    args.push("--full-auto".to_string());
                }
            }
            if use_stdin {
                args.push("-".to_string());
                (args, Some(prompt.to_string()))
            } else {
                args.push(prompt.to_string());
                (args, None)
            }
        }
        AiProviderKind::GeminiApi => return None,
    };

    let mut invocation = wrap_command_for_platform(binary_path, raw_args, stdin_payload, is_windows);
    invocation.working_dir = mcp.map(|m| m.workspace_dir.clone());
    Some(invocation)
}

/// Builds the `--version` readiness check invocation for `binary_path`.
pub fn build_version_invocation(binary_path: &Path, is_windows: bool) -> CliInvocation {
    wrap_command_for_platform(binary_path, vec!["--version".to_string()], None, is_windows)
}

/// Returns whether `provider` currently supports resuming an external CLI session/conversation.
pub fn supports_session_resume(provider: AiProviderKind) -> bool {
    matches!(provider, AiProviderKind::AntigravityCli | AiProviderKind::ClaudeCli)
}

/// Returns the official default installation bin directory for Antigravity CLI (`agy`).
pub fn default_antigravity_bin_dir(
    local_appdata: Option<&Path>,
    home: Option<&Path>,
    is_windows: bool,
) -> Option<PathBuf> {
    if is_windows {
        local_appdata.map(|p| p.join("agy").join("bin"))
    } else {
        home.map(|p| p.join(".local").join("bin"))
    }
}

/// Returns the file extension for an image attachment MIME type.
pub fn attachment_extension(mime_type: &str) -> &'static str {
    let lower = mime_type.to_ascii_lowercase();
    if lower.contains("png") {
        "png"
    } else if lower.contains("webp") {
        "webp"
    } else {
        "jpg"
    }
}

/// Formats an attachment reference block pointing the CLI to a temporary image file.
pub fn format_attachment_reference(note: &AttachmentNote) -> String {
    format!(
        "[Attached PhotoCraft Canvas Image]\n- Label: {}\n- Dimensions: {}x{} px ({})\n- File Path: {}\n(Inspect this local image file directly to analyze exposure, color, composition, or retouching details.)",
        note.label,
        note.width,
        note.height,
        note.mime_type,
        note.file_path.display()
    )
}

/// Builds a clean single-turn or multi-turn prompt from pruned conversation messages
/// and materialized image attachment paths (without MCP instructions).
pub fn build_cli_prompt(
    messages: &[ChatMessage],
    resuming_session: bool,
    attachments: &[AttachmentNote],
) -> String {
    build_cli_prompt_with_mcp(messages, resuming_session, attachments, false)
}

/// Builds a clean single-turn or multi-turn prompt from pruned conversation messages,
/// optionally instructing the CLI agent to use the attached live `photocraft` MCP tools.
pub fn build_cli_prompt_with_mcp(
    messages: &[ChatMessage],
    resuming_session: bool,
    attachments: &[AttachmentNote],
    mcp_enabled: bool,
) -> String {
    let system_header = if mcp_enabled {
        "You are PhotoCraft's built-in AI Assistant embedded in PhotoCraft's right-hand AI sidebar, with the `photocraft` MCP server attached to the live open PhotoCraft desktop window. When the user asks to inspect the canvas, list or query layers, adjust colors, apply filters, or perform any image editing action, prefer calling the `photocraft` MCP tools directly (such as `doc_inspect`, `command_list`, `command_run`, `doc_render_preview`, `ui_inspect`, etc.) on the currently open document. Respond concisely in the same language as the user's request and summarize the inspected details or applied changes."
    } else {
        "You are PhotoCraft's built-in AI Assistant helping a user edit images in PhotoCraft (a native Rust image editor). Provide concise, practical, accurate guidance on layers, masks, color grading, adjustments, filters, retouching, composition, and editing workflows."
    };

    if resuming_session
        && let Some((last_idx, last_user)) = messages
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| m.role == ChatRole::User)
    {
        let mut out = String::new();
        for note in attachments.iter().filter(|a| a.message_index == last_idx) {
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&format_attachment_reference(note));
        }
        if !last_user.content.trim().is_empty() {
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(last_user.content.trim());
        }
        return out;
    }

    let latest_user_idx = messages.iter().rposition(|m| m.role == ChatRole::User);
    let mut history_lines = Vec::new();

    for (idx, msg) in messages.iter().enumerate() {
        if Some(idx) == latest_user_idx {
            continue;
        }
        if msg.tool_call.is_some() {
            continue;
        }
        let text = msg.content.trim();
        let msg_attachments: Vec<&AttachmentNote> = attachments.iter().filter(|a| a.message_index == idx).collect();
        if text.is_empty() && msg_attachments.is_empty() {
            continue;
        }
        let role_tag = match msg.role {
            ChatRole::User => "User",
            ChatRole::Model => "Assistant",
            ChatRole::System => "System",
            ChatRole::Function => continue,
        };
        let mut entry = format!("{role_tag}: {text}");
        for note in msg_attachments {
            entry.push('\n');
            entry.push_str(&format_attachment_reference(note));
        }
        history_lines.push(entry);
    }

    let mut prompt = String::from(system_header);
    if !history_lines.is_empty() {
        prompt.push_str("\n\nConversation history:\n");
        prompt.push_str(&history_lines.join("\n\n"));
    }

    if let Some(last_idx) = latest_user_idx
        && let Some(last_msg) = messages.get(last_idx)
    {
        prompt.push_str("\n\nCurrent user request:\n");
        let last_attachments: Vec<&AttachmentNote> = attachments.iter().filter(|a| a.message_index == last_idx).collect();
        for note in &last_attachments {
            prompt.push_str(&format_attachment_reference(note));
            prompt.push_str("\n\n");
        }
        prompt.push_str(last_msg.content.trim());
    }

    prompt
}

/// Writes an image attachment to `target_dir` and returns its [`AttachmentNote`].
#[cfg(not(target_arch = "wasm32"))]
pub fn write_attachment_to_dir(
    target_dir: &Path,
    message_index: usize,
    att: &ChatImageAttachment,
    unique_stem: &str,
) -> Result<AttachmentNote, String> {
    use base64::Engine;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(att.base64_data.trim())
        .map_err(|e| format!("Failed to decode attached image: {e}"))?;

    std::fs::create_dir_all(target_dir).map_err(|e| format!("Failed to prepare temp attachment folder: {e}"))?;
    let ext = attachment_extension(&att.mime_type);
    let file_path = target_dir.join(format!("{unique_stem}.{ext}"));
    std::fs::write(&file_path, &bytes).map_err(|e| format!("Failed to write temp attachment file: {e}"))?;

    Ok(AttachmentNote {
        message_index,
        label: att.label.clone().unwrap_or_else(|| "Canvas Screenshot".to_string()),
        width: att.width,
        height: att.height,
        mime_type: att.mime_type.clone(),
        file_path,
    })
}

/// RAII guard that cleans up temporary canvas screenshot files when a CLI turn ends.
#[cfg(not(target_arch = "wasm32"))]
struct TempAttachmentGuard {
    paths: Vec<PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TempAttachmentGuard {
    fn materialize(messages: &[ChatMessage], resuming_session: bool) -> (Self, Vec<AttachmentNote>) {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let temp_dir = std::env::temp_dir().join("photocraft-ai-attachments");
        let pid = std::process::id();
        let batch = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let latest_user_idx = messages.iter().rposition(|m| m.role == ChatRole::User);
        let mut paths = Vec::new();
        let mut notes = Vec::new();

        for (idx, msg) in messages.iter().enumerate() {
            if resuming_session && Some(idx) != latest_user_idx {
                continue;
            }
            let Some(ref att) = msg.image else {
                continue;
            };
            let stem = format!("photocraft-canvas-{pid}-{batch}-{idx}");
            match write_attachment_to_dir(&temp_dir, idx, att, &stem) {
                Ok(note) => {
                    paths.push(note.file_path.clone());
                    notes.push(note);
                }
                Err(err) => {
                    log::warn!("Skipping CLI temp image attachment: {err}");
                }
            }
        }

        (Self { paths }, notes)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for TempAttachmentGuard {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// RAII guard that creates an isolated temporary MCP workspace directory for a single
/// CLI turn and removes it automatically when the worker exits (never touching global CLI configs).
#[cfg(not(target_arch = "wasm32"))]
struct TempMcpWorkspaceGuard {
    root: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl TempMcpWorkspaceGuard {
    fn prepare(provider: AiProviderKind, options: &CliMcpOptions) -> Result<(Self, PreparedMcpMount), String> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let pid = std::process::id();
        let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("photocraft-ai-mcp-{pid}-{seq}"));
        match write_mcp_workspace_config(provider, &root, options) {
            Ok(mount) => Ok((Self { root }, mount)),
            Err(err) => {
                let _ = std::fs::remove_dir_all(&root);
                Err(err)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for TempMcpWorkspaceGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Strips ANSI CSI/OSC escape sequences so terminal styling never leaks into chat text.
pub fn strip_ansi_escapes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\x1b' {
            out.push(ch);
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                let _ = chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            Some(']') => {
                let _ = chars.next();
                while let Some(next) = chars.next() {
                    if next == '\x07' {
                        break;
                    }
                    if next == '\x1b' && chars.peek() == Some(&'\\') {
                        let _ = chars.next();
                        break;
                    }
                }
            }
            Some(_) => {
                let _ = chars.next();
            }
            None => {}
        }
    }
    out
}

/// Stateful NDJSON / stream-json line parser for Antigravity, Gemini, Claude,
/// and Codex CLIs, with deduplication and safe plain-text fallback.
#[derive(Clone, Debug)]
pub struct CliStreamParser {
    provider: AiProviderKind,
    emitted_any_text: bool,
    emitted_plain_text: bool,
    gemini_emitted_buf: String,
    claude_turn_delta_buf: String,
    codex_item_emitted: HashMap<String, String>,
    tool_names_by_id: HashMap<String, String>,
    tool_updates: Vec<CliToolUpdate>,
    last_error: Option<String>,
}

impl CliStreamParser {
    /// Creates a new line-by-line parser for `provider`.
    pub fn new(provider: AiProviderKind) -> Self {
        Self {
            provider,
            emitted_any_text: false,
            emitted_plain_text: false,
            gemini_emitted_buf: String::new(),
            claude_turn_delta_buf: String::new(),
            codex_item_emitted: HashMap::new(),
            tool_names_by_id: HashMap::new(),
            tool_updates: Vec::new(),
            last_error: None,
        }
    }

    /// Whether any assistant text chunk has been emitted so far.
    pub fn emitted_any_text(&self) -> bool {
        self.emitted_any_text
    }

    /// Tool step updates observed during this stream.
    pub fn tool_updates(&self) -> &[CliToolUpdate] {
        &self.tool_updates
    }

    /// Last fatal error reported via a structured stream event, if any.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Parses one raw stdout line from the CLI process.
    pub fn parse_line(&mut self, raw_line: &str) -> ParsedStreamLine {
        let stripped = strip_ansi_escapes(raw_line);
        let trimmed = stripped.trim();
        if trimmed.is_empty() {
            return ParsedStreamLine::default();
        }

        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            return self.parse_plain_text_fallback(stripped.trim_end_matches(['\r', '\n']));
        };

        if !value.is_object() {
            return self.parse_plain_text_fallback(trimmed);
        }

        let parsed = match self.provider {
            AiProviderKind::AntigravityCli => self.parse_antigravity_event(&value),
            AiProviderKind::GeminiCli => self.parse_gemini_event(&value),
            AiProviderKind::ClaudeCli => self.parse_claude_event(&value),
            AiProviderKind::CodexCli => self.parse_codex_event(&value),
            AiProviderKind::GeminiApi => ParsedStreamLine::default(),
        };

        if !parsed.tool_updates.is_empty() {
            self.tool_updates.extend(parsed.tool_updates.iter().cloned());
        }
        if let Some(ref err) = parsed.error {
            self.last_error = Some(err.clone());
        }
        if parsed.delta_text.as_ref().is_some_and(|s| !s.is_empty()) {
            self.emitted_any_text = true;
        }
        parsed
    }

    fn record_tool_name(&mut self, call_id: Option<&str>, name: &str) {
        if let Some(id) = call_id.map(str::trim).filter(|s| !s.is_empty())
            && !name.trim().is_empty()
            && name != "tool"
        {
            self.tool_names_by_id.insert(id.to_string(), name.to_string());
        }
    }

    fn resolve_tool_name(&self, call_id: Option<&str>, explicit: Option<&str>) -> String {
        if let Some(name) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
            return name.to_string();
        }
        if let Some(id) = call_id.map(str::trim).filter(|s| !s.is_empty())
            && let Some(known) = self.tool_names_by_id.get(id)
        {
            return known.clone();
        }
        "tool".to_string()
    }

    fn parse_plain_text_fallback(&mut self, line: &str) -> ParsedStreamLine {
        let mut text = String::new();
        if self.emitted_plain_text || (self.emitted_any_text && !line.is_empty()) {
            text.push('\n');
        }
        text.push_str(line);
        self.emitted_plain_text = true;
        self.emitted_any_text = true;
        ParsedStreamLine {
            delta_text: Some(text),
            session_id: None,
            tool_updates: Vec::new(),
            error: None,
        }
    }

    fn parse_antigravity_event(&mut self, val: &Value) -> ParsedStreamLine {
        let mut out = ParsedStreamLine {
            session_id: extract_session_id(val),
            ..Default::default()
        };
        let event = val
            .get("event")
            .or_else(|| val.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("");

        match event {
            "init" => {}
            "step_update" => {
                if let Some(step) = val.get("step_update") {
                    let (delta, tool) = self.parse_antigravity_step_update(step);
                    out.delta_text = delta;
                    if let Some(update) = tool {
                        out.tool_updates.push(update);
                    }
                }
            }
            "result" => {
                if let Some(res) = val.get("result") {
                    if out.session_id.is_none() {
                        out.session_id = extract_session_id(res);
                    }
                    let err_msg = extract_error_message(res);
                    let status = res.get("status").and_then(Value::as_str).unwrap_or("");
                    let failed = status.eq_ignore_ascii_case("error") || status.eq_ignore_ascii_case("failed");
                    if let Some(err) = err_msg {
                        out.error = Some(err);
                    } else if failed {
                        out.error = extract_text_field(res.get("response"))
                            .or_else(|| Some(format!("Antigravity CLI finished with status `{status}`.")));
                    } else if !self.emitted_any_text {
                        out.delta_text = extract_text_field(res.get("response"));
                    }
                } else {
                    let err_msg = extract_error_message(val);
                    if let Some(err) = err_msg {
                        out.error = Some(err);
                    } else if !self.emitted_any_text {
                        out.delta_text = extract_text_field(val.get("response"));
                    }
                }
            }
            "error" => {
                out.error = extract_error_message(val);
            }
            other => {
                out.delta_text = extract_unknown_event_text(other, val);
            }
        }
        out
    }

    fn parse_antigravity_step_update(&mut self, step: &Value) -> (Option<String>, Option<CliToolUpdate>) {
        let step_type = step.get("step_type").and_then(Value::as_str).unwrap_or("");
        match step_type {
            "agent_response" => {
                if let Some(delta) = step.get("text_delta").and_then(Value::as_str)
                    && !delta.is_empty()
                {
                    return (Some(delta.to_string()), None);
                }
                if !self.emitted_any_text
                    && let Some(text) = extract_text_field(step.get("text")).or_else(|| extract_text_field(step.get("response")))
                {
                    return (Some(text), None);
                }
                (None, None)
            }
            "tool" => {
                let step_index = step
                    .get("step_index")
                    .or_else(|| step.get("stepIndex"))
                    .and_then(Value::as_u64);
                let tool_info = step
                    .get("tool_info")
                    .or_else(|| step.get("toolInfo"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let call_id = step
                    .get("call_id")
                    .or_else(|| step.get("callId"))
                    .or_else(|| step.get("id"))
                    .or_else(|| tool_info.get("call_id"))
                    .or_else(|| tool_info.get("id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                let tool_name = tool_info
                    .get("name")
                    .or_else(|| step.get("tool_name"))
                    .or_else(|| step.get("toolName"))
                    .or_else(|| step.get("name"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("tool")
                    .to_string();
                let parameters = tool_info
                    .get("parameters")
                    .or_else(|| tool_info.get("arguments"))
                    .or_else(|| tool_info.get("input"))
                    .or_else(|| step.get("parameters"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let output = tool_info
                    .get("output")
                    .or_else(|| tool_info.get("result"))
                    .or_else(|| step.get("output"))
                    .filter(|v| !v.is_null())
                    .cloned();
                let error = extract_tool_error(&tool_info).or_else(|| extract_tool_error(step));
                let mut state = parse_tool_state(step.get("state").and_then(Value::as_str).unwrap_or(""));
                if error.is_some() && matches!(state, CliToolState::Done | CliToolState::Unknown) {
                    state = CliToolState::Failed;
                }
                self.record_tool_name(call_id.as_deref(), &tool_name);
                (
                    None,
                    Some(CliToolUpdate {
                        call_id,
                        step_index,
                        tool_name,
                        state,
                        parameters,
                        output,
                        error,
                        tool_info,
                    }),
                )
            }
            _ => (None, None),
        }
    }

    fn parse_gemini_event(&mut self, val: &Value) -> ParsedStreamLine {
        let mut out = ParsedStreamLine {
            session_id: extract_session_id(val),
            ..Default::default()
        };
        let kind = val.get("type").and_then(Value::as_str).unwrap_or("");

        match kind {
            "init" => {}
            "tool_use" => {
                let call_id = val
                    .get("tool_id")
                    .or_else(|| val.get("call_id"))
                    .or_else(|| val.get("id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                let tool_name = self.resolve_tool_name(
                    call_id.as_deref(),
                    val.get("tool_name").or_else(|| val.get("name")).and_then(Value::as_str),
                );
                let parameters = val
                    .get("parameters")
                    .or_else(|| val.get("args"))
                    .or_else(|| val.get("input"))
                    .cloned()
                    .unwrap_or(Value::Null);
                self.record_tool_name(call_id.as_deref(), &tool_name);
                out.tool_updates.push(CliToolUpdate {
                    call_id,
                    step_index: None,
                    tool_name,
                    state: CliToolState::Active,
                    parameters,
                    output: None,
                    error: None,
                    tool_info: val.clone(),
                });
            }
            "tool_result" => {
                let call_id = val
                    .get("tool_id")
                    .or_else(|| val.get("call_id"))
                    .or_else(|| val.get("id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                let tool_name = self.resolve_tool_name(
                    call_id.as_deref(),
                    val.get("tool_name").or_else(|| val.get("name")).and_then(Value::as_str),
                );
                let output = val
                    .get("output")
                    .or_else(|| val.get("result"))
                    .or_else(|| val.get("content"))
                    .filter(|v| !v.is_null())
                    .cloned();
                let status_str = val.get("status").and_then(Value::as_str).unwrap_or("success");
                let explicit_err = extract_tool_error(val);
                let is_error = val.get("is_error").and_then(Value::as_bool).unwrap_or(false)
                    || status_str.eq_ignore_ascii_case("error")
                    || status_str.eq_ignore_ascii_case("failed")
                    || explicit_err.is_some();
                let error = if is_error {
                    explicit_err
                        .or_else(|| extract_text_field(output.as_ref()))
                        .or_else(|| Some("Tool call failed.".to_string()))
                } else {
                    None
                };
                out.tool_updates.push(CliToolUpdate {
                    call_id,
                    step_index: None,
                    tool_name,
                    state: if is_error { CliToolState::Failed } else { CliToolState::Done },
                    parameters: Value::Null,
                    output,
                    error,
                    tool_info: val.clone(),
                });
            }
            "message" => {
                let role = val.get("role").and_then(Value::as_str).unwrap_or("");
                if matches!(role, "user" | "system" | "tool") {
                    return out;
                }
                let text = extract_text_field(val.get("content")).or_else(|| extract_text_field(val.get("text")));
                let is_delta = val.get("delta").and_then(Value::as_bool).unwrap_or(false);
                if let Some(text) = text {
                    out.delta_text = self.dedup_gemini_text(&text, is_delta);
                }
            }
            "error" => {
                let severity = val.get("severity").and_then(Value::as_str).unwrap_or("error");
                if !severity.eq_ignore_ascii_case("warning") {
                    out.error = extract_error_message(val);
                }
            }
            "result" => {
                let status = val.get("status").and_then(Value::as_str).unwrap_or("success");
                if status.eq_ignore_ascii_case("error") || val.get("is_error").and_then(Value::as_bool) == Some(true) {
                    out.error = extract_error_message(val);
                } else if !self.emitted_any_text {
                    out.delta_text = extract_text_field(val.get("response"))
                        .or_else(|| extract_text_field(val.get("result")))
                        .or_else(|| extract_text_field(val.get("text")));
                }
            }
            other => {
                out.delta_text = extract_unknown_event_text(other, val);
            }
        }
        out
    }

    fn dedup_gemini_text(&mut self, text: &str, is_delta: bool) -> Option<String> {
        if text.is_empty() {
            return None;
        }
        if is_delta {
            self.gemini_emitted_buf.push_str(text);
            return Some(text.to_string());
        }
        if !self.gemini_emitted_buf.is_empty() {
            if text == self.gemini_emitted_buf {
                return None;
            }
            if let Some(suffix) = text.strip_prefix(&self.gemini_emitted_buf) {
                self.gemini_emitted_buf.push_str(suffix);
                return (!suffix.is_empty()).then(|| suffix.to_string());
            }
        }
        self.gemini_emitted_buf.push_str(text);
        Some(text.to_string())
    }

    fn parse_claude_event(&mut self, val: &Value) -> ParsedStreamLine {
        let mut out = ParsedStreamLine {
            session_id: extract_session_id(val),
            ..Default::default()
        };
        let kind = val.get("type").and_then(Value::as_str).unwrap_or("");

        match kind {
            "system" => {}
            "tool_use" => {
                if let Some(update) = self.parse_claude_tool_use_block(val) {
                    out.tool_updates.push(update);
                }
            }
            "tool_result" => {
                if let Some(update) = self.parse_claude_tool_result_block(val) {
                    out.tool_updates.push(update);
                }
            }
            "user" => {
                let content = val
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .or_else(|| val.get("content"));
                if let Some(blocks) = content.and_then(Value::as_array) {
                    for block in blocks {
                        if block.get("type").and_then(Value::as_str) == Some("tool_result")
                            && let Some(update) = self.parse_claude_tool_result_block(block)
                        {
                            out.tool_updates.push(update);
                        }
                    }
                }
            }
            "stream_event" => {
                if let Some(event) = val.get("event") {
                    let (delta, tool) = self.parse_claude_stream_inner(event);
                    out.delta_text = delta;
                    if let Some(update) = tool {
                        out.tool_updates.push(update);
                    }
                }
            }
            "content_block_delta" | "content_block_start" => {
                let (delta, tool) = self.parse_claude_stream_inner(val);
                out.delta_text = delta;
                if let Some(update) = tool {
                    out.tool_updates.push(update);
                }
            }
            "assistant" => {
                let content = val
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .or_else(|| val.get("content"));
                if let Some(blocks) = content.and_then(Value::as_array) {
                    for block in blocks {
                        if block.get("type").and_then(Value::as_str) == Some("tool_use")
                            && let Some(update) = self.parse_claude_tool_use_block(block)
                        {
                            out.tool_updates.push(update);
                        }
                    }
                }
                let text = extract_text_field(content);
                if let Some(text) = text {
                    out.delta_text = self.dedup_claude_assistant(&text);
                }
                self.claude_turn_delta_buf.clear();
            }
            "result" => {
                let is_err = val.get("is_error").and_then(Value::as_bool).unwrap_or(false)
                    || val
                        .get("subtype")
                        .and_then(Value::as_str)
                        .is_some_and(|s| s.to_ascii_lowercase().contains("error"));
                if is_err {
                    out.error = extract_error_message(val).or_else(|| extract_text_field(val.get("result")));
                } else if !self.emitted_any_text {
                    out.delta_text = extract_text_field(val.get("result"));
                }
            }
            "error" => {
                out.error = extract_error_message(val);
            }
            other => {
                out.delta_text = extract_unknown_event_text(other, val);
            }
        }
        out
    }

    fn parse_claude_tool_use_block(&mut self, block: &Value) -> Option<CliToolUpdate> {
        let call_id = block
            .get("id")
            .or_else(|| block.get("tool_use_id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let tool_name = self.resolve_tool_name(
            call_id.as_deref(),
            block.get("name").or_else(|| block.get("tool_name")).and_then(Value::as_str),
        );
        let parameters = block
            .get("input")
            .or_else(|| block.get("parameters"))
            .cloned()
            .unwrap_or(Value::Null);
        self.record_tool_name(call_id.as_deref(), &tool_name);
        Some(CliToolUpdate {
            call_id,
            step_index: None,
            tool_name,
            state: CliToolState::Active,
            parameters,
            output: None,
            error: None,
            tool_info: block.clone(),
        })
    }

    fn parse_claude_tool_result_block(&self, block: &Value) -> Option<CliToolUpdate> {
        let call_id = block
            .get("tool_use_id")
            .or_else(|| block.get("id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let tool_name = self.resolve_tool_name(
            call_id.as_deref(),
            block.get("name").or_else(|| block.get("tool_name")).and_then(Value::as_str),
        );
        let is_error = block.get("is_error").and_then(Value::as_bool).unwrap_or(false);
        let content = block
            .get("content")
            .or_else(|| block.get("output"))
            .filter(|v| !v.is_null())
            .cloned();
        let error = if is_error {
            extract_tool_error(block)
                .or_else(|| extract_text_field(content.as_ref()))
                .or_else(|| Some("Tool call failed.".to_string()))
        } else {
            None
        };
        Some(CliToolUpdate {
            call_id,
            step_index: None,
            tool_name,
            state: if is_error { CliToolState::Failed } else { CliToolState::Done },
            parameters: Value::Null,
            output: content,
            error,
            tool_info: block.clone(),
        })
    }

    fn parse_claude_stream_inner(&mut self, event: &Value) -> (Option<String>, Option<CliToolUpdate>) {
        let inner_type = event.get("type").and_then(Value::as_str).unwrap_or("");
        if inner_type == "message_start" {
            self.claude_turn_delta_buf.clear();
            return (None, None);
        }
        if inner_type == "content_block_start"
            && let Some(block) = event.get("content_block")
            && block.get("type").and_then(Value::as_str) == Some("tool_use")
        {
            return (None, self.parse_claude_tool_use_block(block));
        }
        if inner_type == "content_block_delta"
            && let Some(delta) = event.get("delta")
            && let Some(text) = delta.get("text").and_then(Value::as_str)
            && !text.is_empty()
        {
            self.claude_turn_delta_buf.push_str(text);
            return (Some(text.to_string()), None);
        }
        (None, None)
    }

    fn dedup_claude_assistant(&mut self, text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }
        if !self.claude_turn_delta_buf.is_empty() {
            if text == self.claude_turn_delta_buf {
                return None;
            }
            if let Some(suffix) = text.strip_prefix(&self.claude_turn_delta_buf) {
                return (!suffix.is_empty()).then(|| suffix.to_string());
            }
        }
        if self.emitted_any_text {
            Some(format!("\n\n{text}"))
        } else {
            Some(text.to_string())
        }
    }

    fn parse_codex_event(&mut self, val: &Value) -> ParsedStreamLine {
        let mut out = ParsedStreamLine {
            session_id: extract_session_id(val),
            ..Default::default()
        };
        let kind = val.get("type").and_then(Value::as_str).unwrap_or("");

        match kind {
            "thread.started" | "turn.started" | "turn.completed" => {}
            "item.agent_message.delta" | "agent_message.delta" | "response.output_text.delta" => {
                let item_id = val.get("item_id").and_then(Value::as_str).unwrap_or("default");
                if let Some(delta) = val.get("delta").and_then(Value::as_str)
                    && !delta.is_empty()
                {
                    let buf = self.codex_item_emitted.entry(item_id.to_string()).or_default();
                    buf.push_str(delta);
                    out.delta_text = Some(delta.to_string());
                }
            }
            "item.started" | "item.updated" | "item.completed" => {
                if let Some(item) = val.get("item") {
                    if let Some(tool) = self.parse_codex_tool_item(kind, item) {
                        out.tool_updates.push(tool);
                    } else if kind != "item.started" {
                        out.delta_text = self.parse_codex_item(item);
                    }
                }
            }
            "turn.failed" | "error" => {
                out.error = extract_error_message(val);
            }
            other => {
                out.delta_text = extract_unknown_event_text(other, val);
            }
        }
        out
    }

    fn parse_codex_tool_item(&mut self, event_kind: &str, item: &Value) -> Option<CliToolUpdate> {
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        if !matches!(item_type, "mcp_tool_call" | "tool_call" | "function_call") {
            return None;
        }
        let call_id = item
            .get("id")
            .or_else(|| item.get("call_id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let tool_name = self.resolve_tool_name(
            call_id.as_deref(),
            item.get("tool")
                .or_else(|| item.get("tool_name"))
                .or_else(|| item.get("name"))
                .and_then(Value::as_str),
        );
        let parameters = item
            .get("arguments")
            .or_else(|| item.get("parameters"))
            .or_else(|| item.get("input"))
            .cloned()
            .unwrap_or(Value::Null);
        let output = item
            .get("result")
            .or_else(|| item.get("output"))
            .filter(|v| !v.is_null())
            .cloned();
        let error = extract_tool_error(item);
        let mut state = item
            .get("status")
            .and_then(Value::as_str)
            .map(parse_tool_state)
            .unwrap_or(CliToolState::Unknown);
        if state == CliToolState::Unknown {
            state = match event_kind {
                "item.completed" => {
                    if error.is_some() {
                        CliToolState::Failed
                    } else {
                        CliToolState::Done
                    }
                }
                _ => CliToolState::Active,
            };
        }
        if error.is_some() && state == CliToolState::Done {
            state = CliToolState::Failed;
        }
        self.record_tool_name(call_id.as_deref(), &tool_name);
        Some(CliToolUpdate {
            call_id,
            step_index: None,
            tool_name,
            state,
            parameters,
            output,
            error,
            tool_info: item.clone(),
        })
    }

    fn parse_codex_item(&mut self, item: &Value) -> Option<String> {
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        if !matches!(item_type, "agent_message" | "assistant_message" | "message") {
            return None;
        }
        if item_type == "message"
            && let Some(role) = item.get("role").and_then(Value::as_str)
            && role != "assistant"
        {
            return None;
        }

        let text = extract_text_field(item.get("text")).or_else(|| extract_text_field(item.get("content")))?;
        if text.is_empty() {
            return None;
        }

        let item_id = item.get("id").and_then(Value::as_str).unwrap_or("default").to_string();
        let already_had_other_items = self.emitted_any_text && !self.codex_item_emitted.contains_key(&item_id);
        let emitted = self.codex_item_emitted.entry(item_id).or_default();

        if text == *emitted {
            return None;
        }
        if let Some(suffix) = text.strip_prefix(emitted.as_str()) {
            emitted.push_str(suffix);
            return (!suffix.is_empty()).then(|| suffix.to_string());
        }

        emitted.clear();
        emitted.push_str(&text);
        if already_had_other_items {
            Some(format!("\n\n{text}"))
        } else {
            Some(text)
        }
    }
}

fn parse_tool_state(raw: &str) -> CliToolState {
    match raw.trim().to_ascii_uppercase().as_str() {
        "ACTIVE" | "RUNNING" | "IN_PROGRESS" | "PENDING" | "STARTED" => CliToolState::Active,
        "DONE" | "COMPLETED" | "SUCCESS" | "SUCCEEDED" => CliToolState::Done,
        "ERROR" | "FAILED" | "CANCELLED" | "CANCELED" => CliToolState::Failed,
        _ => CliToolState::Unknown,
    }
}

fn extract_tool_error(val: &Value) -> Option<String> {
    if let Some(err) = val.get("error") {
        if let Some(s) = err.as_str()
            && !s.trim().is_empty()
        {
            return Some(s.trim().to_string());
        }
        if let Some(msg) = err.get("message").and_then(Value::as_str)
            && !msg.trim().is_empty()
        {
            return Some(msg.trim().to_string());
        }
    }
    None
}

fn extract_session_id(val: &Value) -> Option<String> {
    for key in [
        "conversation_id",
        "conversationId",
        "session_id",
        "sessionId",
        "thread_id",
        "threadId",
    ] {
        if let Some(id) = val.get(key).and_then(Value::as_str) {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_text_field(val: Option<&Value>) -> Option<String> {
    let val = val?;
    match val {
        Value::String(s) => (!s.is_empty()).then(|| s.clone()),
        Value::Array(items) => {
            let mut combined = String::new();
            for item in items {
                if let Some(s) = item.as_str() {
                    combined.push_str(s);
                } else if let Some(obj) = item.as_object() {
                    let block_type = obj.get("type").and_then(Value::as_str).unwrap_or("text");
                    if matches!(block_type, "text" | "output_text" | "text_delta")
                        && let Some(t) = obj.get("text").and_then(Value::as_str)
                    {
                        combined.push_str(t);
                    }
                }
            }
            (!combined.is_empty()).then_some(combined)
        }
        Value::Object(obj) => obj
            .get("text")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

fn extract_error_message(val: &Value) -> Option<String> {
    if let Some(msg) = val.get("message").and_then(Value::as_str)
        && !msg.trim().is_empty()
    {
        return Some(msg.trim().to_string());
    }
    if let Some(err) = val.get("error") {
        if let Some(s) = err.as_str()
            && !s.trim().is_empty()
        {
            return Some(s.trim().to_string());
        }
        if let Some(msg) = err.get("message").and_then(Value::as_str)
            && !msg.trim().is_empty()
        {
            return Some(msg.trim().to_string());
        }
    }
    None
}

fn extract_unknown_event_text(kind: &str, val: &Value) -> Option<String> {
    let lower = kind.to_ascii_lowercase();
    if lower.is_empty()
        || lower.contains("init")
        || lower.contains("system")
        || lower.contains("user")
        || lower.contains("tool")
        || lower.contains("reasoning")
        || lower.contains("usage")
        || lower.contains("stats")
        || lower.contains("ping")
        || lower.ends_with(".started")
        || lower.ends_with(".completed")
    {
        return None;
    }
    for key in ["delta", "text", "output_text", "response"] {
        if let Some(s) = val.get(key).and_then(Value::as_str)
            && !s.is_empty()
        {
            return Some(s.to_string());
        }
    }
    None
}

/// Returns whether `combined_lower` contains indicators of missing login or expired OAuth.
fn looks_like_auth_error(combined_lower: &str) -> bool {
    const DIRECT_PATTERNS: &[&str] = &[
        "not logged in",
        "not signed in",
        "not authenticated",
        "unauthenticated",
        "authentication required",
        "authentication failed",
        "authentication error",
        "auth required",
        "auth error",
        "failed to authenticate",
        "please log in",
        "please login",
        "please sign in",
        "login required",
        "sign in required",
        "re-authenticate",
        "invalid_grant",
        "token expired",
        "session expired",
        "credentials expired",
        "invalid credentials",
        "missing credentials",
        "no credentials",
        "401 unauthorized",
        "status 401",
        "http 401",
        "invalid api key",
        "run /login",
        "agy login",
        "claude login",
        "codex login",
        "gemini auth",
    ];

    if DIRECT_PATTERNS.iter().any(|p| combined_lower.contains(p)) {
        return true;
    }

    if combined_lower.contains("oauth")
        && ["error", "expired", "invalid", "failed", "required", "login", "token", "revoked"]
            .iter()
            .any(|kw| combined_lower.contains(kw))
    {
        return true;
    }

    if combined_lower.contains("unauthorized")
        && ["auth", "token", "login", "credential", "account", "401", "permission"]
            .iter()
            .any(|kw| combined_lower.contains(kw))
    {
        return true;
    }

    false
}

/// Produces a concise, ANSI-free summary from `stream_error` and `stderr`.
pub fn summarize_stderr(stderr: &str, stream_error: Option<&str>) -> String {
    if let Some(err) = stream_error {
        let cleaned = strip_ansi_escapes(err).trim().to_string();
        if !cleaned.is_empty() {
            return truncate_chars(&cleaned, MAX_ERROR_SUMMARY_CHARS);
        }
    }

    let cleaned = strip_ansi_escapes(stderr);
    let meaningful: Vec<&str> = cleaned
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with("(node:") || l.to_ascii_lowercase().contains("error"))
        .take(4)
        .collect();

    let joined = if !meaningful.is_empty() {
        meaningful.join(" | ")
    } else {
        cleaned.lines().map(str::trim).filter(|l| !l.is_empty()).take(3).collect::<Vec<_>>().join(" | ")
    };

    if joined.is_empty() {
        "Process exited without error details.".to_string()
    } else {
        truncate_chars(&joined, MAX_ERROR_SUMMARY_CHARS)
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Classifies a CLI failure into a structured [`AiBackendError`].
pub fn classify_cli_error(
    provider: AiProviderKind,
    exit_code: Option<i32>,
    stderr: &str,
    stream_error: Option<&str>,
) -> AiBackendError {
    let summary = summarize_stderr(stderr, stream_error);
    let combined = format!(
        "{}\n{}",
        stream_error.unwrap_or_default(),
        strip_ansi_escapes(stderr)
    )
    .to_ascii_lowercase();

    if looks_like_auth_error(&combined) {
        return AiBackendError::CliNotAuthenticated {
            provider: provider.display_name(),
            login_command: provider.cli_login_command().unwrap_or("cli"),
            account_label: provider.cli_account_label().unwrap_or("account"),
            details: summary,
        };
    }

    AiBackendError::CliExecutionFailed {
        provider: provider.display_name(),
        exit_code,
        summary,
    }
}

/// Validates that the configured local CLI provider is available on the host.
pub fn validate_ready(provider: AiProviderKind) -> Result<(), AiBackendError> {
    validate_ready_with_override(provider, None)
}

/// Validates that the configured local CLI provider is available on the host,
/// respecting an optional user-configured executable path override.
pub fn validate_ready_with_override(provider: AiProviderKind, custom_override: Option<&str>) -> Result<(), AiBackendError> {
    match detect_cli_with_override(provider, custom_override) {
        CliDetectionStatus::Found(_) => Ok(()),
        CliDetectionStatus::NotFound => Err(cli_not_found_error(provider)),
        CliDetectionStatus::UnsupportedPlatform => Err(AiBackendError::UnsupportedPlatform),
    }
}

/// Starts a streaming multi-turn conversation using a local CLI agent.
pub fn start_stream_turn(
    config: &AiRequestConfig,
    messages: &[ChatMessage],
    wake: WakeCallback,
) -> Result<AiWorkerHandle, AiBackendError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let binary_path = match detect_cli_with_override(config.provider, config.cli_binary_override.as_deref()) {
            CliDetectionStatus::Found(path) => path,
            CliDetectionStatus::NotFound => return Err(cli_not_found_error(config.provider)),
            CliDetectionStatus::UnsupportedPlatform => return Err(AiBackendError::UnsupportedPlatform),
        };
        start_stream_turn_with_binary(config, binary_path, messages, wake)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (config, messages, wake);
        Err(AiBackendError::UnsupportedPlatform)
    }
}

/// Starts a streaming turn against an already-resolved CLI binary path.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_stream_turn_with_binary(
    config: &AiRequestConfig,
    binary_path: PathBuf,
    messages: &[ChatMessage],
    wake: WakeCallback,
) -> Result<AiWorkerHandle, AiBackendError> {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    let provider = config.provider;
    let pruned = crate::state::prune_messages_for_context(messages, MAX_CLI_CONTEXT_TOKENS);
    let resume_id = if supports_session_resume(provider) {
        config.cli_session_id.clone().filter(|s| !s.trim().is_empty())
    } else {
        None
    };
    let mcp_options = if config.mcp_enabled
        && let Some(ref cli_path) = config.photocraft_cli_path
    {
        Some(CliMcpOptions {
            photocraft_cli_path: cli_path.clone(),
            auto_approve_tools: config.auto_approve_mcp_tools,
            server_env: build_photocraft_mcp_env(std::env::var("CRAFT_FLAGS").ok().as_deref()),
        })
    } else {
        None
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_worker = Arc::clone(&stop_flag);

    std::thread::Builder::new()
        .name("photocraft-cli-stream".to_string())
        .spawn(move || {
            let outcome = run_cli_stream_worker(
                CliStreamJob {
                    provider,
                    binary_path: &binary_path,
                    messages: &pruned,
                    resume_session_id: resume_id.as_deref(),
                    mcp_options: mcp_options.as_ref(),
                },
                &stop_flag_worker,
                &tx,
                &wake,
            );
            let _ = tx.send(crate::ai_backend::AiBackendEvent::StreamFinished(outcome));
            wake();
        })
        .map_err(|e| AiBackendError::WorkerSpawn(e.to_string()))?;

    Ok(AiWorkerHandle {
        rx,
        stop_flag,
        pending_function_calls: Vec::new(),
    })
}

/// Starts a background `--version` connectivity / environment test for a CLI provider.
pub fn start_test_connection(config: &AiRequestConfig, wake: WakeCallback) -> Result<AiWorkerHandle, AiBackendError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let binary_path = match detect_cli_with_override(config.provider, config.cli_binary_override.as_deref()) {
            CliDetectionStatus::Found(path) => path,
            CliDetectionStatus::NotFound => return Err(cli_not_found_error(config.provider)),
            CliDetectionStatus::UnsupportedPlatform => return Err(AiBackendError::UnsupportedPlatform),
        };
        start_test_connection_with_binary(config.provider, binary_path, wake)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (config, wake);
        Err(AiBackendError::UnsupportedPlatform)
    }
}

/// Starts a background `--version` check against an already-resolved CLI binary path.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_test_connection_with_binary(
    provider: AiProviderKind,
    binary_path: PathBuf,
    wake: WakeCallback,
) -> Result<AiWorkerHandle, AiBackendError> {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    let (tx, rx) = std::sync::mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_worker = Arc::clone(&stop_flag);

    std::thread::Builder::new()
        .name("photocraft-cli-test".to_string())
        .spawn(move || {
            let result = run_cli_version_check(provider, &binary_path, &stop_flag_worker);
            let _ = tx.send(crate::ai_backend::AiBackendEvent::TestResult(result));
            wake();
        })
        .map_err(|e| AiBackendError::WorkerSpawn(e.to_string()))?;

    Ok(AiWorkerHandle {
        rx,
        stop_flag,
        pending_function_calls: Vec::new(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn build_std_command(invocation: &CliInvocation) -> std::process::Command {
    use std::process::{Command, Stdio};

    let mut cmd = Command::new(&invocation.program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        if invocation.uses_windows_cmd_wrapper {
            if let Some((cmd_line, prefix_args)) = invocation.args.split_last() {
                cmd.args(prefix_args);
                cmd.raw_arg(cmd_line);
            }
        } else {
            cmd.args(&invocation.args);
        }
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.args(&invocation.args);
    }

    if let Some(ref dir) = invocation.working_dir {
        cmd.current_dir(dir);
    }

    if invocation.stdin_payload.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

#[cfg(not(target_arch = "wasm32"))]
fn terminate_child_process(child: &mut std::process::Child, uses_windows_cmd_wrapper: bool) {
    #[cfg(target_os = "windows")]
    {
        if uses_windows_cmd_wrapper {
            use std::os::windows::process::CommandExt;
            let pid = child.id().to_string();
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .status();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = uses_windows_cmd_wrapper;
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_stderr_reader(stderr: Option<std::process::ChildStderr>) -> Option<std::thread::JoinHandle<String>> {
    let mut stderr = stderr?;
    std::thread::Builder::new()
        .name("photocraft-cli-stderr".to_string())
        .spawn(move || {
            use std::io::Read;
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                let remaining = MAX_STDERR_BYTES.saturating_sub(buf.len());
                if remaining > 0
                    && let Some(slice) = chunk.get(..n.min(remaining))
                {
                    buf.extend_from_slice(slice);
                }
            }
            String::from_utf8_lossy(&buf).into_owned()
        })
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_stdin_writer(
    stdin: Option<std::process::ChildStdin>,
    payload: Option<String>,
) -> Option<std::thread::JoinHandle<()>> {
    let mut stdin = stdin?;
    let payload = payload?;
    std::thread::Builder::new()
        .name("photocraft-cli-stdin".to_string())
        .spawn(move || {
            use std::io::Write;
            let _ = stdin.write_all(payload.as_bytes());
            let _ = stdin.flush();
            drop(stdin);
        })
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_stdout_stream_reader(
    stdout: Option<std::process::ChildStdout>,
    provider: AiProviderKind,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    tx: std::sync::mpsc::Sender<crate::ai_backend::AiBackendEvent>,
    wake: WakeCallback,
) -> Option<std::thread::JoinHandle<(bool, Option<String>)>> {
    let stdout = stdout?;
    std::thread::Builder::new()
        .name("photocraft-cli-stdout".to_string())
        .spawn(move || {
            use std::io::{BufRead, BufReader};
            use std::sync::atomic::Ordering;

            let mut reader = BufReader::new(stdout);
            let mut parser = CliStreamParser::new(provider);
            let mut line = String::new();
            let mut last_sent_session_id: Option<String> = None;

            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                let parsed = parser.parse_line(&line);
                if let Some(session_id) = parsed.session_id
                    && last_sent_session_id.as_deref() != Some(session_id.as_str())
                {
                    last_sent_session_id = Some(session_id.clone());
                    let _ = tx.send(crate::ai_backend::AiBackendEvent::SessionId(session_id));
                    wake();
                }
                for tool_update in parsed.tool_updates {
                    let _ = tx.send(crate::ai_backend::AiBackendEvent::ToolUpdate(tool_update));
                    wake();
                }
                if let Some(delta_text) = parsed.delta_text
                    && !delta_text.is_empty()
                {
                    let _ = tx.send(crate::ai_backend::AiBackendEvent::StreamChunk {
                        delta_text,
                        function_calls: Vec::new(),
                    });
                    wake();
                }
            }

            (parser.emitted_any_text(), parser.last_error().map(str::to_string))
        })
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
struct CliStreamJob<'a> {
    provider: AiProviderKind,
    binary_path: &'a Path,
    messages: &'a [ChatMessage],
    resume_session_id: Option<&'a str>,
    mcp_options: Option<&'a CliMcpOptions>,
}

#[cfg(not(target_arch = "wasm32"))]
fn run_cli_stream_worker(
    job: CliStreamJob<'_>,
    stop_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    tx: &std::sync::mpsc::Sender<crate::ai_backend::AiBackendEvent>,
    wake: &WakeCallback,
) -> Result<(), AiBackendError> {
    use std::sync::atomic::Ordering;

    let CliStreamJob {
        provider,
        binary_path,
        messages,
        resume_session_id,
        mcp_options,
    } = job;

    if stop_flag.load(Ordering::Relaxed) {
        return Err(AiBackendError::Cancelled);
    }

    let (_mcp_guard, prepared_mcp) = match mcp_options {
        Some(opts) => {
            let (guard, mount) = TempMcpWorkspaceGuard::prepare(provider, opts).map_err(|err| {
                AiBackendError::CliExecutionFailed {
                    provider: provider.display_name(),
                    exit_code: None,
                    summary: format!("Failed to prepare temporary PhotoCraft MCP workspace: {err}"),
                }
            })?;
            (Some(guard), Some(mount))
        }
        None => (None, None),
    };

    let resuming = resume_session_id.is_some();
    let (_attachment_guard, attachments) = TempAttachmentGuard::materialize(messages, resuming);
    let prompt = build_cli_prompt_with_mcp(messages, resuming, &attachments, prepared_mcp.is_some());

    let Some(invocation) = build_cli_invocation_with_mcp(
        provider,
        binary_path,
        &prompt,
        resume_session_id,
        prepared_mcp.as_ref(),
        cfg!(target_os = "windows"),
    ) else {
        return Err(AiBackendError::CliExecutionFailed {
            provider: provider.display_name(),
            exit_code: None,
            summary: "Provider is not a local CLI backend.".to_string(),
        });
    };

    let mut child = build_std_command(&invocation).spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            cli_not_found_error(provider)
        } else {
            AiBackendError::CliExecutionFailed {
                provider: provider.display_name(),
                exit_code: None,
                summary: format!("Failed to start CLI process: {e}"),
            }
        }
    })?;

    let stdin_handle = spawn_stdin_writer(child.stdin.take(), invocation.stdin_payload.clone());
    let stderr_handle = spawn_stderr_reader(child.stderr.take());
    let stdout_handle = spawn_stdout_stream_reader(
        child.stdout.take(),
        provider,
        std::sync::Arc::clone(stop_flag),
        tx.clone(),
        std::sync::Arc::clone(wake),
    );

    let exit_status = loop {
        if stop_flag.load(Ordering::Relaxed) {
            terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
            return Err(AiBackendError::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(CANCEL_POLL_INTERVAL),
            Err(e) => {
                terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
                return Err(AiBackendError::CliExecutionFailed {
                    provider: provider.display_name(),
                    exit_code: None,
                    summary: format!("Failed while waiting for CLI process: {e}"),
                });
            }
        }
    };

    if let Some(handle) = stdin_handle {
        let _ = handle.join();
    }
    let (emitted_text, stream_error) = stdout_handle
        .and_then(|h| h.join().ok())
        .unwrap_or((false, None));
    let stderr_text = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();

    if stop_flag.load(Ordering::Relaxed) {
        return Err(AiBackendError::Cancelled);
    }

    if !exit_status.success() {
        return Err(classify_cli_error(
            provider,
            exit_status.code(),
            &stderr_text,
            stream_error.as_deref(),
        ));
    }

    if let Some(err) = stream_error
        && !emitted_text
    {
        return Err(classify_cli_error(provider, exit_status.code(), &stderr_text, Some(&err)));
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn run_cli_version_check(
    provider: AiProviderKind,
    binary_path: &Path,
    stop_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<String, AiBackendError> {
    use std::io::Read;
    use std::sync::atomic::Ordering;
    use std::time::Instant;

    let invocation = build_version_invocation(binary_path, cfg!(target_os = "windows"));
    let mut child = build_std_command(&invocation).spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            cli_not_found_error(provider)
        } else {
            AiBackendError::CliExecutionFailed {
                provider: provider.display_name(),
                exit_code: None,
                summary: format!("Failed to start CLI process: {e}"),
            }
        }
    })?;

    let stderr_handle = spawn_stderr_reader(child.stderr.take());
    let stdout_handle = child.stdout.take().and_then(|mut out| {
        std::thread::Builder::new()
            .name("photocraft-cli-ver-out".to_string())
            .spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                while let Ok(n) = out.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    let remaining = MAX_STDERR_BYTES.saturating_sub(buf.len());
                    if remaining > 0
                        && let Some(slice) = chunk.get(..n.min(remaining))
                    {
                        buf.extend_from_slice(slice);
                    }
                }
                String::from_utf8_lossy(&buf).into_owned()
            })
            .ok()
    });

    let started = Instant::now();
    let status = loop {
        if stop_flag.load(Ordering::Relaxed) {
            terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
            return Err(AiBackendError::Cancelled);
        }
        if started.elapsed() >= VERSION_CHECK_TIMEOUT {
            terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
            return Err(AiBackendError::CliExecutionFailed {
                provider: provider.display_name(),
                exit_code: None,
                summary: "CLI version check timed out.".to_string(),
            });
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(CANCEL_POLL_INTERVAL),
            Err(e) => {
                terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
                return Err(AiBackendError::CliExecutionFailed {
                    provider: provider.display_name(),
                    exit_code: None,
                    summary: format!("Failed while waiting for CLI process: {e}"),
                });
            }
        }
    };

    let stdout_text = stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stderr_text = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();

    if !status.success() {
        return Err(classify_cli_error(provider, status.code(), &stderr_text, None));
    }

    let version_line = strip_ansi_escapes(&stdout_text)
        .lines()
        .chain(strip_ansi_escapes(&stderr_text).lines())
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|s| truncate_chars(s, 80));

    let path_str = binary_path.display();
    Ok(match version_line {
        Some(ver) => format!("{} ready ({ver}) — {path_str}", provider.display_name()),
        None => format!("{} ready — {path_str}", provider.display_name()),
    })
}

/// Optional custom executable overrides supplied to a full CLI environment check.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliEnvCheckConfig {
    pub agy_path: String,
    pub gemini_path: String,
    pub claude_path: String,
    pub codex_path: String,
    pub photocraft_cli_path: String,
}

/// Detection and version probe result for one CLI binary in Preferences > Integrations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliEnvEntry {
    pub label: &'static str,
    pub executable: &'static str,
    pub status: CliDetectionStatus,
    pub version: Option<String>,
    pub custom_override_used: bool,
    pub error: Option<String>,
}

/// Complete report returned by [`start_cli_environment_check`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliEnvReport {
    pub entries: Vec<CliEnvEntry>,
}

/// Timeout per binary during a Preferences multi-CLI environment check.
#[cfg(not(target_arch = "wasm32"))]
const ENV_CHECK_BIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Probes `--version` on an arbitrary executable with a short timeout.
#[cfg(not(target_arch = "wasm32"))]
fn probe_binary_version(binary_path: &Path) -> Result<Option<String>, String> {
    use std::io::Read;
    use std::time::Instant;

    let invocation = build_version_invocation(binary_path, cfg!(target_os = "windows"));
    let mut child = build_std_command(&invocation)
        .spawn()
        .map_err(|e| format!("Failed to start executable: {e}"))?;

    let stderr_handle = spawn_stderr_reader(child.stderr.take());
    let stdout_handle = child.stdout.take().and_then(|mut out| {
        std::thread::Builder::new()
            .name("photocraft-cli-env-out".to_string())
            .spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                while let Ok(n) = out.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    let remaining = MAX_STDERR_BYTES.saturating_sub(buf.len());
                    if remaining > 0
                        && let Some(slice) = chunk.get(..n.min(remaining))
                    {
                        buf.extend_from_slice(slice);
                    }
                }
                String::from_utf8_lossy(&buf).into_owned()
            })
            .ok()
    });

    let started = Instant::now();
    let status = loop {
        if started.elapsed() >= ENV_CHECK_BIN_TIMEOUT {
            terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
            return Err("Version check timed out.".to_string());
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(CANCEL_POLL_INTERVAL),
            Err(e) => {
                terminate_child_process(&mut child, invocation.uses_windows_cmd_wrapper);
                return Err(format!("Failed while waiting for process: {e}"));
            }
        }
    };

    let stdout_text = stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stderr_text = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();

    let version_line = strip_ansi_escapes(&stdout_text)
        .lines()
        .chain(strip_ansi_escapes(&stderr_text).lines())
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|s| truncate_chars(s, 80));

    if !status.success() {
        return Err(summarize_stderr(&stderr_text, version_line.as_deref()));
    }

    Ok(version_line)
}

#[cfg(not(target_arch = "wasm32"))]
fn check_single_env_entry(
    label: &'static str,
    executable: &'static str,
    status: CliDetectionStatus,
    custom_override_used: bool,
) -> CliEnvEntry {
    match &status {
        CliDetectionStatus::Found(path) => match probe_binary_version(path) {
            Ok(version) => CliEnvEntry {
                label,
                executable,
                status,
                version,
                custom_override_used,
                error: None,
            },
            Err(err) => CliEnvEntry {
                label,
                executable,
                status,
                version: None,
                custom_override_used,
                error: Some(err),
            },
        },
        CliDetectionStatus::NotFound | CliDetectionStatus::UnsupportedPlatform => CliEnvEntry {
            label,
            executable,
            status,
            version: None,
            custom_override_used,
            error: None,
        },
    }
}

/// Runs a full CLI environment check (`agy`, `gemini`, `claude`, `codex`, `photocraft-cli`)
/// synchronously; called on a background worker thread by [`start_cli_environment_check`].
pub fn run_cli_environment_check(config: &CliEnvCheckConfig) -> CliEnvReport {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let items: [(&'static str, &'static str, CliDetectionStatus, bool); 5] = [
            (
                "Antigravity CLI",
                "agy",
                detect_cli_with_override(AiProviderKind::AntigravityCli, Some(&config.agy_path)),
                !config.agy_path.trim().is_empty(),
            ),
            (
                "Gemini CLI",
                "gemini",
                detect_cli_with_override(AiProviderKind::GeminiCli, Some(&config.gemini_path)),
                !config.gemini_path.trim().is_empty(),
            ),
            (
                "Claude CLI",
                "claude",
                detect_cli_with_override(AiProviderKind::ClaudeCli, Some(&config.claude_path)),
                !config.claude_path.trim().is_empty(),
            ),
            (
                "Codex CLI",
                "codex",
                detect_cli_with_override(AiProviderKind::CodexCli, Some(&config.codex_path)),
                !config.codex_path.trim().is_empty(),
            ),
            (
                "PhotoCraft CLI",
                "photocraft-cli",
                detect_photocraft_cli_with_override(Some(&config.photocraft_cli_path)),
                !config.photocraft_cli_path.trim().is_empty(),
            ),
        ];

        let entries = items
            .into_iter()
            .map(|(label, exe, status, custom)| check_single_env_entry(label, exe, status, custom))
            .collect();
        CliEnvReport { entries }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = config;
        let entries = [
            ("Antigravity CLI", "agy"),
            ("Gemini CLI", "gemini"),
            ("Claude CLI", "claude"),
            ("Codex CLI", "codex"),
            ("PhotoCraft CLI", "photocraft-cli"),
        ]
        .into_iter()
        .map(|(label, executable)| CliEnvEntry {
            label,
            executable,
            status: CliDetectionStatus::UnsupportedPlatform,
            version: None,
            custom_override_used: false,
            error: None,
        })
        .collect();
        CliEnvReport { entries }
    }
}

/// Spawns a non-blocking background thread to check all local AI CLIs and `photocraft-cli`,
/// invoking `wake` when the [`CliEnvReport`] is ready.
pub fn start_cli_environment_check(
    config: CliEnvCheckConfig,
    wake: WakeCallback,
) -> Result<std::sync::mpsc::Receiver<CliEnvReport>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::Builder::new()
            .name("photocraft-cli-env-check".to_string())
            .spawn(move || {
                let report = run_cli_environment_check(&config);
                let _ = tx.send(report);
                wake();
            })
            .map_err(|e| format!("Failed to start CLI environment check thread: {e}"))?;
        Ok(rx)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let report = run_cli_environment_check(&config);
        let _ = tx.send(report);
        wake();
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use super::{
        AttachmentNote, CliMcpOptions, CliStreamParser, CliToolState, attachment_extension, build_cli_invocation,
        build_cli_invocation_with_mcp, build_cli_prompt, build_cli_prompt_with_mcp, build_photocraft_mcp_env,
        build_version_invocation, candidate_executable_names, classify_cli_error, default_antigravity_bin_dir,
        format_toml_string, format_windows_cmd_line, is_windows_batch_script, resolve_cli_in_dirs,
        resolve_photocraft_cli, should_use_stdin_for_prompt, strip_ansi_escapes, summarize_stderr,
        supports_session_resume, write_attachment_to_dir, write_mcp_workspace_config,
    };
    use crate::ai_backend::{AiBackendError, AiProviderKind};
    use crate::state::{ChatImageAttachment, ChatMessage};

    #[test]
    fn antigravity_stream_json_parses_init_deltas_tool_updates_and_result() {
        let mut parser = CliStreamParser::new(AiProviderKind::AntigravityCli);

        let init = parser.parse_line(r#"{"event":"init","conversation_id":"conv-agy-001"}"#);
        assert_eq!(init.session_id.as_deref(), Some("conv-agy-001"));
        assert!(init.delta_text.is_none());

        let tool_active = parser.parse_line(
            r#"{"event":"step_update","step_update":{"step_index":2,"step_type":"tool","tool_name":"doc_inspect","state":"ACTIVE","tool_info":{"name":"doc_inspect","parameters":{"doc_id":1}}}}"#,
        );
        assert!(tool_active.delta_text.is_none());
        let tu_active = tool_active.first_tool_update().expect("active tool update");
        assert_eq!(tu_active.step_index, Some(2));
        assert_eq!(tu_active.correlation_key().as_deref(), Some("step:2"));
        assert_eq!(tu_active.tool_name, "doc_inspect");
        assert_eq!(tu_active.state, CliToolState::Active);
        assert_eq!(tu_active.parameters, json!({"doc_id": 1}));
        assert!(tu_active.output.is_none());
        assert!(tu_active.error.is_none());

        let tool_done = parser.parse_line(
            r#"{"event":"step_update","step_update":{"step_index":2,"step_type":"tool","tool_name":"doc_inspect","state":"DONE","tool_info":{"name":"doc_inspect","parameters":{"doc_id":1},"output":{"layers":3}}}}"#,
        );
        let tu_done = tool_done.first_tool_update().expect("done tool update");
        assert_eq!(tu_done.step_index, Some(2));
        assert_eq!(tu_done.state, CliToolState::Done);
        assert_eq!(tu_done.output, Some(json!({"layers": 3})));
        assert_eq!(parser.tool_updates().len(), 2);

        let tool_failed = parser.parse_line(
            r#"{"event":"step_update","step_update":{"step_index":3,"step_type":"tool","tool_name":"command_run","state":"DONE","tool_info":{"name":"command_run","parameters":{"id":"bad.cmd"},"error":"Unknown command"}}}"#,
        );
        let tu_failed = tool_failed.first_tool_update().expect("failed tool update");
        assert_eq!(tu_failed.step_index, Some(3));
        assert_eq!(tu_failed.state, CliToolState::Failed);
        assert_eq!(tu_failed.error.as_deref(), Some("Unknown command"));

        let d1 = parser.parse_line(
            r#"{"event":"step_update","step_update":{"step_type":"agent_response","text_delta":"Increase "}}"#,
        );
        assert_eq!(d1.delta_text.as_deref(), Some("Increase "));

        let d2 = parser.parse_line(
            r#"{"event":"step_update","step_update":{"step_type":"agent_response","text_delta":"exposure slightly."}}"#,
        );
        assert_eq!(d2.delta_text.as_deref(), Some("exposure slightly."));

        // Result containing full response must not duplicate streamed text_delta chunks.
        let res = parser.parse_line(
            r#"{"event":"result","result":{"conversation_id":"conv-agy-001","status":"success","response":"Increase exposure slightly."}}"#,
        );
        assert_eq!(res.session_id.as_deref(), Some("conv-agy-001"));
        assert!(res.delta_text.is_none());
        assert!(res.error.is_none());
        assert!(parser.emitted_any_text());
    }

    #[test]
    fn antigravity_result_fallback_response_and_auth_error_classification() {
        // When no text_delta preceded `result`, use `result.response`.
        let mut parser = CliStreamParser::new(AiProviderKind::AntigravityCli);
        let res = parser.parse_line(
            r#"{"event":"result","result":{"conversation_id":"conv-agy-002","status":"success","response":"Use Curves to lift midtones."}}"#,
        );
        assert_eq!(res.session_id.as_deref(), Some("conv-agy-002"));
        assert_eq!(res.delta_text.as_deref(), Some("Use Curves to lift midtones."));

        // Result carrying an error must surface error and classify authentication required -> CliNotAuthenticated.
        let mut err_parser = CliStreamParser::new(AiProviderKind::AntigravityCli);
        let err_line = err_parser.parse_line(
            r#"{"event":"result","result":{"conversation_id":"conv-agy-003","status":"error","error":"Authentication required: please sign in"}}"#,
        );
        let err_msg = err_line.error.as_deref().expect("parsed result error");
        assert!(err_msg.contains("Authentication required"));

        let classified = classify_cli_error(AiProviderKind::AntigravityCli, Some(1), "", Some(err_msg));
        match classified {
            AiBackendError::CliNotAuthenticated { provider, login_command, account_label, .. } => {
                assert_eq!(provider, "Antigravity CLI");
                assert_eq!(login_command, "agy");
                assert_eq!(account_label, "Google account");
            }
            other => panic!("expected CliNotAuthenticated, got {other:?}"),
        }
    }

    #[test]
    fn antigravity_command_assembly_with_conversation_resume_and_default_paths() {
        assert!(supports_session_resume(AiProviderKind::AntigravityCli));

        // First turn (no conversation_id yet)
        let first = build_cli_invocation(
            AiProviderKind::AntigravityCli,
            Path::new(r"C:\Users\me\AppData\Local\agy\bin\agy.exe"),
            "Analyze highlights",
            None,
            true,
        )
        .unwrap();
        assert_eq!(first.program, PathBuf::from(r"C:\Users\me\AppData\Local\agy\bin\agy.exe"));
        assert_eq!(first.args, vec!["-p", "Analyze highlights", "--output-format", "stream-json"]);
        assert!(first.stdin_payload.is_none());
        assert!(!first.uses_windows_cmd_wrapper);

        // Follow-up turn resumes via --conversation <id>
        let resumed = build_cli_invocation(
            AiProviderKind::AntigravityCli,
            Path::new("/home/me/.local/bin/agy"),
            "Now warm up the shadows",
            Some("conv-agy-777"),
            false,
        )
        .unwrap();
        assert_eq!(
            resumed.args,
            vec![
                "-p",
                "Now warm up the shadows",
                "--output-format",
                "stream-json",
                "--conversation",
                "conv-agy-777",
            ]
        );

        // Version check uses `agy --version`
        let ver = build_version_invocation(Path::new(r"C:\Users\me\AppData\Local\agy\bin\agy.exe"), true);
        assert_eq!(ver.args, vec!["--version"]);
        assert!(!ver.uses_windows_cmd_wrapper);

        // Default install paths on Windows and macOS/Linux
        assert_eq!(
            default_antigravity_bin_dir(Some(Path::new(r"C:\Users\me\AppData\Local")), None, true),
            Some(PathBuf::from(r"C:\Users\me\AppData\Local").join("agy").join("bin"))
        );
        assert_eq!(
            default_antigravity_bin_dir(None, Some(Path::new("/Users/me")), false),
            Some(PathBuf::from("/Users/me").join(".local").join("bin"))
        );

        // Detection resolves %LOCALAPPDATA%\agy\bin\agy.exe
        let tmp = tempfile::tempdir().unwrap();
        let agy_bin = default_antigravity_bin_dir(Some(tmp.path()), None, true).unwrap();
        std::fs::create_dir_all(&agy_bin).unwrap();
        let exe_path = agy_bin.join("agy.exe");
        std::fs::write(&exe_path, b"MZ").unwrap();
        assert_eq!(resolve_cli_in_dirs("agy", std::slice::from_ref(&agy_bin), true), Some(exe_path));
    }

    #[test]
    fn gemini_stream_json_parses_init_deltas_and_deduplicates_result() {
        let mut parser = CliStreamParser::new(AiProviderKind::GeminiCli);

        let init = parser.parse_line(r#"{"type":"init","session_id":"gem-123","model":"gemini-2.5-pro"}"#);
        assert_eq!(init.session_id.as_deref(), Some("gem-123"));
        assert!(init.delta_text.is_none());

        // Echoed user messages must be ignored.
        let echo = parser.parse_line(r#"{"type":"message","role":"user","content":"Boost contrast"}"#);
        assert!(echo.delta_text.is_none());

        let d1 = parser.parse_line(r#"{"type":"message","role":"assistant","content":"Open ","delta":true}"#);
        assert_eq!(d1.delta_text.as_deref(), Some("Open "));

        let d2 = parser.parse_line(r#"{"type":"message","role":"assistant","content":"Curves.","delta":true}"#);
        assert_eq!(d2.delta_text.as_deref(), Some("Curves."));

        // Repeated full response in result must not duplicate text.
        let res = parser.parse_line(r#"{"type":"result","status":"success","response":"Open Curves."}"#);
        assert!(res.delta_text.is_none());
        assert!(res.error.is_none());
        assert!(parser.emitted_any_text());
    }

    #[test]
    fn gemini_result_fallback_emits_when_no_deltas_preceded_it() {
        let mut parser = CliStreamParser::new(AiProviderKind::GeminiCli);
        let res = parser.parse_line(r#"{"type":"result","status":"success","response":"Use Levels (Ctrl+L)."}"#);
        assert_eq!(res.delta_text.as_deref(), Some("Use Levels (Ctrl+L)."));

        let warn = parser.parse_line(r#"{"type":"error","severity":"warning","message":"Minor deprecation"}"#);
        assert!(warn.error.is_none());

        let fatal = parser.parse_line(r#"{"type":"error","severity":"error","message":"Not logged in"}"#);
        assert_eq!(fatal.error.as_deref(), Some("Not logged in"));
        assert_eq!(parser.last_error(), Some("Not logged in"));
    }

    #[test]
    fn claude_stream_json_parses_init_deltas_assistant_and_errors() {
        let mut parser = CliStreamParser::new(AiProviderKind::ClaudeCli);

        let init = parser.parse_line(r#"{"type":"system","subtype":"init","session_id":"claude-uuid-42"}"#);
        assert_eq!(init.session_id.as_deref(), Some("claude-uuid-42"));
        assert!(init.delta_text.is_none());

        let d1 = parser.parse_line(
            r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"Apply "}}}"#,
        );
        assert_eq!(d1.delta_text.as_deref(), Some("Apply "));

        let d2 = parser.parse_line(
            r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"Unsharp Mask."}}}"#,
        );
        assert_eq!(d2.delta_text.as_deref(), Some("Unsharp Mask."));

        // Assistant summary repeating the streamed deltas must not duplicate.
        let asst = parser.parse_line(
            r#"{"type":"assistant","session_id":"claude-uuid-42","message":{"role":"assistant","content":[{"type":"text","text":"Apply Unsharp Mask."}]}}"#,
        );
        assert!(asst.delta_text.is_none());

        let res = parser.parse_line(
            r#"{"type":"result","subtype":"success","is_error":false,"result":"Apply Unsharp Mask.","session_id":"claude-uuid-42"}"#,
        );
        assert!(res.delta_text.is_none());

        // Separate session without stream_event deltas emits from assistant message.
        let mut parser2 = CliStreamParser::new(AiProviderKind::ClaudeCli);
        let asst2 = parser2.parse_line(
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Use a soft brush."}]}}"#,
        );
        assert_eq!(asst2.delta_text.as_deref(), Some("Use a soft brush."));

        let err_res = parser2.parse_line(r#"{"type":"result","subtype":"error","is_error":true,"result":"Please login first"}"#);
        assert_eq!(err_res.error.as_deref(), Some("Please login first"));
    }

    #[test]
    fn codex_stream_json_parses_thread_deltas_updates_and_ignores_reasoning() {
        let mut parser = CliStreamParser::new(AiProviderKind::CodexCli);

        let started = parser.parse_line(r#"{"type":"thread.started","thread_id":"019ce6ce-thread"}"#);
        assert_eq!(started.session_id.as_deref(), Some("019ce6ce-thread"));

        let reasoning = parser.parse_line(
            r#"{"type":"item.completed","item":{"id":"r1","type":"reasoning","text":"Internal chain of thought"}}"#,
        );
        assert!(reasoning.delta_text.is_none());

        let delta = parser.parse_line(r#"{"type":"item.agent_message.delta","item_id":"m1","delta":"Try "}"#);
        assert_eq!(delta.delta_text.as_deref(), Some("Try "));

        let updated = parser.parse_line(
            r#"{"type":"item.updated","item":{"id":"m1","type":"agent_message","text":"Try Exposure"}}"#,
        );
        assert_eq!(updated.delta_text.as_deref(), Some("Exposure"));

        let completed = parser.parse_line(
            r#"{"type":"item.completed","item":{"id":"m1","type":"agent_message","text":"Try Exposure +0.5 EV."}}"#,
        );
        assert_eq!(completed.delta_text.as_deref(), Some(" +0.5 EV."));

        let failed = parser.parse_line(r#"{"type":"turn.failed","error":{"message":"401 Unauthorized: run codex login"}}"#);
        assert_eq!(failed.error.as_deref(), Some("401 Unauthorized: run codex login"));
    }

    #[test]
    fn stream_parser_falls_back_safely_on_plain_text_ansi_and_unknown_json() {
        let mut parser = CliStreamParser::new(AiProviderKind::GeminiCli);

        let plain1 = parser.parse_line("\x1b[32mStep 1:\x1b[0m Duplicate the background layer.");
        assert_eq!(plain1.delta_text.as_deref(), Some("Step 1: Duplicate the background layer."));

        let plain2 = parser.parse_line("Step 2: Set blend mode to Soft Light.");
        assert_eq!(plain2.delta_text.as_deref(), Some("\nStep 2: Set blend mode to Soft Light."));

        // Future CLI schema with an unknown event type carrying a `text` field.
        let mut parser2 = CliStreamParser::new(AiProviderKind::CodexCli);
        let future = parser2.parse_line(r#"{"type":"assistant_chunk","text":"Future schema chunk"}"#);
        assert_eq!(future.delta_text.as_deref(), Some("Future schema chunk"));

        // Metadata events with unknown type must not leak as assistant text.
        let meta = parser2.parse_line(r#"{"type":"usage_stats","text":"ignored"}"#);
        assert!(meta.delta_text.is_none());

        assert_eq!(strip_ansi_escapes("\x1b[1;31mError\x1b[0m text"), "Error text");
    }

    #[test]
    fn stderr_classification_distinguishes_auth_and_general_failures() {
        let auth_cases = [
            (AiProviderKind::GeminiCli, "Error: OAuth token expired. Please sign in again."),
            (AiProviderKind::ClaudeCli, "Not logged in · Please run /login"),
            (AiProviderKind::CodexCli, "HTTP 401 Unauthorized: authentication required, run codex login"),
            (AiProviderKind::GeminiCli, "invalid_grant: re-authenticate with Google account"),
        ];

        for (provider, stderr) in auth_cases {
            let err = classify_cli_error(provider, Some(1), stderr, None);
            assert!(
                matches!(err, AiBackendError::CliNotAuthenticated { .. }),
                "expected CliNotAuthenticated for {provider:?} stderr={stderr:?}, got {err:?}"
            );
            let msg = err.to_string();
            assert!(msg.contains(provider.cli_login_command().unwrap()));
            assert!(msg.contains("no API key needed"));
        }

        let general = classify_cli_error(
            AiProviderKind::ClaudeCli,
            Some(2),
            "(node:1234) DeprecationWarning: punycode\nError: Network timeout while connecting",
            None,
        );
        match general {
            AiBackendError::CliExecutionFailed { provider, exit_code, summary } => {
                assert_eq!(provider, "Claude CLI");
                assert_eq!(exit_code, Some(2));
                assert!(summary.contains("Network timeout"));
                assert!(!summary.contains("DeprecationWarning"));
            }
            other => panic!("expected CliExecutionFailed, got {other:?}"),
        }

        assert_eq!(summarize_stderr("   \n  ", None), "Process exited without error details.");
    }

    #[test]
    fn command_assembly_for_gemini_claude_and_codex() {
        // Gemini CLI direct invocation on Unix
        let gem = build_cli_invocation(
            AiProviderKind::GeminiCli,
            Path::new("/usr/local/bin/gemini"),
            "Explain Curves",
            None,
            false,
        )
        .unwrap();
        assert_eq!(gem.program, PathBuf::from("/usr/local/bin/gemini"));
        assert_eq!(gem.args, vec!["-p", "Explain Curves", "--output-format", "stream-json"]);
        assert!(gem.stdin_payload.is_none());
        assert!(!gem.uses_windows_cmd_wrapper);

        // Claude CLI with --resume on Windows .exe
        let claude = build_cli_invocation(
            AiProviderKind::ClaudeCli,
            Path::new(r"C:\Users\me\.local\bin\claude.exe"),
            "Next step?",
            Some("sess-99"),
            true,
        )
        .unwrap();
        assert_eq!(claude.program, PathBuf::from(r"C:\Users\me\.local\bin\claude.exe"));
        assert_eq!(
            claude.args,
            vec!["-p", "Next step?", "--output-format", "stream-json", "--verbose", "--resume", "sess-99"]
        );
        assert!(!claude.uses_windows_cmd_wrapper);
        assert!(supports_session_resume(AiProviderKind::ClaudeCli));
        assert!(!supports_session_resume(AiProviderKind::GeminiCli));

        // Codex CLI direct single-line invocation on Unix
        let codex = build_cli_invocation(
            AiProviderKind::CodexCli,
            Path::new("/usr/local/bin/codex"),
            "Explain Dodge and Burn",
            None,
            false,
        )
        .unwrap();
        assert_eq!(codex.args, vec!["exec", "--json", "--skip-git-repo-check", "Explain Dodge and Burn"]);
        assert!(codex.stdin_payload.is_none());
    }

    #[test]
    fn windows_cmd_batch_wrapping_and_quoting() {
        assert!(is_windows_batch_script(Path::new(r"C:\Users\me\AppData\Roaming\npm\gemini.cmd")));
        assert!(is_windows_batch_script(Path::new("claude.BAT")));
        assert!(!is_windows_batch_script(Path::new("claude.exe")));
        assert!(!is_windows_batch_script(Path::new("/usr/local/bin/claude")));

        // Codex CLI on Windows .cmd with multi-line prompt -> wrapped via cmd.exe /D /S /C + stdin
        let codex = build_cli_invocation(
            AiProviderKind::CodexCli,
            Path::new(r"C:\Users\Alice Smith\AppData\Roaming\npm\codex.cmd"),
            "Line 1\nLine 2 with %PATH% & | quotes",
            None,
            true,
        )
        .unwrap();
        assert_eq!(codex.program, PathBuf::from("cmd.exe"));
        assert!(codex.uses_windows_cmd_wrapper);
        assert_eq!(codex.args[..3], ["/D", "/S", "/C"]);
        assert!(codex.args[3].starts_with("\"\"C:\\Users\\Alice Smith\\AppData\\Roaming\\npm\\codex.cmd\""));
        assert!(codex.args[3].contains("\"exec\" \"--json\" \"--skip-git-repo-check\" \"-\""));
        assert_eq!(codex.stdin_payload.as_deref(), Some("Line 1\nLine 2 with %PATH% & | quotes"));

        // Windows .cmd version check wrapping
        let ver = build_version_invocation(Path::new(r"C:\npm\gemini.cmd"), true);
        assert_eq!(ver.program, PathBuf::from("cmd.exe"));
        assert!(ver.uses_windows_cmd_wrapper);
        assert_eq!(ver.args[3], r#"""C:\npm\gemini.cmd" "--version"""#);

        // Windows cmd line quoting escapes embedded quotes and strips newlines
        let formatted = format_windows_cmd_line(Path::new(r"C:\dir\tool.cmd"), &[r#"say "hi""#.into(), "a\nb".into()]);
        assert!(!formatted.contains('\n'));
        assert!(formatted.contains(r#"\"hi\""#));

        assert!(should_use_stdin_for_prompt("line1\nline2", false));
        assert!(should_use_stdin_for_prompt("-leading-dash", false));
        assert!(should_use_stdin_for_prompt("has %ENV%", true));
        assert!(!should_use_stdin_for_prompt("simple prompt", true));
    }

    #[test]
    fn windows_candidate_order_prefers_cmd_over_extensionless_npm_shim() {
        let win_names = candidate_executable_names("gemini", true);
        assert_eq!(win_names, vec!["gemini.exe", "gemini.cmd", "gemini.bat", "gemini.com"]);
        assert_eq!(candidate_executable_names("gemini", false), vec!["gemini"]);

        let tmp = tempfile::tempdir().unwrap();
        let npm_dir = tmp.path().join("npm");
        std::fs::create_dir_all(&npm_dir).unwrap();
        // Create both extensionless POSIX shim and Windows .cmd wrapper
        std::fs::write(npm_dir.join("gemini"), "#!/bin/sh\n").unwrap();
        std::fs::write(npm_dir.join("gemini.cmd"), "@echo off\r\n").unwrap();

        let resolved_win = resolve_cli_in_dirs("gemini", std::slice::from_ref(&npm_dir), true);
        assert_eq!(resolved_win, Some(npm_dir.join("gemini.cmd")));

        // .exe takes priority over .cmd in the same folder
        std::fs::write(npm_dir.join("gemini.exe"), b"MZ").unwrap();
        let resolved_exe = resolve_cli_in_dirs("gemini", &[npm_dir], true);
        assert_eq!(resolved_exe, Some(tmp.path().join("npm").join("gemini.exe")));
    }

    #[test]
    fn multi_turn_prompt_builder_and_temp_attachment_writing() {
        let tmp = tempfile::tempdir().unwrap();
        let att = ChatImageAttachment {
            mime_type: "image/png".to_string(),
            base64_data: "AQIDBA==".to_string(),
            width: 800,
            height: 600,
            label: Some("Canvas: Portrait.psd".to_string()),
        };

        assert_eq!(attachment_extension("image/png"), "png");
        assert_eq!(attachment_extension("image/jpeg"), "jpg");
        assert_eq!(attachment_extension("image/webp"), "webp");

        let note = write_attachment_to_dir(tmp.path(), 2, &att, "canvas-test-1").unwrap();
        assert!(note.file_path.exists());
        assert_eq!(note.file_path.extension().and_then(|e| e.to_str()), Some("png"));
        assert_eq!(std::fs::read(&note.file_path).unwrap(), vec![1, 2, 3, 4]);

        let messages = vec![
            ChatMessage::user("How do I fix a cool color cast?"),
            ChatMessage::model("Warm up the Temperature slider or add a Color Balance adjustment."),
            ChatMessage::user_with_image("Check this updated canvas.", att),
        ];

        // Non-resuming multi-turn prompt carries history + current request + attachment path.
        let full_prompt = build_cli_prompt(&messages, false, std::slice::from_ref(&note));
        assert!(full_prompt.contains("Conversation history:"));
        assert!(full_prompt.contains("User: How do I fix a cool color cast?"));
        assert!(full_prompt.contains("Assistant: Warm up the Temperature slider"));
        assert!(full_prompt.contains("Current user request:"));
        assert!(full_prompt.contains("Check this updated canvas."));
        assert!(full_prompt.contains("800x600 px (image/png)"));
        assert!(full_prompt.contains(&note.file_path.display().to_string()));

        // Resuming session prompt sends only the latest user turn + its attachment.
        let resume_prompt = build_cli_prompt(&messages, true, &[AttachmentNote { ..note.clone() }]);
        assert!(!resume_prompt.contains("Conversation history:"));
        assert!(resume_prompt.contains("Check this updated canvas."));
        assert!(resume_prompt.contains(&note.file_path.display().to_string()));

        // MCP-enabled prompt explicitly instructs the CLI to use live `photocraft` MCP tools.
        let mcp_prompt = build_cli_prompt_with_mcp(&messages, false, std::slice::from_ref(&note), true);
        assert!(mcp_prompt.contains("photocraft"));
        assert!(mcp_prompt.contains("doc_inspect"));
        assert!(mcp_prompt.contains("command_run"));
        assert!(mcp_prompt.contains("doc_render_preview"));
    }

    #[test]
    fn photocraft_cli_resolver_prefers_sibling_dir_over_path_and_handles_windows_exe() {
        let tmp = tempfile::tempdir().unwrap();
        let app_dir = tmp.path().join("dist").join("release");
        let path_dir = tmp.path().join("path-bin");
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(&path_dir).unwrap();

        let current_exe = app_dir.join("photocraft.exe");
        let path_cli = path_dir.join("photocraft-cli.exe");
        std::fs::write(&path_cli, b"MZ").unwrap();

        // Fallback to PATH when sibling binary is absent.
        assert_eq!(
            resolve_photocraft_cli(Some(&current_exe), std::slice::from_ref(&path_dir), true),
            Some(path_cli.clone())
        );

        // Sibling binary alongside `photocraft.exe` takes priority over PATH.
        let sibling_cli = app_dir.join("photocraft-cli.exe");
        std::fs::write(&sibling_cli, b"MZ").unwrap();
        assert_eq!(
            resolve_photocraft_cli(Some(&current_exe), std::slice::from_ref(&path_dir), true),
            Some(sibling_cli)
        );

        // Absent everywhere returns None.
        let empty_dir = tmp.path().join("empty");
        std::fs::create_dir_all(&empty_dir).unwrap();
        assert_eq!(
            resolve_photocraft_cli(Some(&empty_dir.join("photocraft.exe")), &[empty_dir], true),
            None
        );
    }

    #[test]
    fn temporary_mcp_configs_and_approval_flags_for_all_four_cli_providers() {
        let tmp = tempfile::tempdir().unwrap();
        let cli_bin = PathBuf::from(r"F:\PhotoCraft\dist\release\photocraft-cli.exe");
        let env = build_photocraft_mcp_env(Some("AI-MODELS"));
        assert_eq!(env, vec![("CRAFT_FLAGS".to_string(), "AI-MODELS,AI-MCP".to_string())]);
        assert_eq!(build_photocraft_mcp_env(None), vec![("CRAFT_FLAGS".to_string(), "AI-MCP".to_string())]);
        assert_eq!(format_toml_string(r#"C:\a\b "c""#), r#""C:\\a\\b \"c\"""#);

        // 1. Antigravity CLI: `.agents/mcp_config.json`, current_dir = workspace, opt-in --dangerously-skip-permissions
        let agy_ws = tmp.path().join("agy-ws");
        let agy_opts_off = CliMcpOptions {
            photocraft_cli_path: cli_bin.clone(),
            auto_approve_tools: false,
            server_env: env.clone(),
        };
        let agy_mount_off = write_mcp_workspace_config(AiProviderKind::AntigravityCli, &agy_ws, &agy_opts_off).unwrap();
        let agy_cfg_path = agy_ws.join(".agents").join("mcp_config.json");
        assert_eq!(agy_mount_off.config_file.as_deref(), Some(agy_cfg_path.as_path()));
        let agy_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&agy_cfg_path).unwrap()).unwrap();
        assert_eq!(agy_json["mcpServers"]["photocraft"]["command"], cli_bin.to_string_lossy().as_ref());
        assert_eq!(agy_json["mcpServers"]["photocraft"]["args"], json!(["mcp"]));
        assert_eq!(agy_json["mcpServers"]["photocraft"]["env"]["CRAFT_FLAGS"], "AI-MODELS,AI-MCP");

        let agy_inv_off = build_cli_invocation_with_mcp(
            AiProviderKind::AntigravityCli,
            Path::new(r"C:\agy\agy.exe"),
            "Inspect doc",
            None,
            Some(&agy_mount_off),
            true,
        )
        .unwrap();
        assert_eq!(agy_inv_off.working_dir.as_deref(), Some(agy_ws.as_path()));
        assert!(!agy_inv_off.args.contains(&"--dangerously-skip-permissions".to_string()));

        let mut agy_mount_on = agy_mount_off.clone();
        agy_mount_on.auto_approve_tools = true;
        let agy_inv_on = build_cli_invocation_with_mcp(
            AiProviderKind::AntigravityCli,
            Path::new(r"C:\agy\agy.exe"),
            "Inspect doc",
            Some("conv-1"),
            Some(&agy_mount_on),
            true,
        )
        .unwrap();
        assert!(agy_inv_on.args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(agy_inv_on.args.contains(&"--conversation".to_string()));

        // 2. Claude CLI: `mcp-config.json` + `--mcp-config <path>`, opt-in `--allowedTools mcp__photocraft__*`
        let claude_ws = tmp.path().join("claude-ws");
        let claude_opts = CliMcpOptions {
            photocraft_cli_path: cli_bin.clone(),
            auto_approve_tools: true,
            server_env: env.clone(),
        };
        let claude_mount = write_mcp_workspace_config(AiProviderKind::ClaudeCli, &claude_ws, &claude_opts).unwrap();
        let claude_cfg_path = claude_ws.join("mcp-config.json");
        assert_eq!(claude_mount.config_file.as_deref(), Some(claude_cfg_path.as_path()));
        let claude_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&claude_cfg_path).unwrap()).unwrap();
        assert_eq!(claude_json["mcpServers"]["photocraft"]["args"], json!(["mcp"]));

        let claude_inv_on = build_cli_invocation_with_mcp(
            AiProviderKind::ClaudeCli,
            Path::new("/usr/local/bin/claude"),
            "List layers",
            None,
            Some(&claude_mount),
            false,
        )
        .unwrap();
        assert!(claude_inv_on.args.windows(2).any(|w| w[0] == "--mcp-config" && w[1] == claude_cfg_path.to_string_lossy()));
        assert!(claude_inv_on.args.windows(2).any(|w| w[0] == "--allowedTools" && w[1] == "mcp__photocraft__*"));

        let mut claude_mount_off = claude_mount.clone();
        claude_mount_off.auto_approve_tools = false;
        let claude_inv_off = build_cli_invocation_with_mcp(
            AiProviderKind::ClaudeCli,
            Path::new("/usr/local/bin/claude"),
            "List layers",
            None,
            Some(&claude_mount_off),
            false,
        )
        .unwrap();
        assert!(claude_inv_off.args.contains(&"--mcp-config".to_string()));
        assert!(!claude_inv_off.args.contains(&"--allowedTools".to_string()));

        // 3. Gemini CLI: `.gemini/settings.json`, current_dir = workspace, opt-in `--approval-mode yolo`
        let gemini_ws = tmp.path().join("gemini-ws");
        let gemini_opts = CliMcpOptions {
            photocraft_cli_path: cli_bin.clone(),
            auto_approve_tools: true,
            server_env: env.clone(),
        };
        let gemini_mount = write_mcp_workspace_config(AiProviderKind::GeminiCli, &gemini_ws, &gemini_opts).unwrap();
        let gemini_cfg_path = gemini_ws.join(".gemini").join("settings.json");
        assert_eq!(gemini_mount.config_file.as_deref(), Some(gemini_cfg_path.as_path()));
        let gemini_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&gemini_cfg_path).unwrap()).unwrap();
        assert_eq!(gemini_json["mcpServers"]["photocraft"]["args"], json!(["mcp"]));

        let gemini_inv_on = build_cli_invocation_with_mcp(
            AiProviderKind::GeminiCli,
            Path::new("/usr/local/bin/gemini"),
            "Brighten",
            None,
            Some(&gemini_mount),
            false,
        )
        .unwrap();
        assert_eq!(gemini_inv_on.working_dir.as_deref(), Some(gemini_ws.as_path()));
        assert!(gemini_inv_on.args.windows(2).any(|w| w[0] == "--approval-mode" && w[1] == "yolo"));

        let mut gemini_mount_off = gemini_mount.clone();
        gemini_mount_off.auto_approve_tools = false;
        let gemini_inv_off = build_cli_invocation_with_mcp(
            AiProviderKind::GeminiCli,
            Path::new("/usr/local/bin/gemini"),
            "Brighten",
            None,
            Some(&gemini_mount_off),
            false,
        )
        .unwrap();
        assert!(!gemini_inv_off.args.contains(&"--approval-mode".to_string()));

        // 4. Codex CLI: `-c mcp_servers.photocraft.command=...` and `args=["mcp"]`, opt-in `--full-auto`
        let codex_ws = tmp.path().join("codex-ws");
        let codex_opts = CliMcpOptions {
            photocraft_cli_path: cli_bin.clone(),
            auto_approve_tools: false,
            server_env: env,
        };
        let codex_mount_off = write_mcp_workspace_config(AiProviderKind::CodexCli, &codex_ws, &codex_opts).unwrap();
        assert!(codex_mount_off.config_file.is_none());

        let codex_inv_off = build_cli_invocation_with_mcp(
            AiProviderKind::CodexCli,
            Path::new("/usr/local/bin/codex"),
            "Check histogram",
            None,
            Some(&codex_mount_off),
            false,
        )
        .unwrap();
        assert!(
            codex_inv_off
                .args
                .iter()
                .any(|a| a.starts_with("mcp_servers.photocraft.command=") && a.contains("photocraft-cli.exe"))
        );
        assert!(codex_inv_off.args.contains(&"mcp_servers.photocraft.args=[\"mcp\"]".to_string()));
        assert!(!codex_inv_off.args.contains(&"--full-auto".to_string()));

        let mut codex_mount_on = codex_mount_off;
        codex_mount_on.auto_approve_tools = true;
        let codex_inv_on = build_cli_invocation_with_mcp(
            AiProviderKind::CodexCli,
            Path::new("/usr/local/bin/codex"),
            "Check histogram",
            None,
            Some(&codex_mount_on),
            false,
        )
        .unwrap();
        assert!(codex_inv_on.args.contains(&"--full-auto".to_string()));
    }

    #[test]
    fn claude_stream_tool_updates_parse_active_done_and_failed() {
        let mut claude = CliStreamParser::new(AiProviderKind::ClaudeCli);
        let use_line = claude.parse_line(
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Checking document."},{"type":"tool_use","id":"toolu_1","name":"mcp__photocraft__doc_inspect","input":{"layers":true}},{"type":"tool_use","id":"toolu_2","name":"mcp__photocraft__command_run","input":{"id":"bad.cmd"}}]}}"#,
        );
        assert_eq!(use_line.delta_text.as_deref(), Some("Checking document."));
        assert_eq!(use_line.tool_updates.len(), 2);
        assert_eq!(use_line.tool_updates[0].call_id.as_deref(), Some("toolu_1"));
        assert_eq!(use_line.tool_updates[0].tool_name, "mcp__photocraft__doc_inspect");
        assert_eq!(use_line.tool_updates[0].state, CliToolState::Active);
        assert_eq!(use_line.tool_updates[0].parameters, json!({"layers": true}));

        let res_line = claude.parse_line(
            r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_1","content":"{\"width\":800}","is_error":false},{"type":"tool_result","tool_use_id":"toolu_2","content":"Command not found","is_error":true}]}}"#,
        );
        assert_eq!(res_line.tool_updates.len(), 2);
        assert_eq!(res_line.tool_updates[0].call_id.as_deref(), Some("toolu_1"));
        assert_eq!(res_line.tool_updates[0].tool_name, "mcp__photocraft__doc_inspect");
        assert_eq!(res_line.tool_updates[0].state, CliToolState::Done);
        assert_eq!(res_line.tool_updates[0].output, Some(json!("{\"width\":800}")));
        assert_eq!(res_line.tool_updates[1].call_id.as_deref(), Some("toolu_2"));
        assert_eq!(res_line.tool_updates[1].tool_name, "mcp__photocraft__command_run");
        assert_eq!(res_line.tool_updates[1].state, CliToolState::Failed);
        assert_eq!(res_line.tool_updates[1].error.as_deref(), Some("Command not found"));
    }

    #[test]
    fn gemini_stream_tool_updates_parse_active_done_and_failed() {
        let mut gemini = CliStreamParser::new(AiProviderKind::GeminiCli);
        let g_use = gemini.parse_line(
            r#"{"type":"tool_use","tool_id":"gem_call_1","tool_name":"doc_inspect","parameters":{"verbose":false}}"#,
        );
        let gu = g_use.first_tool_update().expect("gemini tool_use");
        assert_eq!(gu.call_id.as_deref(), Some("gem_call_1"));
        assert_eq!(gu.tool_name, "doc_inspect");
        assert_eq!(gu.state, CliToolState::Active);
        assert_eq!(gu.parameters, json!({"verbose": false}));

        let g_ok = gemini.parse_line(
            r#"{"type":"tool_result","tool_id":"gem_call_1","status":"success","output":{"width":1920}}"#,
        );
        let go = g_ok.first_tool_update().expect("gemini tool_result ok");
        assert_eq!(go.call_id.as_deref(), Some("gem_call_1"));
        assert_eq!(go.tool_name, "doc_inspect");
        assert_eq!(go.state, CliToolState::Done);
        assert_eq!(go.output, Some(json!({"width": 1920})));

        let g_err = gemini.parse_line(
            r#"{"type":"tool_result","tool_id":"gem_call_2","tool_name":"command_run","status":"error","error":"Permission denied"}"#,
        );
        let ge = g_err.first_tool_update().expect("gemini tool_result err");
        assert_eq!(ge.state, CliToolState::Failed);
        assert_eq!(ge.error.as_deref(), Some("Permission denied"));
    }

    #[test]
    fn codex_stream_tool_updates_parse_active_done_and_failed() {
        let mut codex = CliStreamParser::new(AiProviderKind::CodexCli);
        let c_start = codex.parse_line(
            r#"{"type":"item.started","item":{"id":"item_9","type":"mcp_tool_call","server":"photocraft","tool":"doc_render_preview","arguments":{"max_edge":512},"status":"in_progress"}}"#,
        );
        let cs = c_start.first_tool_update().expect("codex item.started");
        assert_eq!(cs.call_id.as_deref(), Some("item_9"));
        assert_eq!(cs.tool_name, "doc_render_preview");
        assert_eq!(cs.state, CliToolState::Active);
        assert_eq!(cs.parameters, json!({"max_edge": 512}));

        let c_done = codex.parse_line(
            r#"{"type":"item.completed","item":{"id":"item_9","type":"mcp_tool_call","server":"photocraft","tool":"doc_render_preview","arguments":{"max_edge":512},"result":{"width":512,"height":384},"status":"completed"}}"#,
        );
        let cd = c_done.first_tool_update().expect("codex item.completed");
        assert_eq!(cd.call_id.as_deref(), Some("item_9"));
        assert_eq!(cd.state, CliToolState::Done);
        assert_eq!(cd.output, Some(json!({"width": 512, "height": 384})));

        let c_fail = codex.parse_line(
            r#"{"type":"item.completed","item":{"id":"item_10","type":"tool_call","name":"command_run","arguments":{"id":"x"},"error":{"message":"No active document"},"status":"failed"}}"#,
        );
        let cf = c_fail.first_tool_update().expect("codex item failed");
        assert_eq!(cf.call_id.as_deref(), Some("item_10"));
        assert_eq!(cf.tool_name, "command_run");
        assert_eq!(cf.state, CliToolState::Failed);
        assert_eq!(cf.error.as_deref(), Some("No active document"));
    }

    #[test]
    fn custom_executable_override_takes_priority_over_auto_detect() {
        let tmp = tempfile::tempdir().unwrap();
        let auto_dir = tmp.path().join("auto");
        let custom_dir = tmp.path().join("custom");
        std::fs::create_dir_all(&auto_dir).unwrap();
        std::fs::create_dir_all(&custom_dir).unwrap();

        let auto_agy = auto_dir.join("agy.exe");
        let custom_agy = custom_dir.join("my-agy.exe");
        let custom_dir_agy = custom_dir.join("agy.cmd");
        std::fs::write(&auto_agy, b"MZ").unwrap();
        std::fs::write(&custom_agy, b"MZ").unwrap();
        std::fs::write(&custom_dir_agy, "@echo off\r\n").unwrap();

        // Blank override falls back to auto-detection in search_dirs.
        assert_eq!(
            super::resolve_cli_with_override("agy", Some("   "), std::slice::from_ref(&auto_dir), true),
            Some(auto_agy.clone())
        );

        // Explicit file override takes precedence over search_dirs.
        assert_eq!(
            super::resolve_cli_with_override(
                "agy",
                Some(&custom_agy.display().to_string()),
                std::slice::from_ref(&auto_dir),
                true,
            ),
            Some(custom_agy.clone())
        );

        // Directory override resolves candidate executable inside that folder.
        assert_eq!(
            super::resolve_cli_with_override(
                "agy",
                Some(&custom_dir.display().to_string()),
                std::slice::from_ref(&auto_dir),
                true,
            ),
            Some(custom_dir_agy)
        );

        // Non-existent explicit override does NOT silently fall back to PATH.
        let missing = custom_dir.join("missing-agy.exe");
        assert_eq!(
            super::resolve_cli_with_override(
                "agy",
                Some(&missing.display().to_string()),
                std::slice::from_ref(&auto_dir),
                true,
            ),
            None
        );

        // Same override priority for photocraft-cli.
        let custom_pc_cli = custom_dir.join("custom-photocraft-cli.exe");
        std::fs::write(&custom_pc_cli, b"MZ").unwrap();
        assert_eq!(
            super::resolve_photocraft_cli_with_override(
                Some(&custom_pc_cli.display().to_string()),
                Some(&auto_dir.join("photocraft.exe")),
                std::slice::from_ref(&auto_dir),
                true,
            ),
            Some(custom_pc_cli)
        );
    }

    #[test]
    fn async_cli_environment_check_returns_all_five_entries() {
        let missing = tempfile::tempdir().unwrap().path().join("definitely-missing-bin");
        let missing_str = missing.display().to_string();
        let cfg = super::CliEnvCheckConfig {
            agy_path: missing_str.clone(),
            gemini_path: missing_str.clone(),
            claude_path: missing_str.clone(),
            codex_path: missing_str.clone(),
            photocraft_cli_path: missing_str,
        };
        let woke = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let woke_flag = std::sync::Arc::clone(&woke);
        let rx = super::start_cli_environment_check(
            cfg,
            std::sync::Arc::new(move || {
                woke_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        )
        .expect("spawn env check");

        let report = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("receive env check report");
        assert_eq!(report.entries.len(), 5);
        assert_eq!(report.entries[0].executable, "agy");
        assert_eq!(report.entries[1].executable, "gemini");
        assert_eq!(report.entries[2].executable, "claude");
        assert_eq!(report.entries[3].executable, "codex");
        assert_eq!(report.entries[4].executable, "photocraft-cli");
        for entry in &report.entries {
            assert!(entry.custom_override_used);
            assert_eq!(entry.status, super::CliDetectionStatus::NotFound);
        }
    }
}
