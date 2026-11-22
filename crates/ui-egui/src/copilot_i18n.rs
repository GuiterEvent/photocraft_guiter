//! Taiwan Traditional Chinese (`zh-hant`) fallback translations for fork-added AI Assistant
//! sidebar and Preferences › Integrations UI strings.
//!
//! Official PhotoCraft strings come from `storytold/craft-i18n`. [`crate::i18n::tr`] consults
//! [`zh_hant`] only when the active language is `zh-hant` and the embedded official catalog has no
//! entry for the key, keeping upstream merges minimal and leaving English and all other languages
//! untouched.

/// Static `(english_key, zh_hant_translation)` table for fork-added AI Assistant UI strings.
const ZH_HANT: &[(&str, &str)] = &[
    // Window menu & sidebar chrome
    ("AI Assistant", "AI 助理"),
    ("Resize AI Assistant sidebar", "調整 AI 助理側邊欄寬度"),
    ("Switch chat session", "切換對話紀錄"),
    ("New Chat", "新增對話"),
    ("+ New Chat", "+ 新增對話"),
    ("Undock to floating window", "解除停靠為浮動視窗"),
    ("Dock to right sidebar", "停靠至右側邊欄"),
    ("Clear chat", "清除對話"),
    ("Test connection", "測試連線"),
    ("Chat History", "對話紀錄"),
    ("Close drawer", "關閉抽屜"),
    ("Export MD", "匯出 MD"),
    ("Export JSON", "匯出 JSON"),
    ("Session name", "對話名稱"),
    ("Save name", "儲存名稱"),
    ("Delete session", "刪除對話"),
    ("Rename session", "重新命名對話"),
    // Engine selector & provider labels
    ("Engine", "引擎"),
    ("AI Engine", "AI 引擎"),
    ("Antigravity CLI (Google account)", "Antigravity CLI（Google 帳號）"),
    ("Gemini CLI (Google account)", "Gemini CLI（Google 帳號）"),
    ("Claude CLI (Claude subscription)", "Claude CLI（Claude 訂閱）"),
    ("Codex CLI (ChatGPT subscription)", "Codex CLI（ChatGPT 訂閱）"),
    ("Gemini API (API key)", "Gemini API（API 金鑰）"),
    ("Google account sign-in via Antigravity, no API credits", "透過 Antigravity 登入 Google 帳號，無需 API 點數"),
    ("Google account sign-in, no API credits", "登入 Google 帳號，無需 API 點數"),
    ("Claude subscription, no API credits", "使用 Claude 訂閱方案，無需 API 點數"),
    ("ChatGPT subscription, no API credits", "使用 ChatGPT 訂閱方案，無需 API 點數"),
    ("Google Gemini API key", "Google Gemini API 金鑰"),
    // CLI status strip & recovery guidance
    ("CLI · Ready", "CLI · 已就緒"),
    ("CLI · Desktop only", "CLI · 僅限桌面版"),
    ("CLI · Not found", "CLI · 找不到"),
    ("{exe} detected", "已偵測到 {exe}"),
    ("{exe} not found", "找不到 {exe}"),
    ("Local CLI agents require the desktop app.", "本機 CLI 代理僅支援桌面版應用程式。"),
    ("Re-detect", "重新偵測"),
    ("Re-scan PATH for local CLI binary", "重新掃描 PATH 尋找本機 CLI 執行檔"),
    ("Install:", "安裝方式："),
    ("Copy command", "複製指令"),
    ("1. Install the CLI in your terminal:", "1. 請在終端機中安裝此 CLI："),
    (
        "2. Run the CLI once to sign in with your account (no API key needed).",
        "2. 執行一次 CLI 並使用您的帳號登入（無需 API 金鑰）。",
    ),
    ("3. Click Re-detect below to refresh.", "3. 點擊下方的「重新偵測」以重新整理。"),
    ("Switch to Gemini API", "切換至 Gemini API"),
    ("1. Open your system terminal.", "1. 開啟系統終端機。"),
    (
        "2. Run `{cmd}` once and complete sign-in with your {account} (no API key needed).",
        "2. 執行一次 `{cmd}` 並使用您的 {account} 完成登入（無需 API 金鑰）。",
    ),
    ("3. Return to PhotoCraft and send your message again.", "3. 返回 PhotoCraft 並重新送出訊息。"),
    ("Open Preferences…", "開啟偏好設定…"),
    ("Connection test successful! {provider} is ready.", "連線測試成功！{provider} 已就緒。"),
    ("Testing connection to {provider}…", "正在測試 {provider} 連線…"),
    // PhotoCraft MCP control channel strip
    ("MCP Control · Port {port}", "MCP 控制通道 · 連接埠 {port}"),
    ("MCP Control · Connected", "MCP 控制通道 · 已連線"),
    ("PhotoCraft MCP control is off", "PhotoCraft MCP 控制通道已關閉"),
    ("MCP control disabled in ArtCraft Launcher", "ArtCraft Launcher 已停用 MCP 控制通道"),
    ("PhotoCraft MCP control error", "PhotoCraft MCP 控制通道錯誤"),
    ("Live MCP control requires the desktop app.", "即時 MCP 控制通道僅支援桌面版應用程式。"),
    ("photocraft-cli missing", "找不到 photocraft-cli"),
    (
        "Enable the local control channel so the CLI can inspect and edit this window via MCP.",
        "啟用本機控制通道後，CLI 即可透過 MCP 檢視並編輯目前視窗。",
    ),
    ("Enable PhotoCraft MCP Control", "啟用 PhotoCraft MCP 控制通道"),
    (
        "Enable PhotoCraft in ArtCraft Launcher › Settings › AI agents to allow live MCP control.",
        "請至「ArtCraft Launcher › 設定 › AI 代理」啟用 PhotoCraft，以允許即時 MCP 控制。",
    ),
    (
        "PhotoCraft's live control channel is unavailable in this session.",
        "目前工作階段無法使用 PhotoCraft 即時控制通道。",
    ),
    ("PhotoCraft MCP control channel enabled.", "已啟用 PhotoCraft MCP 控制通道。"),
    (
        "`photocraft-cli` was not found beside PhotoCraft or on PATH. Build/install `photocraft-cli` and click Re-detect.",
        "在 PhotoCraft 同目錄或 PATH 中找不到 `photocraft-cli`。請建置／安裝 `photocraft-cli` 後點擊「重新偵測」。",
    ),
    ("Allow CLI to control PhotoCraft (MCP)", "允許 CLI 透過 MCP 控制 PhotoCraft"),
    ("Waiting for PhotoCraft MCP control channel to be enabled.", "等待啟用 PhotoCraft MCP 控制通道。"),
    ("Waiting for `photocraft-cli` to be detected.", "等待偵測 `photocraft-cli`。"),
    ("Auto-approve headless tool calls", "自動核准背景工具呼叫"),
    (
        "Non-interactive CLIs cannot ask y/n in a terminal. Enable to let the AI invoke PhotoCraft MCP tools directly; leave off for default CLI permissions.",
        "非互動式 CLI 無法在終端機中詢問 y/n。啟用後 AI 可直接呼叫 PhotoCraft MCP 工具；保持關閉則沿用 CLI 預設權限。",
    ),
    // Gemini API key banner
    ("API key is not configured", "尚未設定 API 金鑰"),
    (
        "Configure your Google Gemini API key to enable instant AI assistance for PhotoCraft.",
        "請先設定您的 Google Gemini API 金鑰，以啟用 PhotoCraft AI 助理。",
    ),
    ("Configure in Preferences…", "前往偏好設定配置…"),
    // Composer, vision presets & empty state
    ("Attach current canvas screenshot (Ctrl+Shift+C)", "附加目前畫布截圖（Ctrl+Shift+C）"),
    ("No active document to capture", "目前沒有開啟中的文件可供擷取"),
    ("Vision Presets", "視覺分析範本"),
    ("Exposure & Histogram", "曝光與直方圖"),
    (
        "Analyze the exposure, highlights/shadows clipping, and dynamic range of this canvas image, and suggest adjustments.",
        "請分析目前畫布影像的曝光、亮部與暗部裁切及動態範圍，並提供調整建議。",
    ),
    ("Color Grading Style", "調色風格建議"),
    (
        "Evaluate the color temperature, cast, and tones in this canvas image, and suggest 2-3 suitable color grading styles with concrete PhotoCraft steps.",
        "請評估目前畫布影像的色溫、色偏與色調，並推薦 2–3 種適合的調色風格與具體的 PhotoCraft 操作步驟。",
    ),
    ("Retouching Defects", "修圖瑕疵診斷"),
    (
        "Diagnose any visual defects, unwanted noise, skin blemishes, or compositional issues in this image, and recommend retouching tools to fix them.",
        "請診斷這張影像中的視覺瑕疵、雜訊、肌膚斑點或構圖問題，並推薦適合的修圖工具來修正。",
    ),
    ("Composition & Cropping", "構圖與裁切"),
    (
        "Analyze the composition, rule of thirds, subject balance, and suggest optimal cropping or framing.",
        "請分析這張影像的構圖、三分法與主體平衡，並建議最佳的裁切或取景方式。",
    ),
    ("Please inspect and analyze this canvas image.", "請檢視並分析這張畫布影像。"),
    ("Remove attachment", "移除附件"),
    (
        "Ask AI Assistant… (Enter to send, Shift+Enter for new line)",
        "向 AI 助理提問…（Enter 送出，Shift+Enter 換行）",
    ),
    ("Ask AI Assistant", "向 AI 助理提問"),
    ("Stop Generating", "停止生成"),
    ("Send prompt", "送出提示"),
    ("How can I assist you?", "有什麼我可以協助您的？"),
    (
        "Ask anything about image editing, visual diagnosis, color grading, or shortcuts.",
        "歡迎詢問任何影像編輯、視覺診斷、調色或快捷鍵相關問題。",
    ),
    ("Analyze exposure & histogram of this image", "分析這張影像的曝光與直方圖"),
    ("Suggest color grading styles and filter steps", "推薦適合的調色風格與濾鏡步驟"),
    ("Diagnose retouching defects & noise in this photo", "診斷這張照片的修圖瑕疵與雜訊"),
    ("How do I create and refine a layer mask?", "如何建立與精細調整圖層遮色片？"),
    (
        "Explain the difference between Multiply and Screen blend modes",
        "說明「色彩增值」與「濾色」混合模式的差異",
    ),
    ("What are the most essential keyboard shortcuts?", "有哪些最實用的鍵盤快捷鍵？"),
    ("Failed to decode image preview", "無法解碼影像預覽"),
    ("Canvas Screenshot Preview", "畫布截圖預覽"),
    // Messages, tool cards & workflow plans
    ("You", "您"),
    ("Streaming…", "串流輸出中…"),
    ("Copy text", "複製文字"),
    ("{provider} is thinking…", "{provider} 正在思考中…"),
    ("CLI Tool", "CLI 工具"),
    ("Show details", "顯示詳細資訊"),
    ("Hide details", "隱藏詳細資訊"),
    ("Running…", "執行中…"),
    ("Completed", "已完成"),
    ("Cancelled", "已取消"),
    ("Failed", "失敗"),
    ("Skip", "略過"),
    ("Run", "執行"),
    ("Parameters", "參數"),
    ("Suggested Workflow Plan", "建議工作流程計畫"),
    ("Executing workflow steps automatically…", "正在自動執行工作流程步驟…"),
    ("Execute All Steps", "執行全部步驟"),
    ("Reading document info…", "正在讀取文件資訊…"),
    ("Suggested Action", "建議動作"),
    ("Adjust Layer Properties", "調整圖層屬性"),
    ("Document Info", "文件資訊"),
    ("Inspect Canvas Image", "檢視畫布影像"),
    ("Inspecting active layer image…", "正在檢視目前圖層影像…"),
    ("Inspecting canvas image…", "正在檢視畫布影像…"),
    ("Pending Confirmation", "待確認"),
    ("Executing…", "執行中…"),
    ("Inspecting document info…", "正在檢視文件資訊…"),
    ("Active Layer", "目前圖層"),
    ("Canvas Composite", "畫布合成影像"),
    ("Shadow Clipping", "暗部裁切"),
    ("Highlight Clipping", "亮部裁切"),
    ("Apply / Execute", "套用／執行"),
    ("Dismiss / Cancel", "略過／取消"),
    // Preferences > Integrations › AI Assistant
    ("Default AI Provider", "預設 AI 供應商"),
    ("AI Provider", "AI 供應商"),
    (
        "Local CLI providers (Antigravity, Gemini, Claude, Codex) sign in with your own account and require no API credits.",
        "本機 CLI 供應商（Antigravity、Gemini、Claude、Codex）使用您自己的帳號登入，無需 API 點數。",
    ),
    ("CLI Executable Paths (Optional)", "CLI 執行檔路徑（選填）"),
    (
        "Leave blank to auto-detect from PATH and standard install folders. You may enter either an executable file or its folder.",
        "留空即自動從 PATH 與常見安裝目錄偵測。您可輸入完整執行檔路徑或其所在資料夾。",
    ),
    ("Auto-detect (leave blank)", "自動偵測（留空）"),
    ("Antigravity CLI (agy)", "Antigravity CLI（agy）"),
    ("Gemini CLI (gemini)", "Gemini CLI（gemini）"),
    ("Claude CLI (claude)", "Claude CLI（claude）"),
    ("Codex CLI (codex)", "Codex CLI（codex）"),
    ("PhotoCraft CLI (photocraft-cli)", "PhotoCraft CLI（photocraft-cli）"),
    ("Antigravity CLI Path (agy)", "Antigravity CLI 路徑（agy）"),
    ("Gemini CLI Path (gemini)", "Gemini CLI 路徑（gemini）"),
    ("Claude CLI Path (claude)", "Claude CLI 路徑（claude）"),
    ("Codex CLI Path (codex)", "Codex CLI 路徑（codex）"),
    ("PhotoCraft CLI Path (photocraft-cli)", "PhotoCraft CLI 路徑（photocraft-cli）"),
    ("Check CLI Environment", "檢查 CLI 環境"),
    ("Checking CLI Environment…", "正在檢查 CLI 環境…"),
    ("custom", "自訂"),
    ("auto", "自動"),
    ("Found", "已找到"),
    ("Custom path not found", "找不到自訂路徑"),
    ("Not found", "未找到"),
    ("Desktop only", "僅限桌面版"),
    ("Gemini API Key", "Gemini API 金鑰"),
    ("Gemini Model", "Gemini 模型"),
];

