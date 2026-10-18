# 板條箱與相依性層

下面的層號描述了允許的相依性方向，而不是執行階段權限。板條箱可能僅相依性較低層。獨立格式包保持獨立於指定的其他工作區包。

|層|板條箱 |責任|
|---|---|---|
| L0 |`geom`、`cms`、`color`、`raster`|幾何、ICC 處理、顏色/混合數學、像素格式和寫時複製圖塊 |
| L0 獨立 |`psd`、`codecs`、`affinity`| PSD/PSB與平面點陣格式解析/編碼；有界Affinity預覽擷取 |
| L1 |`doc`|純資料文件、圖層、遮罩、效果、調整、文字、向量和智慧型物件 |
| L2 |`ops`、`paint`、`algo`、`text`、`vector`|歷史操作、繪畫、成像演演算法、類型、路徑和形狀 |
| L3 |`compose`、`gpu`、`format`| CPU組成、wgpu構成、`.pcraft`持久化|
| L4 |`io`|文件模型與外部格式之間的匯入/匯出 |
| L5 |`engine`|`Session`，指令註冊、驗證、歷史整合與檢查 |
| L6 |`ui-egui`、`automation`|egui外殼、無頭自動化、MCP 與活橋 |
|測試支援|`testkit`|共享測試助手|

工作空間應用程式有：

|應用 |路徑|目的|
|---|---|---|
|PhotoCraft桌上型電腦|`apps/photocraft`|eframe/wgpu應用與環回控制伺服器|
|PhotoCraftCLI |`apps/photocraft-cli`|轉換、檢查、執行、批次、JSON 行服務和 MCP |
|PhotoCraft網頁 |`apps/photocraft-web`|為WebAssembly編譯的UI與引擎|

新的箱子必須在`xtask/src/layers.rs`中註冊。`ui-egui`以下的任何內容都可能相依性egui、eframe、winit或rfd。`psd`、`codecs`和`cms`板條箱具有 [`AGENTS.md`](https://github.com/storytold/photocraft/blob/main/AGENTS.md) 中記錄的附加獨立性要求。
