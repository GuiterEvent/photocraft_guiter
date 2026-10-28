//! Gemini Copilot chat state and session management.

use serde::{Deserialize, Serialize};

/// Role in a Gemini Copilot conversation turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Model,
    System,
    Function,
}

impl ChatRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Model => "Gemini Copilot",
            Self::System => "System",
            Self::Function => "Tool Output",
        }
    }
}

/// Status of a tool action initiated by the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolActionStatus {
    /// Awaiting user confirmation or inspection before executing.
    PendingConfirmation,
    /// Currently being executed.
    Executing,
    /// Successfully executed with result.
    Completed,
    /// Cancelled or rejected by the user.
    Cancelled,
    /// Execution failed with an error message.
    Failed,
}

impl ToolActionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::PendingConfirmation => "Pending Confirmation",
            Self::Executing => "Executing…",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        }
    }
}

/// Represents a tool invocation request from Gemini Copilot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallAction {
    /// Tool function name, e.g. "get_document_info", "execute_command", "adjust_layer_property".
    pub name: String,
    /// Arguments provided by the model.
    #[serde(default)]
    pub args: serde_json::Value,
    /// Current execution or confirmation status.
    pub status: ToolActionStatus,
    /// Execution result payload (or error message) if executed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Whether the details card is collapsed/folded in the UI.
    #[serde(default)]
    pub folded: bool,
}

/// Status of an individual step in a multi-step macro workflow plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowStepStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Skipped,
}

impl WorkflowStepStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Executing => "Executing…",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }
}

/// An individual step in a multi-step macro workflow.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    /// Step identifier, e.g. "step_1".
    pub id: String,
    /// User-friendly description of the action.
    pub description: String,
    /// Tool name: "execute_command" or "adjust_layer_property".
    pub tool: String,
    /// Command identifier (for execute_command).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    /// Parameters payload for the action.
    #[serde(default)]
    pub params: serde_json::Value,
    /// Current execution status.
    pub status: WorkflowStepStatus,
    /// Error message if execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// A structured multi-step workflow plan proposed by Gemini Copilot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowPlan {
    /// Title of the macro workflow.
    pub title: String,
    /// Overall objective or description.
    pub description: String,
    /// Sequence of workflow steps.
    pub steps: Vec<WorkflowStep>,
    /// Whether the plan step list is collapsed/folded in the UI.
    #[serde(default)]
    pub folded: bool,
    /// Whether auto-running consecutive steps continuously.
    #[serde(default)]
    pub auto_running: bool,
}

impl WorkflowPlan {
    pub fn new(title: impl Into<String>, description: impl Into<String>, steps: Vec<WorkflowStep>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            steps,
            folded: false,
            auto_running: false,
        }
    }

    pub fn completed_count(&self) -> usize {
        self.steps.iter().filter(|s| s.status == WorkflowStepStatus::Completed).count()
    }

    pub fn is_all_finished(&self) -> bool {
        self.steps.iter().all(|s| matches!(s.status, WorkflowStepStatus::Completed | WorkflowStepStatus::Skipped))
    }

    pub fn next_pending_index(&self) -> Option<usize> {
        self.steps.iter().position(|s| s.status == WorkflowStepStatus::Pending)
    }
}

/// An image attached to a chat message for multimodal inspection (canvas screenshots or layer details).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatImageAttachment {
    /// MIME type of the image, e.g. "image/jpeg" or "image/png".
    pub mime_type: String,
    /// Base64 encoded image data.
    pub base64_data: String,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Optional label or description (e.g. "Canvas Composite", "Layer: Background").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A persistent chat session containing conversation history.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    /// Unique identifier for this session.
    pub id: String,
    /// Display title for the session.
    pub title: String,
    /// Unix timestamp when session was created.
    pub created_at: u64,
    /// Unix timestamp when session was last updated.
    pub updated_at: u64,
    /// Ordered list of messages in this session.
    pub messages: Vec<ChatMessage>,
}

impl ChatSession {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = current_timestamp();
        Self {
            id: id.into(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        }
    }

    pub fn estimated_tokens(&self) -> usize {
        estimate_session_tokens(&self.messages)
    }
}

