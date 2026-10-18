# OpenRaster 文件

PhotoCraft開啟並儲存 OpenRaster (`.ora`)、分層格式 Krita、MyPaint、GIMP、Pinta
和其他人分享。檔案 › 開啟、最近使用的檔案、刪除、儲存與另存為、CLI（`convert`、`info`、
`batch`）和自動化（`doc.open`、`doc.save`、MCP）都使用相同的讀取器和寫入器
`photocraft-io`。瀏覽器版本也接受`.ora`。文件由其`mimetype`識別
條目，因此具有其他副檔名的 OpenRaster 檔案仍會作為一個檔案開啟。

## 保留什麼

| |打開|儲存 |
|---|---|---|
|像素圖層（任何大小和位置，也可以在畫布外）|是的 |是的，裁切為像素 |
|組，嵌套 |是的 |是的 |
|傳遞組 |`isolation="auto"`（0.0.6前，僅無混合模式）|`isolation="auto"`|
|名稱、可見性、不透明度 |是的 |是的 |
|混合模式 |`svg:*`名稱、`svg:plus`和 Krita 的`krita:*`名稱 |`svg:*`，其中規範具有模式，否則`krita:*`（由 Krita 回讀，其他地方為 Normal）|
|鎖|`edit-locked`、`alpha-preserve`|相同|
| 8 位元和 16 位元 |是（最深的層設定深度）| 8 位元文件作為 8 位元 PNG，16 和 32 位元文件作為 16 位元 PNG |
|解析度|`xres`|`xres`與`yres`|
|合併影像與縮圖|未讀（我們自己渲染）|寫入（`mergedimage.png`，`Thumbnails/thumbnail.png`）|

聲明 OpenRaster 0.0.6 或更高版本的檔案預設為隔離組；明確的
`isolation="auto"`無論組本身的複合操作如何都通過。年長的
文件和沒有可識別版本的文件保留了來自群組的遺留推論
混合模式。傳遞組不透明度仍使用PhotoCraft的內插法
背景，這可能與 0.0.6 的重疊子項的每個子項不透明度乘法不同。

逐層保存 OpenRaster 無法儲存的報告：像素遮罩應用於
圖層的像素、填滿不透明度折疊到像素、文字、形狀、填滿和智慧型物件圖層中
被寫為它們的渲染像素，以及調整圖層，圖層樣式，向量遮罩，
剪裁、Alpha 色版、元資料和 sRGB 以外的顏色設定檔案均被忽略。 Xor 和 Paint.NET
顏色加深和顏色減淡變體沒有 OpenRaster 名稱，並保存為“正常”。四色、實驗室
拒絕其他模式，並提示先轉換；灰階儲存為 RGB。

只有當 OpenRaster 檔案包含其中的所有內容時，它才是文件自己的檔案 (#3806)。
`photocraft_io::ora_losses`列出了保存時會遺漏的內容，來自作者的相同檢查
警告，但不編碼任何內容。當該清單不為空時，檔案 › 儲存不會覆蓋
`.ora`：另存為在同名 PSD 上打開，並帶有通知，列出 OpenRaster 將執行的操作
滴。儲存到`.ora`路徑後會寫入副本，就像Photoshop對於無法保留的格式所做的那樣
一切：文件保留其文件及其未儲存的變更。自動化遵循相同的規則
（`doc.save`和`app.save`沒有路徑拒絕，說為什麼）。

`stack`和`layer`以外的 OpenRaster 元素（例如 MyPaint 的文字）缺失
圖層影像和未知複合操作開啟時會出現警告，而不是使檔案失敗。
Krita 的 W3C Soft Light（`svg:soft-light`、`krita:soft_light_svg`）及其自己的 Hard Mix 開啟為
Photoshop的柔光和硬混，略有不同，也帶有警告。

## 限制和安全

閱讀器使用`photocraft-format`的有界 ZIP 讀取器取得存檔（無 ZIP64，
CRC 檢查），以`roxmltree`（16 MiB 上限）解析`stack.xml`，並解碼每層影像
通過`photocraft-codecs`有尺寸限制。畫布每邊最多 30 萬像素
8,192 個圖層和群組，嵌套深度最多為`MAX_GROUP_DEPTH`，解碼像素為 1 GiB
總計。格式錯誤的檔案、XML、數字和影像返回錯誤；被截斷和損壞的文件
被測試覆蓋。

＃＃ 確認

單元測試 (`crates/io/src/ora/tests.rs`) 往返層、群組、直通、偏移、
不透明度、可見性、鎖定以及 8 和 16 位元的每個混合模式、檢查遮罩、渲染層
種類、存檔佈局和格式錯誤的輸入。合成 8/16 位元像素裝置檢查現代
以及傳統的群組隔離預設值、明確隔離和直通。
`crates/automation/tests/openraster.rs`保存，
透過代理介面重新開啟、編輯和儲存，並檢查帶有
調整層永遠不會覆蓋其`.ora`； CLI 測試可以兩種方式進行轉換。

真實文件檢視使用 Krita 創作的預言機
[photocraft-corpus](https://github.com/storytold/photocraft-corpus) (`krita/`，由
`tools/krita-oracles/generate.py`）。每個 Krita 檔案都包含 Krita 自身的渲染
`mergedimage.png`:

```sh
KRITA_TEST_DIR=/path/to/photocraft-corpus/krita cargo test -p photocraft-io --test krita_corpus -- --ignored --nocapture
```

2026-10-09，Krita 5.2.9：35 個`.ora`檔案中的 32 個在 Krita 合併影像的 3/255 範圍內渲染；的
其他三個因`crates/io/tests/krita_corpus.rs`（Krita 的
較深和較淺顏色的平局，以及色版總和恰好為 1 的硬混合。在另一個
方向，Krita 打開了前 34 個檔案的每個PhotoCraft重新匯出並渲染了 33 個
它們與原來的完全相同；例外的是Photoshop的 Soft Light，其寫為
如`svg:soft-light`。 MyPaint 和 GIMP 檔案尚未檢查。

格式參考：[OpenRaster規範](https://www.openraster.org/)。讀者和
writer 是根據規範和 Krita 寫入的文件編寫的；沒有申請程式碼
被使用了。
