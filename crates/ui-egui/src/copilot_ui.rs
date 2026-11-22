//! AI Assistant sidebar and floating conversation interface.
//!
//! Provides an integrated multi-AI sidebar panel for PhotoCraft (docked on the
//! right by default with an optional floating window mode), featuring multi-turn
//! chat, markdown rendering, code block formatting, async background request
//! handling via [`crate::ai_backend`], and preferences integration.

use std::sync::{Arc, Mutex, PoisonError};

use craft::menu::{ItemKind, Row};
use craft::ui::choice::Checkbox;
use craft::ui::contract::{Bind, Entry, Gap, Pad, Props as _, Show as _, ShowContainer as _};
use craft::ui::layout_components::Stack;
use craft::ui::pickers::Select;
use craft::ui::text::TextField;
use egui::{CornerRadius, Margin, Rect, RichText, Sense, Stroke, pos2, vec2};
use serde_json::{Value, json};

use crate::PhotocraftApp;
use crate::ai_backend::{AiBackendError, AiBackendEvent, AiProviderKind, AiRequestConfig, AiWorkerHandle, WakeCallback};
use crate::cli_agent::{CliDetectionStatus, CliToolState, CliToolUpdate};
use crate::state::{ChatImageAttachment, ChatMessage, ChatRole, CopilotState, ToolActionStatus, ToolCallAction, ToolCallOrigin};
use crate::theme::{self, Tokens};

const MIN_COPILOT_SIDEBAR_WIDTH: f32 = CopilotState::MIN_SIDEBAR_WIDTH;
const MAX_COPILOT_SIDEBAR_WIDTH: f32 = CopilotState::MAX_SIDEBAR_WIDTH;
/// Minimum horizontal room kept for the main canvas + panel docking workspace beside the right sidebar.
const MIN_WORKSPACE_WIDTH: f32 = 240.0;
/// Pointer grab thickness centered on the vertical divider between workspace and AI sidebar.
const SIDEBAR_DIVIDER_HIT_WIDTH: f32 = 8.0;

/// Stable egui id for the multiline AI prompt composer.
pub fn composer_input_id() -> egui::Id {
    egui::Id::unique("photocraft-ai-composer-input")
}

/// Whether the AI sidebar's multiline prompt composer currently holds keyboard focus.
pub fn text_input_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.has_focus(composer_input_id()))
}

mod icons {
    use craft::behavior::IconId;
    use craft::ui::buttons::IconButton;
    use craft::ui::contract::{Props as _, Show as _};
    use craft::ui::display::Icon;
    use craft::ui::primitives::IconSpec;
    use egui::{Color32, Rect, Response, Ui};

    #[cfg(test)]
    pub const ALL_COPILOT_ICON_NAMES: &[&str] = &[
        "sparkles",
        "clock",
        "plus",
        "x",
        "float",
        "panel_right",
        "trash",
        "rotate-cw",
        "check",
        "triangle-alert",
        "info",
        "copy",
        "image",
        "eye",
        "sliders-horizontal",
        "search",
        "circle",
        "minus",
        "pencil",
        "message-square",
        "chevron_down",
        "chevron_up",
        "x_circle",
        "palette",
        "sun",
        "crop",
    ];

    pub(super) fn resolve_name(name: &'static str) -> &'static str {
        match name {
            "rotate-cw" => "rotate_cw",
            "sliders-horizontal" => "adjustments",
            other => other,
        }
    }

    pub fn paint(ui: &mut Ui, rect: Rect, name: &'static str, _size: f32, _color: Color32) {
        if !rect.is_finite() {
            return;
        }
        let icon = IconId::new(resolve_name(name));
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            Icon::new(IconSpec::svg(icon)).show(ui);
        });
    }

    pub fn button(ui: &mut Ui, name: &'static str, _size: f32, _active: bool, tooltip: &str) -> Response {
        let icon = IconId::new(resolve_name(name));
        IconButton::new(icon).label(tooltip.to_owned()).tooltip(tooltip.to_owned()).show(ui).response
    }
}

mod widgets {
    use craft::theme::Variant;
    use craft::ui::buttons::Button;
    use craft::ui::contract::{Props as _, Show as _};
    use craft::ui::display::Divider;
    use egui::{Response, Ui};

    pub fn hairline(ui: &mut Ui) {
        Divider::horizontal().show(ui);
    }

    pub fn primary_button(ui: &mut Ui, label: &str, _min_width: f32) -> Response {
        Button::new(label.to_owned()).variant(Variant::PRIMARY).show(ui).response
    }

    pub fn secondary_button(ui: &mut Ui, label: &str, _min_width: f32) -> Response {
        Button::new(label.to_owned()).show(ui).response
    }
}

pub const PANEL_ID: &str = "window.panel.geminiCopilot";

fn copilot_channel_id() -> egui::Id {
    egui::Id::unique("photocraft-gemini-copilot-channel")
}

type CopilotChannel = Arc<Mutex<Option<AiWorkerHandle>>>;

/// Returns the active AI backend provider from engine preferences.
pub fn active_provider(app: &PhotocraftApp) -> AiProviderKind {
    AiProviderKind::from(app.session.prefs().integrations.ai_provider)
}

/// Switches the active AI backend provider through `prefs.set` and syncs `UiState`.
pub fn set_provider(app: &mut PhotocraftApp, provider: AiProviderKind) -> Result<(), String> {
    let changed = active_provider(app) != provider;
    app.run("prefs.set", json!({ "path": "integrations.aiProvider", "value": provider.pref_id() }))?;
    app.ui.copilot.ai_provider = provider;
    if changed {
        app.ui.copilot.clear_error();
        app.ui.copilot.connection_status = None;
    }
    Ok(())
}

/// Returns the optional custom CLI executable path override configured in Preferences for `provider`.
pub fn cli_override_for_provider(app: &PhotocraftApp, provider: AiProviderKind) -> Option<&str> {
    let raw = app.session.prefs().integrations.cli_path_override(provider.into());
    (!raw.is_empty()).then_some(raw)
}

/// Returns the optional custom `photocraft-cli` executable path override configured in Preferences.
pub fn photocraft_cli_override(app: &PhotocraftApp) -> Option<&str> {
    let raw = app.session.prefs().integrations.photocraft_cli_path_override();
    (!raw.is_empty()).then_some(raw)
}

/// Returns the cached CLI detection status for `provider`, scanning once lazily when uncached.
pub fn cached_cli_status(copilot: &mut CopilotState, provider: AiProviderKind) -> CliDetectionStatus {
    cached_cli_status_with_override(copilot, provider, None)
}

/// Returns the cached CLI detection status for `provider`, respecting `custom_override` when scanning.
pub fn cached_cli_status_with_override(
    copilot: &mut CopilotState,
    provider: AiProviderKind,
    custom_override: Option<&str>,
) -> CliDetectionStatus {
    if let Some(status) = copilot.cli_detection_cache.get(&provider) {
        return status.clone();
    }
    let status = crate::cli_agent::detect_cli_with_override(provider, custom_override);
    copilot.cli_detection_cache.insert(provider, status.clone());
    status
}

/// Re-scans `PATH` and common install locations for `provider` and updates `copilot.cli_detection_cache`.
pub fn refresh_cli_detection(copilot: &mut CopilotState, provider: AiProviderKind) -> CliDetectionStatus {
    refresh_cli_detection_with_override(copilot, provider, None)
}

/// Re-scans for `provider` honoring `custom_override` first, updating `copilot.cli_detection_cache`.
pub fn refresh_cli_detection_with_override(
    copilot: &mut CopilotState,
    provider: AiProviderKind,
    custom_override: Option<&str>,
) -> CliDetectionStatus {
    let status = crate::cli_agent::detect_cli_with_override(provider, custom_override);
    copilot.cli_detection_cache.insert(provider, status.clone());
    status
}

/// Returns the cached `photocraft-cli` detection status, scanning once lazily when uncached.
pub fn cached_photocraft_cli_status(copilot: &mut CopilotState) -> CliDetectionStatus {
    cached_photocraft_cli_status_with_override(copilot, None)
}

/// Returns the cached `photocraft-cli` detection status, honoring `custom_override` when scanning.
pub fn cached_photocraft_cli_status_with_override(
    copilot: &mut CopilotState,
    custom_override: Option<&str>,
) -> CliDetectionStatus {
    if let Some(ref status) = copilot.photocraft_cli_detection {
        return status.clone();
    }
    let status = crate::cli_agent::detect_photocraft_cli_with_override(custom_override);
    copilot.photocraft_cli_detection = Some(status.clone());
    status
}

/// Re-scans sibling executable directory and `PATH` for `photocraft-cli` and refreshes cache.
pub fn refresh_photocraft_cli_detection(copilot: &mut CopilotState) -> CliDetectionStatus {
    refresh_photocraft_cli_detection_with_override(copilot, None)
}

/// Re-scans for `photocraft-cli` honoring `custom_override` first, updating cache.
pub fn refresh_photocraft_cli_detection_with_override(
    copilot: &mut CopilotState,
    custom_override: Option<&str>,
) -> CliDetectionStatus {
    let status = crate::cli_agent::detect_photocraft_cli_with_override(custom_override);
    copilot.photocraft_cli_detection = Some(status.clone());
    status
}

/// Current live status of PhotoCraft's desktop MCP control channel (`AgentLink`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpControlState {
    /// Control channel is actively listening for `photocraft-cli mcp` bridge connections.
    Listening {
        port: Option<u16>,
        launch_flags: bool,
        warning: Option<String>,
    },
    /// Suite `AI-MCP` flag is enabled and ArtCraft Launcher's switch for PhotoCraft is off.
    LauncherDisabled,
    /// Control channel is disabled in PhotoCraft's preferences (`connect_agents` or `allow_agent_control` is false).
    Disabled {
        connect_agents: bool,
        allow_agent_control: bool,
    },
    /// Control channel attempted to start but reported an error (e.g. port bind or settings folder error).
    Error(String),
    /// Live control channel is unavailable in this environment (e.g. WebAssembly or no link service).
    Unavailable,
}

impl McpControlState {
    /// Whether this PhotoCraft window is currently listening for MCP control connections.
    pub fn is_listening(&self) -> bool {
        matches!(self, Self::Listening { .. })
    }
}

/// Inspects PhotoCraft's live desktop control channel (`AgentLink` + `integrations` preferences).
pub fn inspect_mcp_control_state(app: &PhotocraftApp) -> McpControlState {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = app;
        McpControlState::Unavailable
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let integrations = &app.session.prefs().integrations;
        let link_status = app.services.agent_link.as_ref().map(|link| link.status());
        let has_control = app.control.is_some();

        if let Some(status) = link_status
            && (status.port.is_some() || has_control)
        {
            return McpControlState::Listening {
                port: status.port,
                launch_flags: status.launch_flags,
                warning: status.problem.as_ref().map(ToString::to_string),
            };
        }
        if has_control {
            return McpControlState::Listening {
                port: None,
                launch_flags: false,
                warning: None,
            };
        }

        let mcp_flag = app.services.agent_switch.as_ref().is_some_and(|sw| sw.mcp_flag());
        let launcher_allows = app.services.agent_switch.as_ref().is_some_and(|sw| sw.launcher());
        if mcp_flag && !launcher_allows {
            return McpControlState::LauncherDisabled;
        }

        if !integrations.connect_agents || !integrations.agents.allow_agent_control {
            return McpControlState::Disabled {
                connect_agents: integrations.connect_agents,
                allow_agent_control: integrations.agents.allow_agent_control,
            };
        }

        if let Some(problem) = link_status.and_then(|s| s.problem.as_ref()) {
            return McpControlState::Error(problem.to_string());
        }

        if app.services.agent_link.is_none() {
            return McpControlState::Unavailable;
        }

        McpControlState::Disabled {
            connect_agents: integrations.connect_agents,
            allow_agent_control: integrations.agents.allow_agent_control,
        }
    }
}

/// Enables PhotoCraft's MCP control channel through `prefs.set`, immediately flushes preferences
/// to disk (so `photocraft-cli mcp` observes them), and synchronizes `AgentLink`.
pub fn enable_mcp_control(app: &mut PhotocraftApp, ctx: &egui::Context) -> Result<(), String> {
    let mut values = serde_json::Map::new();
    values.insert("integrations.connectAgents".to_string(), json!(true));
    if !app.session.prefs().integrations.agents.allow_agent_control {
        values.insert("integrations.allowAgentControl".to_string(), json!(true));
    }

    app.run("prefs.set", json!({ "values": Value::Object(values) }))?;
    crate::prefs_ui::save_preferences(app)?;

    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::prefs_ui::sync_agent_link(app, ctx);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = ctx;
    }

    match inspect_mcp_control_state(app) {
        McpControlState::Listening { .. } => {
            app.ui.copilot.mcp_enabled = true;
            app.ui.copilot.clear_active_cli_session_id();
            app.ui.copilot.clear_error();
            ctx.request_repaint();
            Ok(())
        }
        McpControlState::LauncherDisabled => Err(tl!(
            "Enable PhotoCraft in ArtCraft Launcher › Settings › AI agents to allow live MCP control."
        )
        .to_string()),
        McpControlState::Error(msg) => Err(msg),
        McpControlState::Disabled { .. } | McpControlState::Unavailable => {
            Err(tl!("PhotoCraft's live control channel is unavailable in this session.").to_string())
        }
    }
}

/// Applies a streamed CLI tool status update to `copilot.messages`, updating any matching
/// in-flight tool card in place by `call_id` or `step_index` instead of duplicating cards.
pub fn apply_cli_tool_update(copilot: &mut CopilotState, update: CliToolUpdate) {
    let new_status = match update.state {
        CliToolState::Active => ToolActionStatus::Executing,
        CliToolState::Done => ToolActionStatus::Completed,
        CliToolState::Failed => ToolActionStatus::Failed,
        CliToolState::Unknown => {
            if update.error.is_some() {
                ToolActionStatus::Failed
            } else if update.output.is_some() {
                ToolActionStatus::Completed
            } else {
                ToolActionStatus::Executing
            }
        }
    };

    let corr_key = update.correlation_key();
    let result_payload = if let Some(ref err) = update.error {
        match &update.output {
            Some(out) => Some(json!({ "error": err, "output": out })),
            None => Some(json!({ "error": err })),
        }
    } else if let Some(ref out) = update.output {
        Some(out.clone())
    } else if matches!(new_status, ToolActionStatus::Completed | ToolActionStatus::Failed) && !update.tool_info.is_null() {
        Some(update.tool_info.clone())
    } else {
        None
    };

    let turn_start = copilot
        .messages
        .iter()
        .rposition(|m| m.role == ChatRole::User)
        .map_or(0, |idx| idx.saturating_add(1));

    let existing_idx = if let Some(ref key) = corr_key {
        copilot.messages[turn_start..].iter().rposition(|m| {
            m.tool_call
                .as_ref()
                .is_some_and(|tc| tc.origin == ToolCallOrigin::CliAgent && tc.call_id.as_deref() == Some(key.as_str()))
        })
    } else if matches!(new_status, ToolActionStatus::Completed | ToolActionStatus::Failed) {
        copilot.messages[turn_start..].iter().rposition(|m| {
            m.tool_call.as_ref().is_some_and(|tc| {
                tc.origin == ToolCallOrigin::CliAgent
                    && tc.call_id.is_none()
                    && tc.status == ToolActionStatus::Executing
                    && tc.name == update.tool_name
            })
        })
    } else {
        None
    }
    .map(|offset| turn_start.saturating_add(offset));

    if let Some(idx) = existing_idx
        && let Some(msg) = copilot.messages.get_mut(idx)
        && let Some(ref mut tc) = msg.tool_call
    {
        tc.status = new_status;
        if !update.tool_name.trim().is_empty() && update.tool_name != "tool" {
            tc.name = update.tool_name;
        }
        if !update.parameters.is_null() {
            tc.args = update.parameters;
        }
        if let Some(res) = result_payload {
            tc.result = Some(res);
        }
        if new_status == ToolActionStatus::Failed {
            tc.folded = false;
        }
    } else {
        let action = ToolCallAction {
            name: update.tool_name,
            args: update.parameters,
            status: new_status,
            result: result_payload,
            folded: new_status != ToolActionStatus::Failed,
            origin: ToolCallOrigin::CliAgent,
            call_id: corr_key,
        };
        copilot.messages.push(ChatMessage::tool_call("", action));
    }
}

/// Settles any CLI tool cards still in `Executing` state within the current turn when the stream ends.
fn finalize_in_flight_cli_tools(copilot: &mut CopilotState, terminal_status: ToolActionStatus) {
    let turn_start = copilot
        .messages
        .iter()
        .rposition(|m| m.role == ChatRole::User)
        .map_or(0, |idx| idx.saturating_add(1));

    for msg in &mut copilot.messages[turn_start..] {
        if let Some(ref mut tc) = msg.tool_call
            && tc.origin == ToolCallOrigin::CliAgent
            && tc.status == ToolActionStatus::Executing
        {
            tc.status = terminal_status;
        }
    }
}

/// Immediately requests cancellation of in-flight AI streaming generation.
pub fn stop_generating(app: &mut PhotocraftApp, ctx: &egui::Context) {
    app.ui.copilot.is_loading = false;
    app.ui.copilot.is_streaming = false;
    finalize_in_flight_cli_tools(&mut app.ui.copilot, ToolActionStatus::Cancelled);
    let channel = ctx.data_mut(|d| d.get_temp::<CopilotChannel>(copilot_channel_id()));
    if let Some(channel) = channel {
        let mut guard = channel.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(ref mut st) = *guard {
            st.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        *guard = None;
    }
}

/// Menu ids this panel handles (retaining compatibility with legacy Gemini Copilot ids).
pub fn handles(id: &str) -> bool {
    matches!(
        id,
        PANEL_ID
            | "window.toggle.geminiCopilot"
            | "view.geminiCopilot"
            | "window.panel.aiSidebar"
            | "window.toggle.aiSidebar"
    )
}

/// Checkmark state for the Window menu.
pub fn checked(app: &PhotocraftApp, id: &str) -> Option<bool> {
    handles(id).then_some(app.ui.copilot.open)
}

/// Menu command invocation to toggle the AI assistant sidebar / floating window.
pub fn menu(app: &mut PhotocraftApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if !handles(id) {
        return None;
    }
    let show = params
        .get("show")
        .or_else(|| params.get("on"))
        .and_then(Value::as_bool)
        .unwrap_or(!app.ui.copilot.open);
    app.ui.copilot.open = show;
    if let Some(docked) = params.get("docked").and_then(Value::as_bool) {
        app.ui.copilot.docked = docked;
    }
    Some(Ok(json!({ "open": show, "docked": app.ui.copilot.docked })))
}

/// Inserts the AI Assistant item into the Window menu list under the compatible command id.
pub fn rows(app: &PhotocraftApp, rows: &mut Vec<Row>) {
    let at = rows
        .iter()
        .rposition(|i| i.path.first().map(String::as_str) == Some("Window") && i.command.id.starts_with("window.panel."))
        .unwrap_or(rows.len());
    let idx = (at + 1).min(rows.len());
    rows.insert(idx, Row::new(&["Window"], PANEL_ID, "AI Assistant").enabled(true).kind(ItemKind::Check(app.ui.copilot.open)));
}

/// Queries the active document state and metadata for Gemini Copilot.
pub fn query_document_info(app: &PhotocraftApp) -> serde_json::Value {
    let Some(st) = app.session.active() else {
        return json!({
            "has_document": false,
            "message": "No document is currently open in PhotoCraft."
        });
    };

    let doc = &st.doc;
    let (width, height) = (doc.size.width, doc.size.height);
    let mode_str = match doc.mode {
        photocraft_doc::ColorMode::Rgb => "RGB",
        photocraft_doc::ColorMode::Cmyk => "CMYK",
        photocraft_doc::ColorMode::Grayscale => "Grayscale",
        photocraft_doc::ColorMode::Lab => "Lab",
        photocraft_doc::ColorMode::Bitmap => "Bitmap",
        photocraft_doc::ColorMode::Indexed => "Indexed",
        photocraft_doc::ColorMode::Duotone => "Duotone",
        photocraft_doc::ColorMode::Multichannel => "Multichannel",
    };
    let depth_str = match doc.depth {
        photocraft_doc::SampleType::U8 => "8-bit",
        photocraft_doc::SampleType::U16 => "16-bit",
        photocraft_doc::SampleType::F32 => "32-bit (float)",
    };

    let total_layers = doc.layers.len();
    let mut layer_summaries = Vec::new();
    for l in doc.layers.iter().take(20) {
        layer_summaries.push(json!({
            "id": l.id.0,
            "name": l.name,
            "visible": l.visible,
            "opacity": (l.opacity * 100.0).round() as u32,
            "blend_mode": format!("{:?}", l.blend),
            "is_group": l.is_group(),
        }));
    }

    let active_layer_info = st.active_layer.and_then(|id| doc.layer(id)).map(|l| {
        json!({
            "id": l.id.0,
            "name": l.name,
            "visible": l.visible,
            "opacity": (l.opacity * 100.0).round() as u32,
            "blend_mode": format!("{:?}", l.blend),
        })
    });

    json!({
        "has_document": true,
        "name": doc.name,
        "width": width,
        "height": height,
        "resolution_dpi": doc.resolution_dpi,
        "color_mode": mode_str,
        "bit_depth": depth_str,
        "total_layers": total_layers,
        "active_layer": active_layer_info,
        "layers": layer_summaries,
    })
}

/// Adjusts the opacity, blend mode, or visibility of the active layer.
pub fn execute_adjust_layer_property(app: &mut PhotocraftApp, args: &Value) -> Result<Value, String> {
    let mut props = serde_json::Map::new();

    if let Some(opacity_num) = args.get("opacity").and_then(Value::as_f64) {
        let normalized = if opacity_num > 1.0 && opacity_num <= 100.0 {
            opacity_num / 100.0
        } else {
            opacity_num
        };
        if !(0.0..=1.0).contains(&normalized) {
            return Err(format!("Opacity must be between 0.0 and 1.0 (or 0 to 100), got {opacity_num}"));
        }
        props.insert("opacity".to_string(), json!(normalized));
    }

    if let Some(blend_str) = args.get("blend_mode").and_then(Value::as_str) {
        props.insert("blend".to_string(), json!(blend_str));
    }

    if let Some(visible_bool) = args.get("visible").and_then(Value::as_bool) {
        props.insert("visible".to_string(), json!(visible_bool));
    }

    if props.is_empty() {
        return Err("No layer properties specified to adjust (expected opacity, blend_mode, or visible)".to_string());
    }

    app.run("layer.setProps", Value::Object(props))
}

/// Inspects the current canvas or active layer, capturing a downscaled thumbnail and visual metrics.
pub fn perform_canvas_inspection(
    app: &PhotocraftApp,
    target: &str,
    max_edge: u32,
) -> Result<(ChatImageAttachment, Value), String> {
    let st = app.session.active().ok_or_else(|| "No document is currently open in PhotoCraft.".to_string())?;

    if target == "active_layer" {
        let active_id = st.active_layer.ok_or_else(|| "No active layer is selected.".to_string())?;
        crate::gemini::encode_layer_thumbnail(&st.doc, active_id, max_edge, true)
    } else {
        crate::gemini::encode_canvas_thumbnail(&st.doc, max_edge, true)
    }
}

/// Helper to attach the current canvas screenshot (if not already attached) and send a prompt preset.
fn attach_and_send_preset(app: &mut PhotocraftApp, ctx: &egui::Context, prompt: String) {
    if app.ui.copilot.pending_attachment.is_none()
        && let Ok((att, _)) = perform_canvas_inspection(app, "composite", crate::gemini::DEFAULT_VISION_MAX_EDGE)
    {
        app.ui.copilot.pending_attachment = Some(att);
    }
    app.ui.copilot.input = prompt;
    send_chat(app, ctx);
}

/// Executes a confirmed tool action on PhotoCraft canvas/engine and notifies Gemini of the result.
fn execute_tool_action(app: &mut PhotocraftApp, ctx: &egui::Context, msg_idx: usize) {
    let (name, args) = {
        let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx) else { return };
        let Some(ref mut tc) = msg.tool_call else { return };
        tc.status = crate::state::ToolActionStatus::Executing;
        (tc.name.clone(), tc.args.clone())
    };

    let result = match name.as_str() {
        "execute_command" => {
            if let Some(cmd_id) = args.get("command_id").and_then(Value::as_str) {
                let params = args.get("params").cloned().unwrap_or(json!({}));
                crate::menus::invoke(app, ctx, cmd_id, params)
            } else {
                Err("Missing 'command_id' in execute_command".to_string())
            }
        }
        "adjust_layer_property" => {
            execute_adjust_layer_property(app, &args)
        }
        "inspect_canvas_image" => {
            let target = args.get("target").and_then(Value::as_str).unwrap_or("composite");
            let detail = args.get("detail").and_then(Value::as_str).unwrap_or("standard");
            let max_edge = match detail {
                "high" => crate::gemini::HIGH_DETAIL_VISION_MAX_EDGE,
                _ => crate::gemini::DEFAULT_VISION_MAX_EDGE,
            };
            perform_canvas_inspection(app, target, max_edge).map(|(att, metrics)| {
                json!({
                    "status": "success",
                    "target": target,
                    "metrics": metrics,
                    "width": att.width,
                    "height": att.height,
                })
            })
        }
        other => Err(format!("Unknown tool: {other}")),
    };

    let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx) else { return };
    let Some(ref mut tc) = msg.tool_call else { return };
    match result {
        Ok(val) => {
            tc.status = crate::state::ToolActionStatus::Completed;
            tc.result = Some(json!({
                "status": "success",
                "output": val,
            }));
            let resp_val = json!({
                "status": "success",
                "name": name,
                "output": val,
            });
            app.ui.copilot.messages.push(ChatMessage::tool_response(name, resp_val));
            send_chat_turn(app, ctx);
        }
        Err(err) => {
            tc.status = crate::state::ToolActionStatus::Failed;
            tc.result = Some(json!({
                "status": "error",
                "error": err,
            }));
        }
    }
}

