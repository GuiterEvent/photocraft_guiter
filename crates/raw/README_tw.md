#photocraft-raw

無塵室、純 Rust 相機原始解碼器和開發人員。此板條箱是獨立的（沒有工作空間
相依性關係；它的解碼相依性是 Deflate DNG 的`flate2`和純 Rust 的`jxl-oxide`
JPEG XL 解碼器，適用於 DNG 1.7，僅限本機構建），沒有`unsafe`，沒有 I/O（`&[u8]`中），針對`wasm32-unknown-unknown`構建
（在那裡是順序的，在本機上是人造絲並行的）並且永遠不會對敵意輸入感到恐慌：每個偏移量都是
在分配之前，會根據`Limits`檢查邊界和大小。

```rust
use photocraft_raw::{develop, DevelopOptions, Demosaic};

let dev = develop(&bytes, &DevelopOptions { demosaic: Demosaic::Ahd, ..Default::default() })?;
// dev.rgb: interleaved 16-bit RGB in ProPhoto RGB (ROMM primaries, D50, gamma 1.8)
// dev.warnings: anything approximated or not applied
```

`photocraft-io`使用它，因此開啟原始文件會產生一個普通的 16 位元 RGB 文件，標記為
內建 ProPhoto 相容設定檔。

## 來源（無塵室）

僅根據公共規範、論文和文件觀察來實施：

