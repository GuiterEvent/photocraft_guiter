//! Gemini Copilot UI panel and interactive conversation interface.
//!
//! Provides an integrated AI assistant sidebar panel for PhotoCraft,
//! featuring multi-turn chat, markdown rendering, code block formatting,
//! async background request handling via GeminiClient, and preferences integration.

use std::sync::{Arc, Mutex, PoisonError};

use egui::{CornerRadius, Margin, Rect, RichText, Sense, Stroke, pos2, vec2};
use serde_json::{Value, json};

use crate::PhotocraftApp;
use crate::gemini::GeminiError;
#[cfg(not(target_arch = "wasm32"))]
use crate::gemini::GeminiClient;
use crate::icons;
use crate::menus::MenuItem;
use crate::state::{ChatImageAttachment, ChatMessage, ChatRole};
use crate::theme::{self, Tokens};
use crate::widgets;

pub const PANEL_ID: &str = "window.panel.geminiCopilot";

fn copilot_channel_id() -> egui::Id {
    egui::Id::unique("photocraft-gemini-copilot-channel")
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) enum CopilotWorkerResponse {
    StreamChunk {
        delta_text: String,
        function_calls: Vec<crate::gemini::FunctionCall>,
    },
    StreamFinished(Result<(), GeminiError>),
    #[allow(dead_code)]
    Turn(Result<crate::gemini::ModelTurn, GeminiError>),
    Test(Result<String, GeminiError>),
}

pub(crate) struct CopilotWorkerState {
    pub rx: std::sync::mpsc::Receiver<CopilotWorkerResponse>,
    pub stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub pending_function_calls: Vec<crate::gemini::FunctionCall>,
}

type CopilotChannel = Arc<Mutex<Option<CopilotWorkerState>>>;

/// Immediately requests cancellation of in-flight Gemini streaming generation.
pub fn stop_generating(app: &mut PhotocraftApp, ctx: &egui::Context) {
    app.ui.copilot.is_loading = false;
    app.ui.copilot.is_streaming = false;
    let channel = ctx.data_mut(|d| d.get_temp::<CopilotChannel>(copilot_channel_id()));
    if let Some(channel) = channel {
        let mut guard = channel.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(ref mut st) = *guard {
            st.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        *guard = None;
    }
}

/// Menu ids this panel handles.
pub fn handles(id: &str) -> bool {
    id == PANEL_ID || id == "window.toggle.geminiCopilot" || id == "view.geminiCopilot"
}

/// Checkmark state for the Window menu.
pub fn checked(app: &PhotocraftApp, id: &str) -> Option<bool> {
    handles(id).then_some(app.ui.copilot.open)
}

/// Menu command invocation to toggle the Copilot panel.
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
    Some(Ok(json!({ "open": show })))
}

