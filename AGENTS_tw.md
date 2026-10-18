# AGENTS.md：AI 代理和貢獻者指南

PhotoCraft是一款開源、原生、可與Photoshop相媲美的圖像編輯器，僅使用 Rust 編寫（無 JavaScript 或 TypeScript）。 **沒有 Tauri、Electron 或 webview shell：** 桌面應用程式是wgpu上的本機egui/eframe，Web 建置與編譯為WebAssembly的 Rust 相同（trunk + wasm-bindgen）。切勿新增 Tauri（或任何 webview/JS UI 框架）作為相依性、建置步驟或打包目標。產品名稱總是以使用者為導向的文字形式寫成 **PhotoCraft**（PascalCase 形式的`{Function}Craft`，與其兄弟ArtCraft、ArtCraftX、DesignCraft、VectorCraft、XZZPH8ZX、DesignCraft、XZZPHZZZX、XZZPH一樣）：UI、視窗標題、關於、安裝程式、版本名稱、文件散文。機器名稱保持小寫：套件 (`photocraft-*`)、二進位檔案、檔案名稱、ID (`ai.storyteller.photocraft`)。`../craftrules`中存在跨製作應用程式共享的標準和學習內容（請閱讀其`README.md`；儲存庫不是公開的 - 外部貢獻者應向維護者詢問與其更改相關的標準）。在那裡貢獻可重複使用的知識；共享的 Rust 元件位於 [storytold/craft-libs](https://github.com/storytold/craft-libs) 中，並由不可變的 git 修訂版固定。 UI 庫停留在 L6。目標是 1:1Photoshop奇偶校驗（相同的選單、捷徑、行為和檔案保真度），具有更好的效能，並且每個功能都可由代理程式驅動。首先讀取此文件，然後讀取`docs/`。

## 1. 迎新（5 分鐘）

|閱讀 |為什麼 |
|---|---|
|`docs/architecture.md`| Crate 映射、相依性層、引擎/UI 接縫、文件模型 |
|`docs/development.md`|以程式設計方式建置、測試、執行、驅動應用程式、偵錯技巧 |
|`docs/contributing.md`|規則：無塵室、測試、分層、風格、提交； “新增指令”清單 |
|`docs/control-protocol.md`| JSON 控制色版：代理程式如何驅動和截圖正在執行的應用程式 |
|`docs/ui-design.md`|設計令牌、主題、小部件以及如何匹配Photoshop的外觀 |
|`ROADMAP.md`|一頁摘要：階段、兩個奇偶校驗數、每個維度、語言、下一步 |
|`docs/target-app-parity.md`|針對Photoshop2026 的權威平價評估：數字、重量、證據、時間 |
|`docs/gaps.md`|每個已知的不足，排名：可供選擇的工作清單|
|`docs/roadmap.md`|里程碑、**當前焦點**、下一步的估計 |
|`docs/ui-parity.md`、`docs/brush-parity.md`、`docs/context-menu-parity.md`、`docs/file-format-parity.md`、`docs/hardware-parity.md`、`docs/localization-parity.md`|依維度奇偶校驗清單 |
|`docs/parity-checklist.md`|產生每個Photoshop選單專案（即時或缺少）的清單 |
|`docs/scorecard.md`|產生的記分卡：效能預算和數字、語料庫樓層、每個區域的清單（工具、文件、UI、類型、自動化、可靠性、分佈）、不執行任何操作的設定 |
|`crates/<name>/README.md`（如有）|該箱子的公共 API |
| [photocraft-corpus](https://github.com/storytold/photocraft-corpus) +`docs/development.md`› 测试语料库 |真实文件测试预言机（我们的Photoshop创作的 PSD）；psd-tools、ag-psd和PngSuite由`cargo xtask corpus --all`在`xtask/src/corpus_pins.rs`的引脚处取入`corpus/`|

## 2. 工作區地圖

```text
crates/
  geom cms color raster      L0 foundation (geometry, ICC colour management, pixel formats + blend math, COW tiles)
  psd codecs                 L0 standalone format crates (no workspace deps; publishable)
  tablet                     L0 standalone pen tablet input (macOS AppKit, X11 XInput2); the one isolated unsafe crate
  doc                        L1 document model (layers, masks, adjustments, effects, smart objects: pure data)
  ops paint algo text vector L2 history, brush engine, imaging algorithms, type engine, paths/shapes
  compose gpu format         L3 CPU compositor (the oracle), wgpu compositor, .pcraft native format
  io plugins                 L4 document <-> PSD / flat formats; sandboxed WebAssembly plug-ins
  engine                     L5 Session + command registry (every action is a command)
  ui-egui automation         L6 egui shell (thin: all actions go through the engine); MCP server
  testkit                    test helpers
apps/
  photocraft                 desktop app (eframe/wgpu), TCP control server
  photocraft-cli             headless CLI (convert/info/run/batch/commands/mcp)
  photocraft-web             the same app in the browser (trunk + wasm-bindgen)
  photocraft-web-boot        starts the packaged browser app from compressed parts of its wasm (no file over a host's size cap)
xtask/                       cargo xtask layers | wasm | ci | stats | corpus | test-corpus | parity | perf | scorecard
```

**分層由`cargo xtask layers`強制執行**。板條箱可能僅相依性較低層。`psd`、`codecs`和`cms`不相依性工作區中的任何內容。`ui-egui`以下的任何內容不得使用egui、eframe、winit或rfd。新的箱子必須在`xtask/src/layers.rs`中註冊。

## 3. 黃金法則

### 永遠不會崩潰（超越功能工作）

人們信任PhotoCraft的工作，一旦崩潰就會失去它。格式錯誤的檔案、錯誤的命令或 MCP 參數、損壞的設定檔、奇怪的按鍵或完整的磁碟必須產生使用者或代理可以採取行動的錯誤，而不是恐慌。不要透過添加恐慌路徑來發布功能；在建置之前修復崩潰。共享標準是`../craftrules/standards/never-crash.md`。

- **非測試程式碼永遠不會恐慌。 ** 沒有`unwrap()`、`expect()`、`panic!`、`unreachable!`、`todo!`或`unimplemented!`。傳回 crate 的錯誤類型並使用`?`進行傳播；使用`ok_or(..)?`、`let .. else { return Err(..) }`、`if let`或`unwrap_or*`，其中後備確實正確（絕不會默默地損壞文件）。未完成的功能會傳回「不受支援」的錯誤。唯一的例外是一個可以證明絕對正確的文字：`#[allow(clippy::expect_used)]`加`.expect("why it can't fail")`。
- **無`unsafe`。 ** 工作區設定`unsafe_code = "forbid"`。一個例外是隔離的輔助包`photocraft-tablet`(`crates/tablet`)：winit會丟棄數位板數據，並且在 macOS 上讀取它需要 AppKit 事件監視器（Objective-C 互通）。只有其`src/macos.rs`允許`unsafe`（`unsafe_code = "deny"`板條箱範圍內，每個區塊都有`SAFETY:`註釋，針對真實的`NSEvent`進行測試）；它的 X11 路徑和所有映射程式碼都是安全的。不要在其他任何地方添加`unsafe`。
- **輸入派生的數字是敵對的。 ** 使用`get()`而不是`[i]`/`[a..b]`作為來自檔案、參數、選擇或算術的索引；僅在字元邊界處對字串進行切片；使用`checked_*`/`saturating_*`來取得長度、上限量和強制計數；
- **有深度限製或可見集的綁定遞歸**（文件可以是深的或循環的）。
- **不要級聯。 ** 處理鎖中毒 (XZZPH100​​ZZX) 並將執行緒連接視為`Result`。
- **最後的保護。 ** 應用程式 shell 必須擷取命令調度和檔案匯入/匯出周圍的逃逸恐慌，將其報告為錯誤並保留文件。這是一個安全網，而不是恐慌的許可證。保留`panic = "unwind"`。
- **證明這一點。 ** 每個崩潰修復都會附帶一個小型綜合回歸測試，該測試在修復之前會出現恐慌。
- **由 Clippy 強制執行。 **`clippy.toml`僅在測試中允許`unwrap`/`expect`/`panic`/索引。乾淨的板條箱攜帶`#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]`；新的板條箱從它開始。

1. **一切都是命令。 ** 新的使用者可見行為 = 引擎中的指令（`crates/engine/src/*_cmds.rs`，在`commands.rs`中註冊），帶有 id、標籤、選單路徑、捷徑、參數文件、Photoshop和`run`，以及測試。 UI、CLI、控制色版和MCP都透過id來調度指令。使用`crates/ui-egui/src/menu_catalog.rs`中的 **準確 ID，選單專案會自動生效。只有純視圖/視窗狀態（縮放、面板、螢幕模式）屬於外殼（`menus.rs``UI_COMMANDS`）。
2. **無格式或顏色假設。 ** 位元深度 (8/16/32f) 和色彩模型 (RGB/Gray/CMYK/Lab...) 是執行階段資料。切勿在公用 API 中引入僅限`u8`的像素路徑。切勿假設 sRGB：顏色轉換經過`photocraft-cms`（`Transform`、`transform::cached`）。在幾個深度進行測試。
3. **無塵室。 ** 我們研究了Photoshop和其他專有編輯器的*行為和外觀*。切勿複製他們的程式碼、著色器、設定檔或資產。根據公共規範（Adobe PSD 規範、ICC、ISO 32000 混合模式、論文）和觀察來實施。第三方資產必須獲得許可，將其許可證文件放在旁邊，並在同一更改中在`ATTRIBUTION.md`（路徑、標題、作者、來源、許可證）中獲得一行；原始資產也是如此。`docs/brand/`中的ArtCraft標誌不是開源的 (`docs/brand/LICENSE-brand.txt`)。 **永遠不要提交字體檔案：**字體位於 [storytold/craft-fonts](https://github.com/storytold/craft-fonts) 中（規則：`../craftrules/standards/fonts.md`）。PhotoCraft透過可選的構建輸入`CRAFT_FONTS_DIR=<craft-fonts checkout>`使用它（由`crates/text/build.rs`讀取，公開為`photocraft_text::CRAFT_FONTS`；沒有它則為空，一切都必須仍然有效）。用它建造：`CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo build`；詳細資訊請參閱`docs/development.md`› 字體。
4. **測試是大門。 ** 每一次改變都伴隨著測驗。格式箱使用往返、合成生成器、預言機和模糊測試。防止 PSD 語料庫結果和奇偶校驗下限 (`crates/ui-egui/src/parity.rs`) 回歸。
5. **UI 很薄且由資料驅動。 ** UI 狀態位於`ui-egui/src/state.rs`(serde) 中，因此控制色版可以讀取並驅動它。顏色和半徑來自`theme::Tokens`，從未進行硬編碼。
6. **視覺驗證 UI 變更。 ** 使用`cargo run -pphotocraft-ui-egui--example snapshot`進行離屏渲染（無窗口，無焦點竊取），或使用`--control`啟動並拍攝`ui.screenshot`。看看PNG。演示圖像必須是公共領域的藝術作品，而不是個人照片。取得資產時，切勿將個人姓名、電子郵件或其他個人詳細資料放入請求中（使用者代理、標頭、URL）；使用通用`Photocraft-dev`使用者代理程式。
7. **切勿破壞 wasm。 ** L0–L6 必須`cargo check --target wasm32-unknown-unknown`（執行`cargo xtask wasm`）。檔案系統程式碼是`cfg(not(target_arch = "wasm32"))`或透過平台服務。
8. **效能是一項功能。 ** 在發佈的 24-36 MP 影像上對大量操作進行基準測試。每個圖塊並行工作（人造絲），跳過空圖塊，從不掃描每個畫面的整個表面（每個修訂版快取），並在開發日誌中記錄之前/之後的計時。

9. **永遠不要對輸入感到恐慌**（請參閱上面的*永遠不要崩潰*）。命令的`run`閉包及其呼叫的任何內容都必須返回`Err`，永遠不要驚慌，對於*任何*參數或文件狀態：驗證參數，在索引或劃分之前檢查邊界，並在分配之前拒絕荒謬的大小。`panic_hunt`整合測試使用對抗性參數模糊每個指令，並且必須保持綠色。

## 4. 揀貨工作

優先事項：首先是重要的基礎設施，然後是容易實現的平價，然後是長尾。

0. **讀取`ROADMAP.md`，然後讀取`docs/gaps.md`。 **`docs/target-app-parity.md`表示，尺寸為
   尺寸，其中缺少PhotoCraft；`docs/gaps.md`對作品進行排名。`docs/parity-checklist.md`
   （菜單連接）不是行為的衡量標準。當你的工作改變了數字或縮小了差距時，
   使用日期更新其所屬的奇偶校驗文件和間隙條目，並變更其狀態行
   和修訂歷史（工藝規則`standards/progress-docs.md`）。
1. **在揀貨工作前檢查`docs/scorecard.md`**：每個區域的`missing`和`partial`行，
   超出預算或尚無法衡量的效能場景，以及數量
   不執行任何操作的設定。它的數量是經過測量的；更喜歡它們而不是估計。
2.`docs/roadmap.md`→ **目前焦點**。
3.`cargo xtask parity`→`docs/parity-checklist.md`列出每個缺少的選單專案，依選單分組。容易實現的目標通常是缺少一個指令，其演演算法已存在於`algo`、`paint`、`vector`或`text`中。
4.`log/devlog.md`→ 最近條目的「仍然開放」專案符號。

當奇偶校驗上升時，提高`crates/ui-egui/src/parity.rs`中的`FLOOR`（切勿降低）。

## 5. 完成任務前

```sh
cargo test -p <crates you touched>
cargo clippy -p <crates> --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm            # if you touched L0–L6
cargo xtask parity          # if you added commands; commit the regenerated docs/parity-checklist.md
cargo test -p photocraft-engine --test panic_hunt -- --ignored   # if you added/changed commands: no panic on adversarial input (Rule 9)
cargo xtask scorecard       # if you moved a number: flip the checklist row in scorecard/*.toml, raise a
                            # corpus floor, fix a dead preference, or meet a budget (then set enforce = true
                            # in perf/budgets.toml); commit the regenerated docs/scorecard.md (CI checks it)
cargo xtask perf --quick    # if you touched a hot path; `cargo xtask perf --update-baseline` publishes a full run
cargo xtask test-corpus     # if you touched psd, io, codecs, compose, gpu, text or format (or: --changed decides)
```

迭代時，執行 less 並保留上面的完整命令作為最終入口：`cargo test -p <crate> --lib --tests`會跳過`cargo test`每次建置的範例和文件測試（ui-egui有 11 個範例），並且`cargo test -p <crate> --test <file>`執行一個整合測試檔。`cargo nextest run`跨二進位檔案並行執行測試；請參閱`docs/development.md`› 更快的測試循環。

**測試語料庫。 **`corpus/`中的真實文件語料庫（gitignored，從未提交），在固定提交時獲取並由`cargo xtask corpus --all`進行 sha256 驗證：我們的Photoshop創作的預言機來自 https://github.com/storytold/XZXPHZ17779ag-psd和PngSuite來自其上游。腳位：`xtask/src/corpus_pins.rs`。語料庫測試是選擇加入的（貨物功能`corpus`）：普通`cargo test`會跳過它們，並且缺少語料庫的功能會失敗（“執行`cargo xtask corpus --all`”）。`cargo xtask test-corpus`取得並執行它們。 CI 始終執行它們（`corpus`作業，透過引腳快取）。切勿提交語料庫文件；新的預言機轉到photocraft-corpus（其`AGENTS.md`），然後在此處進行引腳碰撞。詳細資料：`docs/development.md`› 測試語料庫。

指令絕不能因錯誤輸入而恐慌（規則 9）：對於任何參數或文件狀態，每個`run`閉包及其所呼叫的程式碼都會傳回`Err`，而不是恐慌。新指令附帶優雅的失敗測試（空/超出範圍/錯誤類型參數 →`Err`，而不是崩潰）。

然後向`log/devlog.md`附加一個簡潔的條目（已登陸的內容、數字、仍處於開啟狀態的內容）。會話可能會突然結束（崩潰、上下文限制），因此開發日誌加上綠樹就是下一個代理的接續方式。每一步都保持樹木的建造。

## 6. 平行代理

- 使用您自己的目標目錄 (`CARGO_TARGET_DIR=target/agent-<name>`) 以避免 Cargo 建置鎖定，並僅編輯您擁有的檔案。共享檔案（`engine/src/lib.rs`、`engine/src/commands.rs`、`ui-egui/src/menus.rs`、`state.rs`中的`v.extend(...)`清單）進行小型外科手術編輯；編輯前重讀。
- 將新指令放入**新模組**（具有`specs()`函數的`engine/src/<area>_cmds.rs`），而不是產生共用檔案。
- 如果其他人正在進行的編輯破壞了構建，請等待並重試；不要修復他們的文件。
- 保持每個`Cargo.toml`始終有效：`crates/*`全域意味著一個損壞的清單會破壞每個人的建置。 **自動建立或重寫清單**：寫入`crates/`外部的臨時文件，然後將`mv`寫入到位。
- 磁碟空間：每個目標目錄約10 GB。刪除已完成代理程式的`target/agent-*`目錄。

## 7. 追蹤事物的位置


-`docs/roadmap.md`：里程碑 M0–M12、狀態與目前焦點。
-`ROADMAP.md`、`docs/target-app-parity.md`、`docs/gaps.md`和`docs/*-parity.md`清單：我們反對Photoshop的地方。
-`docs/parity-checklist.md`：產生的Photoshop選單覆蓋。
-`docs/scorecard.md`：產生的記分卡（資料來源：`scorecard/*.toml`、`perf/budgets.toml`、`perf/baseline.json`、語料庫樓層、首選項審核）。
-`docs/releasing.md`：剪切版本（`cargo xtask version`、`release`分支）、簽署機密、在`packaging/`中打包指令碼。
-`../craftrules/release/playbook.md`：每個講故事的應用程式如何建立簽名的發布二進位（規範配方；`docs/release-playbook.md`只是指向那裡）；`docs/releasing.md`是PhotoCraft的規格。
-`plan/`（本地，gitignored）：研究、奇偶計劃、執行計劃、估計。
-`log/`（本地，gitignored）：開發日誌。

## 8. 保持原生格式完整

當`photocraft-doc`結構體獲得字段時，`photocraft-format`故意編譯失敗，因此
`.pcraft`保存中不會默默刪除任何內容。新增文件欄位時，將其新增至
`crates/format/src/manifest.rs`和`convert.rs`以及`#[serde(default)]`，因此舊檔案仍會載入。
如果該欄位具有等效的 PSD，也將其對應到`crates/io`中，並逐字保留未知的 PSD 區塊。

## 貢獻者積分（關於視窗）

- 關於 ▸ 貢獻者/模型被編譯成來自`contributors/contributors.json`的二進位文件
  （提交統計資料；生成，從不手動編輯）和`contributors/people.toml`（人們選擇的名稱
  他們自己）。參見`docs/contributors.md`。
- **為貢獻者工作的代理：** 當您準備 PR 時，請檢查您的人員的 GitHub 是否
  使用者名稱在`contributors/people.toml`中有一個`[people.<username>]`條目。如果沒有，請詢問他們一次
  他們是否希望獲得使用者名稱以外的其他資訊：真實姓名、顯示名稱和/或
  他們的公開 GitHub 個人資料名稱 (`sync_github_name = true`)。如果是，請添加**僅自己的**條目
  （複製檔案頂部的模板，或執行
`python3 ../../craftrules/scripts/contributors.py --add-me . --real-name "…" --sync-github-name`)
並將其納入他們的公關中，作為他們的承諾。如果不是，則不做任何更改：它們被記為
  無論如何，`@username`。
- 切勿新增、編輯、猜測或複製任何其他人的條目或名稱（不是來自 git config、提交作者或
  GitHub 設定檔）。切勿手動編輯`contributors.json`。
- 維護者使用`python3 ../../craftrules/scripts/contributors.py .`刷新統計資料（它也
  重新驗證每個`people.toml`條目的編寫者；`--check`僅驗證）。
