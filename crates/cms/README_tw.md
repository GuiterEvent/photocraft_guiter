#photocraft-cms

適用於PhotoCraft的 Pure-Rust ICC 色彩管理。無工作區相依性性（L0，可發布），
沒有C，沒有`unsafe`。從公開實施 ICC.1:2010 (v4.3) 和 ICC.1:2001-04 (v2)
規格、CIE 15 和 Adob​​e 發布的黑點補償文件。

## API

```rust
use photocraft_cms::{Builtin, Intent, Profile, Transform};

let src = Profile::parse(&icc_bytes)?;                    // v2/v4: matrix/TRC, gray TRC, mft1/mft2/mAB/mBA
let dst = Builtin::CoatedCmyk.profile();
let t = Transform::new(&src, dst, Intent::RelativeColorimetric, /*bpc*/ true)?;
t.convert_u8(&rgba8, 4, &mut cmyka8, 5, /*copy alpha*/ true);   // also convert_u16 / convert_f32 / apply (in place)
let mut out = [0.0; 4];
t.eval(&[1.0, 0.0, 0.0], &mut out);                        // exact single colour
```

*`Transform::proof(src, proof, display, intent, bpc, simulate_paper)`：防軟鏈條。
*`GamutCheck::new(src, proof, ΔE)`：色域外測試（色域警告）。
*`Lut3d::from_transform(&t, 33).to_rgba16f_bytes()`：將變換顯示為 3D 紋理。
*`transform::cached(src, dst, opts)`：處理程序範圍的轉換快取。
*`Profile::to_bytes()`：解析設定檔的原始位元組（位元組精確），或 ICC v4.3 編碼。
*`Profile::same_colors(&other)`：無論編碼如何，相同的色彩空間（每個樣本
  ΔE 0.5 內的設備網格（透過兩者），例如Photoshop的sRGB IEC61966-2.1和內置
  sRGB。`content_hash()`比較位元組並且僅適用於快取鍵。

＃＃＃ 評估

* 精確路徑：浮動階段管線（曲線、矩陣、CLUT、Lab↔XYZ、PCS 編碼、BPC /
  絕對縮放）。浮點緩衝區預設使用它（`TransformOptions::precise_float`）。
* 整數緩衝區：矩陣/TRC 鏈使用每個程式碼的輸入表 + 矩陣 + 索引的輸出表
  通過`x^(1/4)`（即使對於純伽馬曲線也能精確到接近黑色）；包含 CLUT 的鍊是
  取樣到設備連結（33³ 用於 3 個輸入，17⁴ 用於 4 個輸入，4096 用於 1 個輸入）進行評估
  四面體插值（4D：兩個四面體之間的第一個輸入呈線性）；純粹地
  分析鏈（例如 Lab ↔ RGB）被精確評估。在人造絲上執行 ≥ 16k 像素的緩衝區
  （僅限本地）。
* 渲染意圖：AToB0/1/2 和 BToA0/1/2 按意圖，並附有 AToB0/BToA0 後備；絕對的
  按介質白色比色標度（`wtpt`；v2 顯示設定檔視為 D50；零，
`CmsError::Invalid`拒絕負白或非有限白；BPC用途
  中性黑點（僅限 Y）在 XYZ 中線性映射。

## 內建設定檔（所有CC0-1.0，由該板條箱產生）

|編號 |說明 |
|---|---|
|`srgb`| sRGB IEC61966-2.1（Rec. 709 原色、D65 Bradford 適應、sRGB TRC）|
|`display-p3`|顯示 P3 |
|`adobe-rgb-compat`| Adob​​e RGB (1998) 相容（已發布的原色，伽馬 563/256）|
|`prophoto-compat`|相容於 ProPhoto（ROMM 原色、D50、伽瑪 1.8）|
|`linear-srgb`| sRGB 原色，線性 |
|`rec2020`|推薦。 2020 年初選，建議。第709章
|`gray-gamma-2.2`、`sgray`|灰階伽瑪2.2；sRGB 曲線的灰色（預設灰色工作空間）|
|`lab-d50`|實驗室身分（v4 編碼）|
|`coated-cmyk`| **PhotoCraft塗層 CMYK（合成，300% TAC，中等 GCR）** — 預設 CMYK |

### CMYK 設定文件

Adobe 的 CMYK 設定檔（U.S. Web Coated SWOP、Coated FOGRA39）是專有的，並且可以免費使用
可下載的基於特徵的設定檔（ECI/FOGRA、彩色、Ghostscript）
再分發條件或不明確的許可證，因此PhotoCraft產生自己的
(`src/synth.rs`)：Yule–Nielsen 修改的 Neugebauer 模型 (n = 2)，涵蓋 16 個套印
ISO 12647-2 塗佈紙實驗室的目標是紙張/C/M/Y/RGB 套印、拋物線網點增大（14% CMY、
17 % K（50 %），GCR 黑色生成（從 20 % 灰色分量開始，K 最大 95 %），總墨水
限制 300 %，具有 L* 黑點縮放和軟色度壓縮的感知表
取樣的色域邊界。`profiles/photocraft-coated-cmyk.icc`（237 KB，AToB 11⁴ lut16，BToA
21³ lut16) 是此輸出，專用於公共領域 (CC0-1.0)。再生與

```sh
PHOTOCRAFT_REGEN_PROFILES=1 cargo test -p photocraft-cms --release --test regen
```

這是一個合理的塗層膠印輪廓，而不是測量的特徵：使用印表機的
生產列印的設定檔。

## 未實現/近似

* v4.4`mpet`（multiProcessElements、D2Bx/B2Dx）標籤、命名顏色、裝置連結和摘要
  設定檔作為變換端點，基於`chad`的 v2 絕對比色法適應。
* BPC目的地黑色直接使用BToA/AToB往返（Adobe的曲線擬合
  行為不良的 LUT 設定檔未實現）； v2/v4感知參考黑色混合不是
  建模超出 v4 LUT 設定檔的 PRMG 黑色。
* 寫入時不計算設定檔 ID (MD5)（留零，這是規範允許的）。
