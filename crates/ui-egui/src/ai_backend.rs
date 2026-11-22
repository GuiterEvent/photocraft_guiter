//! Unified AI backend abstraction for PhotoCraft's built-in multi-AI sidebar.
//!
//! Dispatches streaming chat turns and connectivity checks across supported
//! providers:
//! - [`AiProviderKind::AntigravityCli`] (Google account login via `agy`, no API credits)
//! - [`AiProviderKind::GeminiCli`] (Google account login, no API credits)
//! - [`AiProviderKind::ClaudeCli`] (Claude subscription, no API credits)
//! - [`AiProviderKind::CodexCli`] (ChatGPT subscription, no API credits)
//! - [`AiProviderKind::GeminiApi`] (Google Gemini API key mode)
//!
//! [`AiProviderKind::GeminiApi`] executes HTTP/SSE requests through
//! [`crate::gemini::GeminiClient`], while the local CLI providers execute
//! non-interactive streaming subprocesses through [`crate::cli_agent`].

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

use photocraft_engine::prefs::AiProvider;
use serde::{Deserialize, Serialize};

use crate::gemini::{FunctionCall, GeminiError};
use crate::state::ChatMessage;

/// Maximum estimated token budget kept in conversation history before pruning.
pub const MAX_CONTEXT_TOKENS: usize = 32_000;

/// Supported AI backend providers for the built-in AI assistant sidebar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderKind {
    /// Local Antigravity CLI (`agy`) authenticated via Google account (no API credits).
    AntigravityCli,
    /// Local Gemini CLI authenticated via Google account (no API credits).
    GeminiCli,
    /// Local Claude CLI authenticated via Claude subscription (no API credits).
    ClaudeCli,
    /// Local Codex CLI authenticated via ChatGPT subscription (no API credits).
    CodexCli,
    /// Direct Google Generative Language API using a Gemini API key.
    #[default]
    GeminiApi,
}

impl AiProviderKind {
    /// All providers in dropdown display order (`AntigravityCli` first).
    pub const ALL: [Self; 5] = [
        Self::AntigravityCli,
        Self::GeminiCli,
        Self::ClaudeCli,
        Self::CodexCli,
        Self::GeminiApi,
    ];

    /// Preference JSON value corresponding to this provider (`integrations.aiProvider`).
    pub fn pref_id(self) -> &'static str {
        AiProvider::from(self).name()
    }

    /// Parses a stored preference string into an [`AiProviderKind`].
    pub fn from_pref_id(id: &str) -> Option<Self> {
        AiProvider::parse(id).map(Self::from)
    }

    /// Short English display name (also used as `tl!` key and closed dropdown face).
    pub fn display_name(self) -> &'static str {
        match self {
            Self::AntigravityCli => "Antigravity CLI",
            Self::GeminiCli => "Gemini CLI",
            Self::ClaudeCli => "Claude CLI",
            Self::CodexCli => "Codex CLI",
            Self::GeminiApi => "Gemini API",
        }
    }

    /// Descriptive English selector label shown in dropdown menus (`tl!` key).
    pub fn selector_label(self) -> &'static str {
        match self {
            Self::AntigravityCli => "Antigravity CLI (Google account)",
            Self::GeminiCli => "Gemini CLI (Google account)",
            Self::ClaudeCli => "Claude CLI (Claude subscription)",
            Self::CodexCli => "Codex CLI (ChatGPT subscription)",
            Self::GeminiApi => "Gemini API (API key)",
        }
    }

    /// Concise English description of the authentication / billing model (`tl!` key).
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::AntigravityCli => "Google account sign-in via Antigravity, no API credits",
            Self::GeminiCli => "Google account sign-in, no API credits",
            Self::ClaudeCli => "Claude subscription, no API credits",
            Self::CodexCli => "ChatGPT subscription, no API credits",
            Self::GeminiApi => "Google Gemini API key",
        }
    }

    /// Whether this provider requires a configured Gemini API key.
    pub fn requires_api_key(self) -> bool {
        matches!(self, Self::GeminiApi)
    }

    /// Whether this provider delegates to a local CLI binary.
    pub fn is_cli(self) -> bool {
        !self.requires_api_key()
    }

    /// Whether this provider's execution driver is implemented on the current target.
    pub fn is_implemented(self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            true
        }
        #[cfg(target_arch = "wasm32")]
        {
            matches!(self, Self::GeminiApi)
        }
    }

    /// Expected executable name for local CLI providers, or `None` for API mode.
    pub fn cli_executable(self) -> Option<&'static str> {
        match self {
            Self::AntigravityCli => Some("agy"),
            Self::GeminiCli => Some("gemini"),
            Self::ClaudeCli => Some("claude"),
            Self::CodexCli => Some("codex"),
            Self::GeminiApi => None,
        }
    }

    /// Suggested terminal install command or instructions for local CLI providers.
    pub fn cli_install_hint(self) -> Option<&'static str> {
        match self {
            Self::AntigravityCli => Some("Install Antigravity CLI (agy) from https://antigravity.google"),
            Self::GeminiCli => Some("npm install -g @google/gemini-cli"),
            Self::ClaudeCli => Some("npm install -g @anthropic-ai/claude-code"),
            Self::CodexCli => Some("npm install -g @openai/codex"),
            Self::GeminiApi => None,
        }
    }

    /// Terminal command users run once to sign in to a local CLI provider.
    pub fn cli_login_command(self) -> Option<&'static str> {
        self.cli_executable()
    }

    /// Human-readable account / subscription name used in sign-in guidance.
    pub fn cli_account_label(self) -> Option<&'static str> {
        match self {
            Self::AntigravityCli | Self::GeminiCli => Some("Google account"),
            Self::ClaudeCli => Some("Claude subscription"),
            Self::CodexCli => Some("ChatGPT subscription"),
            Self::GeminiApi => None,
        }
    }
}

