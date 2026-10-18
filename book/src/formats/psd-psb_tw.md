# PSD 和 PSB

`photocraft-psd`是一款基於 Adob​​e 公共 PSD 規範的獨立無塵室讀取器和寫入器。它支援 PSD 版本 1 和 PSB 版本 2，保留未知區塊以實現往返保真度，並延遲解碼色版資料。

解析器接受 crate 處理的所有四種 PSD 壓縮方法，並在`photocraft-io`將它們對應到`photocraft-doc`之前表示格式層級結構。目前的公共 API 和保真度行為記錄在 [`crates/psd/README.md`](https://github.com/storytold/photocraft/blob/main/crates/psd/README.md) 中。

## 實作解析器保護

目前的源強制實施了幾個明確的限制：

- 文件寬度與高度：各最多 300,000 像素；
- 文件頻道：最多56個；
- 層色版數：最多64個；
- 解碼頻道資料：一種解碼佈局最多 2 GiB；
- ActionDescriptor嵌套：最多64層；
- 圖案邊緣：最多30,000像素；
- 行計數和解碼長度的算術使用檢查操作。

格式錯誤的輸入測試、屬性測試和`cargo-fuzz`目標涵蓋主解析器和 ActionDescriptor 解析器。這些控制可以降低風險，但不能證明解析器對於每個惡意檔案都是安全的，也不能證明每個嵌入物件的總工作量是安全的。

## 剩餘硬化

優先事項包括更廣泛的 PSB 和嵌入式物件模糊語料庫，測量聚合分配和解析時間，確保每個嵌套長度都有預算，並保留最小化的崩潰輸入作為回歸裝置。請參閱[模糊測試和回歸測試](../security/fuzzing.md)。
