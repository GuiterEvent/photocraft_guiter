# 義大利語術語和動態 UI 標籤

本註釋與義大利目錄 (`crates/ui-egui/src/i18n/it.tsv`) 的完成一起提供。
該目錄已經涵蓋了所有關鍵的`cargo xtask i18n-coverage`措施，但還缺少 1,287 項
僅在執行階段存在的標籤以及其他目錄（巴西葡萄牙語、荷蘭語、捷克語）
已翻譯：濾鏡和調整參數名稱、Camera Raw 面板、燈光效果、
歷史記錄、圖層、路徑和色版選單標籤、元資料欄位、狀態訊息、系統資訊和
對話方塊文字。在義大利語中，這些內容以英語顯示。
它們現在已被翻譯，並且透過審查修復了錯誤翻譯和不一致的術語
現有條目。翻譯是根據PhotoCraft英文標籤的含義編寫的；不
包括專有翻譯資源、字型、螢幕截圖或使用者文件。

## 註冊和樣式

命令是第二人稱單數命令式（*Elimina*、*Applica*、*Crea
帕切托*）；文字稱使用者為 *tu*。只有第一個單字大寫，除了 in
指令名稱由斜線連接，保留兩個大寫字母（*Luminosità/Contrasto*，
*HSL/Scala di grigio*)。小數使用逗號。

## 術語

|英文來源 |義大利語 |注意|
|---|---|---|
|層|利韋洛 | |
|帆布|四方 |也可在*來四軸*、*應用四軸極限* |
|預設|預設| *薩爾瓦預先定義* |
|預設|預先定義/預定義 |從不*預設*或*valori predefiniti* |
|行程|特拉西亞 |輪廓和繪筆刷觸相似（*Tracce angolate*、*Direzione traccia*）；破折號保持 *Tratto* |
|銳利化|對比/對比 |也在 Camera Raw 中（*定量對比*）|
|噪音|幹擾| *Riduzione 幹擾 di luminanza*、*Riduzione 幹擾 cromatico* |
|亮點|露西 |與 *Ombre* 配對 |
|快照 |伊斯坦堡| |
|智慧型過濾器|過濾器 avanzati | |
|圖案|圖案| *音色圖案* |
|梯度停止 |蓬蒂德拉·sfumatura | |
|專色|丁塔皮塔| *Canale Tinta piatta* |
|點選 |點選 | *Fai clic*、*{key}-clic*、*doppio clic* |
|鍵盤快速鍵|烤肉串 | Scorciatoia da tastiera | |
|片|電影圖| *解釋電影*，*Sostituisci 電影* |
|刻面|斯法塞塔 | |
|半粗體/超粗體/超淺色| Semigrassetto/Extragrassetto/Extraleggero |每種重量都有一種拼寫|
|（顏色）模式 |方法| *Cambia metodo condizionale*，從不*modo* |
|陷阱|誘捕|印前名詞； *Sovrastampa* 是套印，是另一回事 |
| HDR 調色 | Mappatura 色調 HDR |色調映射； *Viraggio* 保留調色 (*Viraggio separato*) |
|斜角和浮雕| Smusso 和 rilievo |與*Rilievo*（浮雕）配對 |
|連結（智慧型物件）|奧傑托學院 | *Inserisci oggetto collegato*，*Converti in oggetto collegato* |

## 原生評論

2026-10-12：一位以義大利語為母語的人閱讀了目錄並修復了錯誤翻譯（Trap、HDR
色調、顯示文字、藤蔓顏色、覆蓋操作「儲存」指令）和不自然的措詞（斜角和
浮雕、東亞/中東特徵、連結的智慧型物件、位元深度標籤）。

## 未涵蓋

引擎錯誤和狀態訊息、使用者文件、圖層和預設名稱以及自動化
與所有語言一樣，命令 ID 仍採用英語。其他目錄帶有的一些動態標籤
（`Ask Before Saving Saved Embedded`、`Clear Selection`、`Heal`系列）不被
當前程式碼；它們被翻譯為與其他目錄保持一致。

＃＃ 驗證

-`italian_covers_generated_filter_options_and_gallery_names`(`crates/ui-egui/src/filter_dialog.rs`)
枚舉每個過濾器和調整參數、符號選擇和圖庫名稱，例如
  韓國語測試，並且每個都需要意大利行。在此更改之前，`main`上的目錄
  它列出了 388 個缺少的標籤；現在沒有了。
-`cargo test -pphotocraft-ui-egui--lib`：目錄測試通過。兩次佈局測試，
`type_controls_fill_the_panel_at_every_width_and_scale`和
`properties_edit_shown_metrics_and_paragraphs_with_a_non_type_primary`，`main`也失敗
  沒有這個改變。
-`cargo xtask i18n-coverage`：`it`為 1,726 / 1,726。
- 螢幕外`snapshot`範例，其中`PHOTOCRAFT_LOCALE=it`渲染了燈光效果，
  Camera Raw 濾鏡和帶有義大利標籤的 Wind，不會遺失字形或剪下文字。
