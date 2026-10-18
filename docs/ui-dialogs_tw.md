# 對話方塊

eguishell 中的模態和非模態參數視窗。它們鏡像Photoshop：沒有暗淡的背景，
打開對話方塊下的畫布平移/縮放，Tab 僅循環文字和數字字段，Esc 取消，Enter
當「確定」啟用時進行確認。自動化驅動與 UI 相同的`fields`映射（`ui.dialog.*`在
[`control-protocol.md`](control-protocol.md)).

＃＃ 建築學

|件|地點 |角色 |
|---|---|---|
|框架、路由、確定/取消/Esc、堆疊 |`crates/ui-egui/src/dialogs.rs`|每個`UiState::dialogs`條目均採用單一集線器 |
| Alt+取消 → 重設（選擇加入）|`crates/ui-egui/src/dialog_alt_reset.rs`|按住 Alt 時重新標記取消；調度每個標記`reset`|
|製表符欄位 |`crates/ui-egui/src/field_tab.rs`|Photoshop-style Tab / ⇧Tab（在同一檔案中測試）|
|狀態|`crates/ui-egui/src/state.rs`|`Dialog`、`DialogKind`、`open_dialog`|
|鈕排|`crates/ui-egui/src/widgets.rs`|`dialog_buttons`、`focus_first_field`;`number_edit`暫存器 Tab |
|選單/指令輸入|`dialogs::open_command_dialog`、`menus.rs`|選擇專用模組、模式對話方塊或裸確認 |
|過濾器|`filter_dialog.rs`|來自引擎參數的架構，即時預覽 |
|調整|`adjust_dialog.rs`|與 Properties 共享編輯器；透過`adjust_preview`預覽 |
|通用形式|`view_cmds.rs`|用於簡單指令參數的`__form`網格 |
|專門機構|請參閱下面的**指令路由** |`body`+ 通常為`open`/`confirm`|

**路由。 ** 非指令類型單獨使用`DialogKind`。`DialogKind::Command`閥體由下列人員選擇
`fields`上的標記鍵（`owns()`或`contains_key("__export")`等）； **第一場比賽
`dialogs::show`獲勝**，因此新標記不得與較早的標記發生碰撞。`show`、`confirm`和
`cancel`共享相同的調度表 - 新增對話方塊時連接所有三個。

**字段合約。 **`Dialog.fields`是一個 JSON 映射：指令參數代理讀寫，加上
私鑰以`__`為前綴。切勿將`__`用於面向使用者或引擎參數。

###`DialogKind`（頂級堆疊）

|親切 |車身模組|典型條目|
|---|---|---|
|`NewDocument`|`new_doc_ui.rs`|檔 › 新的，`file.new`，沒有大小參數 |
|`About`|`dialogs.rs`（關於/貢獻者/模型選項卡）| 幫助 › 關於PhotoCraft；`systemInfo: true`→ 系統資訊變體 |
|`Command`|路由－請參閱下文 |大多數帶有參數的選單指令 |
|`LayerStyle`|`layer_style.rs`|圖層 › 圖層樣式... |
|`Error`|內嵌在`dialogs.rs`|僅`message`欄位（例如移動鎖定、鎖定變換）|

### 指令路由(`DialogKind::Command`)

調度順序與`dialogs.rs`中的`dialogs::show`相符（新增標記之前檢查較早的行）。