///// Cancels a pending tool action without executing it.
fn cancel_tool_action(app: &mut PhotocraftApp, msg_idx: usize) {
    if let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx)
        && let Some(ref mut tc) = msg.tool_call
    {
        tc.status = crate::state::ToolActionStatus::Cancelled;
        tc.result = Some(json!({
            "status": "cancelled",
            "message": "User cancelled this action."
        }));
    }
}

/// Executes a single step in a workflow plan.
fn execute_workflow_step(app: &mut PhotocraftApp, ctx: &egui::Context, msg_idx: usize, step_idx: usize) {
    let (tool, command_id, params) = {
        let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx) else { return };
        let Some(ref mut plan) = msg.workflow_plan else { return };
        let Some(step) = plan.steps.get_mut(step_idx) else { return };
        step.status = crate::state::WorkflowStepStatus::Executing;
        (step.tool.clone(), step.command_id.clone(), step.params.clone())
    };

    let result = match tool.as_str() {
        "execute_command" => {
            if let Some(cmd) = command_id {
                crate::menus::invoke(app, ctx, &cmd, params)
            } else {
                Err("Missing command_id in workflow step".to_string())
            }
        }
        "adjust_layer_property" => {
            execute_adjust_layer_property(app, &params)
        }
        other => Err(format!("Unknown tool in workflow step: {other}")),
    };

    let (advance_next, workflow_completed, plan_title, completed_count, next_step_idx) = {
        let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx) else { return };
        let Some(ref mut plan) = msg.workflow_plan else { return };
        let Some(step) = plan.steps.get_mut(step_idx) else { return };

        match result {
            Ok(_) => {
                step.status = crate::state::WorkflowStepStatus::Completed;
                step.error = None;
            }
            Err(e) => {
                step.status = crate::state::WorkflowStepStatus::Failed;
                step.error = Some(e);
                plan.auto_running = false;
            }
        }

        let is_finished = plan.is_all_finished();
        if is_finished {
            plan.auto_running = false;
        }

        let next_idx = if plan.auto_running {
            plan.next_pending_index()
        } else {
            None
        };

        (plan.auto_running && next_idx.is_some(), is_finished, plan.title.clone(), plan.completed_count(), next_idx)
    };

    app.ui.copilot.sync_active_to_session();

    if advance_next {
        if let Some(next_step) = next_step_idx {
            execute_workflow_step(app, ctx, msg_idx, next_step);
        }
    } else if workflow_completed {
        let resp_val = json!({
            "status": "completed",
            "title": plan_title,
            "steps_completed": completed_count,
        });
        app.ui.copilot.messages.push(ChatMessage::tool_response("plan_workflow_actions", resp_val));
        send_chat_turn(app, ctx);
    }
}

/// Skips a single step in a workflow plan.
fn skip_workflow_step(app: &mut PhotocraftApp, ctx: &egui::Context, msg_idx: usize, step_idx: usize) {
    let next_step_idx = {
        let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx) else { return };
        let Some(ref mut plan) = msg.workflow_plan else { return };
        let Some(step) = plan.steps.get_mut(step_idx) else { return };
        step.status = crate::state::WorkflowStepStatus::Skipped;
        if plan.auto_running {
            plan.next_pending_index()
        } else {
            None
        }
    };
    app.ui.copilot.sync_active_to_session();

    if let Some(next_step) = next_step_idx {
        execute_workflow_step(app, ctx, msg_idx, next_step);
    }
}

/// Executes all pending steps of a workflow plan continuously.
fn execute_entire_workflow(app: &mut PhotocraftApp, ctx: &egui::Context, msg_idx: usize) {
    let next_step_opt = {
        let Some(msg) = app.ui.copilot.messages.get_mut(msg_idx) else { return };
        let Some(ref mut plan) = msg.workflow_plan else { return };
        plan.auto_running = true;
        plan.next_pending_index()
    };

    if let Some(next_idx) = next_step_opt {
        execute_workflow_step(app, ctx, msg_idx, next_idx);
    }
}

/// Processes tool/function calls collected during a completed AI stream turn.
/// Returns `true` when an automatic follow-up turn should be sent with tool outputs.
fn handle_finished_function_calls(app: &mut PhotocraftApp, pending_fcs: Vec<crate::gemini::FunctionCall>) -> bool {
    let mut auto_respond_turn = false;
    for fc in pending_fcs {
        match fc.name.as_str() {
            "plan_workflow_actions" => match crate::gemini::parse_workflow_plan(&fc.args) {
                Ok(plan) => {
                    let summary = format!("{}: {}", tl!("Suggested Workflow Plan"), plan.title);
                    app.ui.copilot.messages.push(ChatMessage::workflow_plan(summary, plan));
                }
                Err(err) => {
                    app.ui.copilot.error_message = Some(format!("Failed to parse workflow plan: {err}"));
                }
            },
            "get_document_info" => {
                let doc_info = query_document_info(app);
                let summary_text = tl!("Reading document info…").to_string();
                let action = crate::state::ToolCallAction {
                    name: fc.name.clone(),
                    args: fc.args.clone(),
                    status: crate::state::ToolActionStatus::Completed,
                    result: Some(doc_info.clone()),
                    folded: true,
                    origin: ToolCallOrigin::GeminiApi,
                    call_id: None,
                };
                app.ui.copilot.messages.push(ChatMessage::tool_call(summary_text, action));
                app.ui.copilot.messages.push(ChatMessage::tool_response("get_document_info", doc_info));
                auto_respond_turn = true;
            }
            "execute_command" => {
                let cmd = fc.args.get("command_id").and_then(Value::as_str).unwrap_or("command");
                let prompt_text = format!("{}: `{cmd}`", tl!("Suggested Action"));
                let action = crate::state::ToolCallAction {
                    name: fc.name.clone(),
                    args: fc.args.clone(),
                    status: crate::state::ToolActionStatus::PendingConfirmation,
                    result: None,
                    folded: false,
                    origin: ToolCallOrigin::GeminiApi,
                    call_id: None,
                };
                app.ui.copilot.messages.push(ChatMessage::tool_call(prompt_text, action));
            }
            "adjust_layer_property" => {
                let prompt_text = tl!("Adjust Layer Properties").to_string();
                let action = crate::state::ToolCallAction {
                    name: fc.name.clone(),
                    args: fc.args.clone(),
                    status: crate::state::ToolActionStatus::PendingConfirmation,
                    result: None,
                    folded: false,
                    origin: ToolCallOrigin::GeminiApi,
                    call_id: None,
                };
                app.ui.copilot.messages.push(ChatMessage::tool_call(prompt_text, action));
            }
            "inspect_canvas_image" => {
                let target = fc.args.get("target").and_then(Value::as_str).unwrap_or("composite");
                let detail = fc.args.get("detail").and_then(Value::as_str).unwrap_or("standard");
                let max_edge = match detail {
                    "high" => crate::gemini::HIGH_DETAIL_VISION_MAX_EDGE,
                    _ => crate::gemini::DEFAULT_VISION_MAX_EDGE,
                };
                let inspect_res = perform_canvas_inspection(app, target, max_edge);
                let summary_text = if target == "active_layer" {
                    tl!("Inspecting active layer image…").to_string()
                } else {
                    tl!("Inspecting canvas image…").to_string()
                };

                let (result_json, attachment_opt) = match inspect_res {
                    Ok((attachment, metrics)) => {
                        let json_val = json!({
                            "status": "success",
                            "target": target,
                            "metrics": metrics,
                            "message": "Canvas image inspected and visual metrics calculated."
                        });
                        (json_val, Some(attachment))
                    }
                    Err(err_msg) => {
                        let json_val = json!({
                            "status": "error",
                            "error": err_msg
                        });
                        (json_val, None)
                    }
                };

                let action = crate::state::ToolCallAction {
                    name: fc.name.clone(),
                    args: fc.args.clone(),
                    status: crate::state::ToolActionStatus::Completed,
                    result: Some(result_json.clone()),
                    folded: false,
                    origin: ToolCallOrigin::GeminiApi,
                    call_id: None,
                };
                let mut tool_msg = ChatMessage::tool_call(summary_text, action);
                tool_msg.image = attachment_opt;
                app.ui.copilot.messages.push(tool_msg);
                app.ui.copilot.messages.push(ChatMessage::tool_response("inspect_canvas_image", result_json));
                auto_respond_turn = true;
            }
            other => {
                let prompt_text = format!("Unknown tool call: {other}");
                let action = crate::state::ToolCallAction {
                    name: fc.name.clone(),
                    args: fc.args.clone(),
                    status: crate::state::ToolActionStatus::Failed,
                    result: Some(json!({ "error": format!("Unsupported tool: {other}") })),
                    folded: false,
                    origin: ToolCallOrigin::GeminiApi,
                    call_id: None,
                };
                app.ui.copilot.messages.push(ChatMessage::tool_call(prompt_text, action));
            }
        }
    }
    auto_respond_turn
}

fn append_stream_delta(app: &mut PhotocraftApp, delta_text: String) {
    if delta_text.is_empty() {
        return;
    }
    if let Some(last) = app.ui.copilot.messages.last_mut()
        && last.role == ChatRole::Model
        && last.tool_call.is_none()
        && last.workflow_plan.is_none()
    {
        last.content.push_str(&delta_text);
    } else {
        app.ui.copilot.messages.push(ChatMessage::model(delta_text));
    }
}

fn finish_stream_result(
    app: &mut PhotocraftApp,
    res: Result<(), AiBackendError>,
    pending_fcs: Vec<crate::gemini::FunctionCall>,
) -> bool {
    let mut auto_respond_turn = false;
    match res {
        Ok(()) => {
            finalize_in_flight_cli_tools(&mut app.ui.copilot, ToolActionStatus::Completed);
            app.ui.copilot.clear_error();
            auto_respond_turn = handle_finished_function_calls(app, pending_fcs);
        }
        Err(err) if err.is_cancelled() => {
            finalize_in_flight_cli_tools(&mut app.ui.copilot, ToolActionStatus::Cancelled);
            app.ui.copilot.clear_error();
        }
        Err(err) => {
            finalize_in_flight_cli_tools(&mut app.ui.copilot, ToolActionStatus::Failed);
            app.ui.copilot.set_backend_error(err);
        }
    }
    app.ui.copilot.sync_active_to_session();
    auto_respond_turn
}

fn finish_test_result(app: &mut PhotocraftApp, res: Result<String, AiBackendError>) {
    match res {
        Ok(detail) => {
            let provider = active_provider(app);
            let status_text = if provider.is_cli() && !detail.trim().is_empty() {
                detail
            } else {
                crate::i18n::fmt(
                    tl!("Connection test successful! {provider} is ready."),
                    &[("provider", provider.display_name())],
                )
            };
            app.ui.copilot.connection_status = Some(status_text);
            app.ui.copilot.clear_error();
        }
        Err(err) if err.is_cancelled() => {
            app.ui.copilot.connection_status = None;
            app.ui.copilot.clear_error();
        }
        Err(err) => {
            app.ui.copilot.connection_status = None;
            app.ui.copilot.error_message = Some(format!("Connection test failed: {err}"));
            app.ui.copilot.last_error = Some(err);
        }
    }
}

/// Polls for completed background responses from AI worker threads.
fn check_background_response(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let channel = ctx.data_mut(|d| d.get_temp::<CopilotChannel>(copilot_channel_id()));
    let Some(channel) = channel else { return };
    let mut guard = channel.lock().unwrap_or_else(PoisonError::into_inner);

    let mut auto_respond_turn = false;
    let mut received_progress = false;

    for _ in 0..64 {
        let Some(st) = guard.as_mut() else { break };
        match st.rx.try_recv() {
            Ok(AiBackendEvent::StreamChunk { delta_text, function_calls }) => {
                app.ui.copilot.is_streaming = true;
                append_stream_delta(app, delta_text);
                if !function_calls.is_empty() {
                    st.pending_function_calls.extend(function_calls);
                }
                received_progress = true;
            }
            Ok(AiBackendEvent::SessionId(session_id)) => {
                let provider = active_provider(app);
                app.ui.copilot.set_active_cli_session_id(provider, session_id);
                received_progress = true;
            }
            Ok(AiBackendEvent::ToolUpdate(update)) => {
                app.ui.copilot.is_streaming = true;
                let finished_tool = matches!(update.state, CliToolState::Done | CliToolState::Failed);
                apply_cli_tool_update(&mut app.ui.copilot, update);
                received_progress = true;
                if finished_tool {
                    ctx.request_repaint();
                }
            }
            Ok(AiBackendEvent::StreamFinished(res)) => {
                let pending_fcs = std::mem::take(&mut st.pending_function_calls);
                *guard = None;
                app.ui.copilot.is_loading = false;
                app.ui.copilot.is_streaming = false;
                auto_respond_turn = finish_stream_result(app, res, pending_fcs);
                received_progress = true;
                break;
            }
            Ok(AiBackendEvent::Turn(res)) => {
                *guard = None;
                app.ui.copilot.is_loading = false;
                app.ui.copilot.is_streaming = false;
                match res {
                    Ok(turn) => {
                        app.ui.copilot.clear_error();
                        if turn.function_calls.is_empty()
                            && let Some(text) = turn.text
                        {
                            app.ui.copilot.messages.push(ChatMessage::model(text));
                        }
                    }
                    Err(err) if err.is_cancelled() => {
                        app.ui.copilot.clear_error();
                    }
                    Err(err) => {
                        app.ui.copilot.set_backend_error(err);
                    }
                }
                app.ui.copilot.sync_active_to_session();
                break;
            }
            Ok(AiBackendEvent::TestResult(res)) => {
                *guard = None;
                app.ui.copilot.is_loading = false;
                app.ui.copilot.is_streaming = false;
                finish_test_result(app, res);
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => break,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                *guard = None;
                app.ui.copilot.is_loading = false;
                app.ui.copilot.is_streaming = false;
                break;
            }
        }
    }

    drop(guard);

    if received_progress {
        ctx.request_repaint();
    }

    if auto_respond_turn {
        send_chat_turn(app, ctx);
    }
}

fn build_request_config(app: &mut PhotocraftApp) -> AiRequestConfig {
    let provider = active_provider(app);
    let cli_session_id = app.ui.copilot.active_cli_session_id(provider);
    let cli_override = cli_override_for_provider(app, provider).map(str::to_string);
    let mut cfg = AiRequestConfig::new(provider, &app.ui.gemini_api_key, &app.ui.gemini_model)
        .with_cli_session_id(cli_session_id)
        .with_cli_binary_override(cli_override);

    if provider.is_cli() {
        let control_listening = inspect_mcp_control_state(app).is_listening();
        let pc_override = photocraft_cli_override(app).map(str::to_string);
        let photocraft_cli_path = cached_photocraft_cli_status_with_override(&mut app.ui.copilot, pc_override.as_deref())
            .path()
            .map(std::path::Path::to_path_buf);
        let effective_mcp = app.ui.copilot.mcp_enabled && control_listening && photocraft_cli_path.is_some();
        let effective_auto_approve = effective_mcp && app.ui.copilot.auto_approve_mcp_tools;
        cfg = cfg.with_mcp(effective_mcp, effective_auto_approve, photocraft_cli_path);
    }

    cfg
}

fn make_wake_callback(ctx: &egui::Context) -> WakeCallback {
    let ctx_clone = ctx.clone();
    Arc::new(move || ctx_clone.request_repaint())
}

/// Sends the user's chat message through the active AI backend.
pub fn send_chat(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let input = app.ui.copilot.input.trim().to_string();
    let has_attachment = app.ui.copilot.pending_attachment.is_some();
    if input.is_empty() && !has_attachment {
        return;
    }

    let config = build_request_config(app);
    if let Err(err) = crate::ai_backend::validate_ready(&config) {
        if config.provider.is_cli() && matches!(err, AiBackendError::CliNotFound { .. }) {
            app.ui
                .copilot
                .cli_detection_cache
                .insert(config.provider, CliDetectionStatus::NotFound);
        }
        app.ui.copilot.is_loading = false;
        app.ui.copilot.is_streaming = false;
        app.ui.copilot.connection_status = None;
        app.ui.copilot.set_backend_error(err);
        return;
    }

    let attachment = app.ui.copilot.pending_attachment.take();
    let content = if input.is_empty() {
        tl!("Please inspect and analyze this canvas image.").to_string()
    } else {
        input
    };

    let msg = match attachment {
        Some(att) => ChatMessage::user_with_image(content, att),
        None => ChatMessage::user(content),
    };

    app.ui.copilot.messages.push(msg);
    app.ui.copilot.input.clear();
    app.ui.copilot.is_loading = true;
    app.ui.copilot.is_streaming = true;
    app.ui.copilot.clear_error();
    app.ui.copilot.connection_status = None;
    app.ui.copilot.sync_active_to_session();

    send_chat_turn(app, ctx);
}

/// Dispatches the current conversation turns through the unified AI backend abstraction.
pub fn send_chat_turn(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let config = build_request_config(app);
    match crate::ai_backend::start_stream_turn(&config, &app.ui.copilot.messages, make_wake_callback(ctx)) {
        Ok(worker_handle) => {
            let channel: CopilotChannel = Arc::new(Mutex::new(Some(worker_handle)));
            ctx.data_mut(|d| d.insert_temp(copilot_channel_id(), channel));
            app.ui.copilot.is_loading = true;
            app.ui.copilot.is_streaming = true;
            app.ui.copilot.clear_error();
        }
        Err(err) => {
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;
            app.ui.copilot.set_backend_error(err);
        }
    }
}

/// Triggers a test connection request through the active AI backend.
pub fn test_connection(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let config = build_request_config(app);
    if config.provider.is_cli() {
        let cli_ov = cli_override_for_provider(app, config.provider).map(str::to_string);
        let pc_ov = photocraft_cli_override(app).map(str::to_string);
        refresh_cli_detection_with_override(&mut app.ui.copilot, config.provider, cli_ov.as_deref());
        refresh_photocraft_cli_detection_with_override(&mut app.ui.copilot, pc_ov.as_deref());
    }
    match crate::ai_backend::start_test_connection(&config, make_wake_callback(ctx)) {
        Ok(worker_handle) => {
            let channel: CopilotChannel = Arc::new(Mutex::new(Some(worker_handle)));
            ctx.data_mut(|d| d.insert_temp(copilot_channel_id(), channel));
            app.ui.copilot.is_loading = true;
            app.ui.copilot.connection_status = Some(crate::i18n::fmt(
                tl!("Testing connection to {provider}…"),
                &[("provider", config.provider.display_name())],
            ));
            app.ui.copilot.clear_error();
        }
        Err(err) => {
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;
            app.ui.copilot.connection_status = None;
            app.ui.copilot.set_backend_error(err);
        }
    }
}

