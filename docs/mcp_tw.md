# MCP 約定

PhotoCraft遵循與FilmCraft#28 相同的約定。啟動無頭會話：

```sh
photocraft-cli mcp --automation-read-root /work/project --automation-write-root /work/project
```

路徑是相對於對應根的，或其下的絕對路徑；無根
檔案系統權限不存在。橋接模式使用`--bridge`和`--control-token-file`作為
在[development.md](development.md)中描述。 UI 工具需要正在執行的桌面應用程式。

## 工具和資源

-`command_list`：發現引擎 ID、參數文件和目前啟用狀態；接受
`filter`和`enabled_only`。傳回一個裸 JSON 陣列
`{id, label, menu, shortcut, params, enabled, disabledReason, needsDocument}`物件（參見
  [指令清單回覆](control-protocol.md#command-list-replies))。
-`command_run`：`id`，選配`params`和`wait`。命令參數仍然由引擎定義。
-`command_batch`：訂購`steps`，可選`stop_on_error`（預設true）；停在第一個
  錯誤，除非錯誤。不是原子事務：早期成功的步驟仍然適用，並且
  沒有回滾。現有的每步驟結果和回覆預算將被保留。
-`doc_inspect`：圖層樹、歷史和選擇；空會話返回`document:null`。
  每個圖層（包括群組和嵌套子圖層）僅報告點陣`hasMask`，
`hasVectorMask`用於向量遮罩存在，而`vectorMaskEnabled`（`true`/`false`時）
  存在，`null`不存在時）。停用的向量遮罩仍具有`hasVectorMask:true`。
-`render_preview`：有界的 PNG 內容；`index`和`max_side`（預設 1024，最大 2048）。
-`ui_inspect`和`ui_screenshot`：橋接模式下的即時應用程式狀態/視窗。

現有文件/會話/作業/UI 工具仍然列出，包括已記錄的
`doc_render_preview`拼字。沒有隱藏的舊別名。每個工具都有一個標題
以及所有四個註釋提示。通用指令/控制工具保守地通告寫入；
文件保存可能會替換現有目標。提示不授予許可。

批次失敗後，`completed`和每步驟`results`報告成功步驟，而不是報告
回滾事務。使用`doc_inspect`檢查文件及其歷史記錄；指令
不必每次都新增一個歷史條目。對於可撤銷的編輯，請使用`command_run`
`{"id":"edit.undo","params":{}}`每個新增的可撤銷歷史記錄條目一次，檢查歷史記錄
當你康復時。撤銷不會回滾寫入磁碟的檔案。如果回覆預算錯誤顯示
目前步驟可能已完成，請在重試或撤銷之前檢查其效果。

在實驗之前，將`command_run`與`image.duplicate`和`{}`參數一起使用，或者
`history.newDocument`和`{}`參數（預設複製目前狀態）。兩者都創建了一個
新的活動文件，保持來源不變；對該副本進行實驗性編輯。
`history.newDocument`的選用`state`參數選擇保留的歷史狀態。

未知工具參數傳回 JSON-RPC`-32602`，在執行之前命名金鑰。批次
步進信封也很嚴格。嵌套引擎`params`保留其特定於指令的處理。
工具執行失敗返回`isError:true`。工人的恐慌變成了工具錯誤，
現有的中毒會話恢復繼續運作。無效的 JSON 行接收`-32700`
`id:null`；下一行仍在處理中。

嵌入式智慧型物件支援`command_run`和`layer.smartObjects.editContents`以及
`layer.smartObjects.convertToLayers`，可選擇定位`params.layer`。使用
`layer.smartObjects.saveContents`在開啟的內容文件中更新其在記憶體中的父級；
從內向外儲存巢狀內容，然後將`doc_save`父級儲存到作用域路徑。
內容文件上的`doc_save`會匯出該文件而不是更新其父文件。
記憶體命令不需要檔案系統根。需要的外部連結來源
文件讀取仍然被拒絕。參見【控制政策】(control-protocol.md#engine-commands)
對於共享限制。

`photocraft://document`和`photocraft://commands`傳回符合`doc_inspect`的即時 JSON
和`command_list`。資源讀取未快取；工具/資源目錄是私有的並且被快取
十分鐘。清單/讀取回應包括 MCP 2026-07-28 結果/快取欄位。

## 進度和取消

`file.export.renderVideo`的直接無頭`command_run`報告幀計數
`_meta.progressToken`存在（字串或數字）。通知嚴格增加，
除完成外，每秒最多限制為 10 個，並且包括`total`。無令牌
意味著沒有進度通知。渲染時 Ping 保持反應。

```json
{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"command_run","arguments":{"id":"file.export.renderVideo","params":{"dir":"frames","format":"png"}},"_meta":{"progressToken":"export-20"}}}
{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":20}}
```

首先建立文件和時間軸（`doc_new`，然後`timeline.create`）。導出使用
設定寫入根並在其下方建立`dir`。文件名必須是單一組成部分；
絕對路徑、遍歷和轉義符號連結都會被拒絕。現有目標檔名
被拒絕，包括符號鏈接，因此取消或失敗的作業無法替換早期的導出。
只有由該作業實際建立的檔案才會在取消、錯誤或逃逸恐慌時被刪除；
不相關的檔案和目標目錄保留。 PNG序列和動畫GIF分享
引擎渲染器。文件和播放頭未更改。

取消會抑制請求回應並在幀邊界處停止（當前
幀/編碼可能首先完成）。未知的請求 ID 將被忽略。渲染影片保持
即使使用`wait:false`也可以同步命令；其他具有工作能力的命令保留其現有的
`jobs_list`/`jobs_cancel`行為。批量匯出不發出 MCP 進度或支援每步
MCP 取消。嵌套引擎操作和橋渲染視訊仍然被現有的拒絕
環境檔案系統策略；新的作用域編寫器可用於直接無頭命令。
