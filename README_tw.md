<p align="center">
<a href="https://getartcraft.com/">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="docs/brand/artcraft-logo-white.svg">
<img alt="ArtCraft" src="docs/brand/artcraft-logo.svg" width="200">
</picture>
</a>
</p>

<h1 align="center">PhotoCraft</h1>

<p align="center">
<b>影像編輯； Adobe Photoshop的開源、Clean-room（淨室）重新實現，以純 Rust 重建。</b><br>
  圖層、遮罩、調整圖層、圖層樣式、類型、向量、筆刷和真實 PSD 文件，<br>
  在完全用 Rust 編寫的本機應用程式中。開源、離線，以及您的。
</p>

<p align="center">
<img alt="100% Rust" src="https://img.shields.io/badge/100%25-Rust-b7410e?style=flat-square&logo=rust">
<img alt="macOS · Windows · Linux · FreeBSD · Web" src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux%20%C2%B7%20FreeBSD%20%C2%B7%20Web-native-2f7bf5?style=flat-square">
<img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
<img alt="Status: early alpha" src="https://img.shields.io/badge/status-early%20alpha-d69e2e?style=flat-square">
</p>

<p align="center">
<a href="https://discord.gg/artcraft"><img alt="Join the ArtCraft community on Discord" src="https://img.shields.io/badge/Join%20us%20on%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40"></a>
</p>

<p align="center">
getartcraft.com 上的<a href="https://getartcraft.com/apps/photocraft"><b>PhotoCraft</b></a>·
<a href="https://getartcraft.com/">ArtCraft</a>·
<a href="https://getartcraft.com/apps">所有製作應用程式</a>
</p>

<br>

<p align="center">
<img src="docs/images/photocraft-demo.jpg" alt="PhotoCraft 正在編輯 Hokusai's The Great Wave: a caption card with a drop shadow, Title and Credit type layers, Vibrance and Curves adjustment layers, and the Curves editor drawn over the image's histogram" width="100%">
<br>
<sub>A 字幕卡，帶有投影、即時類型以及自然飽和度和曲線調整圖層，曲線編輯器開啟。<br>
<i>神奈川巨浪</i>，葛飾北齋，c。 1831</sub>
</p>

