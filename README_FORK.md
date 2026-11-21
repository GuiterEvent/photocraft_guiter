# PhotoCraft（Guiter 分支）差異說明

本文件專門說明此分支（[`GuiterEvent/photocraft_guiter`](https://github.com/GuiterEvent/photocraft_guiter)）相較於上游原版專案（[`storytold/photocraft`](https://github.com/storytold/photocraft)）所**新增與修改的內容**。

關於 PhotoCraft 整體應用程式的功能、架構與一般使用方式，請參閱原始 [README.md](README.md) 或繁體中文譯本 [README_tw.md](README_tw.md)。

> **上游同步基準**：`upstream/main` commit `da5805536`（PhotoCraft `0.23.1`）  
> 本分支刻意維持原始 `README.md` 不變，以避免未來同步上游時產生不必要的合併衝突。

---

## 與原版的主要差異總覽

| 項目 | 上游原版 (`storytold/photocraft`) | 本分支 (`photocraft_guiter`) |
| :--- | :--- | :--- |
| **內建 AI 助理** | 僅支援外部 Agent 透過 CLI / 控制通道 / MCP 操控 | 內建 **Gemini Copilot** 視覺對話助理面板、串流回覆、Function Calling 與多步驟工作流 |
| **Preferences 設定** | `Integrations` 頁僅包含 Agent / MCP 連線設定 | 擴充 `Integrations` 頁，新增密碼遮罩的 **Gemini API Key** 與 **Gemini Model** 選擇 |
| **Windows 打包工具** | 僅提供 `packaging/windows/package.ps1` | 新增一鍵建置腳本 `build.bat`，並強化 `package.ps1` 的 WiX 偵測、容錯與獨立 `.exe` 輸出 |
| **macOS 雲端建置** | 需透過全平台 `release.yml` 與組織簽章憑證 | 新增純手動觸發的 `.github/workflows/build-macos.yml`，可在 GitHub Actions 一鍵產出 `.dmg` 與 CLI |
| **專案文件語言** | 以英文文件為主 | 額外新增 **97 份**繁體中文（台灣）`*_tw.md` 文件 |

---

## 1. 內建 Gemini Copilot AI 助理

本分支在桌面版 UI 中整合了直接呼叫 Google Gemini API 的互動助理，無須額外架設外部 MCP 客戶端即可在 PhotoCraft 內進行影像診斷與操作輔助。

### 核心功能

- **Gemini API 客戶端**（`crates/ui-egui/src/gemini.rs`）
  - 支援 Google Generative Language API（`/v1beta/models/{model}:generateContent`）與 **Server-Sent Events (SSE) 即時串流**（`:streamGenerateContent?alt=sse`），可隨時點擊停止生成。
  - 可選模型：`gemini-2.5-flash`（預設）、`gemini-1.5-pro`、`gemini-2.5-pro`、`gemini-2.0-flash`。
  - 內建完整錯誤分類（未設定／無效 API Key、配額超限 HTTP 429、模型不存在 404、逾時、網路錯誤、安全過濾阻擋，以及 WebAssembly 平台提示）。
- **多模態畫布視覺分析**
  - 可透過工具列按鈕或快捷鍵 **`Ctrl+Shift+C`**（macOS 為 `Cmd+Shift+C`）擷取目前畫布縮圖（標準 `1024px`／高畫質 `1536px`）附加到對話中，點擊預覽圖可開啟放大檢視視窗。
  - 擷取同時自動計算視覺診斷指標：尺寸與長寬比、平均亮度與曝光評估、亮／暗部裁切百分比、平均 RGB 與色偏判斷。
  - 內建 4 組視覺分析預設提示詞（Vision Presets）：
    - 曝光與直方圖分析
    - 調色風格與步驟建議
    - 修圖瑕疵與雜訊診斷
    - 構圖與裁切建議
- **Function Calling 工具整合（需使用者確認）**
  - `get_document_info`：讀取畫布尺寸、色彩模式、位元深度、圖層清單與作用中圖層資訊。
  - `execute_command`：建議並執行 PhotoCraft 引擎指令（如反相、新增／刪除圖層、復原等），執行前會以確認卡片呈現，由使用者點擊「Apply / Execute」或「Dismiss / Cancel」。
  - `adjust_layer_property`：調整作用中圖層的不透明度、混合模式或可見度。
  - `inspect_canvas_image`：擷取合成畫布或單一作用中圖層進行視覺診斷。
  - `plan_workflow_actions`：針對複雜修圖需求產生有序的多步驟巨集清單，支援逐步執行、跳過步驟或一鍵連續執行全部步驟。
- **對話介面與會話管理**（`crates/ui-egui/src/copilot_ui.rs`、`crates/ui-egui/src/state/copilot.rs`）
  - 支援多會話抽屜（新增、切換、重新命名、刪除會話）。
  - 支援 Token 用量估算與 32k Context 自動裁切，以及對話匯出為 Markdown / JSON（含 JSON 匯入）。
  - 支援 Markdown／程式碼區塊排版、單則回覆一鍵複製與 API 連線測試按鈕。

### 使用方式

1. **設定 API Key**：開啟 **`Edit > Preferences > Integrations`**，在頁面下方的 **Gemini Copilot** 區塊填入您的 Google Gemini API Key（輸入框採密碼遮罩）並選擇模型，按下套用／確定。
2. **開啟面板**：從選單列點選 **`Window > Gemini Copilot`**（對應指令 ID：`window.panel.geminiCopilot`、`window.toggle.geminiCopilot`、`view.geminiCopilot`）。
3. **開始對話**：直接輸入問題，或按 `Ctrl+Shift+C` 附加目前畫布截圖進行視覺分析。

> **注意**：網路呼叫使用 `reqwest` + `pollster` 僅於原生平台（Windows / macOS / Linux）啟用；WebAssembly 建置會回傳不支援平台提示。

---

## 2. Windows 一鍵建置與打包強化

- **新增 [`build.bat`](build.bat)**
  - 雙擊或在命令提示字元執行（可選參數：`x64`、`x86`、`arm64`，預設 `x64`）。
  - 自動偵測並設定 Rust `cargo.exe`、`.NET SDK` 與 WiX v5 工具路徑、Windows SDK `rc.exe`（嵌入圖示與版本資訊）。
  - 依序呼叫 `packaging\windows\package.ps1` 產生安裝檔與可攜版壓縮檔，並將獨立的 `photocraft.exe` 與 `photocraft-cli.exe` 一併整理至 `dist\release\`。
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
- `crates/ui-egui/src/gemini.rs` — Gemini API 客戶端、SSE 串流解析、視覺縮圖編碼、影像診斷指標與 Function Calling 定義
- `crates/ui-egui/src/copilot_ui.rs` — Gemini Copilot 浮動對話視窗、歷史抽屜、視覺預覽、工具確認卡片與工作流執行器
- `crates/ui-egui/src/state/copilot.rs` — Copilot 會話、訊息、圖片附件、工具動作與工作流狀態資料結構
- `build.bat` — Windows 一鍵建置與打包啟動腳本
- `.github/workflows/build-macos.yml` — GitHub Actions 手動觸發 macOS `.dmg` 建置流程
- `README_FORK.md` — 本分支差異說明文件

### 修改檔案
- `crates/engine/src/prefs.rs` — 新增 `GEMINI_MODELS`、`default_gemini_model()`，於 `Integrations` 新增 `gemini_api_key`、`gemini_model`
- `crates/engine/src/prefs/validate.rs` — 註冊 `integrations.geminiModel` 選項驗證
- `crates/ui-egui/Cargo.toml`、`Cargo.lock` — 新增 `thiserror`，以及原生平台依賴 `reqwest`（`rustls`、`json`）與 `pollster`
- `crates/ui-egui/src/lib.rs` — 掛載 `copilot_ui` 與 `gemini` 模組
- `crates/ui-egui/src/app/frame.rs` — 於主 UI 繪製迴圈呼叫 `copilot_ui::windows`
- `crates/ui-egui/src/state.rs` — 匯出 `state::copilot` 型別，並於 `UiState` 加入 `gemini_api_key`、`gemini_model`、`copilot`
- `crates/ui-egui/src/menus/invoke.rs`、`items.rs`、`state.rs` — 掛載 `Window > Gemini Copilot` 選單項目、勾選狀態與命令派發
- `crates/ui-egui/src/prefs/prefs_ui/custom_rows.rs`、`dialogs.rs`、`fields.rs`、`save.rs`、`startup.rs` — 於 Preferences `Integrations` 頁面繪製 Gemini 設定並即時同步至 `UiState`
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
cargo check -p photocraft-engine -p photocraft-ui-egui -p photocraft -p photocraft-cli --all-targets
cargo test -p photocraft-engine --lib prefs
cargo test -p photocraft-engine --test it prefs_usage
cargo test -p photocraft-ui-egui --lib copilot_ui
cargo test -p photocraft-ui-egui --lib gemini
cargo test -p photocraft-ui-egui --lib prefs::prefs_ui
cargo clippy -p photocraft-engine -p photocraft-ui-egui --all-targets -- -D warnings
cargo xtask layers
cargo xtask panel-commands
```