/// Inserts the Gemini Copilot item into the Window menu list.
pub fn insert_menu_items(app: &PhotocraftApp, items: &mut Vec<MenuItem>) {
    let at = items
        .iter()
        .rposition(|i| i.path.first().map(|s| s.as_str()) == Some("Window") && i.id.starts_with("window.panel."))
        .unwrap_or(items.len());
    let idx = (at + 1).min(items.len());
    items.insert(
        idx,
        MenuItem {
            id: PANEL_ID.into(),
            label: "Gemini Copilot".into(),
            path: vec!["Window".into()],
            shortcut: None,
            enabled: true,
            checked: Some(app.ui.copilot.open),
            color: None,
        },
    );
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

/// Polls for completed background responses from worker threads.
fn check_background_response(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let channel = ctx.data_mut(|d| d.get_temp::<CopilotChannel>(copilot_channel_id()));
    let Some(channel) = channel else { return };
    let mut guard = channel.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(st) = guard.as_mut() else { return };

    let mut auto_respond_turn = false;

    match st.rx.try_recv() {
        Ok(CopilotWorkerResponse::StreamChunk { delta_text, function_calls }) => {
            app.ui.copilot.is_streaming = true;
            if !delta_text.is_empty() {
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
            if !function_calls.is_empty() {
                st.pending_function_calls.extend(function_calls);
            }
        }
        Ok(CopilotWorkerResponse::StreamFinished(res)) => {
            let pending_fcs = std::mem::take(&mut st.pending_function_calls);
            *guard = None;
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;

            match res {
                Ok(()) => {
                    app.ui.copilot.error_message = None;
                    for fc in pending_fcs {
                        match fc.name.as_str() {
                            "plan_workflow_actions" => {
                                match crate::gemini::parse_workflow_plan(&fc.args) {
                                    Ok(plan) => {
                                        let summary = format!("{}: {}", tl!("Suggested Workflow Plan"), plan.title);
                                        app.ui.copilot.messages.push(ChatMessage::workflow_plan(summary, plan));
                                    }
                                    Err(err) => {
                                        app.ui.copilot.error_message = Some(format!("Failed to parse workflow plan: {err}"));
                                    }
                                }
                            }
                            "get_document_info" => {
                                let doc_info = query_document_info(app);
                                let summary_text = tl!("Reading document info…").to_string();
                                let action = crate::state::ToolCallAction {
                                    name: fc.name.clone(),
                                    args: fc.args.clone(),
                                    status: crate::state::ToolActionStatus::Completed,
                                    result: Some(doc_info.clone()),
                                    folded: true,
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
                                };
                                app.ui.copilot.messages.push(ChatMessage::tool_call(prompt_text, action));
                            }
                        }
                    }
                }
                Err(err) => {
                    app.ui.copilot.error_message = Some(err.to_string());
                }
            }
            app.ui.copilot.sync_active_to_session();
        }
        Ok(CopilotWorkerResponse::Turn(res)) => {
            *guard = None;
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;
            match res {
                Ok(turn) => {
                    app.ui.copilot.error_message = None;
                    if turn.function_calls.is_empty()
                        && let Some(text) = turn.text
                    {
                        app.ui.copilot.messages.push(ChatMessage::model(text));
                    }
                }
                Err(err) => {
                    app.ui.copilot.error_message = Some(err.to_string());
                }
            }
            app.ui.copilot.sync_active_to_session();
        }
        Ok(CopilotWorkerResponse::Test(res)) => {
            *guard = None;
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;
            match res {
                Ok(_) => {
                    app.ui.copilot.connection_status = Some("Connection test successful! Gemini is ready.".to_string());
                    app.ui.copilot.error_message = None;
                }
                Err(err) => {
                    app.ui.copilot.connection_status = None;
                    app.ui.copilot.error_message = Some(format!("Connection test failed: {err}"));
                }
            }
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            // Still waiting
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            *guard = None;
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;
        }
    }

    if auto_respond_turn {
        send_chat_turn(app, ctx);
    }
}

/// Sends the user's chat message to the Gemini API.
pub fn send_chat(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let input = app.ui.copilot.input.trim().to_string();
    let has_attachment = app.ui.copilot.pending_attachment.is_some();
    if input.is_empty() && !has_attachment {
        return;
    }

    if app.ui.gemini_api_key.trim().is_empty() {
        app.ui.copilot.error_message = Some("Gemini API key is not configured. Please enter your API key in Edit > Preferences > AI Integrations.".to_string());
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
    app.ui.copilot.error_message = None;
    app.ui.copilot.connection_status = None;
    app.ui.copilot.sync_active_to_session();

    send_chat_turn(app, ctx);
}

/// Dispatches the current conversation turns and tool declarations to Gemini API with SSE streaming.
pub fn send_chat_turn(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if app.ui.gemini_api_key.trim().is_empty() {
        app.ui.copilot.error_message = Some("Gemini API key is not configured. Please enter your API key in Edit > Preferences > AI Integrations.".to_string());
        app.ui.copilot.is_loading = false;
        app.ui.copilot.is_streaming = false;
        return;
    }

    app.ui.copilot.is_loading = true;
    app.ui.copilot.is_streaming = true;
    app.ui.copilot.error_message = None;

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Prune context messages to keep within 32,000 tokens limit
        let pruned_messages = crate::state::prune_messages_for_context(&app.ui.copilot.messages, 32_000);
        let contents = crate::gemini::chat_messages_to_contents(&pruned_messages);
        let tools = Some(crate::gemini::photocraft_tools());
        let client = GeminiClient::new(&app.ui.gemini_api_key, &app.ui.gemini_model);
        let ctx_clone = ctx.clone();
        let (tx, rx) = std::sync::mpsc::channel::<CopilotWorkerResponse>();
        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
        let worker_state = CopilotWorkerState {
            rx,
            stop_flag,
            pending_function_calls: Vec::new(),
        };
        let channel: CopilotChannel = Arc::new(Mutex::new(Some(worker_state)));
        ctx.data_mut(|d| d.insert_temp(copilot_channel_id(), channel));

        let spawn_res = std::thread::Builder::new()
            .name("photocraft-copilot".to_string())
            .spawn(move || {
                let tx_chunk = tx.clone();
                let ctx_chunk = ctx_clone.clone();
                let result = pollster::block_on(client.stream_chat_turn(
                    &contents,
                    tools,
                    stop_flag_clone,
                    move |delta_text, function_calls| {
                        let _ = tx_chunk.send(CopilotWorkerResponse::StreamChunk {
                            delta_text,
                            function_calls,
                        });
                        ctx_chunk.request_repaint();
                        Ok(())
                    },
                ));
                let _ = tx.send(CopilotWorkerResponse::StreamFinished(result));
                ctx_clone.request_repaint();
            });

        if let Err(e) = spawn_res {
            app.ui.copilot.is_loading = false;
            app.ui.copilot.is_streaming = false;
            app.ui.copilot.error_message = Some(format!("Failed to spawn worker thread: {e}"));
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = ctx;
        app.ui.copilot.is_loading = false;
        app.ui.copilot.is_streaming = false;
        app.ui.copilot.error_message = Some("Gemini Copilot networking is not supported on WebAssembly yet.".to_string());
    }
}

/// Triggers a test connection request to verify the Gemini API key and connectivity.
pub fn test_connection(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if app.ui.gemini_api_key.trim().is_empty() {
        app.ui.copilot.error_message = Some("Gemini API key is not configured. Please enter your API key in Edit > Preferences > AI Integrations.".to_string());
        return;
    }

    app.ui.copilot.is_loading = true;
    app.ui.copilot.connection_status = Some("Testing connection to Gemini API...".to_string());
    app.ui.copilot.error_message = None;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = GeminiClient::new(&app.ui.gemini_api_key, &app.ui.gemini_model);
        let ctx_clone = ctx.clone();
        let (tx, rx) = std::sync::mpsc::channel::<CopilotWorkerResponse>();
        let worker_state = CopilotWorkerState {
            rx,
            stop_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_function_calls: Vec::new(),
        };
        let channel: CopilotChannel = Arc::new(Mutex::new(Some(worker_state)));
        ctx.data_mut(|d| d.insert_temp(copilot_channel_id(), channel));

        let spawn_res = std::thread::Builder::new()
            .name("photocraft-copilot-test".to_string())
            .spawn(move || {
                let result = pollster::block_on(client.test_connection());
                let _ = tx.send(CopilotWorkerResponse::Test(result));
                ctx_clone.request_repaint();
            });

        if let Err(e) = spawn_res {
            app.ui.copilot.is_loading = false;
            app.ui.copilot.error_message = Some(format!("Failed to spawn worker thread: {e}"));
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = ctx;
        app.ui.copilot.is_loading = false;
        app.ui.copilot.error_message = Some("Gemini Copilot networking is not supported on WebAssembly yet.".to_string());
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
                                ui.add(
                                    egui::TextEdit::singleline(&mut copilot.rename_input)
                                        .desired_width(ui.available_width() - 44.0),
                                );
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

/// Main rendering entry point for the Gemini Copilot panel.
pub fn windows(app: &mut PhotocraftApp, ctx: &egui::Context) {
    check_background_response(app, ctx);

    if !app.ui.copilot.open {
        return;
    }

    app.ui.copilot.ensure_active_session();

    let t = Tokens::get(ctx);
    let canvas = app.last_canvas_rect;
    let width = 360.0;
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

    let mut close = false;
    let mut clear_chat = false;
    let mut do_test_connection = false;
    let mut open_prefs = false;
    let mut send_prompt = false;
    let mut capture_screenshot = false;
    let mut open_expanded_image: Option<usize> = None;
    let mut suggestion_to_send: Option<String> = None;
    let mut action_to_execute: Option<usize> = None;
    let mut action_to_cancel: Option<usize> = None;
    let mut toggle_folded: Option<usize> = None;
    let mut execute_workflow_step_action: Option<(usize, usize)> = None;
    let mut skip_workflow_step_action: Option<(usize, usize)> = None;
    let mut execute_workflow_all_action: Option<usize> = None;
    let mut stop_generation = false;
    let mut new_chat = false;
    let mut switch_session_id: Option<String> = None;
    let mut delete_session_id: Option<String> = None;

    // Shortcut Ctrl+Shift+C / Cmd+Shift+C to toggle/attach canvas screenshot
    let shortcut_attach = ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::C));
    if shortcut_attach {
        capture_screenshot = true;
    }

    egui::Window::new(tl!("Gemini Copilot"))
        .id(egui::Id::unique("photocraft-gemini-copilot-window"))
        .title_bar(false)
        .resizable(true)
        .movable(!app.session.prefs().workspace_locked)
        .default_pos(pos2(default_x, default_y))
        .default_size(vec2(width, 520.0))
        .min_width(280.0)
        .max_width(600.0)
        .min_height(320.0)
        .frame(frame)
        .show(ctx, |ui| {
            // Header bar
            ui.horizontal(|ui| {
                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(18.0, 18.0)), "sparkles", 16.0, t.accent);
                ui.add_space(20.0);
                ui.label(RichText::new(tl!("Gemini Copilot")).strong().size(13.0).color(t.text));

                // Session switcher button / drawer toggle
                let active_title = app.ui.copilot.sessions
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
                        ui.label(RichText::new(format!("{truncated_title} ▾")).size(9.5).color(t.text));
                    });
                });
                if chip_resp.response.interact(Sense::click()).clicked() {
                    app.ui.copilot.history_drawer_open = !app.ui.copilot.history_drawer_open;
                }

                // New Chat button
                if icons::button(ui, "plus", 18.0, false, tl!("New Chat")).clicked() {
                    new_chat = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, "x", 20.0, false, tl!("Close")).clicked() {
                        close = true;
                    }
                    if icons::button(ui, "trash", 20.0, false, tl!("Clear chat")).clicked() {
                        clear_chat = true;
                    }
                    if icons::button(ui, "rotate-cw", 20.0, false, tl!("Test connection")).clicked() {
                        do_test_connection = true;
                    }
                });
            });

            ui.add_space(4.0);

            // History drawer if open
            if app.ui.copilot.history_drawer_open {
                render_history_drawer(
                    &mut app.ui.copilot,
                    ui,
                    &t,
                    &mut switch_session_id,
                    &mut delete_session_id,
                    &mut new_chat,
                );
                ui.add_space(4.0);
            }

            crate::widgets::hairline(ui);
            ui.add_space(4.0);

            // API key missing guidance banner
            let has_key = !app.ui.gemini_api_key.trim().is_empty();
            if !has_key {
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
                    ui.label(RichText::new(tl!("Configure your Google Gemini API key to enable instant AI assistance for PhotoCraft.")).size(10.5).color(t.text_dim));
                    ui.add_space(4.0);
                    if widgets::primary_button(ui, tl!("Configure in Preferences…"), 0.0).clicked() {
                        open_prefs = true;
                    }
                });
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
                    ui.horizontal(|ui| {
                        icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "check", 12.0, t.accent);
                        ui.add_space(16.0);
                        ui.label(RichText::new(status).size(10.5).color(t.accent_text));
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
                    ui.horizontal(|ui| {
                        icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "triangle-alert", 12.0, t.danger);
                        ui.add_space(16.0);
                        ui.label(RichText::new(err).size(10.5).color(t.danger));
                    });
                    if err.contains("API key") || err.contains("configured") {
                        ui.add_space(3.0);
                        if widgets::secondary_button(ui, tl!("Open Preferences…"), 0.0).clicked() {
                            open_prefs = true;
                        }
                    }
                });
                ui.add_space(4.0);
            }

            // Chat conversation scroll area
            let scroll_height = (ui.available_height() - 96.0).max(120.0);
            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if app.ui.copilot.messages.is_empty() {
                        render_empty_state(ui, &t, &mut suggestion_to_send);
                    } else {
                        let is_streaming = app.ui.copilot.is_streaming;
                        let last_model_idx = app.ui.copilot.messages
                            .iter()
                            .rposition(|m| m.role == ChatRole::Model && m.tool_call.is_none() && m.workflow_plan.is_none());

                        let mut dispatch = ToolCardDispatch {
                            execute: &mut action_to_execute,
                            cancel: &mut action_to_cancel,
                            toggle_folded: &mut toggle_folded,
                            open_expanded: &mut open_expanded_image,
                            execute_workflow_step: &mut execute_workflow_step_action,
                            skip_workflow_step: &mut skip_workflow_step_action,
                            execute_workflow_all: &mut execute_workflow_all_action,
                            is_streaming,
                            is_last_model_msg: false,
                        };
                        for (idx, msg) in app.ui.copilot.messages.iter().enumerate() {
                            dispatch.is_last_model_msg = Some(idx) == last_model_idx;
                            render_chat_message(
                                ui,
                                idx,
                                msg,
                                &t,
                                &mut dispatch,
                            );
                            ui.add_space(6.0);
                        }
                    }

                    if app.ui.copilot.is_loading && !app.ui.copilot.is_streaming {
                        render_loading_bubble(ui, &t);
                        ui.add_space(4.0);
                    }
                });

            ui.add_space(4.0);
            crate::widgets::hairline(ui);
            ui.add_space(3.0);

            // Visual action toolbar & preset prompts
            ui.horizontal(|ui| {
                let has_doc = app.session.active().is_some();
                let has_att = app.ui.copilot.pending_attachment.is_some();

                // Attach canvas screenshot button
                ui.add_enabled_ui(has_doc, |ui| {
                    let btn_tooltip = if has_doc {
                        tl!("Attach current canvas screenshot (Ctrl+Shift+C)")
                    } else {
                        tl!("No active document to capture")
                    };
                    if icons::button(ui, "image", 20.0, has_att, btn_tooltip).clicked() {
                        capture_screenshot = true;
                    }
                });

                // Presets menu button
                ui.menu_button(RichText::new(tl!("Vision Presets ▾")).size(10.5).color(t.text_dim), |ui| {
                    ui.set_max_width(240.0);
                    if ui.button(tl!("📸 Exposure & Histogram")).clicked() {
                        attach_and_send_preset(app, ctx, tl!("Analyze the exposure, highlights/shadows clipping, and dynamic range of this canvas image, and suggest adjustments.").to_string());
                        ui.close();
                    }
                    if ui.button(tl!("🎨 Color Grading Style")).clicked() {
                        attach_and_send_preset(app, ctx, tl!("Evaluate the color temperature, cast, and tones in this canvas image, and suggest 2-3 suitable color grading styles with concrete PhotoCraft steps.").to_string());
                        ui.close();
                    }
                    if ui.button(tl!("🔍 Retouching Defects")).clicked() {
                        attach_and_send_preset(app, ctx, tl!("Diagnose any visual defects, unwanted noise, skin blemishes, or compositional issues in this image, and recommend retouching tools to fix them.").to_string());
                        ui.close();
                    }
                    if ui.button(tl!("📐 Composition & Cropping")).clicked() {
                        attach_and_send_preset(app, ctx, tl!("Analyze the composition, rule of thirds, subject balance, and suggest optimal cropping or framing.").to_string());
                        ui.close();
                    }
                });
            });

            // Pending attachment preview chip
            if let Some(ref att) = app.ui.copilot.pending_attachment {
                let chip_frame = egui::Frame::NONE
                    .fill(t.field)
                    .stroke(Stroke::new(1.0, t.accent_border))
                    .corner_radius(CornerRadius::same(t.radius_sm as u8))
                    .inner_margin(Margin::symmetric(8, 4));

                let mut remove_attachment = false;
                chip_frame.show(ui, |ui| {
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
                    app.ui.copilot.pending_attachment = None;
                }
                ui.add_space(2.0);
            }

            // Bottom input bar
            ui.horizontal(|ui| {
                let input_width = (ui.available_width() - 40.0).max(120.0);
                let text_edit_response = ui.add(
                    egui::TextEdit::multiline(&mut app.ui.copilot.input)
                        .desired_rows(2)
                        .lock_focus(true)
                        .hint_text(tl!("Ask Gemini Copilot… (Enter to send, Shift+Enter for new line)"))
                        .desired_width(input_width),
                );

                // Check for Enter key to send (Shift+Enter inserts newline)
                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let shift_down = ui.input(|i| i.modifiers.shift);
                if text_edit_response.has_focus() && enter_pressed && !shift_down {
                    if app.ui.copilot.input.ends_with('\n') {
                        app.ui.copilot.input.pop();
                    }
                    send_prompt = true;
                }

                // Send or Stop button
                let is_active_gen = app.ui.copilot.is_loading || app.ui.copilot.is_streaming;
                if is_active_gen {
                    if ui.button(RichText::new("■").strong().size(16.0).color(t.danger)).on_hover_text(tl!("Stop Generating")).clicked() {
                        stop_generation = true;
                    }
                } else {
                    let can_send = !app.ui.copilot.input.trim().is_empty() || app.ui.copilot.pending_attachment.is_some();
                    ui.add_enabled_ui(can_send, |ui| {
                        if icons::button(ui, "sparkles", 32.0, false, tl!("Send prompt")).clicked() {
                            send_prompt = true;
                        }
                    });
                }
            });
        });

    if stop_generation {
        stop_generating(app, ctx);
    }
    if new_chat {
        app.ui.copilot.create_new_session(None);
    }
    if let Some(id) = switch_session_id {
        app.ui.copilot.switch_session(&id);
    }
    if let Some(id) = delete_session_id {
        app.ui.copilot.delete_session(&id);
    }
    if let Some((msg_idx, step_idx)) = execute_workflow_step_action {
        execute_workflow_step(app, ctx, msg_idx, step_idx);
    }
    if let Some((msg_idx, step_idx)) = skip_workflow_step_action {
        skip_workflow_step(app, ctx, msg_idx, step_idx);
    }
    if let Some(msg_idx) = execute_workflow_all_action {
        execute_entire_workflow(app, ctx, msg_idx);
    }

    if capture_screenshot && app.session.active().is_some() {
        if app.ui.copilot.pending_attachment.is_some() {
            app.ui.copilot.pending_attachment = None;
        } else {
            match perform_canvas_inspection(app, "composite", crate::gemini::DEFAULT_VISION_MAX_EDGE) {
                Ok((att, _)) => app.ui.copilot.pending_attachment = Some(att),
                Err(err) => app.ui.copilot.error_message = Some(err),
            }
        }
    }

    if let Some(idx) = open_expanded_image {
        app.ui.copilot.expanded_image_preview = Some(idx);
    }
    if let Some(idx) = app.ui.copilot.expanded_image_preview {
        render_expanded_image_modal(app, ctx, &t, idx);
    }

    if let Some(idx) = toggle_folded
        && let Some(msg) = app.ui.copilot.messages.get_mut(idx)
        && let Some(ref mut tc) = msg.tool_call
    {
        tc.folded = !tc.folded;
    }
    if let Some(idx) = action_to_execute {
        execute_tool_action(app, ctx, idx);
    }
    if let Some(idx) = action_to_cancel {
        cancel_tool_action(app, idx);
    }

    if close {
        app.ui.copilot.open = false;
    }
    if clear_chat {
        app.ui.copilot.clear();
    }
    if do_test_connection {
        test_connection(app, ctx);
    }
    if open_prefs {
        let _ = crate::menus::invoke(app, ctx, "edit.preferences.integrations", json!({}));
    }
    if let Some(suggestion) = suggestion_to_send {
        app.ui.copilot.input = suggestion;
        send_chat(app, ctx);
    } else if send_prompt {
        send_chat(app, ctx);
    }
}

