# 顏色選擇和螢幕採樣

按一下“類型”選項欄或“字元”面板中的顏色樣本以開啟完整的顏色選擇器。它提供色域、分量條、HSB/RGB/Lab/CMYK 和十六進位字段以及當前/新色板。選擇目標範圍內的角色；如果沒有選擇字符，則目標是活動文本層。 **確定**透過`type.setStyle`適用； **取消** 使文字和歷史記錄保持不變。前景色與該對話方塊無關。

當檢色器開啟時，現有的畫布吸管會繼續對文件進行取樣，包括畫布縮放和平移。這無需作業系統螢幕擷取權限即可運作。

## 選擇PhotoCraft以外的顏色

**選擇螢幕顏色**可在完整的顏色選擇器和緊湊的顏色選擇彈出視窗中使用，包括填滿、形狀、漸層、圖層效果、指南和首選項。

在 X11、Windows 和 macOS 上，按一下它會拍攝所連接螢幕的一張快照，並在每台顯示器上開啟一個採樣視窗。將十字線移到某個像素上，使用滾輪將 11 × 11 像素放大鏡從 4× 變更為 24×，然後按一下進行選取。 **Esc** 取消。中心像素具有黑/白輪廓，其取樣的十六進位值顯示在放大鏡下方。

完整的對話方塊接收新顏色以供檢查； **確定**仍需要套用它。緊湊的彈出視窗保留其正常的立即編輯行為。保留現有的 alpha 和添加劑混合；浮動 RGB 小部件保留其線性 RGB 儲存。

螢幕快照保存在記憶體中，不會由選取器寫入或傳輸，並在選擇、取消或變更文件、圖層或對話方塊後釋放。最多 16 個顯示器的組合擷取預算為 64 × 1024 × 1024 像素（約 67 MP），但受 GPU 紋理大小限制。這些限制僅影響螢幕採樣，而不影響文件尺寸。

## 平台行為

- **Linux/X11：** 本機 Rust X11 用戶端擷取組合桌面。宣傳的主顯示器 ICC 設定檔案透過PhotoCraft的色彩管理庫轉換為 sRGB。沒有發布設定檔的輸出使用 sRGB 後備。
- **Linux/Wayland：** 桌面的 XDG`PickColor`入口網站擁有選擇和權限 UI 並傳回 sRGB。放大和縮放控制取決於入口網站實作；PhotoCraft無法直接擷取 Wayland 下的其他應用程式。
- **Windows：** 具有特定於顯示器的全螢幕視窗的本機螢幕擷取。擷取的像素目前使用 sRGB 後備；不保證 HDR 和任意顯示器設定檔的準確性。
- **macOS：** 安全的 CoreGraphics 綁定捕捉組合畫面並使用快照自己的 ICC 設定檔（如果可用）。可能需要螢幕錄製權限。現有的 macOS 11 部署目標將會保留。 Apple 已棄用視窗清單擷取 API，因此目前的 macOS 版本需要明確手動驗證；遷移到現代本機取樣也必須保持舊應用程式的相容性。
- **Web：** 公開標準 EyeDropper API 的瀏覽器使用瀏覽器的本機取樣/放大 UI。瀏覽器支援和縮放控制項各不相同。不支援的瀏覽器保留文件採樣和顏色欄位；外螢幕採樣不可用。

此實現遵循公開記錄的完整類型顏色選擇器和外部影像採樣的Photoshop行為。PhotoCraft使用明確的螢幕選取操作和凍結快照，而不是Photoshop的按下並拖曳手勢。

＃＃ 確認

單元和 UI 整合測試涵蓋選定文字與全層色彩變更、取消/確定和撤銷、將非同步選擇路由到其啟動彈出視窗、Alpha 保留、格式錯誤的擷取邊界、負顯示原點、HiDPI 對應和取消競賽。本機捕獲測試還涵蓋填滿像素佈局和記憶體預算。

使用合成藝術品產生螢幕截圖，而不會暴露您的桌面：

```sh
cargo run -p photocraft-ui-egui --example color_picker_demo -- /tmp/photocraft-color-picker
```

對於可重現的本機 X11 擷取驗證，明確忽略的測試會在隔離的 Xvfb 顯示器中顯示 8 × 8 合成樣本，將其擷取到記憶體中並檢查已知像素：

```sh
xvfb-run -a cargo test -p photocraft screen_capture_reads_a_known_x11_fixture_pixel -- --ignored --test-threads=1
```

Windows、macOS、Wayland 入口網站、瀏覽器啟動和混合 DPI 顯示器設定還需要在這些環境上進行手動驗證；模擬 UI 測試不能取代它。

參考資料：[Adobe 色彩選擇](https://helpx.adobe.com/photoshop/desktop/adjust-color/choose-colors/set-foreground-and-background-colors.html)、[XDG 色彩入口網站](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html)、[Apple 影像色彩空間](https://developer.apple.com/documentation/coregraphics/cgimage/colorspace)、[EyeDropper 規格](https://wicg.github.io/eyedropper-api/)。
