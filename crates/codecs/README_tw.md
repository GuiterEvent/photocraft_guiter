#photocraft-codecs

用於平面點陣影像格式的編解碼器，以純 Rust 編寫。該板條箱是獨立的：它不相依性其他
工作區板條箱並定義自己的`Image`類型。它也適用於`wasm32-unknown-unknown`和
沒有 I/O：解碼器採用`&[u8]`，編碼器返回`Vec<u8>`。

它遵循`docs/architecture.md`§1.1 中的「避免 GIMP 漏洞」規則：

* **對稱。 ** 我們寫的每種格式，我們也可以讀取。唯一的例外情況列於
`ASYMMETRIC_EXCEPTIONS`，各有各的道理。測試套件會檢查這一點。
* **保留深度。 ** 影像可容納 U8、U16、F16 或 F32 樣本，且每種格式都保留
  它可以容納什麼。
* **元資料保留。 ** ICC、EXIF、XMP、DPI 和文字保留在格式支援的任何位置
  他們。
* **誠實。 **`caps(format)`說明每種格式所包含的內容，以及
`fidelity_warnings(&image, format)`在寫入之前列出了匯出將遺失的內容。兩者都
  函數使用與`encode`相同的編碼計劃，因此當文件
  變化。在進入的過程中，`Image::warnings`列出了解碼影像未顯示的內容：幀或
  遺漏的頁面或提前結束的資料。

```rust
use photocraft_codecs::*;

let img = decode(&bytes)?;                               // detect + decode (default Limits)
for w in fidelity_warnings(&img, Format::Jpeg) {         // e.g. "16-bit will be reduced to 8-bit"
    println!("{w}");
}
let out = encode(&img, Format::Tiff, &EncodeOptions::default())?;
```

## 能力矩陣（預設建置）

|格式|閱讀 |寫入 |深度（原生）|佈局（原生）|阿爾法 |國際商會 | EXIF | XMP|深度PI |文字|有損寫入|後端|
|---|---|---|---|---|---|---|---|---|---|---|---|---|
|巴布亞新幾內亞 |是的 |是的 | U8、U16 |灰色、GrayA、RGB、RGBA |是的 |是 (iCCP) |是 (eXIf) |是 (iTXt`XML:com.adobe.xmp`) |是 (pHY) |PH13 |
| JPEG |是的 |是的 | U8 |灰色、RGB、CMYK |沒有|是（多段APP2）|是（APP1）|是（APP1）|是的 (JFIF) |沒有|是的 |`zune-jpeg`/`jpeg-encoder`|
| TIFF |是的 |是的 | U8、U16、F32 |全部六種（包括 CMYK、CMYK+A）|是的 |是（標籤 34675）|沒有|是（標籤 700）|是的 |是（描述、品牌、型號、軟體、日期時間、藝術家、版權）| yes沒有 XZZZKXZ4XZ4XZKXZKXZKXZ）|
|網頁版 |是的 |是的 | U8 | RGB、RGBA |是的 |是的 |是的 |是的 |沒有|沒有|是（VP8，當`webp_lossless`關閉時） |`image-webp`/ 內建VP8 |
|動圖 |是的 |是的 | U8 | RGBA | 1 位元 |沒有|沒有|沒有|沒有|沒有|是（256 色盤）|`image`|
| BMP|是的 |是的 | U8 | RGB、RGBA |是的 |沒有|沒有|沒有|沒有|沒有|沒有|`image`|
| TGA |是的 |是的 | U8 |灰色、GrayA、RGB、RGBA |是的 |沒有|沒有|沒有|沒有|沒有|沒有|`image`（讀取）/內建（RLE 寫入，每個封包一條掃描線）|
|首次代幣發售 |是的 |是的 | U8 | RGBA（最多256×256）|是的 |沒有|沒有|沒有|沒有|沒有|沒有|`image`|
|直達服務 |是的 | **不** | U8 |灰色、GrayA、RGB、RGBA |是的 |沒有|沒有|沒有|沒有|沒有|不適用 |內建 |
| Netpbm（PBM/PGM/PPM/PAM/PFM）|是的 |是的 | U8、U16、F32（浮點：僅 PFM 灰色/RGB）|所有六個 (PAM`CMYK`/`CMYK_ALPHA`) |是沒有的 (PAM`CMYK`/`CMYK_ALPHA`) |是沒有的 (PAM) |
|品質指數 |是的 |是的 | U8 | RGB、RGBA |是的 |沒有|沒有|沒有|沒有|沒有|沒有|`image`|
| OpenEXR |是的 |是的 | F16、F32 | Y、YA、RGB、RGBA |是的 |沒有|沒有|沒有|沒有|沒有|沒有|`exr`|
|輻射 HDR |是的 |是的 | F32| RGB |沒有|沒有|沒有|沒有|沒有|沒有|是（RGBE）|`image`|
|適合 |是的 |是的 | U8、U16、F32 |灰色，RGB |沒有|沒有|沒有|沒有|沒有|是（標題卡）|沒有|內建 |
| DPX |是的 |是的 | U8、U16（完全透過 U16 讀取和寫入 10/12 位元程式碼值）|灰色、RGB、RGBA |是的 |沒有|沒有|沒有|沒有|僅 DPX 標頭字段（創建者、專案、版權、創建時間幀傳輸、色度、沒有位置、幀速率、時間程式碼|
|電影 |是的 | **不** | U16（10 位元碼值）|灰色，RGB |沒有|沒有|沒有|沒有|沒有|沒有|不適用 |內建 |
| AVIF | **不** |僅具有`avif`功能 | U8 | RGB、RGBA |是的 |沒有|沒有|沒有|沒有|沒有|是的 |`image`+`ravif`|
| HEIF/HEIC |具有`heif`功能 | **不** | U8、U16（10/12 位元解碼為 U16）| RGB、RGBA |是（輔助阿爾法）|是（`colr`教授）|是的 |是的 |沒有|沒有不適用 |

