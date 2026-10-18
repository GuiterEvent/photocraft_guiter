//! Gemini AI API client module for PhotoCraft assistant.
//!
//! Provides asynchronous communication with Google Generative Language API
//! (`/v1beta/models/{model}:generateContent`), structured request/response types,
//! and comprehensive error handling.

use serde::{Deserialize, Serialize};

/// Configuration for the Gemini API client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeminiConfig {
    /// Google Gemini API key.
    pub api_key: String,
    /// Model identifier, e.g. "gemini-2.5-flash", "gemini-1.5-pro", "gemini-2.5-pro".
    pub model: String,
    /// Base URL for the Generative Language API.
    pub base_url: String,
    /// Timeout for network requests in seconds.
    pub timeout_secs: u64,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: photocraft_engine::prefs::default_gemini_model(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Errors that can occur during Gemini API interaction.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GeminiError {
    /// API key was empty or missing.
    #[error("Gemini API key is not configured. Please enter your API key in Edit > Preferences > Integrations.")]
    MissingApiKey,

    /// Invalid API key (HTTP 400 with API_KEY_INVALID or HTTP 403 PERMISSION_DENIED).
    #[error("Gemini API key is invalid or unauthorized: {0}")]
    InvalidApiKey(String),

    /// Quota exceeded or rate limited (HTTP 429 RESOURCE_EXHAUSTED).
    #[error("Gemini API quota exceeded or rate limited: {0}")]
    QuotaExceeded(String),

    /// Model not found (HTTP 404).
    #[error("Gemini model not found: {0}")]
    ModelNotFound(String),

    /// Request timed out.
    #[error("Gemini request timed out: {0}")]
    Timeout(String),

    /// Network connection error.
    #[error("Network connection error while contacting Gemini API: {0}")]
    NetworkError(String),

    /// General API error returned from the Gemini server.
    #[error("Gemini API error ({code} {status}): {message}")]
    ApiError {
        code: u16,
        status: String,
        message: String,
    },

    /// Response JSON parsing error.
    #[error("Failed to parse Gemini API response: {0}")]
    ParseError(String),

    /// Response blocked by safety settings.
    #[error("Gemini response was blocked by safety filters: {0}")]
    SafetyBlocked(String),

    /// Response was empty or contained no candidates.
    #[error("Gemini returned an empty response.")]
    EmptyResponse,

    /// Platform does not support this operation.
    #[error("Gemini API is not supported on this platform: {0}")]
    UnsupportedPlatform(String),
}

/// Request payload sent to `/v1beta/models/{model}:generateContent`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemInstruction")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

/// Tool configuration provided in Gemini API requests.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    #[serde(default, rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

/// Declaration of a tool function callable by the model.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// A function call request emitted by the model.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

/// A function response payload sent back to the model.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

/// Constructs the standard PhotoCraft tool declarations for Gemini Function Calling.
pub fn photocraft_tools() -> Vec<Tool> {
    vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "get_document_info".to_string(),
                description: "Query current canvas dimensions, color mode, bit depth, layer count, and active layer details in PhotoCraft.".to_string(),
                parameters: Some(serde_json::json!({
                    "type": "OBJECT",
                    "properties": {}
                })),
            },
            FunctionDeclaration {
                name: "execute_command".to_string(),
                description: "Execute a PhotoCraft engine command by command ID with parameters (e.g., 'image.adjustments.invert', 'layer.new.layer', 'layer.delete', 'edit.undo').".to_string(),
                parameters: Some(serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "command_id": {
                            "type": "STRING",
                            "description": "The exact PhotoCraft command ID to execute, e.g. 'image.adjustments.invert'."
                        },
                        "params": {
                            "type": "OBJECT",
                            "description": "Optional JSON parameters to pass to the command."
                        }
                    },
                    "required": ["command_id"]
                })),
            },
            FunctionDeclaration {
                name: "adjust_layer_property".to_string(),
                description: "Adjust the active layer's opacity, blend mode, or visibility in PhotoCraft.".to_string(),
                parameters: Some(serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "opacity": {
                            "type": "NUMBER",
                            "description": "Opacity value, either 0.0 to 1.0 or 0 to 100 percent."
                        },
                        "blend_mode": {
                            "type": "STRING",
                            "description": "Blend mode name, e.g. 'Normal', 'Multiply', 'Screen', 'Overlay', 'Darken', 'Lighten', 'Color Dodge', 'Soft Light'."
                        },
                        "visible": {
                            "type": "BOOLEAN",
                            "description": "Whether the active layer is visible (true or false)."
                        }
                    }
                })),
            },
            FunctionDeclaration {
                name: "inspect_canvas_image".to_string(),
                description: "Capture and visually inspect the current canvas rendering or active layer to diagnose exposure, color balance, retouching defects, composition, or clipping.".to_string(),
                parameters: Some(serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "target": {
                            "type": "STRING",
                            "description": "Visual target to inspect: 'composite' for the full rendered canvas (default), or 'active_layer' for the active layer alone."
                        },
                        "detail": {
                            "type": "STRING",
                            "description": "Level of image detail: 'standard' (1024px max edge) or 'high' (1536px max edge)."
                        }
                    }
                })),
            },
            FunctionDeclaration {
                name: "plan_workflow_actions".to_string(),
                description: "Propose an ordered multi-step image editing macro workflow plan in PhotoCraft. The user can review the checklist, step through, or execute all steps continuously.".to_string(),
                parameters: Some(serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "title": {
                            "type": "STRING",
                            "description": "Short title of the workflow plan (e.g. 'High-Contrast B&W Conversion', 'Soft Glow Portrait', 'Color Correction')."
                        },
                        "description": {
                            "type": "STRING",
                            "description": "Clear explanation of what the multi-step workflow accomplishes."
                        },
                        "steps": {
                            "type": "ARRAY",
                            "description": "List of action steps in chronological execution order.",
                            "items": {
                                "type": "OBJECT",
                                "properties": {
                                    "id": {
                                        "type": "STRING",
                                        "description": "Step identifier, e.g. 'step_1'."
                                    },
                                    "description": {
                                        "type": "STRING",
                                        "description": "Clear user-facing description of what this step does."
                                    },
                                    "tool": {
                                        "type": "STRING",
                                        "description": "Action type: 'execute_command' or 'adjust_layer_property'."
                                    },
                                    "command_id": {
                                        "type": "STRING",
                                        "description": "PhotoCraft command ID if tool is execute_command (e.g. 'image.adjustments.invert', 'layer.new.layer')."
                                    },
                                    "params": {
                                        "type": "OBJECT",
                                        "description": "Optional parameters for the command or layer property adjustment."
                                    }
                                },
                                "required": ["description", "tool"]
                            }
                        }
                    },
                    "required": ["title", "steps"]
                })),
            },
        ],
    }]
}

/// Represents the parsed result of a model generation turn, including text and/or function calls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelTurn {
    pub text: Option<String>,
    pub function_calls: Vec<FunctionCall>,
}

impl ModelTurn {
    pub fn is_function_call(&self) -> bool {
        !self.function_calls.is_empty()
    }

