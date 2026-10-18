# PostScript 字體分辨率

PSD 文字可以透過其 PostScript 名稱以及家族/風格來識別臉孔。
PostScript 前綴不需要與姓氏匹配，且儲存的姓氏為
執行可以根據該名稱進行猜測。

＃＃ 解決

`FontDb::resolve_postscript`首先尋找具有該精確值的已安裝面
PostScript 名稱（先讀取共享猜測前綴的系列），然後
又回到了名稱的啟發式分割。最後的啟發式分割
連字符。沒有連字符的名稱會遺失尾部大寫樣式單字
（`GmarketSansBold`→ Gmarket Sans，700；`ExampleSansSemiBoldItalic`→ 例
Sans，600，斜體）。小寫結尾不是樣式（`Daylight`），而是樣式
供應商標籤`MT`後面的單字保留在系列中（`ArialRoundedMTBold`→
Arial 圓形 MT 粗體）。

當 PSD 匯入時，啟發式也會給出儲存在 PSD 上的族、權重和傾斜度
確切的面孔不存在。一個名字以風格詞結尾的傳統家族
之前沒有`MT`標籤 (`CooperBlack`)，然後儲存為較短的系列
（庫柏，900）直到安裝好面。

## 缺少字體

開啟 PSD 或Affinity檔案已經將執行的猜測系列替換為
PostScript 名稱解析為已安裝的系列。猜測仍在繼續
當導入時缺少臉部並且從`.pcraft`重新開啟文件時
安裝字型後、開啟後註冊字型時或
指令將`font`和`postscriptName`設定在一起。

缺失字體檢測（`missing_fonts`和`missing_fonts_in_layer`）並替換
因此，缺少字體使用與佈局相同的決策：僅缺少一個執行
當它的儲存家族、確切的 PostScript 外觀和啟發式方法都沒有時
家庭已安裝。已解決的執行不會被替換，因為未解決的執行
Run 分享其家族品牌。

＃＃ 成本

檢測僅解析其儲存系列不存在的執行的 PostScript 名稱
安裝。第一個這樣的查找讀取已安裝面孔的姓名表；的
結果被快取，直到註冊字體或加載系統字體，並且
佈局對這些執行執行相同的查找。
