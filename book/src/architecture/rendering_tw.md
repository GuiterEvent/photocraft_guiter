# 渲染

PhotoCraft有兩個角色不同的合成器：

-`photocraft-compose`是匯出和測試使用的 CPU 合成器和正確性預言機。
-`photocraft-gpu`透過wgpu計畫並執行支援的畫布合成。

當規劃器可以表示文件時，互動式畫布使用 GPU 路徑。不支援的情況會回溯到 CPU 組成。目前文件將多色版文件和高於 GPU 紋理限制的區域識別為後備情況。

`photocraft-raster`中的像素儲存支援執行階段採樣深度，而不是公共 8 位元路徑。色彩模式和設定檔是文件資料；色彩轉換屬於`photocraft-cms`，而不是被視為隱式 sRGB。

＃＃ 確認

GPU 變更需要針對 CPU 預言機進行奇偶校驗檢查。 UI 變更需要實際渲染的影像，使用螢幕外快照範例或控制色版螢幕截圖：

```sh
cargo run -p photocraft-ui-egui --example snapshot
cargo test -p photocraft-gpu
```

對效能敏感的工作應避免每幀的全表面工作、按圖塊進行操作、跳過空圖塊並按修訂進行快取。 [`docs/development.md`](https://github.com/storytold/photocraft/blob/main/docs/development.md) 中列出了目前指令和基準測試入口點。

GPU 處理不是安全沙箱。在建立或分派資源之前，必須先驗證尺寸、解碼緩衝區、著色器和指令參數。