    pub fn first_function_call(&self) -> Option<&FunctionCall> {
        self.function_calls.first()
    }
}

/// A conversation turn or message content.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<Part>,
}

/// Content part (text, inline data, function call, or function response).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Part {
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponse,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
    Text {
        text: String,
    },
}

/// Base64 encoded binary data (e.g. image).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InlineData {
    #[serde(rename = "mimeType", alias = "mime_type")]
    pub mime_type: String,
    pub data: String,
}

impl InlineData {
    pub fn new(mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data: data.into(),
        }
    }

    pub fn jpeg(base64_data: impl Into<String>) -> Self {
        Self::new("image/jpeg", base64_data)
    }

    pub fn png(base64_data: impl Into<String>) -> Self {
        Self::new("image/png", base64_data)
    }
}

impl Part {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn inline_data(mime_type: impl Into<String>, base64_data: impl Into<String>) -> Self {
        Self::InlineData {
            inline_data: InlineData::new(mime_type, base64_data),
        }
    }

    pub fn image_jpeg(base64_data: impl Into<String>) -> Self {
        Self::InlineData {
            inline_data: InlineData::jpeg(base64_data),
        }
    }

    pub fn image_png(base64_data: impl Into<String>) -> Self {
        Self::InlineData {
            inline_data: InlineData::png(base64_data),
        }
    }
}

/// Generation configuration parameters.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxOutputTokens")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "topP")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "topK")]
    pub top_k: Option<u32>,
}

/// Response payload received from `/v1beta/models/{model}:generateContent`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GenerateContentResponse {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "promptFeedback")]
    pub prompt_feedback: Option<PromptFeedback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorDetail>,
}

/// A candidate response from the model.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Candidate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "finishReason")]
    pub finish_reason: Option<String>,
}

/// Usage metadata for the request.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageMetadata {
    #[serde(default, rename = "promptTokenCount")]
    pub prompt_token_count: u32,
    #[serde(default, rename = "candidatesTokenCount")]
    pub candidates_token_count: u32,
    #[serde(default, rename = "totalTokenCount")]
    pub total_token_count: u32,
}

/// Prompt feedback detailing any blocks.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none", rename = "blockReason")]
    pub block_reason: Option<String>,
}

/// Error envelope returned by Google API.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorDetail,
}

/// Detailed error information from Google API.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiErrorDetail {
    #[serde(default)]
    pub code: u16,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub status: String,
}

/// Client for communicating with the Gemini API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeminiClient {
    config: GeminiConfig,
}