impl From<AiProvider> for AiProviderKind {
    fn from(pref: AiProvider) -> Self {
        match pref {
            AiProvider::AntigravityCli => Self::AntigravityCli,
            AiProvider::GeminiCli => Self::GeminiCli,
            AiProvider::ClaudeCli => Self::ClaudeCli,
            AiProvider::CodexCli => Self::CodexCli,
            AiProvider::GeminiApi => Self::GeminiApi,
        }
    }
}

impl From<AiProviderKind> for AiProvider {
    fn from(kind: AiProviderKind) -> Self {
        match kind {
            AiProviderKind::AntigravityCli => Self::AntigravityCli,
            AiProviderKind::GeminiCli => Self::GeminiCli,
            AiProviderKind::ClaudeCli => Self::ClaudeCli,
            AiProviderKind::CodexCli => Self::CodexCli,
            AiProviderKind::GeminiApi => Self::GeminiApi,
        }
    }
}

/// Configuration snapshot passed when dispatching a request to an AI backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiRequestConfig {
    pub provider: AiProviderKind,
    pub gemini_api_key: String,
    pub gemini_model: String,
    /// Optional external CLI session ID used for multi-turn continuation (e.g. Claude `--resume`).
    pub cli_session_id: Option<String>,
    /// Optional user-configured custom executable path override for the active local CLI provider.
    pub cli_binary_override: Option<String>,
    /// Whether to inject a temporary PhotoCraft MCP server configuration into the CLI subprocess.
    pub mcp_enabled: bool,
    /// Whether the user explicitly opted into provider-specific headless tool auto-approval.
    pub auto_approve_mcp_tools: bool,
    /// Resolved path to the local `photocraft-cli` binary used to bridge MCP to this window.
    pub photocraft_cli_path: Option<std::path::PathBuf>,
}

impl AiRequestConfig {
    /// Creates a request config for `provider` with optional Gemini API credentials.
    pub fn new(provider: AiProviderKind, gemini_api_key: impl Into<String>, gemini_model: impl Into<String>) -> Self {
        Self {
            provider,
            gemini_api_key: gemini_api_key.into(),
            gemini_model: gemini_model.into(),
            cli_session_id: None,
            cli_binary_override: None,
            mcp_enabled: false,
            auto_approve_mcp_tools: false,
            photocraft_cli_path: None,
        }
    }

    /// Attaches an existing external CLI session identifier to resume multi-turn state.
    pub fn with_cli_session_id(mut self, session_id: Option<String>) -> Self {
        self.cli_session_id = session_id;
        self
    }