/// A single message in the Gemini Copilot chat session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCallAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_plan: Option<WorkflowPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ChatImageAttachment>,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            timestamp: current_timestamp(),
            tool_call: None,
            workflow_plan: None,
            image: None,
        }
    }

    pub fn user_with_image(content: impl Into<String>, image: ChatImageAttachment) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            timestamp: current_timestamp(),
            tool_call: None,
            workflow_plan: None,
            image: Some(image),
        }
    }

    pub fn model(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Model,
            content: content.into(),
            timestamp: current_timestamp(),
            tool_call: None,
            workflow_plan: None,
            image: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            timestamp: current_timestamp(),
            tool_call: None,
            workflow_plan: None,
            image: None,
        }
    }

    pub fn tool_call(content: impl Into<String>, tool_call: ToolCallAction) -> Self {
        Self {
            role: ChatRole::Model,
            content: content.into(),
            timestamp: current_timestamp(),
            tool_call: Some(tool_call),
            workflow_plan: None,
            image: None,
        }
    }

    pub fn workflow_plan(content: impl Into<String>, plan: WorkflowPlan) -> Self {
        Self {
            role: ChatRole::Model,
            content: content.into(),
            timestamp: current_timestamp(),
            tool_call: None,
            workflow_plan: Some(plan),
            image: None,
        }
    }

    pub fn tool_response(name: impl Into<String>, result: serde_json::Value) -> Self {
        let name_str = name.into();
        Self {
            role: ChatRole::Function,
            content: format!("Tool result: {name_str}"),
            timestamp: current_timestamp(),
            tool_call: Some(ToolCallAction {
                name: name_str,
                args: serde_json::Value::Null,
                status: ToolActionStatus::Completed,
                result: Some(result),
                folded: true,
            }),
            workflow_plan: None,
            image: None,
        }
    }
}

pub(crate) fn current_timestamp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

/// Estimates token count for arbitrary text (~4 chars per Latin token, ~1.5 chars per CJK token).
pub fn estimate_text_tokens(text: &str) -> usize {
    let mut units: usize = 0;
    for c in text.chars() {
        if c.is_ascii() {
            units = units.saturating_add(1);
        } else {
            units = units.saturating_add(3);
        }
    }
    (units / 4).max(1)
}

/// Estimates token count for a single ChatMessage.
pub fn estimate_message_tokens(msg: &ChatMessage) -> usize {
    let mut tokens = estimate_text_tokens(&msg.content);
    if let Some(ref _img) = msg.image {
        tokens = tokens.saturating_add(258);
    }
    if let Some(ref tc) = msg.tool_call {
        tokens = tokens.saturating_add(50).saturating_add(estimate_text_tokens(&tc.name));
    }
    if let Some(ref plan) = msg.workflow_plan {
        tokens = tokens.saturating_add(80).saturating_add(plan.steps.len().saturating_mul(40));
    }
    tokens
}

/// Estimates total token count for a collection of messages.
pub fn estimate_session_tokens(messages: &[ChatMessage]) -> usize {
    messages.iter().map(estimate_message_tokens).sum()
}

/// Prunes older conversation turns so total tokens stay within max_tokens budget,
/// preserving recent dialogue turns and context.
pub fn prune_messages_for_context(messages: &[ChatMessage], max_tokens: usize) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return Vec::new();
    }
    let total = estimate_session_tokens(messages);
    if total <= max_tokens {
        return messages.to_vec();
    }

    let min_tail = 4.min(messages.len());
    let tail_start = messages.len().saturating_sub(min_tail);
    let tail_slice = &messages[tail_start..];

    let mut current_tokens: usize = tail_slice.iter().map(estimate_message_tokens).sum();
    let head_slice = &messages[..tail_start];

    let mut prepend = Vec::new();
    for msg in head_slice.iter().rev() {
        let t = estimate_message_tokens(msg);
        if current_tokens.saturating_add(t) <= max_tokens {
            current_tokens = current_tokens.saturating_add(t);
            prepend.push(msg.clone());
        } else {
            break;
        }
    }
    prepend.reverse();
    prepend.extend(tail_slice.iter().cloned());
    prepend
}