impl GeminiClient {
    /// Creates a client with an API key and model name, using default base URL.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::with_config(GeminiConfig {
            api_key: api_key.into(),
            model: model.into(),
            ..Default::default()
        })
    }

    /// Creates a client with custom configuration.
    pub fn with_config(config: GeminiConfig) -> Self {
        Self { config }
    }

    /// Current configuration reference.
    pub fn config(&self) -> &GeminiConfig {
        &self.config
    }

    /// Current API key.
    pub fn api_key(&self) -> &str {
        &self.config.api_key
    }

    /// Current model identifier.
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Current base URL.
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Formats the target endpoint URL.
    pub fn build_endpoint_url(&self) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        let model = self.config.model.trim();
        format!("{base}/v1beta/models/{model}:generateContent")
    }

    /// Formats the target streaming endpoint URL for Server-Sent Events.
    pub fn build_streaming_endpoint_url(&self) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        let model = self.config.model.trim();
        format!("{base}/v1beta/models/{model}:streamGenerateContent?alt=sse")
    }

    /// Constructs a standard user prompt request payload.
    pub fn build_request(prompt: &str) -> GenerateContentRequest {
        GenerateContentRequest {
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![Part::Text { text: prompt.to_string() }],
            }],
            generation_config: None,
            system_instruction: None,
            tools: None,
        }
    }

    /// Constructs a multi-turn chat request payload with custom contents and optional tools.
    pub fn build_chat_request_with_contents(
        contents: Vec<Content>,
        tools: Option<Vec<Tool>>,
    ) -> GenerateContentRequest {
        GenerateContentRequest {
            contents,
            generation_config: None,
            system_instruction: Some(Content {
                role: None,
                parts: vec![Part::Text {
                    text: "You are Gemini Copilot in PhotoCraft, an open-source native image editor comparable to Photoshop. Provide helpful, accurate, and concise assistance on image editing techniques, tools, layers, masks, blending modes, color grading, adjustments, and keyboard shortcuts. You have multimodal vision capabilities: when images or canvas screenshots are attached, analyze their visual qualities, composition, exposure, color cast, contrast, sharpness, and defects. When the user asks you to inspect or diagnose the canvas or active layer, call inspect_canvas_image or get_document_info. When recommending single edits, suggest concrete PhotoCraft menu commands (via execute_command) or layer adjustments (via adjust_layer_property). When the user asks for multi-step recipes, complex edits, styling sequences, or macros (e.g. B&W conversion, cinematic grading, retouching sequences), propose an actionable sequential checklist via plan_workflow_actions.".to_string(),
                }],
            }),
            tools,
        }
    }

    /// Constructs a multi-turn chat request payload with system instructions.
    pub fn build_chat_request(messages: &[(&str, &str)]) -> GenerateContentRequest {
        let contents = messages
            .iter()
            .map(|(role, text)| Content {
                role: Some(match *role {
                    "user" => "user".to_string(),
                    "model" => "model".to_string(),
                    other => other.to_string(),
                }),
                parts: vec![Part::Text { text: text.to_string() }],
            })
            .collect();

        Self::build_chat_request_with_contents(contents, None)
    }

    /// Parses the raw HTTP response status and body into a GenerateContentResponse, validating errors and safety.
    pub fn parse_generate_content_response(status_code: u16, body: &str) -> Result<GenerateContentResponse, GeminiError> {
        let trimmed_body = body.trim();
        if trimmed_body.is_empty() {
            if status_code == 200 {
                return Err(GeminiError::EmptyResponse);
            }
            return Err(GeminiError::ApiError {
                code: status_code,
                status: "EMPTY_BODY".to_string(),
                message: format!("HTTP error status {status_code} with empty response body"),
            });
        }

        // Check for error envelopes in both 200 and non-200 responses
        if let Ok(envelope) = serde_json::from_str::<ApiErrorEnvelope>(trimmed_body) {
            return Err(Self::map_api_error_detail(envelope.error, status_code));
        }

        if status_code != 200 {
            // Check direct ApiErrorDetail without envelope
            if let Ok(detail) = serde_json::from_str::<ApiErrorDetail>(trimmed_body) {
                return Err(Self::map_api_error_detail(detail, status_code));
            }
            return Err(GeminiError::ApiError {
                code: status_code,
                status: "HTTP_ERROR".to_string(),
                message: trimmed_body.to_string(),
            });
        }

        // Parse successful GenerateContentResponse
        let resp: GenerateContentResponse = serde_json::from_str(trimmed_body)
            .map_err(|e| GeminiError::ParseError(format!("{e}: {trimmed_body}")))?;

        if let Some(error) = resp.error {
            return Err(Self::map_api_error_detail(error, status_code));
        }

        if let Some(ref feedback) = resp.prompt_feedback
            && let Some(ref reason) = feedback.block_reason
        {
            return Err(GeminiError::SafetyBlocked(reason.clone()));
        }

        Ok(resp)
    }

    /// Extracts the text and function calls from a valid GenerateContentResponse candidate.
    pub fn extract_model_turn(resp: &GenerateContentResponse) -> Result<ModelTurn, GeminiError> {
        let Some(first_candidate) = resp.candidates.first() else {
            return Err(GeminiError::EmptyResponse);
        };

        if let Some(finish_reason) = &first_candidate.finish_reason
            && finish_reason == "SAFETY"
        {
            return Err(GeminiError::SafetyBlocked("Candidate finished due to SAFETY".to_string()));
        }

        let Some(content) = &first_candidate.content else {
            return Err(GeminiError::EmptyResponse);
        };

        let mut text_parts = Vec::new();
        let mut function_calls = Vec::new();

        for part in &content.parts {
            match part {
                Part::Text { text } => {
                    if !text.is_empty() {
                        text_parts.push(text.clone());
                    }
                }
                Part::FunctionCall { function_call } => {
                    function_calls.push(function_call.clone());
                }
                _ => {}
            }
        }

        if text_parts.is_empty() && function_calls.is_empty() {
            return Err(GeminiError::EmptyResponse);
        }

        let text = if !text_parts.is_empty() {
            Some(text_parts.join(""))
        } else {
            None
        };

        Ok(ModelTurn {
            text,
            function_calls,
        })
    }

    /// Parses an HTTP response code and body from the Gemini API into a text response.
    pub fn parse_api_response(status_code: u16, body: &str) -> Result<String, GeminiError> {
        let resp = Self::parse_generate_content_response(status_code, body)?;
        let turn = Self::extract_model_turn(&resp)?;
        if let Some(text) = turn.text {
            Ok(text)
        } else if let Some(fc) = turn.function_calls.first() {
            Ok(format!("Function call: {}({})", fc.name, fc.args))
        } else {
            Err(GeminiError::EmptyResponse)
        }
    }

    /// Parses an HTTP response code and body into a structured ModelTurn.
    pub fn parse_api_turn(status_code: u16, body: &str) -> Result<ModelTurn, GeminiError> {
        let resp = Self::parse_generate_content_response(status_code, body)?;
        Self::extract_model_turn(&resp)
    }

    fn map_api_error_detail(detail: ApiErrorDetail, fallback_status: u16) -> GeminiError {
        let code = if detail.code != 0 { detail.code } else { fallback_status };
        let status = detail.status.to_uppercase();
        let msg = detail.message;

        if status == "RESOURCE_EXHAUSTED" || code == 429 || msg.to_lowercase().contains("quota") {
            GeminiError::QuotaExceeded(msg)
        } else if status == "PERMISSION_DENIED"
            || code == 403
            || (status == "INVALID_ARGUMENT" && (msg.contains("API key") || msg.contains("API_KEY")))
        {
            GeminiError::InvalidApiKey(msg)
        } else if status == "NOT_FOUND" || code == 404 {
            GeminiError::ModelNotFound(msg)
        } else {
            GeminiError::ApiError {
                code,
                status: detail.status,
                message: msg,
            }
        }
    }

    /// Asynchronously sends a prompt to the Gemini API and returns the generated text.
    pub async fn generate_content(&self, prompt: &str) -> Result<String, GeminiError> {
        let trimmed_key = self.config.api_key.trim();
        if trimmed_key.is_empty() {
            return Err(GeminiError::MissingApiKey);
        }

        let trimmed_prompt = prompt.trim();
        if trimmed_prompt.is_empty() {
            return Err(GeminiError::EmptyResponse);
        }

        let url = self.build_endpoint_url();
        let request = Self::build_request(trimmed_prompt);

        send_http_request(&url, trimmed_key, &request, self.config.timeout_secs).await
    }

    /// Asynchronously sends a multi-turn conversation history to the Gemini API and returns the generated reply.
    pub async fn generate_chat(&self, messages: &[(&str, &str)]) -> Result<String, GeminiError> {
        let trimmed_key = self.config.api_key.trim();
        if trimmed_key.is_empty() {
            return Err(GeminiError::MissingApiKey);
        }

        if messages.is_empty() {
            return Err(GeminiError::EmptyResponse);
        }

        let url = self.build_endpoint_url();
        let request = Self::build_chat_request(messages);

        send_http_request(&url, trimmed_key, &request, self.config.timeout_secs).await
    }

    /// Asynchronously sends a multi-turn conversation with tool support and returns the generated ModelTurn.
    pub async fn generate_chat_turn(
        &self,
        contents: &[Content],
        tools: Option<Vec<Tool>>,
    ) -> Result<ModelTurn, GeminiError> {
        let trimmed_key = self.config.api_key.trim();
        if trimmed_key.is_empty() {
            return Err(GeminiError::MissingApiKey);
        }

        if contents.is_empty() {
            return Err(GeminiError::EmptyResponse);
        }

        let url = self.build_endpoint_url();
        let request = Self::build_chat_request_with_contents(contents.to_vec(), tools);

        send_http_request_turn(&url, trimmed_key, &request, self.config.timeout_secs).await
    }

    /// Asynchronously streams a multi-turn conversation response using Server-Sent Events.
    /// Emits partial text chunks and function calls via `on_chunk`.
    /// Can be cancelled at any time by setting `stop_flag` to true.
    pub async fn stream_chat_turn<F>(
        &self,
        contents: &[Content],
        tools: Option<Vec<Tool>>,
        stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
        on_chunk: F,
    ) -> Result<(), GeminiError>
    where
        F: FnMut(String, Vec<FunctionCall>) -> Result<(), GeminiError> + Send + 'static,
    {
        let trimmed_key = self.config.api_key.trim();
        if trimmed_key.is_empty() {
            return Err(GeminiError::MissingApiKey);
        }

        if contents.is_empty() {
            return Err(GeminiError::EmptyResponse);
        }

        let url = self.build_streaming_endpoint_url();
        let request = Self::build_chat_request_with_contents(contents.to_vec(), tools);

        send_http_request_stream(&url, trimmed_key, &request, self.config.timeout_secs, stop_flag, on_chunk).await
    }

    /// Tests the configured API key and connectivity with a short prompt.
    pub async fn test_connection(&self) -> Result<String, GeminiError> {
        self.generate_content("Ping! Please reply with 'Gemini connection test successful.'").await
    }

    /// Runs a prompt on a background thread and invokes `on_complete` with the result.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn send_prompt_in_background<F>(&self, prompt: String, on_complete: F)
    where
        F: FnOnce(Result<String, GeminiError>) + Send + 'static,
    {
        let client = self.clone();
        let _ = std::thread::Builder::new()
            .name("photocraft-gemini".to_string())
            .spawn(move || {
                let result = pollster::block_on(client.generate_content(&prompt));
                on_complete(result);
            });
    }
}