/// Renders the multi-session management and history drawer.
fn render_history_drawer(
    copilot: &mut crate::state::CopilotState,
    ui: &mut egui::Ui,
    t: &Tokens,
    switch_session: &mut Option<String>,
    delete_session: &mut Option<String>,
    new_chat: &mut bool,
) {
    let frame = egui::Frame::NONE
        .fill(t.field)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::same(8));

    frame.show(ui, |ui| {
        // Drawer Header
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "clock", 12.0, t.accent);
            ui.add_space(16.0);
            ui.label(RichText::new(tl!("Chat History")).strong().size(11.5).color(t.text));

            let token_count = crate::state::estimate_session_tokens(&copilot.messages);
            let pill = egui::Frame::NONE
                .fill(t.card)
                .corner_radius(CornerRadius::same(t.radius_sm as u8))
                .inner_margin(Margin::symmetric(4, 2));
            pill.show(ui, |ui| {
                ui.label(RichText::new(format!("~{token_count} toks / 32k")).size(9.0).color(t.text_dim));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icons::button(ui, "x", 16.0, false, tl!("Close drawer")).clicked() {
                    copilot.history_drawer_open = false;
                }
            });
        });

        ui.add_space(4.0);

        // Actions: + New Chat, Export Markdown, Export JSON
        ui.horizontal(|ui| {
            if widgets::secondary_button(ui, tl!("+ New Chat"), 0.0).clicked() {
                *new_chat = true;
            }
            if ui.small_button(RichText::new(tl!("Export MD")).size(9.5).color(t.text)).clicked() {
                let md = copilot.export_markdown(None);
                ui.ctx().copy_text(md);
                copilot.connection_status = Some("Markdown conversation copied to clipboard!".to_string());
            }
            if ui.small_button(RichText::new(tl!("Export JSON")).size(9.5).color(t.text)).clicked()
                && let Ok(json_str) = copilot.export_all_json()
            {
                ui.ctx().copy_text(json_str);
                copilot.connection_status = Some("Sessions JSON copied to clipboard!".to_string());
            }
        });

        ui.add_space(6.0);

        // Sessions list
        let list_max_h = 160.0;
        egui::ScrollArea::vertical()
            .max_height(list_max_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let sessions_len = copilot.sessions.len();
                let mut rename_target = None;
                let mut save_rename = false;
                let mut cancel_rename = false;

                for s in &copilot.sessions {
                    let is_active = s.id == copilot.active_session_id;
                    let is_renaming = copilot.renaming_session_id.as_deref() == Some(&s.id);

                    let item_bg = if is_active { t.card } else { t.dock };
                    let item_border = if is_active { t.accent_border } else { t.card_border };

                    let item_frame = egui::Frame::NONE
                        .fill(item_bg)
                        .stroke(Stroke::new(1.0, item_border))
                        .corner_radius(CornerRadius::same(t.radius_sm as u8))
                        .inner_margin(Margin::symmetric(6, 4));

                    item_frame.show(ui, |ui| {
                        if is_renaming {
                            ui.horizontal(|ui| {
                                let edit_resp = TextField::new(tl!("Session name"), Bind::bound(&mut copilot.rename_input))
                                    .submits()
                                    .id_salt(("copilot-session-rename", &s.id))
                                    .show(ui);
                                if edit_resp.submitted {
                                    save_rename = true;
                                }
                                if edit_resp.escaped {
                                    cancel_rename = true;
                                }
                                if icons::button(ui, "check", 14.0, false, tl!("Save name")).clicked() {
                                    save_rename = true;
                                }
                                if icons::button(ui, "x", 14.0, false, tl!("Cancel")).clicked() {
                                    cancel_rename = true;
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                let icon = if is_active { "sparkles" } else { "message-square" };
                                let icon_color = if is_active { t.accent } else { t.text_dim };
                                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), icon, 10.0, icon_color);
                                ui.add_space(14.0);

                                let label_color = if is_active { t.accent_text } else { t.text };
                                let title_resp = ui.add(
                                    egui::Label::new(RichText::new(&s.title).size(10.5).color(label_color))
                                        .sense(Sense::click()),
                                );
                                if title_resp.clicked() && !is_active {
                                    *switch_session = Some(s.id.clone());
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if sessions_len > 1
                                        && icons::button(ui, "trash", 14.0, false, tl!("Delete session")).clicked()
                                    {
                                        *delete_session = Some(s.id.clone());
                                    }
                                    if icons::button(ui, "pencil", 14.0, false, tl!("Rename session")).clicked() {
                                        rename_target = Some((s.id.clone(), s.title.clone()));
                                    }
                                    let s_toks = s.estimated_tokens();
                                    ui.label(RichText::new(format!("~{s_toks}")).size(9.0).color(t.text_dim));
                                });
                            });
                        }
                    });
                    ui.add_space(2.0);
                }

                if let Some((id, title)) = rename_target {
                    copilot.renaming_session_id = Some(id);
                    copilot.rename_input = title;
                }
                if save_rename
                    && let Some(id) = copilot.renaming_session_id.take()
                {
                    let new_name = copilot.rename_input.clone();
                    copilot.rename_session(&id, &new_name);
                }
                if cancel_rename {
                    copilot.renaming_session_id = None;
                    copilot.rename_input.clear();
                }
            });
    });
}

/// Deferred UI actions collected while drawing the AI sidebar or floating window.
#[derive(Default)]
struct PanelActions {
    close: bool,
    toggle_docked: bool,
    switch_provider: Option<AiProviderKind>,
    redetect_cli: bool,
    enable_mcp_control: bool,
    set_mcp_enabled: Option<bool>,
    set_auto_approve_mcp: Option<bool>,
    clear_chat: bool,
    do_test_connection: bool,
    open_prefs: bool,
    send_prompt: bool,
    capture_screenshot: bool,
    open_expanded_image: Option<usize>,
    suggestion_to_send: Option<String>,
    preset_to_send: Option<String>,
    action_to_execute: Option<usize>,
    action_to_cancel: Option<usize>,
    toggle_folded: Option<usize>,
    execute_workflow_step: Option<(usize, usize)>,
    skip_workflow_step: Option<(usize, usize)>,
    execute_workflow_all: Option<usize>,
    stop_generation: bool,
    new_chat: bool,
    switch_session_id: Option<String>,
    delete_session_id: Option<String>,
}

/// Non-overlapping horizontal partition of `AppShell::body` into workspace (left),
/// resize divider (1 px visual seam), and right-docked AI sidebar (right).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockedSidebarSplit {
    pub workspace: Rect,
    pub divider: Rect,
    pub sidebar: Rect,
}

/// Computes disjoint `(workspace, divider, sidebar)` rectangles inside `body` so `panel_docking`
/// (including Color / Properties / Layers) always stays completely to the left of the AI sidebar.
pub fn split_body_rect(body: Rect, desired_sidebar_width: f32) -> DockedSidebarSplit {
    let total_w = body.width().max(0.0);
    let max_sidebar_w = MAX_COPILOT_SIDEBAR_WIDTH
        .min((total_w - MIN_WORKSPACE_WIDTH).max(MIN_COPILOT_SIDEBAR_WIDTH))
        .min(total_w);
    let min_sidebar_w = MIN_COPILOT_SIDEBAR_WIDTH.min(max_sidebar_w);
    let sidebar_w = if desired_sidebar_width.is_finite() {
        desired_sidebar_width.clamp(min_sidebar_w, max_sidebar_w)
    } else {
        CopilotState::DEFAULT_SIDEBAR_WIDTH.clamp(min_sidebar_w, max_sidebar_w)
    };
    let split_x = (body.right() - sidebar_w).clamp(body.left(), body.right());
    let divider_left = (split_x - 1.0).max(body.left());

    DockedSidebarSplit {
        workspace: Rect::from_min_max(body.min, pos2(divider_left, body.bottom())),
        divider: Rect::from_min_max(pos2(divider_left, body.top()), pos2(split_x, body.bottom())),
        sidebar: Rect::from_min_max(pos2(split_x, body.top()), body.max),
    }
}

/// Renders the interactive vertical resize divider between the workspace and right AI sidebar.
fn render_sidebar_divider(
    app: &mut PhotocraftApp,
    ui: &mut egui::Ui,
    t: &Tokens,
    body: Rect,
    split: &DockedSidebarSplit,
) {
    let seam_x = split.sidebar.left();
    let hit_rect = Rect::from_min_max(
        pos2((seam_x - SIDEBAR_DIVIDER_HIT_WIDTH * 0.5).max(body.left()), body.top()),
        pos2((seam_x + SIDEBAR_DIVIDER_HIT_WIDTH * 0.5).min(body.right()), body.bottom()),
    );
    let divider_id = egui::Id::unique("photocraft-ai-sidebar-divider");
    let resp = ui
        .interact(hit_rect, divider_id, Sense::click_and_drag())
        .accessible_name(tl!("Resize AI Assistant sidebar"));

    let active = resp.dragged();
    let hovered = resp.hovered() || resp.has_focus();
    if hovered || active {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }

    let max_w = MAX_COPILOT_SIDEBAR_WIDTH
        .min((body.width() - MIN_WORKSPACE_WIDTH).max(MIN_COPILOT_SIDEBAR_WIDTH))
        .min(body.width());
    let min_w = MIN_COPILOT_SIDEBAR_WIDTH.min(max_w);

    if active
        && let Some(pointer_pos) = resp.interact_pointer_pos()
    {
        let next_w = (body.right() - pointer_pos.x).clamp(min_w, max_w);
        app.ui.copilot.sidebar_width = next_w.clamp(MIN_COPILOT_SIDEBAR_WIDTH, MAX_COPILOT_SIDEBAR_WIDTH);
        ui.ctx().request_repaint();
    } else if resp.double_clicked() {
        app.ui.copilot.sidebar_width = CopilotState::DEFAULT_SIDEBAR_WIDTH;
        ui.ctx().request_repaint();
    } else if resp.has_focus() {
        let step = if ui.input(|i| i.modifiers.shift) { 32.0 } else { 12.0 };
        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            app.ui.copilot.sidebar_width = (app.ui.copilot.clamped_sidebar_width() + step).clamp(min_w, max_w);
            ui.ctx().request_repaint();
        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            app.ui.copilot.sidebar_width = (app.ui.copilot.clamped_sidebar_width() - step).clamp(min_w, max_w);
            ui.ctx().request_repaint();
        }
    }

    let (line_color, line_w) = if active || hovered {
        (t.accent, 2.0)
    } else {
        (t.card_border, 1.0)
    };
    ui.painter().line_segment(
        [pos2(seam_x, body.top()), pos2(seam_x, body.bottom())],
        Stroke::new(line_w, line_color),
    );
}

/// Returns whether `ui` is currently running a sizing or `craft` layout measuring sweep.
fn is_measuring_pass(ui: &egui::Ui) -> bool {
    ui.is_sizing_pass() || craft::ui::runtime::measure::active(ui.ctx())
}

/// Renders the docked AI sidebar body inside an exact, clipped `craft` container rectangle so its
/// widgets get an isolated `Env` (`fill_body` / `fill_share`) and never leak into `AppShell::body`.
fn render_docked_sidebar_contents(app: &mut PhotocraftApp, ui: &mut egui::Ui, sidebar_rect: Rect) {
    let ctx = ui.ctx().clone();
    let parent_measuring = is_measuring_pass(ui);
    if !parent_measuring {
        check_background_response(app, &ctx);
        app.ui.copilot.ensure_active_session();
    }

    let t = Tokens::get(&ctx);
    let provider = active_provider(app);

    if !parent_measuring {
        ui.painter().rect_filled(sidebar_rect, CornerRadius::ZERO, t.chrome);
    }

    let pad = 10.0;
    let inner_rect = Rect::from_min_max(
        pos2((sidebar_rect.left() + pad).min(sidebar_rect.right()), (sidebar_rect.top() + pad).min(sidebar_rect.bottom())),
        pos2((sidebar_rect.right() - pad).max(sidebar_rect.left()), (sidebar_rect.bottom() - pad).max(sidebar_rect.top())),
    );
    let mut actions: Option<PanelActions> = None;
    let _ = Stack::new()
        .id_salt("photocraft-ai-sidebar")
        .pad(Pad::None)
        .show_at(ui, inner_rect, |ui, _| {
            let measuring = is_measuring_pass(ui);
            ui.set_clip_rect(sidebar_rect.intersect(ui.clip_rect()));
            let inner_w = inner_rect.width().max(120.0);
            ui.set_max_width(inner_w);
            ui.set_width(inner_w);
            let mut run_actions = if measuring { PanelActions::default() } else { collect_shortcuts(&ctx) };
            render_panel_body(app, ui, &t, provider, &mut run_actions);
            if !measuring {
                actions = Some(run_actions);
            }
        });

    if !parent_measuring
        && let Some(actions) = actions
    {
        apply_panel_actions(app, &ctx, actions);
    }
}

/// Partitions `AppShell::body` into disjoint left workspace (`show_workspace`) and right-docked
/// AI sidebar rects when the AI sidebar is open and docked. Returns `true` when docked sidebar ran.
///
/// Each side is hosted in its own `craft::ui::layout_components::Stack` placed via
/// `ShowContainer::show_at` so both regions receive an isolated `Env` (`fill_body` / `fill_share`
/// reset via `Env::start_room`) and `ShowAt::Rect` takes no room in `AppShell::body`'s vertical flow.
pub fn show_body_with_docked_sidebar<F>(app: &mut PhotocraftApp, ui: &mut egui::Ui, mut show_workspace: F) -> bool
where
    F: FnMut(&mut PhotocraftApp, &mut egui::Ui),
{
    if !app.ui.copilot.open || !app.ui.copilot.docked {
        show_workspace(app, ui);
        return false;
    }

    let body = ui.available_rect_before_wrap();
    let split = split_body_rect(body, app.ui.copilot.clamped_sidebar_width());

    // 1. Render workspace (DockArea with canvas + Color / Properties / Layers) strictly in the left rect.
    // Using a craft `Stack` via `show_at` isolates `DockRoot`'s `fill_body` so it receives 100% of
    // `split.workspace`'s height without being affected by sidebar controls.
    let _ = Stack::new()
        .id_salt("photocraft-workspace-beside-ai")
        .pad(Pad::None)
        .gap(Gap::None)
        .show_at(ui, split.workspace, |ui, _| {
            ui.set_clip_rect(split.workspace.intersect(ui.clip_rect()));
            show_workspace(app, ui);
        });

    // 2. Render docked AI sidebar strictly in the right rect inside its own isolated craft `Stack`.
    render_docked_sidebar_contents(app, ui, split.sidebar);

    // 3. Render interactive resize divider between workspace and AI sidebar (real pass only).
    if !is_measuring_pass(ui) {
        let t = Tokens::get(ui.ctx());
        render_sidebar_divider(app, ui, &t, body, &split);
    }

    true
}

/// Renders the AI assistant as a right-docked resizable sidebar inside `ui`.
///
/// Kept as a convenience wrapper around [`show_body_with_docked_sidebar`] for standalone tests.
pub fn show_docked_sidebar(app: &mut PhotocraftApp, ui: &mut egui::Ui) -> bool {
    show_body_with_docked_sidebar(app, ui, |_, _| {})
}

/// Renders floating AI assistant windows (detached window mode and expanded image previews).
pub fn windows(app: &mut PhotocraftApp, ctx: &egui::Context) {
    check_background_response(app, ctx);

    if !app.ui.copilot.open {
        return;
    }

    if !app.ui.copilot.docked {
        app.ui.copilot.ensure_active_session();

        let t = Tokens::get(ctx);
        let provider = active_provider(app);
        let canvas = app.last_canvas_rect;
        let width = app.ui.copilot.clamped_sidebar_width();
        let default_x = (canvas.right() - width - 8.0).max(canvas.left() + 8.0);
        let default_y = (canvas.top() + 8.0).max(0.0);

        let frame = egui::Frame::NONE
            .fill(t.chrome)
            .stroke(Stroke::new(1.0, t.card_border))
            .corner_radius(CornerRadius::same(t.radius_lg as u8))
            .shadow(egui::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: t.shadow,
            })
            .inner_margin(Margin::same(10));

        let mut actions = collect_shortcuts(ctx);

        egui::Window::new(tl!("AI Assistant"))
            .id(egui::Id::unique("photocraft-gemini-copilot-window"))
            .title_bar(false)
            .resizable(true)
            .movable(!app.session.prefs().workspace_locked)
            .default_pos(pos2(default_x, default_y))
            .default_size(vec2(width, 520.0))
            .min_width(MIN_COPILOT_SIDEBAR_WIDTH)
            .max_width(MAX_COPILOT_SIDEBAR_WIDTH)
            .min_height(320.0)
            .frame(frame)
            .show(ctx, |ui| {
                render_panel_body(app, ui, &t, provider, &mut actions);
            });

        apply_panel_actions(app, ctx, actions);
    }

    show_modals(app, ctx);
}

/// Renders overlay modals owned by the AI assistant (such as the expanded canvas screenshot preview).
pub fn show_modals(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if !app.ui.copilot.open {
        return;
    }
    if let Some(idx) = app.ui.copilot.expanded_image_preview {
        let t = Tokens::get(ctx);
        render_expanded_image_modal(app, ctx, &t, idx);
    }
}

fn collect_shortcuts(ctx: &egui::Context) -> PanelActions {
    let mut actions = PanelActions::default();
    // Shortcut Ctrl+Shift+C / Cmd+Shift+C to toggle/attach canvas screenshot
    if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::C)) {
        actions.capture_screenshot = true;
    }
    actions
}

/// Shared body rendered inside both the right-docked sidebar and the floating window.
fn render_panel_body(
    app: &mut PhotocraftApp,
    ui: &mut egui::Ui,
    t: &Tokens,
    provider: AiProviderKind,
    actions: &mut PanelActions,
) {
    let panel_width = ui.available_width().max(120.0);
    ui.set_max_width(panel_width);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

    render_header(app, ui, t, actions);
    ui.add_space(4.0);

    let (cli_status, photocraft_cli_status, mcp_control) = if provider.is_cli() {
        let cli_ov = cli_override_for_provider(app, provider).map(str::to_string);
        let pc_ov = photocraft_cli_override(app).map(str::to_string);
        let cli = cached_cli_status_with_override(&mut app.ui.copilot, provider, cli_ov.as_deref());
        let bridge_cli = cached_photocraft_cli_status_with_override(&mut app.ui.copilot, pc_ov.as_deref());
        let control = inspect_mcp_control_state(app);
        (Some(cli), Some(bridge_cli), Some(control))
    } else {
        (None, None, None)
    };

    render_provider_bar(app, ui, t, provider, cli_status.as_ref(), actions);
    ui.add_space(4.0);

    if app.ui.copilot.history_drawer_open {
        render_history_drawer(
            &mut app.ui.copilot,
            ui,
            t,
            &mut actions.switch_session_id,
            &mut actions.delete_session_id,
            &mut actions.new_chat,
        );
        ui.add_space(4.0);
    }

    widgets::hairline(ui);
    ui.add_space(4.0);

    let cli_view = match (&cli_status, &photocraft_cli_status, &mcp_control) {
        (Some(cli), Some(photocraft_cli), Some(control)) => Some(CliStatusView {
            cli,
            photocraft_cli,
            control,
        }),
        _ => None,
    };

    render_status_banners(app, ui, t, provider, cli_view.as_ref(), actions);

    // Chat conversation scroll area
    let composer_reserve = if app.ui.copilot.pending_attachment.is_some() { 132.0 } else { 100.0 };
    let scroll_height = (ui.available_height() - composer_reserve).max(100.0);
    egui::ScrollArea::vertical()
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if app.ui.copilot.messages.is_empty() {
                render_empty_state(ui, t, &mut actions.suggestion_to_send);
            } else {
                let is_streaming = app.ui.copilot.is_streaming;
                let last_model_idx = app
                    .ui
                    .copilot
                    .messages
                    .iter()
                    .rposition(|m| m.role == ChatRole::Model && m.tool_call.is_none() && m.workflow_plan.is_none());

                let mut dispatch = ToolCardDispatch {
                    execute: &mut actions.action_to_execute,
                    cancel: &mut actions.action_to_cancel,
                    toggle_folded: &mut actions.toggle_folded,
                    open_expanded: &mut actions.open_expanded_image,
                    execute_workflow_step: &mut actions.execute_workflow_step,
                    skip_workflow_step: &mut actions.skip_workflow_step,
                    execute_workflow_all: &mut actions.execute_workflow_all,
                    is_streaming,
                    is_last_model_msg: false,
                };
                for (idx, msg) in app.ui.copilot.messages.iter().enumerate() {
                    dispatch.is_last_model_msg = Some(idx) == last_model_idx;
                    render_chat_message(ui, idx, msg, t, &mut dispatch);
                    ui.add_space(6.0);
                }
            }

            if app.ui.copilot.is_loading && !app.ui.copilot.is_streaming {
                render_loading_bubble(ui, t, provider);
                ui.add_space(4.0);
            }
        });

    ui.add_space(4.0);
    widgets::hairline(ui);
    ui.add_space(3.0);

    let has_doc = app.session.active().is_some();
    render_composer(&mut app.ui.copilot, has_doc, ui, t, actions);
}

/// Renders the panel title bar with session switcher, new chat, dock/float toggle, and controls.
fn render_header(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens, actions: &mut PanelActions) {
    ui.horizontal(|ui| {
        icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(18.0, 18.0)), "sparkles", 16.0, t.accent);
        ui.add_space(20.0);
        ui.label(RichText::new(tl!("AI Assistant")).strong().size(13.0).color(t.text));

        // Session switcher button / drawer toggle
        let active_title = app
            .ui
            .copilot
            .sessions
            .iter()
            .find(|s| s.id == app.ui.copilot.active_session_id)
            .map(|s| s.title.as_str())
            .unwrap_or("Chat");
        let truncated_title = if active_title.chars().count() > 10 {
            let mut s: String = active_title.chars().take(8).collect();
            s.push('…');
            s
        } else {
            active_title.to_string()
        };

        let session_chip = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, if app.ui.copilot.history_drawer_open { t.accent_border } else { t.field_border }))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(6, 2));

        let chip_resp = session_chip.show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(10.0, 10.0)), "clock", 9.0, t.text_dim);
                ui.add_space(12.0);
                ui.label(RichText::new(truncated_title).size(9.5).color(t.text));
                let chevron = if app.ui.copilot.history_drawer_open { "chevron_up" } else { "chevron_down" };
                let (chev_rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                icons::paint(ui, chev_rect, chevron, 9.0, t.text_dim);
            });
        });
        if chip_resp
            .response
            .interact(Sense::click())
            .accessible_name(tl!("Switch chat session"))
            .clicked()
        {
            app.ui.copilot.history_drawer_open = !app.ui.copilot.history_drawer_open;
        }

        // New Chat button
        if icons::button(ui, "plus", 18.0, false, tl!("New Chat")).clicked() {
            actions.new_chat = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icons::button(ui, "x", 20.0, false, tl!("Close")).clicked() {
                actions.close = true;
            }
            let (dock_icon, dock_tooltip) = if app.ui.copilot.docked {
                ("float", tl!("Undock to floating window"))
            } else {
                ("panel_right", tl!("Dock to right sidebar"))
            };
            if icons::button(ui, dock_icon, 20.0, false, dock_tooltip).clicked() {
                actions.toggle_docked = true;
            }
            if icons::button(ui, "trash", 20.0, false, tl!("Clear chat")).clicked() {
                actions.clear_chat = true;
            }
            if icons::button(ui, "rotate-cw", 20.0, false, tl!("Test connection")).clicked() {
                actions.do_test_connection = true;
            }
        });
    });
}

