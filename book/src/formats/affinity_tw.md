#Affinity文件

PhotoCraft開啟本機Affinity文件：來自Affinity3 和`.afdesign`的`.af`文件，
來自Affinity1 和 2 的`.afphoto`和`.afpub`檔案。圖層以可編輯的PhotoCraft圖層出現：

- 頁面和畫板成為畫板（出版商跨頁並排放置）；
- 層和組變成組；
- 曲線和形狀（矩形、橢圓形、多邊形、星形和其他幾何形狀）變成
  形狀圖層及其填滿、漸層和筆畫；
- 藝術和框架文字成為文字圖層，使用電腦上安裝的字體繪製；
- 放置的JPEG和PNG影像成為嵌入式智慧型對象，像素圖層成為像素圖層；
- 保留向量和像素遮罩、不透明度、可見性、鎖定、名稱和混合模式。

畫板可透過舊標誌和目前Affinity3 畫板屬性進行識別。
當儲存為`.pcraft`或 PSD 時，它們會保留名稱、邊界和可編輯子專案；移動一個
畫板將其子項移動在一起。嵌套板成為遮罩組，並旋轉或彎曲
板目前使用其邊界矩形。

該文件的解析度為Affinity的 8 位元 RGB。PhotoCraft無法重現的是
文件開啟時在警告中列出，例如圖層效果、調整圖層和即時圖層
濾鏡、筆刷筆畫（繪製為普通筆畫）、雲朵和心形等特殊形狀（繪製為
省略號）、母版頁以及 CMYK 或 Lab 顏色（轉換為 RGB，無需文件設定檔）。
沒有警告就不會丟棄任何東西。

PhotoCraft無法寫入Affinity檔案。開啟的Affinity文件沒有儲存路徑：儲存要求
新檔案（`.pcraft`、PSD、PNG...）並且永遠不會覆蓋Affinity檔案。

如果無法讀取本機資料（損壞的文件，或PhotoCraft不知道的版面），PhotoCraft
開啟檔案中儲存的小 PNG 預覽Affinity，作為名為的單一像素圖層
“Affinity預覽”，警告說明了原因。文件 ▸ 放置無法放置或使用該預覽，
堆疊、批次或其他無法顯示警告的匯入。從下列位置匯出 PSD 或 PNG
Affinity在這種情況下。

請參閱[箱子的註釋](https://github.com/storytold/photocraft/blob/main/crates/affinity/README.md)
了解導入內容、閱讀器的編寫和驗證方式及其安全性的完整列表
限制。
