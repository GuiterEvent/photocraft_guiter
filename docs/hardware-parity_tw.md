# 硬體奇偶校驗

> **最後評論：** 2026-10-10 · **最後更新：** 2026-10-10 · **更改：** 主要（第一個版本：針對Photoshop2026 的 GPU、手寫板、顯示器、每個PH平台的手勢） ·目標：** Adob​​1Ze XZZZ20

Photoshop2026 (27.11.0) 在每個平台上針對PhotoCraft所使用的每個硬體功能。Photoshop
側面：其捆綁包（Metal和 OpenCL 路徑、「HDR 顯示的精確色彩管理」和 EDR
字串、Touch Bar 資源、筆壓/傾斜字串）以及 Adob​​e 的 GPU 常見問題解答。PhotoCraft側：
`crates/tablet`,`crates/ui-egui/src/stylus.rs`,`apps/photocraft/src/gpu_startup.rs`,
`engine/src/display_color.rs`,`apps/photocraft/src/monitor_profile.rs`.

**總結（估計）：** ~50% 準備就緒。 GPU 合成與色彩管理顯示非常可靠；筆
不同平台的輸入不均勻（Windows 缺乏傾斜/旋轉，Wayland 損壞）且沒有 HDR
輸出。剩餘時間：80–150 小時（間隙 G12）。人類：每個作業系統上的平板電腦（Wacom、XP-Pen、Huion）和
HDR 顯示進行驗證。

## GPU

|特色 |Photoshop|PhotoCraft|狀態 |
|---|---|---|---|
| GPU 畫布、縮放/平移、旋轉視圖 |Metal(macOS)、DirectX/OpenCL (Windows) |wgpu30：Metal、Vulkan、DX12、GLGL？後端可選|完成；
| GPU 圖層合成、混合模式、效果 |部分（Photoshop複合材料主要在 CPU 上）|wgpu合成器，≤ 1/255 vs CPU 預言機；多色版和 > 紋理限制區域回落到 CPU |完成（在某些地方領先）|
| GPU 濾鏡（模糊圖庫、液化、智慧銳利化、Camera Raw）|是的 |用於過濾器的CPU（人造絲）；用於合成和效果的 GPU |部分 |
|大於 GPU 紋理限制的文件 |是（平鋪）| CPU 回退；正在進行平鋪 (#49) |部分 |
|無 GPU 時僅使用 CPU 的後備方案 |是的，減少了|`cpu`帆布偏好；在某些 Linux 系統上，如果沒有適配器則無法啟動（#2519、#2412）|部分 |
|舊/整合 GPU |是的 | Intel HD 上的黑線 (#2426)、OpenCore Mac 失敗 (#1859)、Rgba32Float 恐慌 (#2020) |部份 |

## 數位板

|特色 |Photoshop| macOS |窗| Linux X11 | Linux 韋蘭 |網頁 |
|---|---|---|---|---|---|---|
|壓力| 是的 |是（AppKit 監視器）|是的（WM_POINTER），神經質（＃2046）|是（XInput2）|僅透過 Xwayland；GNOME 50 筆無法點擊 (#2622)、KDE (#2297) |是的 |
|傾斜|是的 |是的 | **不** |是的 |沒有|是的 |
|桶旋轉|是的 |是（不適用於尖端角度，#1301）|沒有|部分 |沒有|是（扭轉）|
|橡皮擦端|是的 |是的 |否 (#2285) |部份 |沒有|部份 |
| WinTab 驅動程式模式 |是的 | – |沒有| – | – | – |
|壓力曲線 | Windows 墨水/驅動程式 |首選項中的全域曲線 (#2195) |相同|相同|相同|相同|

## 顯示和顏色

|特色 |Photoshop|PhotoCraft|狀態 |
|---|---|---|---|
|色彩管理畫布（文件 → 顯示器設定檔）|是的 |是的，折疊到 GPU 3D LUT |完成 |
|監控輪廓偵測|每個顯示器，現場|啟動時每個顯示器的 macOS；其他 sRGB 或選定的 .icc |部分 |
| 32 位元與 HDR 影像的 HDR / EDR 輸出 |是（「HDR 顯示的精確色彩管理」）| yes不; 8 位元 sRGB 編碼畫布紋理 |失蹤|
|高DPI，物理100% |是的 |是的（#1943）|完成 |
|多個顯示器：單獨作業系統視窗中的文件 |是的 |是 (`extra_windows`) |完成 |
|單獨作業系統視窗中的面板 |是的 |僅套用內浮動 (UI-217-5) |失蹤|

## 輸入設備

|特色 |Photoshop|PhotoCraft|狀態 |
|---|---|---|---|
|觸控板捏合縮放、兩指平移 |是的 |是的;在 0.6.0 中改為捏合捲軸 (#2463) |部分 |
|觸控板旋轉（旋轉視圖）|是的 |是 (`rotate_view.rs`) |完成 |
|觸控欄 |是（Intel Mac）|沒有|缺少（低值）|
|觸控螢幕（Windows）|是的 |僅基本指標 |部分 |
| 3D 滑鼠、MIDI、Surface Dial | Windows 上的 Surface Dial |沒有|缺失（低值）|
|掃描器（WIA / TWAIN）| WIA 選單 (Windows) |選單專案即時顯示；沒有驅動程式整合|部分 |

## 修訂歷史

|日期 |改變|總結|
|---|---|---|
| 2026年10月10日 |專業|第一版 |
