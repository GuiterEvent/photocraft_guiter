# Krita 文件

PhotoCraft使用其開啟 Krita 的本機文件（`.kra`和`.krz`檔案變體）
層。檔案 › 開啟、最近使用的檔案、刪除、CLI（`convert`、`info`、`batch`）和自動化
（`doc.open`、MCP）皆使用`photocraft-io`中的讀卡機；瀏覽器版本也接受它們。一個
文件由其`mimetype`條目 (`application/x-krita`) 識別。 Krita 文件僅供導入：
Save 建議使用 OpenRaster (`.ora`)，Krita 使用其圖層開啟它，而`.pcraft`保留
一切。

## 讀到的內容

|克里塔 |PhotoCraft|
|---|---|
| 繪畫層，8 位元和 16 位元 RGBA，8 位元和 16 位元灰階 |文件深度的像素層（RGB 文件中的灰色層被轉換）|
|圖層位置（`x`、`y`），在任何地方平鋪，也可以在畫布外 |保留 |
|組，嵌套；傳遞組；折疊狀態|團體；傳遞；展開狀態 |
|名稱、可見性、不透明度、鎖定 |保留 |
|混合模式 | Krita 的 id，映射為 OpenRaster (`docs/ora.md`)；據報道，一些是近似值|
|透明面膜|圖層遮罩（第一個；報告更多並省略）|
|精選面膜|忽略（圖層的本地選擇，不是其外觀的一部分）|
|影像的 ICC 設定檔、解析度 |保留 |

每層都留下警告：向量、過濾器、填滿、文件和克隆層、過濾器、變換
並對遮罩、圖層進行著色，而不是上面的顏色空間，繼承 Alpha，每個色版
混合標誌（色版鎖定）以及超出保存在圖層中的幀的動畫。繪圖
據報告，助手和參考圖像被遺漏。 Krita 的合併圖像仍然顯示
原始外觀，因此結果可能會有所不同。

我們尚未讀取其影像色彩空間的文件（CMYK、Lab、XYZ、YCbCr、16 和 32 位元
float）作為 Krita 的合併圖像 (`mergedimage.png`) 打開，標記為預覽，並帶有警告。

## 觀察到的格式

第一個儲存條目為`mimetype`的 ZIP；`maindoc.xml`持有節點樹（最上面的第一個）
並將每個油漆圖層和遮罩指向`<image name>/layers/`下的檔案。这些文件是一个
文字標題（`VERSION 2`、`TILEWIDTH`、`TILEHEIGHT`、`PIXELSIZE`、`DATA <tiles>`）和每個圖塊
`x,y,LZF,<bytes>`行後面跟著壓縮標誌和位元組平面中的圖塊像素，LZF
壓縮的。 RGB像素為B、G、R、A；16 位元樣本為小尾數法。`.defaultpixel`文件
每個旁邊都有其瓷磚外面的顏色。集團自己的`x`/`y`不移動其
孩子們（對照 Krita 的合併圖像進行檢查）。

## 限制和安全

閱讀器使用`photocraft-format`的有界ZIP閱讀器（無ZIP64，CRC檢查），解析
`maindoc.xml`和`roxmltree`（上限為 32 MiB；DTD 由公開 id 聲明且不取得），
並使用邊界檢查 LZF 解碼器對圖塊進行解碼，該解碼器必須恰好產生一個圖塊。磁磚
每邊最多 1024 px；畫布每邊最多 300,000 像素，解碼像素為 1 GiB；
8,192 個節點；最多嵌套`MAX_GROUP_DEPTH`。版本 1 切片檔案 (Krita 1.x) 被拒絕。
格式錯誤的檔案、XML、數字、圖塊標題和圖塊資料回傳錯誤；被截斷和
測試覆蓋損壞的檔案。

＃＃ 確認

單元測試 (`crates/io/src/kra/tests.rs`) 在程式碼中建立 Krita 檔案（原始和 LZF 切片）並
檢查圖層、群組、直通、偏移、不透明度、可見性、鎖定、遮罩、預設像素、
16 位元和灰階轉換、合併影像回退、LZF 邊緣情況和格式錯誤的輸入。
`crates/automation/tests/krita.rs`透過代理介面開啟一個`.kra`，編輯並儲存
它作為 OpenRaster。`crates/engine/tests/kra.rs`（選擇加入）在以下位置開啟 Krita 創作的 16 位元檔案：
會話，檢查它是唯讀的，進行撤銷編輯並儲存為`.ora`和`.pcraft`。

真實文件檢查使用 Krita 編寫的預言機
[photocraft-corpus](https://github.com/storytold/photocraft-corpus) (`krita/`，由
`tools/krita-oracles/generate.py`，刪除了 Krita 嵌入的第三方 ICC 設定文件，以便
文件是 sRGB）：

```sh
KRITA_TEST_DIR=/path/to/photocraft-corpus/krita cargo test -p photocraft-io --test krita_corpus -- --ignored --nocapture
KRITA_TEST_DIR=/path/to/photocraft-corpus/krita cargo test -p photocraft-engine --test kra -- --ignored
```

2026-10-09，Krita 5.2.9：35 個`.kra`檔案中的 31 個在 Krita 合併影像的 3/255 範圍內渲染；的
其他的則因已知原因而有所不同（OpenRaster 的三個混合模式決勝局，以及
帶有向量圖層的文件，該文件被省略）。在剝離輪廓之前，線性光
16 位元文件及其嵌入的設定檔與 Krita 的合併影像在 0.5/255 內匹配
展平並轉換為 sRGB。

閱讀器是根據 Krita 編寫的文件編寫的；沒有使用 Krita 程式碼。
