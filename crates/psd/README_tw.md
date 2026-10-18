#photocraft-psd

AdobePhotoshop**PSD**（版本 1）和 **PSB**（版本 2，「大文件格式」）檔案的獨立讀取器和編寫器。它保留它讀取的所有內容，因此未修改的檔案會逐位元組寫回。它不相依性其他photocraft板條箱。

* Adob​​e 公共*Photoshop文件格式規範* 的無塵室實施。在規範未提及的情況下，行為遵循 MIT 許可的psd-tools/ag-psd文件，並且程式碼註釋也是如此。
*`#![forbid(unsafe_code)]`。專為`wasm32-unknown-unknown`打造。核心 API 工作在位元組片上；檔案助理僅存在於本機目標上。
* 解析不會因為格式錯誤的輸入而出現恐慌。限制：尺寸≤ 300 000；根據剩餘輸入檢查分配情況；一次解碼最多產生`MAX_DECODED_BYTES`（64 位元目標上為 8 GiB，32 位元目標上為 2 GiB）；描述符嵌套深度 ≤ 64。

## 保證

|物業 |如何|
|---|---|
|`PsdFile::from_bytes(b)?.to_bytes()? == b`用於未修改的檔案 |色版資料保持其原始編碼。未知的區塊和資源是原始儲存的。精確的填滿會在任何可能變化的地方被記錄：標記區塊、圖層資訊、部分預告片。 |
|`from_bytes(to_bytes(f)) == f`適用於任何型號`f`|規格預設值：`padding: None`表示“零填滿到偶數”，只要檔案使用該預設值，解析器就會產生`None`。 |
|延遲解碼 |解析器僅複製編碼位元組。當詢問時色版被解碼（`ChannelData::decode`、`ImageData::decode`）。 |
|沒有合成|`PsdBuilder`從呼叫者取得合併影像。如果沒有，它會寫入一個白色佔位符並在資源 1057 中設定`has_real_merged_data = false`。

## 公共 API（概述）

