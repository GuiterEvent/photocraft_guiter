# macOS DMG 視窗

DMG 開啟時 Finder 顯示的內容：660 × 400 pt 背景，帶有應用程式圖示和
`Applications`並排連結。`package.sh`將這些檔案複製到鏡像中；這裡什麼都沒有
在建置時生成，因此 DMG 仍然使用`hdiutil makehybrid`建置（沒有安裝的設備，沒有
CI 上的 Finder 指令碼，簽名作業中沒有額外安裝任何內容）。

|文件|什麼 |
|---|---|
|`background.svg`|背景來源：應用程式圖示（`assets/app-icon/photocraft-small.svg`，鏈接，未複製）裁切為PhotoCraft色域 (`#2f7bf5`)、墨水和紙張上的封面。它的文字（Inter、JetBrains Mono）有輪廓，因此渲染它不需要字體。 |
|`background.tiff`|一張 HiDPI TIFF 中的背景為 1x（660 × 400 px，72 dpi）和 2x（1320 × 800 px，144 dpi）（16 色調色板、Deflate、sRGB；約 90 KB）。轉至`.background/background.tiff`。 |
|`dmg-layout.DS_Store`| Finder 的音量視圖設定：視窗大小、圖示大小 128、PhotoCraft.app 位於 (326, 205)、`Applications`位於 (574, 205) 以及背景。轉到影像中的`.DS_Store`；命名，這樣它就不會被誤認為（或像忽略一樣）Finder 產生的`.DS_Store`。 |
|`generate.py`|從 SVG 和上方的版面寫入`background.tiff`和`dmg-layout.DS_Store`。 |

## 磁碟區沒有版本

安裝的磁碟區名為`PhotoCraft`，而非`PhotoCraft<version>`。佈局指向背景
透過包含磁碟區名稱的別名，Finder 透過該名稱解析它：
版本化名稱視窗保持其大小和圖示位置，但不顯示背景（已測試）。
DMG 檔名 (`photocraft-<version>-macos-<arch>.dmg`) 仍附有版本。

## 規則

- **當視窗有背景時，Finder 在淺色和深色模式下以黑色繪製圖示標籤，
  因此兩個圖示下方的區域保持淺色（紙張）。
- **圖示框內沒有任何內容：** 圖稿與每個 128 pt 圖示框之間保持 10 pt 的距離
  其下方的標籤條。

## 再生

編輯`background.svg`（或`generate.py`中的佈局常數），然後在任何作業系統上執行：

```sh
pip install 'pillow>=12' ds_store==1.3.3 mac_alias==2.2.3
python3 packaging/macos/dmg/generate.py   # needs resvg on PATH
```

兩個輸出都是逐位元組可再現的，並且不攜帶執行它的機器的任何內容：
SVG渲染沒有任何字體，`dmg-layout.DS_Store`是從頭開始寫的，它的背景
別名僅包含磁碟區名稱和`/.background/background.tiff`。如果將文字新增至 SVG，
概述它（來自 resvg 的`usvg`將文字轉換為路徑）。視窗大小為 660 × 432，Finder 為 32 點
標題列；內容區域為660×400。

這是VectorCraft的`packaging/macos/dmg/`(storytold/vectorcraft#493) 的後續。