    /// Sets an optional custom executable path override for the local CLI provider.
    pub fn with_cli_binary_override(mut self, path: Option<String>) -> Self {
        self.cli_binary_override = path.map(|p| p.trim().to_string()).filter(|p| !p.is_empty());
        self
    }

    /// Configures temporary PhotoCraft MCP server injection and headless tool approval policy.
    pub fn with_mcp(
        mut self,
        enabled: bool,
        auto_approve: bool,
        photocraft_cli_path: Option<std::path::PathBuf>,
    ) -> Self {
        self.mcp_enabled = enabled;
        self.auto_approve_mcp_tools = auto_approve;
        self.photocraft_cli_path = photocraft_cli_path;
        self
    }
}

/// Errors produced when validating or executing an AI backend request.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AiBackendError {
    /// Error returned by the Gemini API backend.
    #[error("{0}")]
    Gemini(#[from] GeminiError),

    /// Required local CLI binary was not found on `PATH` or common install locations.
    #[error("{provider} executable (`{executable}`) was not found. Install it with: {install_hint}")]
    CliNotFound {
        provider: &'static str,
        executable: &'static str,
        install_hint: &'static str,
    },

    /// Local CLI reported missing sign-in or an expired OAuth / account session.
    #[error("{provider} is not signed in. Run `{login_command}` once in a terminal to sign in with your {account_label} (no API key needed).")]
    CliNotAuthenticated {
        provider: &'static str,
        login_command: &'static str,
        account_label: &'static str,
        details: String,
    },

    /// Local CLI exited with a non-zero status or could not be spawned.
    #[error("{provider} failed{code}: {summary}", code = exit_code.map(|c| format!(" (exit {c})")).unwrap_or_default())]
    CliExecutionFailed {
        provider: &'static str,
        exit_code: Option<i32>,
        summary: String,
    },

    /// Request was cancelled by the user (e.g. pressing Stop).
    #[error("Request cancelled.")]
    Cancelled,

    /// Failed to spawn the background worker thread.
    #[error("Failed to spawn worker thread: {0}")]
    WorkerSpawn(String),

    /// Networking or process execution is unsupported on the current target (e.g. WebAssembly).
    #[error("AI Assistant networking is not supported on WebAssembly yet.")]
    UnsupportedPlatform,
}

impl AiBackendError {
    /// Whether this error indicates a missing or unconfigured API key.
    pub fn is_api_key_issue(&self) -> bool {
        matches!(self, Self::Gemini(GeminiError::MissingApiKey | GeminiError::InvalidApiKey(_)))
    }

    /// Whether this error represents an intentional user cancellation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

/// Asynchronous events emitted by a running AI backend worker.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub enum AiBackendEvent {
    /// Incremental streamed text delta and/or function calls.
    StreamChunk {
        delta_text: String,
        function_calls: Vec<FunctionCall>,
    },
    /// External CLI session / thread identifier reported by the CLI stream.
    SessionId(String),
    /// Live tool call / completion status update emitted by a local CLI agent stream.
    ToolUpdate(crate::cli_agent::CliToolUpdate),
    /// Stream completed with either success or a backend error.
    StreamFinished(Result<(), AiBackendError>),
    /// Non-streaming single-turn result (retained for compatibility).
    #[allow(dead_code)]
    Turn(Result<crate::gemini::ModelTurn, AiBackendError>),
    /// Connectivity test result.
    TestResult(Result<String, AiBackendError>),
}

/// Active background AI request channel and cooperative cancellation handle.
pub struct AiWorkerHandle {
    pub rx: Receiver<AiBackendEvent>,
    pub stop_flag: Arc<AtomicBool>,
    pub pending_function_calls: Vec<FunctionCall>,
}

/// Callback invoked from a background worker whenever new events are available for the UI.
pub type WakeCallback = Arc<dyn Fn() + Send + Sync>;

/// Unified trait implemented by each AI backend driver.
pub trait AiBackend {
    /// Provider variant served by this backend.
    fn kind(&self) -> AiProviderKind;

    /// Validates that the backend is ready to accept a request with `config`.
    fn validate_ready(&self, config: &AiRequestConfig) -> Result<(), AiBackendError>;

