# 插件：WebAssembly插件 API (ABI v1)

PhotoCraft不託管本機Photoshop插件（`.8BF`、CEP、UXP）：需要不安全的 FFI 進入
任意機器碼，無法在瀏覽器版本中運作。相反，插件是 **WebAssembly
模組**在沙箱中運作。一個`.wasm`檔案適用於 macOS、Windows、Linux 和 Web。

本頁為插件作者合約及宿主參考
（`crates/plugins`、`crates/engine/src/plugin_cmds.rs`中的`plugin.*`指令）。

## 使用插件


|---|---|---|
|`plugin.install`|`{"path": "/x/y.wasm"}`（本機）或`{"data": "<base64>"}`、`"replace": bool = true`|驗證並安裝模組；回傳其說明。 |
|`plugin.list`|`{}`|`{"plugins": [{id, name, version, kind, author, description, params, paramsSchema, overlap, area, size, source}], "folder": report}`|
|`plugin.run`|`{"id": "…", "params": {…}}`（插件參數也可能是頂級鍵）|在活動像素圖層、目標 Alpha 色版/快速遮罩或智慧型物件上執行濾鏡插件。撤銷一步。 |
|`plugin.remove`|`{"id": "…"}`|卸載。 |
|`plugin.reload`|`{"path": "folder"}`（預設：首選項資料夾）|載入資料夾中的每個`*.wasm`；報告並跳過錯誤模組。 |

在應用程式中，已安裝的過濾器顯示在 **過濾器 › 插件** 下（帶有生成的對話方塊和實時
當它們有參數時預覽），在**安裝插件…**旁邊。

**首選項資料夾。 ** 編輯 › 首選項 › 外掛程式 › *使用其他外掛程式資料夾* plus
*附加插件資料夾*：當兩者都設定時，該資料夾中的每個`*.wasm`（非遞歸）都是
在啟動時以及每當首選項變更時安裝（僅限本機構建置；Web 建置安裝
從位元組）。

插件是按**每個處理程序**安裝的，而不是按文件安裝。從插件記錄的智慧過濾器
(`{"command": "plugin.run", "params": {"id", "params"}}`) 在重新渲染時重新執行該插件
安裝；如果遺失，則跳過過濾器（快取的像素保持原樣），就像其他過濾器一樣
濾波器PhotoCraft未實作。

## 沙箱

