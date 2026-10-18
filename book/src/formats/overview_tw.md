# 格式概述

外部位元組透過四個主要路徑進入PhotoCraft：

-`photocraft-psd`解析並寫入 PSD 和 PSB 結構。
-`photocraft-codecs`解碼和編碼平面點陣格式。
-`photocraft-format`載入並儲存本機`.pcraft`套裝。
-`photocraft-affinity`讀取原生Affinity文件（`.af`、`.afdesign`、`.afphoto`、`.afpub`）：圖層、畫板、向量、文字、圖像和遮罩；它不寫它們。

`photocraft-io`將 PSD 和平面圖像對應到文件模型中，並將文件匯出回這些格式。它是整合層；解析限制仍然是底層格式箱的責任。

格式是主要的安全邊界。檔案副檔名不是信任訊號，元資料可以由攻擊者控制，小型壓縮輸入可以宣告非常大的解碼輸出。呼叫者應該使用有限的預設 API 並保留錯誤，而不是無限制地重試。

請參閱[PSD 和 PSB](psd-psb.md)、[點陣格式](raster-formats.md)、[SVG](svg.md)、[Affinity文件](affinity.md) 和 [`.pcraft`格式。跨格式強化指南位於[解析器強化](../security/parser-hardening.md)。
