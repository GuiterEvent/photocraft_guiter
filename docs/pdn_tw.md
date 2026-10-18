# Paint.NET 文件

PhotoCraft本機開啟 PDN3`.pdn`文件，無需 Paint.NET、Python、.NET 或外部
轉換器。檔案 › 開啟、最近使用的檔案、刪除、CLI 和自動化使用共用匯入器。
瀏覽器檔案選擇器也接受`.pdn`。

每個點陣圖圖層仍然可編輯。圖層順序、Unicode 名稱、可見性、不透明度和像素
阿爾法被保留。繪畫、遮罩、變換、重新排序、複製和撤消使用相同的方法
命令與其他像素層一樣。 CPU 和 GPU 支援所有 14 種 PDN3 混合模式。
Reflect、Glow、Negation 和 XOR 沒有Photoshop等效項，而 Color Burn 和 Color Dodge 有
單獨的 Paint.NET 變體，因為它們的邊緣情況不同。混合選單保持Photoshop的：
使用這六種模式之一的圖層在Photoshop之後列出自己的模式，而其他圖層則不會
給他們看。

PDN 支援**僅限導入**。 「儲存」開啟「另存為」並建議`.pcraft`，這會保留
圖層和每種混合模式。匯出為`.pdn`失敗並出現解釋性錯誤。 PSD 不能
代表 6 個 Paint.NET 變體；將它們匯出為 PSD 報告，它們的混合是
更改為正常。使用`.pcraft`進行繼續編輯，使用 PNG 進行扁平化渲染。

閱讀器支援 PDN3 的類型化 NRBF 對象，包括數字和舊的混合操作記錄，
BGRA32 表面帶有填滿行、共享像素記憶體和 gzip 或原始編號像素區塊。
它驗證引用、維度、步幅、計數、區塊範圍和 gzip 校驗和。元數據
物件嵌套上限為 64，層數上限為 1024，總解碼像素記憶體上限為 1 GiB。進口
可以在解析、解壓縮或複製像素時取消。不支援的佈局和
版本回傳錯誤。 Paint.NET 文件元資料（包括列印解析度和設定檔）
目前已被省略；非空用戶元資料會產生導入警告。圖層資料被保留。

＃＃ 確認

綜合單元測驗練習每個混合模式、名稱、隱藏層、不透明度、alpha、行
填入、反相區塊、舊/新模式記錄、本機儲存、取消和格式錯誤的資料。
GPU 奇偶校驗測試包含附加模式。本地真實文件檢查是可選的：

```sh
PDN_TEST_DIR=/path/to/pdn-and-png-exports cargo test -p photocraft-io --test pdn_corpus -- --ignored --nocapture
PDN_TEST_FILE=/path/to/layered.pdn cargo test -p photocraft-engine --test pdn -- --ignored
```

`PDN_TEST_DIR`中的每個 PDN 都需要具有相同基本名稱的對應`.png`匯出。的
測試根據匯出檢查像素並透過`.pcraft`驗證圖層保留。
將真實文件語料庫保留在應用程式儲存庫之外。

本地 Paint.NET 4.0.10 語料庫（14 個小紋理文件，1-8 層）也與其匹配
2.142/255 內嵌入全尺寸預覽。舊版 Paint.NET 在層之間進行舍入；PhotoCraft
保持浮點精度，因此分層複合不一定是位元組相同的。
提供的5.x文件（1280×720，11層，標題版本`5.3.8488.42200`）透過編輯，
繪畫、撤銷/重做和本機保存檢查。其嵌入式 256×144 預覽與 Catmull–Rom 相匹配
我們的渲染的下採樣，平均誤差為 0.418/255，最大誤差為 8/255。它的鄰近
PNG 與保存的 PDN 預覽不同，因此不適合作為全尺寸渲染預言。
這些檢查並不會與每個 Paint.NET 版本建立相容性。

格式參考：[MS-NRBF](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-nrbf/)
和 [pypdn 閱讀器](https://github.com/addisonElliott/pypdn/blob/master/pypdn/reader.py)。
此閱讀器獨立實作佈局；它不執行 .NET 反序列化。
