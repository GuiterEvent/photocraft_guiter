# 貢獻

在編輯原始碼之前，請先閱讀[`AGENTS.md`](https://github.com/storytold/photocraft/blob/main/AGENTS.md)和現有的[貢獻指南](https://github.com/storytold/photocraft/blob/main/docs/contributing.md)。這些文件定義無塵室策略、板條箱分層、命令清單、測試要求、命名和視覺驗證規則。

重要的儲存庫不變量包括：

- 使用者可見的行為是作為引擎命令實現，除非它是純 shell/視圖狀態；
- 公共像素路徑不能採用 8 位元資料或 sRGB；
- 命令執行必須針對對抗性輸入回傳錯誤而不是恐慌；
- 格式工作包括往返、格式錯誤輸入和極限測試；
- L0-L6 更改必須繼續為WebAssembly進行編譯；
- 第三方資產需要許可許可證、相鄰許可證文件和`ATTRIBUTION.md`條目。

## 安全敏感貢獻

在更改解析器、檔案 I/O、序列化、自動化、MCP、控制伺服器或命令調度之前，請閱讀[威脅模型](../security/threat-model.md) 和[信任邊界](../security/trust-boundaries.md)。說明變更是否增加了已實施的控制、記錄了限製或建議稍後強化。當可以進行安全性測試時，為每個已修復的安全缺陷添加回歸測試。

請勿在問題、提交、固定裝置或模糊語料庫中包含即時漏洞利用負載、秘密、個人文件或未經許可的樣本。未公開的漏洞請關注【漏洞報告】(../security/vulnerability-reporting.md)。