|比賽|模組|角色 |
|---|---|---|
|`__guideLayout`|`guide_layout_ui.rs`|查看 › 新指南版面設定... |
|`__fill`|`fill_ui.rs`|編輯 › 填滿... |
|`__patternFill`|`pattern_fill_ui.rs`|圖案填滿圖層/新建填滿圖層 › 圖案... |
|`__stroke`|`stroke_ui.rs`|編輯 › 中風... |
|`__createShape`|`shape_dialog.rs`|點選形狀工具建立（矩形、橢圓形…）|
|`__deleteLayer`|`delete_layer_prompt.rs`|圖層面板垃圾確認|
|`__newLayer`|`new_layer_ui.rs`| 圖層 › 新建 › 圖層...（⇧⌘N；⌥⇧⌘N 讓圖層沒有它）|
|`__rasterize`|`rasterize_prompt.rs`|繪製前點陣化提示 |
|`__resetAllTools`|`tool_reset.rs`|重置所有工具確認 |
|`__cropPresetNew`/`__cropPresetDelete`|`crop_presets.rs`|新建/刪除裁切預設|
|`__variables`|`variables_ui.rs`|圖片 › 變數（定義、資料集、應用程式）|
|`__web`/`__print`|`file_ui.rs`|儲存為網頁、列印 |
|`__colorPicker`|`color_picker_ui.rs`|前景/背景、巢狀欄位/指令選擇器（請參閱下方的巢狀鍵）|
|`__colorRange`|`color_range_ui.rs`|選擇 › 色彩範圍... |
|`__prefsui`|`prefs_ui.rs`|編輯 › 首選項...與相關預設/導入子對話方塊 |
|`__export`|`export_dialog.rs`|匯出為（文件或圖層）|
|`__sizing`|`sizing.rs`|圖片 › 圖片尺寸... / 畫布尺寸... |
|`__adjust`|`adjust_dialog.rs`|圖片 › 調整…（也設定`__filter`+`__preview`進行即時預覽）|
|`__filter`|`filter_dialog.rs`（+`select_mask_view.rs`，當`__command`為`select.refineEdge`時）|濾鏡庫指令、編輯 › 色彩設定...、選取與遮罩以及其他架構對話方塊 |
|`__form`|`view_cmds.rs`|簡單的參數網格（例如切片選項、許多圖像>套用圖像樣式指令）|
|（以上皆非）| — | **裸確認**：僅標題+確定/取消； OK 透過通用`confirm`使用欄位執行`__command`|

**選單快捷方式。 **`dialogs::open_command_dialog`開啟指南佈局、筆畫和色彩範圍
直接；然後嘗試`adjust_dialog::open`，然後是`filter_dialog::open`；否則只是確認
與`__command`+`__label`。其他專用模組從`menus.rs`、工具或面板打開
其標記已設定的操作。

### 共享私鑰

|關鍵|意義|
|---|---|
|`__command`|引擎指令 ID 運作正常（裸確認、濾鏡、調整大小…）|
|`__label`|標題的選單/目錄標籤 |
|`__document`|目標文件 ID（驗證過時性）|
|`__dialog`/`__field`|巢狀顏色選擇器：父對話方塊 ID 與欄位名稱寫入 OK |
|`__params`|針對指令的顏色選擇器的額外指令參數 |
|`__preview`|啟用即時畫布預覽（濾鏡和調整）|
|`__choices`|`__form`對話方塊的下拉選項 |
|`__filter`|過濾器架構主體（`filter_dialog`）；也設定了預覽管線的調整|
|`__adjust`|圖片 › 調整編輯器類型 (`adjust_dialog`) |
|`__form`|通用`view_cmds::form_body`|
|`__formOrig`|開啟時非`__`參數的快照； Alt+Reset 可恢復`__form`對話方塊 |
|`__variablesOrig`|`__variables`開啟時的快照； Alt+Reset 恢復它 |
|`__altReset`|設定為`false`以從 Alt+Cancel → Reset | 選擇對話方塊
|`__export`|匯出為 |
|`__sizing`|影像尺寸/畫布尺寸|

### 佈線標記（專用`Command`主體）

|關鍵|模組|
|---|---|
|`__guideLayout`|`guide_layout_ui`|
|`__fill`|`fill_ui`|
|`__patternFill`|`pattern_fill_ui`|
|`__stroke`|`stroke_ui`|
|`__createShape`|`shape_dialog`|
|`__deleteLayer`|`delete_layer_prompt`|
|`__newLayer`|`new_layer_ui`|
|`__rasterize`|`rasterize_prompt`|
|`__resetAllTools`|`tool_reset`|
|`__cropPresetNew`,`__cropPresetDelete`|`crop_presets`|
|`__variables`|`variables_ui`|
|`__web`,`__print`|`file_ui`|
|`__colorPicker`|`color_picker_ui`|
|`__colorRange`|`color_range_ui`|
|`__prefsui`|`prefs_ui`（字串子類別：`prefs`、`presetsIO`，...）|

### 巢狀顏色選擇器（堆疊上的`__colorPicker`）

與獨立拾取器相同的集線器條目；`color_picker_ui::confirm`在額外鍵上分支：