/// State for the Gemini Copilot AI assistant chat panel.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CopilotState {
    /// Whether the Copilot panel is currently open/visible.
    pub open: bool,
    /// Multi-session collection.
    pub sessions: Vec<ChatSession>,
    /// Identifier of the currently active session.
    pub active_session_id: String,
    /// Chat conversation history messages (mirrors active session).
    pub messages: Vec<ChatMessage>,
    /// Current content of the user input box.
    pub input: String,
    /// Whether a request is currently in-flight to Gemini.
    #[serde(skip)]
    pub is_loading: bool,
    /// Whether response is actively streaming chunks.
    #[serde(skip)]
    pub is_streaming: bool,
    /// Last error message, if any.
    pub error_message: Option<String>,
    /// Connection test status or prompt feedback status message.
    pub connection_status: Option<String>,
    /// Pending image attachment for the next outgoing message, if any.
    #[serde(skip)]
    pub pending_attachment: Option<ChatImageAttachment>,
    /// Index of message whose attached image is currently displayed in an expanded preview window.
    #[serde(skip)]
    pub expanded_image_preview: Option<usize>,
    /// Whether the sessions drawer/dropdown is open.
    pub history_drawer_open: bool,
    /// Session currently being renamed inline, if any.
    #[serde(skip)]
    pub renaming_session_id: Option<String>,
    /// Temporary text buffer for renaming.
    #[serde(skip)]
    pub rename_input: String,
}

impl CopilotState {
    pub fn clear(&mut self) {
        self.messages.clear();
        self.error_message = None;
        self.connection_status = None;
        self.pending_attachment = None;
        self.expanded_image_preview = None;
        self.sync_active_to_session();
    }

    pub fn add_user_message(&mut self, text: impl Into<String>) {
        self.ensure_active_session();
        self.messages.push(ChatMessage::user(text));
        self.sync_active_to_session();
    }

    pub fn add_model_message(&mut self, text: impl Into<String>) {
        self.ensure_active_session();
        self.messages.push(ChatMessage::model(text));
        self.sync_active_to_session();
    }

    /// Ensures at least one active session exists and matches `self.messages`.
    pub fn ensure_active_session(&mut self) {
        if self.sessions.is_empty() {
            let def_id = "session_default".to_string();
            let mut def_sess = ChatSession::new(&def_id, "Default Chat");
            def_sess.messages = self.messages.clone();
            self.sessions.push(def_sess);
            self.active_session_id = def_id;
        } else if !self.sessions.iter().any(|s| s.id == self.active_session_id)
            && let Some(first) = self.sessions.first()
        {
            self.active_session_id = first.id.clone();
            self.messages = first.messages.clone();
        }
    }

    /// Synchronizes current active `messages` into the active `ChatSession` record.
    pub fn sync_active_to_session(&mut self) {
        self.ensure_active_session();
        if let Some(sess) = self.sessions.iter_mut().find(|s| s.id == self.active_session_id) {
            sess.messages = self.messages.clone();
            sess.updated_at = current_timestamp();

            // Auto-title session from first user message if still default name
            if (sess.title == "New Chat" || sess.title == "Default Chat")
                && let Some(first_user) = self.messages.iter().find(|m| m.role == ChatRole::User)
            {
                let snippet: String = first_user.content.chars().take(28).collect();
                if !snippet.trim().is_empty() {
                    sess.title = snippet.trim().to_string();
                }
            }
        }
    }

    /// Creates a brand-new chat session, sets it active, and clears active dialogue.
    pub fn create_new_session(&mut self, title: Option<String>) -> String {
        self.sync_active_to_session();
        let new_id = format!("sess_{}_{}", current_timestamp(), self.sessions.len().saturating_add(1));
        let title_str = title.unwrap_or_else(|| "New Chat".to_string());
        let session = ChatSession::new(&new_id, title_str);
        self.sessions.push(session);
        self.active_session_id = new_id.clone();
        self.messages.clear();
        self.input.clear();
        self.error_message = None;
        self.connection_status = None;
        self.pending_attachment = None;
        self.history_drawer_open = false;
        new_id
    }

    /// Switches the active conversation to the specified session ID.
    pub fn switch_session(&mut self, session_id: &str) -> bool {
        self.sync_active_to_session();
        if let Some(target) = self.sessions.iter().find(|s| s.id == session_id) {
            self.active_session_id = target.id.clone();
            self.messages = target.messages.clone();
            self.error_message = None;
            self.connection_status = None;
            self.pending_attachment = None;
            self.history_drawer_open = false;
            true
        } else {
            false
        }
    }