/// Converts chat messages from the UI state into Gemini API Content turns.
pub fn chat_messages_to_contents(messages: &[crate::state::ChatMessage]) -> Vec<Content> {
    let mut contents = Vec::new();
    for msg in messages {
        match msg.role {
            crate::state::ChatRole::User => {
                let mut parts = Vec::new();
                if let Some(img) = &msg.image {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: img.mime_type.clone(),
                            data: img.base64_data.clone(),
                        },
                    });
                }
                if !msg.content.is_empty() || parts.is_empty() {
                    parts.push(Part::Text { text: msg.content.clone() });
                }
                contents.push(Content {
                    role: Some("user".to_string()),
                    parts,
                });
            }
            crate::state::ChatRole::Model => {
                let mut parts = Vec::new();
                if let Some(img) = &msg.image {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: img.mime_type.clone(),
                            data: img.base64_data.clone(),
                        },
                    });
                }
                if let Some(tc) = &msg.tool_call {
                    parts.push(Part::FunctionCall {
                        function_call: FunctionCall {
                            name: tc.name.clone(),
                            args: tc.args.clone(),
                        },
                    });
                }
                if !msg.content.is_empty() && (msg.tool_call.is_none() || parts.is_empty()) {
                    parts.push(Part::Text { text: msg.content.clone() });
                }
                if !parts.is_empty() {
                    contents.push(Content {
                        role: Some("model".to_string()),
                        parts,
                    });
                }
            }
            crate::state::ChatRole::Function => {
                if let Some(tc) = &msg.tool_call {
                    let resp = tc.result.clone().unwrap_or(serde_json::Value::Null);
                    contents.push(Content {
                        role: Some("function".to_string()),
                        parts: vec![Part::FunctionResponse {
                            function_response: FunctionResponse {
                                name: tc.name.clone(),
                                response: serde_json::json!({
                                    "name": tc.name.clone(),
                                    "content": resp,
                                }),
                            },
                        }],
                    });
                }
            }
            crate::state::ChatRole::System => {}
        }
    }
    contents
}

/// Standard max dimension in pixels for Gemini vision requests.
pub const DEFAULT_VISION_MAX_EDGE: u32 = 1024;
/// High-detail max dimension in pixels for detailed inspection requests.
pub const HIGH_DETAIL_VISION_MAX_EDGE: u32 = 1536;

/// Encodes raw RGBA8 image buffer to base64 JPEG data with quality 1..=100.
/// Flattens alpha over a clean white background.
pub fn encode_rgba_to_jpeg_base64(width: u32, height: u32, rgba: &[u8], quality: u8) -> Result<String, String> {
    if width == 0 || height == 0 {
        return Err("Cannot encode empty image (width or height is 0)".to_string());
    }
    let expected_len = (width as usize).saturating_mul(height as usize).saturating_mul(4);
    if rgba.len() < expected_len {
        return Err(format!("RGBA buffer length mismatch: expected at least {expected_len}, got {}", rgba.len()));
    }

    let mut rgb = Vec::with_capacity((width as usize).saturating_mul(height as usize).saturating_mul(3));
    for chunk in rgba.as_chunks::<4>().0 {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let a = chunk[3] as f32 / 255.0;
        let r_out = (r * a + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
        let g_out = (g * a + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
        let b_out = (b * a + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
        rgb.extend_from_slice(&[r_out, g_out, b_out]);
    }

    let image = photocraft_codecs::Image::from_u8(
        width,
        height,
        photocraft_codecs::ChannelLayout::Rgb,
        rgb,
    ).map_err(|e| format!("Failed to create image: {e}"))?;

    let opts = photocraft_codecs::EncodeOptions {
        jpeg_quality: quality.clamp(1, 100),
        ..Default::default()
    };
    let bytes = photocraft_codecs::encode(&image, photocraft_codecs::Format::Jpeg, &opts)
        .map_err(|e| format!("Failed to encode JPEG: {e}"))?;

    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// Encodes raw RGBA8 image buffer to base64 PNG data, preserving full alpha channel.
pub fn encode_rgba_to_png_base64(width: u32, height: u32, rgba: &[u8]) -> Result<String, String> {
    if width == 0 || height == 0 {
        return Err("Cannot encode empty image (width or height is 0)".to_string());
    }
    let expected_len = (width as usize).saturating_mul(height as usize).saturating_mul(4);
    if rgba.len() < expected_len {
        return Err(format!("RGBA buffer length mismatch: expected at least {expected_len}, got {}", rgba.len()));
    }

    let image = photocraft_codecs::Image::from_u8(
        width,
        height,
        photocraft_codecs::ChannelLayout::Rgba,
        rgba[..expected_len].to_vec(),
    ).map_err(|e| format!("Failed to create image: {e}"))?;

    let bytes = photocraft_codecs::encode(&image, photocraft_codecs::Format::Png, &photocraft_codecs::EncodeOptions::default())
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;

    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// Computes fast diagnostic visual metrics on an RGBA8 buffer (luminance, clipping, color cast).
pub fn compute_visual_metrics(width: u32, height: u32, rgba: &[u8]) -> serde_json::Value {
    let total_pixels = (width as u64).saturating_mul(height as u64);
    if total_pixels == 0 || rgba.len() < 4 {
        return serde_json::json!({
            "width": width,
            "height": height,
            "error": "Empty or invalid image data"
        });
    }

    let mut sum_r: u64 = 0;
    let mut sum_g: u64 = 0;
    let mut sum_b: u64 = 0;
    let mut sum_lum: f64 = 0.0;
    let mut shadow_clip_count: u64 = 0;
    let mut highlight_clip_count: u64 = 0;

    for chunk in rgba.as_chunks::<4>().0 {
        let r = chunk[0] as u64;
        let g = chunk[1] as u64;
        let b = chunk[2] as u64;
        sum_r = sum_r.saturating_add(r);
        sum_g = sum_g.saturating_add(g);
        sum_b = sum_b.saturating_add(b);

        let lum = (0.2126 * (chunk[0] as f64) + 0.7152 * (chunk[1] as f64) + 0.0722 * (chunk[2] as f64)) / 255.0;
        sum_lum += lum;

        if lum <= 0.02 {
            shadow_clip_count = shadow_clip_count.saturating_add(1);
        } else if lum >= 0.98 {
            highlight_clip_count = highlight_clip_count.saturating_add(1);
        }
    }

    let count_f64 = total_pixels as f64;
    let mean_lum = (sum_lum / count_f64).clamp(0.0, 1.0);
    let avg_r = (sum_r as f64 / count_f64).round() as u32;
    let avg_g = (sum_g as f64 / count_f64).round() as u32;
    let avg_b = (sum_b as f64 / count_f64).round() as u32;

    let shadow_pct = ((shadow_clip_count as f64 / count_f64) * 1000.0).round() / 10.0;
    let highlight_pct = ((highlight_clip_count as f64 / count_f64) * 1000.0).round() / 10.0;

    let tone_summary = if mean_lum < 0.35 {
        "Low-key / Underexposed (dominant shadows)"
    } else if mean_lum > 0.65 {
        "High-key / Overexposed (dominant highlights)"
    } else {
        "Balanced exposure (midtones balanced)"
    };

    let color_cast = if avg_r > avg_b.saturating_add(15) && avg_r > avg_g.saturating_add(15) {
        "Warm / Red-orange cast"
    } else if avg_b > avg_r.saturating_add(15) && avg_b > avg_g.saturating_add(15) {
        "Cool / Blue cast"
    } else if avg_g > avg_r.saturating_add(15) && avg_g > avg_b.saturating_add(15) {
        "Green tint"
    } else {
        "Neutral color balance"
    };

    serde_json::json!({
        "dimensions": {
            "width": width,
            "height": height,
            "aspect_ratio": format!("{:.2}", (width as f32) / (height as f32).max(1.0))
        },
        "exposure": {
            "mean_luminance": (mean_lum * 100.0).round() / 100.0,
            "shadow_clipping_pct": shadow_pct,
            "highlight_clipping_pct": highlight_pct,
            "evaluation": tone_summary
        },
        "color_balance": {
            "average_rgb": [avg_r, avg_g, avg_b],
            "cast": color_cast
        }
    })
}

/// Captures and downsamples a PhotoCraft document into a ChatImageAttachment and diagnostic metrics.
pub fn encode_canvas_thumbnail(
    doc: &photocraft_doc::Document,
    max_edge: u32,
    use_jpeg: bool,
) -> Result<(crate::state::ChatImageAttachment, serde_json::Value), String> {
    let max_edge = max_edge.clamp(64, 4096);
    let rgba_img = photocraft_compose::thumbnail(doc, max_edge);
    if rgba_img.width == 0 || rgba_img.height == 0 {
        return Err("Compositor produced an empty thumbnail".to_string());
    }

    let metrics = compute_visual_metrics(rgba_img.width, rgba_img.height, &rgba_img.pixels);

    let (mime_type, base64_data) = if use_jpeg {
        ("image/jpeg".to_string(), encode_rgba_to_jpeg_base64(rgba_img.width, rgba_img.height, &rgba_img.pixels, 85)?)
    } else {
        ("image/png".to_string(), encode_rgba_to_png_base64(rgba_img.width, rgba_img.height, &rgba_img.pixels)?)
    };

    let attachment = crate::state::ChatImageAttachment {
        mime_type,
        base64_data,
        width: rgba_img.width,
        height: rgba_img.height,
        label: Some(format!("Canvas: {}", doc.name)),
    };

    Ok((attachment, metrics))
}

/// Captures and downsamples an individual layer into a ChatImageAttachment and diagnostic metrics.
pub fn encode_layer_thumbnail(
    doc: &photocraft_doc::Document,
    layer_id: photocraft_doc::LayerId,
    max_edge: u32,
    use_jpeg: bool,
) -> Result<(crate::state::ChatImageAttachment, serde_json::Value), String> {
    let layer = doc.layer(layer_id).ok_or_else(|| format!("Layer ID {:?} not found in document", layer_id))?;
    let layer_name = layer.name.clone();
    let layer_doc = photocraft_engine::layer_menu_cmds::layer_document(doc, layer_id)
        .map_err(|e| format!("Failed to isolate layer document: {e}"))?;

    let (mut att, metrics) = encode_canvas_thumbnail(&layer_doc, max_edge, use_jpeg)?;
    att.label = Some(format!("Layer: {layer_name}"));
    Ok((att, metrics))
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_http_request(
    url: &str,
    api_key: &str,
    request: &GenerateContentRequest,
    timeout_secs: u64,
) -> Result<String, GeminiError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| GeminiError::NetworkError(format!("Failed to build HTTP client: {e}")))?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", api_key)
        .json(request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                GeminiError::Timeout(e.to_string())
            } else {
                GeminiError::NetworkError(e.to_string())
            }
        })?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| GeminiError::NetworkError(format!("Failed to read response body: {e}")))?;

    GeminiClient::parse_api_response(status, &body)
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_http_request_turn(
    url: &str,
    api_key: &str,
    request: &GenerateContentRequest,
    timeout_secs: u64,
) -> Result<ModelTurn, GeminiError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| GeminiError::NetworkError(format!("Failed to build HTTP client: {e}")))?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", api_key)
        .json(request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                GeminiError::Timeout(e.to_string())
            } else {
                GeminiError::NetworkError(e.to_string())
            }
        })?;

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| GeminiError::NetworkError(format!("Failed to read response body: {e}")))?;

    GeminiClient::parse_api_turn(status, &body)
}

