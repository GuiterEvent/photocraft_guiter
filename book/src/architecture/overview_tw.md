# 架構概述

PhotoCraft將文件引擎與表示和傳輸分開。桌面 UI、CLI、JSON 控制色版和 MCP 伺服器透過命令註冊表實現編輯行為，而不是維護單獨的實作。

```text
desktop UI       CLI       JSON control       MCP
    |             |              |              |
    +-------------+--------------+--------------+
                          |
                   command registry
                          |
                    engine Session
                          |
       document model + operations + compositors
                          |
             PSD / raster / .pcraft I/O
```

強制相依性方向從格式無關的基礎包開始，透過文件和操作層，再到組合、I/O、引擎，最後是 UI/自動化。`cargo xtask layers`檢查此規則。

完整的設計敘述和相依性圖保留在 [`docs/architecture.md`](https://github.com/storytold/photocraft/blob/main/docs/architecture.md) 中。該文件包含歷史設計材料以及實施的架構；當安全性或相容性相依性目前來源時，確認目前來源中的行為。

## 核心屬性

-`photocraft-doc`儲存純資料文件模型。
-`photocraft-engine::Session`擁有開放文件、活動狀態、歷史記錄和指令調度。
-`photocraft-compose`是CPU參考合成器；`photocraft-gpu`加速支援的交互路徑。
-`photocraft-io`文件與 PSD 或平面格式之間的對應。
-`photocraft-format`讀取和寫入本機`.pcraft`套裝。
-`photocraft-ui-egui`是引擎指令的薄外殼。
-`photocraft-automation`公開了無頭引擎和即時 GUI 橋。

請參閱[Crates 和相依性層](crates.md) 以了解完整的工作區地圖。