```rust
// Top level
PsdFile::from_bytes(&[u8]) -> Result<PsdFile>
PsdFile::from_bytes_and_merged(&[u8], want) -> Result<(PsdFile, Option<Result<Vec<u8>>>)>
                                            // + decode_merged() when want(&file): ZIP inflated once
PsdFile::to_bytes(&self) -> Result<Vec<u8>>
PsdFile::open(path) / save(path)            // native only
PsdFile { header, color_mode_data, resources, layer_info, layer_info_placement,
          global_layer_mask, global_blocks, layer_mask_trailing, image_data }
PsdFile::layers() -> &[LayerRecord]         // file order = bottom-most first
PsdFile::layers_mut() -> &mut Vec<LayerRecord>
PsdFile::layer(i) -> Option<Layer<'_>>;  iter_layers()
PsdFile::layer_tree() -> Vec<LayerNode>     // nested groups, children bottom-to-top
PsdFile::layer_tree_max_depth(n) -> Vec<LayerNode> // same, groups nested n deep keep no children
PsdFile::decode_merged() -> Result<Vec<u8>> // planar big-endian samples
PsdFile::composite_rgba8() -> Result<RgbaImage>
PsdFile::resource(id), global_block(key), icc_profile(), resolution(),
        has_real_merged_data(), merged_has_alpha(), validate()

// Header
Header { version: Version::{Psd,Psb}, channels, width, height, depth, color_mode, reserved }
ColorMode::{Bitmap, Grayscale, Indexed, Rgb, Cmyk, Multichannel, Duotone, Lab, Unknown(u16)}

// Layers
LayerInfo { merged_alpha, layers: Vec<LayerRecord>, padding }
LayerRecord { rect, channels: Vec<ChannelData>, blend_mode, opacity, clipping, flags,
              filler, mask: MaskData, blending_ranges, name, blocks, extra_trailing }
LayerRecord::name()            // prefers `luni`
LayerRecord::section_divider(), section_type(), layer_id(), fill_opacity(), is_visible(),
             block(key), block_mut(key), channel(id), channel_rect(id), decode_channel(id, depth, version)
ChannelData { id, compression: Option<Compression>, data }   // encoded bytes kept verbatim
ChannelData::encode(id, compression, decoded, w, h, depth, version)
ChannelData::decode(w, h, depth, version); set_decoded(...)
MaskData::{None, Mask(LayerMask), Raw(Vec<u8>)}
LayerMask { rect, default_color, flags, parameters: Option<MaskParameters>, real: Option<RealMask>, trailing, real_first }
BlendMode (27 modes + PassThrough + Unknown([u8;4])); BlendMode::key()/from_key()
Layer<'_>::rgba8() -> Result<RgbaImage>; user_mask() -> Option<Result<GrayImage>>; channel_bytes(id)

// Tagged blocks
TaggedBlock { signature /* 8BIM | 8B64 */, key, data, padding: Option<Vec<u8>> }
TaggedBlock::parsed() -> Option<Result<BlockData>>
BlockData::{UnicodeName, SectionDivider, LayerId, NameSource, BlendClippedAsGroup,
            BlendInteriorElements, Knockout, Protection, SheetColor, FillOpacity, MetadataSetting}
constructors: unicode_name, section_divider, layer_id, name_source, blend_clipped_as_group,
              blend_interior_elements, knockout, protection, sheet_color, fill_opacity
TaggedBlock::check_structure() -> Result<()>   // strict inner re-parse of PlLd, SoLd, SoLE, lnk2/3/D, lfx2
tagged::uses_long_length(version, key)   // PSB 8-byte length keys (the spec's 13 + lnk3, lnkE, pths, extd, extn, FELS, cinf, artd)
LayerInfo::unpadded_len(version), LayerInfo::pad_to(version, 4)   // Photoshop pads the layer info to 4

// Image resources
ImageResource { signature, id, name, data };  ImageResource::parsed() -> Option<Result<ResourceData>>
ResourceData::{ResolutionInfo, LayerState, LayerGroupInfo, Thumbnail, GlobalAngle, IccProfile,
               GlobalAltitude, VersionInfo, Exif, Xmp}

// Compression
Compression::{Raw, Rle, Zip, ZipPrediction, Unknown(u16)}
compression::{encode_planes, decode_planes, PlaneLayout, packbits::{encode, decode}, predict, unpredict}
pixels::{samples_u16, samples_f32, u16_to_bytes, f32_to_bytes, unpack_bits, plane_to_u8}

// Descriptors
descriptor::{Descriptor, VersionedDescriptor, Value, Id, Class, ReferenceItem, UnicodeString, ObjectArray}
Descriptor::from_bytes / to_bytes;  VersionedDescriptor::from_bytes / to_bytes (version 16)

// Builder
PsdBuilder::new(w, h).color_mode(..).depth(8|16).version(..).compression(..).resolution(dpi).icc_profile(..)
builder.push_layer(LayerSpec) / begin_group(GroupSpec) / end_group()? / composite(PixelData)
builder.build() -> Result<PsdFile>;  builder.to_bytes()
PixelData::{Rgba8, Rgba16, GrayA8, GrayA16, Cmyka8, Cmyka16}   // CMYK given as ink, stored inverted

// Test generator (feature `testgen`)
testgen::{all_cases, merged_only, layered, small, sample_descriptor, pattern_plane}
```

## 特徵矩陣

