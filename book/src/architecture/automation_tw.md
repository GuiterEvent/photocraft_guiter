# 自動化架構

PhotoCraft透過多種傳輸公開相同的引擎：

```text
photocraft-cli commands/run/batch
             |
MCP stdio ---+--> photocraft-automation::Headless --> engine Session

MCP bridge --> BridgeClient --> 127.0.0.1 control server --> UI ControlRequest

JSON-lines stdio/TCP --> rpc::respond --> Headless::handle --> engine Session
```

無頭檔操作在`crates/automation/src/files.rs`中實現；RPC調度在`crates/automation/src/rpc.rs`中；MCP工具註冊在`crates/automation/src/server.rs`中；實時用戶端位於`crates/automation/src/bridge.rs`。桌面 TCP 偵聽器是`apps/photocraft/src/control_server.rs`。

控制和無頭 TCP 偵聽器僅限於環回位址。 Stdio MCP 避免偵聽網路套接字，但 MCP 處理程序仍以啟動它的使用者的檔案系統權限進行操作。

環回不是身份驗證。 TCP 傳輸需要承載令牌並強制執行請求位元組，
批次步驟和活動連線限制。遠端檔案方法使用`AuthorizedWorkspace`
獨立的讀寫目錄能力；路徑是相對於設定的根目錄的
當適用的根不存在時，關閉失敗。一般每個方法的範圍、用戶端身分、
加密，營運預算未落實。請參閱
[自動化安全模型](../automation/security-model.md) 和
[自動化安全](../security/automation-security.md) 在公開或擴展這些之前
表面。