/// Represents an event parsed from a Server-Sent Events (SSE) data stream.
#[derive(Clone, Debug, PartialEq)]
pub enum SseEvent {
    /// A chunk containing a parsed GenerateContentResponse.
    Chunk(GenerateContentResponse),
    /// Stream completion signal (`[DONE]`).
    Done,
    /// An empty, ping, or comment event.
    Empty,
}

/// Parses a single SSE line (e.g. "data: {...}").
pub fn parse_sse_line(line: &str) -> Result<SseEvent, GeminiError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return Ok(SseEvent::Empty);
    }

    let payload = if let Some(stripped) = trimmed.strip_prefix("data:") {
        stripped.trim()
    } else {
        trimmed
    };

    if payload == "[DONE]" {
        return Ok(SseEvent::Done);
    }

    if payload.is_empty() {
        return Ok(SseEvent::Empty);
    }

    let resp = GeminiClient::parse_generate_content_response(200, payload)?;
    Ok(SseEvent::Chunk(resp))
}

/// Parses a workflow plan from tool arguments emitted by Gemini.
pub fn parse_workflow_plan(args: &serde_json::Value) -> Result<crate::state::WorkflowPlan, String> {
    let title = args
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Macro Workflow Plan")
        .to_string();

    let description = args
        .get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();

    let steps_val = args
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "Workflow plan must contain a 'steps' array".to_string())?;

    if steps_val.is_empty() {
        return Err("Workflow plan steps array cannot be empty".to_string());
    }

    let mut steps = Vec::new();
    for (i, item) in steps_val.iter().enumerate() {
        let step_id = item
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("step_{}", i.saturating_add(1)));

        let desc = item
            .get("description")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("Step {i} missing description"))?
            .to_string();

        let tool = item
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("execute_command")
            .to_string();

        let command_id = item
            .get("command_id")
            .and_then(serde_json::Value::as_str)
            .map(|s| s.to_string());

        let params = item.get("params").cloned().unwrap_or(serde_json::json!({}));

        steps.push(crate::state::WorkflowStep {
            id: step_id,
            description: desc,
            tool,
            command_id,
            params,
            status: crate::state::WorkflowStepStatus::Pending,
            error: None,
        });
    }

    Ok(crate::state::WorkflowPlan::new(title, description, steps))
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_http_request_stream<F>(
    url: &str,
    api_key: &str,
    request: &GenerateContentRequest,
    timeout_secs: u64,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    mut on_chunk: F,
) -> Result<(), GeminiError>
where
    F: FnMut(String, Vec<FunctionCall>) -> Result<(), GeminiError> + Send + 'static,
{
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| GeminiError::NetworkError(format!("Failed to build HTTP client: {e}")))?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", api_key)
        .json(request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                GeminiError::Timeout(e.to_string())
            } else {
                GeminiError::NetworkError(e.to_string())
            }
        })?;

    let status = response.status().as_u16();
    if status != 200 {
        let body = response.text().await.unwrap_or_default();
        return Err(GeminiClient::parse_api_response(status, &body).err().unwrap_or(GeminiError::ApiError {
            code: status,
            status: "STREAM_ERROR".to_string(),
            message: body,
        }));
    }

    let mut response = response;
    let mut line_buffer = String::new();

    while let Some(chunk_bytes) = response.chunk().await.map_err(|e| GeminiError::NetworkError(e.to_string()))? {
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let chunk_str = String::from_utf8_lossy(&chunk_bytes);
        line_buffer.push_str(&chunk_str);

        while let Some(newline_pos) = line_buffer.find('\n') {
            let line: String = line_buffer.drain(..=newline_pos).collect();
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            match parse_sse_line(trimmed_line) {
                Ok(SseEvent::Chunk(resp)) => {
                    if let Ok(turn) = GeminiClient::extract_model_turn(&resp) {
                        let text = turn.text.unwrap_or_default();
                        let fcs = turn.function_calls;
                        if !text.is_empty() || !fcs.is_empty() {
                            on_chunk(text, fcs)?;
                        }
                    }
                }
                Ok(SseEvent::Done) => {
                    return Ok(());
                }
                Ok(SseEvent::Empty) => {}
                Err(e) => {
                    log::warn!("SSE parse error: {e}");
                }
            }
        }
    }

    let remaining = line_buffer.trim();
    if !remaining.is_empty()
        && let Ok(SseEvent::Chunk(resp)) = parse_sse_line(remaining)
        && let Ok(turn) = GeminiClient::extract_model_turn(&resp)
    {
        let text = turn.text.unwrap_or_default();
        let fcs = turn.function_calls;
        if !text.is_empty() || !fcs.is_empty() {
            let _ = on_chunk(text, fcs);
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn send_http_request(
    _url: &str,
    _api_key: &str,
    _request: &GenerateContentRequest,
    _timeout_secs: u64,
) -> Result<String, GeminiError> {
    Err(GeminiError::UnsupportedPlatform("WebAssembly HTTP networking is not supported yet".to_string()))
}

#[cfg(target_arch = "wasm32")]
async fn send_http_request_turn(
    _url: &str,
    _api_key: &str,
    _request: &GenerateContentRequest,
    _timeout_secs: u64,
) -> Result<ModelTurn, GeminiError> {
    Err(GeminiError::UnsupportedPlatform("WebAssembly HTTP networking is not supported yet".to_string()))
}

#[cfg(target_arch = "wasm32")]
async fn send_http_request_stream<F>(
    _url: &str,
    _api_key: &str,
    _request: &GenerateContentRequest,
    _timeout_secs: u64,
    _stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    _on_chunk: F,
) -> Result<(), GeminiError>
where
    F: FnMut(String, Vec<FunctionCall>) -> Result<(), GeminiError> + Send + 'static,
{
    Err(GeminiError::UnsupportedPlatform("WebAssembly HTTP networking is not supported yet".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_url_construction() {
        let client = GeminiClient::new("test-key", "gemini-2.5-flash");
        assert_eq!(
            client.build_endpoint_url(),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
        );

        let custom_client = GeminiClient::with_config(GeminiConfig {
            api_key: "key123".into(),
            model: "gemini-1.5-pro".into(),
            base_url: "https://my-proxy.example.com/api/".into(),
            timeout_secs: 15,
        });
        assert_eq!(
            custom_client.build_endpoint_url(),
            "https://my-proxy.example.com/api/v1beta/models/gemini-1.5-pro:generateContent"
        );
    }

    #[test]
    fn request_serialization() {
        let req = GeminiClient::build_request("Hello, PhotoCraft!");
        let json_str = serde_json::to_string(&req).expect("serialize request");
        assert!(json_str.contains("Hello, PhotoCraft!"));
        assert!(json_str.contains("\"user\""));
    }

    #[test]
    fn parse_successful_response() {
        let raw_json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "Hello! I am your Gemini assistant."
                            }
                        ],
                        "role": "model"
                    },
                    "finishReason": "STOP"
                }
            ],
            "usageMetadata": {
                "promptTokenCount": 5,
                "candidatesTokenCount": 10,
                "totalTokenCount": 15
            }
        }"#;

        let parsed = GeminiClient::parse_api_response(200, raw_json).expect("valid response");
        assert_eq!(parsed, "Hello! I am your Gemini assistant.");
    }

    #[test]
    fn parse_multi_part_response() {
        let raw_json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            { "text": "Part 1. " },
                            { "text": "Part 2." }
                        ]
                    }
                }
            ]
        }"#;

        let parsed = GeminiClient::parse_api_response(200, raw_json).expect("valid multi-part");
        assert_eq!(parsed, "Part 1. Part 2.");
    }

    #[test]
    fn parse_invalid_api_key_error() {
        let error_json = r#"{
            "error": {
                "code": 400,
                "message": "API key not valid. Please pass a valid API key.",
                "status": "INVALID_ARGUMENT"
            }
        }"#;

        let err = GeminiClient::parse_api_response(400, error_json).expect_err("should fail");
        match err {
            GeminiError::InvalidApiKey(msg) => {
                assert!(msg.contains("API key not valid"));
            }
            other => panic!("expected InvalidApiKey, got {other:?}"),
        }
    }

    #[test]
    fn parse_permission_denied_error() {
        let error_json = r#"{
            "error": {
                "code": 403,
                "message": "The caller does not have permission",
                "status": "PERMISSION_DENIED"
            }
        }"#;

        let err = GeminiClient::parse_api_response(403, error_json).expect_err("should fail");
        assert!(matches!(err, GeminiError::InvalidApiKey(_)));
    }

    #[test]
    fn parse_quota_exceeded_error() {
        let error_json = r#"{
            "error": {
                "code": 429,
                "message": "Resource has been exhausted (e.g. check quota).",
                "status": "RESOURCE_EXHAUSTED"
            }
        }"#;

        let err = GeminiClient::parse_api_response(429, error_json).expect_err("should fail");
        match err {
            GeminiError::QuotaExceeded(msg) => {
                assert!(msg.contains("exhausted"));
            }
            other => panic!("expected QuotaExceeded, got {other:?}"),
        }
    }

    #[test]
    fn parse_model_not_found_error() {
        let error_json = r#"{
            "error": {
                "code": 404,
                "message": "models/non-existent-model is not found",
                "status": "NOT_FOUND"
            }
        }"#;

        let err = GeminiClient::parse_api_response(404, error_json).expect_err("should fail");
        assert!(matches!(err, GeminiError::ModelNotFound(_)));
    }

    #[test]
    fn parse_safety_blocked_response() {
        let blocked_json = r#"{
            "promptFeedback": {
                "blockReason": "SAFETY"
            }
        }"#;

        let err = GeminiClient::parse_api_response(200, blocked_json).expect_err("should fail");
        match err {
            GeminiError::SafetyBlocked(reason) => assert_eq!(reason, "SAFETY"),
            other => panic!("expected SafetyBlocked, got {other:?}"),
        }
    }

    #[test]
    fn parse_empty_response() {
        let empty_json = r#"{"candidates": []}"#;
        let err = GeminiClient::parse_api_response(200, empty_json).expect_err("should fail");
        assert_eq!(err, GeminiError::EmptyResponse);

        let err_blank = GeminiClient::parse_api_response(200, "   ").expect_err("should fail");
        assert_eq!(err_blank, GeminiError::EmptyResponse);
    }

    #[test]
    fn parse_malformed_json() {
        let err = GeminiClient::parse_api_response(200, "{ not valid json }").expect_err("should fail");
        assert!(matches!(err, GeminiError::ParseError(_)));
    }

    #[test]
    fn missing_api_key_check() {
        let client = GeminiClient::new("", "gemini-2.5-flash");
        let result = pollster::block_on(client.generate_content("test"));
        assert_eq!(result, Err(GeminiError::MissingApiKey));

        let client_spaces = GeminiClient::new("   ", "gemini-2.5-flash");
        let result_spaces = pollster::block_on(client_spaces.generate_content("test"));
        assert_eq!(result_spaces, Err(GeminiError::MissingApiKey));
    }

    #[test]
    fn empty_prompt_check() {
        let client = GeminiClient::new("valid-key", "gemini-2.5-flash");
        let result = pollster::block_on(client.generate_content("   "));
        assert_eq!(result, Err(GeminiError::EmptyResponse));
    }

    #[test]
    fn chat_request_serialization() {
        let messages = [
            ("user", "How do I create a layer mask?"),
            ("model", "Click the Add Layer Mask icon at the bottom of the Layers panel."),
            ("user", "How do I invert it?"),
        ];
        let req = GeminiClient::build_chat_request(&messages);
        assert_eq!(req.contents.len(), 3);
        assert_eq!(req.contents[0].role.as_deref(), Some("user"));
        assert_eq!(req.contents[1].role.as_deref(), Some("model"));
        assert_eq!(req.contents[2].role.as_deref(), Some("user"));
        assert!(req.system_instruction.is_some());
    }

    #[test]
    fn tools_declaration_serialization() {
        let tools = photocraft_tools();
        assert_eq!(tools.len(), 1);
        let decls = &tools[0].function_declarations;
        assert_eq!(decls.len(), 5);
        assert_eq!(decls[0].name, "get_document_info");
        assert_eq!(decls[1].name, "execute_command");
        assert_eq!(decls[2].name, "adjust_layer_property");
        assert_eq!(decls[3].name, "inspect_canvas_image");
        assert_eq!(decls[4].name, "plan_workflow_actions");

        let json = serde_json::to_string(&tools).expect("serialize tools");
        assert!(json.contains("functionDeclarations"));
        assert!(json.contains("get_document_info"));
        assert!(json.contains("execute_command"));
        assert!(json.contains("adjust_layer_property"));
        assert!(json.contains("inspect_canvas_image"));
        assert!(json.contains("plan_workflow_actions"));
    }

    #[test]
    fn function_call_and_response_round_trip() {
        let fc_part = Part::FunctionCall {
            function_call: FunctionCall {
                name: "execute_command".into(),
                args: serde_json::json!({
                    "command_id": "image.adjustments.invert"
                }),
            },
        };
        let fc_json = serde_json::to_string(&fc_part).expect("serialize function call");
        assert!(fc_json.contains("\"functionCall\""));
        assert!(fc_json.contains("\"image.adjustments.invert\""));

        let fc_de: Part = serde_json::from_str(&fc_json).expect("deserialize function call");
        match fc_de {
            Part::FunctionCall { function_call } => {
                assert_eq!(function_call.name, "execute_command");
                assert_eq!(
                    function_call.args.get("command_id").and_then(|v| v.as_str()),
                    Some("image.adjustments.invert")
                );
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }

        let fr_part = Part::FunctionResponse {
            function_response: FunctionResponse {
                name: "get_document_info".into(),
                response: serde_json::json!({
                    "width": 1920,
                    "height": 1080
                }),
            },
        };
        let fr_json = serde_json::to_string(&fr_part).expect("serialize function response");
        assert!(fr_json.contains("\"functionResponse\""));
        let fr_de: Part = serde_json::from_str(&fr_json).expect("deserialize function response");
        match fr_de {
            Part::FunctionResponse { function_response } => {
                assert_eq!(function_response.name, "get_document_info");
                assert_eq!(
                    function_response.response.get("width").and_then(|v| v.as_u64()),
                    Some(1920)
                );
            }
            other => panic!("expected FunctionResponse, got {other:?}"),
        }
    }

    #[test]
    fn parse_tool_call_response() {
        let tool_call_json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "functionCall": {
                                    "name": "get_document_info",
                                    "args": {}
                                }
                            }
                        ],
                        "role": "model"
                    },
                    "finishReason": "STOP"
                }
            ]
        }"#;

        let turn = GeminiClient::parse_api_turn(200, tool_call_json).expect("parse tool call");
        assert!(turn.is_function_call());
        assert_eq!(turn.function_calls.len(), 1);
        let fc = turn.first_function_call().expect("first call");
        assert_eq!(fc.name, "get_document_info");
    }

    #[test]
    fn parse_mixed_text_and_function_call() {
        let mixed_json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "I will inspect your canvas now."
                            },
                            {
                                "functionCall": {
                                    "name": "get_document_info",
                                    "args": {}
                                }
                            }
                        ],
                        "role": "model"
                    },
                    "finishReason": "STOP"
                }
            ]
        }"#;

        let turn = GeminiClient::parse_api_turn(200, mixed_json).expect("parse mixed");
        assert_eq!(turn.text.as_deref(), Some("I will inspect your canvas now."));
        assert!(turn.is_function_call());
        assert_eq!(turn.first_function_call().expect("fc").name, "get_document_info");
    }

    #[test]
    fn chat_messages_to_contents_conversion() {
        use crate::state::{ChatMessage, ToolCallAction, ToolActionStatus};

        let messages = vec![
            ChatMessage::user("Invert the layer"),
            ChatMessage::tool_call(
                "Suggested action",
                ToolCallAction {
                    name: "execute_command".into(),
                    args: serde_json::json!({ "command_id": "image.adjustments.invert" }),
                    status: ToolActionStatus::Completed,
                    result: Some(serde_json::json!({ "status": "success" })),
                    folded: false,
                },
            ),
            ChatMessage::tool_response(
                "execute_command",
                serde_json::json!({ "status": "success" }),
            ),
            ChatMessage::model("Layer colors have been inverted!"),
        ];

        let contents = chat_messages_to_contents(&messages);
        assert_eq!(contents.len(), 4);
        assert_eq!(contents[0].role.as_deref(), Some("user"));
        assert_eq!(contents[1].role.as_deref(), Some("model"));
        assert_eq!(contents[2].role.as_deref(), Some("function"));
        assert_eq!(contents[3].role.as_deref(), Some("model"));
    }

    #[test]
    fn multimodal_inline_data_serialization() {
        let inline = InlineData::jpeg("QUJDREVGR0g=");
        let part = Part::InlineData { inline_data: inline };
        let json_str = serde_json::to_string(&part).expect("serialize inlineData");
        assert!(json_str.contains("\"inlineData\""));
        assert!(json_str.contains("\"mimeType\":\"image/jpeg\""));
        assert!(json_str.contains("\"data\":\"QUJDREVGR0g=\""));

        let deserialized: Part = serde_json::from_str(&json_str).expect("deserialize inlineData");
        assert_eq!(deserialized, part);
    }

    #[test]
    fn chat_messages_with_image_conversion() {
        use crate::state::{ChatImageAttachment, ChatMessage};

        let messages = vec![
            ChatMessage::user_with_image(
                "Analyze this exposure",
                ChatImageAttachment {
                    mime_type: "image/jpeg".to_string(),
                    base64_data: "QUJDREVGR0g=".to_string(),
                    width: 640,
                    height: 480,
                    label: Some("Canvas: Test".to_string()),
                },
            ),
        ];

        let contents = chat_messages_to_contents(&messages);
        assert_eq!(contents.len(), 1);
        let parts = &contents[0].parts;
        assert_eq!(parts.len(), 2);
        match &parts[0] {
            Part::InlineData { inline_data } => {
                assert_eq!(inline_data.mime_type, "image/jpeg");
                assert_eq!(inline_data.data, "QUJDREVGR0g=");
            }
            _ => panic!("Expected InlineData as first part"),
        }
        match &parts[1] {
            Part::Text { text } => assert_eq!(text, "Analyze this exposure"),
            _ => panic!("Expected Text as second part"),
        }
    }

    #[test]
    fn encode_rgba_to_jpeg_and_png_roundtrip() {
        // Create 32x32 RGBA test pattern
        let (w, h) = (32u32, 32u32);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let r = ((x * 255) / w) as u8;
                let g = ((y * 255) / h) as u8;
                let b = 128u8;
                let a = 255u8;
                rgba.extend_from_slice(&[r, g, b, a]);
            }
        }

        let jpeg_b64 = encode_rgba_to_jpeg_base64(w, h, &rgba, 85).expect("encode jpeg");
        assert!(!jpeg_b64.is_empty());
        use base64::Engine;
        let jpeg_bytes = base64::engine::general_purpose::STANDARD.decode(&jpeg_b64).expect("decode base64 jpeg");
        let decoded_jpeg = photocraft_codecs::decode(&jpeg_bytes).expect("decode codecs jpeg");
        assert_eq!(decoded_jpeg.width(), w);
        assert_eq!(decoded_jpeg.height(), h);

        let png_b64 = encode_rgba_to_png_base64(w, h, &rgba).expect("encode png");
        assert!(!png_b64.is_empty());
        let png_bytes = base64::engine::general_purpose::STANDARD.decode(&png_b64).expect("decode base64 png");
        let decoded_png = photocraft_codecs::decode(&png_bytes).expect("decode codecs png");
        assert_eq!(decoded_png.width(), w);
        assert_eq!(decoded_png.height(), h);
    }

    #[test]
    fn visual_metrics_computation() {
        let (w, h) = (10u32, 10u32);
        // All black image
        let black = vec![0u8; (w * h * 4) as usize];
        let black_metrics = compute_visual_metrics(w, h, &black);
        assert_eq!(black_metrics["exposure"]["mean_luminance"], 0.0);
        assert_eq!(black_metrics["exposure"]["shadow_clipping_pct"], 100.0);

        // All white image
        let white = vec![255u8; (w * h * 4) as usize];
        let white_metrics = compute_visual_metrics(w, h, &white);
        assert_eq!(white_metrics["exposure"]["mean_luminance"], 1.0);
        assert_eq!(white_metrics["exposure"]["highlight_clipping_pct"], 100.0);

        // Empty image should not panic
        let empty_metrics = compute_visual_metrics(0, 0, &[]);
        assert!(empty_metrics.get("error").is_some());
    }

    #[test]
    fn photocraft_tools_has_inspect_canvas_image() {
        let tools = photocraft_tools();
        let decls = &tools[0].function_declarations;
        assert!(decls.iter().any(|d| d.name == "inspect_canvas_image"));
    }

    #[test]
    fn photocraft_tools_has_plan_workflow_actions() {
        let tools = photocraft_tools();
        let decls = &tools[0].function_declarations;
        assert!(decls.iter().any(|d| d.name == "plan_workflow_actions"));
    }

    #[test]
    fn streaming_endpoint_url_construction() {
        let client = GeminiClient::new("key", "gemini-2.5-flash");
        assert_eq!(
            client.build_streaming_endpoint_url(),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse"
        );
    }

    #[test]
    fn parse_sse_line_events() {
        // SSE Done
        let done_res = parse_sse_line("data: [DONE]").expect("parse done");
        assert_eq!(done_res, SseEvent::Done);

        // SSE Empty or comment
        assert_eq!(parse_sse_line("").unwrap(), SseEvent::Empty);
        assert_eq!(parse_sse_line(": ping").unwrap(), SseEvent::Empty);
        assert_eq!(parse_sse_line("   \n").unwrap(), SseEvent::Empty);

        // SSE Data chunk
        let raw_chunk = r#"data: {"candidates":[{"content":{"parts":[{"text":"Streaming chunk test"}]}}]}"#;
        let chunk_res = parse_sse_line(raw_chunk).expect("parse chunk");
        match chunk_res {
            SseEvent::Chunk(resp) => {
                let turn = GeminiClient::extract_model_turn(&resp).expect("extract turn");
                assert_eq!(turn.text.as_deref(), Some("Streaming chunk test"));
            }
            _ => panic!("Expected SseEvent::Chunk"),
        }
    }

    #[test]
    fn parse_workflow_plan_success() {
        let args = serde_json::json!({
            "title": "B&W High Contrast",
            "description": "Convert canvas to dramatic B&W",
            "steps": [
                {
                    "id": "step_1",
                    "description": "Invert canvas colors",
                    "tool": "execute_command",
                    "command_id": "image.adjustments.invert"
                },
                {
                    "id": "step_2",
                    "description": "Adjust opacity to 80%",
                    "tool": "adjust_layer_property",
                    "params": { "opacity": 0.8 }
                }
            ]
        });

        let plan = parse_workflow_plan(&args).expect("valid workflow plan");
        assert_eq!(plan.title, "B&W High Contrast");
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].tool, "execute_command");
        assert_eq!(plan.steps[0].command_id.as_deref(), Some("image.adjustments.invert"));
        assert_eq!(plan.steps[1].tool, "adjust_layer_property");
        assert_eq!(plan.completed_count(), 0);
        assert_eq!(plan.next_pending_index(), Some(0));
    }

    #[test]
    fn parse_workflow_plan_missing_fields_fails_gracefully() {
        // Missing steps array
        let bad_args = serde_json::json!({
            "title": "Bad Plan"
        });
        assert!(parse_workflow_plan(&bad_args).is_err());

        // Empty steps array
        let empty_args = serde_json::json!({
            "title": "Empty Plan",
            "steps": []
        });
        assert!(parse_workflow_plan(&empty_args).is_err());
    }
}
