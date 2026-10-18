# 信任邊界

當不受信任的位元組或請求變成內部文件狀態或授權的副作用時，就會跨越中心邊界。

```text
UNTRUSTED
  PSD / PSB
  PNG / JPEG / TIFF / WebP / EXR and other raster files
  .pcraft files and directory bundles
  ICC profiles / LUT files / pattern files / embedded objects
  CLI arguments / filesystem paths
  JSON control messages / MCP requests / agent commands
             |
             v
SECURITY BOUNDARY
  format parsers and decode budgets
  automation transports and request validation
  filesystem access layer
  command registry, enabled checks, and parameter validation
  serialization/deserialization
             |
             v
TRUSTED INTERNAL STATE
  document model
  engine Session and history
  CPU/GPU composition plans
  user-authorized filesystem effects
```

「可信任內部狀態」意味著程式碼可以相依性經過驗證的不變量；這並不意味著資料是無秘密的或顯示無害的。源自文件的元資料和文字仍然受到日誌、UI、檔案名稱和外部命令的污染。

## 邊界所有權

|邊界|主要程式碼 |所需檢查|
|---|---|---|
|模型的 PSD/PSB 位元組 |`crates/psd`，然後`crates/io`|長度、尺寸、色版、嵌套、減壓、偏移 |
|點陣位元組到影像|`crates/codecs`|尺寸、像素、解碼分配、特定於格式的限制 |
|`.pcraft`至文件 |`crates/format`|存檔範圍、解壓縮總數、雜湊值、模式/版本驗證 |
| JSON 到指令 |`crates/automation`、`crates/engine`|請求預算、方法/能力、參數類型/範圍、啟用狀態 |
|檔案系統效果的路徑 |`crates/automation/src/files.rs`，應用服務|授權根/句柄、連結策略、建立/替換語意 |
|文件到 GPU 工作 |`crates/gpu`，UI規劃師|紋理限制、緩衝區大小、支援計劃驗證 |

身份驗證和檔案系統讀取/寫入功能是在這些邊界實現的。
通用方法功能、結構化稽核事件和更廣泛的營運預算仍然開放。

## 代理指導

代理程式在修改任何解析器、檔案 I/O、自動化、MCP、控制伺服器、命令執行或序列化/反序列化路徑之前必須查看此頁面。呼叫者提供的路徑或工具請求是數據，而不是使用者授權的證明。