/// Renders the compact AI engine selector dropdown synced with `integrations.aiProvider`.
fn render_provider_bar(
    app: &PhotocraftApp,
    ui: &mut egui::Ui,
    t: &Tokens,
    provider: AiProviderKind,
    cli_status: Option<&CliDetectionStatus>,
    actions: &mut PanelActions,
) {
    let bar_frame = egui::Frame::NONE
        .fill(t.field)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::symmetric(8, 4));

    bar_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(tl!("Engine")).size(10.5).color(t.text_dim));
            let entries: Vec<Entry<AiProviderKind>> = AiProviderKind::ALL
                .iter()
                .map(|&kind| {
                    Entry::new(kind, tl!(kind.selector_label()).to_string())
                        .closed_label(tl!(kind.display_name()).to_string())
                })
                .collect();
            if let Some(picked) = Select::new(tl!("AI Engine").to_string(), Bind::given(provider), &entries)
                .id_salt("copilot-ai-provider-select")
                .show(ui)
                .value()
                .copied()
                && picked != provider
            {
                actions.switch_provider = Some(picked);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if provider == AiProviderKind::GeminiApi {
                    ui.label(RichText::new(&app.ui.gemini_model).size(9.5).color(t.text_dim));
                } else {
                    let (badge_text, badge_color) = match cli_status {
                        Some(CliDetectionStatus::Found(_)) => (tl!("CLI · Ready"), t.accent_text),
                        Some(CliDetectionStatus::UnsupportedPlatform) => (tl!("CLI · Desktop only"), t.text_dim),
                        _ => (tl!("CLI · Not found"), t.warning),
                    };
                    ui.label(RichText::new(badge_text).size(9.5).color(badge_color));
                }
            });
        });
    });
}

/// Renders the compact local CLI status strip with executable name and Re-detect button.
fn render_cli_status_strip(
    ui: &mut egui::Ui,
    t: &Tokens,
    provider: AiProviderKind,
    cli_status: &CliDetectionStatus,
    actions: &mut PanelActions,
) {
    let border_color = match cli_status {
        CliDetectionStatus::Found(_) => t.field_border,
        CliDetectionStatus::NotFound => t.warning,
        CliDetectionStatus::UnsupportedPlatform => t.field_border,
    };

    let strip_frame = egui::Frame::NONE
        .fill(t.field)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::symmetric(8, 5));

    strip_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            match cli_status {
                CliDetectionStatus::Found(path) => {
                    let exe = cli_status
                        .display_file_name()
                        .unwrap_or_else(|| provider.cli_executable().unwrap_or("cli").to_string());
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "check", 11.0, t.accent);
                    ui.add_space(14.0);
                    let status_text = crate::i18n::fmt(tl!("{exe} detected"), &[("exe", &exe)]);
                    ui.label(RichText::new(status_text).size(10.0).color(t.text))
                        .on_hover_text(path.display().to_string());
                }
                CliDetectionStatus::NotFound => {
                    let exe = provider.cli_executable().unwrap_or("cli");
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "triangle-alert", 11.0, t.warning);
                    ui.add_space(14.0);
                    let status_text = crate::i18n::fmt(tl!("{exe} not found"), &[("exe", exe)]);
                    ui.label(RichText::new(status_text).strong().size(10.0).color(t.warning));
                }
                CliDetectionStatus::UnsupportedPlatform => {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "info", 11.0, t.text_dim);
                    ui.add_space(14.0);
                    ui.label(RichText::new(tl!("Local CLI agents require the desktop app.")).size(10.0).color(t.text_dim));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(RichText::new(tl!("Re-detect")).size(9.5).color(t.accent_text))
                    .on_hover_text(tl!("Re-scan PATH for local CLI binary"))
                    .clicked()
                {
                    actions.redetect_cli = true;
                }
            });
        });

        if matches!(cli_status, CliDetectionStatus::NotFound)
            && let Some(install_hint) = provider.cli_install_hint()
        {
            ui.add_space(3.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(tl!("Install:")).size(9.5).color(t.text_dim));
                ui.add(egui::Label::new(RichText::new(install_hint).monospace().size(9.5).color(t.accent_text)).wrap());
            });
        }
    });
}

/// Renders a copyable monospace terminal command row inside guided error cards.
fn render_terminal_command_box(ui: &mut egui::Ui, t: &Tokens, command: &str) {
    let cmd_frame = egui::Frame::NONE
        .fill(t.dock)
        .stroke(Stroke::new(1.0, t.card_border))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::symmetric(6, 4));

    cmd_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            let text_w = (ui.available_width() - 28.0).max(80.0);
            ui.vertical(|ui| {
                ui.set_max_width(text_w);
                ui.add(egui::Label::new(RichText::new(command).monospace().size(10.0).color(t.accent_text)).wrap());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icons::button(ui, "copy", 14.0, false, tl!("Copy command")).clicked() {
                    ui.ctx().copy_text(command.to_string());
                }
            });
        });
    });
}

/// Renders structured recovery guidance inside the error card when available.
fn render_error_guidance(
    ui: &mut egui::Ui,
    t: &Tokens,
    provider: AiProviderKind,
    err_text: &str,
    last_error: Option<&AiBackendError>,
    actions: &mut PanelActions,
) {
    match last_error {
        Some(AiBackendError::CliNotFound { install_hint, .. }) => {
            ui.add_space(4.0);
            ui.label(RichText::new(tl!("1. Install the CLI in your terminal:")).size(10.0).color(t.text));
            ui.add_space(2.0);
            render_terminal_command_box(ui, t, install_hint);
            ui.add_space(3.0);
            ui.label(
                RichText::new(tl!("2. Run the CLI once to sign in with your account (no API key needed)."))
                    .size(10.0)
                    .color(t.text_dim),
            );
            ui.label(RichText::new(tl!("3. Click Re-detect below to refresh.")).size(10.0).color(t.text_dim));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, tl!("Re-detect"), 0.0).clicked() {
                    actions.redetect_cli = true;
                }
                if widgets::secondary_button(ui, tl!("Switch to Gemini API"), 0.0).clicked() {
                    actions.switch_provider = Some(AiProviderKind::GeminiApi);
                }
            });
        }
        Some(AiBackendError::CliNotAuthenticated { login_command, account_label, details, .. }) => {
            ui.add_space(4.0);
            ui.label(RichText::new(tl!("1. Open your system terminal.")).size(10.0).color(t.text));
            let step2 = crate::i18n::fmt(
                tl!("2. Run `{cmd}` once and complete sign-in with your {account} (no API key needed)."),
                &[("cmd", login_command), ("account", account_label)],
            );
            ui.label(RichText::new(step2).size(10.0).color(t.text));
            ui.add_space(2.0);
            render_terminal_command_box(ui, t, login_command);
            ui.add_space(3.0);
            ui.label(RichText::new(tl!("3. Return to PhotoCraft and send your message again.")).size(10.0).color(t.text_dim));
            if !details.is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new(details).monospace().size(9.5).color(t.text_dim));
            }
        }
        Some(AiBackendError::CliExecutionFailed { .. }) => {
            ui.add_space(4.0);
            if widgets::secondary_button(ui, tl!("Test connection"), 0.0).clicked() {
                actions.do_test_connection = true;
            }
        }
        _ => {
            if provider.requires_api_key() && (err_text.contains("API key") || err_text.contains("configured")) {
                ui.add_space(3.0);
                if widgets::secondary_button(ui, tl!("Open Preferences…"), 0.0).clicked() {
                    actions.open_prefs = true;
                }
            }
        }
    }
}

/// Renders the live PhotoCraft MCP control channel status strip, one-click enable banner,
/// `photocraft-cli` detection status, and explicit MCP/headless approval toggles.
fn render_mcp_control_strip(
    app: &PhotocraftApp,
    ui: &mut egui::Ui,
    t: &Tokens,
    photocraft_cli: &CliDetectionStatus,
    control_state: &McpControlState,
    actions: &mut PanelActions,
) {
    let cli_found = matches!(photocraft_cli, CliDetectionStatus::Found(_));
    let control_listening = control_state.is_listening();
    let mcp_ready = control_listening && cli_found;

    let border_color = match control_state {
        McpControlState::Error(_) => t.danger,
        McpControlState::Listening { .. } if cli_found => t.accent_border,
        McpControlState::Unavailable => t.field_border,
        _ => t.warning,
    };

    let strip_frame = egui::Frame::NONE
        .fill(t.field)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::symmetric(8, 6));

    strip_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        // Row 1: Control channel & photocraft-cli readiness
        ui.horizontal(|ui| {
            match control_state {
                McpControlState::Listening { port, .. } => {
                    let (icon, color) = if cli_found { ("check", t.accent) } else { ("triangle-alert", t.warning) };
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), icon, 11.0, color);
                    ui.add_space(14.0);
                    let label = match port {
                        Some(p) => crate::i18n::fmt(tl!("MCP Control · Port {port}"), &[("port", &p.to_string())]),
                        None => tl!("MCP Control · Connected").to_string(),
                    };
                    ui.label(RichText::new(label).strong().size(10.0).color(if cli_found { t.text } else { t.warning }));
                }
                McpControlState::Disabled { .. } => {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "triangle-alert", 11.0, t.warning);
                    ui.add_space(14.0);
                    ui.label(RichText::new(tl!("PhotoCraft MCP control is off")).strong().size(10.0).color(t.warning));
                }
                McpControlState::LauncherDisabled => {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "triangle-alert", 11.0, t.warning);
                    ui.add_space(14.0);
                    ui.label(RichText::new(tl!("MCP control disabled in ArtCraft Launcher")).strong().size(10.0).color(t.warning));
                }
                McpControlState::Error(_) => {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "triangle-alert", 11.0, t.danger);
                    ui.add_space(14.0);
                    ui.label(RichText::new(tl!("PhotoCraft MCP control error")).strong().size(10.0).color(t.danger));
                }
                McpControlState::Unavailable => {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "info", 11.0, t.text_dim);
                    ui.add_space(14.0);
                    ui.label(RichText::new(tl!("Live MCP control requires the desktop app.")).size(10.0).color(t.text_dim));
                }
            }

            if !matches!(control_state, McpControlState::Unavailable) {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| match photocraft_cli {
                    CliDetectionStatus::Found(path) => {
                        let name = photocraft_cli
                            .display_file_name()
                            .unwrap_or_else(|| "photocraft-cli".to_string());
                        ui.label(RichText::new(name).monospace().size(9.0).color(t.accent_text))
                            .on_hover_text(path.display().to_string());
                    }
                    CliDetectionStatus::NotFound => {
                        ui.label(RichText::new(tl!("photocraft-cli missing")).size(9.0).color(t.warning));
                    }
                    CliDetectionStatus::UnsupportedPlatform => {}
                });
            }
        });

        // Action banners when control channel or bridge binary is not ready
        match control_state {
            McpControlState::Disabled { .. } => {
                ui.add_space(3.0);
                ui.label(
                    RichText::new(tl!("Enable the local control channel so the CLI can inspect and edit this window via MCP."))
                        .size(9.5)
                        .color(t.text_dim),
                );
                ui.add_space(4.0);
                if widgets::primary_button(ui, tl!("Enable PhotoCraft MCP Control"), 0.0).clicked() {
                    actions.enable_mcp_control = true;
                }
            }
            McpControlState::LauncherDisabled => {
                ui.add_space(3.0);
                ui.label(
                    RichText::new(tl!("Enable PhotoCraft in ArtCraft Launcher › Settings › AI agents to allow live MCP control."))
                        .size(9.5)
                        .color(t.text_dim),
                );
                ui.add_space(4.0);
                if widgets::secondary_button(ui, tl!("Open Preferences…"), 0.0).clicked() {
                    actions.open_prefs = true;
                }
            }
            McpControlState::Error(err) => {
                ui.add_space(3.0);
                ui.label(RichText::new(err).size(9.5).color(t.danger));
                ui.add_space(4.0);
                if widgets::secondary_button(ui, tl!("Open Preferences…"), 0.0).clicked() {
                    actions.open_prefs = true;
                }
            }
            McpControlState::Listening { warning: Some(warn), .. } => {
                ui.add_space(2.0);
                ui.label(RichText::new(warn).size(9.0).color(t.warning));
            }
            McpControlState::Listening { warning: None, .. } | McpControlState::Unavailable => {}
        }

        if matches!(photocraft_cli, CliDetectionStatus::NotFound) && !matches!(control_state, McpControlState::Unavailable) {
            ui.add_space(3.0);
            ui.label(
                RichText::new(tl!("`photocraft-cli` was not found beside PhotoCraft or on PATH. Build/install `photocraft-cli` and click Re-detect."))
                    .size(9.5)
                    .color(t.warning),
            );
        }

        if matches!(control_state, McpControlState::Unavailable) {
            return;
        }

        ui.add_space(4.0);

        // Toggle 1: Allow CLI to control PhotoCraft (MCP)
        let mut mcp_on = app.ui.copilot.mcp_enabled;
        let prev_mcp_on = mcp_on;
        Checkbox::new(Bind::bound(&mut mcp_on))
            .label(tl!("Allow CLI to control PhotoCraft (MCP)"))
            .id_salt("copilot-mcp-enabled")
            .show(ui);
        if mcp_on != prev_mcp_on {
            actions.set_mcp_enabled = Some(mcp_on);
        }

        if mcp_on && !mcp_ready {
            let reason = if !control_listening {
                tl!("Waiting for PhotoCraft MCP control channel to be enabled.")
            } else {
                tl!("Waiting for `photocraft-cli` to be detected.")
            };
            ui.label(RichText::new(reason).size(9.0).color(t.text_dim));
        }

        // Toggle 2: Explicit headless tool auto-approval opt-in
        let mut auto_approve = app.ui.copilot.auto_approve_mcp_tools;
        let prev_auto = auto_approve;
        ui.add_enabled_ui(mcp_on && mcp_ready, |ui| {
            Checkbox::new(Bind::bound(&mut auto_approve))
                .label(tl!("Auto-approve headless tool calls"))
                .id_salt("copilot-mcp-auto-approve")
                .show(ui);
        });
        if auto_approve != prev_auto {
            actions.set_auto_approve_mcp = Some(auto_approve);
        }
        ui.add(
            egui::Label::new(
                RichText::new(tl!(
                    "Non-interactive CLIs cannot ask y/n in a terminal. Enable to let the AI invoke PhotoCraft MCP tools directly; leave off for default CLI permissions."
                ))
                .size(9.0)
                .color(t.text_dim),
            )
            .wrap(),
        );
    });
}

struct CliStatusView<'a> {
    cli: &'a CliDetectionStatus,
    photocraft_cli: &'a CliDetectionStatus,
    control: &'a McpControlState,
}

/// Renders provider readiness, connection status, and error banners.
fn render_status_banners(
    app: &PhotocraftApp,
    ui: &mut egui::Ui,
    t: &Tokens,
    provider: AiProviderKind,
    cli_view: Option<&CliStatusView<'_>>,
    actions: &mut PanelActions,
) {
    // Missing Gemini API key warning is shown only when Gemini API mode is selected.
    if provider.requires_api_key() && app.ui.gemini_api_key.trim().is_empty() {
        let banner_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.warning))
            .corner_radius(CornerRadius::same(t.radius as u8))
            .inner_margin(Margin::same(8));

        banner_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(16.0, 16.0)), "triangle-alert", 14.0, t.warning);
                ui.add_space(18.0);
                ui.label(RichText::new(tl!("API key is not configured")).strong().size(11.0).color(t.warning));
            });
            ui.add_space(3.0);
            ui.label(
                RichText::new(tl!("Configure your Google Gemini API key to enable instant AI assistance for PhotoCraft."))
                    .size(10.5)
                    .color(t.text_dim),
            );
            ui.add_space(4.0);
            if widgets::primary_button(ui, tl!("Configure in Preferences…"), 0.0).clicked() {
                actions.open_prefs = true;
            }
        });
        ui.add_space(4.0);
    } else if provider.is_cli()
        && let Some(view) = cli_view
    {
        render_cli_status_strip(ui, t, provider, view.cli, actions);
        ui.add_space(4.0);
        render_mcp_control_strip(app, ui, t, view.photocraft_cli, view.control, actions);
        ui.add_space(4.0);
    }

    // Connection status banner
    if let Some(status) = &app.ui.copilot.connection_status {
        let status_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.accent_border))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 6));

        status_frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                icons::paint(ui, icon_rect, "check", 12.0, t.accent);
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width().max(60.0));
                    ui.add(egui::Label::new(RichText::new(status).size(10.5).color(t.accent_text)).wrap());
                });
            });
        });
        ui.add_space(4.0);
    }

    // Error message banner
    if let Some(err) = &app.ui.copilot.error_message {
        let err_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.danger))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 6));

        err_frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                icons::paint(ui, icon_rect, "triangle-alert", 12.0, t.danger);
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width().max(60.0));
                    ui.add(egui::Label::new(RichText::new(err).size(10.5).color(t.danger)).wrap());
                });
            });
            render_error_guidance(ui, t, provider, err, app.ui.copilot.last_error.as_ref(), actions);
        });
        ui.add_space(4.0);
    }
}

/// Renders one vision preset menu item with a built-in SVG icon (no tofu emojis).
fn vision_preset_menu_row(ui: &mut egui::Ui, t: &Tokens, icon: &'static str, label: &str) -> bool {
    let row = egui::Frame::NONE
        .inner_margin(Margin::symmetric(6, 4))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                icons::paint(ui, icon_rect, icon, 12.0, t.accent);
                ui.label(RichText::new(label).size(10.5).color(t.text));
            });
        });
    let resp = row.response.interact(Sense::click()).accessible_name(label);
    if resp.hovered() {
        ui.painter().rect_stroke(resp.rect, CornerRadius::same(t.radius_sm as u8), Stroke::new(1.0, t.accent_border), egui::StrokeKind::Inside);
    }
    resp.clicked()
}

/// Renders the visual action toolbar, pending attachment chip, and prompt input bar.
fn render_composer(
    copilot: &mut crate::state::CopilotState,
    has_doc: bool,
    ui: &mut egui::Ui,
    t: &Tokens,
    actions: &mut PanelActions,
) {
    // Visual action toolbar & preset prompts
    ui.horizontal(|ui| {
        let has_att = copilot.pending_attachment.is_some();

        // Attach canvas screenshot button
        ui.add_enabled_ui(has_doc, |ui| {
            let btn_tooltip = if has_doc {
                tl!("Attach current canvas screenshot (Ctrl+Shift+C)")
            } else {
                tl!("No active document to capture")
            };
            if icons::button(ui, "image", 20.0, has_att, btn_tooltip).clicked() {
                actions.capture_screenshot = true;
            }
        });

        // Presets menu button (built-in SVG chevron instead of Unicode triangle)
        let presets_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 3));
        let presets_btn = presets_frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tl!("Vision Presets")).size(10.5).color(t.text_dim));
                    let (chev_rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                    icons::paint(ui, chev_rect, "chevron_down", 10.0, t.text_dim);
                });
            })
            .response
            .interact(Sense::click())
            .accessible_name(tl!("Vision Presets"));

        egui::Popup::menu(&presets_btn).show(|ui| {
            ui.set_min_width(210.0);
            ui.set_max_width(240.0);
            if vision_preset_menu_row(ui, t, "sun", tl!("Exposure & Histogram")) {
                actions.preset_to_send = Some(
                    tl!("Analyze the exposure, highlights/shadows clipping, and dynamic range of this canvas image, and suggest adjustments.").to_string(),
                );
                ui.close();
            }
            if vision_preset_menu_row(ui, t, "palette", tl!("Color Grading Style")) {
                actions.preset_to_send = Some(
                    tl!("Evaluate the color temperature, cast, and tones in this canvas image, and suggest 2-3 suitable color grading styles with concrete PhotoCraft steps.")
                        .to_string(),
                );
                ui.close();
            }
            if vision_preset_menu_row(ui, t, "search", tl!("Retouching Defects")) {
                actions.preset_to_send = Some(
                    tl!("Diagnose any visual defects, unwanted noise, skin blemishes, or compositional issues in this image, and recommend retouching tools to fix them.")
                        .to_string(),
                );
                ui.close();
            }
            if vision_preset_menu_row(ui, t, "crop", tl!("Composition & Cropping")) {
                actions.preset_to_send = Some(
                    tl!("Analyze the composition, rule of thirds, subject balance, and suggest optimal cropping or framing.").to_string(),
                );
                ui.close();
            }
        });
    });

    // Pending attachment preview chip
    if let Some(ref att) = copilot.pending_attachment {
        let chip_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.accent_border))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 4));

        let mut remove_attachment = false;
        chip_frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "image", 11.0, t.accent);
                ui.add_space(14.0);
                let label = att.label.as_deref().unwrap_or("Screenshot");
                ui.label(RichText::new(format!("{label} ({}×{})", att.width, att.height)).size(10.0).color(t.text));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, "x", 16.0, false, tl!("Remove attachment")).clicked() {
                        remove_attachment = true;
                    }
                });
            });
        });
        if remove_attachment {
            copilot.pending_attachment = None;
        }
        ui.add_space(2.0);
    }

    // Bottom input bar
    let measuring = is_measuring_pass(ui);
    ui.horizontal(|ui| {
        let input_width = (ui.available_width() - 40.0).max(120.0);
        let composer_id = composer_input_id();
        // Plain Enter (no modifiers) submits without inserting a newline; Shift+Enter falls through
        // to `TextEdit::return_key` to insert a newline. Never consume input during a measuring pass.
        let enter_send = !measuring
            && ui.memory(|m| m.has_focus(composer_id))
            && ui.input_mut(|i| {
                let mut consumed = false;
                i.events.retain(|ev| {
                    if !consumed
                        && matches!(
                            ev,
                            egui::Event::Key {
                                key: egui::Key::Enter,
                                pressed: true,
                                modifiers,
                                ..
                            } if !modifiers.shift
                                && !modifiers.ctrl
                                && !modifiers.command
                                && !modifiers.alt
                        )
                    {
                        consumed = true;
                        false
                    } else {
                        true
                    }
                });
                consumed
            });

        if measuring {
            let mut preview_input = copilot.input.as_str();
            let _ = ui.add(
                egui::TextEdit::multiline(&mut preview_input)
                    .id(composer_id.with("measure"))
                    .interactive(false)
                    .desired_rows(2)
                    .hint_text(tl!("Ask AI Assistant… (Enter to send, Shift+Enter for new line)"))
                    .desired_width(input_width),
            );
        } else {
            let _text_edit_response = ui
                .add(
                    egui::TextEdit::multiline(&mut copilot.input)
                        .id(composer_id)
                        .desired_rows(2)
                        .lock_focus(true)
                        .return_key(Some(egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Enter)))
                        .hint_text(tl!("Ask AI Assistant… (Enter to send, Shift+Enter for new line)"))
                        .desired_width(input_width),
                )
                .accessible_name(tl!("Ask AI Assistant"));
        }

        if enter_send {
            actions.send_prompt = true;
        }

        // Send or Stop button
        let is_active_gen = copilot.is_loading || copilot.is_streaming;
        if is_active_gen {
            if icons::button(ui, "x_circle", 32.0, false, tl!("Stop Generating")).clicked() {
                actions.stop_generation = true;
            }
        } else {
            let can_send = !copilot.input.trim().is_empty() || copilot.pending_attachment.is_some();
            ui.add_enabled_ui(can_send, |ui| {
                if icons::button(ui, "sparkles", 32.0, false, tl!("Send prompt")).clicked() {
                    actions.send_prompt = true;
                }
            });
        }
    });
}

