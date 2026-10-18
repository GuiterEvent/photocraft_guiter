# 貢獻

- **語言：** 僅 Rust。沒有 JavaScript 或 TypeScript。在網路上，`wasm-bindgen`產生一個小型載入器；永遠不要手寫JS。
- **許可證：** 貢獻為 MIT 或 Apache-2.0（`LICENSE-MIT`、`LICENSE-APACHE`、`NOTICE`）。`docs/brand/`中的ArtCraft標誌不是開源的 (`docs/brand/LICENSE-brand.txt`)。
- **無塵室：** 請勿從專有軟體（例如Photoshop）複製程式碼、著色器、圖示、ICC 設定檔案或其他資產。透過觀察和公共規格來匹配行為和外觀。僅使用具有許可許可的資產，將其許可證放在資產旁邊（例如`assets/fonts/OFL-*.txt`、`assets/icons/LICENSE-lucide.txt`），並列出`ATTRIBUTION.md`中的每個資產。 **字體** 存在於 [storytold/craft-fonts](https://github.com/storytold/craft-fonts) 中，而不是在此存儲庫中：在那裡添加新字體，並通過可選的`CRAFT_FONTS_DIR`構建輸入（`docs/development.md`級別 字體；`../craftrules/standards/fonts.md`中構建輸入的規則，不是公開的，因此請向維護人員詢問與您的相關的規則）變更）。
- **永不崩潰：**非測試程式碼不得恐慌：沒有`unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!`和沒有`unsafe`。錯誤會經過`Result`和`?`，檢查輸入派生的索引和大小，每個崩潰修復都會附帶回歸測試。請參閱`AGENTS.md`中的「永不崩潰」。
- **命令，而不是處理程序：**新功能是帶有測試的引擎命令，UI 呼叫它們（下面的清單）。
- **分層：**`cargo xtask layers`必須通過。在`xtask/src/layers.rs`中註冊新的 crate。
- **測試：**每次更改都需要。格式程式碼需要往返和格式錯誤的輸入測試。像素程式碼在 8、16 和 32 位元下進行測試。如果您接觸 psd、io、編解碼器、撰寫、GPU、文字或格式，還可以執行真實文件語料庫測試：`cargo xtask test-corpus`（取得固定語料庫，包括來自 https://github.com/storytold/photocraft-corpus的Photoshop預言機，然後執行它們使用它們）。切勿提交語料庫文件；請參閱`docs/development.md`› 測試語料庫。
- **款式：**`cargo fmt`和`cargo clippy -- -D warnings`。匹配周圍的程式碼。評論解釋*為什麼*。
- **UI：** 使用`theme::Tokens`和`widgets::*`。提交前進行目視驗證（螢幕外`snapshot`範例或控制頻道），並將先前/之後的螢幕截圖附加到 PR。
- **承諾：** 小，重點突出，主題行清晰。

## 新增指令

1. **查找 id。 ** 在`crates/ui-egui/src/menu_catalog.rs`中搜尋Photoshop選單專案。使用它的 id 可以使選單專案在沒有 UI 工作的情況下生效。沒有Photoshop選單專案的指令使用相同樣式 (`layer.smartFilter.delete`) 的描述性 ID 和空選單路徑。
2. **演演算法**放入適合的最低箱子中（用於成像的`algo`、`paint`、`vector`、`text`、`cms`），並進行單元測試。它採用與深度無關的表面（`photocraft-raster`），針對每個圖塊工作，並且是確定性的（種子隨機性）。
3. **指令**進入暴露`specs()`的引擎模組 (`crates/engine/src/<area>_cmds.rs`)，該模組在`commands.rs`中向`v.extend(...)`註冊。填寫：
   -`id`、`label`、`menu`路徑和Photoshop的預設`shortcut`，
   - 參數文件字串，例如`{"radius":px=4,"mode":"a|b"="a"}`（這是代理透過`commands`/`command_list`讀取的內容），
   -`enabled`謂詞（使選單專案變灰），
   -`run`，它會傳回 JSON 結果並準確記錄**一個**歷史步驟（使用`coalesce`進行拖曳和鍵入會話）。
4. **尊重上下文：** 主動選擇（羽化）、圖層與遮罩目標、鎖定、顏色模型和深度。
5. 模組中的**測試**：行為、撤銷/重做、停用狀態、錯誤參數、多個深度。
6. **對話方塊**（如果有參數）：過濾器和調整以取得具有即時預覽功能的模式驅動對話方塊（`ui-egui/src/filter_dialog.rs`、`dialogs.rs`）。
7. 執行`cargo xtask parity`並提交更新的`docs/parity-checklist.md`。
