# 控制協議

形狀筆畫預設值可透過 `ui.set {shapeStroke: {width,align,cap,join,
miterLimit、破折號、dashOffset、不透明度}}`；部分欄位合併，null 停用筆畫。
數值使用現有的`shape.edit`名稱和單位。顏色使用`tools.setColors`。
`ui.inspect`公開`toolOptions.shape_stroke`、`stroke_width`和`strokeEditor`。
筆畫選項立即套用有效的 UI 編輯；`strokeEditor`是開啟編輯器狀態，
不是未提交的文件預覽。轉義/外部按一下將其關閉而不恢復編輯。
有關限制、預覽和持久行為，請參閱[形狀筆畫選項](shape-strokes.md)。

桌面應用程式偵聽`127.0.0.1:<port>`（僅環回）。使用令牌檔案啟動它，以便憑證不會在處理程序命令行中公開：

```sh
photocraft --control 7878 --control-token-file /private/path/photocraft-control.token \
  --automation-read-root /work/project --automation-write-root /work/project
```

如果該檔案不存在，PhotoCraft將使用新的 256 位元令牌建立該檔案。在 Unix 上，新檔案的模式為`0600`；在 Windows 上，使用適當的僅限使用者的 ACL 對其進行保護。重複使用現有文件。如果既沒有設定令牌也沒有設定令牌文件，PhotoCraft會為該啟動產生一個令牌並將其寫入標準錯誤。`PHOTOCRAFT_CONTROL_TOKEN`和`PHOTOCRAFT_CONTROL_TOKEN_FILE`是環境變數的等效項。

每個 TCP 連線上的第一個請求必須進行身份驗證。在此成功之前不會調度其他方法：

```json
{"id": "auth", "method": "auth", "params": {"token": "<64 hexadecimal characters>"}}
```

每個連線驗證一次。同一連線上的後續`auth`訊框取得`{"ok": false, "error": "already authenticated"}`，並且連線繼續提供服務。

身份驗證成功回覆後，每個請求都是一行 JSON：

```json
{"id": 1, "method": "ui.inspect", "params": {}}
```

每個回應都是一個 JSON 行，具有相同的`id`：

```json
{"id": 1, "ok": true, "result": { ... }}
{"id": 2, "ok": false, "error": "unknown tool `foo`"}
```

桌面伺服器等待答覆最多 60 秒。在截止日期前仍在排隊的請求在發送前被拒絕；超時不會取消已經開始的工作。兩個超時的讀法不同，都以`timeout`開頭：從未調度的請求（仍在截止日期前排隊）表示`timeout: not run`並且可以安全地重新發送，而`timeout: no reply in 60s`表示命令已調度並且可能仍在執行並仍然適用，因此在重新發送之前請檢查文件。逾時錯誤指出了可能的原因：命令可能仍在執行，或者在視窗未繪製幀時請求已排隊 - 在 Wayland 上，幀更新（以及隨之而來的控制漏）僅在合成器幀回調上執行，因此睡眠狀態的顯示器或完全遮擋的視窗會使每個請求超時，而應用程式本身是健康的。

