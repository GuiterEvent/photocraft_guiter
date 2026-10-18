# 命令引擎

每個使用者可見的編輯操作都由`crates/engine/src/*_cmds.rs`中的命令規範表示，並由`crates/engine/src/commands.rs`註冊。規格提供 ID、標籤、選單路徑、捷徑、參數文件、啟用的謂詞和`run`實作。

```text
caller
  |
  | command id + JSON parameters
  v
command registry -- enabled/parameter checks --> command run closure
                                                   |
                                                   v
                                           Session + one history step
```

桌面 UI、CLI、控制協定和 MCP 工具都分派到此註冊表中。純視圖/視窗狀態是例外，由 UI shell 的命令表處理。

## 輸入合約

命令參數和目前文件狀態在命令邊界不受信任。對於缺失、類型錯誤、超出範圍或矛盾的參數，實作必須傳回`Err`。從輸入派生的值不得達到未經檢查的索引、劃分、分配大小或緊急路徑。

被忽略的`crates/engine/tests/panic_hunt.rs`整合測試使用對抗性參數來練習指令。更改後的命令還需要集中的優雅失敗測試，並應通過以下方式進行驗證：

```sh
cargo test -p photocraft-engine
cargo test -p photocraft-engine --test panic_hunt -- --ignored
```

確切的命令建立清單保留在現有的[貢獻指南](https://github.com/storytold/photocraft/blob/main/docs/contributing.md)中。

## 安全邊界註釋

註冊表驗證命令行為，但目前的自動化傳輸不提供每個命令的功能。到達命令調度的 MCP 或控制用戶端可以請求執行時`enabled`謂詞允許的任何公開命令。能力實施是[建議的自動化強化](../security/automation-security.md)，而不是實施的控制。
