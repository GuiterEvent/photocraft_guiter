#PhotoCraft記分卡

由`cargo xtask scorecard`從`perf/budgets.toml`、`perf/baseline.json`、`crates/io/tests/corpus.rs`、`scorecard/*.toml`和來源樹產生。請勿手動編輯：更改來源並重新產生（CI 執行`cargo xtask scorecard --check`）。檢查清單狀態根據程式碼進行驗證；這裡的數字取代了其他地方的估計。

＃＃ 概括

|面積 |完成 |部分|失蹤|關鍵數字|
|---|---:|---:|---:|---|
| [效能](#performance) | 3 | 22 | 22 24 | 58 個場景：3 個在預算範圍內，22 個超出預算或失敗，9 個僅跟踪，19 個未執行，5 個尚未測量（基線 2026 年 10 月 5 日）|
| [文件相容性](#file-compatibility) | 7 | 14 | 14 6 | io語料庫：oracle≥148/170，往返≥169/170；psd-tools語料庫：預言≥249/309，≥308/309； |
| [工具](#tools) | 33 | 33 11 | 11 0 | |
| [工作區與使用者介面](#workspace-and-ui) | 13 | 6 | 4 |不執行任何操作的設定：168 中的 26 個（目標 0）|
| [型號](#type) | 4 | 3 | 2 | |
| [自動化](#automation) | 5 | 2 | 4 | 631 個選單專案中的 631 個已上線 (100.0%) |
| [可靠性](#reliability) | 5 | 2 | 0 | 29 個箱子中的 29 個有永不崩潰的棉絨；恐慌_狩獵存在|
| [分佈](#distribution) | 5 | 4 | 2 | |
| [未解決的錯誤](#open-bugs) | 29 | 29 2 | 1 | |

＃＃ 表現

預算：`perf/budgets.toml`（目標來自#209、#210、#211）。數字：提交的基線（`perf/baseline.json`，完整執行），僅由`cargo xtask perf --update-baseline`更新。每晚`perf-nightly`工作流程在固定的 macOS 執行器上執行`cargo xtask perf`，並在強制預算破壞或與同一機器類的基線相比 p50 回歸超過 15% 時失敗。 「超出預算」行是程式碼尚未滿足的目標：它們被報告，並且只有它們的回歸失敗。

基準：2026-10-05，提交`f5d2bcb54a`、Apple M4 Pro (48 GB)、GPU Apple M4 Pro、macOS 15.6；執行開始時的平均負載為 224.90 / 226.31 / 196.03（1/5/15 分鐘）。

|身分證 |場景 |目標| p50 毫秒 | p95 女士 |最大毫秒|峰值 RSS |圖形處理器 |狀態 |問題 |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| P1 |使用圖層樣式移動圖層（每次拖曳更新）| ≤ 25 毫秒 p95 | 321 | 321 535 | 535 535 | 535 825 MB | 409 MB |超出預算（尚未達到目標）| [#209](XZZPH26ZX MB |
| P2 |更改圖層不透明度 | ≤ 20 毫秒 p95 | 276 | 276 443 | 443 443 | 443 639 MB | 409 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) MB |
| P3 |更改圖層填滿 | ≤ 20 毫秒 p95 | 279 | 279 467 | 467 467 | 467 506 MB | 409 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P4|更改混合模式 | ≤ 20 毫秒 p95 | 206 | 206 423 | 423 423 | 423 579 MB | 409 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P5|切換可見性 | ≤ 10 毫秒 p95 | 178 | 178 425 | 425 425 | 425 668 MB | 409 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P6|撤銷移動 | ≤ 25 毫秒 p95 | 256 | 256 530 | 530 530 | 530 650 MB | 409 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P7|重做屬性變更 | ≤ 25 毫秒 p95 | 144 | 144 212 | 212 212 | 212 669 MB | 409 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P8|複製 12 MP 圖層（第一）| ≤ 100 毫秒 p95 | 508 | 508 699 | 699 699 | 699 907 MB | 919 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P9|複製 12 MP 圖層（重複）| ≤ 20 毫秒 p95 | 406 | 406 627 | 627 627 | 627 749 MB | 919 MB |超出預算（尚未達到目標）| [#209](https://github.com/storytold/photocraft/issues/209) |
| P10|貼上 12 MP 層（第一）| ≤ 100 毫秒 p95 | 1232 | 1232 3136| 3136| 1.3 GB | 1.3 GB 1.4 GB | 1.4 GB超出預算（尚未達到目標）| [#209](XZXZ49](XZX) |
| P11|貼上 12 MP 圖層（重複）| ≤ 20 毫秒 p95 | 445 | 445 1174 | 1174 1174 | 1174 958 MB | 1.4 GB | 1.4 GB超出預算（尚未達到目標）| [#209](XZXZ) |
| P12|在 150 層文件中使用箭頭鍵輕移 | ≤ 30 毫秒 p95 | – | – | – | – – | **缺失**：驚慌失措：wgpu錯誤：驗證錯誤/引起：/在Queue::write_buffer中/帶有「pc_compose_uniforms」造成：/在Queue::write_buffer/有「pc_compose_uniforms」標籤的緩衝區上無效 |
| P51 | P51 150 層文件中的新圖層 | ≤ 16 毫秒 p95 | – | – | – | – | – |不運作| [#1771](https://github.com/storytold/photocraft/issues/1771) |
| P13 | P13 |在 15000×10000 16 位元文件上刷塗 | ≤ 80 毫秒 p95 | 6.42 | 6.42 146 | 146 146 | 146 839 MB | 2.0GB|超出預算（尚未達到目標）|XZ453 |
| P14 | P14 |在 15000×10000 16 位元文件上移動圖層 | ≤ 80 毫秒 p95 | 2698 | 2698 4318 | 4318 | 1.5 GB | 1.5 GB 2.0GB|超出預算（尚未達到目標） |
| P16 | P16在 20 層的 32 位元文件上刷塗 | ≤ 100 毫秒 p95 | 21.5 | 21.5 161 | 161 161 | 161 793 MB | 1.6 GB | 1.6 GB超出預算（未達到目標）|XZPHZ4X)(XZPHZ) |
| P17 | P17在 20 層的 32 位元文件上移動圖層 | ≤ 100 毫秒 p95 | 386 | 386 681 | 681 681 | 681 1.2 GB | 1.2 GB 1.6 GB | 1.6 GB 預算超出預算 |
| P18 | P18開啟 20 MP 分層 PSD：第一個像素（今天：完全開啟）| ≤ 1000 毫秒 p95 | 1753 | 1753 2794 | 2794 2794 | 2794 2.2 GB | 2.2 GB 1.9 GB | [#210](https://github.com/storytold/photocraft/issues/210) |
| P19 | P19開啟 20 MP 分層 PSD：可編輯 | ≤ 3000 毫秒 p95 | 1753 | 1753 2794 | 2794 2794 | 2794 2.2 GB | 2.2 GB 1.9 GBK |
| P21 | P21 | 10 MP 上的內容感知填滿 | ≤ 1500 毫秒 p95 | 218 | 218 256 | 256 256 | 256 1.8 GB | 1.8 GB 1.8 GB | 1.8 GB通過| [#Z210](XZPHZ60]5) |
| P22 | P22 |點治療中風 10 MP | ≤ 500 毫秒 p95 | 157 | 157 247 | 247 247 | 247 1.8 GB | 1.8 GB 1.9 GB | 1.9 GB 通過 [#210](XZXZ) |
| P24 | P24 |匯入 500 個筆刷（最長的 UI 執行緒步驟）| ≤ 100 毫秒（最大值）| – | – | – | – | – |不執行| [#210](https://github.com/storytold/photocraft/issues/210) |
| P25 | P25取消 24 MP 上的高斯模糊作業（直到工作人員退出）| ≤ 200 毫秒（最大值）| – | – | – | – | – |不執行| [#210](https://github.com/storytold/photocraft/issues/210) |
| P49 | P49取消 24 MP 上的內容感知填入作業（直到工作人員退出）| ≤ 200 毫秒（最大值）| – | – | – | – | – |不執行| [#210](https://github.com/storytold/photocraft/issues/210) |
| P50| 24 MP 高斯模糊作業執行階段的應用程式幀時間 | ≤ 16 毫秒 p95 | – | – | – | – | – |不執行| [#210](https://github.com/storytold/photocraft/issues/210) |
| P26 | P26選擇 › 修改 › 在 24 MP 上平滑（半徑 50）| ≤ 100 毫秒 p95 | 99.9 | 99.9 216 | 216 251 | 251 1.0 GB | 1.0 GB – |
| P27 | P27選擇 › 修改 › 24 MP 上的羽化（半徑 50）| ≤ 100 毫秒 p95 | 122 | 122 357 | 357 360） | 360 1.1 GB | 1.1 GB – |
| P28 | P28內容感知比例 24 MP（至 90% 寬度）| ≤ 3000 毫秒 p95 | 954192 | 954192 | 954192 | 2.1GB| – |超出預算（尚未達到目標）| [#211](XZZPH81ZZZZ) |
| P29 | P29 |在 24 MP 上選擇主題，重複 | ≤ 100 毫秒 p95 | 559 | 559 693 | 693 693 | 693 1.4 GB | 1.4 GB – |超出預算（尚未達到目標）| [#211](XZZ) |
| P30 | P30 |在全解析度 3500×2200 上液化 800 px 筆刷 |每次更新 ≤ 1 毫秒 | – | – | – | – | – |不執行| [#211](https://github.com/storytold/photocraft/issues/211) |
| P31 | P31 Camera Raw 曝光變化（代理）| ≤ 10 毫秒 | – | – | – | – | – |不執行| [#211](https://github.com/storytold/photocraft/issues/211) |
| P32 | P32長行程時刷子釋放 | ≤ 16 ms（滑鼠鬆開時無卡頓）| – | – | – | – | – |不運作| [#211](https://github.com/storytold/photocraft/issues/211) |
| P33 | P33在 A4 300 ppi CMYK 上刷塗（塗刷到螢幕上）| ≤ 6 毫秒 p95 | 34.7 | 34.7 235 | 235 235 | 235 775 MB | 87 MB |超出預算（尚未達到目標）(XZ219)(XZ219)(XZPHZ19) |
| P36 | P36所有 SDR 示波器 (CPU) 的 Camera Raw 曝光變化 | ≤ 10 毫秒 p95 | – | – | – | – | – |不運作| [#211](https://github.com/storytold/photocraft/issues/211) |
| P40 | P40 |在 36 MP 上刷塗 60 px |僅追蹤 | 1.45 | 1.45 3.19 | 3.19 3.19 | 3.19 1.3 GB | 1.3 GB – |追蹤 |
| P41 | P41 36 MP 上的高斯模糊 r 40 |僅追蹤 | 2963 | 2963 3069| 3069| 1.9 GB | 1.9 GB – |追蹤 | |
| P42 | P42曲線對話方塊變更、圖層預覽、36 MP |僅追蹤 | 38.7 | 38.7 158 | 158 158 | 158 1.1 GB | 1.1 GB – |追蹤| |
| P43 | P43儲存 .psd，36 MP，3 層 |僅追蹤 | 3047| 3446 | 3446 3446 | 3446 1.2 GB | 1.2 GB – |追蹤 | |
| P44 | P44 開啟 .psd，36 MP，3 層 |僅追蹤 | 3535 | 3535 4547 | 4547 4547 | 4547 1.6 GB | 1.6 GB – |追蹤 |
| P45 | P45具有圖層效果的全面刷新、熱快取、36 MP (GPU) |僅追蹤 | 105 | 105 160 | 160 160 | 160 714 MB | – |追蹤 | |
| P46 | P46在具有效果的圖層上輕拍，36 MP (GPU) |僅追蹤 | 30.6 | 30.6 83.3 | 83.3 83.3 | 83.3 863 MB | – |追蹤| |
| P47 | P47字體大小拖曳步驟，50 個文字圖層 |僅追蹤 | 23.3 | 23.3 60.5 | 60.5 71.2 | 71.2 600 MB | – |追蹤 | |
| P48 | P48 14000×14000文件全面刷新|僅追蹤 | 8715 | 9897 | 9897 | – | – |追蹤 | |
| P52 | P52鑿子斜角尺寸變化，12 MP 橢圓，尺寸 8 (GPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P53 | P53鑿子斜角尺寸變化，12 MP 橢圓，尺寸 50 (GPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P54 | P54鑿子斜角尺寸變化，12 MP 橢圓，尺寸 250 (GPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P55 | P55鑿子斜角尺寸變化，12 MP 文本段落，尺寸 8 (GPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P56 | P56鑿子斜角尺寸變化，12 MP 文本段落，尺寸 50 (GPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P57 | P57鑿子斜角尺寸變化，12 MP 文本段落，尺寸 250 (GPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P58 | P58鑿子斜角尺寸變化，12 MP 橢圓，尺寸 8 (CPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P59 | P59鑿子斜角尺寸變化，12 MP 文本段落，尺寸 8 (CPU) |僅跟踪 | – | – | – | – | – |不執行| |
| P60 | P60 150 個全畫布圖層之間的圖層不透明度，6 MP（應用程式 GPU 預算）| ≤ 30 毫秒 p95 | – | – | – | – | – |不執行| [#5973](https://github.com/storytold/photocraft/issues/5973) |
| P61 | P61在 150 個全畫布圖層中隱藏一個圖層，6 MP（應用程式 GPU 預算）| ≤ 30 毫秒 p95 | – | – | – | – | – |不執行| [#5973](https://github.com/storytold/photocraft/issues/5973) |

**還無法測量**（程式碼還做不到；沒有記錄數字）：

|身分證 |場景 |目標|為什麼 |問題 |
|---|---|---|---|---|
| P15 | P15 15000×10000 16 位元文件上的縮放/平移幀時間 |每幀 ≤ 16 毫秒 |尚未有無頭畫布框架線束（在 XZZPH1000ZZX 繪製回調中縮放和平移 10101209）| [#209](XZZ) |
| P20 | P20 |開啟 2 GB PSD / 4–5 GB PSB |開啟有進度，可編輯，絕不空白 |開啟在背景運作並有進度 (#210)，但尚未產生大型檔案裝置 (#216) | [#210](https://github.com/storytold/photocraft/issues/210) |
| P23 | P23 | 4K 扭曲/木偶/透視預覽 |每幀 ≤ 16 毫秒，在後台提交 |預覽在 UI 工具內執行；沒有無頭框架線束，沒有後台提交 | [#210](https://github.com/storytold/photocraft/issues/210) |
| P34 | P34刷預覽記憶體，在 8K 視口上小塗抹 | ≤ 2 MiB |實時筆刷預覽緩衝區位於 UI 端；還沒有無頭櫃檯| [#211](https://github.com/storytold/photocraft/issues/211) |
| P35 | P35使用 Alt/Shift 選取框拖曳 24 MP | ≤ 17 毫秒 p95 | Shift/Alt 選取框修改器尚不存在 (#208) 且拖曳預覽位於 UI 端 | [#211](https://github.com/storytold/photocraft/issues/211) |

## 檔案相容性

Corpus 樓層 (`crates/io/tests/corpus.rs`)：強制最低值；語料庫測試（貨物功能`corpus`、`cargo xtask test-corpus`，始終在 CI 中執行）低於它們失敗。

|語料庫 |目錄 |檔 |甲骨文通地板|往返樓層|
|---|---|---:|---:|---:|
| io 語料庫 |`corpus/psd`| 170 | 170 148 | 148 169 | 169
|psd-tools語料庫 |`corpus/psd-tools`| 309 | 309 249 | 249 308 | 308
| Photoshop 神諭 |`corpus/photoshop`| 256 | 256 188 | 188 258 | 258

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
|檔-215-1 |智慧型物件匯出為嵌入式智慧型物件 (SoLd + lnk2)，而不是平面像素 |完成 | [#215](https://github.com/storytold/photocraft/issues/215) | psd_export.rs smart_blocks：PlLd + SoLd，來源位於 lnk2 (.pmarti> PSB)；開啟 8/16 位元導出作為即時智慧型物件。嵌入連結檔 |
|檔-215-2 |智慧型濾鏡可儲存 PSD 並重新開啟（filterFX 描述子讀取與寫入）| 部分 | [#215](https://github.com/storytold/photocraft/issues/215) | smart_map.rs：14 個濾鏡加上經過驗證的 Camera Raw 子集，雙向映射，其他濾鏡逐字使用，其他濾鏡逐字：僅PhotoCraft濾鏡（旋轉、液化…）和未映射的 Camera Raw 設定被排除在 filterFX 之外，並帶有警告 |
|檔-215-3 |濾鏡遮罩可以直接畫，用undo |失蹤| [#215](https://github.com/storytold/photocraft/issues/215) | smart_cmds.rs:382 遮罩只來自選擇 |
|檔-215-4 |調整（色相/飽和度 + 著色、曲線、色階、陰影/亮部）作為智慧型濾鏡 | 套用完成 | [#215](https://github.com/storytold/photocraft/issues/215) |影像 › 智慧型物件上的調整記錄 image.adjustments.XZZPH130ZZust_cmr. adjustment_as_smart_filter；選擇內容成為濾鏡遮罩），透過 adjustment_as_filter 從來源重新渲染，並可使用 layer.smartFilter.setParams 重新編輯； PSD 地圖等級/曲線/陰影 - 雙向突出顯示 (smart_map.rs)。從智慧型過濾器行開啟編輯器是 FILE-215-5 的行工作 |
|檔-215-5 |智慧篩選行：折疊、遮罩縮圖、混合圖示、拖曳重新排序、上下文選單、即時預覽 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) | smart_ui.rs：31行+可見性+編輯器；沒有折疊、拖曳或上下文選單|
|檔案-215-6 |智慧型物件角傾斜與透視（四元或單應性變換）|完成 | [#215](https://github.com/storytold/photocraft/issues/215) | #426：SmartObject.perspective 儲存投影圖（變換保持其仿射近似）；智慧型物件上的扭曲/透視設定它（transform_cm.cm. set_placement），它透過單應性從來源重新渲染，Warp與其組合，PSD放置讀取和寫入nonAffineTransform / Trnf（io/blocks.rs located_projective，smart_map.rs）；測試 actor_survives_edit_contents、warp_on_mmart_tropiced_on_object_low_edit_contents、warp_on_a_tropiced_s_objects，
|檔案-215-7 |分層 TIFF 讀寫（標籤 37724），尊重保存前詢問首選項 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) | psd tiff.rs（ImageSourceData，均為位元組順序）、io tiff_layers.rs、uiiff.rs（ImageSourceData，均為位元組順序）、io tiff_layers.rs、ui tiff_options_ui.色版和快速遮罩（TIFF 編解碼器保存一個透明度樣本）|
|檔案-215-8 |大型和 BigTIFF 導入，並在所有 IFD 上進行帶狀解碼 |完成 | [#215](https://github.com/storytold/photocraft/issues/215) |編解碼器 tiff_ifd.rs（邊界檢查 IFD 鏈 + SubIFD walk、classic 和 BigTIFF、循環安全；tiff_page/decode.diff_pagede帶狀並行條帶/平鋪解碼、平面 1 和 2、BigTIFF 寫入（tiff_bigtiff 或 > 4 GiB）；io 導入_tiff_page；30 MP RGB8 放氣 345 -> 32 毫秒； UI 中還沒有頁面選擇器 |
|檔-215-9 | SVG 開啟並放置為形狀圖層或向量智慧型物件 |完成 | [#215](https://github.com/storytold/photocraft/issues/215) | io svg.rs：路徑、群組、文字輪廓作為形狀圖層（漸層近似；剪輯/遮罩/濾鏡/圖案/影像子樹柵格化）；
|檔案-215-10 |多頁 PDF 儲存、PDF 示範、圖層複合為 PDF、向量/文字輸出、PDF 匯入 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) | io pdf.rs：另存為/儲存副本/CLI 使用 ICC 寫入一個扁平化的 RGB/Grayds/CKi 列印頁面在 PDF 中列印；多頁保存、演示、合成、向量/文字輸出和 PDF 導入 (#2541) 仍然存在 |
|檔-215-11 | RAW：有損 DNG、高亮模式、.xmp sidecar、有線 RAW 首選項、每個攝影機語料庫 |失蹤| [#215](https://github.com/storytold/photocraft/issues/215) |原始感測器.rs：不支援210有損DNG； raw_defaults 未讀取；沒有原始感測器.rs：不支援210有損DNG；
|檔-215-12 | Camera Raw 對話方塊顯示分佈圖；設定保留為 PSD 智慧型濾鏡 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) |互動式 RGB/Lab 示波器。 PSD WB/燈光/存在、曲線、HSL、分級、銳利化/雜訊細節、顆粒和數位暈影控制映射到描述符保留。主動手動光學和未經驗證的小插圖風格使更新的 ACR 18.4 夾具保持不透明； PSD/.pcraft 保存並保留它。 HDR、剩餘設定和Photoshop接受仍處於開放狀態 |
|檔-215-13 |視訊層幀保留在.pcraft中；PSD 視訊圖層映射 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) |格式convert.rs：89 +測試/roundtrip.rs：282持久幀；沒有 PSD 視訊映射 |
|檔案-215-14 |具有 ACO/ASE 匯入和匯出功能的使用者樣本模型 |完成 | [#215](https://github.com/storytold/photocraft/issues/215) |引擎預設/swatches.rs（RGB/HSB/CMYK/Lab/Gray + 逐字顏色書，保留在首選項中）+ swatch_cmK/Lab/Gray + 逐字顏色書，保留在首選項中）+ swatch_cm.drs. (sdv/sdv.2)； ase.rs（組、現貨/全域）讀+寫； ui swatches_ui.rs，顏色選擇器加入到樣本|
|檔案-215-15 |圖層樣式輪廓選擇器、曲線編輯器、預設、.shc 匯入/匯出 |部份 | [#2912](https://github.com/storytold/photocraft/issues/2912) |圖層樣式對話方塊：輪廓縮圖 + 預設選擇器（12 個標準形狀，自己的控制點）以及樣式對話方塊：輪廓縮圖 + 預設選擇器（12 個標準形狀，自己的控制點）以及 Contours Contours Droper/Inner Gdows、St.元素的抗鋸齒功能；輪廓編輯器（添加/拖曳/刪除、角點、輸入/輸出、預設、載入…/儲存….shc 根據 Adobe 的公共規範，psd shc.rs）；角點渲染（組成contour_curve_lut）並映射到PSD Cnty。缺少：使用者輪廓庫（新...，選取器載入/儲存/取代輪廓），抗鋸齒不會更改渲染，.shc 尚未針對Photoshop寫入的檔案進行檢查 |
|檔案-215-16 |進階混合（剔除、混合為群組、遮罩隱藏效果）端對端 |部份 | [#215](https://github.com/storytold/photocraft/issues/215) | doc Advanced.rs + .pcraft + PSD/16 |進階混合體（剔除、混合為群組、遮罩隱藏效果）端對端 |部份 | [#215](https://github.com/storytold/photocraft/issues/215) | doc Advanced.rs + .pcraft + PSD/PU TIFF (knko infx clbl tsly .pcraft + PSD/PU TIFF (knko infx clbl tsly mgmgm )，CPUgmgm 約 .psd-tools淘汰賽神諭現已通過；剔除隔離組（0.45 % AA 邊緣）、高級混合、混合和剪裁仍然不同 |
|檔-215-17 | CMYK/RGB文件預覽專色油墨；點板匯出（TIFF 額外樣本，PDF）|失蹤| [#215](https://github.com/storytold/photocraft/issues/215) |撰寫 multichannel.rs：1 個僅適用於多色版的現場預覽 |
|檔案-215-18 |記得儲存為 Web 每切片設定；匯出為可記住並提供最接近的縮放比例 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) |儲存為 Web 設定透過 prefs.dialogs 中記住的對話方塊設定重新啟動後仍然有效（web_cmds::remember_web_s::openremember_web_ettings.來自會話上次匯出的種子，否則是首選項）。 export_dialog seeds 以相同的方式匯出（seed_remembered：格式、品質、透明度、比例、元資料、TGA 位元）並確認將它們記住為使用的。缺少：匯出為最近鄰縮放 |
|檔案-215-19 |另存為 JPEG 開啟 JPEG 選項對話方塊（品質 0-12，格式選項）|失蹤| [#215](https://github.com/storytold/photocraft/issues/215) | ExportSettings 僅具有 jpeg_quality 1-100 (lib.rs:118) |
|文件-215-20 |帶有手柄和 + 按鈕的畫板工具、旁邊的重複畫板、向量 PDF 頁面 |部分 | [#215](https://github.com/storytold/photocraft/issues/215) | artboard_tool.rs：畫板工具通過手柄繪製、選擇、移動畫板並調整其大小（⇧ 保持比例），其選定的畫板按鈕在一個畫板旁邊添加一個畫板按鈕 (⇧ 保持比例），其選定的畫板按鈕 (⌥ (#3033)；複製畫板將副本放置在旁邊（#1584）；僅點陣 PDF 頁面 |
|檔-215-21 |本機顏色、圖案、圖片和圖層庫，作為連結專案或副本 |失蹤| [#215](https://github.com/storytold/photocraft/issues/215) |僅預設庫（preset_store.rs）|
|檔-216-1 |在 CI 中取得和執行的每個語料庫，透過 pin 快取 |完成 | [#216](https://github.com/storytold/photocraft/issues/216) | ci.yml 在每個 PR 和 main 上都有一個語料庫作業（名稱：語料庫測試）：語料庫固定在 xtask/src/corpus_pins驗證，且遺失的語料庫無法通過測試。快取鍵跟隨引腳，因此它是按該行要求的引腳快取的（#190）|
|檔案-216-2 |大資料夾具：20 MP、150 層、產生 2 GB PSD 和 4-5 GB PSB 壓力測試 |部分 | [#216](https://github.com/storytold/photocraft/issues/216) | 4-5 GB PSB 裝置：psd 範例 big_psb 寫入它們（原始和 Rlim），平面和 RbigB_big 2_psb。打開並檢查它們；選擇加入 io tests/large_psb.rs（2.5 GB 合併圖像）。沒有 2 GB PSD、150 層或 20 MP 夾具，CI 中什麼都沒有 |
|文件-216-3 |通过测试读取 TIFF、RAW 和 EXR 语料库；报告每个摄像机的 RAW 覆盖率部分 | [#216](https://github.com/storytold/photocraft/issues/216) | EXR：corpus/exr（OpenEXR 深度测试图像，已固定）由 CI 语料库作业中的编解码器测试/deep_exr.rs 对照上游 sidecar 的 ID 清单进行读取。 corpus/tiff 和 corpus/raw 仍然是手動複製的，沒有測試讀取它們；沒有每台攝影機的 RAW 報告 |
|檔-216-4 |在第三方 Oracle 編輯器中重新開啟匯出 |失蹤| [#216](https://github.com/storytold/photocraft/issues/216) |往返是我們的出口->我們的進口|
|文件-216-5 |保存路径测试：分层 TIFF、PDF、SVG、ACO/ASE、.shc、智能过滤器、原子保存失败 |部分 | [#216](https://github.com/storytold/photocraft/issues/216) |通过保存和读回来测试存在的每个保存路径：分层 TIFF iotests/tiff_layers.rs（every_compression_keeps_the_layers、advanced_blending_survives_layered_tiff_in_both_byte_orders）、打印 PDF 引擎 print_cmds/tests.rs（print_dry_run_renders_a_pdf_and_reports_the_platform_backend 解码图像）， ACO/ASE 引擎 swatch_cmds/tests.rs（aco_round_trip_keeps_16_bit_values_and_books、ase_round_trip_keeps_groups_floats_and_kinds）、.shc psd shc.rs shc_round_trips_and_rejects_hostile_input 和引擎 layer_style.rs contour_presets_and_custom_curves_apply_in_one_undo_step、智能滤镜引擎测试/smart_psd.rs (smart_filters_survive_psd_in_every_colour_model)、原子保存失败格式/src/atomic/tests.rs (BUG-203-6)。没有 SVG 编写器（仅打开 SVG），也没有PhotoshopPDF 保存 (FILE-215-10)，因此还没有什么可测试的；该行在存在之前保持部分状态 |
|檔-216-6 |透過 MCP 或指令驅動的 30-50 個端對端工作流程驗收任務 |部分 | [#216](https://github.com/storytold/photocraft/issues/216) | Automation/tests/agent_tasks.rs：20 個 MCP 任務 |

＃＃ 工具

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
|工具-213-1 |铅笔工具：别名 1 px 绘画、方形像素对齐光标 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) |工具::铅笔（B 组）：像素网格上的实时锯齿笔划、方形光标、自动擦除；铅笔测试.rs |
|工具-213-2 |工具栏上的混合器画笔工具 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) |画笔 (B) 弹出窗口中的工具::MixerBrush；湿/负载/混合/流动/采样所有层选项、命令/控制调度和选择剪切测试 |
|工具-213-3 |可实时预览已修补区域的修补工具 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) | Paint.patch（retouch_cmds/patch.rs，源/目标）+ Tool::Patch（J组：套索，然后拖动选择）；拖动实时显示修复效果（patch_preview.rs，粗略颜色拟合：52 毫秒内 1000x1000）|
|工具-213-4 |重用修复完成的内容感知移动工具 |部分 | [#213](https://github.com/storytold/photocraft/issues/213) | Paint.contentAwareMove (retouch_cmds/content_aware_move.rs: 移动/扩展、结构 1-7、颜色 0-10、采样所有图层、Transform On Drop 的scaleX/scaleY/rotate；后台作业) + Tool::ContentAwareMove (J 组：套索，然后拖动；Transform On Drop 在放置的补丁上放置一个缩放/旋转框，cam_transform.rs: 输入提交作为一步，Esc取消）。 cam_preview.rs：一旦指標或框靜止，結果就會顯示（較小採樣視窗上的可取消後台作業，僅小選擇，#4385）。缺失：大選擇的漸進式預覽 |
|工具-213-5 |绘制类型/形状/智能/填滿图层可进行栅格化，然后继续描边 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) | rasterize_prompt.rs：确定光栅化然后绘制（两个历史状态），取消不执行任何操作；可通过 ui.dialog.confirm 驱动 |
|工具-213-6 |填入對話方塊：內容、混合、不透明度、保留透明度；記得; Shift+退格鍵|完成 | [#213](https://github.com/storytold/photocraft/issues/213) | edit.fill 上的 fill_ui.rs 對話方塊（fill_cmds.rs）：所有內容、模式、不透明度、保留內容、模式、不透明度和 ShiftF；
|工具-213-7 |每個基於筆刷的修飾工具的即時筆畫預覽 |部分 | [#213](https://github.com/storytold/photocraft/issues/213) |即時（canvas.rsStrokes_live）：筆刷、鉛筆、橡皮擦、模糊、銳利化、塗抹、減淡、加深、海綿筆、複製圖章、修復筆筆、複製圖章、修正版本#10261043）筆劃筆（1026個版本的圖章）； (P32)。尚未上線：斑點修復筆刷、混合筆刷、背景橡皮擦、圖案印章|
|工具-213-8 |複製來源：W/H 連結、繪製疊加（不透明度/混合/剪輯/自動隱藏）、Alt 目標遊標 |部分 | [#213](https://github.com/storytold/photocraft/issues/213) |使用不透明度、反相、圓形剪切、自動隱藏和法線/變暗/變亮/差異混合筆的克隆源（堆疊在圖片. draw_clone_preview、clone_source::overlay_blend）；形狀尖端（採樣位圖、旋轉、圓度）將覆蓋層剪輯到其輪廓（brush_cursor::tip_mask，#4372）；W/H 鏈接（cloneSource.set 鏈接，面板鏈按鈕，#3298）|
|工具-213-9 |採樣時吸管比較環（上面是新顏色，下面是當前顏色）|完成 | [#213](https://github.com/storytold/photocraft/issues/213) | eyedropper_ring (canvas.rs)：在按住按鈕時為吸管和 Alt 採樣繪製的上弧、下前景的採樣顏色；
|工具-213-10 | Ctrl+Alt 拖曳可透過畫布上的讀數調整筆刷大小/硬度大小 | 完成 | [#213](https://github.com/storytold/photocraft/issues/213) | Brush_resize.rs (#297, #370)：在 macOS 上按住 Alt+右鍵拖曳到 Alt+右鍵（Control+Option-Control =1161111111999999(297))。 Control+Alt），水平設定直徑和垂直設定硬度，圓形加上直徑/硬度讀數，一個合併的工具。 setBrush，無油漆或歷史記錄； 9 項測試 |
|工具-213-11 |可編輯的壓力曲線，全域與每個控制項|完成 | [#213](https://github.com/storytold/photocraft/issues/213) |全域：首選項 › 工具 › 壓力曲線（首選項 tools.PressureCurve；在任何筆刷之前應用於 ui-XZZPH203Z 中的每個寫筆樣本；預設的編輯器Pressure_curve_ui.rs）。每個控制項：每個筆刷 Dynamic 都有自己的曲線（繪製 ResponseCurve，相同的單調立方體；Dynamic.curve，由 tools.setBrush 設置，例如 shapeDynamics.size.curve），應用於其 Fade/Pressure/Tilt/Wheel/Rotation 輸入曲線在dynamics.rs24pvalue.
|工具-213-12 |匯入影像檔案作為筆刷筆尖 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) | Brush.presets.importImage（路徑或資料）；筆刷面板和預設選擇器匯入筆刷...採用 .abr 或影像 (#3279) |
|工具-213-13 |筆刷回歸測試：無接縫、硬邊、零散射預設值、ABR 永不黑 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) | Brush_tests.rs 中的接縫、硬邊和零散射測試； abr_map/tests.rs 中的 ABR 顏色測試 |
|工具-213-14 |平板電腦：macOS、Windows 和 Linux 上的畫布上沒有作業系統箭頭遊標 |部分 | [#213](https://github.com/storytold/photocraft/issues/213) |筆刷工具集 CursorIcon::None (canvas.rs:1259-1275);未按作業系統進行驗證 |
|工具-213-18 |圖案圖章工具：S 彈出按鈕中克隆圖章的同級工具 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) | S組中的工具::PatternStamp; Paint.patternStamp 用“對齊”和“印象派”標記圖案面板圖塊（近似值：它抖動每個近似值：ZZ12121Z1212132 XZ121214圖塊（ZPHZ121214圖塊（ZPHZ121213）圖塊的圖塊圖塊（PPHZ121214圖塊（ZPHZ1213）圖塊（近似值：它嗎21Z13）的塗抹印象派外觀）；引擎測試涵蓋深度、接縫、敵意輸入 |
|工具-214-1 |選擇與遮罩工作區：視圖模式、最佳化邊緣筆刷、6 個輸出 |部份 | [#214](https://github.com/storytold/photocraft/issues/214) | select.refineEdge 對話方塊，在Photoshop的視圖模式下進行即時預覽：黑子邊緣、行入微影、木疊、圖面、白色圖面、白色圖面、黑相、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖、圖圖明圖明圖明圖明圖明圖明圖、圖圓、圖、圖、白） (P)、F 循環、X 顯示圖像（select_mask_view.rs，#1754）；最佳化邊緣筆刷（R、大小、擴展檢測區域/恢復原始邊緣、Alt 交換）繪製 select.refineEdge FineStrokes，透過顏色線摳圖估計（matting/brush.rs，#2972）；輸出選擇、圖層遮罩、新圖層、帶遮罩的新圖層、新文件、帶遮罩的新文件 (#2998)；沒有工作區佈局 |
|工具-214-2 |選框：平移方塊、從中心 Alt、實時寬 x 高讀數 |完成 | [#214](https://github.com/storytold/photocraft/issues/214) | canvas.rs marquee_corners：移動正方形或圓形，從中心 Alt，光標讀數旁邊的實時 W x H test（marquee_readout）；
| 214-3 |顏色範圍測試：模糊度 0 時的採樣顏色僅選擇完全匹配 |完成 | [#214](https://github.com/storytold/photocraft/issues/214) | Selection_cmds.rs 中的 color_range_fuzziness_zero_selects_exact_match_only 斷言選擇了精確，並且在模糊度為匹配 _colorly 斷言 10 級像素時拒絕？中涵蓋的色調預設
|工具-214-4 |移動工具：框選圖層、自動選擇懸停輪廓、雙擊適合、拖曳到選項卡 |部分 | [#214](https://github.com/storytold/photocraft/issues/214) |自動選擇懸停輪廓（首選項 › 工具 ›在滑鼠懸停時顯示突出顯示，#2834）並從空白位置框選圖層（move_box.rs，layer.selectInRect，#2844）完成；雙擊 fit 並拖曳到選項卡 open |
|工具-214-5 |自由變換期間右鍵選單（縮放、旋轉、傾斜、扭曲、翻轉、旋轉 90/180）|完成 | [#214](https://github.com/storytold/photocraft/issues/214) | canvas_tool_menu.rs TRANSFORM_MENU：模式，內容感知比例/木偶扭曲並翻轉打開的框（transform_tool::intercept，#2832）|
|工具-214-6 |人偶扭曲網格遵循物件輪廓（約束三角測量）|完成 | [#214](https://github.com/storytold/photocraft/issues/214) | algo/puppet.rs build_mesh：透過擴展生長的不透明區域的追蹤輪廓，用密度間隔晶格進行三角剖分（符合 Delaunay）；不同的形狀得到不同網格的網格 (#3105) |
|Sm-214-7 |透過帶有即時代理預覽的自由變換框進行內容感知縮放 | 完成一個僅縮放的自由變換框 (cas_transform.rs)，選項列包含「數量」、「保護」和「保護色」的副本以進行即時預覽，然後按 Enter 鍵以框的大小和 x/y 為一步執行它 (#3096) |
|工具-214-8 |測量的尺標角度預填滿影像旋轉 > 任意（0.01 度）|完成 | [#214](https://github.com/storytold/photocraft/issues/214) | Analysis_ui.rs：image.rotation.任意打開尺標角度和拉直方向的種子；random_rotation_starts_at_the_ruler_angle 105-105 度的檢查cw/ccw 的三條測量線 |
|工具-214-9 |裁切：寬 x 高 x 解析度（帶單位）、儲存的裁切預設、在選項欄中拉直 |完成 | [#214](https://github.com/storytold/photocraft/issues/214) | 解析度targetWidth/targetHeight/resolution，#2443）；新建/刪除裁剪預設保存和刪除裁剪工具預設（crop_presets.rs、tool.presets.*，帶有工具裁剪，保留首選項）|
|工具-214-10 |筆：Alt 斷開手把、Shift 45 度、Ctrl 直接選擇、Backspace、關閉拖曳手把 |完成 | [#214](https://github.com/storytold/photocraft/issues/214) | Ctrl/Cmd = 直接選擇和 Alt = 轉換現有路徑上的點（direct_select.rs.P. (#1466)；拖曳過程中按住 Alt 鍵會破壞手柄對，按住 Alt 鍵拖曳會單獨轉動最後一個錨點的傳出手柄，拖曳第一個錨點會關閉該錨點的手柄（vector_ui.rs，#3306）；按住 Shift 鍵單擊將下一個錨點會關閉該錨點的手柄（vector_ui.rs，#3306）；按住 Shift 鍵單擊將下一個錨點放置在前面的距離前一個錨點前一個 45 度。度步長，即時（vector_ui.rs pen_shift_click_puts_the_next_anchor_on_a_45_ Degree_ray、pen_shift_drag_turns_handles_to_45_ Degrees_live，#3751）|
|工具-214-11 |筆畫路徑對話方塊：選擇任何基於筆刷的工具，模擬壓力|完成 | [#214](https://github.com/storytold/photocraft/issues/214) |路徑.筆畫工具：筆刷、鉛筆、橡皮擦和所有其他基於筆刷的工具（背景橡皮擦、克隆/圖案圖章、修復/污點修復筆刷、歷史筆刷、模糊、銳利化、塗抹、減淡、加深、海綿、顏色替換、混合筆刷：沿著路徑的繪畫。*鍵點選「筆畫」按鈕 (#2978) |
|工具-214-12 |形狀：畫布上的圓角半徑手柄、形狀羽化、選擇具有公差的形狀 |部分 | [#214](https://github.com/storytold/photocraft/issues/214) |使用路徑選擇、矩形工具和移動 + 顯示變換變換在每個選定的實時矩形上使用畫布上的圓角半徑：小部件：每個小角R：讀出，一個歷史步驟（corner_radius.rs，2026-10-13）；select.convertToShape 具有公差 (vector_cmds.rs:1256)。缺失：形狀羽毛|
|工具-214-13 |遮罩屬性：密度與羽化控制 |部分 | [#214](https://github.com/storytold/photocraft/issues/214) |引擎中的向量遮罩密度/羽化 (vector_cmds.rs:948)；沒有 UI 控制項 |
|工具-214-14 |消失點視窗：繪製和編輯平面、貼上、克隆透視 |部分 | [#214](https://github.com/storytold/photocraft/issues/214) |過濾器.vanishingPoint (vp_cmds.rs:15);畫布上平面編輯器（vp_ui.rs：創建/編輯平面、邊緣節點、有效/可疑/無效網格顏色、網格大小、粘貼到平面上的剪貼板就OK）；窗戶上的克隆印章、大帳篷和可撕下的飛機仍然下落不明
|工具-214-15 |內容感知填滿工作區：繪製取樣區域、預覽、旋轉/縮放/鏡像、新圖層 |部分 | [#214](https://github.com/storytold/photocraft/issues/214) |引擎取樣自動、矩形、自訂 (edit_menu_cmds.rs:324)；通用對話方塊 |
|工具-214-16 |液化：重做 (Cmd+Shift+Z) |完成 | [#214](https://github.com/storytold/photocraft/issues/214) | liquify_ui.rs 透過 Cmd+Shift+Z 和控制色版支援有界重做；新筆畫清除重做|
|工具-213-17 |紅眼工具：點擊瞳孔以中和紅色反射 |完成 | [#213](https://github.com/storytold/photocraft/issues/213) | Paint.redEye (redeye_cmds.rs) + J 彈出視窗中的 Tool::RedEye；瞳孔大小和變暗量；保留聚光燈；錯過就是一個錯誤|
| 214-17 |旋轉視圖工具：拖曳旋轉視圖、重設視圖、匹配旋轉複製角度 |完成 | [#214](https://github.com/storytold/photocraft/issues/214) |手彈出按鈕中的 Tool::RotateView; view.rotateView / view.resetView;匹配旋轉複製views[].rotationview；
|工具-1649-1 |吸管樣本大小（點，3×3 … 101×101 平均值），樣本層，顯示採樣環 |完成 | [#1649](https://github.com/storytold/photocraft/issues/1649) | document.sampleColor (sample_cmds.rs)：文件版模型和深度中裁切到畫布的方格； Alt-click、對話方塊吸管和資訊面板共用；ui.set eyedropperSampleSize/eyedropperSample/eyedropperRing。顏色採樣器工具是獨立的（#1046，sampler_cmds.rs）|
|工具 -214-20 |裁切疊加選項：三分法、網格、對角線、三角形、黃金比例、黃金螺旋；自動/總是/從不顯示疊加；O 循環疊加，Shift+O 其方向 |完成 | [#1919](https://github.com/storytold/photocraft/issues/1919) | crop_overlay.rs：直立和旋轉框架共享的單位方形參考線（canvas.rs），選項列選單，shortcuts.rs中工具鍵前面的O / Shift+O；工具選項crop_overlay/crop_overlay_show/crop_overlay_oricroentation，ui.setcropOlay/crop_overlay_show/crop_overlay_oricroentation，ui.setpOverlay/pOlayS.
|工具-214-18 |裁剪：拖曳到框外以旋轉它（移動 15 度步長，角度讀數）；一步輸入輪換和作物 |完成 | [#1792](https://github.com/storytold/photocraft/issues/1792) | image.crop角度參數（imagecro_cmds.rs：圖像旋轉>任意裁剪相同，pui）；沿其軸轉動、移動框架並調整其大小（經典模式；預設模式改為轉動影像，TOOL-214-23）； UiState.crop_angle |
|工具-214-19 |裁切：箭頭鍵輕移裁切框（Shift 10 px），X 交換縱向/橫向，繪圖或調整大小時讀取寬 x 高 |完成 | [#1919](https://github.com/storytold/photocraft/issues/1919) | crop_ui.rskeys/nudge/swap_orientation（在shortcuts.rs中選擇nudge和tools.swapColors之前；X也交換選項列比例）和大小調整（canvas.rs使用選取框讀取它）；包括翻轉框架|
|工具-214-21 |裁切：選項列中的拉直按鈕並按住 Cmd/Ctrl 拖動畫一條線；框架轉動，使線最終水平（或接近垂直時垂直），並帶有角度讀數 |完成 | [#1919](https://github.com/storytold/photocraft/issues/1919) | Crop_straighten.rs：一次性選項欄位標直/Bmderpoint 標誌拉直尺的標誌」； Straighten_angle 約定；框架適合其在影像內的最大比例（經典模式）；由 Enter | 提交的待處理編輯
工具-214-22 |裁剪齒輪選單：顯示裁剪區域 (H)、啟用帶有匹配畫布的裁剪屏蔽/自定義顏色、不透明度和自動調整不透明度 |完成 | [#1919](https://github.com/storytold/photocraft/issues/1919) | Crop_shield.rs：fill() 解析盾牌（canvas.rs 將其周圍的直立式框框H，自訂...開啟顏色選擇器； ToolOptionscrop_shield，ui.setcropShield。使用經典模式/自動中心預覽：TOOL-214-23 |
|工具-214-23 |裁切：Photoshop的預設模式（圖像旋轉並移動到直立框後面，自動居中預覽將調整大小的框的中心保持在適當位置並將繪製的框居中）並使用齒輪菜單中的經典模式 (P) |完成 |4p [#1919](XZZPH267ZZp. Crop_angle）和image.crop；預設模式將視圖的旋轉設為 -crop_angle，同時工具顯示框架（選擇該工具，在任一模式下，都會使旋轉視圖圍繞視圖中心直立，在 Enter/Esc 或按 P 後不會恢復；經典模式使相機保持直立，繪製的框架已旋轉；如 XZZPH265ZXX 25.1），旋轉以保持框框保持原狀，將視圖和框架一起移動以便向內拖動，按箭頭鍵，將調整大小的框的中心保持在實時放置（自動居中預覽；繪製的框在釋放時居中，移動的框從不居中），讀數 = 圖像的輪次；齒輪菜單順序和經典模式變灰為XZZPH261Z/ZOview.Ft/ZOview. (ui.set CropShield)，P 位於筆鍵前面 |
|工具-1053-1 |形狀筆畫控制：實線/虛線/點線、自訂虛線/間隙/偏移、大寫、連接、對齊、斜接限制和不透明度 |完成 | [#1053](https://github.com/storytold/photocraft/issues/1053) | shape_lines_ui.rs：共享暫存編輯器、渲染預覽器範例； shapeStroke； pcraft 和 vstk 風格的往返。漸層填滿 UI 和儲存的筆畫庫保持開啟狀態；線條保留實心條幾何形狀 |
|工具-213-19 |刪除工具：在一個物件上繪畫（或圈出它），它會從周圍環境中填滿 |部分 | [#213](https://github.com/storytold/photocraft/issues/213) | Paint.remove（retouch_cmds/remove.rs，後台作業）+ J 彈出視窗中的Tool::Remove；循環閉合它們所包圍的內容；透過具有紋理特徵的非局部補丁完成、洋蔥皮開始、最佳補丁複製和梯度域接縫隱藏進行填滿（algo/nonlocal.rs，Newson 等人 IPOL 2017）；瓷磚上的稀疏孔（algo/remove.rs）。對公共領域照片中的 20 個孔進行刪除品質：填滿保留其替換的紋理的 0.89（內容感知填滿完成後的紋理為 0.54），邊緣梯度為原始紋理的 1.03； remove_bench 24 MP：100 px 環 120 毫秒，1000 px 環 24 MP：100 px 環 120 毫秒，1000 px 環 20.5x 1.5 秒。選項列：對所有圖層取樣，建立一個新圖層（上方新圖層上的填滿，`newLayer`），每次筆觸關閉後刪除（筆觸收集為色調，↩將它們應用為一個`strokes`作業，Esc 刪除它們）。缺失：生成式AI模式，結果即時預覽|
|工具-2259-1 |彼此跟隨的一件事（圖層、選擇輪廓、自由變換框）的箭頭鍵輕移是一種歷史狀態，最好將其關閉 |完成 | [#2259](https://github.com/storytold/photocraft/issues/2259) | nudge_bundle.rs：一次執行共享一個`coalesce`暫停工具暫停工具（而每次執行共享一個`coalesce`75ZZX”（而“輕推工具”秒）內進行，並且沒有其他內容觸及文件（編輯、撤消/重做、圖層選擇、保存結束文件）； move_mods.rs 通過它執行 layer.translate 和 select.transformSelection，transform_tool.rs 將框微調折疊到一個會話步驟中。工具 > 捆綁箭頭鍵微移使每次按下都有自己的狀態。選定的像素會浮動直至掉落，這已經是一種狀態； ⌥-輕推（每個副本）從不參加跑步 |


## 工作區與使用者介面

**不執行任何操作的設定：168 項中的 26 項**（目標 0，#204）。由首選項審核 (`xtask/src/scorecard.rs`) 計數：當首選項模型外部的程式碼（`prefs.rs`，其`impl Session`區塊除外；排除測試；架構驅動的對話方塊從不命名欄位）存取`section.field`或`.section`時，XZPHZZX284ZZX 時，XZPHZX284ZZX 時，XZ字段計為讀取，且`.field`在同一檔案中，或呼叫讀取`self.field`的節方法。這是一個啟發式下限：通用名稱看起來可以讀取，但實際上並非如此。

未讀：`general.color_picker`、`general.animated_zoom`、`general.zoom_resizes_windows`、`workspace.auto_show_hidden_panels`、`workspace.enable_floating_document_window_docking`、`file_handling.save_in_background`、`performance.cache_levels`、XZZPH294ZXZ、Z294ZXZ`units_and_rulers.gutter`、`plug_ins.show_extension_panels`、`plug_ins.allow_scripts_to_connect`、`plug_ins.generator_enabled`、`type_.text_engine`、`type_.recent_fonts`、`enhanced_controls.scrubby_slider_acceleration`、`enhanced_controls.touch_gestures`、ZZZZZ、ZZ、ZZ、ZZZ、ZZZ、ZZZ、ZZZ、ZZZ34XZZXZ`raw_defaults.resolution`、`raw_defaults.sharpen_for`、`raw_defaults.open_as_smart_object`、`raw_defaults.apply_auto_tone`、`integrations.allow_agent_control`、`integrations.control_port`。

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
UI-217-1 |工具選項、上次匯出為/儲存為網頁/填滿設定和最近使用的字體仍然存在 |部分 | [#217](https://github.com/storytold/photocraft/issues/217) |418) 將其設定在首選項中保留在首選項中沒有首選項 (exports)；字段；導出對話方塊.rs：21 png/85。每個繪畫工具都有自己的筆刷（paint_mouse.rssync_tool_brush，#218），但它們位於 UiState 中，因此它們會像其他工具選項一樣在退出時丟失 |
UI-217-2 |重新啟動時記住視窗大小與停靠寬度 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) |eframe持久化（ui.ron）：persist_window為窗口，egui自然化為dock面板寬度；winstarter_ivedive(winconn_panel_surshed) | (apps/photocraft/src/main.rs; b07a2e5b, #587) |
UI-217-3 |檔案 > 開啟從上次使用的資料夾開始 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) |file_dialog.rs last_used_dir：開啟對話方塊詢問initial_dir（最新檔案的資料夾），apps services.rs 將其傳遞到 XZZZZZ 將其傳遞到 XZZZPHtory_ZXZ； open_starts_in_the_last_used_folder + 最後_used_dir_takes_the_most_recent_parent |
| UI-217-4 |偏好設定全部生效；最近的文件保留並遵守最近的文件計數 (#204) |失蹤| [#217](https://github.com/storytold/photocraft/issues/217) |請參閱首選項審核和 BUG-204-* |
| UI-217-5 |面板組可以浮動（選單+拖出擴充座）並在啟動時恢復 |失蹤| [#217](https://github.com/storytold/photocraft/issues/217) | dock.rs/panels.rs 中沒有浮動群組 |
| UI-217-6 |文件標籤：溢位清單、上下文選單（關閉/其他/全部/顯示）、拖曳以重新排序 | 完成 | [#217](https://github.com/storytold/photocraft/issues/217) | canvas.rs：關閉/其他/全部選單，有未儲存的提示； Reveal重複使用.revealInFinder(engine layer_pmen)。 document.move | 拖曳選項卡重新排序
| UI-217-7 |指令面板顯示最近在空查詢上執行的指令 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) | Palette.rs：menus::invoke 中的 note_recent 保留最後 8 個指令（dedup，最新的在前）；一個空查詢透過recent_hits列出它們； the_palette_records_and_replays_recent_commands_on_an_empty_query 涵蓋記錄 -> 空查詢 -> 重播 |
UI-217-8 |工具專用畫布右鍵選單；右鍵點選文字欄位中的拼字建議 |部分 | [#217](https://github.com/storytold/photocraft/issues/217) | Paint_mouse.rs 筆刷選擇器和 canvas_tool_menu.rs 選擇操作；text_field_menu.rs 以取消/已刪除文字欄位提供右鍵/貼上////// (#2921)；其他工具缺失；文字工具的編輯選單以拼字建議開始，並在拼字錯誤的單字上新增至字典 (#4370)；忽略/忽略全部，非編輯類型選單和通用文字欄位中的建議尚不存在 |
UI UI-217-9 | UI比例透過zoom_factor預設100/115/125/150/200%，記住|完成 | [#217](https://github.com/storytold/photocraft/issues/217) | prefs::UiScale 提供 Auto plus 75% 到 300%（13 個預設值的預設值）；解決它（自動遵循系統比例，包括小數和未縮放的 4K，#1072/#1278），sync_display_scale 將其與 ctx.set_zoom_factor 一起應用。這是一種偏好，所以會被記住。 input_tests.rs 中的 Zoom_factor 斷言是關於畫布縮放而不是縮放介面，而不是關於此設定 |
UI UI-217-10 |外觀：動畫切換、每個主題的畫布圖案切換、開始畫面最近使用 |部分 | [#217](https://github.com/storytold/photocraft/issues/217) | interface.enableAnimations (prefs_ui.rssync_animations, #3273) 將兩種滾動時間的 XZZ 動畫時間和主程式所列出的主詞（ZvasZXyvasXsvasXZ； home_recent，#2691）；畫布點圖案切換是interface.showCanvasDots（#2264，全局而不是每個主題）；interface.showHomeRecentFiles 隱藏主畫面最近使用的內容 (#4382) |
| UI-217-11 |柔和的黑暗和柔和的光主題，帶有選擇器預覽；淺色主題中的輪廓旋鈕|部分 | [#217](https://github.com/storytold/photocraft/issues/217) | theme.rs ThemeKind 有 14 個主題：12 個早期主題加上 Soft Dark 和 Soft Light（低對比度為 Studio 佈局為調色板顏色4.5:1，可在首選項中使用現有主題卡預覽進行選擇，ui.set {主題}，視窗 > 主題）；淺色主題中的輪廓旋鈕仍然缺少|
UI-217-12 |儲存自訂新文件預設；真正的最近類別|完成 | [#217](https://github.com/storytold/photocraft/issues/217) | document_preset_cmds.rs 和 new_doc_ui.rs：透過首選項儲存/選擇/刪除命名預設；最近列錶框中建立的最後 20 |個文件（document.recent.*，保留預設，最新的在前，重複資料刪除）和未覆蓋的預設值（#3131）；已建立的文件填滿最近類別 |
UI-217-13 |數字欄位接受算術和單位；粗糙的標籤；雙擊重置滑桿 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) | widgets::value_field：以文件解析度轉換的算術加單位（1in、5mm、72pt、50%）（#309；使選項列、滑桿行和字元標籤變得模糊（Shift x10、Alt 微調）；雙擊時具有已知預設重設的滑桿行 |
| UI-217-14 |圖層面板：滾輪混合模式、新建/垃圾重複/刪除、藝術背後的檢查器 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) | panel.rs：滾輪步進混合模式下拉選單（widgets::dropdown_wheel_hovered，#1747）；刪除新圖層/群組/刪除重複/群組/刪除它們的行（footer_drop，#736），刪除效果行刪除刪除該效果或整個像素樣式（fx_trash_drop，#3102）； Shift+加/減步進圖層或繪畫工具的混合模式（blend_keys.rs，#3102）|
UI-217-15 |曲線：平滑/鉛筆模式、黑色/灰色/白色吸管、工具提示 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) |曲線編輯器（adjust_editors.rs、對話方塊和屬性）：編輯點/繪製（鉛筆）按鈕和平滑，全部帶有工具提示； Mpng；#3088）。在「映像」>「調整」>「曲線」對話方塊（調整_dialog.rs，#873）和曲線調整圖層的「屬性」面板（layer_picker.rs：按一下映像為圖層。使用吸管參數設定調整，在圖層下方取樣；Esc 或其他工具解除武裝；#3110）| 設定黑色/中性灰色/白色點吸管。
| UI-217-16 |色相/飽和度：飽和度和亮度滑桿獲得色相跟隨顏色軌道 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) | Saturation_track/lightness_track (adjust_editors.rs)：灰色 -> 色調和黑色 -> 色調 -> 隨著滑塊的白色，隨著滑度和色桿；
| UI-217-17 |顏色面板：滾輪加RGB/HSB/CMYK/Lab/Gray/Web滑桿模式；與其群組一起調整大小 |失蹤| [#217](https://github.com/storytold/photocraft/issues/217) | panel.rs:1837 單一欄位 + 十六進位/RGB 讀數 |
| UI-217-18 | Alt+滾輪以約 5% 的幅度緩慢縮放；變焦控制不受限制|失蹤| [#217](https://github.com/storytold/photocraft/issues/217) | canvas.rs:1107-1125 無 Alt 處理 |
| UI-217-19 |每個平台呈現的捷徑提示；顯示每個作業系統的標籤 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) |提示與工具提示透過 Shortcuts::pretty (6266b7a9, #262; Pretty_uses_the_platform_noinder);中（在資源管理器中顯示/在資料夾中顯示）； Reveal_label_names_the_platform_file_manager + smart_objects_reveal_item_names_the_platform_file_manager |
UI-217-20 | “在新文件中顯示尺標”首選項；指南存儲為 f64 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) |查看 › 尺標被記錄在unitsAndRulers.showRulers中，並在啟動和更改時應用（#2918； Vec<f64>到新指南、.pcraft 和 PSD 1032 (guides_keep_sub_pixel_positions_through_pcraft_and_psd) |
| UI-217-21 |一個預設管理器適用於每種預設類型，包括資料夾、搜尋、重新排序、載入/儲存集 |部分 | [#217](https://github.com/storytold/photocraft/issues/217) | edit_menu_cmds.rs:714 涵蓋筆刷、自訂形狀、可重新排序的圖案 |
| UI-217-22 |將可見標記標記到保留原件的新圖層中； Alt+Merge 保留原始內容 |完成 | [#217](https://github.com/storytold/photocraft/issues/217) | stamp_cmds.rs layer.stampVisible (Cmd+Alt+Shift+E) 將可見圖層「新圖層」中將原始圖層的新圖層合併. (Cmd+Alt+E) 也是如此。與Photoshop一樣，它們沒有自己的選單專案：Alt + Layer > Merge Visible / Merge Down / Merge Layers 執行它們（stamp_cmds::alt_variant、menus.rs）; stamp_tests.rs alt_click_on_merge_items_stamp
| UI-217-23 |圖層剪貼簿：在文件之間複製/貼上整個圖層（遮罩、樣式、類型）|部分 | [#217](https://github.com/storytold/photocraft/issues/217) | layer.copyToDocument將圖層複製到另一個開啟的文件中，保留名稱、內容、遮罩、樣式和混合，轉換為目標的設定檔和深度，作為一個歷史步驟（layer_copy_cmds.rs，#589）。它是在檔案之間拖曳的命令；仍然沒有複製/貼上圖層剪貼簿|

＃＃ 類型

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
|類型-218-1 |在路徑上鍵入：路徑形狀、沿曲線的字形、插入符號、PSD 往返 |部分 | [#218](https://github.com/storytold/photocraft/issues/218) | TextShape::Path (doc text.rs) ，字形按照弧長沿著曲線嚴格放置 (text warp.rs PathFrame)，type.A.缺少：使用「文字」工具點擊路徑、曲線後面的插入符號和選擇以及 PSD 路徑文字（另存為點文字）|
|類型-218-2 |面板和選項列中的段落方向（自動/LTR/RTL）；視覺 RTL 插入符 |失蹤| [#218](https://github.com/storytold/photocraft/issues/218) |引擎有 TextDirection (text.rs:155)；沒有使用者介面 |
|類型-218-3 |泰語/高棉語/印度語後備字體；複雜指令碼和 RTL 的黃金整形測試 |部分 | [#218](https://github.com/storytold/photocraft/issues/218) |後備字體涵蓋泰語、高棉語、老撾語、緬甸語（fonts.rs 中的THAI_FAMILIES、KHMER_FAMILIES、LAO_FAMILIES、MYANMAR_FAMILIES；#1909、#3259）、九種印度文字（INDIC_FAMILIES：Nirmala UI、Kohinoor 和 Sangam MN、Noto Sans）以及阿拉伯語和希伯來語。仍然缺少：針對參考輸出的每個指令碼的黃金整形測試（僅對安裝的字體形狀進行冒煙測試，而無需 .notdef，加上 bidi 測試）|
|類型-218-4 |來自字體軸的可變字體軸滑塊； opsz 遵循點大小 |完成 | [#218](https://github.com/storytold/photocraft/issues/218) | “字符”部分（“屬性”和“窗口”>“字符”）中樣式字體的每個軸一個滑塊，設置 type.setStyle 變體遵循大小；opsz 類型(layout.rsvariation_list)；樣式或權重會重設權重軸 (#2985)。 PSD 尚不往返軸值 |
|類型-218-5 | OpenType UI：字元面板按鈕、數字和文體集子選單 |部分 | [#218](https://github.com/storytold/photocraft/issues/218) |類型 > OpenType 切換 (type_extra_cmds.rs:512)。字元面板（和「屬性字元」部分）有Photoshop的 8 個 OpenType 按鈕：標準連字、上下文替代、任意連字、花飾、文體替代、標題替代、序數、分數，在選擇該功能時按下並調度 type.openType.* 命令 (type_tool.rs opentype_rowool.rs opentype_row; character_panel_opentype_buttons_toggle_and_show_the_feature，#3752）。仍然缺失：數字和文體集子選單，以及字體缺乏的灰色功能 |
|類型-218-6 |手動雙字距調整（Alt+箭頭），光學字距調整渲染，皆為往返 PSD (#206) |完成 | [#218](https://github.com/storytold/photocraft/issues/218) | Alt+向左/向右折插入符可調整該對的字距（type_tool.rs.rs. 100；type_tool_alt_arrows_kern_the_pair）。光學字距調整是根據 text/src/optical.rs 中的字形輪廓計算的，並在執行模式為光學時由 layout.rs 應用（optical_matches_photoshop、optical_kerning_tightens_open_pairs）。模式和手動值都透過 PSD 往返（io/tests/psd_kerning.rs；光纖模式在 Txt2、txt2_carries_optical_kerning 中傳輸） |
|類型-218-7 |扭曲文字對話方塊即時預覽並以圖層目前的扭曲開啟 |失蹤| [#218](https://github.com/storytold/photocraft/issues/218) | view_cmds.rs:834 始終播種弧線，彎曲 50 |
|類型-218-8 |文字編輯：全選、雙擊/三次/四次單擊選擇、貼上、帶測試 |完成 | [#218](https://github.com/storytold/photocraft/issues/218) |與Photoshop一樣，單擊 2 次選擇單詞，3 次單擊佈局行（垂直類型的一列），4 次回車之前的段落次所有文本，然後下一次快速點擊重新開始（#3493；type_tool.rs next_click_count、click_selection、multi_click）。透過點、段落和垂直類型的真實畫布點擊進行測試，使用貼上替換選擇內容並使用 Cmd+A 選擇所有內容（type_tool_tests.rs multi_click_selects_word_line_paragraph_and_all_in_point_text、multi_click_separates_wrapped_lines_from_paragraphs、click_counts_cycle_and_restart、click_selection_without_layout_and_on_hostile_input、without​​_iout_oo_o_o_no_no_no_ooo_no)_ooo_ooo_oo;
|類型-218-9 |在Photoshop中對實時類型進行扭曲/透視/扭曲（是：提供“點陣化並繼續？”；Photoshop不提示）|完成 | [#218](https://github.com/storytold/photocraft/issues/218) | XZZPH390ZX 2020 以下提示：2020 2020 任何提示：2020 20120 20920即時類型扭曲保持啟用狀態，沒有點陣化提示，並且僅提供扭曲樣式（無法拖曳網格），因此類型保持可編輯狀態；扭曲、透視和分割扭曲呈灰色。PhotoCraft執行相同操作：變形型別一步提交圖層的變形文字（transform_tool.rs Enter_type_warp、warp_cmds.rs warp_type_layer，#3004；warp_on_type_takes_a_style_and_keeps_the_type_editable），扭曲/900011000011 月 |

## 自動化

選單接線 (`docs/parity-checklist.md`)：631 個選單專案中的 631 個處於活動狀態 (100.0%)。這算的是調度，不是行為。

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
|自動1 |每個選單專案都會派發一個指令 ID（UI、CLI、控制頻道和 MCP 共用它們）|完成 | | docs/parity-checklist.md：所有選單專案均已生效；選單.rs is_live |
|自動-219-1 |以獨立於文件的形式記錄操作（隱式層、合併類型擊鍵）|失蹤| [#219](https://github.com/storytold/photocraft/issues/219) | actions.rs:47 複製具有明確圖層 ID 的原始日誌條目 |
|自動-219-2 |操作面板：步驟與對話方塊切換、重新排序、刪除步驟、停止、設定、.atn 導入 |完成 | [#219](https://github.com/storytold/photocraft/issues/219) | 2026-10-12：設定資料夾（可折疊、在集合之間拖曳、actions.assignSet）、對話方塊切換列（播放時會開啟步驟的對話方塊，並在確定時繼續；actions.dialog）、插入停止…（actions.insert）、步驟切換、重新排序、刪除步驟；面板選單中的載入/批次動作 .at (atn_cmds.rs)：90 個Photoshop事件雙向映射到命令，其餘事件逐字保留，按名稱播放時失敗；步驟切換、重新排序和刪除已完成的步驟；停止步驟播放（actions.stopMessage）但沒有插入停止菜單專案；設置和對話方塊切換被存儲和往返，但對話方塊不按組切換 |
|自動-219-3 |用於代理程式和 CLI 的 actions.record/stop/play/list 指令 |完成 | [#219](https://github.com/storytold/photocraft/issues/219) | actions_cmds.rs 中的 actions.list/get/record/stop/play/delete；面板、CLI 和 MCP 共享它們。嵌套步驟傳遞會話授權掛鉤。 |
|自動-219-4 |在第二個文件上重播裁切、篩選器、選取和樣式文字的測試 |完成 | [#219](https://github.com/storytold/photocraft/issues/219) | actions_cmds.rs：a_recorded_edit_run_replays_on_a_second_document |在一個文件上記錄裁切、高斯模糊、選取範圍填滿和人造粗體類型，並將所有這些重播到第二個文件上（加上原始圖層/選擇/填滿重播）|
|自動-219-5 |批次：子資料夾、開啟文件、命名範本、進度/取消、錯誤日誌、停止 |部分 | [#219](https://github.com/storytold/photocraft/issues/219) |file_cmds.rs 和 file_cmds/batch_naming.rs：資料夾/陣列輸入、每個檔案錯誤、{name}/cmds/batch_naming.rs：資料夾/陣列輸入、每個檔案錯誤、{name}/cmds/batch_naming.rs：資料夾/陣列輸入、每個檔案錯誤、{name}/cmds/batch_naming.rs：資料夾/陣列輸入、每個檔案錯誤、{name}/cmds/batch_naming。輸出模板和起始序列、包括所有子資料夾（includeSubfolders：深度/循環/文件計數有界，結果保留其子資料夾；批次和映像處理器對話方塊，#3753）；沒有開啟文件來源、取消或錯誤日誌檔案 |
|自動-219-6 |插件 ABI v2：面板類型、進度/取消呼叫、讀取其他層、最後一個過濾器重複 |失踪| [#219](https://github.com/storytold/photocraft/issues/219) |執行階段.rs:15 ABI_VERSION = 1; manifest.rs:18 Kind::僅過濾 |
|自動-219-7 |標記後面的本地 ONNX 模型，在首選項中進行管理；主題選擇，背景去除|失踪| [#219](https://github.com/storytold/photocraft/issues/219) |沒有 ONNX 執行階段相依性性 |
|自動-219-8 | xtask 檢查每個面板按鈕是否通過命令 id 路由 |失踪| [#219](https://github.com/storytold/photocraft/issues/219) |沒有這樣的 xtask 檢查 |
|自动-219-9 |用于画布内类型编辑、液化/木偶描边、变换手柄的无头命令 |完成 | [#219](https://github.com/storytold/photocraft/issues/219) |类型.hitTest、类型.插入符、类型.navigate； filter.liquify 笔画、edit.puppetWarp 引脚、type.edit、edit.transform |
|自動-219-10 |代理驗收套件成長到 30-50 個工作流程 |部分 | [#219](https://github.com/storytold/photocraft/issues/219) | Automation/tests/agent_tasks.rs 透過 MCP 執行 10 個工作流程 |

## 可靠性

29 個板條箱根中的 29 個具有永不崩潰屬性。

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
|相對1 | panic_hunt 使用對抗性參數模糊每個命令 |完成 | | engine/tests/panic_hunt.rs：每個命令，自己的執行緒，4秒超時；#[忽略]，跑進大門|
| REL-2 | 每個工作區板條箱都帶有永不崩潰的拒絕（clippy :: unwrap_used，...）屬性|完成 | |上面自動計數（板條箱根位於 crates/ 和 apps/ 下）；testkit 在 #221 中獲得了它 |
| 完成 | Guard_turns_a_panic_into_an_error 涵蓋導入/導出 (apps/photocraft) |
| REL-4 |每個文件寫入的原子保存（臨時、fsync、重新命名）|完成 | [#203](https://github.com/storytold/photocraft/issues/203) | photocraft_format::atomic_write (format/src/atomic.rs, #230): 相同目錄命名，sync_all，使用 WindowsmissionDUniedUnixed，UniedUnixUni; fsync；每個文件寫入網站都會透過它進行路由並 format/src/atomic/tests.rs 失敗注入寫入、重新命名和只讀目錄錯誤 |
| REL-212-1 |貼上/擦除/填滿/轉換/儲存/導入的錯誤分配，清除記憶體不足錯誤|部分 | [#212](https://github.com/storytold/photocraft/issues/212) |點陣區域/圖塊緩衝區、貼上/擦除/填滿/轉換和扭曲輸出使用易出錯的保留；導入/導入邊界保留現有分配狀態，但內部編碼/刪除/導入錯誤的保留
| REL-212-2 |當估計檔案大小超過 2 GB 時，匯出切換到 PSB |完成 | [#212](https://github.com/storytold/photocraft/issues/212) | PSD 匯出使用柵格、遮罩、色版和保留元資料的檢查大小估計 |
| REL-212-3 |測試：分配限制注入使每個命令出錯且文件不變 |部分 | [#212](https://github.com/storytold/photocraft/issues/212) |引擎故障點測試涵蓋粘貼、擦除、填滿、轉換、導入和保存事務邊界；編解碼器級分配器故障覆蓋範圍仍然開放

＃＃ 分配

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
| DIST-1 | macOS 通用建置 (aarch64 + x86_64)，已簽署與公證 |完成 | | release.yml:66,75,93 (macos-15); 0.2.0 出貨公證 |
| DIST-2 | Windows x64 和 x86 版本 |部分 | |發布.yml：113-114；跳過簽名並帶有警告，直到簽名材料存在 |
| DIST-3 | Linux x86_64 與 aarch64：AppImage、deb、rpm、tar.gz |完成 | |發佈.yml：162-163,186 |
| DIST-4 | Flatpak 單一檔案組合包、x86_64 與 aarch64 |完成 | |發佈.yml:204-216 (#163) |
| DIST-5 | Web 建置（wasm32、主幹）|完成 | |發佈.yml：312 |
| DIST-6 | FreeBSD 14 x86_64 tar.gz（/usr/local 樹）在 FreeBSD VM 中建立 |部分 | | release.yml:242-283（freebsd 作業）+打包/freebsd/package.sh；使用 --dry-run 檢查佈局，尚未發布佈局，尚未由發布佈局 |
| DIST-220-1 |幫助 > 檢查更新以及在所有桌面上選擇退出定期發布源檢查 |失踪| [#220](https://github.com/storytold/photocraft/issues/220) |板條箱中沒有更新檢查 |
DIST-220-2 |發佈矩陣中的 Windows ARM64 (aarch64-pc-windows-msvc)，簽章類似 x64 |完成 | [#220](https://github.com/storytold/photocraft/issues/220) | release.yml 在 Windows 矩陣中具有 { archtar: arm64, pcget: a64, pcget-window-pcms-pcms | .github/workflows/windows-arm64.yml 軟體包，MSI 將其安裝在 Windows 11 ARM64 執行程式上，檢查安裝的二進位檔案是否為 ARM64，並在卸載之前本地執行 CLI。簽名採用與 x64 相同的路徑：秘密是可選的，缺少秘密會產生未簽名的工件和警告（請參閱 DIST-2）|
| DIST-220-3 |應用程式內錯誤報告，包含可審閱的診斷信息，可打開預先填寫的問題 |失踪| [#220](https://github.com/storytold/photocraft/issues/220) |板條箱中沒有錯誤報告對話方塊 |
| DIST-220-4 | xtask 報告的每種語言的翻譯覆蓋率；阿拉伯語和希伯來語的 RTL 佈局 |部分 | [#220](https://github.com/storytold/photocraft/issues/220) |`cargo xtask i18n-coverage`報告每種語言的覆蓋範圍 (#427)； RTL 語言機鏡像內部 (XZZZ)，但ZZZTL）
| DIST-220-5 | Linux AppImage x86_64 + aarch64 船； Flatpak 保留；後來的 Flathub (#173) |部分 | [#220](https://github.com/storytold/photocraft/issues/220) | AppImage 和 Flatpak 發布（release.yZZPH438ZZX) | Flathub 16）；

## 開啟錯誤

|身分證 |目標|狀態 |問題 |注意|
|---|---|---|---|---|
| BUG-203-1 |每個文件寫入都會經過一次atomic_write (temp, fsync, rename) |完成 | [#203](https://github.com/storytold/photocraft/issues/203) | photocraft_format::atomic_write (format/src/atomic.rs, #230/8227071)cm; write_file和preset_store.rs、自動化files.rs/headless.rs/workspace.rs、應用程式services.rs write_atomic、格式store.rs和autosave.rs |
| BUG-203-2 | Unix 上同一目錄中的暫存檔案、sync_all、重新命名目標、fsync 目錄完成 | [#203](https://github.com/storytold/photocraft/issues/203) | atomic.rs：temp_name在同一個dir下，create_new + write_all +sync_all，重命名後的目錄 |
| BUG-203-3 | Windows：在 PermissionDenied 上透過退避重試重新命名 |完成 | [#203](https://github.com/storytold/photocraft/issues/203) | atomic.rs RenameRetry::platform()：在 Windows 上從 10 毫秒開始重試七次； retry_renamed 與自動化的 capststd 工作區共享；
| BUG-203-4 |明顯的不可寫目錄錯誤，永遠不會回退到就地寫入 |完成 | [#203](https://github.com/storytold/photocraft/issues/203) | atomic.rs 將只讀目錄、磁碟已滿和跨設備重命名錯誤映射到表明原始文件未更改的消息；沒有就地後備路徑|
| BUG-203-5 |發生任何故障時，原始檔案都不會受到影響，且臨時檔案會被刪除 |完成 | [#203](https://github.com/storytold/photocraft/issues/203) | atomic.rs 在每次失敗時刪除臨時檔案；測試斷言原始位元組在寫入和重命名失敗後仍然存在
| BUG-203-6 |失敗注入測試：寫入中、重新命名和只讀目錄錯誤保留原始位元組 |完成 | [#203](https://github.com/storytold/photocraft/issues/203) | format/src/atomic/tests.rs：故障 AtomicFs 注入寫入中的 StorageFull、重命名失敗和只讀目錄和試讀模式
| BUG-204-1 |介面首選項（UI 比例、字型大小、色彩頻道、滑桿、工具提示）有線或隱藏 |完成 | [#204](https://github.com/storytold/photocraft/issues/204) | prefs_ui.rs 勾選讀取 ui_scale (sync_display_scale)、ui_font_size (themetaat) _p:p)、ui_sadp)、 (sync_tooltips)； show_channels_in_color 連接在channel_thumbs 和畫布色版視圖中 (#2385)；dynamic_color_sliders 位於 prefs::HIDDEN_UNTIL_IMPLMENTED |
| BUG-204-2 | workspace.large_tabs、animated_zoom、auto_show_home_screen 有線或隱藏 |完成 | [#204](https://github.com/storytold/photocraft/issues/204) | workspace.large_tabs 控制canvas.rs中的文件選項卡高度； prefs::HIDDEN_UNTIL_IMPLMENTED |
| BUG-204-3 | RAW首選項與ask_before_ saving_layered_tiff讀取或隱藏|完成 | [#204](https://github.com/storytold/photocraft/issues/204) | Ask_before_ saving_layered_tiff 驅動程式 TIFF 選項提示 (ui-XZZ.4577 月)； raw_defaults.open_in_camera_raw 驅動 Camera Raw 對話方塊 (camera_raw_ui.rs)；其他 raw_defaults 欄位位於 prefs::HIDDEN_UNTIL_IMPLMENTED | 中
| BUG-204-4 |最近的檔案在重新啟動後保留透過首選項 | 完成 | [#204](https://github.com/storytold/photocraft/issues/204) |crates/ui-egui/src/lib.rs::push_recent 將清單儲存在首選項中； prefs_uiload 還原它，驗證重新加載 |
| BUG-204-5 |最近文件列表遵循最近文件計數（預設 20）|完成 | [#204](https://github.com/storytold/photocraft/issues/204) | lib.rs push_recent 使用centre_cap (recent_file_count) 限制列表； menus.rs 涵蓋預設和自訂限制 |
| BUG-204-6 |測試：每個首選項欄位都會在某處或隱藏的白名單上讀取 |完成 | [#204](https://github.com/storytold/photocraft/issues/204) | crates/engine/tests/prefs_usage.rs every_preference_is_read_or_hidden：每個欄位都在 pre]EDA_EDA_UU_U_U_5_6_67777_6_6 月(F儘管))A4idden：每個欄位讀取中列出，但絕不會兩者都 |
| BUG-205-1 |帶選取範圍的自由變換僅移動連結遮罩的選定部分 |完成 | [#205](https://github.com/storytold/photocraft/issues/205) | linked_mask_with_a_selection_moves_only_the_selected_region（transform_cmds.rs；5a5facc，242）|#242）|#242) |
| BUG-205-2 |即時變換預覽繪製應用了遮罩的圖層 |完成 | [#205](https://github.com/storytold/photocraft/issues/205) | Transform_tool.rs Preview_mask + Preview_applies_the_linked_mask（5a5facc，#242）|
| BUG-205-3 | edit.transform 尊重目標掩碼：只轉換目標、未連結的遮罩 |完成 | [#205](https://github.com/storytold/photocraft/issues/205) | Targeted_unlinked_mask_transforms_alone 和 Targeted_linked_mask_moves_withform_layer (transformer). 5a5facc, #242) |
| BUG-205-4 |測試：沒有選擇的鏈接點陣遮罩隨像素移動 |完成 | [#205](https://github.com/storytold/photocraft/issues/205) | linked_raster_mask_moves_with_the_pixels (transform_cmds.rs)：帶有隱藏一半的全部顯示遮罩，鏈接，無選擇；斷言，隱藏的一半並且留下任何像素的東西沒有顯示遮罩，鏈接，無選擇；
| BUG-205-5 |測試：連結遮罩+選擇保持對齊；未關聯的住宿；僅限目標面膜|完成 | [#205](https://github.com/storytold/photocraft/issues/205) |在transform_cmds.rs中的所有三個：linked_mask_with_a_selection_moves_only_the_selected_region檢查每個移動像素下的掩碼值，unlinked_mask_stays_put在有和沒有選擇的情況下覆蓋它，targeted_unlinked_mask_stays_put在有和沒有選擇的情況下覆蓋它，targeted_unlinked_mask_stays_magone_transforms_alone圖時在移動圖層時保持不變圖面碼|
| BUG-206-1 | CharStyle 模型按字元手動字距調整與自動字距調整模式分開 |完成 | [#206](https://github.com/storytold/photocraft/issues/206) | doc/src/text.rs CharStyle 包含：kern：用於自動模式（公制/字符/關閉）的字距調整和 kern：f302 1002 一個 1002 1002 1002 1002進行手動字距調整，新增至該對的自動字距調整 |
| BUG-206-2 | PSD 導入保留光學與手動字距調整值 |完成 | [#206](https://github.com/storytold/photocraft/issues/206) | io/tests/psd_kerning.rs manual_and_off_kerning_round_trip_through_psd 和 Optical_kerning_off_kerning_round_trip_through_psd 和 Optical_kerning_r_off_kerning_round_trip_through_psd 和 Optical_kerning_r_off_kerning_r_trip_through_psd 和 Optical_kerning_r_trips) 導入每個字元的對；光學模式在文件 Txt2 區塊中傳播 (#1348) |
| BUG-206-3 | PSD 匯出寫入真正的 AutoKerning 模式與 Kerning 值 |完成 | [#206](https://github.com/storytold/photocraft/issues/206) | Manual_and_off_kerning_round_trip_through_psd (io/tests/psd_kerning_kerning_round_trip_through_psd (io/tests/psd_kerning.rs) 從寫入字距的字距調整後的字距字元和A 字距中的字距中字距的 Tyning. (false, 100)，T 之後的 (true, -50)，並在樣式所示位置自動關閉字距調整 |
| BUG-206-4 | Alt+向左/向右折疊插入符可調整字距調整；字元面板欄位 |完成 | [#206](https://github.com/storytold/photocraft/issues/206) | type_tool.rs：Alt+Left/Right 在折疊插入符號處呼叫 kern_pair（每步 20，並使用 Cmd 為 1000 線值： parse_kerning）。由 type_tool_alt_arrows_kern_the_pair 和 kerning_field_values (input_tests.rs) 涵蓋 |
| BUG-206-5 |測試：手動和光學字距調整的 PSD 往返，oracle 語料庫中的真實文件 |部分 | [#206](https://github.com/storytold/photocraft/issues/206) | io/tests/psd_kerning.rs 涵蓋的往返行程（手動、關閉和光學，加上 hostile_kerning_valueues 和txt2_extras_survive_a_resave_of_unchanged_text）； corpus_text_layers 讀取真實的 PSD 並應用其 Txt2 光學字距調整，但沒有專門用於字距調整值的 oracle 檔案 |
BUG-207-1 |修飾工具尊重繪畫目標（遮罩、Alpha 色版、快速遮罩）|完成 | [#207](https://github.com/storytold/photocraft/issues/207) | run_lines 繪製channel_cmds::target_sur; retouch_lines_paint_the_targeted_cms ( #234) |
| BUG-207-2 |對所有圖層進行取樣適用於模糊、銳利化和塗抹 |完成 | [#207](https://github.com/storytold/photocraft/issues/207) | retouch_ui.rs 將sampleAllLayers 傳遞給paint.blur/sharpen/smudge； Sample_all_layers_reaches_the_paint.blur/sharpen/smudge； Sample_all_layers_reaches_the_command），#sme44），sm
|BUG-207-3 |塗抹混合預乘 RGBA（透明邊緣無暗條紋）|完成 | [#207](https://github.com/storytold/photocraft/issues/207) | Paint/src/retouch.rs 預乘；smudge_into_transparency_leaves_no_dark_fringe（e2444ce，234）|#234) |
| BUG-207-4 | edit.transform 轉換目標 Alpha 頻道 |完成 | [#207](https://github.com/storytold/photocraft/issues/207) | edit.transform 遵循目標 alpha 頻道或快速遮罩（自 #242 以來為顯式目標，自 #2835 以來沒有目標/圖層參數的路徑變換色版也被轉換到目標，自 #2835 以來沒有目標/圖層參數； Transforms_follow_the_targeted_alpha_channel | Transforms_follow_the_targeted_alpha_channel |
| BUG-207-5 |測試：每個工具遮罩/色版目標、所有圖層取樣、無污跡邊緣 |失蹤| [#207](https://github.com/storytold/photocraft/issues/207) |僅 smudge_drags_colour_along_the_lines (retouch.rs:407) |
| BUG-208-1 |拖曳矩形/橢圓選框時按住 Shift 鍵將其限制為正方形/圓形 |完成 | [#208](https://github.com/storytold/photocraft/issues/208) | canvas.rs marquee_corners 透過 Drag::marquee_mods 應用 Shift； marquee_tests.rs::shift_during_the_drag_draws_a_square_and_the_readout_follows 覆蓋拖曳 |
| BUG-208-2 |按住 Alt 的同時拖曳選取框從中心開始繪製 | 完成 | [#208](https://github.com/storytold/photocraft/issues/208) | canvas.rs marquee_corners 居中 Alt 拖曳；marquee_tests.rs::alt_draws_from_theift_centre_andfrom_theift_mand_shfrom_theift_draws_from_theift_re_and_shs_red_fir_sh;
| BUG-208-3 |拖曳選框時間標旁的寬 x 高 px 讀數 |完成 | [#208](https://github.com/storytold/photocraft/issues/208) | canvas.rs marquee_readout 繪製讀數； marquee_tests.rs 在拖曳時斷言尺寸並在釋放後將其刪除 |
| BUG-208-4 |從拖曳先前按下的修改器新增/減去/相交，由測試保留 |部分 | [#208](https://github.com/storytold/photocraft/issues/208) |按下時捕獲的 Mod（canvas.rs：1679，已使用：1779）；僅徽章測試 (tool_feedback.rs:195) |
| BUG-208-5 | Kittest：Shift 拖曳給出一個正方形，Alt 拖曳居中，讀數與大小匹配 |完成 | [#208](https://github.com/storytold/photocraft/issues/208) | crates/ui-egui/src/marquee_tests.rs 涵蓋 Shift、Alt、c/H 完成範圍 |