/// Applies CLI provider, binary detection, and MCP control actions collected from the panel.
fn apply_cli_and_mcp_actions(app: &mut PhotocraftApp, ctx: &egui::Context, actions: &PanelActions) {
    if let Some(provider) = actions.switch_provider
        && let Err(err) = set_provider(app, provider)
    {
        app.ui.copilot.error_message = Some(err);
        app.ui.copilot.last_error = None;
    }
    if actions.redetect_cli {
        let provider = active_provider(app);
        if provider.is_cli() {
            let cli_ov = cli_override_for_provider(app, provider).map(str::to_string);
            let pc_ov = photocraft_cli_override(app).map(str::to_string);
            let status = refresh_cli_detection_with_override(&mut app.ui.copilot, provider, cli_ov.as_deref());
            let _ = refresh_photocraft_cli_detection_with_override(&mut app.ui.copilot, pc_ov.as_deref());
            if matches!(status, CliDetectionStatus::Found(_))
                && matches!(app.ui.copilot.last_error, Some(AiBackendError::CliNotFound { .. }))
            {
                app.ui.copilot.clear_error();
            }
        }
    }
    if actions.enable_mcp_control {
        match enable_mcp_control(app, ctx) {
            Ok(()) => {
                app.ui.copilot.connection_status = Some(tl!("PhotoCraft MCP control channel enabled.").to_string());
            }
            Err(err) => {
                app.ui.copilot.connection_status = None;
                app.ui.copilot.error_message = Some(err);
                app.ui.copilot.last_error = None;
            }
        }
    }
    if let Some(mcp_on) = actions.set_mcp_enabled {
        app.ui.copilot.mcp_enabled = mcp_on;
        app.ui.copilot.clear_active_cli_session_id();
    }
    if let Some(auto_approve) = actions.set_auto_approve_mcp {
        app.ui.copilot.auto_approve_mcp_tools = auto_approve;
        app.ui.copilot.clear_active_cli_session_id();
    }
}

/// Applies deferred panel actions after UI construction completes.
fn apply_panel_actions(app: &mut PhotocraftApp, ctx: &egui::Context, actions: PanelActions) {
    if actions.stop_generation {
        stop_generating(app, ctx);
    }
    apply_cli_and_mcp_actions(app, ctx, &actions);
    if actions.toggle_docked {
        app.ui.copilot.toggle_docked();
        ctx.request_repaint();
    }
    if actions.new_chat {
        app.ui.copilot.create_new_session(None);
    }
    if let Some(id) = actions.switch_session_id {
        app.ui.copilot.switch_session(&id);
    }
    if let Some(id) = actions.delete_session_id {
        app.ui.copilot.delete_session(&id);
    }
    if let Some((msg_idx, step_idx)) = actions.execute_workflow_step {
        execute_workflow_step(app, ctx, msg_idx, step_idx);
    }
    if let Some((msg_idx, step_idx)) = actions.skip_workflow_step {
        skip_workflow_step(app, ctx, msg_idx, step_idx);
    }
    if let Some(msg_idx) = actions.execute_workflow_all {
        execute_entire_workflow(app, ctx, msg_idx);
    }

    if actions.capture_screenshot && app.session.active().is_some() {
        if app.ui.copilot.pending_attachment.is_some() {
            app.ui.copilot.pending_attachment = None;
        } else {
            match perform_canvas_inspection(app, "composite", crate::gemini::DEFAULT_VISION_MAX_EDGE) {
                Ok((att, _)) => app.ui.copilot.pending_attachment = Some(att),
                Err(err) => app.ui.copilot.error_message = Some(err),
            }
        }
    }

    if let Some(idx) = actions.open_expanded_image {
        app.ui.copilot.expanded_image_preview = Some(idx);
    }
    if let Some(idx) = actions.toggle_folded
        && let Some(msg) = app.ui.copilot.messages.get_mut(idx)
        && let Some(ref mut tc) = msg.tool_call
    {
        tc.folded = !tc.folded;
    }
    if let Some(idx) = actions.action_to_execute {
        execute_tool_action(app, ctx, idx);
    }
    if let Some(idx) = actions.action_to_cancel {
        cancel_tool_action(app, idx);
    }
    if actions.close {
        app.ui.copilot.open = false;
    }
    if actions.clear_chat {
        app.ui.copilot.clear();
    }
    if actions.do_test_connection {
        test_connection(app, ctx);
    }
    if actions.open_prefs {
        let _ = crate::menus::invoke(app, ctx, "edit.preferences.integrations", json!({}));
    }
    if let Some(preset) = actions.preset_to_send {
        attach_and_send_preset(app, ctx, preset);
    } else if let Some(suggestion) = actions.suggestion_to_send {
        app.ui.copilot.input = suggestion;
        send_chat(app, ctx);
    } else if actions.send_prompt {
        send_chat(app, ctx);
    }
}

/// Renders a welcoming placeholder with suggestion prompt chips.
fn render_empty_state(ui: &mut egui::Ui, t: &Tokens, suggestion_out: &mut Option<String>) {
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        let (icon_rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
        icons::paint(ui, icon_rect, "sparkles", 26.0, t.accent);
        ui.add_space(4.0);
        ui.label(RichText::new(tl!("How can I assist you?")).strong().size(13.5).color(t.text));
        ui.add_space(2.0);
        ui.add(
            egui::Label::new(
                RichText::new(tl!("Ask anything about image editing, visual diagnosis, color grading, or shortcuts."))
                    .size(10.5)
                    .color(t.text_dim),
            )
            .wrap(),
        );
        ui.add_space(10.0);

        let suggestions: [(Option<&'static str>, &str); 6] = [
            (Some("sun"), tl!("Analyze exposure & histogram of this image")),
            (Some("palette"), tl!("Suggest color grading styles and filter steps")),
            (Some("search"), tl!("Diagnose retouching defects & noise in this photo")),
            (None, tl!("How do I create and refine a layer mask?")),
            (None, tl!("Explain the difference between Multiply and Screen blend modes")),
            (None, tl!("What are the most essential keyboard shortcuts?")),
        ];

        for (icon, prompt) in suggestions {
            let chip_frame = egui::Frame::NONE
                .fill(t.field)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(t.radius_sm as u8))
                .inner_margin(Margin::symmetric(10, 5));

            let resp = chip_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    if let Some(icon_name) = icon {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(13.0, 13.0), Sense::hover());
                        icons::paint(ui, icon_rect, icon_name, 11.0, t.accent);
                    }
                    ui.vertical(|ui| {
                        ui.set_max_width(ui.available_width().max(60.0));
                        ui.add(egui::Label::new(RichText::new(prompt).size(10.0).color(t.text)).wrap());
                    });
                });
            });

            if resp.response.interact(Sense::click()).accessible_name(prompt).clicked() {
                *suggestion_out = Some(prompt.to_string());
            }
            ui.add_space(3.0);
        }
        ui.add_space(6.0);
    });
}

/// Dispatch handlers for interactive tool call action cards and workflow plans.
struct ToolCardDispatch<'a> {
    execute: &'a mut Option<usize>,
    cancel: &'a mut Option<usize>,
    toggle_folded: &'a mut Option<usize>,
    open_expanded: &'a mut Option<usize>,
    execute_workflow_step: &'a mut Option<(usize, usize)>,
    skip_workflow_step: &'a mut Option<(usize, usize)>,
    execute_workflow_all: &'a mut Option<usize>,
    is_streaming: bool,
    is_last_model_msg: bool,
}

/// Retrieves or decodes and caches an egui TextureHandle for an image attachment.
fn get_or_load_attachment_texture(
    ctx: &egui::Context,
    att: &ChatImageAttachment,
) -> Option<egui::TextureHandle> {
    let slice_len = att.base64_data.len().min(24);
    let key_id = egui::Id::unique(("copilot-att-tex", att.width, att.height, &att.base64_data[..slice_len]));

    let cached = ctx.data_mut(|d| d.get_temp::<egui::TextureHandle>(key_id));
    if let Some(tex) = cached {
        return Some(tex);
    }

    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(&att.base64_data).ok()?;
    let img = photocraft_codecs::decode(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let color_img = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], &rgba);
    let tex = ctx.load_texture(
        format!("copilot-img-{}", att.width),
        color_img,
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|d| d.insert_temp(key_id, tex.clone()));
    Some(tex)
}

/// Renders an attached screenshot card inside a chat message or tool card.
fn render_image_attachment(
    ui: &mut egui::Ui,
    att: &ChatImageAttachment,
    t: &Tokens,
    open_expanded: &mut Option<usize>,
    msg_idx: usize,
) {
    let Some(tex) = get_or_load_attachment_texture(ui.ctx(), att) else {
        ui.label(RichText::new(tl!("Failed to decode image preview")).size(10.0).color(t.danger));
        return;
    };

    let aspect = (att.width as f32) / (att.height as f32).max(1.0);
    let max_w = (ui.available_width() - 16.0).clamp(100.0, 260.0);
    let max_h = 160.0;
    let (disp_w, disp_h) = if aspect > (max_w / max_h) {
        (max_w, (max_w / aspect).max(1.0))
    } else {
        ((max_h * aspect).max(1.0), max_h)
    };

    let frame = egui::Frame::NONE
        .fill(t.dock)
        .stroke(Stroke::new(1.0, t.card_border))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::same(4));

    frame.show(ui, |ui| {
        let resp = ui.add(
            egui::Image::new((tex.id(), vec2(disp_w, disp_h)))
                .corner_radius(CornerRadius::same((t.radius_sm - 1.0).max(0.0) as u8))
                .sense(Sense::click()),
        );

        if resp.clicked() {
            *open_expanded = Some(msg_idx);
        }

        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(10.0, 10.0)), "image", 9.0, t.text_dim);
            ui.add_space(12.0);
            let label = att.label.as_deref().unwrap_or("Screenshot");
            let format_str = if att.mime_type.contains("png") { "PNG" } else { "JPEG" };
            ui.label(RichText::new(format!("{label} ({}×{} px · {format_str})", att.width, att.height)).size(9.5).color(t.text_dim));
        });
    });
}

/// Renders a modal/floating expanded preview of an image attachment.
fn render_expanded_image_modal(app: &mut PhotocraftApp, ctx: &egui::Context, t: &Tokens, idx: usize) {
    let Some(msg) = app.ui.copilot.messages.get(idx) else {
        app.ui.copilot.expanded_image_preview = None;
        return;
    };
    let Some(att) = &msg.image else {
        app.ui.copilot.expanded_image_preview = None;
        return;
    };

    let mut close = false;
    let title = att.label.as_deref().unwrap_or_else(|| tl!("Canvas Screenshot Preview"));

    egui::Window::new(title)
        .id(egui::Id::unique("copilot-expanded-image-modal"))
        .title_bar(true)
        .resizable(true)
        .collapsible(false)
        .default_size(vec2(520.0, 440.0))
        .min_width(320.0)
        .min_height(260.0)
        .open(&mut true)
        .show(ctx, |ui| {
            if let Some(tex) = get_or_load_attachment_texture(ctx, att) {
                let avail_w = ui.available_width().max(100.0);
                let avail_h = (ui.available_height() - 36.0).max(100.0);
                let aspect = (att.width as f32) / (att.height as f32).max(1.0);
                let (w, h) = if aspect > (avail_w / avail_h) {
                    (avail_w, (avail_w / aspect).max(1.0))
                } else {
                    ((avail_h * aspect).max(1.0), avail_h)
                };

                ui.vertical_centered(|ui| {
                    ui.add(egui::Image::new((tex.id(), vec2(w, h))).corner_radius(CornerRadius::same(t.radius_sm as u8)));
                });
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{}: {}×{} px", tl!("Dimensions"), att.width, att.height)).size(10.5).color(t.text_dim));
                ui.label(RichText::new(format!("{}: {}", tl!("Format"), att.mime_type)).size(10.5).color(t.text_dim));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::secondary_button(ui, tl!("Close"), 0.0).clicked() {
                        close = true;
                    }
                });
            });
        });

    if close {
        app.ui.copilot.expanded_image_preview = None;
    }
}

/// Renders an interactive multi-step workflow macro checklist card.
fn render_workflow_plan_card(
    ui: &mut egui::Ui,
    msg_idx: usize,
    msg: &ChatMessage,
    plan: &crate::state::WorkflowPlan,
    t: &Tokens,
    dispatch: &mut ToolCardDispatch<'_>,
) {
    let border_color = if plan.auto_running {
        t.accent_border
    } else if plan.is_all_finished() {
        t.card_border
    } else {
        t.field_border
    };

    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(10, 8));

    frame.show(ui, |ui| {
        // Header
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "sparkles", 12.0, t.accent);
            ui.add_space(16.0);
            ui.label(RichText::new(&plan.title).strong().size(11.5).color(t.text));

            // Status pill on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (status_text, status_color) = if plan.auto_running {
                    (tl!("Running…").to_string(), t.accent_text)
                } else if plan.is_all_finished() {
                    (tl!("Completed").to_string(), t.accent_text)
                } else {
                    (format!("{}/{} {}", plan.completed_count(), plan.steps.len(), tl!("Done")), t.text_dim)
                };

                let pill = egui::Frame::NONE
                    .fill(t.field)
                    .corner_radius(CornerRadius::same(t.radius_sm as u8))
                    .inner_margin(Margin::symmetric(5, 2));
                pill.show(ui, |ui| {
                    ui.label(RichText::new(status_text).size(9.5).color(status_color));
                });
            });
        });

        ui.add_space(4.0);

        // Optional explanation text
        if !msg.content.is_empty() && msg.content != plan.title {
            render_markdown(ui, &msg.content, t);
            ui.add_space(4.0);
        }

        // Progress bar
        let total = plan.steps.len().max(1) as f32;
        let completed = plan.completed_count() as f32;
        let progress_ratio = (completed / total).clamp(0.0, 1.0);
        let bar_height = 4.0;
        let (bar_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), bar_height), Sense::hover());
        ui.painter().rect_filled(bar_rect, CornerRadius::same(2), t.field);
        if progress_ratio > 0.0 {
            let fill_w = bar_rect.width() * progress_ratio;
            let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_height));
            ui.painter().rect_filled(fill_rect, CornerRadius::same(2), t.accent);
        }

        ui.add_space(6.0);

        // Steps list
        let list_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 6));

        list_frame.show(ui, |ui| {
            for (step_idx, step) in plan.steps.iter().enumerate() {
                ui.horizontal(|ui| {
                    let (icon_name, icon_color) = match step.status {
                        crate::state::WorkflowStepStatus::Pending => ("circle", t.text_dim),
                        crate::state::WorkflowStepStatus::Executing => ("rotate-cw", t.accent),
                        crate::state::WorkflowStepStatus::Completed => ("check", t.accent),
                        crate::state::WorkflowStepStatus::Skipped => ("minus", t.text_dim),
                        crate::state::WorkflowStepStatus::Failed => ("triangle-alert", t.danger),
                    };
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), icon_name, 10.0, icon_color);
                    ui.add_space(14.0);

                    let step_title_color = match step.status {
                        crate::state::WorkflowStepStatus::Completed | crate::state::WorkflowStepStatus::Skipped => t.text_dim,
                        crate::state::WorkflowStepStatus::Failed => t.danger,
                        _ => t.text,
                    };
                    let title_text = format!("{}. {}", step_idx + 1, step.description);
                    ui.label(RichText::new(title_text).size(10.5).color(step_title_color));

                    // Step action buttons if pending
                    if step.status == crate::state::WorkflowStepStatus::Pending && !plan.auto_running {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(RichText::new(tl!("Skip")).size(9.0).color(t.text_dim)).clicked() {
                                *dispatch.skip_workflow_step = Some((msg_idx, step_idx));
                            }
                            if ui.small_button(RichText::new(tl!("Run")).size(9.0).color(t.accent_text)).clicked() {
                                *dispatch.execute_workflow_step = Some((msg_idx, step_idx));
                            }
                        });
                    }
                });

                if let Some(ref cmd) = step.command_id {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(RichText::new(format!("{}: {cmd}", tl!("Command"))).size(9.0).color(t.text_dim));
                    });
                }

                if let Some(ref err) = step.error {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(RichText::new(format!("{}: {err}", tl!("Error"))).size(9.5).color(t.danger));
                    });
                }

                if step_idx + 1 < plan.steps.len() {
                    ui.add_space(2.0);
                }
            }
        });

        // Bottom action buttons
        if !plan.is_all_finished() {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if plan.auto_running {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "rotate-cw", 11.0, t.accent);
                    ui.add_space(14.0);
                    ui.label(RichText::new(tl!("Executing workflow steps automatically…")).size(10.5).color(t.accent_text));
                } else {
                    if widgets::primary_button(ui, tl!("Execute All Steps"), 0.0).clicked() {
                        *dispatch.execute_workflow_all = Some(msg_idx);
                    }
                }
            });
        }
    });
}

/// Renders a single user, model, or tool chat message card.
fn render_chat_message(
    ui: &mut egui::Ui,
    msg_idx: usize,
    msg: &ChatMessage,
    t: &Tokens,
    dispatch: &mut ToolCardDispatch<'_>,
) {
    if msg.role == ChatRole::Function {
        render_function_response_chip(ui, msg_idx, msg, t, dispatch.toggle_folded);
        return;
    }

    if let Some(plan) = &msg.workflow_plan {
        render_workflow_plan_card(ui, msg_idx, msg, plan, t, dispatch);
        return;
    }

    if let Some(tc) = &msg.tool_call {
        render_tool_call_card(
            ui,
            msg_idx,
            msg,
            tc,
            t,
            dispatch,
        );
        return;
    }

    let (bg, border, role_label, role_color) = match msg.role {
        ChatRole::User => (t.field, t.field_border, tl!("You"), t.accent_text),
        ChatRole::Model => (t.card, t.card_border, tl!("AI Assistant"), t.accent),
        ChatRole::System | ChatRole::Function => (t.dock, t.card_border, tl!("System"), t.text_dim),
    };

    let msg_frame = egui::Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(10, 8));

    msg_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        let streaming_this = msg.role == ChatRole::Model && dispatch.is_streaming && dispatch.is_last_model_msg;
        ui.horizontal(|ui| {
            if msg.role == ChatRole::Model {
                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "sparkles", 12.0, t.accent);
                ui.add_space(16.0);
            }
            ui.label(RichText::new(role_label).strong().size(11.0).color(role_color));
            if streaming_this {
                ui.label(RichText::new(tl!("Streaming…")).italics().size(9.5).color(t.text_dim));
            }

            // Copy button for Copilot responses
            if msg.role == ChatRole::Model {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, "copy", 16.0, false, tl!("Copy text")).clicked() {
                        ui.ctx().copy_text(msg.content.clone());
                    }
                });
            }
        });

        ui.add_space(4.0);

        if msg.role == ChatRole::Model {
            render_markdown(ui, &msg.content, t);
        } else {
            ui.add(egui::Label::new(RichText::new(&msg.content).size(11.5).color(t.text)).wrap());
        }

        if let Some(att) = &msg.image {
            ui.add_space(4.0);
            render_image_attachment(ui, att, t, dispatch.open_expanded, msg_idx);
        }
    });
}

/// Maximum Unicode scalar values rendered inside an expanded CLI/MCP tool payload box
/// so huge base64 previews never stall UI layout.
const MAX_TOOL_PREVIEW_CHARS: usize = 1_200;

/// Extracts a concise `(badge, display_name)` pair from a CLI / MCP tool identifier.
fn cli_tool_display_parts(raw: &str) -> (&'static str, &str) {
    if let Some(rest) = raw.strip_prefix("mcp__photocraft__") {
        ("MCP · PhotoCraft", rest)
    } else if let Some(rest) = raw.strip_prefix("mcp_photocraft_") {
        ("MCP · PhotoCraft", rest)
    } else if let Some(rest) = raw.strip_prefix("photocraft:") {
        ("MCP · PhotoCraft", rest)
    } else if matches!(
        raw,
        "doc_inspect"
            | "doc_new"
            | "doc_open"
            | "doc_save"
            | "doc_export"
            | "doc_close"
            | "doc_select"
            | "doc_place"
            | "doc_render_preview"
            | "render_preview"
            | "command_list"
            | "command_run"
            | "command_batch"
            | "control_call"
            | "session_list"
            | "ui_inspect"
            | "ui_set"
            | "ui_screenshot"
            | "ui_menu_invoke"
            | "ui_pointer"
            | "jobs_list"
            | "jobs_cancel"
            | "agent_folders"
    ) {
        ("MCP · PhotoCraft", raw)
    } else {
        ("CLI Tool", raw)
    }
}

/// Resolved presentation metadata for a CLI/MCP tool card, unwrapping generic MCP wrappers
/// such as Antigravity CLI's `call_mcp_tool` into `server / tool` and clean inner arguments.
#[derive(Clone, Debug, PartialEq)]
struct CliToolDisplay {
    badge: &'static str,
    title: String,
    effective_tool: String,
    display_args: Option<Value>,
}

fn is_generic_mcp_wrapper(raw_name: &str) -> bool {
    matches!(raw_name, "call_mcp_tool" | "mcp_tool_call" | "use_mcp_tool" | "tool")
        || raw_name.ends_with(":call_mcp_tool")
        || raw_name.ends_with("__call_mcp_tool")
}

fn non_empty_str_field<'a>(obj: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|k| obj.get(*k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()))
}

fn resolve_cli_tool_display(raw_name: &str, args: &Value) -> CliToolDisplay {
    if let Some(obj) = args.as_object() {
        let server = non_empty_str_field(obj, &["ServerName", "server_name", "server"]);
        let tool = non_empty_str_field(obj, &["ToolName", "tool_name", "tool"]);
        if let Some(tool_name) = tool
            && (is_generic_mcp_wrapper(raw_name) || server.is_some())
        {
            let badge = match server {
                Some(s) if s.eq_ignore_ascii_case("photocraft") => "MCP · PhotoCraft",
                Some(_) => "MCP",
                None => cli_tool_display_parts(tool_name).0,
            };
            let title = match server {
                Some(s) => format!("{s} / {tool_name}"),
                None => tool_name.to_string(),
            };
            let inner_args = obj
                .get("Arguments")
                .or_else(|| obj.get("arguments"))
                .or_else(|| obj.get("params"))
                .and_then(normalize_tool_arg_value);
            return CliToolDisplay {
                badge,
                title,
                effective_tool: tool_name.to_string(),
                display_args: inner_args,
            };
        }
    }

    let (badge, display_name) = cli_tool_display_parts(raw_name);
    let display_args = normalize_tool_arg_value(args);
    CliToolDisplay {
        badge,
        title: display_name.to_string(),
        effective_tool: display_name.to_string(),
        display_args,
    }
}