|額外鑰匙 |行為 |
|---|---|
|`__layerStyleColor`|圖層樣式子專案; OK 在父`LayerStyle`對話方塊中寫入效果顏色 |
|`__textStyleColor`|字元/段落樣式顏色（類型面板）|
|`__shapeFill`|透過`shape.edit`形狀層實體填滿 |
|`__command`=`layer.fill.solidColor.set`|純色填滿層（無`__shapeFill`）|
|`__dialog`+`__field`|決定更新父指令對話方塊上的一個欄位（例如填滿 › 色彩...）|

## 平行模態系統（不是主堆疊）

|系統|地點 |當 |
|---|---|---|
|未儲存的變更 |`discard_ui.rs`|關閉、恢復、關閉其他/全部、退出—儲存/不儲存/取消|
|外殼對話方塊 |`control.rs`、`ui.inspect`id`"shell"`|工作區、自訂像素方面、顯示附加選項 |
|相機原始資料 |`camera_raw_ui.rs`|桌面上有獨立的作業系統視窗；請參閱控制協定 § Camera Raw |
|錯誤訊息 |`DialogKind::Error`|例如`move_lock`—訊息+確定|
|嵌套選擇器 |`color_picker_ui`（+ **嵌套顏色選擇器**中的標記）|堆疊在父級上；家長在孩子打開時確認拒絕|

## 規則

### 架構和命令

1. **瘦UI，引擎命令OK。 **執行`app.run`或專用`*_ui::confirm`；一個歷史步驟，除非命令本身批次處理。
2. **在中心注册。 ** 添加`owns()`（或稳定标记）、`body()`、`open()`（如果需要），并在`dialogs::show`、`confirm`和`cancel`（以及菜单驱动时的`open_command_dialog`）中添加匹配臂。
3. **首先擴展現有路徑。 ** 过滤器 →`filter_dialog`；调整→`adjust_dialog`；简单参数网格 →`__form`；避免第二个模态堆栈，除非产品需要 Camera Raw 级别的 UX。
4. **`fields`是自動化合約。 ** 任何代理程式都必須在命名欄位中驅動，而不僅僅是本機egui狀態。
5. **成功后记住上次使用的值**，其中Photoshop会执行此操作（`filter_dialog::remember`、首选项`dialogs["…"]`、特定于模块的帮助程序）。



6. **使用`dialog_buttons`** 進行確定/取消/應用 — 透過`ButtonRole`進行平台訂單（角落裡的 macOS 預設操作）。
7. **遵守`dialogs::show`中的寬度規則**，除非適用`file_ui::dialog_width`、顏色選擇器或特定類型的分鐘數（典型 380–440；圖層樣式/匯出/選擇器最多 600；新文件 800）。
8. **在內容內捲動長主體**（`DIALOG_CHROME`用於過濾器）；標題和按鈕行保持可見。
9. **透明背景**－無調光；預覽是根據真實對比來判斷的。
10. **位置：** 第一開居中；然後保留偏移量（每個指令或種類`place_id`）；標題列拖曳；夾在螢幕上。
11. **堆疊：**只有**頂部**對話方塊是互動式的；非頂部實例呈現停用；如果較舊的模式搶走了焦點，則引發即時對話方塊。

### 鍵盤和焦點

12. **Tab / ⇧Tab：** 僅文字和數字欄位。框架呼叫`field_tab::begin`/`end`；每個欄位都會呼叫`field_tab::register`（`number_edit`執行此操作；自訂`TextEdit`必須手動註冊）。
13. **不要透過滑桿、下拉式選單、複選框或預設卡相依性egui預設選項卡**。
14. **在第一個值上開啟：**`first_field_focused`+`widgets::focus_first_field`用於值優先對話方塊（大多數濾鏡；圖像/畫布大小；不是選擇和遮罩視圖鍵）。
15. **透過 Tab 或欄位箭頭到達時，**全選焦點**（`field_tab::select_all`；請參閱`value_field`、`new_doc_ui`）。
16. **當啟用OK時輸入**確認（`can_ok`）；例外必須是明確的（`guide_layout_ui`，顏色選擇器）。
17. **Esc** 在最上面的對話方塊中，沒有彈出視窗開啟 → 取消。 **按一下外部不會關閉**（畫布仍可透過`free_pointer_over`、`pan_delta`、`free_press`平移/縮放）。 Esc 忽略 Alt（它不會變成 Reset）。
18. 當對話方塊透過`dialog_alt_reset::supported`選擇加入時（對話方塊保持開啟），**Alt** 將取消變為 **重設**。選擇加入標記：`__guideLayout`、`__fill`、`__variables`、`__colorPicker`、`__sizing`、`__adjust`（所有類型）、`__filter`、`__adjust`（所有類型）、`__filter`、XZPH252525251ZZX、XZX25252525251ZZX、XZ。每個模組實作自己的`reset`；將`"__altReset": false`設定為停用。

