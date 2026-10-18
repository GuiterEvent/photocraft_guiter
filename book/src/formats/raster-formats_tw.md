# 點陣格式

`photocraft-codecs`提供格式偵測、功能聲明、解碼限制、元資料處理和平面影像編碼。預設版本聲明對 PNG、JPEG、TIFF、WebP、GIF、BMP、TGA、ICO、Netpbm/PFM、QOI、OpenEXR 和 Radiance HDR 的讀取/寫入支援。 HEIF/HEIC（iPhone 和 Mac 照片）是唯讀的，由`heic-rs`透過可選的`photocraft-heif`套件以純 Rust 進行解碼。它是`heif`貨物功能 (`cargo build -pphotocraft--features heif`) 背後的構建時選擇：官方構建包含它；如果沒有它，HEIC 文件仍然可以被識別，並且打開一份報告，表明該版本中不包含 HEIC 支援。 AVIF編碼可選；目前 crate 中未實現 AVIF 解碼。

## 分層 TIFF

Photoshop及其圖層保存的 TIFF 將其包含在兩個私有標記中：37724（`ImageSourceData`，與 PSD 具有相同的圖層部分，前面帶有字串`AdobePhotoshopDocument Data Block`）和 34377（PSD 影像資源）。`photocraft-codecs`在`Metadata`上作為不透明位元組傳遞；`photocraft-io`根據它們和 TIFF 像素重建 PSD 模型，並將其交給 PSD 導入器，因此群組、遮罩、圖層樣式、類型、形狀、智慧型物件和調整圖層從 TIFF 打開並從 PSD 中寫回完全相同的方式完全相同，並以相同的方式寫回，並以 PSD 開啟完全相同。在 Intel 順序 (`II`) 檔案中，Photoshop儲存位元組交換的整個區塊結構（反向鍵，例如`MIB8`和`ryaL`、小端整數和雙精確度），而像素樣本則保持大端；`photocraft-psd`的 XZZZPH24ZXZPH模組在兩個訂單之間逐字段進行轉換，並在發出警告的情況下丟棄任何其不知道佈局的區塊，而不是將位元組交換的資料傳遞給其他讀取器。限制：TIFF 保存一個透明度樣本，因此不會寫入額外的 Alpha 色版；實驗室文件以平面方式保存。另存為詢問「包含圖層會增加檔案大小」（使用「放棄圖層並儲存副本」），而「首選項 > 檔案處理」中的「儲存分層 TIFF 檔案前詢問」處於啟用狀態。指令碼化和代理保存會寫入平面 TIFF，除非它們要求圖層：`--tiff-layers`用於保存、批次和匯出命令中的`photocraft-cliconvert`、`run`和`batch`以及`"tiffLayers": true`和 MCP XZZPH30ZX。

## 網頁版

無損WebP（VP8L）由`image-webp`編碼和解碼。有損 WebP 由PhotoCraft自己的 VP8 關鍵影格編碼器在`crates/codecs/src/codecs/vp8`中編寫，`crates/codecs/src/codecs/vp8`是 RFC 6386 的潔淨室實作：BT.601 4:2:0 轉換、透過重建誤差為每個巨集區塊選擇 1×16 和 Walshdam 和 Walshdam變換與位元精確逆變換配對、量化RFC 的`dc_qlookup`/`ac_qlookup`表、具有每幀機率更新的令牌編碼以及平坦區域的跳過巨集區塊。`EncodeOptions::webp_lossless`選擇編碼器，`webp_quality`（0–100，libwebp 比例）設定量化器；alpha 在`ALPH`區塊中無損傳輸，ICC/EXIF/XMP 在擴展 (`VP8X`) 容器的自己的區塊中無損傳輸。解碼器應用幀的環路濾波器；雙方均不涉及 C 庫。

樣本深度、色版佈局、alpha、ICC、EXIF、XMP、DPI 和動畫行為等功能詳細資訊由`crates/codecs/src/format.rs`定義。不要僅從檔案副檔名推斷保真度。

## 匯出文件