/// Renders a welcoming placeholder with suggestion prompt chips.
fn render_empty_state(ui: &mut egui::Ui, t: &Tokens, suggestion_out: &mut Option<String>) {
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        icons::paint(ui, Rect::from_center_size(ui.cursor().center(), vec2(32.0, 32.0)), "sparkles", 26.0, t.accent);
        ui.add_space(34.0);
        ui.label(RichText::new(tl!("How can I assist you?")).strong().size(13.5).color(t.text));
        ui.add_space(2.0);
        ui.label(
            RichText::new(tl!("Ask anything about image editing, visual diagnosis, color grading, or shortcuts."))
                .size(10.5)
                .color(t.text_dim),
        );
        ui.add_space(10.0);

        let suggestions = [
            tl!("📸 Analyze exposure & histogram of this image"),
            tl!("🎨 Suggest color grading styles and filter steps"),
            tl!("🔍 Diagnose retouching defects & noise in this photo"),
            tl!("How do I create and refine a layer mask?"),
            tl!("Explain the difference between Multiply and Screen blend modes"),
            tl!("What are the most essential keyboard shortcuts?"),
        ];

        for prompt in suggestions {
            let chip_frame = egui::Frame::NONE
                .fill(t.field)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(t.radius_sm as u8))
                .inner_margin(Margin::symmetric(10, 5));

            let resp = chip_frame.show(ui, |ui| {
                ui.label(RichText::new(prompt).size(10.0).color(t.text));
            });

            if resp.response.interact(Sense::click()).clicked() {
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
        ChatRole::Model => (t.card, t.card_border, tl!("Gemini Copilot"), t.accent),
        ChatRole::System | ChatRole::Function => (t.dock, t.card_border, tl!("System"), t.text_dim),
    };

    let msg_frame = egui::Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(10, 8));

    msg_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            if msg.role == ChatRole::Model {
                icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "sparkles", 12.0, t.accent);
                ui.add_space(16.0);
            }
            ui.label(RichText::new(role_label).strong().size(11.0).color(role_color));

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
            if dispatch.is_streaming && dispatch.is_last_model_msg {
                let mut streaming_content = msg.content.clone();
                streaming_content.push('▮');
                render_markdown(ui, &streaming_content, t);
            } else {
                render_markdown(ui, &msg.content, t);
            }
        } else {
            ui.label(RichText::new(&msg.content).size(11.5).color(t.text));
        }

        if let Some(att) = &msg.image {
            ui.add_space(4.0);
            render_image_attachment(ui, att, t, dispatch.open_expanded, msg_idx);
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
                let fold_label = if tc.folded {
                    format!("{} ▾", tl!("Details"))
                } else {
                    format!("{} ▴", tl!("Details"))
                };
                if ui.small_button(RichText::new(fold_label).size(10.0).color(t.accent_text)).clicked() {
                    *dispatch.toggle_folded = Some(msg_idx);
                }
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

        // Action confirmation buttons for PendingConfirmation
        if tc.status == crate::state::ToolActionStatus::PendingConfirmation {
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
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(12.0, 12.0)), "check", 10.0, t.accent);
            ui.add_space(14.0);
            ui.label(RichText::new(&msg.content).size(10.0).color(t.text_dim));

            if let Some(tc) = &msg.tool_call
                && tc.result.is_some()
            {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let arrow = if tc.folded { "▾" } else { "▴" };
                    if ui.small_button(RichText::new(arrow).size(10.0).color(t.text_dim)).clicked() {
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
            let s = serde_json::to_string(res).unwrap_or_default();
            ui.label(RichText::new(s).monospace().size(9.5).color(t.text_dim));
        }
    });
}