> [!NOTE]
> **ArtCraft 是一個由各行各業的藝術家組成的社群。 ** 數位、生成、音樂、
> 遊戲－如果你創造東西，你就是我們中的一員。 **[來 Discord 打個招呼](https://discord.gg/artcraft).**

<p align="center">
<a href="#features">功能特色</a>·
<a href="#everything-in-the-box">完整功能清單</a>·
<a href="#psd-without-compromise">PSD</a>·
<a href="#built-for-agents">Agent 代理支援</a>·
<a href="#under-the-hood">底層架構</a>·
<a href="#get-started">開始使用</a>·
<a href="#downloads">下載</a>·
<a href="#the-crafting-apps">製作應用程式</a>·
<a href="https://discord.gg/artcraft">Discord</a>
</p>

<br>

<table>
<tr>
<td width="25%" valign="top">
<h3>🎛️ 設計熟悉</h3>
      選單、快捷方式、面板和工具位於您的雙手所期望的位置，從 ⌘J 到 ⇧⌘D。如果您了解Photoshop，那麼您就已經了解PhotoCraft。
</td>
<td width="25%" valign="top">
<h3>⚡ 原生且快速</h3>
wgpu（Metal、Vulkan、DX12、WebGPU）上的 GPU 合成器、寫入時複製（COW）圖塊和多執行緒濾鏡。沒有 Electron，沒有網頁視圖，無需等待。
</td>
<td width="25%" valign="top">
<h3>🗂️ 真實 PSD 檔案</h3>
      開啟、編輯和儲存分層的Photoshop文件。重新儲存會保留 309 個psd-tools測試檔案中 307 個的渲染。
</td>
<td width="25%" valign="top">
<h3>🤖 Agent-ready（智慧代理支援）</h3>
      每個操作都是一個命令，因此您可以從 UI、CLI、JSON 控制通道或 MCP 伺服器驅動相同的引擎。
</td>
</tr>
</table>

<br>

＃＃ 特徵

這裡的每個螢幕截圖都是公共領域藝術上的真實應用程序，透過其控制色版在螢幕外呈現。

<table>
<tr>
<td width="50%" valign="top">
<img src="docs/images/photocraft-adjustments.jpg" alt="Monet's Impression, Sunrise with Levels and Vibrance adjustment layers; the Levels editor and the Histogram panel with mean, standard deviation and median are open on the right" width="100%">
<br>
<sub>等級和振動調整圖層，帶有即時分佈圖面板。<br><i>Impression、Sunrise</i>、Claude Monet、1872</sub>
<h3>無悔編輯</h3>
      調整圖層使每次編輯都保持活躍。堆疊層級、曲線、自然飽和度、色相/飽和度等等，將它們屏蔽到某個區域、重新排序或關閉它們，而您的原始像素永遠不會改變。
<br><br>
<b>16 調整圖層</b>也直接應用於像素，包括每個色版編輯的曲線、具有即時分佈圖的等級、黑白、色版混合器、漸層貼圖、照片濾鏡、選擇性色彩和色彩尋找（.cube、.3dl、.look）。添加陰影/亮部、取代顏色、符合顏色、HDR 色調、去飽和度和均衡。
</td>
<td width="50%" valign="top">
<img src="docs/images/photocraft-layer-styles.jpg" alt="The Layer Style dialog editing an Outer Glow on the EARTHRISE type layer, with Stroke also enabled, over the Apollo 8 Earthrise photograph" width="100%">
<br>
<sub>「圖層樣式」對話方塊中即時類型圖層上的外部發光和筆畫。<br><i>Earthrise</i>，William Anders / NASA，1968</sub>
<h3>出售鏡頭的款式</h3>
      投影、內陰影、外發光和內發光、斜角和浮雕、緞面、筆畫和顏色、漸層和圖案疊加，可在任何圖層上即時顯示，包括文字。圖案來自庫（內建、編輯 › 定義圖案、<code>.pat</code>導入/匯出）和 PSD<code>Patt</code>塊。
<br><br>
在圖層之間複製和貼上樣式，立即隱藏所有效果，並直接從 PSD 開啟樣式，渲染為符合Photoshop。
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="docs/images/photocraft-masks.jpg" alt="Vermeer's Girl with a Pearl Earring: an elliptical selection around the face, and a Hue/Saturation layer with an elliptical mask that turns everything outside the face gray" width="100%">
<br>
<sub>橢圓選取範圍成為色相/飽和度圖層的遮罩，因此只有臉部保留其顏色。<br><i>戴珍珠耳環的女孩</i>，Johannes Vermeer，c。 1665</sub>
<h3>了解您形象的選擇</h3>
      跑馬燈、套索和魔棒以確保精確；當您希望電腦進行描畫時，快速選擇、物件選擇和選擇主題；選擇並遮罩以細化頭髮細邊緣。
<br><br>
羽化、擴展、收縮、平滑、生長、重新選擇。將任何選取範圍轉換為圖層遮罩、向量路徑或形狀。智慧選擇在您的機器上執行，無需雲，無需帳戶。
</td>
<td width="50%" valign="top">
<img src="docs/images/photocraft-type.jpg" alt="Bierstadt's Among the Sierra Nevada with the headline SIERRA NEVADA being edited on the canvas in Georgia, a byline, and a paragraph caption; the Character and Paragraph controls are open" width="100%">
<br>
<sub>A 標題已就位編輯，附有署名和一段正文。<br><i>加州內華達山脈</i>，Albert Bierstadt，1868</sub>
<h3>精美套裝</h3>
      點和段落文本，直接在畫布上編輯，具有完整的字元和段落控制項：字體、粗細、大小、行距、追蹤、對齊和顏色。<a href="docs/color-picker.md">全型檢色器</a>也支援使用像素放大鏡進行螢幕取樣。
<br><br>
文字圖層保持可編輯狀態，採用圖層樣式，並透過 PSD 進行往返。
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="docs/images/photocraft-vector.jpg" alt="A lotus badge built from shape layers (Lotus, Water, Sun, Badge and Dotted Ring) over Monet's Water Lilies, with the lotus path's anchor points selected" width="100%">
<br>
<sub>A 徽章由形狀層組成：漸層填滿的蓮花、星星和虛線環。<br><i>睡蓮</i>，克勞德·莫奈，1906</sub>
<h3>像素完美向量</h3>
      矩形、橢圓、三角形、多邊形、直線和鋼筆、自由形狀鋼筆和曲率鋼筆工具，具有與解析度無關的形狀圖層、漸層填滿以及虛線、對齊筆畫。
<br><br>
組合形狀（合併、減去、相交、排除），將路徑保留在「路徑」面板中，將它們用作向量遮罩，或筆畫並填滿它們。我們的 PSD 語料庫中 116 個形狀圖層中有 116 個與Photoshop的像素相符。
</td>
<td width="50%" valign="top">
<img src="docs/images/photocraft-filters.jpg" alt="The Twirl filter dialog at a 320 degree angle, previewing live inside an elliptical selection on Van Gogh's The Starry Night" width="100%">
<br>
<sub>Twirl 在畫布上即時預覽，僅在選取範圍內。<br><i>星夜</i>，文森梵谷，1889</sub>
<h3>提交前請先查看</h3>
      透過您的選擇，每個濾鏡對話方塊都會在畫布上即時預覽。模糊（高斯、盒子、運動、徑向、表面、智慧、鏡頭、形狀和模糊畫廊：傾斜移位、光圈、場、旋轉、路徑）、銳利化、降噪、扭曲（旋轉、波浪、波紋、位移、剪切、之字形…）、像素化、風格化（油畫、風、擠出…）、渲染（雲、纖維、鏡頭光暈、燈光效果）等等。
<br><br>
在智慧型物件上執行過濾器，它們保持可編輯狀態：隨時更改、隱藏、重新排序或封鎖它們。
<br><br>
大半徑模糊使用跨所有核心的執行總和框傳遞：3.6 MP 上的半徑 180 高斯需要不到一秒鐘。
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="docs/images/photocraft-transform.jpg" alt="Free Transform handles around a rotated copy of Ansel Adams' The Tetons and the Snake River, with the History panel listing Open, Duplicate Layer, Free Transform and other steps" width="100%">
<br>
<sub>在旋轉列印上自由變換，每一步都在歷史記錄中列出。<br><i>提頓和蛇河</i>，安塞爾亞當斯，1942</sub>
<h3>隨心所欲地塑造</h3>
      自由變換，包括縮放、旋轉、傾斜、扭曲和透視；精確的 90° 和 180° 旋轉和翻轉；再次轉變。圖層、類型、形狀、遮罩和選擇都一起變換。
<br><br>
完整歷史記錄、切換最後狀態和歷史筆刷意味著每一步都可以撤銷，甚至一次可以撤銷一筆筆刷。
</td>
<td width="50%" valign="top">
<img src="docs/images/photocraft-export-light.jpg" alt="The Export As dialog in the light theme over Klimt's The Kiss: JPG at quality 90, scaled to 50 percent, with a preview and an estimated size of about 684K" width="100%">
<br>
<sub>匯出 與淺色主題一樣，帶有預覽和檔案大小估計。<br><i>The Kiss</i>，古斯塔夫·克林姆特，1907–1908</sub>
<h3>運送到任何地方</h3>
      匯出 與格式、品質、透明度和比例一樣，加上預覽和即時檔案大小估計。一鍵快速導出為 PNG。
<br><br>
選擇深色 Pro 主題、輕快的 Studio 主題或經典外觀。
</td>
</tr>
</table>

<br>

## 盒子裡的所有東西

<table>
<tr>
<td width="33%" valign="top">
<h4>🧰 67 個工具</h4>
      移動·畫板·矩形和橢圓形選框·單行選框·單列選框·套索·多邊形套索·磁性套索·物件選擇·快速選擇·魔棒·裁剪·透視裁剪·切片·切片選擇·框架·吸管·顏色採樣器·尺標·註釋·計數·刪除·斑點修復畫·筆 筆刷筆選擇·框架·吸管·顏色採樣器·尺標·註釋·計數·刪除圖案圖章 · 歷史筆刷 · 橡皮擦 · 背景橡皮擦 · 魔術橡皮擦 · 漸層 · 油漆桶 · 模糊 · 銳利化 · 塗抹 · 減淡 · 加深 · 海綿 · 筆 · 自由形狀型 · 曲率型筆 · 增加遮位 · 錨點尺寸直接選取 · 矩形 · 橢圓· 三角形 · 多邊形 · 線 · 自訂形狀 · 手 · 旋轉視圖 · 縮放
</td>
<td width="33%" valign="top">
<h4>🖌️真正的刷機引擎</h4>
      形狀動態、散射、紋理、雙筆刷、顏色動態、轉移、筆刷姿勢、濕邊緣、構建和平滑（包括拉線），由筆壓力、傾斜、旋轉和方向驅動。筆刷預設、從選擇中定義筆刷以及確定性、可重播的筆觸。
</td>
<td width="33%" valign="top">
<h4>🗃️ 圖層，正確完成</h4>
      群組、剪切遮罩、像素和向量遮罩、填滿圖層（純色、漸層和圖案）、調整圖層、具有智慧型濾鏡和無損變換和扭曲的即時智慧型物件、具有對齊、分佈和連結的多層選擇、Alpha 色版和快速遮罩、27Photoshop混合模式以及 6 個 Paint.NET變體、不透明度和填滿、鎖定、顏色標籤、圖層類型過濾器、合併、展平、點陣化、透過複製/剪切分層、貼上進入。
</td>
</tr>
<tr>
<td width="33%" valign="top">
<h4>🎨任何顏色，任何深度</h4>
      每個色版 8、16 和 32 位元的 RGB、灰階、CMYK 和 Lab 文件。位元深度和顏色模型是執行階段數據，因此每個工具都可以在每個深度上工作。
<br><br>
純 Rust 中的真實 ICC 色彩管理：嵌入式設定、分配並轉換為設定文件，具有所有四種渲染意圖和黑點補償、GPU 上的軟打樣 (⌘Y) 和色域警告 (⇧⌘Y)。
</td>
<td width="33%" valign="top">
<h4>🗂️ 格式</h4>
      PSD 和 PSB、分層 TIFF（TIFF 中Photoshop的圖層數據，以任一位元組順序讀取和寫入）、<a href="docs/pdn.md">Paint.NET PDN3（讀取，具有可編輯圖層和所有 14種混合模式）</a>、<a href="docs/ora.md">OpenRaster（讀取和寫入，具有圖層、群組和混合模式，適用於 Krita、 MyPaint 和 GIMP)</a>、<a href="docs/kra.md">Krita .kra（讀取，帶有繪畫圖層、群組、混合模式和透明遮罩）</a>、SVG（作為形狀圖層打開，作為向量智慧型物件放置），以及平面 PNG、JPEG、TIFF、WebP（有損且無損）、GIF、BMP、TGA、ICO、QOI、TIFF、WebP（有損且無損）、GIF、BMP、TGA、ICO、QOI、PNM、OpenEXR、RD. RGB，保留標頭）以及本機<code>.pcraft</code>格式。平面格式保留其支援的深度：PNG 保留 8/16 位元樣本，TIFF 保留 8/16 位元和 32 位元浮點樣本，OpenEXR 保留半/浮點樣本； JPEG、WebP 和其他 8 位元格式會減少較高深度的樣本，並發出匯出警告。請參閱每種格式的<a href="crates/codecs/README.md#capability-matrix-default-build">codec 功能矩陣</a>。來自 iPhone 和 Mac 的 HEIC 照片是唯讀的（包含在官方版本中；可選<code>- 具有用於源版本的 heif</code>）。 AVIF 是只寫的，具有可選的<code>avif</code>構建功能； AVIF 解碼和 JPEG XL 不可用。Affinity文件（<code>.af</code>、<code>.afdesign</code>、<code>.afphotoXZZPH29 1ZZX、<code>.afpub</code>）以本機方式打開，只讀，畫板、圖層、向量、文字、圖像和遮罩；效果、調整和其他不支援的部分會在警告中列出，並且Affinity匯出不可用。