fn normalize_tool_arg_value(val: &Value) -> Option<Value> {
    match val {
        Value::Null => None,
        Value::Object(map) if map.is_empty() => None,
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "{}" {
                None
            } else if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']'))
            {
                serde_json::from_str::<Value>(trimmed).ok().or_else(|| Some(Value::String(trimmed.to_string())))
            } else {
                Some(Value::String(trimmed.to_string()))
            }
        }
        other => Some(other.clone()),
    }
}

/// Formats a tool parameter or output JSON payload for compact monospace preview,
/// unpacking standard MCP `{content: [{type: "text", text: "..."}]}` wrappers and pretty-printing JSON.
fn format_tool_payload_preview(val: &Value, max_chars: usize) -> String {
    let raw = if let Some(s) = val.as_str() {
        pretty_if_json_string(s)
    } else if let Some(content) = val.get("content").and_then(Value::as_array) {
        let mut texts = Vec::new();
        for item in content {
            if let Some(t) = item.get("text").and_then(Value::as_str) {
                texts.push(pretty_if_json_string(t));
            }
        }
        if !texts.is_empty() {
            texts.join("\n")
        } else {
            serde_json::to_string_pretty(val).unwrap_or_default()
        }
    } else {
        serde_json::to_string_pretty(val).unwrap_or_default()
    };

    if raw.chars().count() <= max_chars {
        raw
    } else {
        let mut out: String = raw.chars().take(max_chars.saturating_sub(14)).collect();
        out.push_str("… (truncated)");
        out
    }
}

fn pretty_if_json_string(raw: &str) -> String {
    let trimmed = raw.trim();
    if ((trimmed.starts_with('{') && trimmed.ends_with('}')) || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        && let Ok(parsed) = serde_json::from_str::<Value>(trimmed)
        && let Ok(pretty) = serde_json::to_string_pretty(&parsed)
    {
        return pretty;
    }
    raw.to_string()
}

/// Renders a live, collapsible tool status card emitted by a local CLI agent (typically MCP).
/// Never displays local Apply / Cancel confirmation buttons.
fn render_cli_tool_card(
    ui: &mut egui::Ui,
    msg_idx: usize,
    tc: &crate::state::ToolCallAction,
    t: &Tokens,
    dispatch: &mut ToolCardDispatch<'_>,
) {
    let border_color = match tc.status {
        ToolActionStatus::Executing => t.accent_border,
        ToolActionStatus::Completed => t.card_border,
        ToolActionStatus::Cancelled => t.field_border,
        ToolActionStatus::Failed => t.danger,
        ToolActionStatus::PendingConfirmation => t.field_border,
    };

    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(8, 6));

    let resolved = resolve_cli_tool_display(&tc.name, &tc.args);
    let icon_name = match resolved.effective_tool.as_str() {
        "doc_inspect" | "ui_inspect" | "session_list" => "eye",
        "doc_render_preview" | "render_preview" | "ui_screenshot" => "image",
        "command_run" | "command_batch" | "command_list" | "ui_set" => "sliders-horizontal",
        _ => "sparkles",
    };

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        let has_details = resolved.display_args.is_some() || tc.result.is_some();

        ui.horizontal(|ui| {
            let icon_color = match tc.status {
                ToolActionStatus::Failed => t.danger,
                ToolActionStatus::Executing => t.accent,
                _ => t.accent_text,
            };
            let (icon_rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
            icons::paint(ui, icon_rect, icon_name, 11.0, icon_color);

            let tag_frame = egui::Frame::NONE
                .fill(t.field)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(t.radius_sm as u8))
                .inner_margin(Margin::symmetric(4, 1));
            tag_frame.show(ui, |ui| {
                ui.label(RichText::new(tl!(resolved.badge)).size(8.5).color(t.text_dim));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if has_details {
                    let (chevron, tip) = if tc.folded {
                        ("chevron_down", tl!("Show details"))
                    } else {
                        ("chevron_up", tl!("Hide details"))
                    };
                    if icons::button(ui, chevron, 22.0, !tc.folded, tip).clicked() {
                        *dispatch.toggle_folded = Some(msg_idx);
                    }
                }

                let (status_text, status_color) = match tc.status {
                    ToolActionStatus::Executing => (tl!("Running…"), t.accent_text),
                    ToolActionStatus::Completed => (tl!("Done"), t.accent_text),
                    ToolActionStatus::Cancelled => (tl!("Cancelled"), t.text_dim),
                    ToolActionStatus::Failed => (tl!("Failed"), t.danger),
                    ToolActionStatus::PendingConfirmation => (tl!("Running…"), t.text_dim),
                };
                let pill = egui::Frame::NONE
                    .fill(t.field)
                    .corner_radius(CornerRadius::same(t.radius_sm as u8))
                    .inner_margin(Margin::symmetric(5, 1));
                pill.show(ui, |ui| {
                    ui.label(RichText::new(status_text).size(9.0).color(status_color));
                });
            });
        });

        // Tool title on its own wrapped line so long `server / tool` names never collide with badges or clip.
        ui.add_space(2.0);
        ui.add(egui::Label::new(RichText::new(&resolved.title).monospace().strong().size(10.0).color(t.text)).wrap());

        if !tc.folded && has_details {
            ui.add_space(4.0);
            let detail_frame = egui::Frame::NONE
                .fill(t.field)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(t.radius_sm as u8))
                .inner_margin(Margin::symmetric(6, 5));
            detail_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(ref args_val) = resolved.display_args {
                    ui.label(RichText::new(tl!("Parameters")).strong().size(9.0).color(t.text_dim));
                    let formatted_args = format_tool_payload_preview(args_val, MAX_TOOL_PREVIEW_CHARS);
                    ui.add(egui::Label::new(RichText::new(formatted_args).monospace().size(9.0).color(t.text)).wrap());
                }
                if let Some(ref res) = tc.result {
                    if resolved.display_args.is_some() {
                        ui.add_space(4.0);
                    }
                    if let Some(err) = res.get("error").and_then(Value::as_str) {
                        ui.label(RichText::new(tl!("Error")).strong().size(9.0).color(t.danger));
                        ui.add(egui::Label::new(RichText::new(err).monospace().size(9.0).color(t.danger)).wrap());
                    } else {
                        ui.label(RichText::new(tl!("Result")).strong().size(9.0).color(t.text_dim));
                        let formatted_res = format_tool_payload_preview(res, MAX_TOOL_PREVIEW_CHARS);
                        ui.add(egui::Label::new(RichText::new(formatted_res).monospace().size(9.0).color(t.text_dim)).wrap());
                    }
                }
            });
        }
    });
}

/// Renders a tool invocation card with status, parameters, details, and action buttons.
fn render_tool_call_card(
    ui: &mut egui::Ui,
    msg_idx: usize,
    msg: &ChatMessage,
    tc: &crate::state::ToolCallAction,
    t: &Tokens,
    dispatch: &mut ToolCardDispatch<'_>,
) {
    if tc.origin == ToolCallOrigin::CliAgent {
        render_cli_tool_card(ui, msg_idx, tc, t, dispatch);
        return;
    }

    let border_color = match tc.status {
        crate::state::ToolActionStatus::PendingConfirmation => t.accent_border,
        crate::state::ToolActionStatus::Executing => t.accent_border,
        crate::state::ToolActionStatus::Completed => t.card_border,
        crate::state::ToolActionStatus::Cancelled => t.field_border,
        crate::state::ToolActionStatus::Failed => t.danger,
    };

    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(10, 8));

    frame.show(ui, |ui| {
        // Header
        ui.horizontal(|ui| {
            let icon_name = match tc.name.as_str() {
                "get_document_info" => "eye",
                "adjust_layer_property" => "sliders-horizontal",
                "inspect_canvas_image" => "image",
                _ => "sparkles",
            };
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), icon_name, 12.0, t.accent);
            ui.add_space(16.0);

            let header_title = match tc.name.as_str() {
                "get_document_info" => tl!("Document Info"),
                "adjust_layer_property" => tl!("Adjust Layer Properties"),
                "inspect_canvas_image" => tl!("Inspect Canvas Image"),
                _ => tl!("Suggested Action"),
            };
            ui.label(RichText::new(header_title).strong().size(11.0).color(t.accent_text));

            // Status badge on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (status_text, status_color) = match tc.status {
                    crate::state::ToolActionStatus::PendingConfirmation => (tl!("Pending Confirmation"), t.warning),
                    crate::state::ToolActionStatus::Executing => (tl!("Executing…"), t.accent_text),
                    crate::state::ToolActionStatus::Completed => (tl!("Completed"), t.accent_text),
                    crate::state::ToolActionStatus::Cancelled => (tl!("Cancelled"), t.text_dim),
                    crate::state::ToolActionStatus::Failed => (tl!("Failed"), t.danger),
                };
                let pill = egui::Frame::NONE
                    .fill(t.field)
                    .corner_radius(CornerRadius::same(t.radius_sm as u8))
                    .inner_margin(Margin::symmetric(5, 2));
                pill.show(ui, |ui| {
                    ui.label(RichText::new(status_text).size(9.5).color(status_color));
                });
            });
        });

        ui.add_space(4.0);

        // Optional explanation text from model
        if !msg.content.is_empty() {
            render_markdown(ui, &msg.content, t);
            ui.add_space(4.0);
        }

        // Arguments preview box
        let arg_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 6));

        arg_frame.show(ui, |ui| {
            match tc.name.as_str() {
                "get_document_info" => {
                    ui.horizontal(|ui| {
                        icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "search", 10.0, t.text_dim);
                        ui.add_space(14.0);
                        ui.label(RichText::new(tl!("Inspecting document info…")).size(10.5).color(t.text_dim));
                    });
                }
                "execute_command" => {
                    let cmd_id = tc.args.get("command_id").and_then(Value::as_str).unwrap_or("");
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Command:").strong().size(10.5).color(t.text_dim));
                        ui.label(RichText::new(cmd_id).monospace().size(10.5).color(t.text));
                    });
                    if let Some(params) = tc.args.get("params").and_then(Value::as_object)
                        && !params.is_empty()
                    {
                        ui.add_space(2.0);
                        let p_str = serde_json::to_string(params).unwrap_or_default();
                        ui.label(RichText::new(format!("Params: {p_str}")).monospace().size(10.0).color(t.text_dim));
                    }
                }
                "adjust_layer_property" => {
                    if let Some(opacity) = tc.args.get("opacity").and_then(Value::as_f64) {
                        let pct = if opacity <= 1.0 { (opacity * 100.0).round() as i64 } else { opacity.round() as i64 };
                        ui.label(RichText::new(format!("{}: {}%", tl!("Opacity"), pct)).size(10.5).color(t.text));
                    }
                    if let Some(blend) = tc.args.get("blend_mode").and_then(Value::as_str) {
                        ui.label(RichText::new(format!("{}: {}", tl!("Blend Mode"), blend)).size(10.5).color(t.text));
                    }
                    if let Some(visible) = tc.args.get("visible").and_then(Value::as_bool) {
                        let vis_str = if visible { "Visible" } else { "Hidden" };
                        ui.label(RichText::new(format!("{}: {}", tl!("Visibility"), vis_str)).size(10.5).color(t.text));
                    }
                }
                "inspect_canvas_image" => {
                    let target = tc.args.get("target").and_then(Value::as_str).unwrap_or("composite");
                    let target_name = if target == "active_layer" { tl!("Active Layer") } else { tl!("Canvas Composite") };
                    ui.horizontal(|ui| {
                        icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "image", 10.0, t.text_dim);
                        ui.add_space(14.0);
                        ui.label(RichText::new(format!("{}: {}", tl!("Target"), target_name)).size(10.5).color(t.text));
                    });
                }
                _ => {
                    let a_str = serde_json::to_string(&tc.args).unwrap_or_default();
                    ui.label(RichText::new(a_str).monospace().size(10.0).color(t.text));
                }
            }
        });

        if let Some(att) = &msg.image {
            ui.add_space(4.0);
            render_image_attachment(ui, att, t, dispatch.open_expanded, msg_idx);
        }

        // Result details or folding
        if let Some(res) = &tc.result {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let (chevron, tip) = if tc.folded {
                    ("chevron_down", tl!("Show details"))
                } else {
                    ("chevron_up", tl!("Hide details"))
                };
                if icons::button(ui, chevron, 20.0, !tc.folded, tip).clicked() {
                    *dispatch.toggle_folded = Some(msg_idx);
                }
                ui.label(RichText::new(tl!("Details")).size(10.0).color(t.accent_text));
            });

            if !tc.folded {
                ui.add_space(2.0);
                let res_frame = egui::Frame::NONE
                    .fill(t.field)
                    .corner_radius(CornerRadius::same(t.radius_sm as u8))
                    .inner_margin(Margin::symmetric(6, 4));
                res_frame.show(ui, |ui| {
                    if let Some(err) = res.get("error").and_then(Value::as_str) {
                        ui.label(RichText::new(err).size(10.0).color(t.danger));
                    } else if tc.name == "get_document_info" {
                        let w = res.get("width").and_then(Value::as_u64).unwrap_or(0);
                        let h = res.get("height").and_then(Value::as_u64).unwrap_or(0);
                        let mode = res.get("color_mode").and_then(Value::as_str).unwrap_or("RGB");
                        let depth = res.get("bit_depth").and_then(Value::as_str).unwrap_or("8-bit");
                        let layers = res.get("total_layers").and_then(Value::as_u64).unwrap_or(0);
                        let active = res.get("active_layer").and_then(|a| a.get("name")).and_then(Value::as_str).unwrap_or("None");

                        ui.label(RichText::new(format!("{}: {}×{} px", tl!("Canvas"), w, h)).size(10.0).color(t.text));
                        ui.label(RichText::new(format!("{}: {}, {}", tl!("Color Mode"), mode, depth)).size(10.0).color(t.text));
                        ui.label(RichText::new(format!("{}: {} ({}: {})", tl!("Layers"), layers, tl!("Active Layer"), active)).size(10.0).color(t.text_dim));
                    } else if tc.name == "inspect_canvas_image" {
                        if let Some(metrics) = res.get("metrics") {
                            let exp = &metrics["exposure"];
                            let cb = &metrics["color_balance"];
                            let lum = exp.get("mean_luminance").and_then(Value::as_f64).unwrap_or(0.0);
                            let shadow = exp.get("shadow_clipping_pct").and_then(Value::as_f64).unwrap_or(0.0);
                            let high = exp.get("highlight_clipping_pct").and_then(Value::as_f64).unwrap_or(0.0);
                            let eval = exp.get("evaluation").and_then(Value::as_str).unwrap_or("");
                            let cast = cb.get("cast").and_then(Value::as_str).unwrap_or("Neutral");

                            ui.label(RichText::new(format!("{}: {:.0}% ({})", tl!("Luminance"), lum * 100.0, eval)).size(10.0).color(t.text));
                            ui.label(RichText::new(format!("{}: {:.1}% · {}: {:.1}%", tl!("Shadow Clipping"), shadow, tl!("Highlight Clipping"), high)).size(10.0).color(t.text));
                            ui.label(RichText::new(format!("{}: {}", tl!("Color Balance"), cast)).size(10.0).color(t.text_dim));
                        } else {
                            let formatted = serde_json::to_string_pretty(res).unwrap_or_default();
                            ui.label(RichText::new(formatted).monospace().size(9.5).color(t.text_dim));
                        }
                    } else {
                        let formatted = serde_json::to_string_pretty(res).unwrap_or_default();
                        ui.label(RichText::new(formatted).monospace().size(9.5).color(t.text_dim));
                    }
                });
            }
        }

        // Action confirmation buttons for PendingConfirmation (Gemini API local tool proposals only)
        if tc.origin == ToolCallOrigin::GeminiApi && tc.status == crate::state::ToolActionStatus::PendingConfirmation {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, tl!("Apply / Execute"), 0.0).clicked() {
                    *dispatch.execute = Some(msg_idx);
                }
                if widgets::secondary_button(ui, tl!("Dismiss / Cancel"), 0.0).clicked() {
                    *dispatch.cancel = Some(msg_idx);
                }
            });
        }
    });
}

/// Renders a subtle system response chip for function results.
fn render_function_response_chip(
    ui: &mut egui::Ui,
    msg_idx: usize,
    msg: &ChatMessage,
    t: &Tokens,
    toggle_folded: &mut Option<usize>,
) {
    let frame = egui::Frame::NONE
        .fill(t.field)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::symmetric(8, 4));

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "check", 10.0, t.accent);
            ui.add_space(14.0);
            ui.label(RichText::new(&msg.content).size(10.0).color(t.text_dim));

            if let Some(tc) = &msg.tool_call
                && tc.result.is_some()
            {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (chevron, tip) = if tc.folded {
                        ("chevron_down", tl!("Show details"))
                    } else {
                        ("chevron_up", tl!("Hide details"))
                    };
                    if icons::button(ui, chevron, 20.0, !tc.folded, tip).clicked() {
                        *toggle_folded = Some(msg_idx);
                    }
                });
            }
        });

        if let Some(tc) = &msg.tool_call
            && !tc.folded
            && let Some(res) = &tc.result
        {
            ui.add_space(2.0);
            let s = serde_json::to_string_pretty(res).unwrap_or_default();
            ui.add(egui::Label::new(RichText::new(s).monospace().size(9.5).color(t.text_dim)).wrap());
        }
    });
}

/// Renders an in-flight thinking bubble.
fn render_loading_bubble(ui: &mut egui::Ui, t: &Tokens, provider: AiProviderKind) {
    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(Stroke::new(1.0, t.card_border))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(10, 8));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "sparkles", 12.0, t.accent);
            ui.add_space(16.0);
            let thinking = crate::i18n::fmt(
                tl!("{provider} is thinking…"),
                &[("provider", provider.display_name())],
            );
            ui.label(RichText::new(thinking).italics().size(11.0).color(t.text_dim));
        });
    });
}

/// Renders a fenced code block that fits inside the sidebar without clipping horizontally.
fn render_code_block(ui: &mut egui::Ui, code: &str, t: &Tokens) {
    let block_frame = egui::Frame::NONE
        .fill(t.field)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius_sm as u8))
        .inner_margin(Margin::symmetric(8, 6));

    block_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::ScrollArea::horizontal().auto_shrink([false, true]).show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(code).monospace().size(10.5).color(t.text)).wrap());
        });
    });
}

/// Lightweight, panic-safe Markdown parser and renderer.
///
/// Supports headings (#, ##, ###), code blocks (```), bullet points (- / *),
/// numbered lists, and inline formatting (**bold**, `code`).
pub fn render_markdown(ui: &mut egui::Ui, markdown: &str, t: &Tokens) {
    let mut in_code_block = false;
    let mut code_accumulator = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code_block {
                render_code_block(ui, &code_accumulator, t);
                code_accumulator.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            if !code_accumulator.is_empty() {
                code_accumulator.push('\n');
            }
            code_accumulator.push_str(line);
            continue;
        }

        if let Some(h1) = trimmed.strip_prefix("# ") {
            ui.add_space(4.0);
            ui.add(egui::Label::new(RichText::new(h1).strong().size(13.5).color(t.text)).wrap());
            ui.add_space(2.0);
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(3.0);
            ui.add(egui::Label::new(RichText::new(h2).strong().size(12.5).color(t.text)).wrap());
            ui.add_space(1.0);
        } else if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(2.0);
            ui.add(egui::Label::new(RichText::new(h3).strong().size(11.5).color(t.text)).wrap());
        } else if let Some(bullet) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            ui.horizontal_top(|ui| {
                ui.label(RichText::new("•").strong().color(t.accent));
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width().max(40.0));
                    render_inline_markdown(ui, bullet, t);
                });
            });
        } else if let Some((num, rest)) = trimmed.split_once(". ")
            && num.chars().all(|c| c.is_ascii_digit())
        {
            ui.horizontal_top(|ui| {
                ui.label(RichText::new(format!("{num}.")).strong().color(t.accent));
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width().max(40.0));
                    render_inline_markdown(ui, rest, t);
                });
            });
        } else if trimmed.is_empty() {
            ui.add_space(3.0);
        } else {
            render_inline_markdown(ui, line, t);
        }
    }

    // Flush any open code block if input ended without closing backticks
    if in_code_block && !code_accumulator.is_empty() {
        render_code_block(ui, &code_accumulator, t);
    }
}

/// Renders a single line with inline formatting (**bold**, `code`).
fn render_inline_markdown(ui: &mut egui::Ui, text: &str, t: &Tokens) {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.break_anywhere = false;
    job.wrap.max_width = ui.available_width().max(40.0);

    let mut remaining = text;
    while !remaining.is_empty() {
        if let Some(start) = remaining.find('`') {
            if start > 0 {
                append_formatted_spans(&mut job, &remaining[..start], t);
            }
            let after = &remaining[start + 1..];
            if let Some(end) = after.find('`') {
                let code_text = &after[..end];
                job.append(
                    code_text,
                    0.0,
                    egui::TextFormat {
                        font_id: theme::mono(11.0),
                        color: t.accent_text,
                        background: t.field,
                        ..Default::default()
                    },
                );
                remaining = &after[end + 1..];
            } else {
                append_formatted_spans(&mut job, &remaining[start..], t);
                break;
            }
        } else {
            append_formatted_spans(&mut job, remaining, t);
            break;
        }
    }

    ui.add(egui::Label::new(job).wrap());
}

