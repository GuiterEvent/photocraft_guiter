# 命令列介面

`apps/photocraft-cli`提供以下目前子指令：

|命令 |目的|
|---|---|
|`convert`|開啟一個文件並將其匯出為另一種格式 |
|`info`|以 JSON 格式列印文件大小、模式、深度和圖層資訊 |
|`run`|開啟或建立文件，執行引擎指令，並可選擇儲存 |
|`batch`|將操作清單套用至一個輸入目錄中的檔案 |
|`droplet`|針對檔案或資料夾執行PhotoCraftDroplet |
|`commands`|列出指令 ID 和參數文件 |
|`mcp`|在 stdio 上啟動 MCP 伺服器，可選擇橋接至桌面應用程式 |
|`serve`|在 stdio 或環回 TCP 上啟動 JSON-lines 無頭伺服器 |

使用`photocraft-cli--help`（或`photocraft-cli<subcommand> --help`）作為可執行的事實來源。典型的有界命令執行是：

```sh
photocraft-cli run input.psd \
  --cmd image.adjustments.invert \
  --out output.png
```

### 缺少字體警告

如果文字圖層使用未安裝的字型系列，則`convert`和`run`
導出之前向標準錯誤寫入警告；他們並沒有默默地呈現
後備渲染為完全匹配。`info --compact`包括相同的
在其`warnings`JSON 陣列中發出警告，而不將其寫入 stderr。`run`
也將`warnings`加入到首先公開的命令的 JSON 結果中
失踪的家人。多命令`run`報告每個失蹤家庭一次，
即使後面的命令仍然使用它。

警告不會更改已儲存文件的字體聲明，也不會停止
操作。若要有意列出或替換缺失的族，請使用
文件上的`type.resolveMissingFonts`。

### 將選定的調整圖層匯出為 .cube LUT

引擎命令`file.export.colorLookupTables`從
恆等式 RGB 晶格。它可以匯出所有可見的頂級調整圖層
（預設）、目前圖層面板選擇或提供的列表
調整圖層 ID。`scope`和`layers`參數互為
當`scope`為`selected`時互斥：

```json
{"size": 33, "scope": "selected", "path": "chosen-look.cube"}
```

對於不使用「圖層」面板的自動化，請提供`"layers": [10, 14]`
相反。輸入 ID 順序不會改變最終色彩外觀：調整是
始終按照**文件堆疊順序**從下到上進行烘焙。當`path`為
省略，該命令返回`cube`中的 .cube 文字；兩種形式都返回
`size`和`layerCount`。

有效的 LUT 尺寸為 2–129（與PhotoCraft的 LUT 讀取器的最大值相同）。
選定的 ID 必須存在，是可見的頂級調整圖層，並且包含
沒有重複項；無效的選擇報告錯誤而不是默默地報告
匯出不同的外觀。遮罩、向量遮罩和剪裁是空間性的
操作並且不能用純色 LUT 忠實地表示，因此
貝克忽略了它們。群組包含的調整不會作為獨立導出
層，因為這會失去其組的混合語意。

提供給目前 CLI 的路徑是普通作業系統路徑。 CLI 不將它們限制為工作區根目錄。批次輸出建立請求的輸出目錄，儲存作業可能會建立父目錄。`batch`拒絕將輸出目錄作為其輸入目錄（結果將取代原始目錄），除非給予`--in-place`。僅在適當隔離的帳戶或環境中執行不受信任的操作文件。

CLI 解析將格式錯誤的標誌和 JSON 報告為錯誤，而子命令未採用的標誌（例如`--fromat`等拼寫錯誤）是使用錯誤（退出狀態 2）而不是被忽略。它不是安全性策略層。引擎命令仍然負責驗證自己的參數。
