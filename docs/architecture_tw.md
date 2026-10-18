#PhotoCraft架構

> **最後審查：** 2026-10-10 · **最後更新：** 2026-10-13 · **更改：** 次要（重新計算工作區大小和測試計數；添加當前狀態摘要；保留下面的設計計劃，並列出其偏差） · **目標：** Adob​​ePhotoshop200262

## 目前狀態 (2026-10-10)

今天`main`上存在什麼。下面編號的部分是最初的設計方案
2026年9月30日；如果程式碼不同，則此摘要獲勝。

- **工作區：**`crates/`下的 25 個庫箱、4 個應用程式（`photocraft`桌面、`photocraft-cli`、
`photocraft-web`、`photocraft-web-boot`)、`xtask`，一個範例插件和 5 個模糊套件； ~539,000 行 Rust
  約 1,310 個檔案；約 8,040 個`#[test]`函數（ui-egui2,809、引擎 1,796、演演算法 572、io 539、
  編解碼器 426，psd 341），統計日期為 2026 年 10 月 13 日。板條箱地圖和圖層：[`AGENTS.md`](../AGENTS.md) §2 和
`xtask/src/layers.rs`（由`cargo xtask layers`強制執行）。
- **UI 工具包：** 決定：wgpu上的egui/eframe（無 Web 視圖，無 JavaScript）。相同的 Rust 構建
  瀏覽器為WebAssembly（trunk + wasm-bindgen）。
- **引擎：**`crates/engine`中註冊了約 700 個命令（`commands.rs`中的`CommandSpec`）；的
  UI、CLI、JSON 控制色版和 MCP 伺服器均按命令 ID 進行調度。 628Photoshop選單項
  映射到它們（`crates/ui-egui/src/menu_catalog.rs`）。
- **渲染：** CPU 合成器（`compose`，預言機）和wgpu合成器（`gpu`，≤ 1/255）
  中央處理器）；透過我們自己的 ICC 引擎 (`cms`) 進行顏色轉換，將顯示轉換折疊到 GPU 3D LUT 中。
- **檔案：**自己的 PSD/PSB 箱 (`psd`)、平面編解碼器 (`codecs`)、相機原始資料 (`raw`)、HEIF (`heif`)、
Affinity(`affinity`)、文件映射 (`io`)、本機`.pcraft`（`format`，原子保存）。
  覆蓋範圍：[`file-format-parity.md`](file-format-parity.md)。
- **尚未根據計劃建置：**`ml`箱（無 ML 執行階段；請參閱間隙 G15），插件 ABI v2。
- **與以下計劃的偏差：**`photocraft-tablet`是允許的一個板條箱`unsafe`
  （AppKit 筆事件）；新增了`affinity`、`heif`和`raw`板條箱；第 15 節中的分階段路線圖
  被 [`roadmap.md`](roadmap.md) 取代。

---

# 設計方案 (2026-09-30)

該計劃規定：
- 貨物工作區，
- 保持引擎獨立於 UI 的規則，
- UI 必須實現的介面，
- PSD、合成、歷史和自動化部分如何組合在一起。

---

## 1. 目標與原則

**目標**
- 原生、可移植（macOS / Windows / Linux）、完全開源的分層影像編輯器，隨著時間的推移大致匹配Photoshop的功能表面。
- **UI 可以在不觸及核心的情況下進行交換**：今天是egui，也許以後是 Slint、Qt 或其他東西。
- **核心編譯為`wasm32`，** 這使得瀏覽器演示成為可能。
- PSD 讀寫位於其**自己的板條箱中，可以單獨發布。 **

**原則**

1. **引擎優先，無頭優先。 ** 每個功能都可以在沒有 GUI 的情況下實現：透過測試、CLI 和 MCP。 GUI 是引擎的用戶端之一。
2. **一切都是命令。 ** 每個使用者可見的操作都有一個穩定的`CommandId`（例如`filter.blur.gaussianBlur`），具有類型化、可序列化的參數。選單、命令面板、快捷方式、記錄的操作/巨集、CLI、MCP 和插件都調度相同的命令。
3. **資料描述UI；工具包繪製它。 **工具產生*覆蓋基元*（線、手把、行進螞蟻路徑）作為資料。過濾器和調整對話​​框是從*參數模式*產生的。新的工具包為每個對話方塊實作一個渲染器，而不是大約 150 個自訂對話方塊。
4. **一種演演算法，一個參數結構體。 **每種演演算法都有一個CPU參考實作（Rust + rayon，確定性）。有些還獲得 GPU 實作 (WGSL)。兩者讀取相同的`#[repr(C)]``bytemuck::Pod`參數結構。我們從不保存多種語言的演演算法的手動同步副本。
5. **不可變快照，寫時複製圖塊。 ** 像素資​​料儲存在`Arc`共享的 256² 圖塊中。拍攝文件快照的時間複雜度為 O（層數），這使得撤銷、後台作業、自動儲存和 UI 讀取成本低廉且無鎖定。
6. **確定性。 ** 相同的輸入在 CPU 和 GPU（在容差範圍內）、執行緒計數以及預覽和導出之間提供相同的輸出。雜訊是根據文件座標進行雜湊處理的。
7. **預設純 Rust。 ** C/C++ 相依性僅允許在 I/O 邊緣 crate 中的 Cargo 功能後面（例如可選的 LibRaw）。
8. **乾淨的房間。 **我們只研究其他編輯者的*行為*，我們不能**複製專有原始碼**（Rust、WGSL、C++ 或 JS），即使它是作為可讀原始碼提供。規範來自公共格式文件（Adobe PSD 規範、ISO/ICC）、學術論文（PatchMatch、泊松混合、ARAP）和觀察到的行為。切勿貼上其他產品的程式碼。

### 1.1 避免GIMP的漏洞

對 GIMP 的批評在歷史上是真實的，其中大部分在 2026 年仍然適用。

**位元深度。 **
- GIMP 的原始核心硬編碼每聲道 8 位元圖塊。
- 高位元深度需要全新的引擎 GEGL。 GEGL 於 2000 年左右開始，在 2.6 (2008) 中作為可選引入，並且該端口僅在 **GIMP 2.10 (2018)** 中完成，這是僅該端口“六年工作的結果”。
- 直到那時，GIMP 才獲得 16/32 位元整數和浮點處理。