</td>
<td width="33%" valign="top">
<h4>🪄 日常必需品</h4>
      自動色調、對比和色彩 · 均衡 · 影像與畫布大小 · 裁切與修剪 · 顯示全部 · 編輯 › 填滿與筆畫 · 複製合併 · 貼上 · 參考線、尺標、網格與捕捉 · 動作記錄與重播 · 指令調色盤 (⌘F)。
</td>
</tr>
</table>

<br>

## PSD 不妥協

PhotoCraft的 PSD 支援是一個獨立的包，根據 Adob​​e 的公共規範編寫，並針對真實世界文件的語料庫進行了測試。

- **忠實的往返：** 開啟並重新儲存文件對於psd-tools測試集中的 309 個檔案中的 307 個檔案和我們的混合ag-psd/psd-tools集中的 170 個檔案和我們的 169 個檔案中的 169 個呈現相同的效果。`cargo xtask corpus --psd-tools`），以及我們尚未建模的任何內容（原始區塊、描述符、額外內容）都會被保留而不是被丟棄。重新儲存的檔案與其來源的位元組不相同：PhotoCraft重寫影像資源、圖層記錄和合成。只有獨立的`photocraft-psd`套件在沒有文件模型的情況下解析和寫入文件，才能逐位元組重現每個可解析的語料庫文件 (`crates/psd/tests/corpus.rs`)。
- **匹配的像素：** 複合預言機將我們的渲染與Photoshop自己的合併圖像進行比較，涵蓋漸層插值（經典、感知和線性）、圖層效果、形狀筆畫、剪切和填滿不透明度。
- **大型文件：** PSB、16 和 32 位元檔案以及 CMYK 和 Lab 文件可本機開啟。

