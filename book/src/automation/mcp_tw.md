#MCP

`photocraft-automation`透過 stdio 實作具有兩個後端的 MCP 伺服器：

- **無頭：** 處理程序內`Headless`會話擁有引擎並執行檔案 I/O。
- **橋接器：**`BridgeClient`將工具轉送到以`photocraft--control <port>`啟動的桌面應用程式。

使用以下命令啟動無頭伺服器：

```sh
photocraft-cli mcp --automation-read-root /work/project --automation-write-root /work/project
```

使用應用程式使用的相同私有令牌檔案橋接到正在執行的桌面應用程式：

```sh
photocraft-cli mcp --bridge 127.0.0.1:7878 --control-token-file /private/path/control.token
```

工具涵蓋會話/文件操作、命令發現和執行、批次以及橋接時的 UI 檢查和控制。工具模式提高了正確性，但不是授權邊界。

## 來自已安裝的版本

發布包與桌面應用程式一起提供`photocraft-cli`，因此不需要建置：

|安裝 |命令列 |
|---|---|
| Windows (MSI) |預設為`C:\Program Files\PhotoCraft\photocraft-cli.exe`（無論您以其他方式安裝在何處），而不是在`PATH`上 |
| Linux（deb、rpm）|`/usr/bin/photocraft-cli`|
| macOS |單獨的`photocraft-cli-<version>-macos-<arch>.zip`發布資產（`.app`僅包含桌面應用程式）|

```sh
# Windows, default install folder
claude mcp add photocraft -- "C:\Program Files\PhotoCraft\photocraft-cli.exe" mcp --automation-read-root <dir> --automation-write-root <dir>
# Linux, or macOS with the CLI unzipped onto PATH
claude mcp add photocraft -- photocraft-cli mcp --automation-read-root <dir> --automation-write-root <dir>
```

## 安全說明

Stdio MCP 不開啟網路偵聽器。它的用戶端僅接收讀和寫根
在啟動時明確授予；省略根就省略了該權限。橋接模式驗證其
在呼叫任何控制方法之前使用不記名令牌進行環回控制連接並使用
root 設定在桌面處理程序上。

目前伺服器不發出範圍會話或區分受信任的工具和高影響力的工具。身份驗證證明擁有令牌；它不會限制經過身份驗證的用戶端僅存取文件。在不新增外部策略邊界的情況下，請勿透過不受信任的代理、shell、代理或遠端服務公開任何傳輸。

檔案系統功能已實現。保留通用工具和方法功能
[建議](../security/automation-security.md)。

五個共享核心工具分別是`command_list`、`command_run`、`command_batch`、`doc_inspect`
和`render_preview`。現有記錄的工具（包括`doc_render_preview`）仍然保留
列出。`photocraft://document`和`photocraft://commands`公開即時 JSON。直接無頭
渲染影片支援使用授予的寫入根的進度和取消；看到
[`docs/mcp.md`](../../../docs/mcp.md) 用於參數、清理和傳輸行為。