將文件寫入平面格式的每個路徑（另存為、CLI、MCP`doc_export`、圖層到檔案、圖層 > 匯出為）都以相同的方式將其展平：

- 沒有透明度的格式（JPEG、Radiance HDR）將影像合成為白色，就像「導出為」和「快速導出」一樣；在CMYK中，白色是沒有墨水的。
- 無法嵌入 ICC 設定檔案的格式（GIF、BMP、TGA、QOI、ICO、Netpbm/PFM）將另一個設定檔案（線性光、顯示 P3 等）中的 RGB 文件透過顏色引擎（感知意圖）轉換為 sRGB，因為未標記的檔案被讀取為 sRGB。 OpenEXR 和 Radiance HDR 改為線性 sRGB（灰階 OpenEXR 的線性灰色），這就是它們在開啟時的標記方式。
- TGA，如Photoshop的 Targa：RGB Targa 為每像素 24 或 32 位元。另存為並儲存副本詢問（Targa 選項，從 32 開始，帶有 Alpha 色版，其他為 24）；匯出為有解析度選擇（自動、24 或 32 位元/像素）；指令碼傳遞`tgaBits`（CLI 上的`--tga-bits`）。在 32 位元時，文件的 Alpha 色版就是檔案的 Alpha（專色色版不計算在內；有多個時，不使用任何色版），並且顏色被精確寫入，不會在 Alpha 為 0 的情況下預乘或清除，因此遊戲紋理的遮罩及其顏色都會保留下來。如果沒有可用的 Alpha 色版，圖層的透明度就是 Alpha（沒有任何透明度則不透明）。 24 位元不寫入 alpha，在白色上合成透明度。沒有選擇：32 具有 Alpha 色版或透明度，否則 24。32 位元 Targa 打開為不透明背景，其 alpha 為“Alpha 1”，因此它是往返的。灰階 Targas 為 8 位。

合成 CMYK 文件的平面 TIFF 和 JPEG 導出將 RGB 合成轉換回
文件的 CMYK ICC 設定檔案（帶有黑點補償的相對色度）並嵌入
那個確切的輪廓。未標記的複合材料使用並嵌入PhotoCraft的合成塗層 CMYK 設定；
無效、非 CMYK 或不可用的嵌入設定檔案在合成前會被拒絕。 TIFF 保留
8/16/32f 深度和直 Alpha； JPEG 縮減至 8 位，並帶有警告和遮罩透明度
過零墨白。普通的單一柵格圖層保留其原始樣本。僅使用 RGB 格式
直接RGB合成。此 RGB 複合往返**不**保留原始墨水
分色、黑色生成或本機 CMYK 混合，且不是Photoshop奇偶校驗的聲明。

## 實作解碼限制

`DecodeOptions::default()`在解碼像素分配之前套用`Limits`：

|限制|預設 |
|---|---:|
|最大寬度| 262,144 像素 |
|最大高度| 262,144 像素 |
|最大像素數 | 1,073,741,824 像素 |
| 最大解碼分配| 64 位元目標上為 8 GiB；32 位元目標（包括 wasm32）上 2 GiB |

格式適配器將相容的分配限制傳遞到基於 PNG、TIFF、WebP 和`image`的解碼器中，而其他實作則執行明確檢查的維度/緩衝區驗證。測試涵蓋類似炸彈的標頭、截斷、位元翻轉、標頭覆蓋和隨機數據。

`Limits::none()`是為有意放寬政策的呼叫者而存在的。它仍然受到可表示記憶體大小的限制，但它刪除了預設資源策略，並且不應用於不受信任的輸入。

動畫容器和多頁 TIFF 由目前編解碼器 API 表示為單一解碼幀或頁面；打開一個會新增導入警告，例如「僅導入了 3 幀中的第一幀」。打開其影像資料內部的 JPEG 截斷，並顯示警告，表示其資料提前結束（當它根本不包含影像資料時會出現錯誤）。有關目前行為，請參閱 [`crates/codecs/README.md`](https://github.com/storytold/photocraft/blob/main/crates/codecs/README.md) 和來源功能聲明。