## 專為Agent 代理支援打造

每個選單專案、工具和對話方塊都執行一個包含 500 多個命令的註冊表中的命令。 UI、CLI、JSON 控制通道和 MCP 伺服器都會呼叫相同的命令，因此您可以按一下的任何內容，指令碼或 AI 代理程式也可以執行。

```sh
# Headless: open, edit, save
photocraft-cli run wave.psd \
  --cmd filter.sharpen.smartSharpen     --params '{"amount":80}' \
  --cmd layer.newAdjustmentLayer.curves --params '{"points":[[0,0],[64,48],[192,212],[255,255]]}' \
  --out wave-final.png

# Apply one action list to a folder of images
photocraft-cli batch --actions grade.json --in ./raw --out ./graded

# Every subcommand explains itself
photocraft-cli batch --help

# Let an agent drive it over MCP (headless, or bridged to the running app)
photocraft-cli mcp
```

桌面應用程式還提供經過驗證的僅環回控制色版 (`photocraft--control`)，用於檢查 UI 狀態、使用指標事件驅動工具以及取得螢幕外螢幕截圖。本自述文件中的每個圖像都是以這種方式呈現的。請參閱[`docs/control-protocol.md`](docs/control-protocol.md)。

## 在底層架構

- **引擎優先：** 純資料文件模型和命令引擎，頂部有一個精簡的eguiUI。分層是在建置時在 24 個 crate 中強制執行的。
- **兩個合成器：** CPU 合成器充當參考預言機，wgpu合成器將畫布放在 GPU 上。他們互相考驗。
- **寫入時複製圖塊：** 256² 稀疏圖塊使撤消成本低廉且巨大的畫布變輕，效果圖按層狀態進行快取。
- **在瀏覽器中執行：**整個引擎和 UI 編譯為WebAssembly。
- **無塵室：** 僅根據公共規範和觀察到的行為實施。沒有專有程式碼、著色器或資產。
- **已測試：** 超過 1,700 項測試，包括 PSD 往返、合成產生器、合成器預言機和多深度檢查。