/// Parses bold spans (**...**) within non-code text and appends to LayoutJob.
fn append_formatted_spans(job: &mut egui::text::LayoutJob, text: &str, t: &Tokens) {
    let mut remaining = text;
    while !remaining.is_empty() {
        if let Some(start) = remaining.find("**") {
            if start > 0 {
                job.append(
                    &remaining[..start],
                    0.0,
                    egui::TextFormat {
                        font_id: theme::medium(11.5),
                        color: t.text,
                        ..Default::default()
                    },
                );
            }
            let after = &remaining[start + 2..];
            if let Some(end) = after.find("**") {
                let bold_text = &after[..end];
                job.append(
                    bold_text,
                    0.0,
                    egui::TextFormat {
                        font_id: theme::semibold(11.5),
                        color: t.text,
                        ..Default::default()
                    },
                );
                remaining = &after[end + 2..];
            } else {
                job.append(
                    &remaining[start..],
                    0.0,
                    egui::TextFormat {
                        font_id: theme::medium(11.5),
                        color: t.text,
                        ..Default::default()
                    },
                );
                break;
            }
        } else {
            job.append(
                remaining,
                0.0,
                egui::TextFormat {
                    font_id: theme::medium(11.5),
                    color: t.text,
                    ..Default::default()
                },
            );
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ChatMessage, ChatRole, ToolActionStatus, ToolCallAction};

    #[test]
    fn handles_copilot_panel_ids() {
        assert!(handles("window.panel.geminiCopilot"));
        assert!(handles("window.toggle.geminiCopilot"));
        assert!(handles("view.geminiCopilot"));
        assert!(!handles("window.panel.layers"));
    }

    #[test]
    fn chat_message_constructors() {
        let u = ChatMessage::user("Hello");
        assert_eq!(u.role, ChatRole::User);
        assert_eq!(u.content, "Hello");

        let m = ChatMessage::model("World");
        assert_eq!(m.role, ChatRole::Model);
        assert_eq!(m.content, "World");

        let s = ChatMessage::system("Init");
        assert_eq!(s.role, ChatRole::System);
        assert_eq!(s.content, "Init");
    }

    #[test]
    fn copilot_state_helpers() {
        let mut state = crate::state::CopilotState::default();
        assert!(!state.open);
        assert!(state.messages.is_empty());

        state.add_user_message("Test question");
        state.add_model_message("Test reply");
        assert_eq!(state.messages.len(), 2);

        state.error_message = Some("Sample error".into());
        state.connection_status = Some("OK".into());
        state.clear();
        assert!(state.messages.is_empty());
        assert!(state.error_message.is_none());
        assert!(state.connection_status.is_none());
    }

    #[test]
    fn query_document_info_empty_session() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let info = query_document_info(&app);
        assert_eq!(info.get("has_document").and_then(Value::as_bool), Some(false));
    }

    #[test]
    fn query_document_info_with_active_document() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({ "width": 100, "height": 80 })).expect("new document");
        let info = query_document_info(&app);
        assert_eq!(info.get("has_document").and_then(Value::as_bool), Some(true));
        assert_eq!(info.get("width").and_then(Value::as_u64), Some(100));
        assert_eq!(info.get("height").and_then(Value::as_u64), Some(80));
        assert_eq!(info.get("color_mode").and_then(Value::as_str), Some("RGB"));
        assert!(info.get("total_layers").and_then(Value::as_u64).unwrap_or(0) >= 1);
    }

    #[test]
    fn adjust_layer_property_validation() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        // No doc opened -> fails gracefully with Err, no panic
        let err = execute_adjust_layer_property(&mut app, &json!({ "opacity": 0.5 })).unwrap_err();
        assert!(!err.is_empty());

        // Invalid opacity range
        let err_range = execute_adjust_layer_property(&mut app, &json!({ "opacity": -5.0 })).unwrap_err();
        assert!(err_range.contains("Opacity must be between"));

        // Empty args
        let err_empty = execute_adjust_layer_property(&mut app, &json!({})).unwrap_err();
        assert!(err_empty.contains("No layer properties specified"));
    }

    #[test]
    fn tool_call_action_status_and_message_construction() {
        let action = ToolCallAction {
            name: "execute_command".into(),
            args: json!({ "command_id": "image.adjustments.invert" }),
            status: ToolActionStatus::PendingConfirmation,
            result: None,
            folded: false,
            origin: ToolCallOrigin::GeminiApi,
            call_id: None,
        };
        assert_eq!(action.status.label(), "Pending Confirmation");

        let msg = ChatMessage::tool_call("Apply Invert", action);
        assert_eq!(msg.role, ChatRole::Model);
        assert!(msg.tool_call.is_some());

        let resp_msg = ChatMessage::tool_response("execute_command", json!({ "success": true }));
        assert_eq!(resp_msg.role, ChatRole::Function);
        assert!(resp_msg.tool_call.is_some());
        assert_eq!(resp_msg.tool_call.unwrap().status, ToolActionStatus::Completed);
    }

    #[test]
    fn perform_canvas_inspection_empty_session() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let res = perform_canvas_inspection(&app, "composite", 512);
        assert!(res.is_err());
    }

    #[test]
    fn perform_canvas_inspection_with_document() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({ "width": 200, "height": 150 })).expect("new document");
        let (att, metrics) = perform_canvas_inspection(&app, "composite", 512).expect("inspect canvas");
        assert_eq!(att.width, 200);
        assert_eq!(att.height, 150);
        assert_eq!(att.mime_type, "image/jpeg");
        assert!(!att.base64_data.is_empty());
        assert_eq!(metrics["dimensions"]["width"], 200);
        assert_eq!(metrics["dimensions"]["height"], 150);
    }

    #[test]
    fn chat_message_with_image_attachment() {
        let att = ChatImageAttachment {
            mime_type: "image/jpeg".to_string(),
            base64_data: "AQID".to_string(),
            width: 100,
            height: 100,
            label: Some("Canvas".to_string()),
        };
        let msg = ChatMessage::user_with_image("Look at this", att.clone());
        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, "Look at this");
        assert!(msg.image.is_some());
        assert_eq!(msg.image.unwrap().width, 100);
    }

    #[test]
    fn session_management_and_switching() {
        let mut state = crate::state::CopilotState::default();
        state.ensure_active_session();
        assert_eq!(state.sessions.len(), 1);
        let first_id = state.active_session_id.clone();

        state.add_user_message("First conversation question");
        state.add_model_message("First conversation response");
        assert_eq!(state.messages.len(), 2);

        // Create new session
        let second_id = state.create_new_session(Some("Project Work".into()));
        assert_eq!(state.sessions.len(), 2);
        assert_eq!(state.active_session_id, second_id);
        assert!(state.messages.is_empty());

        state.add_user_message("Second session question");
        assert_eq!(state.messages.len(), 1);

        // Switch back to first session
        state.switch_session(&first_id);
        assert_eq!(state.active_session_id, first_id);
        assert_eq!(state.messages.len(), 2);

        // Rename second session
        state.rename_session(&second_id, "Photo Editing");
        let second_session = state.sessions.iter().find(|s| s.id == second_id).expect("session exists");
        assert_eq!(second_session.title, "Photo Editing");

        // Delete second session
        state.delete_session(&second_id);
        assert_eq!(state.sessions.len(), 1);
        assert_eq!(state.active_session_id, first_id);
    }

    #[test]
    fn history_export_and_import() {
        let mut state = crate::state::CopilotState::default();
        state.ensure_active_session();
        state.add_user_message("How do I sharpen an image?");
        state.add_model_message("Use Filter > Sharpen > Unsharp Mask.");

        // Export markdown
        let md = state.export_markdown(None);
        assert!(md.contains("How do I sharpen an image?"));
        assert!(md.contains("Filter > Sharpen"));

        // Export JSON
        let json_str = state.export_all_json().expect("export JSON");
        assert!(json_str.contains("sharpen"));

        // Import into clean state
        let mut new_state = crate::state::CopilotState::default();
        new_state.import_sessions_json(&json_str).expect("import JSON");
        assert_eq!(new_state.sessions.len(), 1);
        assert_eq!(new_state.messages.len(), 2);
        assert_eq!(new_state.messages[0].content, "How do I sharpen an image?");
    }

    #[test]
    fn token_estimation_and_context_pruning() {
        let mut messages = Vec::new();
        for i in 0..20 {
            messages.push(ChatMessage::user(format!("User message {i}: detailed instructions for photo editing workflow.")));
            messages.push(ChatMessage::model(format!("Model response {i}: step by step adjustments for exposure, color, contrast.")));
        }

        let initial_tokens = crate::state::estimate_session_tokens(&messages);
        assert!(initial_tokens > 200);

        // Prune to a small budget (e.g. 100 tokens)
        let pruned = crate::state::prune_messages_for_context(&messages, 100);
        let pruned_tokens = crate::state::estimate_session_tokens(&pruned);
        assert!(pruned.len() < messages.len());
        assert!(pruned_tokens <= initial_tokens);

        // Ensure the latest message is retained in pruned list
        assert_eq!(pruned.last().unwrap().content, messages.last().unwrap().content);
    }

    #[test]
    fn workflow_step_execution_and_progression() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({ "width": 100, "height": 80 })).expect("new document");

        let step1 = crate::state::WorkflowStep {
            id: "step_1".into(),
            description: "Set layer opacity to 60%".into(),
            tool: "adjust_layer_property".into(),
            command_id: None,
            params: json!({ "opacity": 0.6 }),
            status: crate::state::WorkflowStepStatus::Pending,
            error: None,
        };

        let step2 = crate::state::WorkflowStep {
            id: "step_2".into(),
            description: "Invert colors".into(),
            tool: "execute_command".into(),
            command_id: Some("image.adjustments.invert".into()),
            params: json!({}),
            status: crate::state::WorkflowStepStatus::Pending,
            error: None,
        };

        let plan = crate::state::WorkflowPlan::new("Test Workflow", "Demonstration workflow", vec![step1, step2]);
        let msg = ChatMessage::workflow_plan("I've prepared a workflow plan for you:", plan);
        app.ui.copilot.messages.push(msg);

        let ctx = egui::Context::default();
        // Execute step 0
        execute_workflow_step(&mut app, &ctx, 0, 0);

        let plan_ref = app.ui.copilot.messages[0].workflow_plan.as_ref().expect("workflow plan");
        assert_eq!(plan_ref.steps[0].status, crate::state::WorkflowStepStatus::Completed);
        assert_eq!(plan_ref.steps[1].status, crate::state::WorkflowStepStatus::Pending);

        // Verify layer opacity actually changed
        let active_st = app.session.active().expect("active document");
        let active_layer_id = active_st.active_layer.expect("active layer");
        let layer = active_st.doc.layer(active_layer_id).expect("layer");
        assert!((layer.opacity - 0.6).abs() < 0.05);

        // Skip step 1
        skip_workflow_step(&mut app, &ctx, 0, 1);
        let plan_ref2 = app.ui.copilot.messages[0].workflow_plan.as_ref().expect("workflow plan");
        assert_eq!(plan_ref2.steps[1].status, crate::state::WorkflowStepStatus::Skipped);
        assert!(plan_ref2.is_all_finished());
    }

    #[test]
    fn workflow_step_error_handling() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        // No document open -> adjust_layer_property will return Err
        let step = crate::state::WorkflowStep {
            id: "step_1".into(),
            description: "Set opacity".into(),
            tool: "adjust_layer_property".into(),
            command_id: None,
            params: json!({ "opacity": 0.5 }),
            status: crate::state::WorkflowStepStatus::Pending,
            error: None,
        };
        let plan = crate::state::WorkflowPlan::new("Failing Workflow", "Workflow that fails", vec![step]);
        app.ui.copilot.messages.push(ChatMessage::workflow_plan("Plan", plan));

        let ctx = egui::Context::default();
        execute_workflow_step(&mut app, &ctx, 0, 0);

        let plan_ref = app.ui.copilot.messages[0].workflow_plan.as_ref().expect("workflow plan");
        assert_eq!(plan_ref.steps[0].status, crate::state::WorkflowStepStatus::Failed);
        assert!(plan_ref.steps[0].error.is_some());
    }

    #[test]
    fn stop_generating_resets_flags() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.ui.copilot.is_loading = true;
        app.ui.copilot.is_streaming = true;

        let ctx = egui::Context::default();
        stop_generating(&mut app, &ctx);

        assert!(!app.ui.copilot.is_loading);
        assert!(!app.ui.copilot.is_streaming);
    }

    #[test]
    fn copilot_state_defaults_to_docked_right_with_bounded_width() {
        let mut state = crate::state::CopilotState::default();
        assert!(state.is_docked());
        assert!(state.clamped_sidebar_width() >= MIN_COPILOT_SIDEBAR_WIDTH);
        assert!(state.clamped_sidebar_width() <= MAX_COPILOT_SIDEBAR_WIDTH);

        state.sidebar_width = 120.0;
        assert_eq!(state.clamped_sidebar_width(), MIN_COPILOT_SIDEBAR_WIDTH);
        state.sidebar_width = 1200.0;
        assert_eq!(state.clamped_sidebar_width(), MAX_COPILOT_SIDEBAR_WIDTH);

        state.toggle_docked();
        assert!(!state.is_docked());
        state.toggle_docked();
        assert!(state.is_docked());

        // Legacy serialized state missing docked / sidebarWidth / aiProvider must still load cleanly.
        let legacy: crate::state::CopilotState =
            serde_json::from_str(r#"{"open": true}"#).expect("legacy CopilotState deserializes");
        assert!(legacy.open);
        assert!(legacy.is_docked());
        assert_eq!(legacy.ai_provider, AiProviderKind::GeminiApi);
        assert!(legacy.clamped_sidebar_width() >= MIN_COPILOT_SIDEBAR_WIDTH);
    }

    #[test]
    fn menu_command_toggles_open_and_docked_and_accepts_legacy_ids() {
        assert!(handles("window.panel.geminiCopilot"));
        assert!(handles("window.toggle.geminiCopilot"));
        assert!(handles("view.geminiCopilot"));
        assert!(handles("window.panel.aiSidebar"));
        assert!(handles("window.toggle.aiSidebar"));

        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let res = menu(&mut app, "window.panel.geminiCopilot", &json!({}))
            .expect("handled")
            .expect("toggle open");
        assert_eq!(res.get("open").and_then(Value::as_bool), Some(true));
        assert_eq!(res.get("docked").and_then(Value::as_bool), Some(true));

        let res_float = menu(
            &mut app,
            "window.panel.geminiCopilot",
            &json!({ "show": true, "docked": false }),
        )
        .expect("handled")
        .expect("switch to floating");
        assert_eq!(res_float.get("open").and_then(Value::as_bool), Some(true));
        assert_eq!(res_float.get("docked").and_then(Value::as_bool), Some(false));
        assert!(!app.ui.copilot.is_docked());
    }

    #[test]
    fn set_provider_updates_engine_prefs_and_copilot_state() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        assert_eq!(active_provider(&app), AiProviderKind::GeminiApi);

        set_provider(&mut app, AiProviderKind::ClaudeCli).expect("set provider");
        assert_eq!(app.ui.copilot.ai_provider, AiProviderKind::ClaudeCli);
        assert_eq!(
            app.session.prefs().integrations.ai_provider,
            photocraft_engine::prefs::AiProvider::ClaudeCli
        );
        assert_eq!(active_provider(&app), AiProviderKind::ClaudeCli);
    }

    #[test]
    fn cli_session_id_persists_per_provider_and_clears_with_chat() {
        let mut state = crate::state::CopilotState::default();
        state.ensure_active_session();

        state.set_active_cli_session_id(AiProviderKind::ClaudeCli, "claude-sess-42".into());
        assert_eq!(
            state.active_cli_session_id(AiProviderKind::ClaudeCli).as_deref(),
            Some("claude-sess-42")
        );
        // Cross-provider lookup must not leak another CLI's session ID.
        assert_eq!(state.active_cli_session_id(AiProviderKind::AntigravityCli), None);
        assert_eq!(state.active_cli_session_id(AiProviderKind::GeminiCli), None);
        assert_eq!(state.active_cli_session_id(AiProviderKind::CodexCli), None);

        // Antigravity CLI records its conversation_id independently.
        state.set_active_cli_session_id(AiProviderKind::AntigravityCli, "conv-agy-101".into());
        assert_eq!(
            state.active_cli_session_id(AiProviderKind::AntigravityCli).as_deref(),
            Some("conv-agy-101")
        );
        assert_eq!(state.active_cli_session_id(AiProviderKind::ClaudeCli), None);

        // Serde round-trip preserves cliSessionId while remaining compatible with legacy sessions.
        let exported = state.export_all_json().expect("export sessions");
        assert!(exported.contains("cliSessionId"));
        assert!(exported.contains("conv-agy-101"));

        let legacy_json = r#"[{"id":"s1","title":"Old","createdAt":1,"updatedAt":1,"messages":[]}]"#;
        let mut loaded = crate::state::CopilotState::default();
        loaded.import_sessions_json(legacy_json).expect("legacy session imports");
        assert_eq!(loaded.active_cli_session_id(AiProviderKind::ClaudeCli), None);

        // Clearing chat resets the external CLI session ID and structured error.
        state.set_backend_error(crate::cli_agent::cli_not_found_error(AiProviderKind::ClaudeCli));
        state.clear();
        assert_eq!(state.active_cli_session_id(AiProviderKind::ClaudeCli), None);
        assert!(state.last_error.is_none());
        assert!(state.error_message.is_none());
    }

    #[test]
    fn cli_sidebar_uses_cached_detection_and_renders_guidance_without_api_key_banner() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, &crate::theme::AppTheme::default());
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.ui.copilot.open = true;
        app.ui.copilot.docked = true;
        app.ui.gemini_api_key.clear();
        set_provider(&mut app, AiProviderKind::ClaudeCli).expect("switch to Claude CLI");

        // Pre-populate cache so rendering does not scan PATH during the test.
        app.ui.copilot.cli_detection_cache.insert(
            AiProviderKind::ClaudeCli,
            CliDetectionStatus::Found(std::path::PathBuf::from(r"C:\npm\claude.cmd")),
        );
        app.ui.copilot.set_backend_error(AiBackendError::CliNotAuthenticated {
            provider: "Claude CLI",
            login_command: "claude",
            account_label: "Claude subscription",
            details: "Not logged in".into(),
        });

        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0))),
            ..Default::default()
        };

        let mut shown = false;
        let _ = crate::testing::run_ui(&ctx, input.clone(), |ui| {
            shown = show_docked_sidebar(&mut app, ui);
            egui::CentralPanel::no_frame().show(ui, |_ui| {});
        });
        assert!(shown);
        // Cached status remains intact without rescanning.
        assert_eq!(
            app.ui.copilot.cli_detection_cache.get(&AiProviderKind::ClaudeCli),
            Some(&CliDetectionStatus::Found(std::path::PathBuf::from(r"C:\npm\claude.cmd")))
        );

        // Also exercise NotFound strip and CliNotFound error card rendering.
        app.ui
            .copilot
            .cli_detection_cache
            .insert(AiProviderKind::ClaudeCli, CliDetectionStatus::NotFound);
        app.ui
            .copilot
            .set_backend_error(crate::cli_agent::cli_not_found_error(AiProviderKind::ClaudeCli));
        let _ = crate::testing::run_ui(&ctx, input, |ui| {
            show_docked_sidebar(&mut app, ui);
            egui::CentralPanel::no_frame().show(ui, |_ui| {});
        });
    }

    #[test]
    fn background_channel_handles_session_id_stream_chunks_and_cancellation() {
        let ctx = egui::Context::default();
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        set_provider(&mut app, AiProviderKind::ClaudeCli).expect("set Claude CLI");
        app.ui.copilot.ensure_active_session();
        app.ui.copilot.is_loading = true;
        app.ui.copilot.is_streaming = true;

        let (tx, rx) = std::sync::mpsc::channel();
        let handle = AiWorkerHandle {
            rx,
            stop_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_function_calls: Vec::new(),
        };
        let channel: CopilotChannel = Arc::new(Mutex::new(Some(handle)));
        ctx.data_mut(|d| d.insert_temp(copilot_channel_id(), channel));

        tx.send(AiBackendEvent::SessionId("claude-stream-99".into())).expect("send session id");
        tx.send(AiBackendEvent::StreamChunk {
            delta_text: "Partial ".into(),
            function_calls: Vec::new(),
        })
        .expect("send chunk 1");
        tx.send(AiBackendEvent::StreamChunk {
            delta_text: "reply".into(),
            function_calls: Vec::new(),
        })
        .expect("send chunk 2");
        tx.send(AiBackendEvent::StreamFinished(Err(AiBackendError::Cancelled)))
            .expect("send cancelled");

        check_background_response(&mut app, &ctx);

        assert!(!app.ui.copilot.is_loading);
        assert!(!app.ui.copilot.is_streaming);
        assert_eq!(
            app.ui.copilot.active_cli_session_id(AiProviderKind::ClaudeCli).as_deref(),
            Some("claude-stream-99")
        );
        assert_eq!(app.ui.copilot.messages.last().map(|m| m.content.as_str()), Some("Partial reply"));
        // Cancellation must not raise a red error banner.
        assert!(app.ui.copilot.error_message.is_none());
        assert!(app.ui.copilot.last_error.is_none());
    }

    #[test]
    fn gemini_api_empty_key_reports_missing_api_key_error() {
        let ctx = egui::Context::default();
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        set_provider(&mut app, AiProviderKind::GeminiApi).expect("set provider");
        app.ui.copilot.input = "Hello".into();

        send_chat(&mut app, &ctx);
        assert!(!app.ui.copilot.is_loading);
        assert!(app.ui.copilot.error_message.is_some());

        app.ui.copilot.error_message = None;
        test_connection(&mut app, &ctx);
        assert!(app.ui.copilot.connection_status.is_none());
        assert!(app.ui.copilot.error_message.is_some());
    }

    #[test]
    fn docked_sidebar_renders_inside_body_and_records_width() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, &crate::theme::AppTheme::default());
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.ui.copilot.open = true;
        app.ui.copilot.docked = true;

        let full_width = 1280.0;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(full_width, 800.0))),
            ..Default::default()
        };

        let mut shown = false;
        let mut remaining_width = full_width;
        let _ = crate::testing::run_ui(&ctx, input.clone(), |ui| {
            shown = show_body_with_docked_sidebar(&mut app, ui, |_, ui| {
                remaining_width = ui.available_width();
            });
        });

        assert!(shown);
        assert!(remaining_width < full_width);
        assert!(app.ui.copilot.sidebar_width >= MIN_COPILOT_SIDEBAR_WIDTH);
        assert!(app.ui.copilot.sidebar_width <= MAX_COPILOT_SIDEBAR_WIDTH);

        // Switching to floating must leave the body untouched.
        app.ui.copilot.toggle_docked();
        let mut shown_floating = true;
        let mut floating_remaining_width = 0.0;
        let _ = crate::testing::run_ui(&ctx, input, |ui| {
            shown_floating = show_body_with_docked_sidebar(&mut app, ui, |_, ui| {
                floating_remaining_width = ui.available_width();
            });
        });
        assert!(!shown_floating);
        assert!(floating_remaining_width > remaining_width);
    }

    #[test]
    fn cli_tool_updates_update_cards_in_place_by_call_id_and_step_index_without_duplicating() {
        let ctx = egui::Context::default();
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        set_provider(&mut app, AiProviderKind::AntigravityCli).expect("set Antigravity CLI");
        app.ui.copilot.add_user_message("Inspect my canvas and brighten it");
        app.ui.copilot.is_loading = true;
        app.ui.copilot.is_streaming = true;

        let (tx, rx) = std::sync::mpsc::channel();
        let handle = AiWorkerHandle {
            rx,
            stop_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_function_calls: Vec::new(),
        };
        let channel: CopilotChannel = Arc::new(Mutex::new(Some(handle)));
        ctx.data_mut(|d| d.insert_temp(copilot_channel_id(), channel));

        // 1. Step-index correlated tool call (Antigravity): Active -> Done updates the same card.
        tx.send(AiBackendEvent::ToolUpdate(CliToolUpdate {
            call_id: None,
            step_index: Some(1),
            tool_name: "doc_inspect".into(),
            state: CliToolState::Active,
            parameters: json!({"include_layers": true}),
            output: None,
            error: None,
            tool_info: Value::Null,
        }))
        .expect("send step 1 active");
        tx.send(AiBackendEvent::ToolUpdate(CliToolUpdate {
            call_id: None,
            step_index: Some(1),
            tool_name: "doc_inspect".into(),
            state: CliToolState::Done,
            parameters: json!({"include_layers": true}),
            output: Some(json!({"width": 1200, "height": 800})),
            error: None,
            tool_info: Value::Null,
        }))
        .expect("send step 1 done");

        // 2. Call-id correlated tool call (Claude/Gemini/Codex): Active -> Failed updates in place & unfolds.
        tx.send(AiBackendEvent::ToolUpdate(CliToolUpdate {
            call_id: Some("toolu_99".into()),
            step_index: None,
            tool_name: "mcp__photocraft__command_run".into(),
            state: CliToolState::Active,
            parameters: json!({"id": "image.adjustments.exposure", "params": {"exposure": 0.4}}),
            output: None,
            error: None,
            tool_info: Value::Null,
        }))
        .expect("send call active");
        tx.send(AiBackendEvent::ToolUpdate(CliToolUpdate {
            call_id: Some("toolu_99".into()),
            step_index: None,
            tool_name: "tool".into(),
            state: CliToolState::Failed,
            parameters: Value::Null,
            output: None,
            error: Some("No active document".into()),
            tool_info: Value::Null,
        }))
        .expect("send call failed");

        tx.send(AiBackendEvent::StreamChunk {
            delta_text: "I checked the canvas.".into(),
            function_calls: Vec::new(),
        })
        .expect("send text");
        tx.send(AiBackendEvent::StreamFinished(Ok(())))
            .expect("send finish");

        check_background_response(&mut app, &ctx);

        // User message + 2 distinct tool cards (updated in place, NOT 4!) + 1 model text message = 4 messages.
        assert_eq!(app.ui.copilot.messages.len(), 4);

        let card1 = app.ui.copilot.messages[1].tool_call.as_ref().expect("card 1");
        assert_eq!(card1.origin, ToolCallOrigin::CliAgent);
        assert_eq!(card1.call_id.as_deref(), Some("step:1"));
        assert_eq!(card1.name, "doc_inspect");
        assert_eq!(card1.status, ToolActionStatus::Completed);
        assert_eq!(card1.result, Some(json!({"width": 1200, "height": 800})));

        let card2 = app.ui.copilot.messages[2].tool_call.as_ref().expect("card 2");
        assert_eq!(card2.origin, ToolCallOrigin::CliAgent);
        assert_eq!(card2.call_id.as_deref(), Some("call:toolu_99"));
        // Preserves original real name instead of overwriting with generic "tool".
        assert_eq!(card2.name, "mcp__photocraft__command_run");
        assert_eq!(card2.status, ToolActionStatus::Failed);
        assert!(!card2.folded, "failed tool cards auto-expand so the error is visible");
        assert_eq!(card2.result, Some(json!({"error": "No active document"})));

        assert_eq!(app.ui.copilot.messages[3].content, "I checked the canvas.");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn enable_mcp_control_updates_prefs_flushes_disk_and_starts_agent_link() {
        use craft::automation::agents::{AgentLink, AgentSwitch};
        use craft::settings::testing::MemoryFiles;

        let files = Arc::new(MemoryFiles::new());
        let mut services = crate::testing::services_over(&files);
        let link_dir = tempfile::tempdir().expect("temp dir for AgentLink");
        services.agent_link = Some(AgentLink::new(Some(link_dir.path().to_path_buf()), "PhotoCraft test"));
        services.agent_switch = Some(AgentSwitch::fixed(false, false));

        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        // Disable both flags to verify enable_mcp_control turns both on.
        app.run(
            "prefs.set",
            json!({"values": {"integrations.connectAgents": false, "integrations.allowAgentControl": false}}),
        )
        .expect("disable agent prefs");
        assert!(matches!(
            inspect_mcp_control_state(&app),
            McpControlState::Disabled {
                connect_agents: false,
                allow_agent_control: false
            }
        ));

        let ctx = egui::Context::default();
        enable_mcp_control(&mut app, &ctx).expect("enable MCP control");

        assert!(app.session.prefs().integrations.connect_agents);
        assert!(app.session.prefs().integrations.agents.allow_agent_control);
        assert!(app.control.is_some());
        assert!(matches!(inspect_mcp_control_state(&app), McpControlState::Listening { port: Some(_), .. }));

        // Verify preferences were flushed immediately to disk (not left in debounce buffer).
        let saved_toml = files
            .text(std::path::Path::new("/memory/photocraft/config.toml"))
            .expect("config.toml flushed immediately");
        let parsed: serde_json::Value = toml::from_str(&saved_toml).expect("valid saved TOML");
        assert_eq!(parsed["integrations"]["connectAgents"], json!(true));
        assert_eq!(parsed["integrations"]["allowAgentControl"], json!(true));
    }

    #[test]
    fn cli_sidebar_renders_mcp_strip_and_live_tool_cards_cleanly() {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, &crate::theme::AppTheme::default());
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.ui.copilot.open = true;
        app.ui.copilot.docked = true;
        set_provider(&mut app, AiProviderKind::AntigravityCli).expect("switch to Antigravity CLI");
        app.ui.copilot.cli_detection_cache.insert(
            AiProviderKind::AntigravityCli,
            CliDetectionStatus::Found(std::path::PathBuf::from(r"C:\agy\agy.exe")),
        );
        app.ui.copilot.photocraft_cli_detection = Some(CliDetectionStatus::Found(
            std::path::PathBuf::from(r"F:\PhotoCraft\photocraft-cli.exe"),
        ));

        apply_cli_tool_update(
            &mut app.ui.copilot,
            CliToolUpdate {
                call_id: Some("c1".into()),
                step_index: None,
                tool_name: "mcp__photocraft__doc_inspect".into(),
                state: CliToolState::Active,
                parameters: json!({"layers": true}),
                output: None,
                error: None,
                tool_info: Value::Null,
            },
        );
        apply_cli_tool_update(
            &mut app.ui.copilot,
            CliToolUpdate {
                call_id: Some("c2".into()),
                step_index: None,
                tool_name: "command_run".into(),
                state: CliToolState::Failed,
                parameters: json!({"id": "bad.cmd"}),
                output: None,
                error: Some("Unknown command".into()),
                tool_info: Value::Null,
            },
        );

        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0))),
            ..Default::default()
        };

        let mut shown = false;
        let _ = crate::testing::run_ui(&ctx, input, |ui| {
            shown = show_docked_sidebar(&mut app, ui);
        });
        assert!(shown);

        // Verify display helpers.
        assert_eq!(cli_tool_display_parts("mcp__photocraft__doc_inspect"), ("MCP · PhotoCraft", "doc_inspect"));
        assert_eq!(cli_tool_display_parts("doc_render_preview"), ("MCP · PhotoCraft", "doc_render_preview"));
        assert_eq!(cli_tool_display_parts("read_file"), ("CLI Tool", "read_file"));

        let mcp_wrapped = json!({"content": [{"type": "text", "text": "hello world"}]});
        assert_eq!(format_tool_payload_preview(&mcp_wrapped, 100), "hello world");
        let long_str = Value::String("x".repeat(500));
        let truncated = format_tool_payload_preview(&long_str, 40);
        assert!(truncated.ends_with("… (truncated)"));
        assert!(truncated.chars().count() <= 40);
    }

    #[test]
    fn all_copilot_ui_icons_resolve_in_catalog_and_no_tofu_symbols_remain() {
        crate::icons::register();
        for &name in icons::ALL_COPILOT_ICON_NAMES {
            let resolved = icons::resolve_name(name);
            assert!(
                craft::ui::primitives::has_icon(&craft::behavior::IconId::new(resolved)),
                "copilot_ui icon `{name}` (resolved `{resolved}`) is missing from icon catalog"
            );
        }
        // Guard against reintroducing tofu glyphs in copilot_ui source.
        let src = include_str!("copilot_ui.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or(src);
        for forbidden in ['▾', '▴', '▸', '■', '▮', '📸', '🎨', '🔍', '📐'] {
            assert!(
                !prod.contains(forbidden),
                "copilot_ui production code must not use missing-glyph character `{forbidden}`"
            );
        }
    }

    #[test]
    fn cli_tool_card_unwraps_call_mcp_tool_server_tool_and_arguments() {
        // PascalCase Antigravity CLI `call_mcp_tool` wrapper
        let agy_args = json!({
            "ServerName": "photocraft",
            "ToolName": "doc_inspect",
            "Arguments": { "layers": true }
        });
        let resolved = resolve_cli_tool_display("call_mcp_tool", &agy_args);
        assert_eq!(resolved.badge, "MCP · PhotoCraft");
        assert_eq!(resolved.title, "photocraft / doc_inspect");
        assert_eq!(resolved.effective_tool, "doc_inspect");
        assert_eq!(resolved.display_args, Some(json!({ "layers": true })));

        // snake_case wrapper with JSON-encoded Arguments string
        let snake_args = json!({
            "server_name": "photocraft",
            "tool_name": "command_run",
            "arguments": "{\"id\":\"image.adjustments.invert\"}"
        });
        let resolved_snake = resolve_cli_tool_display("call_mcp_tool", &snake_args);
        assert_eq!(resolved_snake.badge, "MCP · PhotoCraft");
        assert_eq!(resolved_snake.title, "photocraft / command_run");
        assert_eq!(resolved_snake.effective_tool, "command_run");
        assert_eq!(resolved_snake.display_args, Some(json!({ "id": "image.adjustments.invert" })));

        // Third-party MCP server
        let other_mcp = json!({
            "ServerName": "github",
            "ToolName": "list_issues",
            "Arguments": {}
        });
        let resolved_other = resolve_cli_tool_display("call_mcp_tool", &other_mcp);
        assert_eq!(resolved_other.badge, "MCP");
        assert_eq!(resolved_other.title, "github / list_issues");
        assert_eq!(resolved_other.display_args, None);

        // Pretty-prints embedded JSON text in MCP output wrapper
        let mcp_json_output = json!({
            "content": [{"type": "text", "text": "{\"width\":1200,\"height\":800}"}]
        });
        let formatted = format_tool_payload_preview(&mcp_json_output, 500);
        assert!(formatted.contains("\"width\": 1200"));
        assert!(formatted.contains("\"height\": 800"));
    }

    #[test]
    fn docked_sidebar_splits_workspace_disjointly_and_never_overlaps_dock_panels() {
        // Pure geometry invariants across window widths
        for total_w in [640.0_f32, 960.0, 1280.0, 1440.0] {
            let body = Rect::from_min_size(pos2(0.0, 60.0), vec2(total_w, 800.0));
            for desired in [260.0_f32, 360.0, 480.0, 620.0] {
                let split = split_body_rect(body, desired);
                assert!(
                    split.workspace.right() <= split.sidebar.left(),
                    "workspace ({:?}) must sit strictly left of sidebar ({:?})",
                    split.workspace,
                    split.sidebar
                );
                assert!(split.sidebar.width() >= MIN_COPILOT_SIDEBAR_WIDTH.min(total_w));
                assert!(split.workspace.right() + 1.0 >= split.sidebar.left() - 0.01);
            }
        }

        // Full-app layout check: Color / Properties / Layers dock stays completely left of AI sidebar.
        let mut session = photocraft_engine::Session::new();
        session.execute("file.new", json!({ "width": 400, "height": 300 })).unwrap();
        let mut app = PhotocraftApp::new(session, Default::default());
        app.ui.copilot.open = true;
        app.ui.copilot.docked = true;
        app.ui.copilot.sidebar_width = 360.0;

        let mut h = crate::shortcut_dispatch::tests::app_harness(app);
        let layers_360 = crate::panel_docking::probe::group_rect(&h, "layers")
            .expect("Layers panel group should be visible in right dock");
        let color_360 = crate::panel_docking::probe::group_rect(&h, "color")
            .expect("Color panel group should be visible in right dock");
        let screen_w = 1440.0_f32;
        let expected_sidebar_left_360 = screen_w - 360.0;
        assert!(
            layers_360.right() <= expected_sidebar_left_360 + 0.5,
            "Layers panel right ({}) must not overlap AI sidebar left ({})",
            layers_360.right(),
            expected_sidebar_left_360
        );
        assert!(
            color_360.right() <= expected_sidebar_left_360 + 0.5,
            "Color panel right ({}) must not overlap AI sidebar left ({})",
            color_360.right(),
            expected_sidebar_left_360
        );

        // Widening the AI sidebar dynamically pushes the right dock further left without overlap.
        h.state_mut().ui.copilot.sidebar_width = 480.0;
        h.run_steps(4);
        let layers_480 = crate::panel_docking::probe::group_rect(&h, "layers")
            .expect("Layers panel group remains visible after widening sidebar");
        let expected_sidebar_left_480 = screen_w - 480.0;
        assert!(
            layers_480.right() <= expected_sidebar_left_480 + 0.5,
            "Layers panel right ({}) must shrink left of widened AI sidebar ({})",
            layers_480.right(),
            expected_sidebar_left_480
        );
        assert!(
            layers_480.right() < layers_360.right() - 80.0,
            "Layers panel should shift left when AI sidebar widens (before={}, after={})",
            layers_360.right(),
            layers_480.right()
        );
    }

    #[test]
    fn docked_sidebar_preserves_full_workspace_vertical_height() {
        use egui_kittest::kittest::Queryable;

        let mut session = photocraft_engine::Session::new();
        session.execute("file.new", json!({ "width": 400, "height": 300 })).unwrap();
        let mut app = PhotocraftApp::new(session, Default::default());
        app.ui.copilot.open = false;
        app.ui.copilot.docked = true;
        app.ui.copilot.sidebar_width = 360.0;

        // Baseline: measure full DockArea bottom edge (Layers group & canvas) when AI sidebar is closed.
        let mut h = crate::shortcut_dispatch::tests::app_harness(app);
        let layers_baseline = crate::panel_docking::probe::group_rect(&h, "layers")
            .expect("Layers panel group should be visible when sidebar is closed");
        let canvas_baseline = h.state().last_canvas_rect;
        assert!(
            layers_baseline.bottom() > 800.0 && canvas_baseline.bottom() > 800.0,
            "baseline workspace should fill vertical body down to status bar (layers={layers_baseline:?}, canvas={canvas_baseline:?})"
        );

        // Open docked AI sidebar in CLI mode with rich craft controls (Select, Checkbox x2, IconButton,
        // Button, Divider, CLI tool card, and composer) so any fill_body leakage would immediately steal height.
        {
            let app = h.state_mut();
            set_provider(app, AiProviderKind::AntigravityCli).expect("switch to Antigravity CLI");
            app.ui.copilot.open = true;
            app.ui.copilot.docked = true;
            app.ui.copilot.sidebar_width = 360.0;
            app.ui.copilot.cli_detection_cache.insert(
                AiProviderKind::AntigravityCli,
                CliDetectionStatus::Found(std::path::PathBuf::from(r"C:\agy\agy.exe")),
            );
            app.ui.copilot.photocraft_cli_detection = Some(CliDetectionStatus::Found(
                std::path::PathBuf::from(r"F:\PhotoCraft\photocraft-cli.exe"),
            ));
            app.ui.copilot.connection_status = Some("Connected to Antigravity CLI".into());
            app.ui.copilot.set_backend_error(AiBackendError::CliExecutionFailed {
                provider: "Antigravity CLI",
                exit_code: Some(1),
                summary: "Sample error banner with action button".into(),
            });
            app.ui.copilot.add_user_message("Inspect the active canvas and summarize layers.");
            apply_cli_tool_update(
                &mut app.ui.copilot,
                CliToolUpdate {
                    call_id: Some("call_height_test".into()),
                    step_index: None,
                    tool_name: "call_mcp_tool".into(),
                    state: CliToolState::Done,
                    parameters: json!({
                        "ServerName": "photocraft",
                        "ToolName": "doc_inspect",
                        "Arguments": { "include_layers": true }
                    }),
                    output: Some(json!({ "width": 400, "height": 300 })),
                    error: None,
                    tool_info: Value::Null,
                },
            );
            app.ui.copilot.add_model_message("Canvas is 400×300 px with one active layer.");
        }
        h.run_steps(6);

        let layers_cli = crate::panel_docking::probe::group_rect(&h, "layers")
            .expect("Layers panel group should remain visible beside docked CLI sidebar");
        let canvas_cli = h.state().last_canvas_rect;
        assert!(
            (layers_cli.bottom() - layers_baseline.bottom()).abs() <= 2.0,
            "Layers panel bottom ({}) must stay aligned with full body bottom ({}) when docked AI sidebar is open",
            layers_cli.bottom(),
            layers_baseline.bottom()
        );
        assert!(
            (canvas_cli.bottom() - canvas_baseline.bottom()).abs() <= 2.0,
            "Canvas bottom ({}) must stay aligned with full body bottom ({}) when docked AI sidebar is open",
            canvas_cli.bottom(),
            canvas_baseline.bottom()
        );
        assert!(
            layers_cli.right() <= 1440.0 - 360.0 + 0.5,
            "Layers panel right ({}) must remain strictly left of docked AI sidebar",
            layers_cli.right()
        );

        // Verify sidebar composer itself sits near the same body bottom edge (both sides stretch to status bar).
        let composer_rect = h
            .query_by_label("Ask AI Assistant")
            .expect("AI composer input should be present in accessibility tree")
            .rect();
        assert!(
            (composer_rect.bottom() - layers_cli.bottom()).abs() <= 40.0,
            "AI sidebar composer bottom ({}) should align near workspace bottom ({})",
            composer_rect.bottom(),
            layers_cli.bottom()
        );

        // Switch to Gemini API mode (missing API key card with primary Button) & widen sidebar to 480 px.
        {
            let app = h.state_mut();
            set_provider(app, AiProviderKind::GeminiApi).expect("switch to Gemini API");
            app.ui.gemini_api_key.clear();
            app.ui.copilot.sidebar_width = 480.0;
        }
        h.run_steps(6);

        let layers_api = crate::panel_docking::probe::group_rect(&h, "layers")
            .expect("Layers panel group should remain visible in Gemini API mode");
        let canvas_api = h.state().last_canvas_rect;
        assert!(
            (layers_api.bottom() - layers_baseline.bottom()).abs() <= 2.0,
            "Layers panel bottom ({}) must still fill full body height ({}) in Gemini API mode",
            layers_api.bottom(),
            layers_baseline.bottom()
        );
        assert!(
            (canvas_api.bottom() - canvas_baseline.bottom()).abs() <= 2.0,
            "Canvas bottom ({}) must still fill full body height ({}) in Gemini API mode",
            canvas_api.bottom(),
            canvas_baseline.bottom()
        );
        assert!(
            layers_api.right() <= 1440.0 - 480.0 + 0.5,
            "Layers panel right ({}) must remain strictly left of 480 px sidebar",
            layers_api.right()
        );
    }

    #[test]
    fn composer_paste_and_typing_do_not_trigger_canvas_shortcuts() {
        for docked in [true, false] {
            let mut session = photocraft_engine::Session::new();
            session.execute("file.new", json!({ "width": 160, "height": 120 })).unwrap();
            let mut app = PhotocraftApp::new(session, Default::default());
            // Put pixels on the canvas clipboard so canvas `edit.paste` IS enabled and would create a layer if intercepted.
            app.run("select.rect", json!({ "x": 0, "y": 0, "width": 16, "height": 16 })).unwrap();
            app.run("edit.copy", json!({})).unwrap();
            app.run("select.deselect", json!({})).unwrap();
            app.ui.tool = crate::state::Tool::Brush;
            app.ui.copilot.open = true;
            app.ui.copilot.docked = docked;
            app.ui.copilot.input.clear();

            let initial_layers = app.session.active().unwrap().doc.layers.len();
            let mut h = crate::shortcut_dispatch::tests::app_harness(app);

            // Focus the AI sidebar composer input (both docked and floating modes).
            h.ctx.memory_mut(|m| m.request_focus(composer_input_id()));
            h.run_steps(3);
            assert!(
                text_input_focused(&h.ctx),
                "composer should hold focus (docked={docked})"
            );

            // 1. Paste via Event::Paste (as Windows/macOS egui-winit emits on Ctrl+V with text clipboard).
            let cmd = egui::Modifiers {
                command: true,
                ctrl: !cfg!(target_os = "macos"),
                mac_cmd: cfg!(target_os = "macos"),
                ..Default::default()
            };
            h.input_mut().events.extend([
                egui::Event::ModifiersChanged(cmd),
                egui::Event::Paste("Brighten shadows".into()),
                egui::Event::Key {
                    key: egui::Key::V,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: cmd,
                },
                egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            ]);
            h.run_steps(4);

            assert_eq!(
                h.state().ui.copilot.input,
                "Brighten shadows",
                "pasted text must land in AI composer (docked={docked})"
            );
            assert_eq!(
                h.state().session.active().unwrap().doc.layers.len(),
                initial_layers,
                "canvas edit.paste must NOT fire while composer is focused (docked={docked})"
            );

            // 2. Ctrl+A must select text inside composer, NOT create a canvas selection.
            crate::shortcut_dispatch::tests::tap(&mut h, egui::Key::A, egui::Modifiers::COMMAND);
            assert!(
                h.state().session.active().unwrap().doc.selection.is_none(),
                "canvas select.all must NOT fire while composer is focused (docked={docked})"
            );

            // 3. Typing single tool letters (B, V, M, Space) must NOT switch tools.
            let tool_before = h.state().ui.tool;
            for (key, ch) in [
                (egui::Key::V, "v"),
                (egui::Key::M, "m"),
                (egui::Key::Space, " "),
                (egui::Key::B, "b"),
            ] {
                h.input_mut().events.extend([
                    egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::Text(ch.into()),
                    egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]);
                h.run_steps(2);
            }
            assert_eq!(
                h.state().ui.tool,
                tool_before,
                "typing B/V/M/Space must NOT trigger tool shortcuts (docked={docked})"
            );
            assert!(
                h.state().ui.copilot.input.contains("vm b"),
                "typed letters must appear in composer input, got {:?} (docked={docked})",
                h.state().ui.copilot.input
            );

            // 4. Shift+Enter inserts a newline without sending the message.
            let msgs_before = h.state().ui.copilot.messages.len();
            crate::shortcut_dispatch::tests::tap(&mut h, egui::Key::Enter, egui::Modifiers::SHIFT);
            assert!(
                h.state().ui.copilot.input.contains('\n'),
                "Shift+Enter must insert a newline in composer, got {:?} (docked={docked})",
                h.state().ui.copilot.input
            );
            assert_eq!(
                h.state().ui.copilot.messages.len(),
                msgs_before,
                "Shift+Enter must not submit the prompt (docked={docked})"
            );
        }
    }
}