主機是 [`wasmi`](https://github.com/wasmi-labs/wasmi)，一個純 RustWebAssembly解譯器
（麻省理工學院或 Apache-2.0）。它也為`wasm32-unknown-unknown`構建，因此插件也可以在網路中執行
構建。 Limits (`photocraft_plugins::Limits`, defaults):

|限制|預設 |關於違規 |
|---|---|---|
|主機進口|無：無 WASI、檔案系統、網路、時鐘或隨機性 |模組在安裝時被拒絕|
|模組尺寸| 32 MiB |被拒絕 |
|線性記憶體|每個實例 512 MiB |`memory.grow`回傳 -1；初始記憶體過大失敗 |
|說明（燃油）|對於`pc_filter`，每次呼叫 50 M + 每個樣本 4,000 個`Limit("instruction budget")`|
|掛鐘|每次運轉 60 秒（本機；每 20 M 指令檢查一次）|`Limit("time budget")`|
|通話深度| 1,024 幀 |陷阱|
|模組形狀| wasmi 的嚴格限制（≤ 10,000 個函數，1 個記憶體，...） |被拒絕 |

每個帶都在**新實例**中執行，因此在呼叫和並行帶之間沒有任何狀態存在
從不共享記憶體。陷阱、耗盡預算、畸形模組、不良清單、超出範圍
指標和非零回傳碼都變成錯誤；該文件保持不變（編輯是
交易）。當故障停止同級帶時，這些帶保留原始錯誤
而不是用通用取消訊息替換它。輸出 NaN 或的樣本
無限保持其原始價值；整數深度
文件和 alpha 被限制為 0-1。

wasmi 是用它的*便攜式*（循環）調度程序構建的：預設的尾部呼叫調度程序相依性 LLVM
兄弟呼叫最佳化，當這種情況沒有發生時，長時間執行的模組會溢出主機
堆疊，它會中止處理程序而不是回傳錯誤。

## ABI v1

插件是核心WebAssembly模組（MVP + 通常的後 MVP 功能：大容量記憶體、符號
擴充、多值、飽和轉換；沒有 SIMD，沒有記憶體64），**沒有導入** 以及這些
出口。除非另有說明，所有整數均為`i32`；指標是模組記憶體中的偏移量。

|出口|簽名|意義|
|---|---|---|
|`memory`|記憶 |此模組的線性記憶體。 |
|`pc_abi_version`|`() -> i32`|必須回傳`1`。 |
|`pc_manifest`|`() -> i64`| UTF-8 清單 JSON 的`(len << 32) \| ptr`(≤ 64 KiB)。 |
|`pc_alloc`|`(size) -> ptr`|主機可以寫入`size`位元組區塊，或是在失敗時寫入`0`。從未被釋放：每個樂團都會獲得一個新的實例。 |
|`pc_filter`|`(buf, buf_len, width, height, channels, format, params, params_len) -> i32`| 就地濾波像素；`0`= 成功，其他皆為錯誤碼。 |

＃＃＃ 顯現

```json
{
  "id": "org.example.tone",
  "name": "Tone…",
  "version": "1.0.0",
  "kind": "filter",
  "author": "Example",
  "description": "What it does.",
  "params": {
    "amount": {"type": "number", "min": 0, "max": 100, "default": 50},
    "steps":  {"type": "int", "min": 1, "max": 16, "default": 4},
    "mono":   {"type": "bool", "default": false},
    "mode":   {"type": "choice", "options": ["soft", "hard"], "default": "soft"}
  },
  "overlap": 0,
  "area": "content"
}
```

-`id`：`A-Z a-z 0-9 . _ -`中的 1–64 個字元（建議使用反向 DNS）。安裝模組
  使用相同的 id 替換舊的。
-`name`：選單標籤（1–64 個可列印字元）。任何指令碼都可以，但控製字元和
  Unicode 格式字元（雙向覆蓋和標記、零寬度空格、軟連字元）被拒絕，
  除零寬度連接符和非連接符外，且至少有一個字元必須可見。
`kind`：`"filter"`（v1中唯一的一種）。
-`params`：依對話順序。名稱為`[A-Za-z0-9_]`，不是以`_`開頭，也不是`id`/`layer`。
  主機在呼叫前驗證使用者的值：缺失值採用預設值，數字為
  箝位到`min..max`（整數四捨五入），錯誤的類型或未知的選擇是錯誤。
-`overlap`(0–256)：每個輸出像素周圍濾鏡所需的上下文像素（模糊半徑）。
  主機將每個帶的每一側填滿`overlap`，重複畫布的邊緣像素（或
  畫布外層內容），並僅保留結果的內部矩形。
-`area`：`"content"`（預設）過濾由`overlap`生長的圖層的非透明邊界；
`"canvas"`過濾整個畫布，包括透明像素（用於生成器）。

### 像素

`buf`儲存`width × height × channels`**小端`f32`樣本**、交錯、行優先、
在每個位元深度標準化為 0-1（32 位元文件可能保存高於 1 的值）。緩衝區是
`width × height`包括重疊邊距。色版是文件自己的顏色模型，沒有
轉換：

`format`=`depth | alpha << 8 | mode << 16`

-`depth`（位元 0–7）：文件的位元深度，8、16 或 32（資訊性：樣本始終為`f32`）。
-`alpha`（位元 8）：當最後一個色版是（直接，未預乘）alpha 時為 1。
-`mode`（位元 16–23）：色彩模式，依照 PSD 檔案格式編號：0 位元圖、1 灰階、
  2 索引（儲存為 RGB）、3 RGB、4 CMYK、7 多色版、8 雙色調（儲存為灰色）、9 Lab。

Alpha 色版和快速遮罩目標是作為一個灰階色版到達。

主機將層轉換為帶（`f32`最多約 4 MiB 的平鋪對齊區塊），並在其中執行它們
在本機構建上並行，以圖層的格式寫回結果，然後與
選擇：`out = original + (filtered − original) × coverage`，與內建濾波器完全相同。一個
插件永遠不會看到選擇，並且不需要任何特殊的東西來支援它。

＃＃＃ 參數

`params`指向 UTF-8 JSON 的`params_len`位元組：經過驗證的參數，加上`_image`
帶的幾何形狀：

```json
{"amount": 50, "mode": "soft",
 "_image": {"x": 0, "y": 256, "width": 1024, "height": 256, "overlap": 0,
            "canvasWidth": 6000, "canvasHeight": 4000,
            "mode": "rgb", "depth": 8, "alpha": true, "channels": 4}}
```

`x, y`是緩衝區左上角樣本的文件位置（包括邊距），因此
位置相關的濾波器（漸層、暈影）在各個頻段之間是無縫的。

## 用 Rust 寫插件

[`examples/plugins/invert-rs`](../examples/plugins/invert-rs)是一個完整的`no_std`插件
（無分配器，建造量低於 1 KiB）。建構它：

```sh
rustup target add wasm32-unknown-unknown
examples/plugins/invert-rs/build.sh      # cargo build --release --target wasm32-unknown-unknown
```

該指令碼還刷新了`crates/plugins/tests/fixtures/invert.wasm`，這是由
那個來源。任何針對核心WebAssembly的語言都可以（C、Zig、AssemblyScript、WAT...），如
只要模組沒有導入：在 Rust 中，這意味著`no_std`或`std`建置永遠不會接觸
瓦西。

＃＃ 表現

解釋器的每個指令的成本大約是本機速度的 10-100 倍，因此插件適合每個像素和
小鄰域濾波器。由於頻段並行執行，範例 Invert 在 6000 × 4000
RGBA 8 位元層（24 MP，發布，重負載下的 14 核心 Apple Silicon）需要 1.1–1.6 秒
`plugin.run`，而（單執行緒）內建圖片為 3.6–5.1 秒 › 調整 › 反相
（`cargo run --release -pphotocraft-engine--example bench_plugin_invert`；單獨插件：
`cargo run --release -pphotocraft-plugins--example bench_invert`）。峰值額外記憶體約為每個工作執行緒三個帶（輸入、
原始的選擇混合和實例的記憶體），即每個執行緒約 12 MiB。

## v1 中沒有

- 面板/UI插件、讀取其他圖層或文件的插件、主機回調（進度、
  取消），外掛程式提供的文件格式。
- 過濾器 › 最後一個過濾器還不會重複插件運作。