/// Renders an in-flight thinking bubble.
fn render_loading_bubble(ui: &mut egui::Ui, t: &Tokens) {
    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(Stroke::new(1.0, t.card_border))
        .corner_radius(CornerRadius::same(t.radius as u8))
        .inner_margin(Margin::symmetric(10, 8));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(14.0, 14.0)), "sparkles", 12.0, t.accent);
            ui.add_space(16.0);
            ui.label(RichText::new(tl!("Gemini is thinking…")).italics().size(11.0).color(t.text_dim));
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
                // End code block
                let block_frame = egui::Frame::NONE
                    .fill(t.field)
                    .stroke(Stroke::new(1.0, t.field_border))
                    .corner_radius(CornerRadius::same(t.radius_sm as u8))
                    .inner_margin(Margin::symmetric(8, 6));

                block_frame.show(ui, |ui| {
                    ui.label(RichText::new(&code_accumulator).monospace().size(10.5).color(t.text));
                });
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
            ui.label(RichText::new(h1).strong().size(13.5).color(t.text));
            ui.add_space(2.0);
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(3.0);
            ui.label(RichText::new(h2).strong().size(12.5).color(t.text));
            ui.add_space(1.0);
        } else if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(2.0);
            ui.label(RichText::new(h3).strong().size(11.5).color(t.text));
        } else if let Some(bullet) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("•").strong().color(t.accent));
                render_inline_markdown(ui, bullet, t);
            });
        } else if let Some((num, rest)) = trimmed.split_once(". ")
            && num.chars().all(|c| c.is_ascii_digit())
        {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{num}.")).strong().color(t.accent));
                render_inline_markdown(ui, rest, t);
            });
        } else if trimmed.is_empty() {
            ui.add_space(3.0);
        } else {
            render_inline_markdown(ui, line, t);
        }
    }

    // Flush any open code block if input ended without closing backticks
    if in_code_block && !code_accumulator.is_empty() {
        let block_frame = egui::Frame::NONE
            .fill(t.field)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(t.radius_sm as u8))
            .inner_margin(Margin::symmetric(8, 6));

        block_frame.show(ui, |ui| {
            ui.label(RichText::new(&code_accumulator).monospace().size(10.5).color(t.text));
        });
    }
}

/// Renders a single line with inline formatting (**bold**, `code`).
fn render_inline_markdown(ui: &mut egui::Ui, text: &str, t: &Tokens) {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.break_anywhere = false;
    job.wrap.max_width = ui.available_width();

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

    ui.label(job);
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
}