/// Looks up the Taiwan Traditional Chinese (`zh-hant`) fallback translation for a fork-added
/// English UI key. Returns `None` when the key is not in the fork fallback table.
pub fn zh_hant(key: &str) -> Option<&'static str> {
    ZH_HANT.iter().find_map(|&(en, zh)| (en == key).then_some(zh))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{self, Lang};

    fn placeholders(s: &str) -> Vec<&str> {
        let mut names: Vec<&str> = s
            .split('{')
            .skip(1)
            .filter_map(|part| part.split_once('}').map(|(name, _)| name))
            .collect();
        names.sort_unstable();
        names
    }

    #[test]
    fn fallback_table_has_unique_keys_and_preserves_placeholders_and_ellipses() {
        let mut seen = std::collections::HashSet::new();
        for &(en, zh) in ZH_HANT {
            assert!(seen.insert(en), "duplicate copilot_i18n key: {en:?}");
            assert!(!zh.trim().is_empty(), "empty translation for key: {en:?}");
            assert_eq!(
                placeholders(en),
                placeholders(zh),
                "placeholder mismatch for {en:?} -> {zh:?}"
            );
            assert_eq!(
                en.ends_with('…'),
                zh.ends_with('…'),
                "trailing ellipsis mismatch for {en:?} -> {zh:?}"
            );
        }
    }

    #[test]
    fn zh_hant_translates_official_menus_and_copilot_fallback_while_english_stays_english() {
        if !i18n::needs_catalogs() {
            return;
        }
        let zh = Lang::from_code("zh-hant").expect("zh-hant should be available when craft-i18n is embedded");

        // Official craft-i18n strings translate normally.
        assert_ne!(i18n::tr(zh, "File"), "File");
        assert_ne!(i18n::tr(zh, "Edit"), "Edit");
        assert_ne!(i18n::tr(zh, "Layer"), "Layer");
        assert_ne!(i18n::tr(zh, "Window"), "Window");

        // Fork AI sidebar & Preferences strings resolve via zh-hant fallback.
        assert_eq!(i18n::tr(zh, "AI Assistant"), "AI 助理");
        assert_eq!(i18n::tr_id(zh, crate::copilot_ui::PANEL_ID, "AI Assistant"), "AI 助理");
        assert_eq!(i18n::tr(zh, "Vision Presets"), "視覺分析範本");
        assert_eq!(
            i18n::tr(zh, "Allow CLI to control PhotoCraft (MCP)"),
            "允許 CLI 透過 MCP 控制 PhotoCraft"
        );
        assert_eq!(i18n::tr(zh, "Auto-approve headless tool calls"), "自動核准背景工具呼叫");
        assert_eq!(i18n::tr(zh, "Default AI Provider"), "預設 AI 供應商");
        assert_eq!(i18n::tr(zh, "Check CLI Environment"), "檢查 CLI 環境");
        assert_eq!(i18n::tr(zh, "Gemini API Key"), "Gemini API 金鑰");
        assert_eq!(
            i18n::fmt(i18n::tr(zh, "MCP Control · Port {port}"), &[("port", "9420")]),
            "MCP 控制通道 · 連接埠 9420"
        );

        // English stays untouched for both official and fork strings.
        assert_eq!(i18n::tr(Lang::EN, "File"), "File");
        assert_eq!(i18n::tr(Lang::EN, "AI Assistant"), "AI Assistant");
        assert_eq!(i18n::tr(Lang::EN, "Vision Presets"), "Vision Presets");
        assert_eq!(i18n::tr(Lang::EN, "Check CLI Environment"), "Check CLI Environment");

        // Other locales do not receive zh-hant fallback text.
        if let Some(ja) = Lang::from_code("ja") {
            assert_eq!(i18n::tr(ja, "Vision Presets"), "Vision Presets");
        }
        if let Some(zh_hans) = Lang::from_code("zh-hans") {
            assert_eq!(i18n::tr(zh_hans, "Vision Presets"), "Vision Presets");
        }
    }
}