傳輸器為`apps/photocraft/src/control_server.rs`，處理程序位於`crates/ui-egui/src/control.rs`。 MCP 伺服器（`photocraft-climcp --bridge 127.0.0.1:<port>`，板條箱`photocraft-automation`）包裝了相同的協定。請參閱下面的[MCP 橋](#mcp-bridge)。

＃＃ 方法

-`engine.execute {command, params}`：透過 id 執行任何引擎或 UI 指令。引擎命令直接使用預設參數執行，並且從不開啟對話方塊。使用`ui.menu.invoke`進行選單點擊行為，這會在未給出參數時開啟命令對話方塊。`params`必須是 JSON 物件（省略它或傳遞`null`為空）；在每個傳輸（控制色版、`serve`、MCP 和 CLI）中，陣列、字串、數字或布林值是命名命令的錯誤
-`engine.commands`：列出具有啟用功能的指令；`result`是指令物件的 JSON 陣列（請參閱[指令清單回覆](#command-list-replies)）
-`ui.inspect`：完整的 UI 狀態（工具、面板、檢視、對話方塊、視窗、視窗大小）。不包括選單樹；使用`ui.menu.list`。`brushPicker`關閉時為`null`，筆刷預設選取器開啟時為`{pos: [x, y], size: [w, h]?, list: {collapsed, filter, renaming, showName, showStroke, showTip, scale}}`（以螢幕點為座標；`size`是拖曳其角夾點後選取器的內容大小，在此之前為`null`，並且拖曳選取在至少有一個始終打開；`scale`是頁腳滑塊，0.15–2 乘以 1 標準，將卡片的寬度和高度相乘 — 高度停止在一半大小，名稱和提示大小文本保持其 1.0 大小 — 並將提示的大小數字降低到 0.40 及以下；重新啟動時會記住部件和比例，而正在進行的重命名組和比例，而正在進行搜索時將不會重命名）。`view`包含視圖/視窗/型別首選項：`screen_mode`、`extras`、`show`和`snap_to`標誌、`flip_horizontal`、XZZPH52›ZX（版面 像素 比排列）、寬版面尺寸。`perf.timings.gpuInfo`包含圖形適配器、後端、驅動程式、啟動時選擇的後端及其原因、畫布渲染器 (`gpu`/`cpu`)，以及裝置遺失後的`lost`（`help.systemInfo`傳回與 XZZPH58ZXZZXZXZXZXZXZXZXZXZXZ）返回與 XZZ 的值相同）
-`ui.set {tool?, panels?, dockTabs?, dock?, dockWidth?, colorPanel?, maskTarget?, vectorMaskTarget?, selectionMode?, zoom?, center?, rotation?, fit?, theme?, brushSize?, brushSection?, brushTab?, brushesView?, brushPicker?, brushPickerName?, brushPickerStroke?, brushPickerTip?, brushPickerScale?, gradientBlendMode?, gradientClassic?, shapeStroke?}`：更改UI狀態；任何其他欄位都是錯誤，在任何更改之前進行檢查（`zoom`是畫布縮放，其中 1 = 100%，每個物理顯示像素一個文件像素，因此縮放顯示不會放大圖像；`theme`是`pro`、`proMedium`、`studio`、`studioLight`或`classic`;`selectionMode`是選擇工具的選項列模式，0 New、1 Add、2 Subtract、3 Intersectect；XZZPH69ZZZ = 筆筆刷筆刷；`brushesView`是筆刷的`list`或`grid``brushPickerName`、`brushPickerStroke`和`brushPickerTip`是筆刷預設選擇器的卡片零件（布林值，至少一個），`brushPickerTip`是筆刷預設選擇器的卡片零件（布林值，至少一個），`brushPickerScale`是其102121212121 月 3032133張刻度的卡片。為標準）；`brushPicker`是螢幕點中的`[x, y]`，用於開啟筆刷預設選擇器（如使用繪畫工具右鍵單擊即可）或`null`將其關閉，然後在其中選擇預設為`tools.setBrush {"preset": name}`；`dock`為`{order: ["layers", …], heights: {"properties": 180}, collapsed: ["color"], tabOrder: {"layers": ["Paths", "Layers", "Channels"]}}`，右側停靠組從上到下，其高度以磅為單位，組折疊到其選項卡條，每個組的選項卡順序按面板名稱從左到右（未列出的組保持其預設順序，忽略未知名稱，且後面未列出選項卡）`dockTabs`保持索引卡的預設順序）；XZPHZ984ZZX 是位停靠圖Z（ZZ4Z）的預設順序）；XZPHZ13851256Z（ZPHZFXZ）位的預設順序）；XZPHZPHZ（ZZ4Z4）11251399999999999位的預設）；`{background}`，無論「顏色」面板是否編輯背景顏色；`maskTarget`會與Photoshop一樣，進行交換為遮罩保留的前景/背景對（白色/黑色，直到選擇一個），並在再次定位像素時將像素對交換回來）
  -`typeTransform`是臨時 Ctrl/Cmd 類型工具拖曳之外的`null`。在拖曳過程中，它報告其來源文件/修訂版、原始仿射、定向`frame`（圖層、局部邊界、文件空間角點和樞軸）並捕獲`gesture`。在`down`處用`ui.pointer`和`command: true`駕駛它；`move`/`up`可能會釋放調節劑。文件/歷史記錄在預覽期間保持不變，並且`up`在 XZZPH100​​ZZX 的會話中應用一個`type.edit`。在拖曳過程中按 Esc 鍵只會取消預覽。該指標狀態永遠不會從已儲存的 UI 狀態復原。
-`ui.menu.invoke {id, params?, wait?}`/`ui.menu.list`：透過id啟動選單專案；列出選單樹。啟動背景作業的選單專案（沒有對話方塊的篩選器，例如「模糊更多」）會在套用作業後回覆作業的結果；對於`"wait": false`，立即回覆`{job, pending: true}`。視窗以`window.document.<index>`開啟的文件結束（選項卡順序，選取活動文件，#2991）；呼叫一個將該文件帶到前面並回覆`{document: index}`
-`ui.set`也接受漸層工具選項列的`gradientBlendMode`（圖層混合模式名稱，例如`Difference`）和`gradientClassic`（布林值）。
-`ui.set`也接受`camTransformOnDrop`（布林值），內容感知移動工具的「放置時變換」複選框：放置然後在補丁上放置縮放/旋轉框（`ui.inspect`›`camTransform`：其文件/修訂版，拖曳 XZZZ/PH`frame`），驅動為`ui.pointer`（按自由變換處理；在框外旋轉），與`ui.key Enter`一起提交（一個`paint.contentAwareMove`與`scaleX`、`scaleY`、XZPHZ1ZZX、XZPHZ1ZZX、XZPHZ1ZZX、ZZ11ZZX、ZZ）1
-`ui.set`也接受`tileGrid`（布林值），這是活動層儲存圖塊的開發人員覆蓋：文件上的圖塊網格、填滿並標記為`tx,ty`的每個儲存圖塊、具有共享顏色的相同像素的圖塊和藍色的獨特像素的圖塊。它沒有選單項，並且查看 › 附加功能不會隱藏它。`ui.inspect`將其報告為`view.tile_grid`。
-`ui.set`也接受`strokeTrace`（`true`、`false`、`"stderr"`或`"off"`）：在執行階段開啟或關閉筆劃延遲追蹤；每個筆劃中的摘要圖位於 XZZPHZZ3135XZ。僅在啟動時設定每幀的 CSV 檔案（`PHOTOCRAFT_STROKE_TRACE=<file.csv>`/`--stroke-trace <file.csv>`）：控制色版不會在自動寫入根之外寫入文件
-`ui.set`也接受`layerFilter`(#5048)，圖層面板過濾器行：`{by?, kinds?, name?, mode?}`，任何子集。`by`是`kind`、`name`或`mode`（濾鏡類型下拉選單）；`kinds`列出了開啟的種類切換（`pixel`、`adjustment`、`type`、`shape`、`smart`）；`name`匹配圖層名稱的一部分，忽略大小寫；是混合模式名稱（`Multiply`、`Pass Through`）。僅在開啟類型或鍵入文字後才進行類型和名稱過濾；模式總是先按「正常」進行過濾，如Photoshop中。省略的欄位保留其價值；壞的就是一個錯誤，不會改變任何事情。
- 每個繪畫工具都保留自己的筆刷，如Photoshop中：`tools.setBrush`和`ui.set {brushSize}`更改活動工具的筆刷，並在該工具自己的開關工具中交換（大小、硬度、模式、不透明度、流量、動態、平滑）。第一次使用的工具是從手中的筆刷開始的。顏色、擦除標誌和筆劃種子保持共享。因此，`ui.set {tool: "eraser", brushSize: 12}`設定橡皮擦的尺寸，然後`ui.set {tool: "brush"}`返回筆刷本身的尺寸。
-`ui.set`也接受吸管選項列 (#1649)：`eyedropperSampleSize`（`"point"`或 1、3、5、11、31、51、101：平均繪畫的一側，由吸管、工具的 Alt-click吸管、曲線/顏色選擇器吸管和資訊共享面板）、`eyedropperSample`（`current`、`currentAndBelow`、`all`、`allNoAdjustments`、`currentAndBelowNoAdjustments`）和`eyedropperRing`（布林值，採樣環）。`ui.inspect`將其報告為`toolOptions`（`eyedropper_size`、`eyedropper_sample`、`eyedropper_ring`）。引擎指令`document.sampleColor {x, y, size, sampleLayer}`傳回樣本選取的顏色。
-`ui.set`也接受裁切工具的疊加選單 (#1919)：`cropOverlay`(`thirds`、`grid`、`diagonal`、`triangle`、XZZ、`diagonal`、`triangle`、XZZ、185Z （`auto`：只有當裁剪框被拖曳時，Photoshop的預設值；`always`；`never`）和`cropOverlayOrientation`（0..3；三角形有兩個方向，金色螺旋有四個方向，其他忽略它）。`ui.inspect`將其報告為`toolOptions`（`crop_overlay`、`crop_overlay_show`、`crop_overlay_orientation`）。在應用程式中，O 循環疊加並 ⇧O 其方向，同時裁剪工具顯示一個框。
-`ui.set`也接受`cropFill`（`background`、Photoshop的背景（預設）或`contentAware`）、裁切工具的填滿下拉式選單 (#3823)：使用`contentAware`）、裁切工具的填滿下拉式選單 (#3823)：使用`contentAware`，將轉送或旋轉的框架將傳遞至`contentAware`，將旋轉或延伸至`contentAware`ZPHZPH202ZXZPHZPH202ZXZPHZPHZPH2023XZ，或旋轉的框架將傳遞至`contentAware`ZPHZPH202ZXZPHZPHZ202ZXZPHZPHZ2023XZ，將連接至`contentAware`ZPHZPHZ2023XZPHZPHZ202ZXZPHZPHZ或：`image.crop`，填滿影像周圍未覆蓋的區域。`ui.inspect`將其報告為`toolOptions.crop_content_aware`。
兩種漸層模式均使用目前預設，包括其不透明度停止點。選擇`Foreground to Transparent`到`gradient.presets.select`，或傳遞自訂`stops`和`transparency: [[0,100],[1,0]]`（位置 0..1，不透明度 0..100）。使用`applyToLayer: false`僅編輯工具預設。經典漸層拖曳調度`paint.gradient`而不覆蓋它們。
-`ui.set`还接受裁剪工具的齿轮菜单 (#1919) 作为`cropShield`，一个修补`show_cropped_area`（显示裁剪区域；关闭隐藏画布颜色后面框外的所有内容，应用程序中的 H）、`enabled`（启用裁剪屏蔽）、`color`的物件（`matchCanvas`、黏貼簿顏色或`custom`）、`custom_color`（`[r, g, b]`，0..255）、`opacity`（0..100，預設 75）、XZPH241223ZZX（0..100，預設 75）、XZPH24424：ZZXZ（ZZXZ）時，`classic_mode`（使用經典模式，預設為關閉：框轉動並在圖像上移動；關閉是Photoshop的預設模式，其中框保持直立，視圖旋轉和平移其後面的圖像，同時裁剪工具顯示框架，應用程式中的P；在兩種模式下，選擇該工具都會使旋轉視圖繞其中心直立，經典模式使相機保持直立，繪製的框架轉動，並且視圖在提交或取消後保持直立；經典模式將“顯示裁剪區域”灰顯，然後打開，然後自動居中預覽）和`auto_center_preview`（預設打開：在預設模式下，正在調整大小的螢幕框在其中心繪製會使其重新居中；移動永遠不會使框重新居中）。`ui.inspect`將其報告為`toolOptions.crop_shield`。掛起的幀（`crop_rect`、`crop_angle`：輪到影像）和提交執行的`image.crop`在兩種模式下都是相同的。
-`ui.dialog.open {kind, fields?}`（類型`newDocument`、`about`、`layerStyle {effect?}`、`colorPicker {target: foreground|background}`、`command {command}`、`newCropPreset`/`deleteCropPreset`ZPH239ZXZXZPH239ZXZZPHZ`selected: [names]`；裁切預設是`crop`工具預設，因此`tool.presets.new/list/edit {"tool":"crop"}`也驅動它們）/`ui.dialog.set {dialog, field, value}`/`ui.dialog.confirm {dialog, wait?}`/`ui.dialog.cancel {dialog}`。與`ui.menu.invoke`一樣，`ui.dialog.confirm`等待其指令啟動的背景作業（過濾器對話方塊正常）並回覆結果；`"wait": false`立即回覆`{job, pending: true}`
- 圖層樣式顏色欄位可開啟與`ui.dialog.open {"kind":"colorPicker","parent":12,"effect":"fx2","field":"color"}`的共用選取器。`parent`是圖層樣式對話方塊 ID；`effect`是其效果實例 ID。`field`必須是此實例的色彩參數之一（`color`或用於漸層疊加的`from`/`to`）。選擇器預覽待處理的樣式，而不更改文件。 Picker OK 僅更改父參數；取消會放棄預覽。之後確認父級以套用該樣式。當其選擇器開啟時，家長確認被拒絕；父級取消會關閉兩者。此結合在`ui.inspect`中公開為`__layerStyleColor`；更改/關閉目標或文件切換使其無效。
  -`ui.dialog.open {kind:"colorPicker", textStyle:{kind:"character"|"paragraph", id:u32}}`開啟共用顏色選擇器，以在樣式選項中明確啟用顏色屬性。驗證後開啟匹配樣式面板/編輯器；必須先透過現有樣式指令啟用繼承的顏色。選擇器在`ui.inspect`中的`__textStyleColor`下報告其綁定文件/修訂/樣式。變更獨立文件中的預覽； 「確定」執行現有樣式`set`指令一次，「取消」不會變更任何內容。文件編輯/切換或關閉/更改樣式選項會丟棄選擇器。`textStyle`不能與`parent`、`effect`、`field`或`target`組合使用。
  - shell 自己的對話方塊（視窗 › 工作區 › 新工作區/刪除工作區、檢視 › 像素長寬比 › 自訂像素長寬比…、檢視 › 顯示 › 顯示附加選項…、檢視 › 32 位元預覽選項…）列在`ui.inspect`的 XZZPHZZX 中開啟一個10ZZXZ（ZPHZXZ）中最多 10ZXZ（ZPHZXZ）中開啟一個1ZXZ（ZPHZXZ）中，最多 10ZXZ（ZPHZXZXZ）中開啟一個1370ZXZ（ZPHZXZ）中，最多為 XZXZ（ZPHZXZ）中開啟一個10ZXZ（ZPHZXZXZ）中最多 10ZXZ（ZPHZXZ）中開啟一個10ZXZ（ZPHZXZ）中最多 10ZXZ。`ui.dialog.set`、`ui.dialog.confirm`和`ui.dialog.cancel`取`{"dialog": "shell"}`； OK 執行對話方塊的指令並在失敗時保持對話方塊打開
  - 未儲存的變更提示（關閉文件、恢復、退出未儲存的工作）在`ui.inspect`的`dialogs`中列出，ID 為`"discard"`，當它啟動時：`fields`給出等待的 XZZPH280Z2（XZPHZZ、ZXZ、ZPHZ281Z），它詢問。`document`，`remaining`有多少，以及它的`choices`。`ui.dialog.confirm`接受`{"dialog": "discard", "choice": "save" | "dontSave" | "cancel"}`（預設`save`；恢復接受`dontSave`或`cancel`）並像視窗按鈕一樣回答它，一次一個文件；`ui.dialog.cancel`與`{"dialog": "discard"}`取消它。儲存無標題文件首先會開啟儲存對話方塊，因此回覆的`fileDialog`為 true，並且提示將保持開啟狀態，直到得到答案。
- 圖層樣式（`layerStyle {effect?}`在該效果類型上開啟）：欄位保存對話方塊的整個狀態，因此代理可以像使用者一樣讀取和驅動它。`effects`將圖層順序中的每個效果實例列為`{id, kind, on, params, fx}`（`fx`是對話方塊載入的效果快照；「確定」使用`params`編輯該快照，因此對話方塊未建模的任何內容都會保留）。`selected`選擇頁（`blendingOptions`或實例 ID），`preview`（預設開啟）控制即時畫布預覽，`p:blendingOptions`編輯混合模式/不透明度/填滿不透明度，`patternList`命名可用模式。 OK 一步替換圖層效果（`layer.layerStyle.replace`）；取消丟棄。 「新樣式...」透過`style.presets.new {effects}`儲存待處理狀態，點選色板即可套用樣式預設，並使用`layer.layerStyle.makeDefault`/`layer.layerStyle.defaultFor`設為/重設預設值
  - 首選項 (`ui.menu.invoke {id: "edit.preferences.interface"}`) 顯示其`values`欄位中的部分。在首選項上套用 (`ui.dialog.apply`) 和確定 (`ui.dialog.confirm`) 會因控製而被拒絕，因為自動化拒絕整體首選項更新和檔案系統承載首選項部分。拒絕的`ui.dialog.apply`傳回錯誤並保持對話方塊開啟；拒絕`ui.dialog.confirm`回傳錯誤，但關閉並刪除對話方塊。`ui.dialog.cancel`關閉對話方塊而不儲存。可使用`prefs.set`（例如`{"command": "prefs.set", "params": {"path": "interface.showTooltips", "value": false}}`）透過`engine.execute`變更個人安全首選項。標記為下次啟動的設定仍需要重新啟動。
  - 編輯 › 填滿…（`ui.menu.invoke {id: "edit.fill"}`、Shift+F5、Shift+Backspace）開啟「填滿」對話方塊；它的欄位是`edit.fill`的參數（`contents`、`color`、`pattern`、`colorAdaptation`、`mode`、`opacity`、`preserveTransparency`），它們會在首選項中記住它們（ZZXZ）。
  - 儲存為網頁 (`ui.menu.invoke {id: "file.export.saveForWebLegacy"}`) 可透過獨立捲動設定調整大小。對話方塊欄位`__view`(`original|optimized|2up|4up`)、`__zoom`（`null`= 適合，否則為係數；1 = 物理顯示像素中的 100%）和`__center: [u,v]`控制共用預覽導航。`__previewRects`報告螢幕點中的窗格矩形。拖曳預覽進行平移，滾動/捏合以在指針處縮放，或雙擊以適合。 GIF/PNG-8 公開`colorTable: ["#rrggbb", …]`（不透明來源條目）和`colorTableEdits: {"0":"#rrggbb", …}`（量化後從零開始的替換）。樣本打開一個顏色選擇器；編輯更新預覽和保存的文件，無需重新映射像素索引或更改文件。重置清除替換。變更調色板/大小/遮罩/透明度設定會重新產生表格並清除基於索引的編輯。預覽和產生的表格使用有界的 2048 像素代理；全尺寸匯出使用未更改的明確表。`__paletteApproximate`報告代理派生的調色板生成，並標記放大的代理預覽。引擎匯出命令接受相同的表格/編輯參數；其無路徑估計返回來源`colorTable`和`transparentIndex`以及大小統計資料。開啟新對話方塊不會攜帶先前圖像的表格。
- 在類型、形狀、智慧型物件或填滿圖層上按下像素工具（例如透過`ui.pointer`）會開啟「柵格化？」提示而不是繪畫：具有`__rasterize`(`type|shape|smartObject|fill`)、`message`、XZZPH344ZXZZ的對話方塊。`ui.dialog.confirm`執行`layer.rasterize.*`指令，然後在`at`處進行繪製（兩個歷史狀態，`{"rasterized", "painted"}`）；`ui.dialog.cancel`什麼都不做。
  - 編輯 › 變換 › 可編輯類型上的變形（`ui.menu.invoke {"id":"edit.transform.warp"}`，或打開的變形框的變形開關）會開啟變形模式，而不會出現提示，如Photoshop所示：無法拖曳或分割網格，僅套用選項列的變形樣式、彎曲、水平和垂直，並且提交可設定圖層程式設計圖層的變形樣式。直接：`engine.execute {"command":"edit.transform.warp","params":{"style":"flag","bend":40}}`在類型層上。對於類型，扭曲和透視保持灰色。
-`ui.window.open {document?}`/`ui.window.close {window}`：額外的文件窗口
-`ui.pointer {events: [{kind: down|move|up, x, y, pressure?, tiltX?, tiltY?, rotation?}], tool?, modifiers?, button?}`：在文件座標中驅動活動工具（壓力0..1，傾斜度-90..90，桶旋轉0..360：類比筆）。修飾符標誌（`shift`、`alt`、`command`、`ctrl`、`space`）要麼位於頂層，要麼分組在`modifiers`下；當`modifiers`存在時，頂級標誌將存在時，頂級標誌將存在。`space: true`持有空間（裁剪框、選框、套索或繪製的形狀然後移動而不是增長）。當「裁切」工具處於預設模式（關閉「使用經典模式」）時，在框內拖曳可移動影像，在框外拖曳則可旋轉影像，視圖也會隨之平移和旋轉；當視圖在其`down`處顯示文件時，會讀取拖曳的`move`/XZZPH368ZZZZZPHZZZX/XZZ`up`不會再有任何變化。`command: true`使用繪畫、修飾、橡皮擦、漸層、滴管或其他非選擇工具使手勢成為移動工具，就像按住 ⌘（Mac 上為 Ctrl）在應用程式中所做的那樣：拖曳移動活動圖層（存在選擇時的選定像素），然後使用`alt: true`也沒有拾取圖層，然後使用`alt: true`也沒有拾取圖層的工具。手形、旋轉視圖、縮放、裁剪、切片、路徑選擇、鋼筆、形狀、類型和選擇工具保留了`command`的功能。`ctrl: true`與`alt: true`和筆刷工具是畫布上筆刷調整大小（在應用程式中，按住 Ctrl+Alt 並從 Mac 上拖曳）。`button: "right"`（或`"secondary"`）是右鍵：使用「移動」工具（或使用任何工具的`command: true`）開啟畫布圖層選單（`ui.inspect`中的`layerMenu`列出那裡的圖層，首先是最上面的； 选择一个）；使用选取框、套索、魔棒、对象选择或笔，它会打开一个工具上下文菜单（`ui.inspect`中的`canvasToolMenu`，带有有序命令 ID、分隔符和启用状态）；使用“缩放”或“手动”，它会打开那里的视图菜单（围绕按下的点进行“放大”和“缩小”步骤；`view.fillScreen`是其「填滿螢幕」行）；使用繪畫工具，它會開啟筆刷預設選取器，或使用首選項 › 工具 › 使用繪畫工具右鍵點擊 = 擦除。選單在按下的文件點的螢幕位置處開啟。使用`ui.context.choose {id}`來啟用工具上下文操作。筆選單記錄在`docs/context-menu-parity.md`中。`button`是`left`（預設）、`right`、`secondary`或`middle`，事件的`kind`是`down`、`move`Z（預設）或XZZ（預設）使用顏色選擇器作為頂部對話方塊，`down`和`move`將圖像採樣為新顏色（其吸管，就像在畫布上單擊一樣）並且不驅動該工具。`down`/`up`位於同一點，使用矩形、橢圓形、三角形或多邊形工具開啟建立矩形/橢圓形/三角形/多邊形（Photoshop的點選調整大小對話方塊）；使用`ui.dialog.set`設定其 XZZPH401ZZZ 和`height`（文件像素）、`fromCenter`、`radii`[tl, tr, br, bl]（矩形）或`sides`和`starRatio`（百分比、多邊形），然後設定 XZZPH408ZZ4
- 磁性套索 (`tool: "magneticLasso"`)：一對`down`/`up`即可點擊。第一個設定第一個緊固點（其修飾符設定選擇模式）；稍後按一下固定一個點，或關閉第一個點上的邊框。`move`事件在有或沒有按住按鈕的情況下追蹤邊界，自行緊固點；對於`alt: true`，點擊可繪製直線段，拖曳可徒手繪製直線段。`ui.inspect`顯示`magnetic`（`path`、`anchors`、`live`、`mode`）下正在進行的邊框。`ui.key`Enter 沿著邊緣關閉它，Backspace 刪除最後一個緊固點，Escape 取消。若要在一次呼叫中沿邊緣進行選擇，請使用`select.magneticLasso`指令以及形狀周圍的粗糙點。
  - 色階或曲線調整圖層，屬性滴管配備（`ui.inspect`中的`layerPicker`：`{layer, point, tool}`，透過面板的設定黑色/中性灰色/白色點按鈕設定）：`down`將圖層下方直接發送影像中的灰色/白色點按鈕設定）：`down`將圖層下方直接發送影像中的該點指令設定為一個 XZZZPH427）（CPH427）並且未將該工具Esc、另一個工具或另一個層可以解除它。
-`ui.click {x, y, button?, count?}`/`ui.move {x, y}`：螢幕點中的合成指針輸入（`button`：`left`（預設）、`right`（或`secondary`）或`middle`）。`count`（預設為1，雙擊為`2`）最多為256；較大的誤差是錯誤且不會排隊
-`ui.key {key, command?, shift?, alt?, ctrl?}`（標誌也可以分組在`modifiers`下）：按下並釋放一個按鍵，例如`{"key": "ArrowLeft", "shift": true}`。`{"key": "AltLeft"}`是按 Alt，將鍵盤置於視窗內選單列上（然後`ArrowRight`、`ArrowDown`、`Enter`、`Escape`或選單字母驅動它）；`{"key": "F", "alt": true}`使用此助記符開啟選單 (#4988) (#4988)；
-`ui.pointer`、`ui.click`、`ui.move`、`ui.key`和`ui.screenshot`在操作之前驗證每個參數，如`ui.set`所做的那樣：未知字段（頂級、XZZPH452ZZ類型的值（`"x": "abc"`、`"ctrl": "yes"`）、`button`、`kind`、`tool`或`key`超出接受的集合，或超出其範圍（`pressure`、傾斜、`rotation`、`count`）的數字是錯誤命名參數，並且不會排隊或應用任何內容。`null`欄位被視為省略並採用預設值
- 引擎指令（`engine.execute`、`photocraft-clirun`、MCP`command_run`）拒絕其`params`文件未為`STRICT`（`crates/engine/src/param_check.rs`、今天的指令 XZZPH470ZXZ 和XZPHZ41`{"ids":[4]}`是錯誤`unknown parameter \`ids\` (expected: layer)`，並且沒有任何運作。其他命令仍然會忽略未知密鑰，直到其`params`文件完成並將它們添加到該清單中（`coalesce`和`target`始終被接受）。
-`ui.type {text}`：鍵入文字（前往對焦的小部件，或在文字工具編輯時前往畫布）
-`ui.resize {width, height}`：調整主視窗大小
-`ui.gpu.simulateLoss {error?}`：就像wgpu裝置遺失一樣（或報告`error: true`錯誤）。該應用程式在會話的其餘部分切換到 CPU 影像合成器，並顯示與實際遺失相同的恢復警告。`ui.inspect`中的`gpuFallbackNotice`包含警告開啟時的原因。 Keep Use CPU 為下次啟動儲存 CPU 相容性；重試 GPU 會儲存 GPU 模式並要求使用者儲存並重新啟動。視窗渲染器仍然需要工作圖形或軟體適配器。返回`wasActive`和`gpuInfo`。用於測試後備
-`ui.screenshot {path?, focus?}`：捕捉主視窗（PNG）。回覆中沒有包含路徑
  Base64 PNG 資料；路徑是相對於自動寫入根的。首先升起車窗
  （預設）因為被遮蔽的 macOS 視窗停止渲染
-`ui.focus`：將主視窗置於前面
-`app.open {path}`/`app.save {path?, tgaBits?}`：透過設定的自動化根的相對檔案 I/O（`app.open`在讀根下讀取，`app.save`在寫入根下寫入；絕對路徑、`..`和轉義根的路徑都被拒絕根拒絕時兩者都以`warnings`回應（匯入/匯出註釋，例如「調整圖層展平」；沒有時為`[]`），也會在狀態列中顯示給使用者並作為通知（`ui.inspect`中的`notices`）還返回文件還返回文件中的 XZZZZ498ZZ；`name`（目錄`.pcraft`— 本機磁碟形式 — 透過讀取根的功能載入；單一檔案 (zip) 形式採用位元組路徑）、`app.save`和寫入的`path`。沒有`path`的`app.save`僅寫回文件自己的 PSD、PSB 或`.pcraft`文件，如文件 › 保存。`tgaBits`（24 或 32）設定 TGA 每像素的位數：32 將 Alpha 色版（或透明度）寫入其 Alpha，24 無；如果沒有它，則為 32，帶有 Alpha 色版或透明度，否則為 24。自動化保存從不顯示 TGA 或 TIFF 選項。自動化開啟並儲存永不觸發指令碼事件。使用這兩個而不是控制色版拒絕的`file.open`、`file.save`、`file.saveAs`或`file.saveACopy`（請參閱[引擎指令](#engine-commands)）
-`app.quit`

## 引擎指令

使用`prefs.set`更改 UI 語言而無需重新啟動：
`{"path":"interface.language","value":"fr"}`。支援的程式碼和預覽/應用行為
記錄在 [UI 本地化](localization.md) 中。`prefs.get`與現有偏好
儲存暴露並保留相同的設定；指令碼繼續使用規範的命令 ID。

`engine.execute`透過 id 執行任何指令。`engine.commands`（或引擎指令`command.list`）列出了所有指令，包括標籤、選單路徑、捷徑、參數說明以及每個指令目前是否已啟用（回覆形狀：[指令清單回覆](#command-list-replies)）。每行包含`enabled`(bool)、`disabledReason`（啟用時為`null`，否則為失敗的前提條件，例如`"no document open"`、`"no selection"`或`"the clipboard is empty"`）和 XZPH527 true，因為沒有文件打開，因此第一步是建立或開啟一個；此後命令可能仍然有自己的前提條件，例如由於其他原因啟用或關閉時活動文字圖層為 False）。在沒有開啟文件的情況下，幾乎每個文件指令都會報告`enabled: false, needsDocument: true`；圍繞`needsDocument`進行規劃，而不是單獨閱讀`enabled`(#3664)。範例：

|命令 |參數|
|---|---|
|`file.new`|`{"width":1920,"height":1080,"mode":"rgb","depth":8,"background":"white"}`|
|`layer.new.layer`|`{"name":"Ink"}`|
|`layer.select`|`{"layer":id\|str,"mode":"replace\|toggle\|range\|add\|remove"}`或`{"name":"Background","occurrence":0?}`（出現是從零開始的文件遍歷順序；重複名稱需要）： ⌘-點選 = 切換，⇧-點選 = 範圍；`document.inspect`報告 XZZPH541ZXZX
|`channel.target`/`channel.setVisible`|色版面板目標和眼睛（`"composite"`、顏色名稱、Alpha 索引或名稱、`"quickMask"`）。`document.inspect`（和`channel.list`）報告`channels`：具有可見性的複合/顏色/Alpha 行、色版選項、`quickMask`、`target`。沒有`target`參數的像素指令遵循目標頻道 |
|`layer.setProps`|`{"layer":id?,"name":…,"visible":…,"opacity":0..1,"blend":"Multiply"}`|
|`layer.fill.solidColor.set`|`{"layer":id?,"document":id?,"color":"#rrggbb"}`：在一個撤銷步驟中編輯純色填滿；不含 Alpha 的 RGB 會保留填滿的 Alpha。浮點 RGB 陣列和儲存的`Color`也被接受。`document`拒絕過時的對話方塊目標。 |
|`layer.fill.pattern.set`|`{"layer":id?,"document":id?,"pattern":id\|name?,"scale":1..1000?,"angle":-180..180?,"link":bool?,"phase":[x,y]?}`：在一個撤消步驟中編輯圖案填滿（圖案填滿對話方塊確定）；省略的值保留圖層的值，`phase``[0,0]`是對齊到原點。選單中的`layer.newFillLayer.pattern`（無參數）和圖案填滿圖層上的`layer.layerContentOptions`開啟「圖案填滿」對話方塊。 |
|`layer.layerMask.edit`|`{"layer":id?,"document":id?,"density":0..100?,"feather":0..1000?}`：編輯像素遮罩密度 (%) 和羽化 (px)，而不更改遮罩像素，包括調整遮罩。至少需要一項設定；`document`拒絕過時的目標。共享的`coalesce`鍵將滑桿更新分組為一個撤銷步驟。 |
|`layer.setExpanded`|`{"layer":id?,"expanded":bool?,"all":bool?}`：在「圖層」面板中開啟/關閉群組（無`expanded`：切換；`all`：每組）。與文件一起儲存的視圖變更（PSD 開啟/關閉資料夾），而不是撤銷步驟；`document.inspect`在群組上報告`expanded`|
|`layer.newAdjustmentLayer.hueSaturation`|`{"hue":30,"saturation":10}`|
|`paint.stroke`|`{"points":[[x,y,pressure],…],"size":20,"color":"#ff0000"}`|
|`select.rect`|`{"x":0,"y":0,"width":100,"height":50,"mode":"add","ellipse":false}`|
|`document.inspect`|`{}`：圖層樹、歷史記錄、快照（`[{id, name}]`，適用於`history.selectSnapshot`）、選擇範圍 |
|`layer.list`|`{"document":index?,"flat":bool?}`：僅圖層，由上而下：`id`、`name`、`kind`、`visible`、`bounds`（`[x,y,w,h]`或空）、`bounds`（`[x,y,w,h]`或空組）、XZPHZ59335；`children`，或對於`flat`，每一層都是一行，其中包含其`depth`和`parent`群組 ID |
|`document.pixel`|`{"x":10,"y":10}`：複合RGBA |
|`document.presets.list`|`{}`→`{presets:[{name,settings}]}`：已儲存新文件設定 |
|`document.presets.save`|`{"name":"Product square","settings":{"width":1600,"height":1600,"background":"transparent"}}`：保存快照；名稱被修剪，長度為 1-255 個字符，並且拒絕 ASCII 不區分大小寫的重複項 |
|`document.presets.get`|`{"name":"Product square"}`→`{preset,params}`：取得設定；將`params`傳遞給`file.new`以建立文件，可選擇新增文件`name`|
|`document.presets.delete`|`{"name":"Product square"}`：刪除已命名的已儲存設定 |
|`document.recent.list`|`{}`→`{recent:[settings]}`：文件 › 新建 › 最近，最近使用新建文件對話方塊創建的文件的設置，最新的在前（最多 20 個）|
|`document.recent.add`|`{"settings":{…as document.presets.save}}`：將設定放在「最近」的第一位（對話方塊在「建立」之後執行此操作；相同的條目會折疊）；未記錄到操作中|
|`type.hitTest`|`{"layer":id?,"x":px,"y":px}`：文件點下的字元。`layer`不會選擇最上面的可見文字圖層。結果`{"layer","index","line","inside"}`|
|`type.caret`|`{"layer":id?,"index":char}`：文件像素中的插入符號段，`{"index","line","segment":[[x,y],[x,y]]}`（包括旋轉和垂直類型）|
|`type.navigate`|`{"layer":id?,"index":char,"move":"wordPrev\|wordNext\|linePrev\|lineNext\|lineStart\|lineEnd\|start\|end","x":px?}`：相鄰插入符。`x`保持立柱越線移動。結果`{"index"}`|
|`actions.list`|`{}`→`{actions:[{name, steps}], recording}`（正在記錄的名稱，或為空）|
|`actions.get`|`{"action": name or index}`→`{name, steps:[[id, params], …], disabled:[step, …]}`。`steps`是`file.automate.batch`的形狀和水滴的形狀；`disabled`列出播放跳過的從 0 開始的步驟 |
|`actions.record`|`{"name":"Red"}`開始新操作（預設名稱`Action N`）。`{"action": name or index}`附加到現有的一個 |
|`actions.stop`|`{}`→`{action, steps}`。複製自`actions.record`以來的可重播日記條目（省略查詢和操作編輯命令）|
|`actions.play`|`{"action": name or index, "from": step?}`。`from`和`failed.step`是從 0 開始的。傳回`{action, ran, failed?:{step, id, error}}`，並且當步驟失敗時仍然傳回 ok，因此報告部分執行。每執行一步留下一個歷史記錄步驟。支援最多16級嵌套動作呼叫，拒絕循環。錄製時，播放另一個動作會附加一個名為`actions.play`的步驟，而不是其擴充指令。在不受信任的會話上，每個步驟的授權方式與頂級命令相同|
|`actions.move`|`{"action": name or index, "step": index?, "to": index}`在其動作中移動一個步驟，或省略`step`時移動整個動作。`to`是最終的從零開始的索引。錄製時可以對待處理的步驟重新排序；無效的移動使清單保持不變。 |
|`actions.delete`|`{"action": name or index, "step": index?}`。用零基礎的`step`：`{action, deletedStep, steps}`，也允許同時錄音。沒有`step`：`{deleted}`，錄音時被拒絕|
|`actions.toggle`|`{"action": name or index, "step": index?, "enabled": bool?}`→`{action, disabled}`。開啟或關閉一個基於 0 的步驟（包括待記錄的步驟）；省略`enabled`會翻轉它。如果沒有`step`，則開啟每個步驟：除非所有步驟都已經開啟。播放、批次和 Droplet 會跳過關閉的步驟。在桌面面板中，按一下步驟的複選框 |
|`actions.rename`|`{"action": name or index, "name": str}`→`{action, index}`。空的、僅包含空格或重複的名稱將被拒絕並保留舊名稱。按名稱呼叫操作的巢狀`actions.play`步驟及其功能鍵遵循新名稱。在桌面面板中，雙擊操作名稱以將其重命名（Enter 提交，Esc 取消）|

操作清單填入可調整大小的停靠組。

操作面板在錄製時立即顯示可重播的步驟。點擊一個人
選擇它的步驟，然後使用垃圾桶按鈕刪除該步驟；選擇動作
相反，標題是針對整個動作的。錄音過程中也可以刪除步驟
並且不會撤銷文件編輯。控制等效為
`actions.delete {"action": "Name", "step": 0}`（從零開始）。省略`step`刪除
整個動作，仍然需要停止錄製。

桌面操作還可以記錄和重播視圖 › 適合螢幕 (`view.fitOnScreen`)，
100% (`view.actualPixels`)，透過選單指令或捷徑放大縮小。
選擇一個動作並按下「記錄」以附加步驟；加號按鈕開始新的操作。
當佈局畫布時，最後的`["view.fitOnScreen", {}]`步驟適合調整大小的文件。
無頭播放將不支援的視圖命令報告為失敗的步驟。

在桌面「操作」面板中，右鍵點選一個操作以指派功能鍵 (F1–F12)，
或選擇“無”將其刪除。綁定遵循操作名稱，無論選擇什麼
行，並儲存在首選項中。例如，現有的`edit.keyboardShortcuts`指令
接受`{"set":{"actions.play:My action":"F6"},"allowUnknown":true,"removeConflicts":false}`。
當命名操作存在並綁定時，其優先權高於普通選單快捷方式；
刪除其綁定或刪除它會恢復該選單快捷方式。面板將鑰匙移開
分配它時來自另一個操作。動作回放仍然檢查每個嵌套命令的
自動化權限。

`engine.execute`和`ui.menu.invoke`也接受 UI 層級指令（`view.zoomIn`、`window.theme.pro`、`edit.search`...）。

透過路徑而不是透過自動化根讀取或寫入檔案的命令將被拒絕，並顯示「自動化命令`…`使用環境檔案系統路徑並被禁用；使用功能範圍的文件方法」。這涵蓋了除`file.new`、`file.close*`命令和一些無路徑命令（如`file.fileInfo`）之外的所有`file.*`命令，因此`file.open`、XZZPH690ZX、XZZ、PHZX、XZZ691Z6Z在此始終失敗：開啟並儲存`app.open`/`app.save`。其他指令（`layer.exportAs {path}`、`filter.distort.displace {mapPath}`、預設導入和插件安裝/重新載入）的路徑參數也會以相同的方式被拒絕。`prefs.set`拒絕整體首選項更新和檔案支援部分（`colorSettings`、`scriptEvents`、`historyLog`、`plugIns`和`scratchDisks`），因此自動化無法間接設定環境檔案存取。桌面應用程式也拒絕`image.mode.*`，它可以載入其首選項中設定的顏色設定檔。規則在`crates/automation/src/workspace.rs`中。

嵌入式智慧型物件可以使用`layer.smartObjects.editContents {layer?}`或
用`layer.smartObjects.convertToLayers {layer?}`拆封。這包括Photoshop來源
透過 UUID 儲存在文件的嵌入連結層塊中。需要磁碟讀取的來源
仍然被拒絕，包括在解包期間需要重新渲染的嵌套連結物件。
在開啟的內容文件中，使用`layer.smartObjects.saveContents {}`更新父級
記憶體中，或關閉髒內容文件。對於嵌套對象，從最深處開始保存
內容向外。`app.save`將活動文件匯出到範圍文件；它不是替代品
用於保存內容。編輯完成後，將父文件儲存到磁碟。這些規則
也適用於記錄的操作和合成 UI 輸入；互動式桌面命令保留
他們通常的文件存取。來源替換、重新連結和匯出命令仍然受到限制。

`type.editText`開始對活動文字圖層進行內聯編輯並選擇其所有文本，例如
雙擊圖層中的縮圖。文字輸入、提交和取消使用現有的文字工具
編輯會話；非類型圖層傳回錯誤而不更改工具或文件。

### 命令列表回复

`engine.commands`（控制色版和`serve`），引擎命令`command.list`和MCP
工具`command_list`全部回覆一個裸 JSON 陣列，而不是一個物件：沒有
`{"commands": [...]}`包裝紙。在控制色版和`serve`上，陣列是回應的
`result`(`{"id":1,"ok":true,"result":[…]}`); MCP 工具將其作為 JSON 文字傳回。
每個元素描述一個命令，依註冊表順序：

```json
{"id":"file.new","label":"New…","menu":["File"],"shortcut":"Cmd+N","params":"{\"width\":1..300000=1920,…}","enabled":true,"disabledReason":null,"needsDocument":false}
```

|關鍵|類型 |意義|
|---|---|---|
|`id`|字串|傳遞給`engine.execute`/`command_run`的 id |
|`label`|字串|選單標籤|
|`menu`|字串陣列 |專案上方的選單路徑（`[]`對於沒有選單專案的指令）|
|`shortcut`|字串或`null`|預設快捷方式，例如`"Cmd+Shift+X"`|
|`params`|字串|人類可讀的參數文件（不是 JSON 模式）|
|`enabled`|布林 |目前狀態下指令是否可以運作 |
|`disabledReason`|字串或`null`|為什麼指令現在無法運作（`null`當`enabled`時） |
|`needsDocument`|布林 |`true`由於沒有開啟文件而關閉 (#3664) |
|`keywords`|字串，可選 |該指令的其他名稱可透過 (#5060) 找到，例如`select.rect`的橢圓跑馬燈；僅出現在具有它們的指令上 |

過濾縮小同一陣列而不更改其元素：`serve`的 `engine.commands
{filter}` and the MCP tool's `filter` 保留 id、標籤或關鍵字包含文字的指令
（不區分大小寫），MCP 工具的`enabled_only`保留那些`enabled: true`。的
桌面控制色版的`engine.commands`不採用過濾器，並且始終返回每個命令。
此形狀由`crates/automation/src/rpc.rs`(#3684) 中的測試固定。

### 後台作業 (#210)

長命令（每個`filter.*`、`edit.contentAwareFill`、`edit.contentAwareScale`、`file.automate.photomerge`、`brush.presets.importAbr`）、檔案開啟和互動式儲存（檔案 › 儲存、另存為和 TIFF選項延續，#2017）在桌面應用程式中作為後台作業執行：視窗不斷繪製，狀態列透過「取消」按鈕顯示進度，以及作業鎖定活動文件會顯示模式進度對話方塊（Esc 取消）。`engine.execute`、`ui.menu.invoke`、`ui.dialog.confirm`、`app.save`仍預設等待結果；透過`"wait": false`立即取得`{job, pending: true}`，然後輪詢`jobs.list`立即移除（`{job, pending: true}`，然後輪詢`jobs.list`（XZXZ6ZZX13756ZZX、38371371373337333373日； 0–1；結果或錯誤）並使用`jobs.cancel {job}`停止它。取消或失敗的作業會使文件保持不變。當作業執行時，編輯其文件的命令會失敗，並顯示「...仍在該文件上執行」。`ui.inspect`報告`jobs`（執行作業、開啟檔案、儲存檔案）。設定`PHOTOCRAFT_INLINE_JOBS=1`以內聯執行所有內容。

## 偏好設定

首選項（編輯 > 首選項，像Photoshop的對話方塊部分一樣分組）位於引擎中，因此
相同的命令適用於應用程式、CLI 和無頭 MCP。路徑是點狀駝峰式鍵：
`<section>.<key>`，帶部份`general`、`interface`、`workspace`、`tools`、`historyLog`、
`fileHandling`,`export`,`performance`,`scratchDisks`,`cursors`,`transparencyAndGamut`,
`unitsAndRulers`,`guidesGridAndSlices`,`plugIns`,`type`,`enhancedControls`,`rawDefaults`,
`integrations`，加上`shortcuts.<command id>`、`menus`（`hidden`、`colors.<id>`）、`toolbar`
（`hidden`、`order`）和`colorSettings`（編輯 › 顏色設定）。

|命令 |參數|
|---|---|
|`prefs.get`|`{"path":"performance.historyStates"}`；沒有路徑回傳一切|
|`prefs.set`|`{"path":"cursors.painting","value":"precise"}`或`{"values":{"unitsAndRulers.rulers":"cm","guidesGridAndSlices.gridColor":"#ff8800"}}`。數值經過驗證（選擇、範圍、`#rrggbb`顏色、快速語法）；批次應用全部或全部。節路徑採用物件並按鍵合併 |
|`prefs.reset`|`{"path":"performance"}`（部分或按鍵）；沒有一條道路可以重置一切。單獨的`shortcuts.<id>`或`menus.colors.<id>`重置會刪除覆蓋並返回`null`，包括當它已經不存在時 |
|`edit.preferences.<section>`|該部分的值；在應用程式中（無參數），它會開啟該部分的「首選項」對話方塊 |
|`edit.keyboardShortcuts`|`{"set":{"edit.fill":"Cmd+Shift+F"},"reset":true\|["id",…],"removeConflicts":true,"filter":"blur","list":false,"importKys":"<.kys XML>"}`：返回覆蓋、匹配指令和衝突。移動到另一個命令的捷徑將從其舊所有者中刪除，除非`removeConflicts`為 false。`""`刪除捷徑，`null`恢復預設。保留的臨時工具也是可綁定的：`tools.temporary.hand`（空格；也重新定位正在繪製的選取範圍）、`tools.temporary.zoomIn`（Cmd+Space）、`tools.temporary.zoomOut`（Alt+Space；Alt添加到保留的放大、Cmd+Alt+Space，也縮小）、XZPHZ11``; with the Brush or Pencil, a stroke started while it is held is in Clear mode, `paint.lines`/`paint.pencil` `"模式":"清除"`, erasing with that tool's brush); they list with `"按住": true`. ⌘ (Ctrl off the Mac) held alone is the Move tool with the painting, retouching and other non-selection tools, as inPhotoshop; it is a modifier, not a binding. `importKys` applies aPhotoshop`.kys` set (the file's XML text, at most 4 MiB): rows are matched to commands by our id (`pc-id`, in setsPhotoCraftexported) or else by menu label, a key equal to the default restores it, keys also given in `set` win, and the result gains `import` (`name`, `imedXZZXZZ432ZZXnameXZZPHX33ZZXportedXZZX 5ZZX未知`, `不可讀`, `備用`, `toolKeys`, `windows`). A set saved byPhotoshopon Windows (no ⌘ or Opt anywhere, Ctrl used) reads Ctrl as the primary modifier, so `Ctrl+L` is the `CmdmdyZZPH842ZXCtrl+L` is the `Cmdmd+LXZZPH842ZXZX： true`). Tool keys are matched to tools by `pc-id` or name (a tool's own letter keeps its Shift cycling); `import.tools` lists the keys given. In the app it opens Keyboard Shortcuts and Menus, whose Import Shortcuts… button loads a `.kys` set into the dialog (opening or dropping a `.kysXZZPH847ZZ X鍵盤Shortcuts.psp`, does the same). On a desktop withPhotoshopinstalled, the first launch that has no shortcut overrides yet asks whether to import the newest install's live set (Import / Don't Import; `ui.inspect` shows it as `kysOffer`); the answer is recorded in the preferences as `dialogs.photosho pShortcuts`, so it is asked once. **Sets** (Edit › Keyboard Shortcuts › Set): `useSet` loads a saved set's bindings (`""` or `"PhotoCraft預設值「`: the defaults) before the call's other changes; then `saveS et` saves the bindings as they end up under a name and uses it, `deleteSet` removes one (the bindings stay), `sets` replaces every saved set (`{“<name>”：{“快捷方式”：{…}，“備用快捷方式”： {…}}`, the dialog's OK) and `activeSet` names the set in use without loading it. Names are trimmed, 1–64 characters, at most 64 sets. Every reply has `sets`, `activeSet` (`""` for the defaults), `defaultsName` and `modified` for the defaults), `defaultsName` and `modifiedXZZPH864:ZXZportKys true` adds `kys`, the bindings of every bindable command and tool as a `.kys` file (Photoshop's macOS key spelling; each row carries our id as `pc-id`, so the file imports back exactly); `總結：true` (or `{「mac」： false}`) adds `html`,Photoshop's Summarize page. A bad parameter anywhere in the call, an export path that can't be written included, changes nothing. **Headless parity:** the shell's commands (View › Zoom In, the panels; `photocraft_engine::shell_commands::SHELL_COMMANDS`) and the tools (`tools.select.<tool>`, listed with `"tool": trueXZPH884ZZX`, listed with `"tool": trueXZPH874) 上的被拒絕，並排除在導入集中（`import.reserved`）。檔案：`importKys: {"path": …}`讀取`.kys`，`exportKys: {"path": …}`和`summarize: {"path": …}`寫入檔案（回覆有`kysPath`/`htmlPath`）；自動化 (MCP) 拒絕這些路徑，就像 XZZPH882
|`edit.menus`/`edit.toolbar`|`{"hide":["edit.fade"],"show":[…],"color":{"edit.fill":"red"},"reset":false}`/`{"hidden":["Sponge"],"order":[…]}`|
|`edit.colorSettings`|`{"workingRgb":"display-p3","workingCmyk":"coated-cmyk","workingGray":"sgray","policyRgb":"preserve\|convert\|off",…,"askOnMismatch":true,"intent":"perceptual","bpc":true}`；開啟檔案（`file.openAs`，應用程式的檔案 › 開啟）以及圖片 › 模式和「工作」設定檔規格時受到尊重。`color.profileMismatch {"action":"preserve\|convert\|discard\|assignWorking"}`回答不符提示|

應用程式即時尊重的價值：歷史狀態（每個開啟的文件）、效果快取預算、介面
主題、畫布顏色和邊框、棋盤尺寸和顏色（CPU 和 GPU 畫布）、色域警告
顏色和不透明度、指南/網格/智慧指南顏色和樣式、網格間距和細分、尺標
單位（尺標、資訊面板、影像大小預設單位）、影像內插（影像大小預設）、
繪畫和其他遊標，使用滾輪縮放，使用 Shift 鍵進行工具切換，捆綁箭頭鍵
微移和微移捆綁暫停（與`ui.key`一起發送的箭頭鍵在暫停期間相互跟隨
將圖層、選擇輪廓或自由變換框作為一個歷史步驟輕推；代理商打電話
`layer.translate`或`select.transformSelection`本身與共享的`coalesce`金鑰相同），
鍵盤快速鍵、隱藏和彩色選單專案、自動保存間隔和崩潰恢復以及歷史記錄
日誌文字檔。 GPU 開/關和 GPU 圖塊大小將在下次啟動時套用。

桌面應用程式將它們儲存在平台設定目錄中的`preferences.json`中 (macOS
`~/Library/Application Support/Photocraft`、Windows`%APPDATA%\Photocraft`、Linux
`$XDG_CONFIG_HOME/photocraft`；用`PHOTOCRAFT_CONFIG_DIR`覆蓋）；自動儲存轉到其
`Recovery`資料夾（在 Windows 上為`%LOCALAPPDATA%\Photocraft\Recovery`，不在漫遊範圍內）
簡介；早期版本留下的自動保存在啟動時移動到那裡），以及視窗大小和
位置、停靠寬度和浮動面板幾何形狀
到它旁邊的`ui.ron`（保存會導致啟動崩潰的值的檔案被丟棄）。在便攜式模式下（`portable.txt`或`PhotoCraft.portable`檔案位於
可執行文件，如 Windows 可移植 zip 中）該目錄是`PhotoCraftData`旁邊
可執行檔代替。 Web 建置將它們保留在`localStorage`中。儲存僅寫入值
該實例自上次載入或儲存到目前儲存空間以來發生了變化，因此第二個
瀏覽器標籤（或應用程式視窗）永遠不會恢復對方的變更；對於兩者的值都發生變化的情況，
最新保存獲勝。失敗的寫入保持掛起狀態並重試（2 秒後，加倍最多 30 秒，並且
進一步更改後不久）；狀態列報告第一次失敗並發出通知
執著的一位。

使用者和匯入的 (`.abr`) 筆刷預設位於設定目錄的`Presets`資料夾中：一
每個預設組的`.pcbrushes`JSON 檔案、`tips/`下的內容定址提示點陣圖以及
`index.json`具有群組順序並刪除了內建功能（請參閱`photocraft_engine::preset_store`）。
商店在啟動時在後台加載，並在每次筆刷預設更改後同步；內建的
從來沒有寫過。操作清單是同一資料夾中的`actions.json`，並且在重新啟動後仍然有效。
無頭 CLI/MCP 會話和 Web 建置保留筆刷預設和會話的操作列表
僅限，除非附屬商店。漸層預設（包括導入的`.grd`群組）持續存在
的偏好。

命名的新文件預設也保留在桌面和桌面上的首選項 (`presets.documents`)
網路。檔案 › 新建 › 儲存預設…儲存目前表單； 「已儲存」標籤選擇一個設定或
刪除它。選擇僅填寫表格：Create still running`file.new`，以及文件名稱
與預設名稱無關。設定包括像素寬度/高度、解析度（ppi）、
模式、深度、背景和顯示`unit`（`px`、`in`、`cm`、`mm`、`pt`、`pica`）和
`resolutionUnit`（`in`、`cm`）。預設匹配檔 › 新建。背景顏色捕捉
工具箱顏色為`backgroundColor: [r,g,b]`(0..1)，因此重複使用它不會遵循後續工具箱
變化。最多可保留 256 個預設。無效的保存條目將被獨立跳過；其他
預設和首選項仍然載入。自動最近的設定是單獨的後續工作。

## 捕捉

開啟「檢視」>「捕捉」後，工具手勢會捕捉到「檢視」>「捕捉」目標（參考線和網格）
顯示 8 個螢幕內的圖層邊緣和中心、文件邊界和中心、選擇邊緣）
像素：移動工具拖曳（移動圖層的邊界）、選框、裁切、形狀、類型框和筆點，
自由變換手把並在框內拖曳和參考線。扭曲角捕捉啟用
參考線及其交叉點，使用實際的角，即使其手柄偏離中心。
當隱藏「指南」、「畫布指南」或「附加」時，對於所有捕捉手勢，指南目標都會被排除。
自由變換中的 Ctrl/Cmd 角扭曲與其啟用的捕捉目標具有相同的效果。控股
Ctrl 停用其他手勢的捕捉；用於允許角變形的 Ctrl 會不斷捕捉。
開啟視圖 › 顯示 › 智慧參考線後，移動工具也會捕捉其他圖層並進行繪製
洋紅色對齊線。`ui.pointer`驅動相同的程式碼，因此代理程式得到相同的結果。

## MCP 橋

`photocraft-automation`提供了一個基於官方 Rust SDK (`rmcp`) 建構的 MCP 伺服器。它以兩種模式之一執行：

- **Headless** (`photocraft-climcp`)：正在處理的`photocraft_engine::Session`。沒有窗戶。
- **橋接** (`photocraft-climcp --bridge 127.0.0.1:7878 --control-token-file <path>`)：每個工具都透過此協定轉發到正在執行的`photocraft--control 7878 --control-token-file <path>`，以便代理查看並驅動即時應用程式。

橋接器將一個經過身份驗證的 TCP 連接保持打開狀態，一旦連接空閒足夠長的時間以至於應用程式（在 30 秒後關閉空閒連接）可能會斷開連接，就會在呼叫之前斷開並重新連接。發送請求或等待回覆時發生傳輸失敗（包括回覆逾時），會中斷連線並報告操作可能已完成；在重試編輯之前檢查文件。編輯永遠不會自動重新發送（#1007）；無法建立的連線將報告為「請求未發送」。 MCP 伺服器選擇其純讀取工具（`ui_inspect`、`doc_inspect`、`command_list`、`jobs_list`、`session_list`、`ui_screenshot`）在新連線上自動重試，因為客戶不會套用任何內容；`call_read`進行每次通話。下一個單獨的呼叫重新連接並進行身份驗證，並且`id`與請求不匹配的回復行將被跳過。用戶端的呼叫逾時為 55 秒，嚴格低於伺服器的 60 秒回復期限，因此橋接器的錯誤是呼叫者看到的；可以執行更長時間的命令應傳遞`"wait": false`並輪詢`jobs.list`。它只接受環回地址，因為應用程式只偵聽環回。為其提供不記名代幣`--control-token-file`、`--control-token`、`PHOTOCRAFT_CONTROL_TOKEN_FILE`或`PHOTOCRAFT_CONTROL_TOKEN`：

```sh
photocraft-cli mcp --bridge 127.0.0.1:7878 \
  --control-token-file /private/path/photocraft-control.token
```

每個 MCP 工具如何對應到橋接模式下的控制方法：

| MCP 工具 |控制方式|
|---|---|
|`command_run {id, params, wait?}`|`engine.execute {command: id, params, wait}`|
|`jobs_list`/`jobs_cancel {job?}`|`jobs.list`/`jobs.cancel {job?}`|
|`command_list {filter?, enabled_only?}`|`engine.commands`（由MCP伺服器過濾；使用相同的裸陣列進行回复，行保留`disabledReason`和`needsDocument`，請參閱[命令列表回复]（#command-list-replies））|
|`doc_new {…}`|`engine.execute {command: "file.new", params}`|
|`doc_inspect`|`engine.execute {command: "document.inspect"}`|
|`doc_open {path}`|`app.open {path}`|
|`doc_place {path, …}`|`engine.execute {command: "file.placeEmbedded", params}`|
|`doc_export_layers {dir, …}`|`engine.execute {command: "file.export.layersToFiles", params}`|
|`doc_export_data_sets {dir, …}`|`engine.execute {command: "file.export.dataSetsAsFiles", params}`|
|`doc_import_data_sets {path, …}`|`engine.execute {command: "file.import.variableDataSets", params}`|
|`doc_save {path?, tgaBits?}`/`doc_export {path, tgaBits?}`|`app.save {path?, tgaBits?}`|
|`doc_render_preview {max_side?}`|`ui.screenshot`，直接回傳為PNG圖片內容|
|`session_list`,`ui_inspect`|`ui.inspect`|
|`ui_screenshot {max_side?}`|`ui.screenshot`，以PNG影像內容返回|
|`ui_pointer {events, modifiers?, button?}`|`ui.pointer`（其他參數錯誤）|
|`ui_menu_invoke {id}`|`ui.menu.invoke`|
|`ui_set {fields}`|`ui.set`|
| XZZPH100​​0ZZX |任何方法，不變地通過 |

Bridge 預覽擷取應用程式視窗；傳遞 XZZPH100​​1ZZX 是一個錯誤。無頭 XZZPH100​​2ZZX 支援 XZZPH100​​3ZZX，無需更改活動文件。

XZZPH100​​04ZZX 和 XZZPH100​​55ZZX 僅在無頭模式下工作。 XZZPH100​​6ZZX 工具和 XZZPH100​​7ZZX 僅在橋接模式下工作；在無頭模式下，它們傳回一個工具錯誤，解釋如何啟動橋接模式。

**安全說明：** TCP 控制使用不記名令牌，而不是用戶端身分或常規每種方法
授權。擁有令牌的用戶端接收非檔案系統控制面，
包括UI輸入、命令執行和應用程式控制。將令牌文件保密，不要
提交或記錄令牌，並且不要直接在處理命令的共享系統上傳遞令牌
線條可見。協定未加密，必須保持環回；不使用隧道或代理
將其傳送給不受信任的主機。

除非授予啟動時讀取和/或寫入根權限，否則檔案系統存取將無法關閉
`--automation-read-root`和`--automation-write-root`。環境變數等價物
(`PHOTOCRAFT_AUTOMATION_READ_ROOT`/`PHOTOCRAFT_AUTOMATION_WRITE_ROOT`) 支援
桌上型`photocraft`處理程序，但不是無頭`photocraft-climcp`或`serve`；
對於這些 CLI 模式，請明確提供標誌。在MCP橋接模式下，設定根
桌面處理程序而不是橋接 CLI 上。請求路徑必須是
適用根下的非空、正斜線相對路徑，或位於的絕對路徑
在它下面。絕對路徑與作為文本的根進行比較，其餘部分則像檢查一樣進行檢查
相對路徑；檔案系統上沒有解決任何問題。根、父級之外的絕對路徑
遍歷、相對路徑中的備用分隔符號、磁碟機/裝置/串流前綴、格式錯誤
元件和連結轉義在文件生效之前被拒絕。讀寫權限分別是
獨立的；新輸出文件的父文件必須已經存在。仍在使用的引擎命令
環境檔案系統路徑將被停用以實現自動化，直到它們遷移到相同的路徑
能力介面。互動式桌面檔案選擇器保留正常的使用者選擇的存取權限。

## 無頭伺服器

`photocraft-cliserve`保持一個無頭引擎會話（無窗口，無 GPU）並回答相同
stdio 上的 JSON 行信封，或帶有`--port <port>`的`127.0.0.1:<port>`上的 JSON 行信封（僅環回；每個
經過身份驗證的連線共用會話）。 TCP 使用相同的第一幀`auth`交換
作為桌面控制的令牌選項。 Stdio 不需要此 TCP 握手，因為存取是
從處理程序管線繼承。這是指令碼或代理進行多次編輯的最快方法：
無 MCP 框架，每個命令無需啟動應用程式。使用相同的設定其文件訪問
`--automation-read-root`和`--automation-write-root`標誌。執行：
`crates/automation/src/rpc.rs`.

|方法|參數|
|---|---|
|`engine.execute`|`{command, params?, wait?}`：任何引擎指令（`wait: false`啟動一個長指令作為後台作業：`{job, pending}`）|
|`jobs.list`/`jobs.cancel`|`{}`/`{job?}`：後台作業；取消一項或全部。每個請求（和 MCP 工具呼叫）首先套用已完成的作業，因此`doc.save`、`doc.inspect`、`doc.render`和`session.list`包含已完成作業的結果，而無需先輪詢`jobs.list`|
|`engine.commands`|`{filter?}`：包含參數文件和啟用的註冊表（`enabled`、`disabledReason`、`needsDocument`）；`result`是指令的列表陣列（[Z1045）
|`session.list`|開啟文件與活動索引 |
|`doc.open`/`doc.new`|`{path}`/`file.new`參數 |
|`doc.place`|`{path, scale? (percent), fit? (true), center? ([x, y]), index?}`：將檔案作為智慧型物件圖層放置在活動檔案上方並傳回`{layer, scale, bounds}`。`path`在讀取根目錄下解析，因此這是功能範圍的`file.placeEmbedded`— 該指令本身仍然拒絕環境路徑 |
|`doc.exportLayers`|`{dir, format? (png), prefix?, visibleOnly? (true), quality?, index?}`→`{files, count}`：每個匯出圖層一個文件，命名方式類似於`file.export.layersToFiles`的名稱。`dir`在寫入根目錄下解析，因此環境指令仍被拒絕。位元組來自命令自己的核心，因此兩個路徑寫入相同的檔案；寫入功能拒絕覆蓋現有檔案|
|`doc.exportDataSets`|`{dir, format? (png), dataSets? (all), naming? ("{name}\|{index}\|{document}"), index?}`→`{files, count}`：能力範圍的`file.export.dataSetsAsFiles`。在寫入任何文件之前，首先計劃批處理，並拒絕衝突的名稱，如命令 | 所示。
|`doc.importDataSets`|`{path, delimiter? (,), index?}`→`{imported}`：能力範圍的`file.import.variableDataSets`; CSV 在讀取根目錄下讀取 |
|`doc.save`|`{path?, format?, quality?, index?, tiffLayers?, tgaBits?}`（`.pcraft`本機，否則透過擴展導出；設定`tiffLayers: true`以保留 TIFF 圖層，而不是匯出展平的 TIFF；`tgaBits`24 像素，而不是導出展平的 TIFF；`tgaBits`24 像素，所做的那樣）。如果沒有`path`，則僅 PSD、PSB 或`.pcraft`文件以其自己的格式寫回自己的文件；其他任何內容都是錯誤，並且文件保持不變。成功的分層保存（PSD、PSB 或`.pcraft`）會將目前修訂記錄為已儲存並使該文件成為文件路徑，因此`session.list`報告`dirty: false`；平面匯出是一個副本，會使文件變髒 |
|`doc.inspect`|`{index?}`：與`document.inspect`相同的 JSON |
|`doc.render`|`{index?, maxSide? (1024; 0 = full), path?}`：PNG 到`path`，否則`{mime, base64}`|
|`doc.select`/`doc.close`|`{index}`/`{index?}`|
|`batch`|`{steps: [{command, params?, wait?} \| {method, params?}], stopOnError? (true)}`→`{completed, failed, results}`（`wait: false`的步驟啟動一個長指令作為後台作業，如`engine.execute`）|
|`methods`|上面的列表 |

```sh
printf '%s\n' \
  '{"id":1,"method":"doc.open","params":{"path":"in.jpg"}}' \
  '{"id":2,"method":"batch","params":{"steps":[{"command":"image.adjustments.invert"},{"command":"filter.blur.gaussianBlur","params":{"radius":3}}]}}' \
  '{"id":3,"method":"doc.save","params":{"path":"out.png"}}' | \
  photocraft-cli serve --automation-read-root /work/project --automation-write-root /work/project
```

MCP 伺服器具有與`command_batch`工具 (`{steps:[{id, params, wait?}], stop_on_error}`) 相同的批次，
在無頭和橋接模式下。`wait: false`的步驟立即回到`{job, pending}`，如`command_run`
確實；編輯同一文件的後續步驟將失敗，並顯示一條訊息，指定該作業直至結束。

## 運輸限制

桌面和無頭 TCP 偵聽器目前強制執行：

- 最大 1 MiB 的編碼請求行；
- 最大 8 MiB 編碼的 JSON 回复，包括換行符；
- 每個偵聽器最多同時服務 16 個連線；
- 30秒的套接字讀/寫超時；
- 無頭`batch`或 MCP`command_batch`請求中最多 256 個步驟。

過大的線路、過多的連線、未經身份驗證的請求或未經授權的檔案系統路徑
在命令發送或文件生效之前被拒絕。無頭 JSON-lines stdio 伺服器也
強制執行請求和回覆位元組上限； MCP over stdio (`photocraft-climcp`) 不
cap 請求位元組，因為 MCP SDK 讀取其自己的請求行（僅 MCP 工具結果為
檢查）。 TCP 連線在逾時後關閉
線；在 stdio 上，超大或非 UTF-8 行會收到一個錯誤回覆 (`id: null`)，其餘的
線路將被跳過而不被調度，並且會話及其打開的文件繼續提供服務。 MCP 工具結果以編碼 JSON 形式進行檢查，
包括文字/圖像內容信封，MCP 橋接器限制傳入的桌面回覆。
`batch`或`command_batch`在第一步驟停止，其結果不再符合回覆預算
（該步驟可能已執行；後面的步驟則不會）。 MCP 根據內部逃脫的大小對每個結果進行收費
文字內容，因此回應仍然列出了`completed`、`failed`以及迄今為止的所有結果，結束
與預算錯誤。

無頭自動化預覽 (`doc.render`/ MCP`doc_render_preview`) 允許最大請求
邊緣最多 2048 像素，來源文件最多 67,108,864 像素。`maxSide: 0`（MCP
`max_side: 0`) 仍然表示全尺寸，但如果文件的最長邊超過 2048，則會失敗。
在進行 Base64 編碼或寫入渲染檔案之前，PNG 結果的上限為 5 MiB。預覽
合成前執行尺寸/來源檢查；編碼後的 PNG 檢查在編碼後執行。
明確可信任本地 CLI 呈現保留其現有行為。

批次回覆具有聚合位元組預算，並為外部回覆和 ID 保留空間。
即使`stopOnError`為 false，耗盡它也會停止後續步驟。結果可以超過回复
編輯執行後的預算：錯誤表示操作可能已完成。先前的步驟
沒有回滾；重試前檢查狀態。 MCP 橋丟棄過大的傳入
回覆而不自動重試該操作。橋接螢幕截圖的解碼方式為
8192 像素邊緣、16,777,216 像素和 64 MiB 分配上限，即使沒有縮小規模。

這些上限沒有實現明確的 JSON 深度策略、總會話/文件記憶體
記帳、合成器暫存空間記帳、指令取消/持續時間限制，或
一般每種方法的功能。桌面截圖擷取/編碼和文件匯入/匯出
還需要自己的營運預算； UI 建立後桌面回覆上限適用
它的回應。有界輸出並不意味著有界命令成本。

### 渲染模式

`performance.renderingMode`接受`auto`、`gpu`或`cpu`。自動是新的預設設定
設定；較舊的`useGpu: false`或`gpuBackend: cpu`首選項繼續選擇 CPU 模式
直到保存顯式模式。更改將在下次發佈時套用。自動和 GPU 兩者
依靠圖形錯誤而不是風險文件。 CPU 相容性合成影像
在 CPU 上，並且更喜歡軟體視窗適配器（如果可用）。 macOS 仍使用Metal
窗戶。失敗的視窗渲染器初始化在 CPU 相容模式下重試一次；
下次啟動時啟動標記會偵測到驅動程式處理程序崩潰。

### Camera Raw 對話方塊

在桌面上，該對話方塊是一個自己的視窗（標題欄，像任何視窗一樣移動和調整大小），就像 Adob​​e Camera Raw 一樣；主視窗變暗，打開時不接受任何輸入，關閉視窗為「取消」。如果平台沒有額外的視窗（網路建置），它會繪製在主視窗上。

**截圖：**`ui.screenshot`僅捕獲主窗口，eframe無法捕獲主窗口
直接視口，因此雖然 Camera Raw 有自己的窗口，但 PNG 顯示變暗的主窗口，
不是對話方塊。透過`ui.inspect`讀取對話方塊的狀態（`cameraRaw`：參數、分佈圖、
縮放、控制矩形）並使用下面的`ui`參數驅動它。要“看到”它，請在螢幕外渲染
(`cargo run -pphotocraft-ui-egui--example snapshot -- --script '[["ui.menu.invoke", {"id": "filter.cameraRaw"}]]'`
或`egui_kittest`線束）：這些嵌入
視口，因此對話方塊繪製在主視窗上並出現在影像中。

`ui.menu.invoke {"id":"filter.cameraRaw","params":{}}`在活動狀態下開啟 Camera Raw
RGB/灰階層。`params: {"smartFilter": {"layer": id, "index": i}}`在現有的
相反，Camera Raw 智慧濾鏡（雙擊「圖層」面板中的濾鏡即可）：
儲存該濾鏡下方像素的設置，透過濾鏡遮罩預覽；提交
然後使用每個設定執行`layer.smartFilter.setParams`。`params.ui`接受`set`（濾鏡設定）、`before`、`scope`、`view`、`commit`和
`cancel`。一個請求的所有部分在應用任何部分之前都會經過驗證；被拒絕的請求
保持設定、視圖狀態、首選項和文件不變（並關閉一個對話方塊）
打開）。未知`ui`或設定屬性、非布林值`before`/`commit`/`cancel`和
`commit`和`cancel`都是錯誤。點曲線（`pointCurve`、`pointCurveRed`、
`pointCurveGreen`、`pointCurveBlue`）為空或 0-255 範圍內的 2-16 個有限點，輸入為
至少相距一層。 Commit調度一條`filter.cameraRaw`引擎指令；失敗的提交
保持對話方塊開啟以進行更正。沒有其他東西可以寫入文件歷史記錄。

**開啟原始文件。 ** 互動式開啟（檔案 › 開啟、開啟最近的檔案、拖放、
根據感測器資料開發的原始相機的命令列）首先顯示此對話方塊，標題為
“Camera Raw（名稱）”，帶有 **開啟** 和 **取消**，因為Photoshop在 Adob​​e Camera Raw 中開啟 raw。
`ui.inspect.cameraRaw.openingRaw`則為`{name, path}`。提交（開放）重新開發原始資料時
溫度、色調、曝光、陰影、高光、清晰度、紋理或去霧度已更改（拍攝時
白平衡增益、顯影曝光以及相機之前的線性光中的其他五個
色調曲線，就像 Camera Raw 對原始檔案所做的那樣；如果沒有拍攝時的白平衡，或者沒有檔案的位元組，它們將保持 RGB
調整），將剩餘設定作為一個`filter.cameraRaw`步驟套用，並保留
文件未經修改；其結果為`{document, redeveloped, filter?}`。取消關閉
文件。`app.open`和其他自動化開啟從不顯示對話方塊。偏好
`rawDefaults.openInCameraRaw`（預設`true`）將其關閉。

處理設定全部為映射或中性的導入 PSD Camera Raw 濾鏡使用此濾鏡
同一個編輯。`params.__cameraRawPsd`保留匯入/匯出元資料：編輯時保留它
儲存的參數。其他處理欄位或版本仍然是不透明的Photoshop過濾器。參見
[PSD智慧濾鏡](camera-raw-histogram.md#psd-smart-filters)支援設定與匯出
限制。

響應和`ui.inspect.cameraRaw`包含：

-`histogram`：`source`(`before`|`after`)、`size`、`approximate`、256 槽`red`/ XZ11711/Z
`blue`、`samples`、`transparent`、`invalid`、每色版`underflow`/`overflow`以及精確的
  端點計數器`shadows`(≤0) /`highlights`(≥1)。計數描述有界預覽
  其 RGB 樣本域中的代理，而不是全解析度或 ICC 顯示色域統計。
-`previewRevision`（僅在重新開發代理程式時前進）、`renderMs`、`histogramMs`
  （僅限 CPU；WebAssembly上為零）。
-`curveState`（`selected`、`drag`）和`curveRect: [left, top, right, bottom] | null`（當
  曲線部分已關閉）。更改`set.pointCurve`會取消陳舊的手勢。
-`scope`（下列視圖狀態）、`hasScopeSelection`、`vectorscope`（`bins`、`samples`、
`before`、`selectedRegion`) 或 null、`scopeMs`、`scopeRevision`、`overlayRevision`。
-`pointerReadout`和`samplerReadouts`：`position`、`space`（預覽範例中的`rgb`0–255）
  空間，或透過文件 ICC 設定檔`lab`→ D50)、`values`、`alpha`；不可見時為空
  像素。
-`hoverSample`：`position`、`rgb`（分佈圖域）和`hueSaturation`（圈數，0–1；僅限
  當顯示向量示波器時），或 null。
-`hoveredZone`,`previewRect`,`scopeRect`,`vectorscopeRect`.
-`view`（`zoom: null`用於擬合，或數字因子，其中 1 = 100%、`center: [u,v]`、
`hand`)、`zoom`（有效係數，佈局前為空）、`sourceSize`、`viewportRect`。
  100% 時，一個來源像素會對應到一個物理顯示像素，包括在 HiDPI 顯示器上。
-`previewApproximate`、`detailPending`、`detailError`：可見影像是否仍使用
  代理、是否正在執行全解析度細化以及任何細化錯誤。

`params.ui.view`僅更改導航：`zoom: "fit" | 0.0001..=16`、`center: [u,v]`
（0..=1 中的有限座標，夾緊以將影像保持在視圖內），`hand: bool`。適合最近的人
並跟隨視窗大小調整；開啟不同的 Camera Raw 對話方塊時導覽會重設。
無效的混合請求是原子的，就像範圍和過濾器設定一樣。查看請求不
重新開發代理程式或更改過濾器參數/歷史記錄。

使用縮放工具 (Z)，按一下切換「適合/100%」；按一下並向右/向左拖曳進出。
按住 Ctrl 鍵並拖曳（macOS 上的命令）以適合矩形區域。 H選擇手；空間暫時平移，
包括放置顏色採樣器時。 Alt/Option + 滾輪縮放指針；未經修改的
輪盤。捏合縮放。 Ctrl/Command +/- 縮放、0 適合、Alt/Option + Ctrl/Command + 0 顯示
100%；按住 Ctrl/Command+Shift 鍵並點選也可以選擇 100%。右鍵點選開啟適合/100%/預設
百分比。工具雙擊適合圖像。
文字欄位保留其按鍵，設定面板上的滾輪手勢仍可滾動。

更改設定時，代理仍然是快速路徑。在需要更多來源像素的地方，
Camera Raw 使用 **相同引擎 Camera Raw 管線** 在 PixelScale=1 和
相同的選擇/過濾器遮罩覆蓋範圍。本機細化執行在 UI 執行緒之外（一次修訂位於
一次）；過時的結果將被丟棄。僅將有限的可見裁切上傳到 GPU，並且
平移/縮放重複使用開發的像素。之前和中性設定直接讀取原始像素。
荒謬的代理域（超過 512 MP，包括遙遠的稀疏畫布外像素）被拒絕
在讀取像素之前。全解析度細化上限為 64 MP，以限制現有的全影像引擎管；
高於此，它明確報告不可用並保留代理。超大視口作物
還保留代理。如果沒有瀏覽器工作者，wasm 明確保留過濾後的代理
而不是阻止其 UI；先前和中性設定仍會裁剪原始來源像素。
分佈圖和向量儀繼續分析有界代理；讀數和剪裁
一旦細化可見，警告將使用全解析度像素。

`params.ui.scope`僅更改示範：`shadows`/`highlights`（剪切警告，U / O），
`lab`、`samplerTool`(S)、`vectorscope`、`selectedRegion`（需選擇文件）、
`redRight`、`hideSkinLine`、`floating`、`floatingRect: [left, top, right, bottom]`（200×150至
4096×4096點），`sample: [u,v] | null`，探頭列表：`samplers: [[u,v], …]`取代它，
或`clearSamplers: true`、`removeSampler: index`和`addSampler: [u,v]`編輯它，應用於該
順序（兩種形式不能組合）。座標標準化為顯示的代理；在
最多九個探頭。`tone: {zone: "blacks|shadows|exposure|highlights|whites", delta}`調整
與分佈圖拖曳相同的參數（`reset: true`將其設為 0）；曝光箝位至±5檔，
其他為±100。

懸停、探針移動、主題更改和麵板大小調整永遠不會重新顯示圖像；之前/之後
區域變化僅重建相依性分析。顯示選項和浮動面板
幾何形狀堅持偏好`dialogs["filter.cameraRaw.scope"]`；探頭和向量示波器
對話方塊開啟時可見性重設。未實現 HDR 範圍。參見
[camera-raw-histogram.md](camera-raw-histogram.md)。

### 編輯與撰寫動作

新記錄的「選擇圖層」步驟儲存圖層名稱而不是特定於文件的 ID，
操作面板顯示`Select Layer (Background)`。重複的名稱包含從零開始的
文件行走順序中的`occurrence`（包括組內的層）；保持順序一致
來源文件和目標文件之間。缺少名稱或事件會停止播放；一個名字沒有
當不明確時，事件會被拒絕。名稱是在選擇時捕獲的，因此稍後
操作中的重命名不會更改先前的目標。仍然支援數字`layer`ID
對於直接命令和較舊的錄音，面板將其標記為`Select Layer (ID 26)`。
導入的Photoshop操作中的字串值`layer`目標保持 main 的最頂層名稱匹配
並顯示相同名稱的標籤；任一目標形式都支援`remove`選擇模式。
重新記錄舊的選擇圖層步驟：僅透過 ID 無法復原它們的原始名稱。

建立一個動作會將其行捲動到視圖中；新記錄的步數會立即顯示。
將同一操作中的一個步驟拖曳到另一個步驟的上方或下方以對其重新排序。拖曳操作標題
重新排序操作列表。插入線標記放置位置；拖曳到行附近
頂部或底部滾動列表。重新排序與操作一起儲存，並且不會撤銷影像編輯。

錄製時，按下指派給另一個操作的功能鍵（或從面板播放）。
錄製器將 **Play Action:<name>** 新增為一步，並且呼叫的操作仍會執行。
每次播放都會解析名稱，因此編輯或移動呼叫的動作不會凍結或
重新定位呼叫者。稍後重新分配 F6 不會更改已錄製的命名呼叫。
缺少操作、遞歸呼叫和嵌套超過 16 層都會明顯失敗；孩子的失敗停止了
父級在下一步之前。每個巢狀指令都保留正常的自動化權限檢查。
包含擴展命令的現有記錄保持不變；刪除這些步驟並記錄
如果需要，可以再次進行指定呼叫。

操作位於**設定資料夾**（`actions.list`/`actions.get`上的`set`，由載入操作填滿，
由Save Actions編寫，用`actions.assignSet`更改）：面板首先顯示鬆散的動作，
然後每個設定為可折疊資料夾（`ui.actions.collapsedSets`）；放棄其中的一個動作
另一組的行將其移至該組。每個步驟都有Photoshop的兩個切換：檢查
（`actions.toggle`，關閉時跳過）和 **對話方塊** 切換 (`actions.dialog`)。演奏一個
其對話方塊開啟的步驟會開啟該指令的對話方塊，其中包含記錄的值並暫停：
播放結果帶有`paused: {step, id, dialog}`，OK執行對話方塊的值並繼續
從下一步開始，取消將停止操作。特工 (`automation_input`) 與
記錄值。面板選單的 **Insert Stop…** 在
選定的步驟（`actions.insert`接受任何命令 ID 和參數）。 `actions.conditional {如果，那麼，
else}` 是Photoshop的插入條件步驟：它測試文件或活動層
（`always`、`landscape`、`modeIsRGB`、`hasLayers`、`isBackground`、`hasPixelMask`…請參考其參數）
並按名稱播放匹配的操作，當該操作失敗時，呼叫者將失敗。

圖層面板 **快速匯出為 PNG** 將每個選定的圖層匯出為單獨的 PNG。單一
儲存對話方塊選擇第一個檔案名稱；剩餘的檔案被編號為已清理的同級文件
層名稱。在開啟對話方塊之前捕獲選擇；未選擇的圖層被排除。

## 共享面板對接

面板排列支援選項卡分組和重新排序、水平和垂直拆分、
以及應用程式視窗內可移動、可調整大小的浮動群組。拖曳面板選項卡，
或使用其上下文選單。浮動組保留在應用程式視窗內。

共享佈局從第一次對接作業時的現有工作區開始。
工作區保存保留其群組、活動標籤、分割大小、浮動幾何圖形和
隱藏面板的位置。重設內建工作區會恢復其預設停靠列。

-`window.panel.move {panel, anchor, zone?, before?}`將面板移到目標群組中。
`zone`是`tab`（預設）、`center`、`left`、`right`、`top`或`bottom`；
  按面板 ID 排列的選項卡。目的地必須已經可見。
-`window.panel.float {panel, x?, y?, width?, height?}`拆下面板。
-`window.panel.dock {panel, anchor?, zone?, before?}`將面板恢復到先前的狀態
  如果可能的話，將其放置在碼頭上，或放置在提供的目的地上。
-`window.panel.activate {panel}`顯示並選擇一個面板，恢復其儲存的
  盡可能的位置。`window.panel.close {panel}`隱藏它。

面板 ID 跨主題保持穩定。無效的 ID、非有限或無效的幾何圖形，以及
無效的目的地傳回錯誤而不改變安排。

鎖定工作區可防止移動、拆卸和關閉面板；啟動面板仍可使用。

`window.panel.layout {action}`也接受共享的、外部標記的操作架構。
例如，`{"action":{"MoveFloating":{"panel":"properties","rect":[44,66,360,450]}}}`
更新浮動組的位置和大小；`{"action":{"ResizeSplit":{"path":[],"size":{"Ratio":0.35}}}}`
調整根分隔符號的大小（`false`/`true`路徑條目選擇第一個/第二個子專案）。
`Move`與`placement: {"Tab":{"before":"swatches"}}`重新排序選項卡，以及
`SetStackOpen {panel,open}`/`ResizeStack {panel,height}`控製手風琴條目。
UI 調度這些相同的命令。面板 ID 未知、幾何圖形無效且陳舊
分割路徑會傳回錯誤而不更改工作空間。

現有`ui.set`面板可見度、`dockTabs`選擇和`dockWidth`控件
定制後仍然可用。寬度保持 250-520 點箝位。命名
`dock.hidden_tabs`和完整的本地組折疊設定也被保留。一個
更改扁平組順序或高度的傳統`dock`替換件在之後被拒絕
自訂：使用`window.panel.layout`變更自訂分割樹。替代品
當自訂根組混合時，更改舊式折疊設定也會被拒絕
來自不同本土群體的小組；使用`window.panel.group`折疊或展開
那個團體。跨混合`ui.set`請求的驗證是原子的。

`window.panel.group {panel, operation}`適用`close`、`collapse`、`expand`、`up`、
或`down`到面板的整個目前組。透過`operation:"move"`，提供
`anchor`和可選的`zone`/`before`（對於面板移動），或序列化的`placement`，
以原子方式轉移所有成員，同時保留其順序並選擇
請求`panel`。
當浮動組的頭夾落在一個物體上時，也會轉移整個組。
停靠目標。面板選單和圖示列保留
在自訂工作區中可用。`window.panel.<id> {show:true}`總顯、精選
並展開面板，包括選擇另一個選項卡時。工作區快照
保留自訂折疊設定和時間軸可見度。