「本機」表示資料無需轉換即可儲存和讀回。其他任何內容都透過以下方式轉換
編碼計劃，`fidelity_warnings`在遺失資訊時報告轉換：

* 寫入整數格式的浮點影像將會傳送到該格式支援的 U16。價值觀
  0..1 之外加註`RangeClipped`。
* 寫入僅浮點格式（EXR、HDR）的整數資料將變為 F32 而不會遺失。
* CMYK 使用未經顏色管理的簡單公式轉換為 RGB。`CmykConverted`是
  升高並降低 ICC 設定文件，因為 CMYK 設定檔案不描述 RGB 資料。
* 寫入僅 RGB 格式的灰色會無損地擴展為 RGB，因此不會發出警告。

`Image::convert`應用相同的ICC規則：僅當顏色模型保持不變時才保留設定文件
一樣的。

調色板 BMP 和 PNG 像素在`Image`中解碼為 RGB(A)；`bmp_palette`和`png_palette`暴露
嵌入表格（以及 PNG 的透明條目），因此`photocraft-io`可以在索引顏色模式下開啟它們。

## 記錄的不對稱性和限制

* **DDS.** 2D 紋理的頂部 mip 等級：由色版遮罩描述的未壓縮像素
  (8、16、24 或 32 位，RGB 或亮度，帶或不帶 alpha）或 BC1/BC2/BC3 (DXT1/DXT3/DXT5)，
  附傳統標頭或 DX10 標頭。一切都以U8開啟；其他 MIP 等級是
  被忽略。立方體貼圖、紋理陣列、體積紋理、預乘 DXT2/DXT4 等格式
  （BC4–BC7，浮動）被`CodecError::Unsupported`拒絕。不提供寫作。

* **FITS.** 讀取主 HDU 或第一個`IMAGE`擴充中的影像；`NAXIS3 = 3`
  以 RGB 形式打開，任何其他平面計數以`DecodeWarning::MorePages`打開第一個平面。
  行遵循`ROWORDER`（預設情況下自下而上，如 Siril 和 DS9 所做的那樣）。適合的物理值
  保持與 U8/U16 完全相同（相機的`BITPIX = 16`、`BZERO = 32768`）；其餘變為F32，取為
  最多為 2 時進行歸一化，否則以 65535（或最大值）縮放。標題卡是
  保留為文字（`HIERARCH`表示長名稱）。平鋪壓縮 (fpack) 檔案被拒絕
`CodecError::Unsupported`;拜耳馬賽克開啟時未經過拜耳處理。

* **DPX.** 任一位元組順序的 SMPTE 268M 訊框（`SDPX`、`XPDS`）：第一個影像元素，
  描述符 6（亮度）、50（RGB）或 51（RGBA），無符號且未編碼，8、10、12 或 16 位元。
  10位元資料可以打包（0）或填入三個到一個字（1，方法A；2，方法B），12位元數據
  打包 (0) 或 1 到 16 位元字（1，方法 A）。 10 位元和 12 位元程式碼值以 U16 形式開啟
`round(c × 65535 / (2^b − 1))`，因此在其深度寫回的畫面可以準確地保持它們。否
  應用顏色變換：`DecodeWarning::DpxEncoding`命名傳輸特性並
  色度規範，因此對數框架報告為對數。比第一個元素更多的元素
  給出`DecodeWarning::MoreElements`；影像方向在導入時套用。寫作是
  big-endian，一個未編碼的元素，位於`EncodeOptions::dpx_bits`（8/10/12/16；預設為 8）
  U8 和 16 否則），對於 10 和 12 位元打包 1；帶有 alpha 的灰色被寫為 luma。
  其他描述符、帶符號或遊程編碼資料和其他位元深度返回
`CodecError::Unsupported`命名欄位。

* **Cineon（唯讀）。 ** 柯達的 10 位元對數底片掃描格式（DPX 的起源），
  從相同的 Kodak 標頭系列讀取：通用 1024 位元組標頭、big-endian、一（灰色）
  或三個 (RGB) 色版、每個 32 位元字中打包的三個 10 位元樣本、一個字上的行
  邊界。 10 位元程式碼值開啟為 U16`round(c × 65535 / 1023)`；檔案是黑/白的
  參考點是對數轉換選擇，不適用。深度不是 10 位元或
  1/3 以外的色版數傳回`CodecError::Unsupported`；截斷錯誤而非
  驚慌失措。沒有 Cineon 編寫器 (`ASYMMETRIC_EXCEPTIONS`)；另存為 DPX、TIFF 或 EXR。

* **相機原始檔案**（DNG、CR2、NEF、ARW...皆為 TIFF 結構）被辨識並拒絕
  對於`CodecError::Unsupported`：它們是感測器數據，而不是平面圖像。`photocraft-raw`解碼
  並開發它們，`photocraft-io`將它們路由到那裡。

* **HEIF/HEIC（唯讀，功能`heif`）。 ** 解碼位於選購的`photocraft-heif`箱中
  非預設`heif`功能背後（官方PhotoCraft版本啟用此功能；經銷商選擇）。
  如果沒有它，HEIF 仍然會被偵測到並且解碼回傳`CodecError::Unsupported`("HEIC/HEIF
  此版本不包含支援」）。 iPhone 和 Mac 照片使用`heic-rs`解碼（純 Rust，無`unsafe`）：
  單張圖片和網格平鋪照片，8 至 12 位，輔助 alpha、ICC、EXIF 和 XMP。的
  應用容器的`irot`/`imir`/`clap`並將 EXIF 方向改寫為 1（HEIF 的 EXIF
  標籤僅重複容器的標籤，並且套用兩者會將照片翻轉兩次）。灰階
  解碼為 RGB。影像序列、疊加與身份推導、多層 HEVC 等
  4:2:2/4:4:4 流返回`CodecError::Unsupported`或`Malformed`，絕對不會出現錯誤的像素。寫作
  需要一個HEVC編碼器，每個成熟的編碼器都是C，所以HEIF被列在`ASYMMETRIC_EXCEPTIONS`。
* **AVIF.** 編碼使用`ravif`，這是純 Rust 的。解碼
  需要`dav1d`，它是 C。因此 AVIF 不支援讀取，且寫入支援受到限制
  非預設`avif`功能背後。在預設建置中，它既不可讀也不可寫，
  所以對稱保證成立。它被列在`ASYMMETRIC_EXCEPTIONS`。
* **有損 WebP。 ** 無損（VP8L，透過`image-webp`）是預設值。`webp_lossless: false`
  使用我們自己的無塵室 VP8 關鍵幀編碼器（`codecs::vp8`，來自
  RFC 6386；libwebp 尺度上的`webp_quality`0–100 設定量化器）。 Alpha 無損傳播
  在`ALPH`區塊中，ICC/EXIF/XMP 在擴展 (`VP8X`) 容器的自己的區塊中。的
  編碼器尚未做出速率失真決策，且邊長超過 16383 px（VP8 幀
  標頭的限制）被`CodecError::Encode`拒絕。我們讀取有損和無損檔案。
* **動畫和多頁檔案**（APNG、動畫 GIF/WebP、多頁 TIFF）：僅第一個
  幀或頁被解碼，並且單個幀被寫入。`FormatCaps::animation`標記
  可容納更多框架的容器。解碼後的圖像隨後攜帶
`DecodeWarning::MoreFrames`/`MorePages`中的`Image::warnings`，附總時檔
  聲明它（幀和 TIFF 目錄在不解碼的情況下進行計數；降低分辨率
  TIFF 目錄和透明度遮罩不是頁面）。任何 TIFF 頁面仍然可以打開
  與`decode_tiff_page`（和`photocraft_io::import_tiff_page`）。 GIF 的每一幀都來自
  來自`decode_gif_frames`：每個幀都透過處理合成到邏輯螢幕上
  應用的方法，如 RGBA8，延遲以毫秒為單位，在`Limits`內，
`MAX_GIF_FRAMES`。視訊層指令使用它（`photocraft_io::import_gif_frames`）。
* **TIFF 中的 EXIF** 儲存為子 IFD 而非 blob，因此尚未保留。
`caps.exif = false`為 TIFF，並發出警告。
* **方向**（JPEG、PNG`eXIf`和 WebP 中的 EXIF 標記 274；TIFF 和 WebP 中的解碼頁面標記
  BigTIFF）應用於
  解碼，如Photoshop：像素恢復直立，並且 EXIF/XMP 方向被重寫
  改為 1（`DecodeOptions::keep_orientation`選擇退出）。編碼器始終寫入 Orientation = 1
  （`upright_exif`、`upright_xmp`，僅更改標籤的值，保留自己的類型），因此直立像素
  永遠不會旋轉兩次。解析嚴格：8位元組頭內有IFD0或之前被截斷
  它的下一個 IFD 指標、不完全是 SHORT 或 LONG 的 Orientation，或超出範圍的值
  讀作 1；如果有重複條目，則第一個獲勝。`Limits`在立柱上再次檢查
  大小（5-8 交換寬度和高度），並且旋轉的緩衝區分配錯誤。
`Image::oriented`在並行頻帶中轉動任何佈局和深度（對於 24 MP RGB8 in 大約 10 毫秒）
  釋放）。
* **解析度**（#1691，模組`resolution`）：沒有自己格式解析度的文件
（沒有 PNG`pHYs`，WebP，HEIF，...）從其 XMP`tiff:XResolution`/`YResolution`/
`ResolutionUnit`，否則來自 EXIF IFD0（RATIONAL X/Y 分辨率，單位 2 英寸 — 預設值）
  缺失 — 或 3 公分；單元 1 或任何格式錯誤的內容均視為未解決）。 JPEG 使用，例如
Photoshop，第一個可用的：APP13PhotoshopResolutionInfo（0x03ED），XMP，EXIF，並且僅
  然後是 JFIF 密度（相機只寫入 EXIF，許多工具寫入預設的 JFIF 72）。出口時
  EXIF 和 XMP 解析度被重寫為影像的（`export_exif`、`export_xmp`）：
  RATIONAL 值到位，ResolutionUnit 設定為 2，無法重寫已刪除的專案
  來自 IFD0，而不移動任何其他資料。 IFD1（縮圖）保持原樣。
* **JPEG**
  * 僅 8 位。兩個解碼器後端都不支援 12 位元。
  * 在其影像資料內部被切斷的檔案可以寬鬆地解碼（基線 JPEG 的缺失行來自
    灰色）但絕不是默默地：圖像帶有`DecodeWarning::Truncated`。一個結束的文件
    在任何掃描資料出現錯誤之前。
  * CMYK 總是以 4:4:4（子採樣 CMYK 不可移植）寫為 Adob​​e 反相 CMYK，並帶有
    APP14 標記。
  * 由於沒有 COM 段支援，因此未寫入文字。
  * EXIF 和 XMP 必須各自適合一個 APP1 段（65 533 位元組及其標頭）。元數據
    不適合被丟棄，導出仍然成功，帶有
`MetadataTooLarge`警告（「EXIF（...位元組）對於格式而言太大；它將被刪除」）。
    在刪除 XMP 之前，編碼器會先刪除分層文件屬性
    （`DocumentAncestors`、`TextLayers`）在平面 JPEG 中沒有任何意義。擴充 XMP 不是
    寫的。
* **TIFF 解碼**
  * 經典 TIFF 和 BigTIFF（版本 43，8 位元組偏移），位元組順序、條帶或圖塊，
    平面設定 1 或 2。
* 支援1、2和4位灰度，8/16/32/64位元整數（32和64位降為U16），
    F16、F32 和 F64（轉換為 F32）、WhiteIsZero（#691：色調不再反相）、
    調色板圖像（以 RGB 形式開啟）、關聯的 alpha（直接）和額外的樣本（刪除）
    除非第一個是阿爾法）。
  * 每個目錄均可存取：`tiff_info`列出主 IFD 鍊及其 SubIFD（標籤 330）
    與每個頁面的大小，種類（頁面，降低解析度的副本，遮罩）和儲存一起，走
    see-set、深度限制和 10 000 個目錄上限；`decode_tiff_page`開啟其中任何一個。
`decode`開啟第一個全解析度頁面，如Photoshop（跳過縮圖）。
  * 帶狀解碼：圖像以其最終尺寸分配一次，並且每個條帶或行
    在本機目標上並行地直接將圖塊解碼到其中，最多有一個
    每個工作人員的條帶/圖塊大小的暫存緩衝區（30 MP RGB8：放氣 345 → 32 ms，LZW 428 → 43
    發布中的毫秒數）。文件末尾截斷的圖像資料解碼為
`DecodeWarning::Truncated`；檔案對於其聲明的大小來說太小（比任何檔案都大）
    壓縮可能會擴展）在分配之前被拒絕。
  * YCbCr、CIE Lab、`tiff`板條箱缺少的有符號整數和壓縮（CCITT、
    TIFF 中的 JPEG）傳回`Unsupported`。
* **TIFF 編碼** 在設定`EncodeOptions::tiff_bigtiff`時寫入經典 TIFF 或 BigTIFF 或
  該文件可能超過 4 GiB。
* **EXR**
  * 多部分檔案（Maya/Arnold 渲染為每個 AOV 寫入一個部分）開啟該部分
    看起來最像彩色影像 — RGBA 擊敗 RGB 擊敗 YA 擊敗單色版，所以
    僅深度部分不再使檔案失敗 - 使用`DecodeWarning::MoreParts`命名方式
    存在許多部分。`exr_info`列出每個部分（名稱、視圖、大小、色版、深度/平鋪）
    不解碼像素，`decode_exr_part`解碼選定的像素。深部保留
使用深度解碼器：具有任何內容的檔案在此處將被拒絕並帶有指向它的指標。
  * 從資料視窗以全解析度讀取。
  * 色版名稱透過後綴匹配，因此`layer.R`算作`R`。
  * 不支援二次取樣色版。
  * Cryptomatte 層（規範 1.2，由 Arnold、V-Ray、Redshift、Mantra/Karma 編寫）
    和 Cycles）按結構列出並解碼：`cryptomatte_layers`讀取標頭
    屬性、色版和嵌入清單（不含像素），`decode_cryptomatte`返回
    每個像素（ID、覆蓋範圍）對，以及`cryptomatte_id`/`cryptomatte_key`/`cryptomatte_preview_color`
    按照參考實作的方式將雜湊名稱轉換為 ID（官方測試向量
    測試套件）。
  * 深度資料（掃描線或單級圖塊、壓縮 NONE、RLE 或 ZIPS）以平面形式開啟
    image：每個像素的樣本合成一個，其中
`DecodeWarning::DeepFlattened`，因為不保留每像素深度。深 ZIP（16 行
    區塊）和深度 mip-map 或 rip-map 切片返回`Unsupported`。
  * 僅提供無損壓縮用於寫入：None、RLE、ZIP1、ZIP16 和 PIZ。
* **Netpbm 書寫** 從圖像中選擇子類型：float 給出 PFM，gray 給出 P5，RGB 給出
  P6，以及任何帶有 alpha 或 CMYK 的內容都會給予 P7 (PAM)。因此寫入`.pbm`檔案會產生
  其中任何一個符合數據。
* **顏色轉換**（CMYK↔RGB，RGB→灰色）是幼稚的。顏色管理轉換屬於
`photocraft-color`.

## 限制（減壓炸彈）

`DecodeOptions { limits: Limits { max_width, max_height, max_pixels, max_alloc }, .. }`防護罩
解碼。在分配像素緩衝區之前檢查標頭尺寸，預算為
也傳遞給底層解碼器。違規返回`CodecError::LimitExceeded`。的
預設值為每邊 262144 像素、2^30 像素和 8 GiB（在 32 位元目標（例如 wasm）上為 2 GiB）。
PNG 也透過`max_alloc`綁定聚合解碼文字和 XMP（UTF-8 關鍵字和值），
包括 IDAT 之後的區塊。該預算與像素無關，而不是總的處理程序記憶體上限。

## 測試

`cargo test -pphotocraft-codecs`涵蓋：

*完整的往返矩陣（每個可寫入格式×4個樣本類型×6個佈局），精確用於
  無損格式且高於有損格式的 PSNR 臨界值；
* 依格式儲存元數據，對照`caps`檢查；
* 幻數檢測，包括負數；
* 保真度警告，並交叉檢查警告是否與實際往返結果相符；
* 對稱能力保證；
* 格式錯誤的輸入：截斷、位元翻轉和 proptest 隨機位元組；
* Adam7 PNG 對抗漸進式 PNG 和作為預言機的`image`板條箱；
* 限制執行。

如果`corpus/pngsuite/*.png`存在於儲存庫根目錄中，則其中的每個檔案都會與
`image`板條箱的解碼器。如果沒有它，該測試將被跳過。