* TIFF 6.0、TIFF/EP (ISO 12234-2) 和 Adob​​e DNG 規範 1.7。
* ITU-T T.81 (ISO 10918-1) 附件 H：無損 JPEG，流程 14（「LJ92」）。
* Canon 的 CR2 容器的已發布描述（標頭、原始 IFD、切片標籤 0xC640）。
* 公開記錄的製造商註解/私有標籤（ExifTool 的標籤表）：Canon ModelID (0x0010)，
  SensorInfo (0x00E0) 和 ColorData (0x4001)，Nikon WB_RBLevels (0x000C) 和 BlackLevel (0x003D)，索尼
  BlackLevel (0x7310)、WB_RGGBLevels (0x7313)、SonyRawFileType (0x7000) 和 SonyToneCurve
  (0x7010)、PanasonicRaw IFD0 標籤、奧林巴斯影像處理 (0x2040) 和 CameraSettings
  (0x2020) 預覽標籤、Fujifilm RAF 標頭與元資料記錄 (RawImageCropTopLeft /
  RawImageCroppedSize、FujiLayout、XTransLayout、WB_GRGBLevels、RawExposureBias）和 FujiIFD
  （RawImageFullWidth / 高度、BitsPerSample、StripOffsets / ByteCounts、BlackLevel、
  WB_GRBLevels）。
* Nikon 壓縮 NEF：Laurent Clévy 的 NEF 結構筆記和 Bill Claff 的“NEF 壓縮”
  （散文：`0x0096`表佈局，有損曲線，後面是固定表霍夫曼階段），
  ExifTool 的 Nikon 標記名稱文件和 ITU-T T.81 附件 F / H。霍夫曼表不
  儲存在文件中；它們是透過對 CC0 raw.pixls.us 樣本進行黑盒分析而恢復的（
LightCraft的相同場景（壓縮和未壓縮）（storytold/lightcraft#86）和
  在那裡對來自約 35 個機構的約 45 個文件進行了驗證。
* Sony cRAW (ARW 2)：H. Dietz，“Sony ARW2 壓縮：偽影和可信修復”
  (Electronic Imaging 2016) 以及 11 + 7 位元方案的 RawDigger / diglloyd 文章；
  透過觀察樣本文件確定了精確的位元佈局和色調曲線比例。
* Panasonic RW2 RawFormat 5、未壓縮的 Olympus ORF 和未壓縮的 Fujifilm RAF：已建立
  透過觀察樣本文件（位元打包、頁面佈局、樣本調整；對於 RAF
三個樣本儲存和 X-Trans / Bayer 佈局記錄的相反順序，每個
  透過根據相機預覽對每個候選階段進行開發來驗證）。
* Panasonic RW2 RawFormat 3/4（供應商壓縮）：固定速率每 14 像素 16 位元組，
  具有每個奇偶校驗預測器的預測器群組、2/4/8 位元欄位寬度、頁排列
  向下的位元順序和位於記錄標籤上方 15 個計數的黑色電平全部
  透過觀察樣本文件建立。
* 去馬賽克：Malvar、He 和 Cutler（ICASSP 2004）；平川＆帕克斯，“自適應
  同質定向去馬賽克」（IEEE TIP 2005）；對於 X-Trans 和其他非拜耳 CFA，
  梯度加權綠色插值（Lu & Tan，“彩色濾光片陣列去馬賽克：新方法
  和效能測量”，IEEE TIP 2003），具有邊緣感知色差。
* McCamy 的 CCT 近似（1992）；布拉德福德色適應變換。
* 來自相機嵌入 JPEG 的每個檔案顏色（`fit_look`、`src/look.rs`）：方法如下
LightCraft的`docs/camera-preview-colour.md`中的散文描述（storytold/lightcraft#165，
  Apache-2.0，同一組織）；未使用LightCraft程式碼。池相鄰違規者等滲
  回歸（Ayer 等人，1955）、Tukey 的雙權重（Beaton & Tukey，1974）、嶺回歸
  (Hoerl & Kennard, 1970) 和 CIE 1976 L\*a\*b\* 色差。

沒有讀取或使用來自 dcraw、LibRaw、rawspeed、rawler、rawloader 或 darktable 的程式碼，也沒有
相機顏色表被複製。

## 支援矩陣

|格式|狀態 |
|---|---|
| DNG |未壓縮（8–16 位元，包裝或未包裝）、Deflate（壓縮 8、預測變數 1 和 2）、無損 JPEG、有損 JPEG（DNG 1.4 壓縮 34892：8 位元基線 JPEG 切片或條帶，CFA 為灰色，LinearRaw 為 YCb 的 RPEG 切片或條帶，CFA 為灰色，LinearRaw 為 YCb 的 R217175 637625963；位元值然後經過相同的 LinearizationTable / BlackLevel / WhiteLevel 與其他深度一樣）和 JPEG XL（DNG 1.7，例如 Samsung Expert RAW；由`jxl-oxide`解碼；不在 wasm32 版本中，它是`Unsupported`）；和膠卷掃描器 DNG，開發為中性灰色，沒有白平衡或顏色矩陣）； LinearizationTable、BlackLevel（+ 重複、DeltaH/V）、WhiteLevel、ActiveArea、DefaultCrop、ColorMatrix1/2、CameraCalibration、ForwardMatrix、AnalogBalance、AsShotNeb達、AsShotNeal、/ GainMap（鏡頭著色）|
| DNG（有損 JPEG、浮點；GainMap 以外的操作碼；ProfileGainTableMap） |不支援/未套用（已報告）|
| CR2 |無損 JPEG，帶有製作者註釋中的切片、邊框和拍攝白平衡，黑色是在遮罩邊框上測量的。 CR2 沒有 CFA 標籤，行相位因型號而異，因此它是根據數據（綠色對角線）測量的，並以佳能型號 ID 表作為後備（參見`src/cr2.rs`）|
| CR2 sRAW / mRAW |不支援 |
| NEF / NRW、ARW、PEF 和其他 TIFF/EP 原始檔案 |未壓縮和無損-JPEG（包括索尼無損 ARW）CFA 資料 |
|Nikon壓縮 NEF（壓縮 34713）：無損和有損類型 1 / 2、12 和 14 位元 |解碼：固定霍夫曼表+製造商註釋`0x0096`種子和曲線（參見`src/nefc.rs`）。 BlackLevel`0x003d`以 14 位元為單位讀取。不支援「分割後有損」文件（預覽回退）|
|Sony壓縮 ARW（“cRAW”，SonyRawFileType 2）|解碼：11 位元最小/最大 + 7 位元增量區塊，SonyToneCurve 為 14 位，未寫入 BlackLevel 標籤時曲線的 512 黑電平（第一代機身，例如 ILCE-7）|
| Panasonic / Leica RW2、RawFormat 5（12 位元和 14 位元封裝）|解碼，搭配 PanasonicRaw 黑色/白色/WB/感測器邊框 |
| Panasonic / Leica RW2，RawFormat 3 和 4（供應商壓縮，12 位元）|已解碼；讀取的黑色電平比記錄標籤高 15 個計數； PanasonicRaw 白色/WB/感測器邊框|
|Olympus ORF，未壓縮 16 位元（E-1、E-400…）|解碼，使用 ImageProcessing 黑色/WB/ValidBits/裁切 |
|尼康「分割後有損」NEF、Sony「壓縮 RAW 2」、賓得壓縮 PEF、RW2 RawFormat 6 及更新版本、奧林巴斯壓縮 ORF |不受支援：除了 GPL 解碼器來源之外，沒有找到這些程式碼的公開描述，該板條箱可能不使用該來源（無塵室）。`photocraft-io`改為開啟內嵌 JPEG 預覽 |
| Fujifilm RAF，未壓縮（約 2010 年以來的 X-Trans I–V、GFX 和 Bayer 機身）|解碼：16 位元容器（位元組順序）、12 位元 LSB 優先打包、32 位元字中的 14 位元； X-Trans 6×6 或 Bayer 2×2 預設方向 |
| Fujifilm 壓縮 RAF、早期 FinePix / SuperCCD RAF（無 CFA TIFF）|不支援（沒有壓縮的公開描述）；預覽回退|
| CR3 |不支援本機 CRX；具有 CMT1 方向的有界 JPEG 軌道/PRVW 預覽回退。 I/O 將結果標記為 8 位元相機預覽 |
| X-Trans 和其他週期性非拜耳 CFA（高達 16×16）|無論選擇`Demosaic`，都透過一種與圖案無關的邊緣定向方法進行去馬賽克 |

尼康製造商註釋壓縮模式 6/10 選擇檢查小尾數壓縮 12/14 位
即使 TIFF 壓縮為 34713 時，也會出現行。現代有損`0x44 0x40`曲線使用
其較小的編碼域，同時保留感測器 ADU。有效的尼康作物面積
座標優先於模型後備。 HE/HE* 和分割後有損
仍然不受支援。四個固定的 CC0 Z6 檔案涵蓋兩個位元深度和兩個路徑。

## 開發流程

1. 線性化表，每個位置的黑色電平，縮放到白電平。
2. 白平衡（如拍攝；或文件沒有時的灰色世界；或顯式乘數），
   歸一化，因此最小的乘數為 1，然後剪輯為 1，使高光溢出保持白色。
3. 去馬賽克：`Bilinear`、`Mhc`（Malvar-He-Cutler）或`Ahd`（預設）用於拜耳；非拜耳 CFA
   (X-Trans) 始終使用邊緣定向通用方法 (`demosaic_cfa`)。
4. 相機 → XYZ (D50) 依據 DNG 規格（由白色的顏色矩陣內插）
   相關色溫，或 ForwardMatrix），或適合嵌入 JPEG 的相機外觀，
   → 線性 ProPhoto；曝光（基線曝光+使用者EV）；外觀的色調曲線；伽瑪1.8；
   16 位。
5. 方向。

開發在分配之前驗證公共感測器/作物/CFA 元資料。工作記憶
限制檢查尖峰即時開發緩衝區，而不是對不相交的階段求和。 AHD 限制
僅在需要時同時進行刮擦樂隊；保留正常的負載平衡和像素。
預設 2 GiB 限制保持不變，且不包括呼叫者擁有的感測器/文件儲存。

沒有顏色校準的檔案（CR2、NEF、ARW、RW2、ORF、RAF…）沒有相機矩陣。每一個
它們攜帶相機從相同感測器資料渲染的 JPEG，因此`fit_look`估計
每個檔案的相機渲染：感測器資料（如`develop`那樣進行白平衡）和
解碼後的 JPEG 被縮減為長邊最多 256 個單元的公共網格，以及 3×3
相機 → ProPhoto 矩陣加上一條單調色調曲線（曝光後應用於每個色版）
透過交替等滲透壓回歸曲線和
矩陣的加權嶺規化最小平方法（殘差縮放至近似 L\*，
針對未對齊邊緣和局部色調映射的 Tukey 雙權重；留下被剪裁和接近黑色的單元格
出）。三分之一的單元格被保留：僅當它以相同的顏色顯示相同的圖片時才使用該外觀，
此矩陣是合理的，並且保留的平均 CIE76 ΔE 最多下降到後備值的 85%。
將其傳遞為`DevelopOptions::camera_look`；`develop`然後記錄“適合相機的顏色”
嵌入 JPEG（平均 ΔE …）」。板條箱無法解碼 JPEG 本身：`photocraft-io`解碼
最小預覽至少 512 像素長，一次性安裝和開發。
Camera Raw 的調整導入首先適合預設曝光和拍攝白平衡，
然後將使用者的曝光、溫度和色調應用於相同渲染。
D5100 在 4992 × 3280 的 (8, 8) 處測量的影像區域為 4928 × 3264
感測器平面； D90 的尺寸為 4352 × 2868 平面中 (10, 10) 處的 4288 × 2848；每個
去除邊框而不改變電平、WB 或校準。
測量的影像區域保留明確的文件裁剪。

對於稍緊的相機 JPEG，擬合可以使用有界線來對齊參考代理
90–100% 以訓練細胞為中心的搜尋。它需要很強的相關性改進和
保留保留的色門；開發的裁切和相機校準保持不變。

沒有可用的配合（沒有預覽，不相關的單色或不同形狀的配合，或適合
沒有明顯幫助）記錄的中性回退保持：白平衡相機色版是
被視為線性 sRGB 原色，沒有色調曲線（看似合理但平坦的灰色顏色；轉換
DNG 給校準顏色），`develop`是這麼說的。

根據每個檔案的 8 個 Nikon D3200 NEF（12 位元有損壓縮室內場景）進行測量
嵌入式 JPEG（平均 CIE76 ΔE，300 像素網格，sRGB 完全開發輸出）：回退 11.6–26.0，
安裝1.0–7.5；所有 8 次擬合均被接受。剩餘誤差最高的是相機拍攝的照片
局部提亮陰影（主動 D-Lighting），這是一條全局曲線無法遵循的。適合
（包括預覽解碼）24 MP 檔案大約需要 50-70 毫秒。這是對相機的估計
外觀，而不是測量的校準：圖片風格和色調相關的渲染是近似的。

根據Photoshop的 Camera Raw（`src/cameras.rs`：Nikon D4）測量輪廓的相機
校準，因此沒有適合它們的外觀。在 6 個 D4 NEF 上針對Photoshop25.4 的預設相機
原始渲染（四分之一大小，幀上的中位數）測量的輪廓與其實際值相差 0.048 檔
平均 CIE76 ΔE 為 1.8 的亮度；適合嵌入 JPEG 的外觀將為 0.30 檔，
ΔE 6.4，因為它遵循相機的圖片風格。開放時間重新開發
（`photocraft-io`的`import_raw_tuned`）及其預覽所使用的色調曲線（`tone_curve`）採用
與預設開啟的外觀相同，因此更改滑桿不會改變顏色渲染。

＃＃ 工具

`cargo run --release -pphotocraft-raw--example rawinfo -- [--dump] [--demosaic ahd] [--png DIR] FILE...`
列印解碼的內容，進行解碼和顯影，並可以編寫 sRGB PNG 預覽和
嵌入 JPEG 預覽。

`cargo run --release -pphotocraft-io--example rawlook -- [--png DIR] FILE...`倍
相機-JPEG 顏色匹配，報告 ΔE 與嵌入 JPEG 的中性後備和
適合，並且可以寫入 JPEG 和 1200 像素的 sRGB PNG；`--synthetic`乘以
產生 24 MP 檔案。
