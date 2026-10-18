# 上下文選單平價合約

> **最後審核：** 2026-10-13 · **最後更新：** 2026-10-13 · **更改：** 次要（圖層遮罩縮圖選單，#4575） · **目標：** Adob​​ePhotoshop2026

狀態：實施合同，更新於 2026 年 10 月 9 日。範圍：畫布、文件標籤和麵板。 「右鍵」是指輔助指標點擊； macOS 按住 Control 鍵點擊也應該打開平台發送上下文選單手勢的相同選單。Photoshop使用活動工具、選擇和麵板專案變更上下文選單。 Adobe 記錄了該原則，但沒有發布完整的當前選單目錄。 **記錄**表示 Adob​​e 來源明確命名了手勢或操作。 **推斷** 表示PhotoCraft設計目標或從主選單中已知的專案，其存在於Photoshop上下文選單中仍需要當前產品捕獲。不要將推斷的行呈現為精確的Photoshop選單。 [Adobe：面板和選單](https://helpx.adobe.com/photoshop/using/panels-menus.html)、[Adobe：捷徑和上下文選單](https://helpx.adobe.com/sa_en/photoshop/using/customizing-keyboard-shortcuts.html)。

## 派遣合約

1. 在開啟之前對目標進行命中測試：模式操作/手柄、文字選擇、畫布內容、面板行或圖示、選項卡，然後是空工作區。通用畫布選單不能吞沒更具體的目標。
2. 選單開啟時拍攝目標和選擇的快照。現有多重選擇中的一行作用於該集合；在其命令執行之前，其外部的一行將成為目標。將相同的規則套用於圖層、色版、路徑和預設。
3. 從由目標類型、活動工具、文件狀態、平台和活動操作鍵入的純選單模型中解析專案。此模型提供標籤、命令 ID、參數、啟用狀態和原因。每個變異項都會發出一個引擎命令；僅查看專案可能會更新 shell 狀態。不可用的功能會因某種原因而被停用或省略，而不是可點擊的無操作。
4. 保留主按鈕工具的行為。二次點擊僅開啟選單，除非右鍵拖曳手勢或使用者明確的右擦除首選項擁有該手勢。拖曳不得短暫開啟選單，且開啟選單不得繪製、變換、取樣或提交。
5. 錨定指標並夾住視窗；支援鍵盤上下文選單鍵/Shift+F10（如果可用）、Escape/外部點擊、僅在非空組之間分隔符號、選取/單選狀態、鍵盤導航和螢幕閱讀器標籤。
6. 使用與功能表列/選項列相同的命令 ID 和啟用。提供UI檢查資料和控制色版操作來選擇沒有像素座標的選單項，同時保留真實的指標測試和螢幕截圖。
7. 獨立測試每個選單模型的條件行和錯誤狀態安全性；測試實際的輔助點選路由、選單選擇、撤銷/重做以及無意外的主要操作。

## 畫布工具矩陣

「分支點的PhotoCraft」欄記錄了此 PR 之前的行為。 Picker指的是paint_mouse.rs中的筆刷預設快速選擇器； Layers 是指layer_pick_ui.rs 中經過命中測試的圖層清單。 Command/Ctrl + 二次按一下可使用每個工具開啟圖層列表，而「移動」則無需使用修改器即可開啟圖層列表。 **建議**超出 Adob​​e 記錄行的精確Photoshop二次點擊專案，直到從目前Photoshop擷取為止。

|PhotoCraft中的工具 |Photoshop/ 所需的上下文操作 |PhotoCraft位於分支點 |優先權、證據 |
|---|---|---|---|
|移動|在該點選擇一個圖層；選項列中也存在自動選擇選項 |層 | P0; [Adobe圖層移動](https://helpx.adobe.com/photoshop/using/moving-stacking-locking-layers.html);精確的彈出佈局需要捕獲 |
|矩形/橢圓選框、套索、多邊形套索、磁性套索、魔棒、快速選擇、物件選擇 |主動選擇：2026 年選擇選單（請參閱下方的*選擇畫布參考*）；沒有選擇：進行選擇|快速選擇：選取器（筆刷工具，如Photoshop）；其他六個：以下的選擇選單（#1614413）；其他六個：
|作物|裁切期間：比例/預設、覆蓋、清除/重置、提交/取消 **（推斷上下文位置）** |沒有工具選單| P1；待定的作物目標|
|吸管|樣本量/來源選擇**（推斷）** |沒有工具選單| P2; [Adobe 插圖使用吸管](https://helpx.adobe.com/photoshop/using/panels-menus.html)，但不發布專案文字 |
|尺、筆記、計數|單位/測量、註釋編輯/刪除、計陣列操作**（推斷）** |沒有工具選單| P2 |
|筆刷、鉛筆、橡皮擦、背景橡皮擦、斑點修復、修復、克隆圖章、歷史筆刷、模糊、銳利化、塗抹、減淡、加深、海綿|筆刷預設、尺寸和硬度（如適用）|揀選員| P0; [Adobe 筆刷設定](https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/brushes-presets/create-brush-set-painting-options.html)。在 Windows 上按 Alt+輔助拖曳，在 macOS 上按 Control+Option-拖曳可調整大小/硬度（[Adobe 遊標捷徑](https://helpx.adobe.com/in/photoshop/desktop/get-started/settings-and-preferences/change-tool-pointers.html)）|
|漸層，油漆桶|漸層預設/類型/模式或填滿源/容差**（推斷的上下文位置）** |沒有工具選單| P1 |
|類型 |在文字中：剪下/複製/貼上、拼字建議、文字格式；外部文字：類型層操作**（推斷的上下文位置）** |編輯文字時 (canvas_tool_menu.rs`TYPE_EDIT_MENU`)：取消、剪下、複製、貼上、全選（在文字上）、檢查拼字...、尋找和取代文字...、仿粗體/斜體（選取）、點陣化類型、建立工作路徑、轉換為形狀、水平/垂直和抗鋸齒模式（選取）、轉換為段落/點文字、扭曲文字...； Esc 僅關閉選單。對於拼字錯誤的單詞，選單會以最多 3 個拼字建議（取代該單字）開始並新增至字典 (#4370)；忽略/忽略全部和非編輯類型層選單保持開啟狀態 | P1；單獨的編輯和非編輯狀態；[Adobe 文字編輯](https://helpx.adobe.com/photoshop/desktop/text-typography/get-started-with-text/edit-text.html) |
|手，變焦|適用於螢幕、100%、200%、列印尺寸；縮放步驟選擇**（推斷上下文位置）** |視圖選單 (#3242)：放大、縮小、適合螢幕、填滿螢幕、100%、200%、列印尺寸。圍繞右鍵單擊的點進行放大/縮小步驟（首選項 › 工具 › 將單擊的點縮放到中心以使其居中）。在圖像之外，貼板選單仍然獲勝。 | P1；檢視操作不會出現在文件歷史記錄中；行順序仍需要過時的Photoshop擷取 |
|筆，路徑選擇|路徑/點操作、進行選擇、填滿/筆畫路徑、自由變換路徑 **（推斷上下文放置）** |鋼筆畫布右鍵單擊遵循用戶提供的 20 行Photoshop捕獲（如下）。路徑行提供選擇、填滿和筆畫。路徑選擇自己的畫布選單保持開啟。 | P1；區分節點、線段、路徑與空白畫布 |
| 矩形、橢圓形、三角形、多邊形、直線、自訂形狀 |形狀/路徑操作、填滿/筆畫設定、變換**（推斷上下文位置）** |沒有工具選單| P2 |
|切片、切片選擇 |切片屬性、複製/刪除、分割、參考線 **（推斷上下文位置）** |沒有工具選單| P2 |
|自由變換處於活動狀態時的任何工具 |縮放、旋轉、傾斜、扭曲、透視、變形、旋轉/翻轉變體、提交/取消 **（推斷準確的上下文位置）** |缺少 docs/scorecard.md TOOL-214-5 | P0;變換操作 [Adobe 記錄](https://helpx.adobe.com/photoshop/desktop/crop-resize-transform/transform-manipulate-reshape/adjust-scale-rotation-and-perspective.html)；
|任何具有 Command/Ctrl + 二次點擊的工具 |列出指標下具有像素的可見圖層 |圖層 | P0;保留現有路線|

空像素或全隱藏點擊上的圖層清單選單應該是空/關閉；它不能選擇透明或隱藏圖層。右擦除首選項是替代的PhotoCraft行為，其選定狀態僅對於筆刷/橡皮擦而言優先於筆刷選擇器。

## 鋼筆畫布參考擷取 (2026-10-07)

使用者提供了路徑模式下閉合路徑的Photoshop螢幕截圖。此表依其順序保留每一行；組間出現分隔符號邊界。此捕獲中的灰色行仍然可見，但對於普通工作路徑被停用。啟用隨活動形狀圖層、儲存的路徑、複製的樣式和對稱狀態而變化。省略號行在更改文件之前開啟參數對話方塊。

|集團|依螢幕截圖順序排列的行 |PhotoCraft作用及適用性|
|---|---|---|
|路徑|建立向量遮罩；刪除路徑 |`layer.vectorMask.fromPath`在符合條件的圖層上；工作/儲存路徑上的`path.delete`|
|預設|定義自訂形狀... |`edit.defineCustomShape`帶名稱對話方塊 |
|选择/油漆|做出选择……；形状的新指南；填滿路径...；描边路径... |`path.toSelection`对话框；`view.newGuidesFromShape`用于形状层；像素层上的`path.fill`和`path.stroke`对话框 |
|出口|剪下路徑... |已儲存路徑上的`path.clippingPath.set`，具有平坦度參數 |
|幾何|自由變換路徑|`path.transform`具有平移、縮放和旋轉參數 |
|形状操作|统一形状；减去正面形状；在重叠处合并形状；减去重叠处的形状 |具有多个组件的形状图层上的四个`layer.combineShapes.*`操作 |
|複製樣式|複製填滿；複製完整筆畫|`path.style.copyFill`/`copyStroke`當形狀有那個風格 |
|貼上樣式|貼上填滿；貼上完整筆畫|複製符合樣式時的`path.style.pasteFill`/`pasteStroke`|
|圖層視圖|隔離圖層 |`select.isolateLayers`用於活動圖層 |
|對稱性|製作對稱路徑；禁用對稱路徑 |`paint.symmetryFromPath`在路徑上；對稱啟動時的`paint.symmetryDisable`|

該參考沒有建立Photoshop在單一錨點、片段或空畫布上右鍵單擊的行為；這些仍然需要單獨擷取。PhotoCraft的停用行不會呼叫指令。

## 選擇畫布參考 (2026-10-08)

使用選框、套索、多邊形套索、魔術棒或物件選擇工具右鍵。行保留命令 ID、對話方塊和選單列雙胞胎 (`canvas_tool_menu::SELECTION_MENU`) 的啟用；灰色的行永遠不會執行。

|集團|具有活動選擇的行（Photoshop2026 訂單）|PhotoCraft指令 |
|---|---|---|
|選擇|取消選擇；選擇逆；羽毛…;選擇並遮罩...|`select.deselect`、`select.inverse`、`select.modify.feather`、`select.selectAndMask`|
|商店 |儲存選擇...；制定工作路徑... |`select.saveSelection`;公差對話方塊後的`select.toWorkPath`|
|層|透過複製分層；透過切割分層；新層... |`layer.new.layerViaCopy`、`layer.new.layerViaCut`、`layer.new.layer`|
|轉變|自由變換；變換選擇；扭曲;觀點|`edit.freeTransform`、`select.transformSelection`、`edit.transform.distort`、`edit.transform.perspective`|
|填寫 | 充滿…;中風…;內容感知填滿... |`edit.fill`、`edit.stroke`、`edit.contentAwareFill`|
|過濾|最後一個過濾器 |`filter.lastFilter`|
| 淡出|淡出… |`edit.fade`|

Photoshop的產生填滿、刪除和填滿選擇行被省略：PhotoCraft沒有此類指令。在未選擇任何內容的情況下，選單提供「全選」、「重新選擇」、「顏色範圍...」和「載入選擇...」(`NO_SELECTION_MENU`)。 Adobe 相鄰來源記錄了顏色範圍；其他行是**推斷的**，直到有日期的捕獲確認它們為止。

## 面板和工作區矩陣

|表面/目標|所需選項與狀態 |現在PhotoCraft|來源與信心|
|---|---|---|---|
|圖層面板行|混合選項、複製/刪除、分組、匯出、轉換/點陣化、遮罩、剪輯、連結、樣式複製/貼上/清除、合併/展平、重命名；按圖層類型和選擇過濾 |在layer_menu_ui.rs中實現；專案啟用使用選單命令|PhotoCraft庫存。 Adobe 文件圖層重新命名/刪除、顏色標記、點陣化和展平[此處](https://helpx.adobe.com/photoshop/using/layers.html)；無需即時擷取即可**推斷**的精確組合排序 |
|圖層面板顏色標籤/縮圖/遮罩縮圖/效果行|彩色標籤；特定於點擊圖標的上下文操作 |圖層遮罩縮圖：禁用/刪除/應用圖層遮罩、添加/減去/與選擇相交遮罩、選擇和遮罩…（layer_menu_ui.rs`show_mask`，遮罩」缺失；其他目標上的通用行；延長前檢查| Adob​​e 明確記錄右鍵單擊圖層/群組以指定顏色（[圖層](https://helpx.adobe.com/photoshop/using/layers.html)）；其他圖示選單需要擷取|
|色版面板行|複製/刪除/重命名、色版顯示和疊加顏色、點轉換；新建/拆分/合併 |在channels_panel.rs 中實現|PhotoCraft庫存；確切的Photoshop訂單 **推斷** |
|筆刷和預設面板|重新命名/刪除筆刷或群組、匯出選定的筆刷、檢視/重設預設選項 |重新命名/刪除和預設群組/專案選單存在 | Adob​​e 明確記錄了從筆刷/群組右鍵點選選取的筆刷 ([筆刷](https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/brushes-presets/create-brush-set-painting-options.html))；PhotoshopZPH81ZZXZPH81ZZXZ彈出面板重新命名/刪除/重置/檢視選項已[記錄](https://helpx.adobe.com/photoshop/using/panels-menus.html)，儘管有些是面板按鈕選單 |
|路徑、歷史記錄、動作、色板、漸層、圖案、樣式、庫、屬性 |特定於目標的複製/刪除/重命名以及面板存在的面板操作 |此 PR 中添加了路徑行菜單；其他面板需要審核| **推斷**目錄；在聲明奇偶校驗之前記錄Photoshop捕獲 |
| 文件標籤 |關閉、關閉其他、關閉全部、在文件管理器中顯示、複製/移動到支援的視窗 |關閉、關閉其他、關閉全部、在 Finder 中顯示以及拖曳重新排序已實現；複製/移動到支援保持開啟狀態的視窗 | **推斷**確切狀態清單；P1 |
|空白貼片/工作區|灰色、黑色、客製化帆布顏色選擇；驗證目前Photoshop擷取中的任何其他條目 |沒有選單| Adob​​e [在影像外部右鍵單擊文件以取得畫布顏色](https://helpx.adobe.com/ph_en/photoshop/using/filling-stroking-selections-layers-paths.html)；確切的目前佈局仍需要擷取|
|文字欄位/選取的文字 |本機編輯選單和拼字建議，無需路由至活動影像工具 |在 text_field_menu.rs 中為每個文字欄位實現的編輯選單（撤銷、剪下、複製、貼上、刪除、全選；在所選內容中右鍵點選可保留它；永遠不會到達畫布或工具選單）。拼字建議位於文字工具的編輯選單 (canvas_tool_menu.rs) 中，而不是在通用欄位中，通用欄位不包含文字圖層文字 (UI-217-8) | P1; **推斷**準確的拼字清單 |
|選項列中的工具圖示 |重置工具，重設所有工具|稽核|明確記錄在 [AdobePhotoshop參考](https://helpx.adobe.com/archive/en/photoshop/cc/2015/photoshop_reference.pdf) 中；歷史淵源，驗證現代標籤|
|自適應廣角約束線|自動、水平、垂直方向 |目前右鍵刪除約束 | P1 行為不符：[Adobe 文件方向選擇](https://helpx.adobe.com/photoshop/using/adaptive-wide-angle-filter.html) |

## 交付切片和驗收

1. 核心路由：目標命中測試、選單模型、鍵盤可存取性、控制檢查、衝突規則。現有的移動圖層清單和繪畫選擇器仍然有效。
2.畫布高頻：選擇、自由變換、文字、手動/縮放、漸層/油漆桶。僅在基礎命令起作用的地方添加行。在聲明 1:1 奇偶校驗之前記錄準確的Photoshop標籤/順序/停用狀態擷取。
3. 面板：路徑行操作、文件標籤和不同圖層眼睛/縮圖/遮罩/效果目標；完成筆刷匯出並觀察色版/預設間隙。 Adobe [記錄眼睛圖示可見性手勢](https://helpx.adobe.com/content/dam/help/en/photoshop/using/default-keyboard-shortcuts/photoshop-keyboard-shortcuts.pdf)。
4. 長尾：形狀/路徑、裁切、元資料和專業工具。保持每個工具的固定裝置沒有選擇、活動選擇、遮罩/鎖定層以及活動操作（如果適用）。

每個切片都需要純模型測試（包括調度已知命令的每個專案）、輔助單擊和修飾符優先級的 UI 指針測試、命令/撤消測試、禁用狀態測試以及實際打開選單時的PhotoCraft螢幕截圖。將標籤、順序和狀態與過時的Photoshop螢幕截圖進行比較；不同工具或功能表列的螢幕截圖不足以作為證據。在每個捕獲的參考旁邊維護一個清單：Photoshop版本/平台、目標描述、工具、選擇/文件狀態、修改器和觀察到的確切專案。

## 修訂歷史

|日期 |改變|總結|
|---|---|---|
| 2026-10-13 |次要|圖層遮罩縮圖選單 (#4575) |
| 2026年10月10日 |微不足道|狀態列、修訂歷史記錄 |
| 2026-10-09 |次要|選擇畫布參考與工具矩陣更新 |
| 2026-10-07 |次要|筆畫布參考捕獲 |