## 開始吧

```sh
git clone https://github.com/storytold/photocraft
cd photocraft
cargo run --release -p photocraft -- image.psd   # the desktop app
cargo test --workspace                           # the test suite
```

UI 和文字工具的日文字體來自 [craft-fonts](https://github.com/storytold/craft-fonts)，這是一個可選的建置輸入（桌面版本建置始終包含它）。如果沒有它，PhotoCraft將使用系統的 CJK 字體：

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p photocraft
```

新貢獻者和 AI 代理：從 [`AGENTS.md`](AGENTS.md) 開始，然後是 [`docs/`](docs/)。

適用於 macOS、Windows、Linux、FreeBSD 和 Web 的安裝程式均附加到每個 [GitHub 版本](https://github.com/storytold/photocraft/releases)。在 Linux 上，您可以選擇 AppImage、`.deb`、`.rpm`、tarball 或 Flatpak 套件。該捆綁包需要來自 [Flathub](https://flathub.org/setup) 的 freedesktop 執行階段，`flatpak`提供與其一起安裝：

```sh
flatpak install --user photocraft-<version>-linux-x86_64.flatpak   # or -linux-aarch64
flatpak run ai.storyteller.photocraft
```

AppImage 無需安裝：第一次執行會在`~/.local/share`中註冊其啟動器圖示和選單專案，因此擴充座會在 Wayland 上顯示PhotoCraft的圖示。設定`PHOTOCRAFT_NO_DESKTOP_INTEGRATION=1`來跳過此操作，然後參考 [`docs/releasing.md`](docs/releasing.md) › Linux 來撤銷它。

在 Wayland 會話中，拖曳到視窗上的檔案尚未開啟：winit0.30（egui下的視窗庫）沒有 Wayland 拖放功能（#386）。使用檔案 › 打開，或在檔案總管中複製映像並使用 Ctrl+V 貼上。若要拖放，請在 XWayland 下啟動PhotoCraft：`WAYLAND_DISPLAY=photocraft`、`WAYLAND_DISPLAY= ./photocraft-<version>-linux-x86_64.AppImage`或`flatpak run --nosocket=wayland --socket=x11 ai.storyteller.photocraft`。在 Wayland 上，PhotoCraft在通知中顯示安裝命令，直到您將其關閉。首選項 › 效能 › Linux 顯示伺服器 = X11 每次啟動時都會執行相同的操作。使用分數顯示比例（125%、150%、175%）時，XWayland 視窗以較低解析度繪製並由合成器放大，因此介面在那裡變得模糊；原生 Wayland 保持敏銳（#1072）。

在 macOS 上，命令列工具為`photocraft-cli-<version>-macos-universal.zip`。此二進位檔案使用與應用程式相同的開發者 ID 進行簽名，並由 Apple 進行公證。裸二進位檔案無法像 DMG 那樣攜帶裝訂的公證票，因此第一次執行它時，macOS 會在線檢查公證。你可以自己確認一下：

```sh
ditto -x -k photocraft-cli-<version>-macos-universal.zip .
spctl --assess --type install -vv photocraft-cli-<version>-macos-universal/photocraft-cli
# ... accepted, source=Notarized Developer ID
```

在 FreeBSD 14 (x86_64) 上，该版本的 tarball 布局类似于`/usr/local`。安裝執行階段庫，然後將其解壓縮：

```sh
pkg install libxkbcommon wayland libX11 libXcursor libXrandr libXi libxcb mesa-libs vulkan-loader gtk3 fontconfig freetype2 alsa-lib
tar -xzf photocraft-<version>-freebsd-x86_64.tar.gz --strip-components 1 -C /usr/local
photocraft
```

維護者：[`docs/releasing.md`](docs/releasing.md) 解釋如何建置、簽署和發布版本。

> [!IMPORTANT]
> **狀態：**PhotoCraft處於早期 alpha 階段，我們希望直接了解它的立場：Photoshop的大部分功能表面以某種形式存在，但 **它還不是日常專業工作的Photoshop替代品**。最大的差距是人工智慧/生成功能、缺少 15 個工具、排版和專業工作流程的深度以及插件相容性。每個Photoshop選單專案都連接到一個指令 ([`docs/parity-checklist.md`](docs/parity-checklist.md))，但這測量的是接線，而不是行為。誠實的、逐個維度的圖片以及我們下一步要去的地方在[路線圖](ROADMAP.md)和[奇偶評估](docs/target-app-parity.md)中。預計會有粗糙的邊緣，請提交問題（包括您的作業系統、文件大小、層數和螢幕截圖）。您也可以告訴我們 [Discord](https://discord.gg/artcraft) 上發生了什麼問題。

## 社群包

這些安裝程序由社區成員維護，而不是由PhotoCraft團隊維護。

在 Gentoo 上，[::snakebyte 覆蓋](https://github.com/switch87/snakebyte-overlay) 將 Linux 版本打包為`media-gfx/photocraft-bin`：

```sh
eselect repository add snakebyte git https://github.com/switch87/snakebyte-overlay.git
emaint sync -r snakebyte
echo 'media-gfx/photocraft-bin ~amd64' >> /etc/portage/package.accept_keywords/photocraft
emerge --ask media-gfx/photocraft-bin
```

## 文件

開發人員、架構、自動化、格式和安全文件在 [PhotoCraft文件手冊](book/) 中維護。

若要使用 Docker 自託管瀏覽器版本，請參閱[Docker 虛擬主機](packaging/web/README.md#docker)。

＃＃ 安全

[安全文件](book/src/security/) 和儲存庫[安全策略](SECURITY.md) 中涵蓋了安全架構、威脅建模、解析器強化、模糊測試和漏洞報告。

## 測試語料庫

PhotoCraft針對真實檔案進行了測試：我們自己的Photoshop創作的 oracle PSD
[photocraft-corpus](https://github.com/storytold/photocraft-corpus) 加上psd-tools、ag-psd和PngSuite套件，已固定並
sha256 驗證。使用`cargo xtask corpus --all`取得它們並執行測試
`cargo xtask test-corpus`（詳細資訊請參閱 [docs/development.md](docs/development.md#test-corpora)）。

## 下載

**PhotoCraft新手？ ** 從 [getartcraft.com 上的PhotoCraft頁面](https://getartcraft.com/apps/photocraft) 下載它。這是最簡單的安裝方法。

**想要特定的版本或格式？ ** 在 GitHub 上，[最新版本](https://github.com/storytold/photocraft/releases/latest) 包含下面列出的每個版本，[所有版本](https://github.com/storytold/photocraft/releases) 包含早期版本及其註釋。檔案名稱中的`<ver>`是版本號，`SHA256SUMS.txt`列出每個檔案的校驗和。

### 視窗

|建置 |安裝人員 |便攜式|
|---|---|---|
| x64（64 位元英特爾/AMD）|`photocraft-<ver>-windows-x64.msi`|`photocraft-<ver>-windows-x64-portable.zip`|
| arm64（Snapdragon 和其他 ARM PC）|`photocraft-<ver>-windows-arm64.msi`|`photocraft-<ver>-windows-arm64-portable.zip`|
| x86（32 位元）|`photocraft-<ver>-windows-x86.msi`|`photocraft-<ver>-windows-x86-portable.zip`|

安裝程式和可執行檔經過程式碼簽署。

### macOS

|建構 |文件|筆記|
|---|---|---|
|應用程序，通用（蘋果晶片 + 英特爾）|`photocraft-<ver>-macos-universal.dmg`|簽名並公證 |
|通用命令列工具 |`photocraft-cli-<ver>-macos-universal.zip`|簽章並公證 |

### Linux

|格式| x86_64 | x86_64 aarch64 (ARM64) | aarch64 (ARM64) | aarch64 (ARM64)筆記|
|---|---|---|---|
|應用影像|`photocraft-<ver>-linux-x86_64.AppImage`|`photocraft-<ver>-linux-aarch64.AppImage`|隨時隨地運作；使用 [AppImageUpdate](https://github.com/AppImageCommunity/AppImageUpdate)（`.zsync`檔案）更新自身 |
|扁平包裝|`photocraft-<ver>-linux-x86_64.flatpak`|`photocraft-<ver>-linux-aarch64.flatpak`|沙盒化；`flatpak install --user <file>`|
| Debian/Ubuntu |`photocraft-<ver>-linux-x86_64.deb`|`photocraft-<ver>-linux-aarch64.deb`| |
| Fedora/RHEL/openSUSE |`photocraft-<ver>-linux-x86_64.rpm`|`photocraft-<ver>-linux-aarch64.rpm`| |
|壓縮包 |`photocraft-<ver>-linux-x86_64.tar.gz`|`photocraft-<ver>-linux-aarch64.tar.gz`|隨處拆包|

RISC-V (riscv64)：僅限`photocraft-<ver>-linux-riscv64.tar.gz`，針對 Ubuntu 24.04 建置（需要 glibc 2.39+）。

### FreeBSD

|建置 |檔案|
|---|---|
| x86_64 | x86_64`photocraft-<ver>-freebsd-x86_64.tar.gz`|

### 網頁 (WebAssembly)

|建構 |文件|筆記|
|---|---|---|
|靜態網站|`photocraft-web-<ver>.zip`|在現代瀏覽器中運作；將其託管在任何靜態伺服器上|

## 製作應用程式

PhotoCraft是 **Crafting Apps** 之一：來自
[ArtCraft](https://getartcraft.com/) 團隊，每個人都用 Rust 從頭開始編寫，並且每個人都能夠
獨立自主。

| |應用程式 |這是做什麼用的 |程式碼|了解更多 |
|:-:|---|---|---|---|
|<img src="https://raw.githubusercontent.com/storytold/photocraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.photocraft.png" alt="" width="32" height="32">| **PhotoCraft** | **圖像編輯：圖層、遮罩、文字和真實 PSD 檔案 · 你在這裡** | [GitHub](https://github.com/storytold/photocraft) | [網站](https://getartcraft.com/apps/photocraft) |
|<img src="https://raw.githubusercontent.com/storytold/vectorcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.vectorcraft.png" alt="" width="32" height="32">| **VectorCraft** |向量圖| [GitHub](https://github.com/storytold/vectorcraft) | [網站](https://getartcraft.com/apps/vectorcraft) |
|<img src="https://raw.githubusercontent.com/storytold/filmcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.filmcraft.png" alt="" width="32" height="32">| **FilmCraft** |影片編輯、色彩與聲音 | [GitHub](https://github.com/storytold/filmcraft) | [網站](https://getartcraft.com/apps/filmcraft) |
|<img src="https://raw.githubusercontent.com/storytold/lightcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.lightcraft.png" alt="" width="32" height="32">| **LightCraft** |照片庫與原始開發| [GitHub](https://github.com/storytold/lightcraft) | [網站](https://getartcraft.com/apps/lightcraft) |
|<img src="https://raw.githubusercontent.com/storytold/pdfcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.pdfcraft.png" alt="" width="32" height="32">| **PdfCraft** |閱讀、組織與保護 PDF | [GitHub](https://github.com/storytold/pdfcraft) | [網站](https://getartcraft.com/apps/pdfcraft) |
|<img src="https://raw.githubusercontent.com/storytold/effectcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.effectcraft.png" alt="" width="32" height="32">| **EffectCraft** |動態圖形與視覺效果| [GitHub](https://github.com/storytold/effectcraft) | [網站](https://getartcraft.com/apps/effectcraft) |
|<img src="https://raw.githubusercontent.com/storytold/designcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.designcraft.png" alt="" width="32" height="32">| **DesignCraft** |頁面佈局和發布| [GitHub](https://github.com/storytold/designcraft) | [網站](https://getartcraft.com/apps/designcraft) |

還有 [**ArtCraft**](https://getartcraft.com/) 本身，我們的 AI 圖像和視訊工作室，適合想要真正控制的藝術家。

Crafting 應用程式具有相同的約定：無塵室和純 Rust，原生於 macOS、Windows 和 Linux，透過WebAssembly在瀏覽器中執行，並且完全可由代理程式驅動。

<br>

<p align="center">
<a href="https://discord.gg/artcraft"><img alt="Join the ArtCraft community on Discord" src="https://img.shields.io/badge/Join%20us%20on%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40"></a>
</p>

<h3 align="center">來跟我們一起做東西</h3>

<p align="center">
我們的 Discord 是各種藝術家閒逛的地方：繪畫、拍攝、繪畫、剪輯電影的人們，
  設定類型，人們仍在弄清楚他們喜歡做什麼。分享你正在做的事情，
  尋求協助，告訴我們出了什麼問題，或告訴我們您希望這些工具能做什麼。
  無論您使用哪種媒介，也無論您從事該行業多久，這裡都歡迎您。
</p>

<p align="center">
<a href="https://discord.gg/artcraft"><b>discord.gg/artcraft</b></a>·
<a href="https://getartcraft.com/">getartcraft.com</a>·
<a href="https://getartcraft.com/apps">製作應用程式</a>·
<a href="https://getartcraft.com/apps/photocraft">PhotoCraft</a>
</p>

---

## 許可證和積分

PhotoCraft具有 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 雙重許可，由您選擇。
版權所有 (c) 2026ArtCraft團隊和PhotoCraft貢獻者。所需通知請見[通知](NOTICE)。

捆綁的字體、圖標、圖像和其他資產保留自己的開放許可證；每一項都被列出
其作者、來源和許可位於 [ATTRIBUTION.md](ATTRIBUTION.md)。

每件展示的藝術品均屬於公共領域（維基共享資源、美國太空總署、美國國家檔案館）；來源列在 [`docs/images/SOURCES.md`](docs/images/SOURCES.md) 中。

ArtCraft名稱、字標和 [`docs/brand/`](docs/brand/) 中的標誌是
ArtCraftTeam 不受此授權的保護。它們只能在未經修改的情況下使用，並且只能作為
此儲存庫和PhotoCraft的一部分，位於 [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt) 下。
分叉和修改版本必須刪除它們。

<sub>Adobe、Photoshop、Illustrator、Premiere Pro、Lightroom、Acrobat、After Effects 和 InDesign 是 Adob​​e Inc. 在美国和/或其他国家/地区的商标或注册商标。PhotoCraft是一个独立的开源项目，不隶属于 Adob​​e Inc.、不受 Adob​​e Inc. 赞助或认可；这些名称仅用于描述与其兼容的工作流程。</sub>

<p align="center">
<a href="https://getartcraft.com/"><img alt="ArtCraft" src="docs/brand/artcraft-mark.svg" width="28"></a><br>
<sub>由<a href="https://getartcraft.com/">ArtCraft</a>團隊和社群製作。</sub>
</p>

## 明星歷史

[![明星歷史圖](https://api.star-history.com/svg?repos=storytold/photocraft&type=Date&legend=top-left)](https://www.star-history.com/?repos=storytold%2Fphotocraft&type=date&legend=top-left)