    /// Starts a streaming multi-turn conversation request on a background worker.
    fn start_stream_turn(
        &self,
        config: &AiRequestConfig,
        messages: &[ChatMessage],
        wake: WakeCallback,
    ) -> Result<AiWorkerHandle, AiBackendError>;

    /// Starts a connectivity / readiness check on a background worker.
    fn start_test_connection(&self, config: &AiRequestConfig, wake: WakeCallback) -> Result<AiWorkerHandle, AiBackendError>;
}

/// Direct Gemini HTTP/SSE API backend backed by [`crate::gemini::GeminiClient`].
struct GeminiApiBackend;

impl AiBackend for GeminiApiBackend {
    fn kind(&self) -> AiProviderKind {
        AiProviderKind::GeminiApi
    }

    fn validate_ready(&self, config: &AiRequestConfig) -> Result<(), AiBackendError> {
        if config.gemini_api_key.trim().is_empty() {
            return Err(AiBackendError::Gemini(GeminiError::MissingApiKey));
        }
        Ok(())
    }

    fn start_stream_turn(
        &self,
        config: &AiRequestConfig,
        messages: &[ChatMessage],
        wake: WakeCallback,
    ) -> Result<AiWorkerHandle, AiBackendError> {
        self.validate_ready(config)?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let pruned_messages = crate::state::prune_messages_for_context(messages, MAX_CONTEXT_TOKENS);
            let contents = crate::gemini::chat_messages_to_contents(&pruned_messages);
            let tools = Some(crate::gemini::photocraft_tools());
            let client = crate::gemini::GeminiClient::new(&config.gemini_api_key, &config.gemini_model);
            let (tx, rx) = std::sync::mpsc::channel::<AiBackendEvent>();
            let stop_flag = Arc::new(AtomicBool::new(false));
            let stop_flag_worker = Arc::clone(&stop_flag);
            let wake_chunk = Arc::clone(&wake);

            std::thread::Builder::new()
                .name("photocraft-ai-stream".to_string())
                .spawn(move || {
                    let tx_chunk = tx.clone();
                    let result = pollster::block_on(client.stream_chat_turn(
                        &contents,
                        tools,
                        stop_flag_worker,
                        move |delta_text, function_calls| {
                            let _ = tx_chunk.send(AiBackendEvent::StreamChunk { delta_text, function_calls });
                            wake_chunk();
                            Ok(())
                        },
                    ))
                    .map_err(AiBackendError::from);
                    let _ = tx.send(AiBackendEvent::StreamFinished(result));
                    wake();
                })
                .map_err(|e| AiBackendError::WorkerSpawn(e.to_string()))?;

            Ok(AiWorkerHandle {
                rx,
                stop_flag,
                pending_function_calls: Vec::new(),
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (messages, wake);
            Err(AiBackendError::UnsupportedPlatform)
        }
    }

    fn start_test_connection(&self, config: &AiRequestConfig, wake: WakeCallback) -> Result<AiWorkerHandle, AiBackendError> {
        self.validate_ready(config)?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = crate::gemini::GeminiClient::new(&config.gemini_api_key, &config.gemini_model);
            let (tx, rx) = std::sync::mpsc::channel::<AiBackendEvent>();
            let stop_flag = Arc::new(AtomicBool::new(false));

            std::thread::Builder::new()
                .name("photocraft-ai-test".to_string())
                .spawn(move || {
                    let result = pollster::block_on(client.test_connection()).map_err(AiBackendError::from);
                    let _ = tx.send(AiBackendEvent::TestResult(result));
                    wake();
                })
                .map_err(|e| AiBackendError::WorkerSpawn(e.to_string()))?;

            Ok(AiWorkerHandle {
                rx,
                stop_flag,
                pending_function_calls: Vec::new(),
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wake;
            Err(AiBackendError::UnsupportedPlatform)
        }
    }
}

/// Local CLI agent backend delegating to [`crate::cli_agent`].
struct CliAgentBackend(AiProviderKind);

impl AiBackend for CliAgentBackend {
    fn kind(&self) -> AiProviderKind {
        self.0
    }

    fn validate_ready(&self, config: &AiRequestConfig) -> Result<(), AiBackendError> {
        crate::cli_agent::validate_ready_with_override(self.0, config.cli_binary_override.as_deref())
    }

    fn start_stream_turn(
        &self,
        config: &AiRequestConfig,
        messages: &[ChatMessage],
        wake: WakeCallback,
    ) -> Result<AiWorkerHandle, AiBackendError> {
        crate::cli_agent::start_stream_turn(config, messages, wake)
    }

    fn start_test_connection(&self, config: &AiRequestConfig, wake: WakeCallback) -> Result<AiWorkerHandle, AiBackendError> {
        crate::cli_agent::start_test_connection(config, wake)
    }
}

/// Concrete backend dispatcher avoiding heap allocation for each request.
enum BackendDispatch {
    GeminiApi(GeminiApiBackend),
    CliAgent(CliAgentBackend),
}

impl BackendDispatch {
    fn for_provider(provider: AiProviderKind) -> Self {
        match provider {
            AiProviderKind::GeminiApi => Self::GeminiApi(GeminiApiBackend),
            AiProviderKind::AntigravityCli
            | AiProviderKind::GeminiCli
            | AiProviderKind::ClaudeCli
            | AiProviderKind::CodexCli => Self::CliAgent(CliAgentBackend(provider)),
        }
    }
}

impl AiBackend for BackendDispatch {
    fn kind(&self) -> AiProviderKind {
        match self {
            Self::GeminiApi(b) => b.kind(),
            Self::CliAgent(b) => b.kind(),
        }
    }

    fn validate_ready(&self, config: &AiRequestConfig) -> Result<(), AiBackendError> {
        match self {
            Self::GeminiApi(b) => b.validate_ready(config),
            Self::CliAgent(b) => b.validate_ready(config),
        }
    }

    fn start_stream_turn(
        &self,
        config: &AiRequestConfig,
        messages: &[ChatMessage],
        wake: WakeCallback,
    ) -> Result<AiWorkerHandle, AiBackendError> {
        match self {
            Self::GeminiApi(b) => b.start_stream_turn(config, messages, wake),
            Self::CliAgent(b) => b.start_stream_turn(config, messages, wake),
        }
    }

    fn start_test_connection(&self, config: &AiRequestConfig, wake: WakeCallback) -> Result<AiWorkerHandle, AiBackendError> {
        match self {
            Self::GeminiApi(b) => b.start_test_connection(config, wake),
            Self::CliAgent(b) => b.start_test_connection(config, wake),
        }
    }
}

/// Validates that the configured provider in `config` is ready to run.
pub fn validate_ready(config: &AiRequestConfig) -> Result<(), AiBackendError> {
    BackendDispatch::for_provider(config.provider).validate_ready(config)
}

/// Dispatches a streaming multi-turn conversation through the provider selected in `config`.
pub fn start_stream_turn(
    config: &AiRequestConfig,
    messages: &[ChatMessage],
    wake: WakeCallback,
) -> Result<AiWorkerHandle, AiBackendError> {
    BackendDispatch::for_provider(config.provider).start_stream_turn(config, messages, wake)
}

/// Dispatches a connectivity check through the provider selected in `config`.
pub fn start_test_connection(config: &AiRequestConfig, wake: WakeCallback) -> Result<AiWorkerHandle, AiBackendError> {
    BackendDispatch::for_provider(config.provider).start_test_connection(config, wake)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use photocraft_engine::prefs::AiProvider;

    use super::{AiBackendError, AiProviderKind, AiRequestConfig, start_stream_turn, start_test_connection, validate_ready};
    use crate::gemini::GeminiError;
    use crate::state::ChatMessage;

    fn noop_wake() -> super::WakeCallback {
        Arc::new(|| {})
    }

    #[test]
    fn provider_kind_pref_round_trips_and_metadata() {
        assert_eq!(AiProviderKind::default(), AiProviderKind::GeminiApi);
        for kind in AiProviderKind::ALL {
            let pref: AiProvider = kind.into();
            assert_eq!(AiProviderKind::from(pref), kind);
            assert_eq!(AiProviderKind::from_pref_id(kind.pref_id()), Some(kind));
            assert!(!kind.display_name().is_empty());
            assert!(!kind.selector_label().is_empty());
            assert!(!kind.subtitle().is_empty());
        }
        assert_eq!(AiProviderKind::from_pref_id("unknown"), None);
        assert!(AiProviderKind::GeminiApi.requires_api_key());
        assert!(AiProviderKind::GeminiApi.is_implemented());
        assert_eq!(AiProviderKind::GeminiApi.cli_executable(), None);
        assert_eq!(AiProviderKind::GeminiApi.cli_install_hint(), None);
        assert_eq!(AiProviderKind::GeminiApi.cli_login_command(), None);
        assert_eq!(AiProviderKind::GeminiApi.cli_account_label(), None);

        assert_eq!(AiProviderKind::ALL.len(), 5);
        assert_eq!(AiProviderKind::ALL[0], AiProviderKind::AntigravityCli);
        assert_eq!(AiProviderKind::AntigravityCli.display_name(), "Antigravity CLI");
        assert_eq!(AiProviderKind::AntigravityCli.cli_executable(), Some("agy"));

        for cli in [
            AiProviderKind::AntigravityCli,
            AiProviderKind::GeminiCli,
            AiProviderKind::ClaudeCli,
            AiProviderKind::CodexCli,
        ] {
            assert!(!cli.requires_api_key());
            assert!(cli.is_cli());
            assert_eq!(cli.is_implemented(), cfg!(not(target_arch = "wasm32")));
            assert!(cli.cli_executable().is_some());
            assert!(cli.cli_install_hint().is_some());
            assert!(cli.cli_login_command().is_some());
            assert!(cli.cli_account_label().is_some());
        }
    }

    #[test]
    fn gemini_api_requires_non_empty_api_key() {
        let missing = AiRequestConfig::new(AiProviderKind::GeminiApi, "   ", "gemini-2.5-flash");
        assert_eq!(validate_ready(&missing), Err(AiBackendError::Gemini(GeminiError::MissingApiKey)));
        assert!(matches!(
            start_stream_turn(&missing, &[ChatMessage::user("Hi")], noop_wake()),
            Err(AiBackendError::Gemini(GeminiError::MissingApiKey))
        ));
        assert!(matches!(
            start_test_connection(&missing, noop_wake()),
            Err(AiBackendError::Gemini(GeminiError::MissingApiKey))
        ));

        let configured = AiRequestConfig::new(AiProviderKind::GeminiApi, "test-key", "gemini-2.5-flash");
        assert_eq!(validate_ready(&configured), Ok(()));
    }

    #[test]
    fn request_config_carries_optional_cli_session_id_and_mcp_options() {
        let cfg = AiRequestConfig::new(AiProviderKind::ClaudeCli, "", "gemini-2.5-flash")
            .with_cli_session_id(Some("claude-sess-7".to_string()))
            .with_mcp(true, true, Some(std::path::PathBuf::from("/opt/photocraft/photocraft-cli")));
        assert_eq!(cfg.provider, AiProviderKind::ClaudeCli);
        assert_eq!(cfg.cli_session_id.as_deref(), Some("claude-sess-7"));
        assert!(cfg.mcp_enabled);
        assert!(cfg.auto_approve_mcp_tools);
        assert_eq!(
            cfg.photocraft_cli_path.as_deref(),
            Some(std::path::Path::new("/opt/photocraft/photocraft-cli"))
        );
    }

    #[test]
    fn cli_backend_error_variants_format_and_classify_cleanly() {
        let not_found = crate::cli_agent::cli_not_found_error(AiProviderKind::GeminiCli);
        assert!(!not_found.is_api_key_issue());
        assert!(!not_found.is_cancelled());
        assert!(not_found.to_string().contains("npm install -g @google/gemini-cli"));

        let unauth = AiBackendError::CliNotAuthenticated {
            provider: "Claude CLI",
            login_command: "claude",
            account_label: "Claude subscription",
            details: "Not logged in".to_string(),
        };
        assert!(!unauth.is_api_key_issue());
        assert!(unauth.to_string().contains("claude"));
        assert!(unauth.to_string().contains("no API key needed"));

        let exec_err = AiBackendError::CliExecutionFailed {
            provider: "Codex CLI",
            exit_code: Some(1),
            summary: "connection reset".to_string(),
        };
        assert!(exec_err.to_string().contains("exit 1"));
        assert!(exec_err.to_string().contains("connection reset"));

        assert!(AiBackendError::Cancelled.is_cancelled());
        assert!(!AiBackendError::Cancelled.is_api_key_issue());
    }
}