    /// Deletes a session by ID. If active, switches to an adjacent session.
    pub fn delete_session(&mut self, session_id: &str) {
        self.sessions.retain(|s| s.id != session_id);
        if self.active_session_id == session_id {
            if let Some(first) = self.sessions.first() {
                self.active_session_id = first.id.clone();
                self.messages = first.messages.clone();
            } else {
                let _ = self.create_new_session(Some("New Chat".to_string()));
            }
        }
    }

    /// Renames a session.
    pub fn rename_session(&mut self, session_id: &str, new_title: &str) {
        if let Some(sess) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            let trimmed = new_title.trim();
            if !trimmed.is_empty() {
                sess.title = trimmed.to_string();
            }
        }
    }

    /// Exports a session as formatted Markdown text.
    pub fn export_markdown(&self, session_id: Option<&str>) -> String {
        let sess_opt = match session_id {
            Some(id) => self.sessions.iter().find(|s| s.id == id),
            None => self.sessions.iter().find(|s| s.id == self.active_session_id),
        };

        let Some(sess) = sess_opt else {
            return "# PhotoCraft Gemini Copilot Chat\n\nNo active session found.\n".to_string();
        };

        let mut md = format!("# {}\n\n", sess.title);
        md.push_str(&format!("*Created: {} | Messages: {}*\n\n---\n\n", sess.created_at, sess.messages.len()));

        for msg in &sess.messages {
            let role_label = match msg.role {
                ChatRole::User => "**User**",
                ChatRole::Model => "**Gemini Copilot**",
                ChatRole::System => "**System**",
                ChatRole::Function => "**Tool Response**",
            };
            md.push_str(&format!("{role_label}:\n\n"));

            if let Some(ref att) = msg.image {
                let lbl = att.label.as_deref().unwrap_or("Attachment");
                md.push_str(&format!("*[Attached Image: {lbl} ({}×{})]*\n\n", att.width, att.height));
            }

            if !msg.content.is_empty() {
                md.push_str(&msg.content);
                md.push_str("\n\n");
            }

            if let Some(ref tc) = msg.tool_call {
                md.push_str(&format!("> **Tool Call `{}`** ({:?})\n", tc.name, tc.status));
                if !tc.args.is_null() {
                    md.push_str(&format!("> Args: `{}`\n", tc.args));
                }
                if let Some(ref res) = tc.result {
                    md.push_str(&format!("> Result: `{}`\n", res));
                }
                md.push('\n');
            }

            if let Some(ref plan) = msg.workflow_plan {
                md.push_str(&format!("### Workflow Plan: {}\n\n{}\n\n", plan.title, plan.description));
                for (i, step) in plan.steps.iter().enumerate() {
                    let status_icon = match step.status {
                        WorkflowStepStatus::Completed => "[x]",
                        WorkflowStepStatus::Skipped => "[-]",
                        WorkflowStepStatus::Failed => "[!]",
                        _ => "[ ]",
                    };
                    md.push_str(&format!("{}. {} **{}** ({})\n", i.saturating_add(1), status_icon, step.description, step.tool));
                }
                md.push('\n');
            }

            md.push_str("---\n\n");
        }

        md
    }

    /// Exports all sessions as a JSON string.
    pub fn export_all_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.sessions)
    }

    /// Imports sessions from a JSON string, appending newly found sessions.
    pub fn import_sessions_json(&mut self, json_str: &str) -> Result<usize, String> {
        let imported: Vec<ChatSession> = if let Ok(list) = serde_json::from_str::<Vec<ChatSession>>(json_str) {
            list
        } else if let Ok(single) = serde_json::from_str::<ChatSession>(json_str) {
            vec![single]
        } else {
            return Err("Invalid JSON format for chat sessions".to_string());
        };

        let count = imported.len();
        for sess in imported {
            if !self.sessions.iter().any(|s| s.id == sess.id) {
                self.sessions.push(sess);
            }
        }
        if let Some(last) = self.sessions.last() {
            self.switch_session(&last.id.clone());
        }
        Ok(count)
    }
}