**非破壞性編輯。 **
-Photoshop大約從 1994 年開始就有調整層。
- 由於 GTK3 端口，GIMP 在 **3.0（2025 年 3 月）** 中獲得了非破壞性*過濾器*，大約在 2.10 後七年。
- **3.2（2026 年 3 月）** 中獲得了非破壞性的*圖層類型*（連結圖層、向量圖層）。

**CMYK。 **
- GIMP 3.x 內部仍然是「100% RGB 編輯器」。
- CMYK 僅作為導出 (JPEG/TIFF/PSD) 和軟打樣存在。

**PSD。 **
- 導入仍然有損。Photoshop調整圖層和大多數圖層樣式都沒有忠實地繼承，文字經常需要重新輸入。
- 3.2 將傳統的投影和內部陰影轉換為 GIMP 濾鏡，並添加了 PSB 導出。

**根本原因：**基本屬性**作為假設而不是數據被納入核心。 **像素深度、色彩模型和「圖層是像素緩衝區」都是硬編碼的。後來的每一個觸及他們的功能都需要重寫建構在其之上的所有內容。

資料來源：[GIMP 2.10 發行說明](https://gimp.org/release-notes/gimp-2.10.html)、[維基百科上的 GEGL](https://en.wikipedia.org/wiki/GEGL)、[GIMP 3.2 發行說明](https://www.gimp.org/release-notes/gimp-3.2.html)、[GIMP 3.0 顏色上的 LWN66](XZPH65ZZX)、[GIMP 3.0 顏色上的 LWN6](XZPH665ZXZ)。

**我們從第一天開始就採用的規則**（現在很便宜，並且每個規則都會在以後強制重寫）：

|物業 |規則|哪裡 |
|---|---|---|
| **位元深度** |每個表面都帶有執行階段`PixelFormat { depth: U8\|U16\|F16\|F32, model, alpha }`。演演算法通常是透過`Sample`特徵編寫的，並按深度進行單態化，或在 f32 工作緩衝區上執行並在圖塊邊緣進行轉換。任何公共引擎 API 中都不會出現`u8`像素類型。 CI 在 8 樓、16 樓和 32 樓運營黃金套房。 |`raster`、`color`、`algo`|
| **顏色模型** |`ColorMode`（RGB、灰色、CMYK、Lab、索引、點陣圖以及多色版/專色）從一開始就位於`doc`中。色版數是動態的（1–N 加 alpha 加點）。只有 RGB/灰色*渲染*首先發貨，但儲存、PSD 往返和轉換路徑從不假設有 3 或 4 個色版。 |`doc`、`raster`、`psd`|
| **色彩管理** |每份文件都有一個 ICC 設定檔。合成器採用工作空間描述符。顯示轉換是一個單獨的最後階段。混合空間（伽瑪與線性）是每個文件的選項。 |`color`、`compose`|
| **施工無損** |`LayerContent`是一個枚舉，包括 v1 中的調整、填滿、文字、形狀和智能，以及智慧濾鏡作為任何圖層上的堆疊。合成器評估*圖表*，而不是展平緩衝區的清單。破壞性操作只是“應用並烘烤”。 |`doc`、`compose`|
| **檔案格式** |格式透過`trait ImageDecoder/Encoder`以及帶有功能標誌（深度、模式、圖層、alpha、元資料、ICC）的註冊表插入。新增格式是一個板條箱功能。它永遠不需要改變核心類型。無損 PSD 往返保留未知區塊。 |`codecs`、`io`、`psd`|
| **尺寸** |`TileStore`特徵背後的平鋪稀疏存儲，具有 PSB 規模畫布 (300k × 300k) 的 64 位元座標。沒有什麼假設文件適合一種分配或一種 GPU 紋理。 |`raster`|
| **使用者介面工具包** |核心從不相依性工具包（§3–§4）。 GIMP 花了數年時間將 GTK2 移植到 GTK3，而我們的 UI 箱可以在不接觸引擎的情況下進行更換。 |分層規則2 |

測試很簡單：**新增 64 位元浮點、新的顏色模型或新的檔案格式應該觸及 Leaf crate，而不是基礎。 ** 我們在第 2 階段透過儘早新增（隱藏的）CMYK 和 32f 程式碼路徑（甚至在它們在 UI 中公開之前）來檢查這一點。

---

## 2. 工作區佈局

```text
photocraft/
├─ Cargo.toml                  # [workspace], shared deps, lints, profiles
├─ crates/
│  │  ── foundation (no_std-friendly where practical, wasm-safe) ──
│  ├─ geom/                    photocraft-geom      points, rects, affine/perspective, tile coords, bezier (kurbo)
│  ├─ cms/                     photocraft-cms       pure-Rust ICC colour management: profiles, transforms, intents, BPC, soft proof, 3D LUTs
│  ├─ color/                   photocraft-color     pixel formats, color spaces, blend-mode math (scalar reference); conversions via cms
│  ├─ raster/                  photocraft-raster    tiled COW surfaces, masks, mip pyramids, damage regions, pixel iterators
│  │  ── document ──
│  ├─ doc/                     photocraft-doc       document model: layer tree, masks, effects, channels, paths, guides, metadata (pure data + serde)
│  ├─ ops/                     photocraft-ops       reversible operations on a doc + transactions + history/undo
│  │  ── algorithms (CPU reference impls, rayon, wasm-safe) ──
│  ├─ algo/                    photocraft-algo      adjustments, filters, selection, inpaint/heal, warp/transform, resampling
│  ├─ paint/                   photocraft-paint     brush engine: dabs, dynamics, stroke smoothing, brush presets (.abr import later)
│  ├─ text/                    photocraft-text      font DB trait, shaping + layout (parley/skrifa/harfrust), text-layer rasterization
│  ├─ vector/                  photocraft-vector    shapes, paths, strokes → coverage (vello_cpu / kurbo)
│  │  ── rendering ──
│  ├─ compose/                 photocraft-compose   layer tree → DrawOp plan; CPU compositor (reference + export + wasm fallback)
│  ├─ gpu/                     photocraft-gpu       wgpu backend for the DrawOp plan + GPU kernels (WGSL), residency cache, memory budget
│  ├─ viewport/                photocraft-viewport  camera (zoom/pan/rotate), visible tiles/mips, renders a doc into a wgpu texture; toolkit-agnostic
│  │  ── I/O ──
│  ├─ psd/                     photocraft-psd       PSD/PSB read + write; its OWN format-level model; depends on nothing in this workspace
│  ├─ adobe-assets/            photocraft-adobe-assets  .abr .asl .aco/.ase .grd .pat .csh .atn .cube/.3dl, ACR .xmp presets (standalone, like psd)
│  ├─ codecs/                  photocraft-codecs    flat raster decode/encode at format-supported depths; AVIF encode only (feature `avif`), HEIF/HEIC decode only (feature `heif`); no JPEG XL
│  ├─ heif/                    photocraft-heif      optional HEIF/HEIC decoder (heic-rs, pure Rust); used only by codecs behind its `heif` feature (standalone)
│  ├─ affinity/                photocraft-affinity  bounded native Affinity reader: archive, object stream, layers, vectors, text, pixels (standalone; read only)
│  ├─ raw/                     photocraft-raw       clean-room camera RAW decode (DNG, CR2, TIFF/EP) + develop pipeline (standalone, like psd)
│  ├─ format/                  photocraft-format    native document format (.pcraft bundle): manifest + content-addressed tiles
│  ├─ io/                      photocraft-io        import/export orchestration; doc ⇄ PSD mapping; PDF/SVG import (features)
│  │  ── intelligence ──
│  ├─ ml/                      photocraft-ml        model registry + InferenceBackend trait (ort native / burn|candle web); SAM2, matting, depth, sky
│  ├─ plugins/                 photocraft-plugins   sandboxed WebAssembly filter plug-ins (wasmi host, limits, ABI v1, registry)
│  │  ── the façade every frontend talks to ──
│  ├─ tools/                   photocraft-tools     tool state machines (move, marquee, lasso, brush, clone, gradient, crop, transform, pen, text…)
│  ├─ engine/                  photocraft-engine    Session: open docs, command registry + dispatch, jobs, events, view-models, preferences
│  │  ── platform + frontends ──
│  ├─ platform/                photocraft-platform  traits: file dialogs, clipboard, fonts, tablet input, menus, storage; native + web impls
│  ├─ ui-egui/                 photocraft-ui-egui   the (first) GUI shell: panels, dialogs, canvas widget, theme
│  ├─ automation/              photocraft-automation  MCP server (rmcp) + JSON-RPC over the command registry
│  └─ testkit/                 photocraft-testkit   golden images, perceptual diff, fixtures, PSD corpus helpers
├─ apps/
│  ├─ photocraft/              desktop binary (winit + wgpu + ui-egui + platform-native)
│  ├─ photocraft-cli/          headless batch CLI (open → commands → export), also hosts `mcp` subcommand
│  ├─ photocraft-web/          wasm32 binary (ui-egui + platform-web), demo
│  └─ photocraft-web-boot/     wasm32 binary that starts the packaged web app from parts of its wasm
├─ shaders/                    (or inside gpu/) WGSL, with generated struct headers
├─ assets/                     icons, bundled fonts, default brushes/swatches/presets
├─ fuzz/                       cargo-fuzz targets (psd, codecs, format)
├─ xtask/                      build tasks: layer-check, shader-gen, corpus fetch, release
└─ plan/                       this directory
```

**今天存在的東西。 **此佈局是目標設計。目前已完成：`geom`、`cms`、`color`、`raster`、`psd`、`codecs`、`doc`、XZZPH9999X、PH97ZZX、`doc`、XZZPH9999X、XZZ100Z`vector`、`compose`、`gpu`、`format`、`raw`、`io`、`plugins`、`engine`、ZZX、`plugins`、`engine`、ZZZX`testkit`，以及三個應用程式。尚未拆分：`viewport`和`tools`位於`ui-egui`和`engine`內；`platform`服務是每個應用程式（`ui_egui::Services`）注入的函數鉤子；

**板條箱粒度：**從上面的板條箱開始。一旦任何模組傳遞了大約 10k 行，或編譯時間受到損害，則將`algo`拆分為`-adjust`、`-filters`、`-select`、`-inpaint`和`-warp`。它的內部模組邊界應該已經遵循這些線。

**命名：**目錄很短，包裝的字首為`photocraft-`。如果我們將`photocraft-psd`發佈到 crates.io，它將獲得一個中立的、可發布的名稱（例如`psd-rw`）。

---

## 3. 相依性分層（強迫）

```text
 L7  apps/*                         (binaries: wire everything together)
 L6  ui-egui · automation · platform
 L5  engine
 L4  tools · viewport · io · ml · plugins
 L3  compose · gpu · format
 L2  ops · algo · paint · text · vector
 L1  doc
 L0  geom · cms · color · raster           psd, codecs, raw, adobe-assets (standalone, no workspace deps)
```

**規則**（由 CI 中的`cargo xtask layers`檢查，解析`cargo metadata`）：

1. 一個 crate 可能只相依性**較低**層中的 crate。沒有循環，也沒有橫向相依性性，除非列出了。
2. **L6 以下的任何內容都不能相依性任何 UI 工具包、winit或`platform`實作。 ** 平台服務透過`engine`（或`platform`的僅限特徵的核心）中定義的特徵到達引擎，並在啟動時注入。
3. **`photocraft-psd`不相依性工作空間箱。 ** 文件 ⇄ PSD 映射位於`io`中。這使得 PSD 套件可以被其他專案發布和重複使用。這同樣適用於每個獨立的 crate，但有一個已記錄的例外：`codecs`→`heif`（獨立的和可發布的；`STANDALONE_EXCEPTIONS`在`xtask/src/layers.rs`中）。
4. **wasm 閘：** L0–L5 中的每個 crate（除了功能閘控的本機後端）都必須為`wasm32-unknown-unknown`建置。 CI 執行`cargo build -pphotocraft-engine--target wasm32-unknown-unknown --no-default-features --features web`。
5.`gpu`為`engine`選配。引擎功能為`gpu`（預設開啟）和`cpu-only`，`cpu-only`建置是無頭 CLI 和 CI 測試使用的。
6. **C 相依性**（可選 LibRaw、pdfium）僅在功能後面，僅在`codecs`、`raw`或`io`中，且預設不會對 Web 目標啟用。

---

## 4. UI-swap接縫（最重要的介面）

前端需要核心的五件事。保持這些小和麵向資料是 UI 可替換的原因。

### 4.1`Session`：指令輸入，事件輸出

```rust
// photocraft-engine
pub struct Session { /* docs, registry, job pool, prefs, services */ }

impl Session {
    pub fn new(services: Services) -> Self;                         // platform traits injected
    pub fn dispatch(&self, doc: Option<DocId>, cmd: CommandInvocation) -> Result<JobHandle, CommandError>;
    pub fn registry(&self) -> &CommandRegistry;                     // metadata for menus/palette/shortcuts
    pub fn snapshot(&self, doc: DocId) -> Arc<DocSnapshot>;         // lock-free read (arc-swap)
    pub fn events(&self) -> EventReceiver;                          // DocChanged{damage}, JobProgress, Toast, …
    pub fn view_models(&self, doc: DocId) -> ViewModels;            // layers, history, channels, tool options…
}

pub struct CommandInvocation { pub id: CommandId, pub params: serde_json::Value /* validated vs schema */ }
```

### 4.2`CommandRegistry`：選單、調色盤和自動化目錄

每個指令都會註冊以下內容：
-`id`(`"filter.blur.gaussianBlur"`)、`label`、`menu_path`(`["Filter","Blur"]`) 和`default_shortcut`。
-`params: schemars::Schema`，加上選購的`ui_hints`（滑桿範圍、單位、對​​數刻度、分組）。
-`enabled(&DocSnapshot, &ToolState) -> bool`和`checked(...) -> Option<bool>`。
-`preview: PreviewMode`（無/即時/代理）。這可以讓對話方塊一般顯示畫布上的預覽。
-`kind`：即時/對話/互動式（手動控制工具，例如自由變換）。

**消費者：**
- GUI 從中建立選單列、⌘F 調色板和快捷方式編輯器。
-`automation`將註冊表公開為 MCP 工具。
-`photocraft-cli`暴露`photocraft-clirun in.psd --cmd filter.blur.gaussianBlur --params '{"radius":4}' --out out.png`。
- 動作錄製簡單來說就是`Vec<CommandInvocation>`。

### 4.3 工具：指標事件輸入、操作和覆寫輸出

```rust
// photocraft-tools
pub struct PointerEvent { pub pos_doc: Point, pub pressure: f32, pub tilt: Vector, pub twist: f32,
                          pub time_us: u64, pub buttons: Buttons, pub mods: Modifiers, pub kind: PointerKind /* mouse|pen|eraser|touch */ }

pub trait Tool {
    fn id(&self) -> ToolId;
    fn options_schema(&self) -> Schema;                       // options bar generated from schema
    fn on_pointer(&mut self, ev: &PointerEvent, cx: &mut ToolCx) -> ToolResponse;   // ToolCx gives snapshot + op sink
    fn on_key(&mut self, key: Key, cx: &mut ToolCx) -> ToolResponse;
    fn overlay(&self, view: &ViewTransform) -> Vec<OverlayPrim>;                     // drawn by any UI
    fn cursor(&self) -> CursorSpec;
}

pub enum OverlayPrim { Path{path: BezPath, style: StrokeStyle, space: Space}, MarchingAnts{path: BezPath},
                       Handle{pos: Point, kind: HandleKind}, BrushOutline{center: Point, radius: f32, ..},
                       Text{pos: Point, text: String}, Guide{..}, Grid{..} }
```

合併的高速率筆樣本來自`platform`，並作為批次交給工具，因此即使 UI 以 60-120 Hz 渲染，筆刷也能看到每個樣本。

### 4.4 視窗：文件到像素

```rust
// photocraft-viewport
pub struct Viewport { pub camera: Camera /* zoom, pan, rotation, dpr */, /* tile/mip cache */ }
impl Viewport {
    pub fn render_gpu(&mut self, snap: &DocSnapshot, gpu: &GpuContext, target: &wgpu::TextureView, damage: &Region);
    pub fn render_cpu(&mut self, snap: &DocSnapshot, target: &mut RgbaImageMut, damage: &Region); // wasm fallback, tests
    pub fn doc_to_screen(&self) -> Affine;  pub fn screen_to_doc(&self) -> Affine;
}
```

- 任何可以託管`wgpu::TextureView`（egui繪製回調、Slint 的wgpu紋理導入、Bevy、原始winit表面）的工具包都以相同的方式嵌入畫布。
- 沒有wgpu的工具包可以回退到`render_cpu`。

### 4.5 視圖模型與模式

- 面板（圖層、歷史記錄、色版、路徑、屬性、導航器、分佈圖、樣本）讀取從快照產生的簡單結構，例如`LayersModel { rows: Vec<LayerRow{ id, name, kind, depth, visible, locked, thumb: ThumbHandle, blend, opacity, has_mask, effects: Vec<..> }> }`。縮圖是 UI 透過引擎縮圖快取解析為紋理的`ThumbHandle`。
- 對話方塊是從指令`params`模式產生的。工具包可以註冊一些複雜的指令*自訂*對話方塊小工具：曲線、色階、Camera Raw、液化、圖層樣式。

**接縫的石蕊測試：**`apps/photocraft-cli`和自動化伺服器必須能夠完成 GUI 所做的“所有事情”，除了指針繪製之外，它們通過工具重播`PointerEvent`流來完成指針繪製。

---

## 5.文件模型（`photocraft-doc`）

```rust
pub struct Document {
    pub id: DocId, pub size: Size, pub resolution_dpi: f32,
    pub mode: ColorMode /* Rgb | Gray | Cmyk | Lab | Bitmap | Indexed */, pub depth: Depth /* U8 | U16 | F32 */,
    pub profile: IccProfileRef,
    pub root: LayerGroup,            // tree
    pub channels: Vec<AlphaChannel>, // saved selections/spot
    pub paths: Vec<NamedPath>, pub guides: Guides, pub slices: .., pub metadata: Metadata /* XMP, EXIF */,
    pub selection: Option<Mask>,     // current selection (a raster mask + optional vector outline)
}

pub struct Layer { pub id: LayerId, pub name: String, pub visible: bool, pub locks: Locks,
                   pub blend: BlendMode, pub opacity: f32, pub fill_opacity: f32, pub clipped: bool,
                   pub mask: Option<RasterMask>, pub vector_mask: Option<VectorMask>,
                   pub effects: Effects, pub blend_if: BlendIf, pub content: LayerContent }

pub enum LayerContent {
    Raster(TiledSurface),
    Group(LayerGroup /* children, pass-through or isolated */),
    Adjustment(Adjustment /* Curves, Levels, HueSat, … as params */),
    Fill(Fill /* solid | gradient | pattern */),
    Text(TextLayer /* runs, paragraph styles, warp; cached raster */),
    Shape(ShapeLayer /* vector paths + fill/stroke */),
    Smart(SmartObject /* embedded or linked source + transform + smart filters */),
}
```

- **像素。 **`raster::TiledSurface<F>`是從`TileCoord`到`Arc<Tile<F>>`的稀疏地圖，具有 256×256 個圖塊。缺少的圖塊是透明的，或採用填滿值。突變呼叫`Arc::make_mut`，因此它是寫入時複製。
- **圖塊產生計數器**為 GPU 駐留快取和縮圖快取提供資料。
- **Mip 金字塔**是每層延遲導出的，用於縮小視圖。
- **大文件。 ** 磁貼儲存背後的`TileStore`特徵：現在在記憶體中，稍後磁碟溢位（「暫存碟」），在全域記憶體預算下進行 LRU 驅逐。
- **第一個里程碑支援 RGB 和灰色，為 8/16/32f。 ** CMYK 和 Lab 從第一天起就以類型進行建模（因此 PSD 往返不會丟失它們），但在後期階段之前透過轉換為 RGB 進行渲染。

---

## 6. 操作與歷史（`photocraft-ops`）

- **每個突變都是`Op`**，例如`SetLayerProps`、`PaintTiles{layer, tiles_before, tiles_after}`、`AddLayer`、`MoveLayer`或`ApplyFilter`。
- **指令產生帶有標籤（「高斯模糊」）的`Transaction`操作**。
- **歷史記錄儲存交易。 ** 因為圖塊是`Arc`共享的，「之前」狀態只花費實際更改的圖塊。撤銷意味著交換`Arc`，因此不會建立像素副本。
- **每個狀態都會記住其目標圖層**（活動圖層和圖層選擇，就像建立狀態時一樣：開啟文件時，或在建立文件的步驟之後）。選擇圖層不是一個步驟，也不會更改任何狀態的目標，但撤銷和重做會再次以恢復狀態的圖層為目標，就像Photoshop所做的那樣。
- **筆劃**每次筆劃累積為一筆交易。圖塊隨損壞區域一起增量發布，因此視口會即時更新。
- **為讀者提供的快照。 ** 每次提交操作後，引擎都會透過`arc-swap`發布新的`Arc<DocSnapshot>`。 UI、背景作業（過濾器、AI 推理、匯出）和自動儲存無鎖定讀取快照。
- **後台作業** 從快照 N 進行計算並作為事務提交。如果文件同時發生更改，則作業會重新設定基準，這表示它會重新定位相同的圖層 ID（如果它仍然存在），或報告衝突。
- **歷史記錄功能：**歷史記錄面板、具有最舊優先驅逐的記憶體上限以及命名快照。歷史筆刷稍後出現，讀取過去狀態的圖塊。

---

## 7. 渲染

### 7.1 規劃器 (`compose`)

`plan(&DocSnapshot, region) -> Vec<DrawOp>`將層樹扁平化為線性指令流：
-`PushGroup{isolated, blend, opacity}`/`PopGroup`
-`BeginClipBase`/`Clipped`
-`DrawTiles{layer, blend, opacity, mask}`
-`Adjust{program}`
-`Effect{kind, params}`
-`Fill{...}`

兩個後端都使用相同的計劃。這是唯一編碼Photoshop層語義的地方：直通組、剪切組、填滿與不透明度、blend-if 和剔除。

### 7.2 後端

- **CPU (`compose`)。 ** 參考實現，使用人造絲按塊完成。它用於導出、測試、CLI 以及WebGPU不可用時的 Web 回退。
- **GPU (`gpu`)。 **wgpu計算和渲染執行相同計劃的管線。他們使用：
  -`rgba16float`/`rgba32float`蓄電池，
  - 每個混合系列的專用管線和調整（WGSL`override`常數，而不是一個巨大的`switch`），
  - 由`(TileId, generation)`鍵控的駐留快取，
  - 快取的群組和效果結果因生成而失效，
  - 一個用於 GPU 記憶體的全域`Budget`。
- **奇偶校驗測試。 ** 對於每種混合模式、調整和效果，CPU 和 GPU 結果必須在 8 位元輸出的 ≤1/255 容差範圍內匹配。它在軟體適配器（帶有 lavapipe 或 WARP 的wgpu）上的 CI 中執行。

### 7.3 顏色

- 混合預設發生在文件空間中，與Photoshop相容。每個文件的“線性光混合”選項也可用。
- **顯示轉換** (`engine/src/display_color.rs`)：畫布始終進行色彩管理，文件設定檔 → 顯示器設定檔（相對色度 + BPC），按（文件設定檔、模式、顯示器）進行快取。在 GPU 畫布上，變換加上上校樣顏色/色域警告/32 位元預覽，被烘焙到畫布著色器的最終色版應用的 33³ 3D LUT 中； CPU 畫佈在合成上執行 8 位元`photocraft-cms`變換。當文件設定檔與顯示器（sRGB 上的 sRGB）相符時，就沒有 LUT 也沒有轉換。線性複合材料（EXR/HDR、導入時標記為線性 sRGB）以 sRGB 編碼儲存在 8 位元畫布紋理中。 CMYK 文件透過其嵌入的 CMYK 設定檔（`photocraft_color::convert::with_cmyk_space`，由合成器輸入並合成導出）來讀取。
- **顯示器設定檔：** 編輯 › 色彩設定 › 顯示器設定檔：`auto`、內建 RGB 設定檔或`.icc`路徑。在`auto`中，每個視窗都使用其所在顯示器的設定檔 (#569)：桌面應用程式在啟動時讀取每個顯示器的 id、名稱、幀和 ICC 設定文件，當應用程式返回到前面時、當任何視窗位於未知或調整大小的顯示器上時（最多每 30 秒；該視窗返回到前面時、當任何視窗位於未知或調整大小的顯示器上時（最多每 30 秒；該視窗內的編輯設定時被延遲）以及當已開啟時間設定時（最多每 30 秒；該視窗內的編輯設定時被延遲）以及當開啟的色彩設定顯示器時（最多每 30 秒；該視窗內的 › 顯示器開啟時則被延遲設定時；在該視窗內已被延遲設定時；該視窗內的編輯設定時會被延遲； (`ui-egui/src/monitor_status.rs`)，桌面應用程式會讀取每個顯示器的 id、名稱、框架和 ICC 設定檔案。 macOS 通过`osascript`读取 AppKit`NSScreen`（无 FFI）； X11 读取 RandR 输出以及与`XRANDR_name`精确匹配的默认颜色設定文件； Wayland 会话中的本机 Wayland 和 Xwayland 窗口保留在 sRGB 上，因为eframe/winit無法安全地設定應用程式表面的 Wayland 影像描述。 macOS 幫助程式在 10 秒後停止。故意不定期重新讀取（#569 決策：讀取消耗約 0.2 秒的 CPU 時間），因此重新分配設定文件，同時PhotoCraft保持在前面，並且在下次返回到前面或顏色設定時會拾取沒有視窗移動的情況。 X11 透過winit的目前監視器名稱選擇根視窗的 RandR 輸出，透過重疊的 RandR 訊框選擇其他視窗； macOS 遵循`NSWindow.screen`。其他地方`auto`是 sRGB。 GPU畫布為每個文件保留一個紋理，並為每個（文件，顯示）保留一個顯示LUT； CPU 畫布紋理是針對每個（文件、顯示）的。`ColorState::monitor_status_for`說明每個顯示器應用的內容：`auto`、`manual`或`fallback`到 sRGB 的原因（遺失、不可讀、非 RGB 或無法用作目標），以及讀數是否為重新讀取失敗後保留的較早讀取；`edit.colorSettings`（`monitorStatus`、`displays`）、說明 › 系統資訊與色彩設定對話方塊會顯示它，而`auto`中的後備會發布通知。讓 macOS 顏色與畫布相符是#581（架構決策）。
- **HDR/EDR 輸出**（具有擴展範圍色彩空間的`rgba16float`表面）是後期功能。此介面已搭載`f32`像素。

### 7.4 神諭

**PSD 檔案包含Photoshop自己的合併複合材料。 ** 在 PSD 的圖層上執行我們的合成器並與其嵌入的複合材料進行比較，從而提供免費的大型一致性測試套件。`testkit`透過 PSD 語料庫自動執行此操作。語料庫由`cargo xtask corpus --all`在固定提交時獲取到`corpus/`（從未提交）；我們自己的Photoshop創作的預言機位於 https://github.com/storytold/photocraft-corpus（請參閱測試 XZZPH269ZX ）。

---

## 8.`photocraft-psd`：獨立 PSD/PSB 箱

**範圍：** PSD (v1) 和 PSB（v2，64 位元長度）的忠實格式層級讀寫。它不相依性我們的文件模型。

```text
psd/src/
  header.rs         color mode, depth, size
  color_mode_data.rs
  resources/        image resources (1005 resolution, 1036 thumbnail, 1039 ICC, 1060 XMP, 1058 EXIF, slices, guides, …)
  layers/           layer records, channel image data (raw | RLE/PackBits | ZIP | ZIP+prediction), masks, blending ranges
  tagged/           additional layer info blocks, one module per key:
                    luni lsct lyid lfx2/lrFX (effects) TySh (text) SoLd/PlLd/lnk2 (smart objects) vmsk/vsms/vogk (vector)
                    adjustment keys (curv levl hue2 blnc selc mixr grdm phfl brit thrs post nvrt expA vibA clrL blwh)
                    SoCo GdFl PtFl (fills), Patt, Txt2, …
  descriptor.rs     ActionDescriptor / OSType parser+writer (used by many tagged blocks)
  engine_data.rs    text "EngineData" (PostScript-like) parser+writer
  image_data.rs     merged composite
  write.rs          serializer (+ RLE/ZIP encoders)
```

- **無損往返。 ** 未知或不支援的標記區塊將保留為原始位元組並按原樣重新發送。這樣，文件就可以保留我們尚未建模的功能，例如 3D 或影片。
- **延遲解碼。 ** 解析器索引色版資料並僅根據請求對其進行解碼。這使得開啟 2 GB PSB 變得快速且受記憶體限制。
- **寫入時合併的複合由呼叫者提供**作為影像緩衝區。`io`使用`compose`渲染它，因此`psd`永遠不需要合成器。
- **在`io::psd`中進行對應。 ** 它將`psd::File`轉換為`doc::Document`並返回，保真度等級為：
  1.扁平化影像。
  2. 柵格圖層、遮罩、混合/不透明度/填滿、可見性、名稱。
  3. 組（包括直通）、剪切、混合、調整和填滿圖層。
  4.圖層效果（lfx2）。
  5.文字圖層：形狀為`text`；原始描述符被保留用於往返。
  6. 智慧型物件：嵌入和鏈接，雙向活動（`io::smart_map`）。導入讀取
     放置層資料（`SoLd`：檔案 ID、變換四邊形、扭曲）、智慧型濾鏡堆疊
     （`filterFX`：建模過濾器成為其PhotoCraft命令和參數；任何其他過濾器
     保持逐字並被列為不可編輯）和過濾器遮罩（全域`FEid`）。出口
     寫入`PlLd`+`SoLd`，將來源嵌入`lnk2`中（`.pcraft`來源成為
     嵌套文件，由我們自己的 PSD 編寫器編寫）和帶有未過濾的過濾器快取
`FEid`中的像素和掩模；未經編輯的導入智慧型物件保持位元組相同。
  7.向量遮罩和形狀圖層。
- **測試：**
  - 未修改檔案的往返位元組穩定性。
  - 開啟→儲存→開啟平等。
  - 複合預言機差異（§7.4）。
  - 解析器上的`cargo-fuzz`。
  - 來自 MIT/BSD 許可的測試集（ag-psd、psd-tools）的語料庫，由`xtask`獲取，未提交。
- 我們自己的Photoshop創作的 oracle PSD（智慧濾鏡、效果形狀、類型、每種模式和深度的調整）來自 https://github.com/storytold/photocraft-corpus，以相同的方式在固定提交中取得。
- **現有板條箱：**`psd`（唯讀，有限）和新的`ag-psd`Rust 連接埠。我們在第一周評估它們，然後要么相依性一個，分叉它，要么編寫我們自己的。鑑於 PSD 的核心地位，我們有可能在穩定的 API 背後編寫我們自己的 PSD。

---

## 9.本機檔案格式（`photocraft-format`）

- **格式：**`.pcraft`檔案是 zip（儲存模式）或目錄包。
  -`manifest.json`：帶有 serde 的版本化文件樹，包括`format_version`和遷移。
  -`tiles/<blake3>.zst`：內容尋址圖塊，zstd 壓縮。
  -`thumb.png`和`composite/`mips 用於快速檢視和預覽。
- **儲存是增量式的。 ** 磁貼是內容尋址的，因此保存僅寫入新磁貼和清單，並垃圾收集未引用的磁貼。
- **自動儲存和崩潰復原** 將相同寫入器重複使用到復原目錄中，從背景執行緒上的快照讀取。
- **PSD 是交換格式，**`.pcraft`是無損本機格式。

---

## 10. 演演算法（`algo`、`paint`、`text`、`vector`、`ml`）

|面積 |內容（第一遍）| GPU核心？ |
|---|---|---|
|調整|色階、曲線、亮度/對比、曝光、自然飽和度、色調/飽和度、色彩平衡、黑白、照片濾鏡、色版混合器、漸層圖、選擇性顏色、反相/色調分離/臨界值、3D LUT是（LUT + 小內核）|
|濾鏡|高斯/框/運動/徑向/表面/鏡頭模糊、反銳利化/智慧銳利化、高通、雜訊添加/減少/中位數/灰塵、扭曲系列（旋轉、波浪、極座標、球形、置換）、風格化、渲染（雲、鏡頭眩光）、像素化、顏色到 Alpha |許多 |
|選擇|選框/套索幾何→遮罩、魔棒（洪水填滿容差）、顏色範圍、快速選擇（圖形切割/超像素）、羽化、增長/收縮、細化邊緣 |一些|
|修復/修復 |點修復、修復筆刷（泊松）、內容感知填滿（PatchMatch + 多尺度 EM）|稍後|
|經紗 |自由變換（仿射/透視/扭曲網格）、液化、木偶 (ARAP)、鏡頭校正、重採樣（Lanczos/雙三次）|是的 |
|油漆（`paint`）|輕拍產生（硬度、圓度、角度、間距、散佈）、壓力/傾斜/速度動態、平滑（拉繩/懶惰滑鼠）、濕邊緣、噴槍、橡皮擦、克隆/修復來源、每個筆畫的混合模式、`.abr`導入 |
|文字（`text`）|字體資料庫特徵（系統字體本機、網路上捆綁的字體）、parley 佈局、每次執行樣式、段落選項、扭曲；渲染到點陣快取 | — |
|向量（`vector`）|形狀、貝塞爾曲線路徑、鋼筆工具幾何形狀、筆劃（破折號、對齊、大寫）、布林運算 → 透過 vello_cpu 進行覆蓋 | — |
| ML (`ml`) |`InferenceBackend`特徵；任務：`segment_point/box`(SAM2)、`matte_subject`（背景去除）、`depth`、`sky`；具有 SHA-256 驗證與按需下載 |

**每種演演算法的規則：**
- **簽名：**純函數`fn(input: &SurfaceView, params: &P, region: Rect, cancel: &CancelToken) -> Surface`。
- **參數：**`P: Pod + JsonSchema + Serialize + Deserialize`。
- **註冊：** 由引擎而不是演演算法箱本身向命令註冊表註冊，因此`algo`保持與 UI 和引擎無關。
- **磁磚：** 磁磚感知，要求`radius`像素的輸入光環。
- **測試：**`testkit`中的黃金影像測試，以及標準基準測試。

---

## 11. 引擎執行階段模型

```text
 UI thread ──dispatch()──► Engine thread (owns mutable docs, applies ops, publishes snapshots)
     ▲                         │  spawns
     │ events (damage, progress)│
     │                         ▼
     │                  Job pool (rayon): filters, AI, export, autosave — read snapshots, return transactions
     │
 Render (UI thread or dedicated): Viewport::render_gpu(snapshot) into toolkit texture
```

- **低延遲的筆輸入路徑：*平台合併樣本→工具（在引擎執行緒上）→將點陣化為COW圖塊→損壞發布→視口僅重新合成損壞的圖塊。目標：筆畫出現在下一幀內。
- **取消：**每個作業都需要一個`CancelToken`，而UI可以取消任何有進度的作業。
- **服務注入`Session`：**
  -`FileSystem`（本機 fs / web 檔案系統存取 / 記憶體中用於測試）
  -`Clipboard`,`FontSource`,`ModelStore`
  -`Clock`,`Notifier`

---

## 12. 自動化與可擴充性

- **`automation`：** 基於`rmcp`建構的 MCP 伺服器。其工具名稱使用底線（點分線
  諸如`doc.open`之類的名稱屬於 JSON 控制協定（`docs/control-protocol.md`）。它暴露了：
  -`session_list`、`doc_open`、`doc_new`、`doc_place`（從讀取根放置為智慧型物件），
`doc_export_layers`、`doc_export_data_sets`（在寫入根下寫入）、`doc_import_data_sets`（來自讀取根的 CSV）、
`doc_save`、`doc_export`、`doc_inspect`（JSON 格式的層樹）、`doc_render_preview`(PNG)、`doc_select`、`doc_close`。
  -`command_list`、`command_run`和`command_batch`，從登錄機碼產生；`jobs_list`和`jobs_cancel`用於背景作業。
  - 僅橋接器（正在執行的應用程式）：`ui_inspect`、`ui_screenshot`、`ui_pointer`、`ui_menu_invoke`、`ui_set`、`control_call`。完整清單位於`docs/development.md`› MCP。
  - 用於代理 CLI 的 Stdio，以及可選的帶有令牌的環回 TCP，以便它可以附加到正在執行的 GUI。
  -`AuthorizedWorkspace`，具有獨立的讀寫目錄能力。遠端
    路徑是經過驗證的相對名稱；仍然需要環境檔案系統的引擎命令
    訪問在自動化邊界處失敗關閉。
- **操作：** 記錄的`Vec<CommandInvocation>`，可批次重播（檔案 → 自動 → 批次）。
- **指令碼（稍後）：** 在同一登錄中嵌入指令碼語言。選項是 Rhai 或通過 mlua (C) 的 Lua。如果我們想要熟悉Photoshop指令碼，則可以透過 QuickJS 進行 JS。
- **插件：** 沙盒WebAssembly過濾器插件（`photocraft-plugins`，L4）由`wasmi`執行，純 Rust 解釋器，具有燃料、記憶體、堆疊和牆壁時間限制，並且沒有主機導入。它們由`plugin.*`命令驅動，並列在過濾器 › 插件下；ABI 位於 [`plugins.md`](plugins.md) 中。本機Photoshop`.8BF`/CEP/UXP 託管超出範圍（它需要不安全的 FFI 並且無法在 Web 上運作）。面板插件稍後推出。

---

## 13. 測試和質量

|層|什麼 |
|---|---|
|單位|每個板條箱；針對小型手動計算裝置的演演算法|
|金色影像|`testkit::assert_image_eq(actual, "golden/name.png", tol)`；`xtask bless`再生的金色 |
| CPU/GPU 平價 | CI 中的每個 GPU 核心及其 CPU 參考、軟體適配器
| PSD |往返、複合預言機、模糊測驗 |
|引擎|指令級測試：開啟fixture→調度指令→斷言快照/渲染；動作回放 |
|使用者介面|面板無頭egui測試 (egui_kittest)；因為 UI 很薄，所以保持很薄 |
|效能 | CI 中的標準工作台 + 預算檢查（複合 4k² 20 層，刷 DAB 吞吐量，開放 100 MP）|
|便攜性| CI 矩陣：macOS arm64、Windows x64、Linux x64、wasm32 建置檢查 |

**v1 的效能目標：**
- 120 Hz 下一幀內的筆畫到像素。
- 在 100 MP/30 層文件上顯示刷新時進行平移和縮放。
- 在 1.5 秒內開啟 100 MP JPEG。
- 16 GB 機器上的 20k×20k 16 位元文件。

---

## 14. 貨物工作區草圖

```toml
[workspace]
resolver = "3"
members = ["crates/*", "apps/*", "xtask"]

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"        # decision pending, see §16
rust-version = "1.95"

[workspace.dependencies]
# foundation
bytemuck = { version = "1", features = ["derive"] }
kurbo = "*"          # pin at scaffold time
glam = "*"
smallvec = "1"
arc-swap = "1"
rayon = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
blake3 = "1"
zstd = "*"           # native; ruzstd for wasm decode
# color / codecs
# ICC: our own photocraft-cms (pure Rust)
image = { version = "*", default-features = false }
jxl-oxide = "*"
# gpu
wgpu = "*"           # one version shared by gpu, viewport, ui
# text / vector
parley = "*"
skrifa = "*"
vello_cpu = "*"
# ml
ort = { version = "*", optional = true }
# automation
rmcp = "*"

[workspace.lints.rust]
unsafe_code = "deny"   # allowed per-crate with justification (gpu, platform)

[profile.release]
lto = "thin"
codegen-units = 1

[profile.dev.package."*"]
opt-level = 2          # image code is unusable at opt-level 0
```

當我們使用`rust-framework-options.md`中的數位建造支架時，版本會被固定。

---

## 15. 分階段路線圖

每個階段都以所有目標上的可演示構建和綠色 CI 結束。階段 8-10（色彩/印刷專業、攝影師/設計師專業、前緣）和每個功能的階段分配位於 [`target-app-parity.md`](target-app-parity.md) 中。

|相|範圍 |完成時間 |
|---|---|---|
| **0。骨骼**（1-2 週）|工作區，所有板條箱已存根，分層檢查，CI 矩陣，包括。 wasm、測試套件、xtask |`cargo xtask ci`在 3 OS + wasm 上呈綠色 |
| **1.觀眾** |點陣文件、圖塊、編解碼器 (png/jpeg/tiff/webp)、GPU + CPU 後備上的視口平移/縮放/旋轉、`.pcraft`儲存/開啟、CLI`convert`| 100 MP JPEG，以重新開啟儲存率 |
| **2.層** |圖層樹、所有 27 種混合模式、不透明度/填滿、遮罩、組、剪裁；規劃器 + CPU/GPU 合成器 + 奇偶校驗測試；歷史; **PSD 範圍** 保真度 1–3 |複合預言機套件 ≥ 95% 在語料庫的容差讀取** 保真度 1–3 |複合預言機套件 ≥ 95% 在語料庫的容差讀取** 內 |
| **3.繪製並選擇** |平台平板電腦輸入、筆刷引擎+預設、橡皮擦、選取框/套索/魔杖、移動、自由變換、裁剪 |使用筆壓力以 <1 幀延遲進行繪製；精選限制繪畫|
| **4. 調整與過濾** |調整圖層（前 10 個）、前 20 個帶有模式生成對話方塊的過濾器 + 即時預覽；**PSD 寫入**；動作記錄| PSD 開啟 → 編輯 → 儲存在Photoshop/Photopea 中正確開啟 |
| **5.文字、向量、樣式** |文字圖層、形狀/筆/路徑、圖層效果（陰影、筆畫、發光、疊加、斜角）| PSD 文字/形狀/效果保真度 4-5 往返 |
| **6。智慧功能** | ML 後端 + SAM2 選擇主題/物件、背景去除、內容感知填滿、修復、RAW 開發 |在 <3 秒內選擇主題 + 刪除肖像上的背景 |
| **7. 自動化與網路** | MCP伺服器+CLI奇偶校驗、批次；`photocraft-web`示範 |代理透過 MCP 編輯 PSD；網路示範開啟 PSD、繪畫和濾鏡 |

---

## 16. 公開決策

1. **UI 工具包。 ** 主要選項是winit+wgpu上的egui。 Bevy、Slint 和 Qt 是替代方案（請參閱`rust-framework-options.md`）。這個計劃適用於他們中的任何一個。
2. **許可證。 ** MIT/Apache-2.0 最大限度地可重複使用。 GPL-3.0 是 Krita/GIMP 模型，可防止專有分叉。請注意，一些候選 dep 是 LGPL（rawler）或 GPL-or-commercial（Slint），而 MIT/Apache 將限制我們可以使用其中的哪一個。
3. **專案名稱。 **`photocraft`（儲存庫名稱）是假定的。
4. **PSD 箱。 ** 建構我們自己的，或採用或分叉`ag-psd`/`psd`。在第一周高峰後做出決定。
5. **預設工作深度和混合。 ** 8 位元 + 伽瑪空間混合與Photoshop相符。 16 位元/浮點 + 線性是現代預設值。建議預設遵循Photoshop並將另一個設定為每個文件選項。
6. **Photoshop插件相容性範圍。 ** 決定：無本機`.8BF`託管；改為沙盒WebAssembly插件 API（請參閱 [`plugins.md`](plugins.md)）。

## 修訂歷史

|日期 |改變|總結|
|---|---|---|
| 2026-10-13 |次要|工作空間大小和`#[test]`計數重新計算（393k → 539k 行，5,460 → 8,040 次測試）；其他當前狀態數字不變|
| 2026年10月10日 |次要|當前狀態總結以及與計劃的偏差；狀態行；`photoshop-parity.md`的斷開鏈接重新指向`target-app-parity.md`|
| 2026-09-30 |專業|設計方案v1 |
