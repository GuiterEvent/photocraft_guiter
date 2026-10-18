＃ 安裝

PhotoCraft目前是早期 alpha 專案。發布工件（如果可用）將透過儲存庫的 GitHub 發布頁面發布。在執行下載的工件之前驗證發行說明和校驗和。

對於來源構建，請繼續[構建](building.md)。桌面應用程式、無頭 CLI 和 Web 目標是單獨的工作區應用程式：

-`apps/photocraft`：本機桌面應用程式
-`apps/photocraft-cli`：無頭 CLI、JSON 行伺服器和 MCP 伺服器
-`apps/photocraft-web`：使用 Trunk 建置的WebAssembly應用程式

平台打包和簽名詳細資訊保留在現有的[發布文件](https://github.com/storytold/photocraft/blob/main/docs/releasing.md)中。簽名以設定的發布金鑰為條件；文物的存在本身並不能證明它是有簽名的。

## 信任輸入文件

將 PSD、PSB、`.pcraft`、點陣影像、設定檔、LUT、模式和嵌入文件內容視為潛在惡意內容。在特權或敏感環境中使用未知檔案之前，請先查看[安全文件處理](../security/secure-file-handling.md)。
