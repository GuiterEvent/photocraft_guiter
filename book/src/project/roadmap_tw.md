# 路線圖

即時產品路線圖、里程碑定義、奇偶性測量和目前焦點均保留在 [`docs/roadmap.md`](https://github.com/storytold/photocraft/blob/main/docs/roadmap.md) 中。選單覆蓋範圍由`cargo xtask parity`產生為 [`docs/parity-checklist.md`](https://github.com/storytold/photocraft/blob/main/docs/parity-checklist.md)。

安全強化是跨領域的，而不是取代這些產品里程碑。目前的安全順序是：

1.威脅模型和報告政策；
2.經過身份驗證的控制會話；
3. 能力範圍的 MCP/控制存取；
4.工作空間根檔案系統句柄；
5. 請求、批次、渲染和連線預算；
6.更廣泛的解析器模糊測試和惡意輸入回歸語料庫；
7. 相依性策略並發布 SBOM/provenance。

本書僅建立第1項中的文件基礎。在實施和測試之前，後續的控制措施仍將被提議。