|面積 |閱讀 |寫入 |鍵入 |筆記|
|---|:-:|:-:|:-:|---|
|標頭，PSD v1 / PSB v2 | ✓ | ✓ | ✓ |深度 1/8/16/32；所有 8 種顏色模式加上`Unknown`|
|色彩模式資料| ✓ | ✓ |原料|`composite_rgba8`使用索引調色盤 |
|圖片資源| ✓ | ✓ |部分 | 1005、1024、1026、1033/1036（原始）、1037、1039、1049、1057、1058、1060 |
|圖層記錄| ✓ | ✓ | ✓ |標誌、剪裁、填滿、混合範圍（原始 + 訪問器）|
|圖層遮罩 | ✓ | ✓ | ✓ | 20 / 36 位元組形式、遮罩參數、真實遮罩；無法解析的遮罩保持原始狀態 |
|頻道 ID -1 / -2 / -3 | ✓ | ✓ | ✓ |通過`channel_rect`的每色版矩形 |
|原始/RLE/ZIP/ZIP+ 預測 | ✓ | ✓ | – | 8/16/32 位元預測（32：位元組平面混洗 + 增量）；RLE 計數在 PSD 中為 u16，在 PSB 中為 u32 |
|合併影像資料 | ✓ | ✓ | – |在解析時驗證（大小、RLE 計數、zlib 流，包括校驗和）；聲明超過`MAX_DECODED_BYTES`的 zlib 流會逐字保留，而不會對其進行膨脹，並且對其進行解碼會報告限制 |
|標記塊（原始直通）| ✓ | ✓ | – |保持`8BIM`/`8B64`和精確的填滿 |
| PSB 8 位元組長度金鑰 | ✓ | ✓ | – | LMsk Lr16 Lr32 Layr Mt16 Mt32 Mtrn Alph FMsk lnk2 FEid FXid PxSD |
| Lr16 / Lr32 / Layr 圖層資訊 | ✓ | ✓ | ✓ |提升至`layer_info`；重新插入原來的位置|
| luni lsct lsdk lyid lnsr clbl infx knko lspf lclr iOpa shmd | ✓ | ✓ | ✓ |`TaggedBlock::parsed()`|
| ActionDescriptor（所有作業系統類型）| ✓ | ✓ | ✓ |包含參考文獻、`ObAr`（盡力而為）、`UnFl`、`Pth `、版本 16 包裝器 |
|圖層樹（群組）| ✓ | ✓ | ✓ |開啟/關閉的資料夾以及邊界分隔線；容忍畸形築巢|
| RGBA8擷取| ✓ | – | – | RGB/灰階/CMYK圖層；合併影像也支援索引/點陣圖/雙色調 |
|建造者| – | ✓ | – | RGB / 灰色 / CMYK 8/16 位元、遮罩、群組、混合模式、不透明度、填滿、可見性、剪裁 |
|圖案| ✓ | ✓ | ✓ |`Patt`/`Pat2`/`Pat3`、`.pat`和`.abr`嵌入圖案；非空圖塊允許邊界內色版子矩形，解碼大小受到限制，PackBits 嵌入圖案受限制 |
|行動（`.atn`）| ✓ | ✓ | ✓ |`atn::parse`/`atn::write`：版本 16 套 (Photoshop6+)：帶有功能鍵、修飾符和顏色的動作；帶有事件程式碼或字串名稱、面板名稱、啟用/標誌符號及其描述符名稱及其描述符的動作名稱。精確的往返行程；版本 12 (Photoshop5) 檔案因不支援而被拒絕 |

圖案色版子矩形用零填滿到平鋪矩形。所有頻道
在解碼任何平面之前，都會根據每個平面共享的一個`PatternBudget`檢查元數據
解析模式（區塊、`.pat`檔案）：`MAX_DECODED_BYTES`已解碼，512 MiB 為零
總共填滿。`parse_pattern_block_with`讓一個文件的多個區塊共享預算；
`.abr`模式計入檔案的樣本預算。模式的所有色版必須具有
相同的深度。完全填滿的色版重複使用其解碼緩衝區，無需第二個副本；
填滿色版的分配在失敗時傳回錯誤。

### 保留原始數據，尚未輸入（計劃用於 M8 的解析器）