### 畫布和預覽

19. **濾鏡/調整的即時預覽**使用`filter_preview`/`adjust_preview`；`cancel()`清除預覽和相關 shell 狀態。
20. **對話方塊下方畫布** 尊重對話方塊矩形並開啟彈出視窗；僅在自由畫布上使用手/空間/中鍵平移。

### 標籤和 i18n

21. **標題：**指令對話方塊使用`__label`（編目；尾隨`…`在鍍鉻中移除）；固定種類使用`dialogs::title`+`tl!`。
22. **使用者可見字串：**`tl!("…")`；盡可能使用選單目錄中的指令標籤。

### 確認/取消安全

23. **子對話方塊優先**—如果巢狀顏色選擇器打開，則拒絕父對話方塊確認；取消父項在實作時關閉子專案。
24. **當參數不一致時（`guide_layout_ui::validate`in`confirm`），**關閉前驗證**。
25. **當所需輸入缺失時，停用「確定」（`crop_presets::can_confirm`，未更改時套用首選項）。
26. **优先于控制：** 整个对话框 OK/Apply 可能会被拒绝；通过`prefs.set`实现安全的单一首选项（请参阅控制协议）。

### 測試

27. **egui_kittest：** 打开对话框、`run_steps`、按键、断言`fields`或辅助功能角色（`field_tab`测试是参考工具）。
28. **製表符順序回歸：** 當對話方塊獲得多個文字/數字欄位時新增測試。
29. **過濾器預覽：**`dialogs.rs`測試跨後台作業的預覽持久性。

### 反模式

30.`body`/`confirm`中不會因錯誤輸入（狀態或驗證錯誤）而出現恐慌。
31. 無硬編碼顏色或半徑 -`theme::Tokens`和`widgets.rs`。
32. 不要跳過`confirm`調度 — 輸入，`ui.dialog.confirm`使用它。
33. 不要指望在`egui::Modal`上點擊外部關閉。
34. 除非產品明確更改Photoshop奇偶校驗，否則請勿註冊`field_tab`的滑桿或下拉式選單。

## 新增指令對話方塊（清單）

1. 選擇路徑：請參閱**指令路由**（`adjust_dialog`、`filter_dialog`、`__form`、`__sizing`，...，或具有唯一標記的新`*_ui`）。
2.`open`：建構`fields`（`__command`、`__label`、參數、標記）；`app.ui.open_dialog(DialogKind::Command, f)`。
3.`body(app, ui, &mut fields)`－僅突變`fields`；使用共享小工具。
4.`confirm`/`cancel`臂位於`dialogs.rs`中（或委託給模組`confirm`）。
5. Tab：註冊每個文字/數字欄位；盡可能使用`value_field`/`number_edit`。
6. **Alt+Reset（選用）：** 在模組上實作`reset`，並在`dialog_alt_reset`中加入手臂（與`dialogs::show`的調度順序相同）。重置時，`__*Orig`鍵下的快照開啟值應恢復「開啟狀態」而不是出廠預設值。
7. 測試：如果重要的話，至少打開+確認或Tab序列；如果支援 Alt+Reset，則斷言 Reset 將恢復欄位並保持對話方塊開啟。
8. 如果選單可見：指令已在引擎中；選單 ID 透過現有選單接線開啟對話方塊。

## 已知差距

- **普通`TextEdit`的標籤註冊** 是手動的；通用`__form`字串欄位可能無法連接`field_tab`，除非已連線。
- **「對話方塊模組」** 是從`dialogs::show`路由的任何 UI，而不僅僅是名為`*dialog*`的檔案。
- **數字欄位奇偶校驗**（表達式中的單位、滑動標籤）在 [`ui-parity.md`](ui-parity.md)（導航/數字欄位）中進行跟踪，此處不重複。

## 修訂歷史

|日期 |改變|
|---|---|
| 2026年10月11日 |對話方塊程式碼庫審計的初始文件 |
| 2026年10月11日 |記錄所有`DialogKind`值與`Command`路由標記 |
| 2026年10月11日 | Alt+取消 → 重置集線器 (`dialog_alt_reset`)；選擇加入標記 |
