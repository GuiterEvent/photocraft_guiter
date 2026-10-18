# 西班牙文術語和動態 UI 標籤

本說明與西班牙目錄的審查和完成一起進行
(`crates/ui-egui/src/i18n/es.tsv`)。它已經涵蓋了由
`cargo xtask i18n-coverage`；術語審查的措詞與詞彙a一致
西班牙語圖像編輯器已經知道，修復了錯誤翻譯，並進行了註冊
一致。後來的一次添加了它仍然缺少的 1,276 個僅執行階段標籤。無專有
包括翻譯資源、字體、螢幕截圖或使用者文件：每個翻譯都是
根據PhotoCraft自己的英文標籤的含義編寫。

## 註冊和樣式

- **正式的「usted」**，如Photoshop的西班牙語 UI：「Haga clic」、「Seleccione」、「¿Desea Guardar…？」。
  目錄混合了“usted”和“tú”；現在每個面向使用者的指令都使用“usted”。
- **句子大小寫**在第一個單字之後，面板和工具名稱除外，它們保留大寫
  如Photoshop：「Panel Carácter」、「Herramienta Pluma」。
- 引號是「anglees」：「{name}」。
- 中性西班牙語，與 Adob​​e 的單一西班牙語目錄相符（例如“coma flantante”）。

## 動態UI覆蓋

合併較新的上游目錄行後，目錄有 4,010 個條目。它涵蓋了
完整的義大利語執行階段鍵集，並保留其他兩個西班牙語設定檔提示標籤
目錄也使用。附加標籤包括過濾器和調整參數和選擇，
濾鏡庫名稱、Camera Raw、燈光效果、歷史記錄和圖層控制、元資料、
對話方塊、狀態包裝和系統資訊標籤。在此之前，這些控制權回落到
英文是因為很多是動態組裝的，靜態覆蓋掃描無法找到。

`filter_dialog.rs`中的`spanish_covers_generated_filter_options_and_gallery_names`枚舉
產生的過濾器和調整表面，如果未來的標籤沒有西班牙語行，則會失敗。幾個
未使用的舊標籤將被保留，以便完整的目錄保持一致。

## 術語

|英文來源 |西班牙文使用者介面 |是 |
|---|---|---|
|畫板|工作台 | Lienzo（與*Canvas*發生衝突）|
|圖案（填滿、覆蓋、印章）|莫蒂沃 |贊助人 |
|預設|調整 preestablecido |預調整 |
|鍵盤快速鍵 |技術縮寫方法 |阿塔霍斯·德·特克拉多 |
|網格|庫阿德里庫拉 |雷吉拉 |
|切片|部門|部分 |
|插件 |插件 |補充 |
|暫存盤|虛擬紀念迪斯可|臨時迪斯可|
|傀儡扭曲|自由位置變形 |克拉沃斯變形 |
|針（木偶扭曲）|欽切塔 |針 |
|減淡/加深（工具）| Sobreexponer / Subexponer | **交換** |
|顏色減淡/顏色加深| Sobreexposición / Subexposición de color | **交換** |
|線性減淡（添加）/線性加深| Sobreexposición lineal (añadir) / Subexposición lineal | 直系子展示|
|硬光/針光/硬混合| Luz fuerte / Luz 焦點 / Mezcla definida | Luz dura / Luz puntual / Mezcla dura | 盧茲杜拉
|穿越 (群展) |夜市 | través |埃倫西亞 |
|克隆印章/圖案印章| Tampón de clonar / Tampón de 動機 |塞洛…
|羽毛|卡拉 | Desvanecer（與*Fade*發生衝突）|
|淡出|譯 |毀滅者 |
|顏色範圍|色彩伽瑪 |色彩間隔 |
|選擇並遮罩|選擇和塗抹睫毛膏 |選擇和睫毛膏 |
|選取顏色 |選擇性校正 |顏色選擇|
|陰影/亮部；亮點|黑影/照明；照明|黑影/盧斯；盧塞斯 |
|顏色查找 |顏色諮詢|顏色 | Búsqueda
|內容感知填滿/縮放/移動| Relleno / Escala según el contenido;偵測內容的移動器 | Relleno de contenido… |
|自適應廣角|格蘭角適應性強|自適應安古洛安喬 |
|約束比例|限制原蟲 | **構造** Prorciones |
|渲染（濾鏡選單）|口譯員 |渲染|
|智能銳利化|恩福克·蘇維扎多 | Enfoque 智能 |
|混合如果 |融合si |組合式 |
|淘汰賽（混合選項）|額外 |科貝爾圖拉 |
|明亮/黑暗（Camera Raw）|克拉羅斯/奧斯庫羅斯|托諾斯·克拉羅斯 / 托諾斯·奧斯庫羅斯 |
|主控（色相/飽和度）|待辦事項 |一般|

選單名稱同時也是操作，保留了Photoshop的區別：**Vista**（*查看*選單，而不是
«Ver»), **Volver** (*Revert*), **Borrar** (*Clear*), **Cortar** (*Trim*, 不同於 *Crop* «Recortar»),
**Decubrir**（*揭示*），**Purgar**（*淨化*）。產品與技術名稱（Photomerge、Camera
Raw、droplet、OpenType、sRGB、Wayland）未翻譯。

## 超出範圍

引擎產生的錯誤詳細資訊、狀態文字和參數幫助不透過 UI 傳遞
每種語言的目錄均保持英文（參見
[`localization-parity.md`](localization-parity.md))。使用者文件、圖層和預設名稱以及
自動化命令 ID 也保持不變。完成工作的貢獻者是
@gorevyoneticisi 與 OpenAI Codex 作為儲存庫的 MIT OR 下的 AI 貢獻者
Apache-2.0 術語；仍然歡迎母語人士的評論。