* 圖層效果：`lfx2`、`lrFX`、`lmfx`。裡面的描述子可以用`VersionedDescriptor::from_bytes(&block.data[4..])`按需解析。
* 類型層：`TySh`、`tySh`以及其中的 EngineData。
* 智慧型物件：`SoLd`、`PlLd`、`SoLE`、`lnk2`、`lnkD`、`lnk3`。
* 向量遮罩和形狀：`vmsk`、`vsms`、`vogk`、`vscg`。
* 調整和填滿圖層：`curv`、`levl`、`hue2`、`SoCo`、`GdFl`、`PtFl`，...，加上`Patt`、`Txt2`、`FMsk`、3D 和視頻塊。
* 全域圖層遮罩資訊（原始，帶有存取器）、縮圖（原始）、切片、參考線、路徑和所有其他資源 ID。

## 測試

`cargo test -pphotocraft-psd--all-features`執行單元測試、整合測試和 proptest：

* 每個生成案例的位元組穩定性和模型往返：所有深度、顏色模式和壓縮、PSD 和 PSB。
* PackBits 邊緣情況。
* 8/16/32 位元預測的已知向量。
* PSD 與 PSB 長度處理。
* 截斷掃描：幾個小檔案的每個前綴必須回傳`Err`。
* 隨機突變和隨機位元組屬性測試：永遠不要驚慌。
* 分層樹案例。
* 生成器 →`rgba8`正確性。
* 未知的混合模式和未知的區塊不變地通過。

當此目錄存在時，`tests/corpus.rs`會在工作區根目錄處遍歷`corpus/psd/**/*.{psd,psb}`。對於每個文件，它斷言解析成功並且文件逐位元組往返，並列印每個文件的結果。如果沒有目錄，它會默默地跳過。

`cargo-fuzz`骨架住在`fuzz/`中。它是它自己的工作區，不包括在photocraft的工作區中。使用`crates/psd`中的`cargo +nightly fuzz run parse`（或`descriptor`）執行它。

## 規範的歧義與決策

* **標記塊填滿。 ** 規範表示長度「四捨五入為偶數位元組數」。在實踐中，編寫器要么在長度中包含填滿，要么將其附加在資料後面，然後填滿到 2 或 4。解析器偵測實際跟隨每個區塊的填滿：最小的 k ≤ 3，之後出現下一個簽名或區域的結尾。它會儲存該填滿，除非它與預設值相符。
* **圖層資訊填滿**（2 vs 4）以相同的方式記錄。
* **當主層資訊為空時，從全域`Lr16`/`Lr32`區塊讀取 16/32 位元層**。區塊資料是沒有內部長度欄位的層資訊主體。
* **遮罩記錄佈局。 ** 順序是：矩形、預設顏色、標誌、參數（當設定標誌位元 4 時），然後是實際標誌、背景和矩形（只要剩餘至少 18 個位元組）。剩下的任何內容（例如 20 個位元組形式的 2 個填滿位元組）都保留在`trailing`中。
* **遮罩頻道矩形。 ** 頻道 -2 使用遮罩矩形。頻道 -3 使用真實遮罩矩形。
* **合併的 ZIP 資料** 是所有平面上的單一 zlib 流。預測應用於所有平面上的每行。
* **深度 1** 的 ZIP 預測被視為逐位元組增量。
* **`ObAr`** 被讀取為 u32 前綴，後面跟著描述符形狀的主體，跟隨ag-psd的讀取。規範沒有記錄它。
* **描述符`bool`** 除 0/1 之外的位元組在寫入時標準化為 1。
* **Pascal 名稱** 將其填滿位元組寫為零。僅包含零層資訊長度的層和遮罩部分被標準化為空部分。這些是唯一已知的非位元組精確情況，而且都是病態的。
* **標頭限制。 ** 兩個版本的解析器最多接受 300 000 × 300 000。`PsdFile::validate()`強制執行 PSD 規範的 30 000 個限制。
