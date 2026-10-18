#PhotoCraft應用程式圖標

**九尾眉菱狐**：雕刻肖像，坐著，回頭看
檢視器，在 Crafting Apps 的貓頭鷹模板框架中（全出血字段、頭部和身體填滿
瓷磚，尾巴從邊緣跑掉）。

## 調色板

正好三種顏色：

|顏色 |十六進位 |用於 |
|---|---|---|
|墨水|`#0b0b0c`|線條與輪廓|
|紙|`#efe9dc`|該圖 |
|PhotoCraft藍色（應用程式顏色）|`#2f7bf5`|全出血領域|

＃＃ 幾何學

512 單元圖塊 (`viewBox="0 0 512 512"`)，帶有`rx=112`的圓角正方形，無邊框。蘋果系統
渲染將其填滿到 Apple 的 824/1024 圖示網格上； Windows 和 Linux 渲染裁剪減少 22 個單位
每條邊的尺寸為 16-48 像素。

## 出處

擁有者的原始繪圖，使用ArtCraft製作（2880 像素，雕刻風格，摳像調色板），
使用 craftrule`assets/logo-options/_tools/vectorize_tile.py`進行向量化。源 PNG 保留在
`assets/app-icons/photocraft/source.png`的製程規則。許可證：請參閱`LICENSE.txt`。

## 文件

-`photocraft.svg`：規範母版（追蹤至 2048 像素）。
-`photocraft-small.svg`：較淺的追蹤（以 1024 像素追蹤）；還有 hicolor 可縮放圖示。
-`photocraft-1024.png`：在 macOS 網格上渲染 1024 像素。
-`photocraft.icns`：macOS 套裝圖示 (`CFBundleIconFile`)。
-`photocraft.ico`：Windows 圖標，16–256 像素，由`apps/photocraft/build.rs`嵌入到`.exe`中。
-`hicolor/<size>/apps/ai.storyteller.photocraft.png`(16–512) 和`hicolor/scalable/...svg`：
  Linux 圖示主題。

該應用程式還在執行階段設定視窗圖示和 Wayland 應用程式 ID（`apps/photocraft/src/app_icon.rs`、
`main.rs`）。在 Windows 上，工作列顯示啟動功能表捷徑的圖標
exe，因此MSI的`<Icon Id>`保留了`.exe`副檔名（ICE50）；`app_icon.rs`記錄了
詳細說明並測試`.ico`尺寸和 WiX 圖示參考。

## 再生

取代`photocraft.svg`（和`photocraft-small.svg`），然後執行`packaging/icons.sh`。它需要
`resvg`，以及 macOS 上的`iconutil`；`.ico`由`cargo xtask ico`包裝。
