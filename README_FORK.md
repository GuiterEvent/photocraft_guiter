# PhotoCraft（Guiter 分支）差異說明

本文件專門說明此分支（[`GuiterEvent/photocraft_guiter`](https://github.com/GuiterEvent/photocraft_guiter)）相較於上游原版專案（[`storytold/photocraft`](https://github.com/storytold/photocraft)）所**新增與修改的內容**。

關於 PhotoCraft 整體應用程式的功能、架構與一般使用方式，請參閱原始 [README.md](README.md) 或繁體中文譯本 [README_tw.md](README_tw.md)。

> **上游同步基準**：`upstream/main` commit `da5805536`（PhotoCraft `0.23.1`）  
> 本分支刻意維持原始 `README.md` 不變，以避免未來同步上游時產生不必要的合併衝突。

---

## 與原版的主要差異總覽

| 項目 | 上游原版 (`storytold/photocraft`) | 本分支 (`photocraft_guiter`) |
| :--- | :--- | :--- |
| **內建多 AI 側邊欄助理** | 僅支援外部 Agent 透過 CLI / 控制通道 / MCP 操控 | 內建**右側停靠多 AI 側邊欄**（可切換浮動視窗），支援免 API 點數的 **Antigravity CLI (`agy`)**、**Gemini CLI (`gemini`)**、**Claude CLI (`claude`)**、**Codex CLI (`codex`)** 與 **Gemini API**，支援即時串流、自動掛載 PhotoCraft MCP 控制通道與工具執行卡片 |
| **Preferences 設定** | `Integrations` 頁僅包含 Agent / MCP 連線設定 | 擴充 `Integrations` 頁，新增預設 AI 供應商選擇、各 CLI / `photocraft-cli` 自訂路徑覆寫、一鍵非同步 **Check CLI Environment** 環境檢查，以及密碼遮罩的 **Gemini API Key** 與 **Gemini Model** 設定 |
| **Windows 打包工具** | 僅提供 `packaging/windows/package.ps1` | 新增一鍵建置腳本 `build.bat`（自動拉取並嵌入 `craft-i18n` 多國語系與 `craft-fonts` 字型），並強化 `package.ps1` 的 WiX 偵測、容錯與獨立 `.exe` 輸出 |
| **繁體中文（台灣）介面** | 需手動設定 `CRAFT_I18N_DIR` 與 `CRAFT_FONTS_DIR` | 本機 `cargo build` 與 `build.bat` 自動偵測專案根目錄的 `craft-i18n/` 與 `craft-fonts/`，並透過獨立模組 `copilot_i18n.rs` 補齊 AI 側邊欄與 Preferences AI 區塊的台灣繁體中文翻譯 |
| **macOS 雲端建置** | 需透過全平台 `release.yml` 與組織簽章憑證 | 新增純手動觸發的 `.github/workflows/build-macos.yml`，可在 GitHub Actions 一鍵產出 `.dmg` 與 CLI |
| **專案文件語言** | 以英文文件為主 | 額外新增 **97 份**繁體中文（台灣）`*_tw.md` 文件 |

---

## 1. 內建多 AI 側邊欄助理

本分支在桌面版 UI 中整合了**右側停靠式多 AI 側邊欄**（預設寬度 `360px`，可拖拉或以鍵盤調整 `300px–560px`，並可一鍵切換為浮動視窗），與既有的 `Color / Properties / Layers` 停靠面板採獨立分割區域排布，展開或縮放時不會重疊遮擋畫布或面板。

### 支援的 AI 供應商（5 種後端）

| 供應商 | 執行檔 / 連線方式 | 計費與登入方式 | MCP 操控畫布 |
| :--- | :--- | :--- | :--- |
| **Antigravity CLI**（預設） | `agy`（headless NDJSON 串流） | 使用 Google 帳號登入，**免額外 API 點數** | 自動掛載臨時 PhotoCraft MCP 設定檔 |
| **Gemini CLI** | `gemini` | Google 帳號 / API Key（依 CLI 設定） | 自動掛載臨時 PhotoCraft MCP 設定檔 |
| **Claude CLI** | `claude` | Claude 訂閱或 API（依 CLI 設定） | 自動掛載臨時 PhotoCraft MCP 設定檔 |
| **Codex CLI** | `codex` | OpenAI / ChatGPT 帳號（依 CLI 設定） | 自動掛載臨時 PhotoCraft MCP 設定檔 |
| **Gemini API** | 直接呼叫 Google Generative Language API | 需於 Preferences 填入 Gemini API Key | 內建 5 項 Function Calling 工具與工作流卡片 |

### 免 API 點數快速上手：Antigravity CLI（`agy`）

在 Windows 上不需申請付費 API Key，即可透過個人 Google 帳號免費使用 Antigravity CLI 在 PhotoCraft 側邊欄直接修圖與操控圖層：

1. **安裝 Antigravity CLI**：前往 [antigravity.google](https://antigravity.google) 安裝 Antigravity（Windows 預設安裝路徑通常為 `%LOCALAPPDATA%\agy\bin\agy.exe`，PhotoCraft 會自動掃描 `PATH` 與常見安裝目錄；若安裝於自訂位置，也可於 Preferences 指定路徑）。
2. **完成一次帳號登入**：在終端機（PowerShell 或命令提示字元）執行 `agy` 並依指示完成 Google 帳號登入。
3. **開啟側邊欄**：於 PhotoCraft 選單點選 **`Window > Gemini Copilot`**（對應指令 ID：`window.panel.geminiCopilot`、`window.toggle.geminiCopilot`、`view.geminiCopilot`），右側即展開 AI 側邊欄。
4. **啟用 MCP 控制通道**：在側邊欄上方確認 Engine 為 **Antigravity CLI**，若顯示 `MCP Control: Off`，點擊右側 **Enable** 一鍵開啟 PhotoCraft 本地控制通道。此時每次發送指令，側邊欄都會自動為 CLI 掛載指向目前 PhotoCraft 實例的 `photocraft-cli mcp` 臨時設定。
5. **（選用）自動核准無頭工具呼叫**：若希望 CLI 不必中斷詢問就能直接呼叫 PhotoCraft MCP 工具修圖，可勾選側邊欄的 **Auto-approve headless tool calls**（預設關閉，需由使用者明確勾選）。

### 核心功能

- **多後端統一抽象與 CLI 串流引擎**（`crates/ui-egui/src/ai_backend.rs`、`crates/ui-egui/src/cli_agent.rs`）
  - 透過 `AiBackend` trait 統一封裝 4 種本機 CLI Agent 與 Gemini API，支援即時串流輸出、隨時中止生成、一鍵連線測試（Test Connection）與清楚的未安裝／未登入引導提示。
  - CLI 執行過程中即時解析工具呼叫事件，在對話中繪製可展開／收合的 **MCP 工具卡片**；針對 `call_mcp_tool` 等通用包裝呼叫，會自動解開並以實際名稱（例如 `photocraft / doc_inspect`）作為標題，清楚呈現輸入參數與格式化輸出結果。
- **Gemini API 客戶端與內建 Function Calling**（`crates/ui-egui/src/gemini.rs`）
  - 支援 `/v1beta/models/{model}:generateContent` 與 **Server-Sent Events (SSE)** 串流（`:streamGenerateContent?alt=sse`）。
  - 可選模型：`gemini-2.5-flash`（預設）、`gemini-1.5-pro`、`gemini-2.5-pro`、`gemini-2.0-flash`。
  - 內建 5 項需使用者確認的 Function Calling 工具：`get_document_info`、`execute_command`、`adjust_layer_property`、`inspect_canvas_image`、`plan_workflow_actions`（多步驟工作流逐項或一鍵執行）。
- **多模態畫布視覺分析**
  - 可透過工具列按鈕或快捷鍵 **`Ctrl+Shift+C`**（macOS 為 `Cmd+Shift+C`）擷取目前畫布縮圖（標準 `1024px`／高畫質 `1536px`）附加到對話中，點擊縮圖可開啟放大檢視視窗。
  - 擷取同時自動計算視覺診斷指標（尺寸、長寬比、平均亮度與曝光評估、亮／暗部裁切百分比、平均 RGB 與色偏判斷），並提供 4 組 **Vision Presets** 快捷提示選單（曝光與直方圖、調色風格、修圖瑕疵、構圖與裁切）。
- **側邊欄排版、輸入體驗與會話管理**（`crates/ui-egui/src/copilot_ui.rs`、`crates/ui-egui/src/state/copilot.rs`）
  - 支援多會話抽屜（新增、切換、重新命名、刪除）、Token 用量估算與 32k Context 自動裁切、Markdown／程式碼區塊自動換行排版、單則回覆一鍵複製，以及對話匯出／匯入（Markdown / JSON）。
  - 底部輸入框支援 `Enter` 送出、`Shift+Enter` 換行，且取得焦點時 `Ctrl+V` / `Ctrl+C` / `Ctrl+X` / `Ctrl+A` 與一般單鍵打字皆完整作用於輸入框，不會被畫布全域快捷鍵攔截。

### Preferences > Integrations 設定說明

開啟 **`Edit > Preferences > Integrations`** 可統一管理 AI 助理環境：

- **Default AI Provider**：切換預設啟動的 AI 引擎（`Antigravity CLI`、`Gemini CLI`、`Claude CLI`、`Codex CLI`、`Gemini API`）。
- **自訂執行檔路徑覆寫（選填）**：提供 `agy`、`gemini`、`claude`、`codex` 與 `photocraft-cli` 五組覆寫欄位；留空時維持自動偵測，填入路徑（支援執行檔完整路徑或所在資料夾）時將優先採用並自動更新偵測快取。
- **Check CLI Environment**：一鍵在背景非同步掃描本機所有 CLI 與 `photocraft-cli` 的安裝狀態、實際路徑與版本號，不阻塞 UI。
- **Gemini API Key & Model**：完整保留密碼遮罩的 API Key 輸入與模型選擇（供選用 `Gemini API` 後端時使用）。

> **注意**：CLI 子行程呼叫與原生 HTTP 串流（`reqwest` + `pollster`）僅於原生桌面平台（Windows / macOS / Linux）啟用；WebAssembly 建置會安全隱藏 CLI 路徑設定並於執行時回傳不支援平台提示。

---

## 2. Windows 一鍵建置與打包強化

- **新增 [`build.bat`](build.bat)**
  - 雙擊或在命令提示字元執行（可選參數：`x64`、`x86`、`arm64`，預設 `x64`）。
  - 自動偵測並設定 Rust `cargo.exe`、`.NET SDK` 與 WiX v5 工具路徑、Windows SDK `rc.exe`（嵌入圖示與版本資訊）。
  - **自動拉取官方多國語系與字型**：打包前會自動檢查 `CRAFT_I18N_DIR` 與 `CRAFT_FONTS_DIR`；若未設定且專案根目錄尚未快取，會透過 `git` 依 `craft-i18n.rev` 與鎖定版本自動拉取官方 `storytold/craft-i18n`（含完整台灣繁體中文 `zh-hant`）至 `craft-i18n/`、拉取 `storytold/craft-fonts` 至 `craft-fonts/` 並匯出環境變數（這兩個資料夾皆列於 `.gitignore`，不進 Git；若離線抓取失敗會印出警告並安全降級為純英文／系統字型，不中斷建置）。
  - 本機直接執行 `cargo build` 時，`crates/ui-egui/build.rs` 與 `crates/text/build.rs` 亦會在未設定環境變數的情況下自動偵測專案根目錄的 `craft-i18n/locales` 與 `craft-fonts`，使本機開發建置預設即具備完整繁體中文與內嵌字型（WASM 建置依上游規則維持不內嵌字型）。
  - 依序呼叫 `packaging\windows\package.ps1` 產生安裝檔與可攜版壓縮檔，並將獨立的 `photocraft.exe` 與 `photocraft-cli.exe` 一併整理至 `dist\release\`。

### 繁體中文介面啟用方式

- **自動偵測（預設 `Auto`）**：在繁體中文版 Windows（`zh-TW` / `zh-HK` / `zh-MO`）啟動包含 `craft-i18n` 的建置版本時，`Language = Auto` 會自動切換為繁體中文（`zh-hant`）。
- **手動切換**：開啟 **`Edit > Preferences > Interface > Language`**（繁體中文介面為 **`編輯 > 偏好設定 > 介面 > 語言`**），選擇 **繁體中文** 並按 **Apply / OK** 即可立即生效。
- **AI 側邊欄與 Preferences 繁體中文備援**：除官方 `craft-i18n` 主介面翻譯外，本分支新增的 AI 側邊欄與 Preferences `Integrations` AI 設定區塊由獨立模組 `crates/ui-egui/src/copilot_i18n.rs` 提供台灣繁體中文備援（僅在語系為 `zh-hant` 且官方語料無對應項目時啟用，不影響英文與其他語系）。
- **強化 [`packaging/windows/package.ps1`](packaging/windows/package.ps1)**
  - 當 `wix.exe` shim 不可用或 `.NET` 安裝在使用者目錄（`%USERPROFILE%\.dotnet`）時，自動透過 `dotnet exec` 執行 WiX 5.0.2 的 `wix.dll`。
  - `photocraft-cli --version` 煙霧測試改用 `try / catch` 包裝，避免受特定本機環境限制而中斷整體打包流程。
  - 打包完成後除了 `.msi`、`-portable.zip`、`-symbols.zip` 外，一併將獨立執行檔 `photocraft.exe` 與 `photocraft-cli.exe` 複製到輸出目錄。

---

## 3. macOS 雲端一鍵建置（GitHub Actions）

- **新增 [`.github/workflows/build-macos.yml`](.github/workflows/build-macos.yml)**
  - 純手動觸發（`workflow_dispatch`），不會因 `push` 或排程自動執行，也不會與上游 workflow 衝突。
  - 在 GitHub 倉庫 **`Actions` → `Build macOS` → `Run workflow`** 可選擇架構（`aarch64` Apple Silicon、`universal` 雙架構通用、`x86_64` Intel），自動拉取 `craft-i18n` 與 `craft-fonts` 並呼叫 `packaging/macos/package.sh` 產出 `.dmg` 與 CLI `.zip` 供下載。
  - 下載安裝到 `/Applications` 後，因屬 ad-hoc 簽章（未經 Apple Developer ID 公證），首次開啟前於 macOS 終端機執行 `xattr -cr /Applications/PhotoCraft.app` 即可正常啟動。

---

## 4. 繁體中文（台灣）專案文件

本分支額外收錄了 **97 份**繁體中文文件（檔名皆以 `*_tw.md` 結尾，不覆蓋原版英文文件）：

- **根目錄（5 份）**：`README_tw.md`、`AGENTS_tw.md`、`ATTRIBUTION_tw.md`、`ROADMAP_tw.md`、`SECURITY_tw.md`
- **專案手冊 `book/`（36 份）**：架構說明、自動化／CLI／MCP、安全模型、檔案格式、開發與測試指南
- **技術文件 `docs/`（51 份）**：架構、控制協議、本地化、效能調校、UI 設計與相容性評分表等
- **其他模組與打包文件（5 份）**：`assets/app-icon/README_tw.md`、`crates/raw/README_tw.md`、`examples/plugins/invert-rs/README_tw.md`、`packaging/macos/dmg/README_tw.md`、`packaging/web/README_tw.md`

---

## 程式碼異動清單（相對於 `upstream/main`）

以下列出所有與程式碼、設定、依賴及打包相關的新增／修改檔案（不含 `*_tw.md` 文件）：

### 新增檔案
- `crates/ui-egui/src/ai_backend.rs` — 多 AI 後端統一抽象層（`AiBackend`、`AiStreamEvent`、`AiRequestConfig`）與後端工廠
- `crates/ui-egui/src/cli_agent.rs` — 本機 CLI Agent 執行器（`agy`、`gemini`、`claude`、`codex`）、自動偵測與自訂路徑覆寫、PhotoCraft MCP 臨時設定檔生成、NDJSON 串流解析與非同步環境檢查
- `crates/ui-egui/src/gemini.rs` — Gemini API 客戶端、SSE 串流解析、視覺縮圖編碼、影像診斷指標與 Function Calling 定義
- `crates/ui-egui/src/copilot_i18n.rs` — 本分支新增 AI 側邊欄與 Preferences AI 設定之獨立台灣繁體中文（`zh-hant`）翻譯備援對照表與單元測試
- `crates/ui-egui/src/copilot_ui.rs` — 右側停靠／浮動多 AI 側邊欄 UI、工作區無重疊分割與拖拉調整、歷史抽屜、視覺預覽、MCP 工具卡片與工作流執行器
- `crates/ui-egui/src/state/copilot.rs` — AI 側邊欄會話、訊息、圖片附件、CLI 工具進度卡片、停靠寬度與 MCP 自動核准狀態資料結構
- `build.bat` — Windows 一鍵建置與打包啟動腳本（含自動拉取 `craft-i18n` 與 `craft-fonts`）
- `.github/workflows/build-macos.yml` — GitHub Actions 手動觸發 macOS `.dmg` 建置流程
- `README_FORK.md` — 本分支差異說明文件

### 修改檔案
- `.gitignore` — 忽略本機快取的 `/craft-i18n`（與既有的 `/craft-fonts` 一致，避免提交進 Git）
- `crates/text/build.rs`、`crates/ui-egui/build.rs` — 在未設定 `CRAFT_FONTS_DIR` / `CRAFT_I18N_DIR` 時自動偵測專案根目錄的 `craft-fonts/` 與 `craft-i18n/locales`
- `crates/engine/src/prefs.rs`、`crates/engine/src/prefs/validate.rs`、`crates/engine/src/prefs/tests.rs` — 新增 `AiProvider`、`GEMINI_MODELS` 與 `Integrations` 欄位（`ai_provider`、`agy_cli_path`、`gemini_cli_path`、`claude_cli_path`、`codex_cli_path`、`photocraft_cli_path`、`gemini_api_key`、`gemini_model`）及驗證規則
- `crates/ui-egui/Cargo.toml`、`Cargo.lock` — 新增 `thiserror`，以及原生平台依賴 `reqwest`（`rustls`、`json`）與 `pollster`
- `crates/ui-egui/src/lib.rs` — 掛載 `ai_backend`、`cli_agent`、`copilot_i18n`、`copilot_ui` 與 `gemini` 模組
- `crates/ui-egui/src/i18n.rs`、`crates/ui-egui/src/i18n/tests.rs` — 於 `tr()` 僅在 `zh-hant` 查無官方翻譯時備援查詢 `copilot_i18n::zh_hant`，並補齊繁體中文驗證測試
- `crates/ui-egui/src/app.rs`、`crates/ui-egui/src/input/shortcuts.rs` — 確保 AI 側邊欄輸入框聚焦時 `Ctrl+V` / `Ctrl+C` / `Ctrl+X` / `Ctrl+A` 與單鍵輸入正常作用，不被畫布全域快捷鍵攔截
- `crates/ui-egui/src/app/frame.rs` — 於 `AppShell::body` 整合 `copilot_ui::show_body_with_docked_sidebar` 嚴格切分工作區與右側 AI 側邊欄，並於主繪製迴圈呼叫 `copilot_ui::windows`
- `crates/ui-egui/src/state.rs` — 匯出 `state::copilot` 型別，並於 `UiState` 加入 `ai_provider`、`gemini_api_key`、`gemini_model`、`copilot`
- `crates/ui-egui/src/menus/bar.rs`、`invoke.rs`、`items.rs`、`state.rs` — 掛載 `Window > AI Assistant` 選單項目、勾選狀態與命令派發，並將選單英文名稱快取改為 `thread_local!` 以避免多語系測試並行競態
- `crates/ui-egui/src/prefs/prefs_ui.rs`、`agents.rs`、`custom_rows.rs`、`dialogs.rs`、`fields.rs`、`save.rs`、`startup.rs`、`tests.rs` — 完善 Preferences `Integrations` 頁的 AI Assistant 區塊（預設供應商、CLI 路徑覆寫、一鍵非同步 Check CLI Environment、Gemini API Key 與 Model）、與偵測快取同步，並修正 `dialogs.rs` 中 `auto` 語系透過 `crate::i18n::system_lang()` 解析
- `packaging/windows/package.ps1` — WiX `dotnet exec` 備援偵測、煙霧測試容錯與獨立 `.exe` 輸出

---

## 後續同步上游與驗證

日後若要再次從原版更新，可使用以下指令：

```powershell
git fetch upstream
git merge upstream/main
```

解決完任何潛在衝突後，建議執行下列檢查確認建置與整合無誤：

```powershell
cargo check -p photocraft-text -p photocraft-engine -p photocraft-ui-egui -p photocraft -p photocraft-cli --all-targets
cargo test -p photocraft-engine --lib prefs
cargo test -p photocraft-engine --test it prefs_usage
cargo test -p photocraft-ui-egui --lib i18n
cargo test -p photocraft-ui-egui --lib copilot_i18n
cargo test -p photocraft-ui-egui --lib copilot_ui
cargo test -p photocraft-ui-egui --lib gemini
cargo test -p photocraft-ui-egui --lib ai_backend
cargo test -p photocraft-ui-egui --lib cli_agent
cargo test -p photocraft-ui-egui --lib prefs::prefs_ui
cargo clippy -p photocraft-engine -p photocraft-ui-egui --all-targets -- -D warnings
cargo xtask layers
cargo xtask panel-commands
```
