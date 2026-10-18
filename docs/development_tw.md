# 開髮指南

## 先決條件

- 防鏽穩定（1.95+）。使用`rustup target add wasm32-unknown-unknown`新增 Web 目標。
- macOS、Windows 或 Linux。在 Linux 上，安裝視窗、GL 和 GTK 開發套件：
  - Debian/Ubuntu：`sudo apt install libxkbcommon-dev libwayland-dev libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev`
  - Fedora/RHEL：`sudo dnf install libxkbcommon-devel wayland-devel libX11-devel libXrandr-devel libXi-devel mesa-libGL-devel gtk3-devel`
  - 拱門：`sudo pacman -S libxkbcommon wayland libx11 libxrandr libxi mesa gtk3`
  - openSUSE：`sudo zypper install libxkbcommon-devel wayland-devel libX11-devel libXrandr-devel libXi-devel Mesa-libGL-devel gtk3-devel`

### Windows 原始碼構建

使用 Rust MSVC 工具鏈並安裝 Visual Studio 或 Microsoft C++ 建置工具
**使用 C++** 工作負載進行桌面開發。關注微軟的
[Rust 設定指南](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup),
然後打開一個新終端並檢查`cargo --version`和`rustc --version`。

在 Windows 11 上，智慧型應用程式控制可以阻止建置過程中產生的 Cargo、rustc 或執行檔。
如果建置報告`An Application Control policy has blocked this file. (os error 4551)`
([#1585](https://github.com/storytold/photocraft/issues/1585))，檢查：

- **Windows 安全性 → 應用程式和瀏覽器控制 → 智慧型應用程式控制設定** 其狀態。
- **事件檢視器 → 應用程式和服務日誌 → Microsoft → Windows → CodeIntegrity →
  對於被封鎖的可執行檔和策略可操作**，尤其是在託管電腦上。

此錯誤表示應用程式控制限制；移動`CARGO_TARGET_DIR`或
重新安裝 Rust 並不能證明被封鎖的可執行檔是可信的。參見微軟的
[智慧型應用控制常見問題](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions)
對於可用的控制項；沒有每個應用程式的例外。在受管理的裝置上，詢問您的
管理員有關批准的開發環境的資訊。如果您只想執行PhotoCraft，
使用[打包的 Windows 版本](https://github.com/storytold/photocraft/releases) 來避免
本機建置；下載的應用程式仍需接受 Windows 應用程式控制檢查。

## 建置並執行

```sh
cargo run --release -p photocraft -- path/to/image.psd      # desktop app
cargo run --release -p photocraft -- --control 7878 --control-token-file .private/control.token \
  --automation-read-root work --automation-write-root work
cargo test --workspace                                     # everything
cargo xtask ci                                             # fmt + clippy + tests + layers + wasm
cargo xtask stats                                          # tests and lines per crate
cargo xtask parity                                         # Photoshop menu coverage -> docs/parity-checklist.md
```

影像程式碼在`opt-level 0`處速度較慢，因此工作區設定檔在`opt-level 2`處建立相依性。使用`--release`進行任何交互作用。測試構建在`opt-level 0`上編譯 ui-egui本身（`Cargo.toml`中的`[profile.test.package.photocraft-ui-egui]`）：它是最大的測試構建單元之一，並且其測試不執行自己的繁重像素程式碼。

開發設定檔使用`debug = "line-tables-only"`建置：恐慌回溯保留檔案和行，測試二進位檔案連結速度大約是原來的兩倍。若要檢查偵錯器中的局部變量，請使用`CARGO_PROFILE_DEV_DEBUG=true`進行建置。

### 更快的測試循環

`cargo test`執行的大部分內容是連結測試二進位檔案並依序執行它們。迭代時：

```sh
cargo test -p <crate> --lib --tests        # skip examples and doctests (ui-egui has 11 examples)
cargo test -p <crate> --test <file>        # one integration-test file
cargo nextest run --workspace              # parallel across binaries (cargo install cargo-nextest --locked)
cargo test --doc --workspace               # nextest does not run doctests
```

[cargo-nextest](https://nexte.st) 在自己的處理程序中執行每個測試。`.config/nextest.toml`將 GPU 測試（`photocraft-gpu`和 ui-egui的整合和畫布/GPU 測試）一次最多 4 個一組，因為每個處理程序都會開啟自己的wgpu設備，並且不再序列化它們。在 32 執行緒 Windows 電腦上，完整工作空間從 297 秒 (`cargo test`) 縮短到大約 120 秒 (#2204)。忽略使用`cargo nextest run --run-ignored only`執行的測試。 CI 使用`--profile ci`（快速失敗，10 分鐘掛起超時）執行它，加上用於文件測試的`cargo test --doc`(#6117)；完整的`cargo test`保留 PR 之前的最終檢查。

## 字體（工藝字體）

Crafting 應用程式共享的字體資源位於 [storytold/craft-fonts](https://github.com/storytold/craft-fonts) 中，從未在此儲存庫中：不要在此處提交字體檔案（`assets/fonts/`中的 Inter 和 JetBrains Mono 是唯一的例外；新字體將轉到 craft-fonts）。規則位於同級`craftrules`結帳中的`craftrules/standards/fonts.md`中（見下文）；該儲存庫不是公開的，因此外部貢獻者可以向維護者詢問適用於其更改的規則。

craft-fonts 是一個 **可選的建置輸入**，絕不是 Cargo 相依性：

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p photocraft
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo test --workspace     # runs the Japanese font tests too
```

- 使用絕對路徑 (`$PWD/...`)：`build.rs`在`crates/text`中執行，因此將從那裡解析相對`CRAFT_FONTS_DIR`。
-`crates/text/build.rs`讀取`$CRAFT_FONTS_DIR/fonts/manifest.txt`並將字型嵌入為`photocraft_text::CRAFT_FONTS`(`crates/text/src/craft_fonts.rs`)。取消設置，`CRAFT_FONTS`為空，建置不變。錯誤的路徑是建置警告或`CRAFT_FONTS_REQUIRED=1`錯誤（發布版本設定兩者）。
- **UI：** 日文字體（首先是 BIZ UDPGothic Regular）是惰性 CJK 載入程式 (`crates/ui-egui/src/cjk_fonts.rs`) 中的第一個日文後備字體，位於系統日文字體之前並採用相同的語言環境指令碼順序，最後附加到每個egui系列，並採用通常的基線對齊方式。
- **鍵入工具：** 文字引擎將它們註冊在`FontDb::new`中（因此也沒有系統字體），並將它們放在按語言環境排序的後備列表的日語插槽中的第一位：用於 sans 執行的 BIZ UDPGothic，用於襯線執行的 Shippori Mincho / BIZ UDMincho。
- **Web：** wasm32 構建從未嵌入工藝字體，即使設定了`CRAFT_FONTS_DIR`：在 2026 年 10 月 06 日測量，僅 UI 介面就將發布的 wasm 從 24.19 MB 增加到了 28.87 MB，超過了`packaging/web/package.sh`中的 MB 增加到 MiB 門。相反，它會按需加載旁邊提供的字體：`fonts/manifest.txt`位於`index.html`旁邊，採用 craft-fonts 的清單格式（`crates/text/src/served.rs`；託管：`packaging/web/README.md`› 字體）。`PHOTOCRAFT_WEB_FONTS_DIR="$PWD/../craft-fonts" packaging/web/package.sh`將工藝字體放在那裡。服務的系列位於字體選單中，並在選擇或文件文字需要它們時取得；它們尚未出現在指令碼後備清單中，因此以拉丁字體設定的日文文字仍需要選擇日文字體（#1615）。
- 當`CRAFT_FONTS`為空時需要跳過字體並顯示訊息的測試； CI 在其自己的 Linux 作業 (`test (with craft-fonts)`) 中再次使用`CRAFT_FONTS_DIR`集執行它們。桌面版本在`.github/workflows/release.yml`中固定的提交（`CRAFT_FONTS_REF`；ci.yml 固定相同的提交）處檢查工藝字體，並將每個字體的`OFL.txt`發送為`OFL-<family>.txt`。

## 每個使用者一個實例

在 Windows 和 Linux 上，第一個PhotoCraft在設定目錄中的`instance.lock`上持有獨佔鎖，並偵聽`instance.json`中命名的環回連接埠（使用新的 256 位元令牌；檔案在 Unix 上為 0600）。稍後的啟動會將其檔案參數傳遞到此處並退出，因此從資源管理器或檔案管理器開啟的檔案將成為執行視窗中的選項卡（#3708）。只有當主裝置的 UI 取得檔案後才會回答`ok`；取得其他任何內容（主實例正在啟動、掛起或關閉）的啟動會持續重試鎖定長達 10 秒，一旦舊實例釋放，它本身就會成為主實例，然後啟動一個單獨的實例，因此檔案永遠不會被確認和遺失。當事件循環結束時（並且在 CPU 相容性重新啟動之前），主伺服器拒絕新檔案並釋放鎖定。它僅適用於每個使用者設定資料夾：可移植副本（可執行檔案旁邊的`PhotoCraftData`）或`PHOTOCRAFT_CONFIG_DIR`可以在使用者之間共用，因此這些啟動是單獨執行。`--new-instance`（或`PHOTOCRAFT_NEW_INSTANCE=1`）、每次`--control`啟動以及給定每個處理程序選項的啟動（`--safe-gpu`、`--present-mode`、`--frame-latency`、`--stroke-trace`、XZZPHZZX、XZZZ）也是如此。當已安裝的PhotoCraft正在向前執行階段，`cargo run`：透過`--new-instance`或設定`PHOTOCRAFT_CONFIG_DIR`。 macOS 從 LaunchServices (`apple_events.rs`) 獲得相同的行為。在本機 Wayland 上，正在執行的視窗無法自行前進（這需要啟動器的`XDG_ACTIVATION_TOKEN`，而winit0.30 無法用一個啟動現有視窗），因此它會要求引起注意，工作列會突出顯示它； X11、XWayland 和 Windows 將其帶到了前台。 Flatpak 需要自己的後續（#1540）。程式碼：`apps/photocraft/src/single_instance.rs`。

## 圖形啟動和裝置遺失

`--safe-gpu`從一次啟動的 CPU 渲染器開始（沒有 GPU 畫布；平台有一個視窗的軟體適配器：Windows 上的 WARP，Linux 上的 llvmpipe over GL）。在創建wgpu裝置之前，應用程式會在 config 目錄中寫入並鎖定 XZZPH100​​ZZX；一旦第一幀渲染完畢，它就會清除它。發現解鎖標記的啟動知道先前的啟動已在圖形驅動程式中死亡（#4），並使用下一個更安全的後端（Windows：Vulkan→ DX12 → CPU；Linux：Vulkan→ GL → CPU；macOS：XZZPH98：Vulkan→ GL → CPU； › GPU 後端，使用 **重置 GPU後端**）。 CPU 復原是暫時的，直到在復原對話方塊中選擇 **Keep Use CPU**；關閉對話方塊或不選擇退出會保留先前的 GPU 偏好設定 (#3483)。對於`auto`，Windows 上的英特爾轉接器使用 DX12。說明 › 系統資訊顯示適配器、後端、驅動程式和後備狀態。

在 DX12 上，著色器編譯器是 FXC（`d3dcompiler_47.dll`，Windows 的一部分），或放置在`photocraft.exe`旁的`dxcompiler.dll`，透過其完整路徑載入。wgpu預設會依照名稱尋找`dxcompiler.dll`，它到達目前目錄和`PATH`並載入其他程式的不相容版本 (#712)。

處理的視窗和應用程式初始化錯誤會清除此啟動的啟動標記，恢復任何
之前的碰撞證據。例如，沒有`DISPLAY`或`WAYLAND_DISPLAY`的 Linux 啟動不會
更改下一次啟動的圖形後端。渲染器初始化錯誤和驅動程式崩潰仍然存在
保留標記以便恢復。

如果裝置在運作時遺失 (#243)，每個 GPU 入口點都會先檢查裝置的運作狀況標誌，畫佈在會話的其餘部分切換到 CPU 合成器，並顯示一則通知「GPU 裝置遺失；使用 CPU 渲染器」。`ui.gpu.simulateLoss`從控制頻道觸發此路徑。

## 紀錄

桌面应用程序将其`log`记录写入标准错误和设置目录中的`logs/photocraft.log`（Linux`~/.config/photocraft/logs/`、macOS`~/Library/Application Support/Photocraft/logs/`、Windows`%APPDATA%\Photocraft\logs\`或`PHOTOCRAFT_CONFIG_DIR`/便携式数据文件夹下）。從桌面選單或 Dock 啟動的啟動沒有終端，因此該檔案是附加到錯誤報告的內容：GPU 啟動回退、遺失的裝置和防撞程式的恐慌報告都在那裡。每次啟動都會將前一個日誌移至`photocraft.1.log`（並將該日誌移至`photocraft.2.log`），因此崩潰的執行日誌會在下一次啟動時繼續存在。文件在 16 MiB 時停止增長。`--version`和命令列錯誤不寫入檔案。

預設情況下，PhotoCraft自己的板條箱記錄在`info`上，其他所有內容記錄在`warn`上。`RUST_LOG`將其替換為 env_logger 樣式指令，例如`RUST_LOG=debug`、`RUST_LOG=warn,photocraft_gpu=trace`或`RUST_LOG=info,wgpu_core=warn`；以`*`結尾的指令涵蓋以它 (`photocraft*=debug`) 開頭的每個目標。記錄器是`apps/photocraft/src/logging.rs`； Web 建置會記錄到瀏覽器控制台。

## 環境變數

|變數|效果|
|---|---|
|`PHOTOCRAFT_CONTROL_PORT`|與`--control <port>`相同|
|`PHOTOCRAFT_NO_PRINT`|設定為`0`以外的任何值：`file.print`/`file.printOneCopy`拒絕假脫機（`dryRun`仍然有效）。 CI設定它，用`LPDEST=photocraft-no-such-printer`作為第二個網；在本地執行恐慌搜尋或模糊器時同時設定 |
|`RUST_LOG`|標準錯誤和日誌檔案的日誌等級（請參閱[日誌](#logs)）|
|`PHOTOCRAFT_CONTROL_TOKEN`|用於控制 TCP 的 64 進位承載令牌（避免在可以進行環境檢查的共用系統上）|
|`PHOTOCRAFT_CONTROL_TOKEN_FILE`|讀取或為伺服器建立控制承載令牌檔案 |
|`PHOTOCRAFT_AUTOMATION_READ_ROOT`|僅限桌面應用程式：自動讀取根目錄。無頭 CLI 模式需要`--automation-read-root`標誌 |
|`PHOTOCRAFT_AUTOMATION_WRITE_ROOT`|僅限桌面應用程式：自動寫入根目錄。無頭 CLI 模式需要`--automation-write-root`標誌 |
|`PHOTOCRAFT_CPU_CANVAS=1`|強制使用 CPU 畫布路徑而非wgpu著色器畫布 |
|`PHOTOCRAFT_NATIVE_WAYLAND=1`| Linux：連接筆時保持在本機 Wayland 上（預設情況下，視窗透過 Xwayland 打開，因為 Wayland 不給應用程式提供筆輸入；#639）。筆是具有`BTN_TOOL_PEN`和`ABS_X`/`ABS_Y`軸的輸入裝置；聲稱每個按鍵都不是筆的虛擬鍵盤 (#1072) |
|`WGPU_BACKEND=dx12`|選擇wgpu後端（`vulkan`、`dx12`、`metal`、`gl`）；覆蓋`performance.gpuBackend`和啟動回退 |
|`WGPU_DX12_COMPILER=fxc`| DX12 著色器編譯器（`fxc`、`dxc`、`auto`）；`dxc`和`auto`通過 DLL 搜尋路徑查找`dxcompiler.dll`|
|`PHOTOCRAFT_GPU_TILE=2048`|強制 GPU 畫布平鋪（測試平鋪接縫）|
|`PHOTOCRAFT_FX_NOCACHE=1`|繞過CPU圖層效果圖快取（`compose::effect_maps`）|
|`PHOTOCRAFT_CPU_COMPOSE=1`|保留wgpu畫布，但在 CPU 上進行合成（比較 GPU 與 CPU 渲染，例如與快照範例）|
|`PHOTOCRAFT_LOCALE`|這次發布涵蓋 Auto 的原生 UI 語言；不支援的標籤使用英文。請參閱[系統語言檢測](localization.md#first-launch-and-system-language)。 |
|`PHOTOCRAFT_FX_TRACE=1`|列印每次重建花在 GPU 效果形狀和距離場上的 CPU 時間 |
|`PHOTOCRAFT_STROKE_TRACE=1`（或`=<file.csv>`；還有執行階段的`--stroke-trace <stderr|file.csv>`或`ui.set {"strokeTrace": true|false}`，沒有檔案）|對每個筆觸的每一幀進行計時；列印每個筆畫的摘要，並使用路徑將每個幀附加到該筆畫延遲（追蹤）
|`PHOTOCRAFT_PRESENT_MODE=novsync`（或`--present-mode`）|本次發布的當前模式：`fifo`、`autovsync`或`novsync`（還有`immediate`、`mailbox`：無端驅動程式提供的垂直同步程式預設值：垂直同步，或當「首選項」>「效能」>「低延遲繪圖」開啟（出廠時已關閉）時，無垂直同步。用於延遲比較；筆畫軌跡中顯示|
|`PHOTOCRAFT_FRAME_LATENCY=1`（或`--frame-latency`）|交換鏈可能排隊的幀，1 到 3（wgpu的 GL 後端忽略它）。用於延遲比較；筆畫軌跡中顯示|
|`PHOTOCRAFT_THEME_FILE=tokens.json`| **僅偵錯建置：**即時設計令牌覆蓋，更改時重新讀取|

### 現場設計代幣

```json
{ "card": "#323232", "tab_strip": "#262626", "accent": "#378ef0", "radius": 4 }
```

鍵是`theme::Tokens`(`crates/ui-egui/src/theme.rs`) 的欄位名稱。在應用程式執行階段編輯並儲存文件，即可立即查看更改，無需重新編譯。這是從發布版本編譯出來的。

### 中風潛伏期追蹤

若要尋找給定機器上筆畫的輸入到像素時間的位置 (#1343)，請使用`PHOTOCRAFT_STROKE_TRACE=1`（標準錯誤上每個筆劃的摘要）或`PHOTOCRAFT_STROKE_TRACE=strokes.csv`（也是附加到該檔案的每個畫面）啟動。用滑鼠或筆畫幾筆。每個摘要給出 p50/p95（以毫秒為單位）：

```text
stroke trace: 62 frames on vulkan NVIDIA GeForce RTX 5090 (p50/p95 ms): interval 8.33/8.53 moves/frame 1.00/2.00 stroke 0.80/1.53 canvas 0.40/0.75 ui 0.91/1.41 cpu 2.20/2.84 wait 6.04/6.64
```

|專欄 |它是什麼 |
|---|---|
|`interval`|繪製時幀之間的時間開始（應用程式保持執行階段顯示器的刷新週期）|
|`moves/frame`|指針移動每幀交付的作業系統（控制色版`ui.pointer`輸入繞過原始輸入併計數為 0）|
|`stroke`|渲染筆畫的新點（即時筆劃；在最後一幀上，提交）|
|`canvas`|畫布刷新：合成並上傳|
|`ui`|建立 UI，上面兩個包括 |
|`cpu`|eframe每幀的 CPU 時間：UI、曲面細分與 GPU 提交，無需垂直同步等待 |
|`wait`|`interval − cpu`：等待垂直同步或交換鏈，或閒置 |

應用程式無法計時的是光子輸入的其餘部分：幀開始前的作業系統輸入佇列（最多一個`interval`）、排隊等待顯示的幀（表面最多保留兩個；GL 後端忽略該設定）以及合成器。因此，第一行命名了後端以及框架的呈現方式。若要比較示範設定而不重建，請使用`PHOTOCRAFT_PRESENT_MODE`和`PHOTOCRAFT_FRAME_LATENCY`啟動（請參閱[環境變數](#environment-variables)）：使用`novsync`，應用程式不再等待垂直同步（`wait`接近 0）：每次刷新畫面可能會顯示在最新的畫面時，即僅在最新的畫面可能會顯示螢幕時，即僅在刷新螢幕的情況下。最後的總結也在控制色版的`perf.strokeTrace`（`ui.inspect`）。程式碼是`crates/ui-egui/src/stroke_trace.rs`。

## 以程式設計方式驅動應用程式

使用私有令牌檔案啟動應用程式。然後，它接受`127.0.0.1:7878`上經過驗證的 JSON 行：

```sh
TOKEN=$(tr -d '\r\n' < .private/control.token)
printf '%s\n' "{\"id\":\"auth\",\"method\":\"auth\",\"params\":{\"token\":\"$TOKEN\"}}" \
              '{"id":1,"method":"engine.execute","params":{"command":"layer.newAdjustmentLayer.hueSaturation","params":{"hue":30}}}' \
              '{"id":2,"method":"ui.screenshot","params":{"path":"evidence/shot.png"}}' | nc 127.0.0.1 7878
```

每種方法請參閱`docs/control-protocol.md`。尖端：

- **macOS 不會渲染被遮蔽的視窗。 **`ui.screenshot`首先升起視窗（預設為`focus: true`）。在進行其他目視檢查之前使用`ui.focus`。
- **僅捕獲我們自己的視窗。 ** 對於本機捕獲（查看真實的標題列和交通信號燈），透過視窗 ID (`screencapture -l <CGWindowID>`) 捕獲，而不是螢幕區域，它可以捕獲其他應用程式。
- **指標手勢：**`ui.pointer`取得*文件*座標中的事件，並透過與滑鼠相同的工具狀態機執行它們。
- **查看圖塊儲存：**`ui.set {"tileGrid": true}`覆蓋活動圖層的儲存圖塊，這顯示了圖層的稀疏程度以及哪些圖塊共享像素 (`ui-egui/src/tile_overlay.rs`)。

## 無頭 CLI

`apps/photocraft-cli`建置二進位檔`photocraft-cli`：

```sh
cargo run -p photocraft-cli -- convert in.psd out.pcraft               # any supported format -> any
cargo run -p photocraft-cli -- info out.pcraft                          # JSON: size, mode, depth, layer tree
cargo run -p photocraft-cli -- run in.png --cmd layer.new.layer --params '{"name":"Ink"}' \
                                          --cmd filter.blur.gaussianBlur --params '{"radius":3}' --out out.psd
cargo run -p photocraft-cli -- run --new '{"width":800,"height":600}' --cmd document.inspect
cargo run -p photocraft-cli -- batch --actions actions.json --in photos/ --out done/ --format jpg
cargo run -p photocraft-cli -- commands --filter blur                    # the command registry
```

`actions.json`儲存記錄作業的步驟：`[["<id>", {…}], …]`、`[{"command": "<id>", "params": {…}}, …]`或裸 ID，作為清單或包裝在`{"actions": …}`或`{"steps": …}`中；Droplet 檔案也可以。它也可以是一個動作集（動作面板的`actions.json`、`{"actions": [{"name": …, "steps": […]}, …]}`）：`--action <name>`選擇要播放的動作（當該集包含一個動作時可選），並且為每個文件加載整個集，因此播放該集的另一個動作的步驟有效。`batch`拒絕作為`--in`資料夾的`--out`資料夾，因為結果將取代原始資料夾，除非給出`--in-place`。`run`對於每個指令結果列印一行 JSON。

## 本機格式 (.pcraft)

`crates/format`定義無損本機組合包。它可以是 STORE 條目的 ZIP，也可以是相同佈局的目錄：

-`manifest.json`：版本化文件樹，在`format/src/migrate.rs`中進行遷移。
-`tiles/<blake3>.zst`和`blobs/<blake3>.zst`：zstd 壓縮對象，透過其未壓縮位元組的 BLAKE3 雜湊進行內容尋址。
-`thumb.png`和`composite/preview.png`：預覽。

每個開啟的文件保留一個`PcraftWriter`：重新儲存然後僅壓縮並寫入更改的圖塊。目錄包在資料夾中第一次遇到對象時驗證對象；稍後保存，在文件大小和修改時間不變的情況下重用它們，並重新驗證更改的對象，修復丟失或損壞的對象，並對未引用的對象進行垃圾收集。`format::Autosaver`將快照寫入後台執行緒上的復原目錄。`list_recovery`/`recover`/`discard_recovery`實現崩潰恢復，`format::RecoveryStore`是桌面應用程式使用的生命週期：新文件在每次啟動鍵下自動保存（文件 ID 每次啟動都會重新啟動，因此它們永遠不會覆蓋舊條目），恢復的文件採用其來源的條目。該條目會被下一次自動儲存替換，並且僅在儲存或關閉文件時刪除，而不僅僅是因為它已恢復，因此第二次當機不會遺失任何內容。 Web 建置沒有崩潰復原（沒有自動儲存服務）。

啟動時，桌面會列出恢復元數據，然後透過現有的後台作業系統一次解碼一個文件。此視窗保持可用，狀態列中顯示恢復進度和取消。取消會跳過該啟動恢復佇列的其餘部分並阻止目前結果開啟；解碼器可能在背景完成。失敗或取消的載入會保留其復原檔案完整以供日後啟動。關閉首選項 › 檔案處理 › 啟動時恢復會完全跳過發現。

`photocraft-io`透過`import`/`export`中的這個板條箱路由`.pcraft`，透過魔法或擴充來偵測它。

## MCP（代理）

`photocraft-climcp`使用基於`rmcp`構建的`crates/automation`在 stdio 上提供 MCP：

- **無頭：**`photocraft-climcp --automation-read-root <dir> --automation-write-root <dir>`。它驅動處理程序內引擎會話，並且在省略 root 時沒有檔案權限。
- **即時應用程式：**啟動`photocraft--control 7878 --control-token-file <private-path> --automation-read-root <dir> --automation-write-root <dir>`，然後執行`photocraft-climcp --bridge 127.0.0.1:7878 --control-token-file <private-path>`。桌面處理程序擁有根源。參見`docs/control-protocol.md#mcp-bridge`。

對於無頭 MCP 用戶端，將絕對路徑傳遞到啟動參數中故意選擇的受信任工作區目錄。桌面`PHOTOCRAFT_AUTOMATION_READ_ROOT`和`PHOTOCRAFT_AUTOMATION_WRITE_ROOT`環境變數**不**授予無頭 CLI 存取權限。省略根標誌會故意拒絕該訪問方向。

工具：

-`session_list`
-`doc_open`、`doc_new`、`doc_place`、`doc_export_layers`、`doc_export_data_sets`、`doc_import_data_sets`、`doc_save`、XZPHZ292Z24XZZX PNG影像），`doc_select`，`doc_close`
-`command_list`、`command_run`、`command_batch`（每次呼叫多個指令）
-`jobs_list`（執行有進度的後台作業，然後最近完成的作業及其結果或錯誤），`jobs_cancel`（按 ID 的一項作業，或每個正在執行的作業）
- 僅橋：`ui_inspect`、`ui_screenshot`、`ui_pointer`、`ui_menu_invoke`、`ui_set`、`control_call`

克勞德程式碼（儲存庫根目錄中的`.mcp.json`或`claude mcp add`）：

```json
{
  "mcpServers": {
    "photocraft": {
      "command": "/path/to/photocraft/target/release/photocraft-cli",
      "args": ["mcp", "--automation-read-root", "/absolute/path/to/trusted/workspace", "--automation-write-root", "/absolute/path/to/trusted/workspace"]
    },
    "photocraft-live": {
      "command": "/path/to/photocraft/target/release/photocraft-cli",
      "args": ["mcp", "--bridge", "127.0.0.1:7878", "--control-token-file", "/private/path/photocraft-control.token"]
    }
  }
}
```

```sh
cargo build --release -p photocraft-cli
mkdir -p "$PWD/photocraft-work"
claude mcp add photocraft -- "$PWD/target/release/photocraft-cli" mcp \
  --automation-read-root "$PWD/photocraft-work" \
  --automation-write-root "$PWD/photocraft-work"
```

`doc_inspect`（和引擎命令`document.inspect`）報告帶有種類的圖層樹，
邊界、遮罩、選擇、效果 (`effects.items[].kind`)、智慧型濾鏡 (`smartFilters[]`)、智能
物件來源（`smartSource`：`embedded`及其檔名，或`linked`及其檔案路徑，或對於
PSD放置圖層，`Idnt`uuid及其重複項共享），共享內容（`smartContents`：`id`
每份相同內容都有，且數量為`instances`；替換內容改變它們
全部並在結果中列出它們），輸入
文字、調整設定、色版和歷史記錄，以便代理可以驗證他們所做的事情，而無需
截圖。`crates/automation/tests/agent_tasks.rs`為參考：十個現實編輯任務
（標題卡、顏色等級、撤銷/重做、可編輯智慧模糊、遮罩、已儲存的選擇、對齊、
功能範圍的匯出、調整大小/裁切、CMYK + 本機儲存）完全透過 MCP 驅動。

如果沒有 MCP，`photocraft-cliserve [--port N]`會保持無頭會話開啟並回應 JSON 行
（參見`docs/control-protocol.md#headless-server`）。

典型的代理循環（將輸入放置在設定的讀取根下，將輸出放置在寫入根下）：

1.`doc_open {path}`
2.`command_list {filter:"blur"}`
3.`command_run {id:"filter.blur.gaussianBlur", params:{radius:4}}`
4.`doc_render_preview`查看結果
5.`doc_save {path:"out.pcraft"}`

## 色彩管理

`crates/cms`是我們自己的純 Rust ICC 引擎（v2/v4 解析、矩陣/TRC 和 LUT 設定文件，所有四個
意圖，黑點補償）。它提供 CC0 內建設定文件，包括合成的
“PhotoCraft塗層 CMYK”，因為 Adob​​e 的 CMYK 設定檔是專有的（請參閱`crates/cms/README.md`）。

- 文件隨附可選的嵌入式 ICC 設定檔案 (`Document::icc_profile`)；`edit.assignProfile`
`edit.convertToProfile`改變它。模式變化（`image.mode.*`）透過cms轉換。
- **校樣顏色** (⌘Y)、**校樣設定** 和 **色域警告** (⇧⌘Y) 烘焙 3D LUT
  (`cms::Lut3d`) 畫布著色器適用；文件像素永遠不會改變。
- 使用`photocraft_cms::transform::cached(src, dst, opts)`轉換顏色：轉換被快取
  處理程序範圍和整數緩衝區使用預先計算的表或裝置連結。

## 選單奇偶校驗

`cargo xtask parity`將Photoshop的選單樹 (`crates/ui-egui/src/menu_catalog.rs`) 與
即時命令註冊表 (`menus::is_live`) 並重寫 [`docs/parity-checklist.md`](parity-checklist.md)。測試
如果即時計數低於`parity::FLOOR`，`parity::tests::parity_does_not_regress`將會失敗。

`cargo xtask i18n-coverage`列印中每種語言的 UI 翻譯覆蓋範圍
`crates/ui-egui/src/i18n/mod.rs`註冊表，採用穩定的語言程式碼順序。它的英文鍵集是
源自 UI`tl!`文字、選單目錄和 UI 命令表以及選單命令標籤
發動機源。驗證目錄行是否存在格式錯誤和重複的按鍵；未使用的遺產
翻譯不影響分母。未註冊的語言環境 TSV 檔案將被忽略。

## 測試策略

- **每個板條箱中的單元和屬性測試**。 proptest 用於tile COW、區域和編解碼器。
- **格式化板條箱：**
  - 合成發電機（`psd::testgen`）
  - 位元組精確的往返
  - 格式錯誤的輸入掃描（在每個偏移處截斷）
  - 模糊目標（`crates/*/fuzz`）
-`corpus/`中的 **真實文件語料庫**（gitignored，在固定提交時獲取，sha256 驗證）：透過`corpus`貨物功能在本地選擇加入，始終在 CI 中執行。參見[測試語料庫](#test-corpora)。
- **複合預言機：** PSD 的嵌入合併影像與我們的合成器的輸出進行比較。通過率在路線圖中進行追蹤。
- **UI：** 小部件和狀態的單元測試，以及通過控制色版的螢幕截圖檢查。

## 效能說明

- 畫布由自訂 WGSL 著色器 (`ui-egui/src/gpu_canvas.rs`) 呈現：mip 映射/最近採樣、程式棋盤、像素網格、平鋪。筆觸僅上傳其傷害矩形。
- **GPU 上的畫布合成**（`photocraft-gpu`，由`gpu_canvas.rs`驅動），包括圖層效果。規劃器無法表達的內容返回`Unsupported`，畫布回退到CPU合成器（`photocraft-compose`，也是導出和測試的參考）：多色版文件，以及超出紋理限制的文件或效果區域。 GPU 無法自行匯出的部分會在 CPU 上進行點陣化並快取（`compose::masks`用於向量遮罩，`compose::shape_split`用於具有剪切層的筆畫形狀、效果距離場）。互動路徑時序：`cargo run --release -pphotocraft-ui-egui--example interactive_bench`；效果：`--example fx_bench`（`--compare files…`用於 GPU 與 CPU）；更改鑿子斜角尺寸的成本（拖曳滑桿、GPU 和 CPU、橢圓和文字）：`--example bevel_bench`；拖曳滑桿、GPU 和 CPU、橢圓和文字）：`--example bevel_bench`；拖曳提供其峰值記憶體）：`--example large_image_bench -- --size 14000x14000 --op psd`。應用程式請求適配器本身的紋理限制（egui的預設值為 8192 px；請參閱`gpu_canvas::use_adapter_limits`）；除此之外，CPU 回退合成並按帶 (`compose::render_bands`) 上傳，並且還導出、縮圖和展平流帶，因此不會保存全尺寸略合成。渲染保真度：`cargo run --release -pphotocraft-io--example oracle_diff -- corpus/psd`（以秒為單位的整個 PSD 預言表）。`ui.inspect`→`perf.timings.gpuFallback`命名原因（GPU 路徑上的`null`）。
- **GPU 上的圖層效果**（`gpu/src/fx.rs`，`gpu/src/compose.wgsl`中的核心）。每個啟用的效果都成為圖層效果區域上的「地圖程式」（移位、膨脹、高斯模糊、發光漸層、斜角高度和陰影、輪廓、筆畫帶），逐步鏡像`compose::effects`；然後，分塊的複合材料會透過貼圖進行繪製，剪裁到該區域，並將結果複製回背景中，因此一個小的圖層僅花費其小的像素。圖層的形狀 (`compose::layer_shape`) 及其距離場（`compose::effects::distance_field`：一種打破並行 GPU 色版平局的變換，無法逐位再現）來自 CPU 上的組合：並行計算的帶，除了精確發光和輪廓分明的斜角的歐裡得場（與斜幾角大小的光環一樣高的光環），這將在一個空間上執行器的空間。每個圖層狀態都會快取所有內容：不相關的編輯或效果的顏色或不透明度不會重建任何內容；筆刷重新計算效果範圍內所觸及的 256² 區塊；更改一個效果的幾何形狀只會重建該效果。快取預算`gpu::FX_BUDGET`（1.5 GB，先驅逐最近最少繪製的層）。`PHOTOCRAFT_FX_TRACE=1`列印每次重建的 CPU 時間。
- 畫布透過其周圍圖層的效果範圍 (`canvas::effect_reach`) 來增加筆畫的傷害矩形，因此效果也超出了 DAB 刷新範圍（在兩條路徑上）。
- **數字**（7360 × 4912，8 個文字圖層 + 一個陰影 + 筆畫 + 斜角的繪圖層，色相/飽和度位於頂部；M4 Pro；`cargo run --release -pphotocraft-ui-egui--example fx_bench`），CPU 回退 → GPU：帶有溫暖效果圖的完全刷新 5.7 秒 → 47 毫秒； 32 → 1.7 ms；在效果圖層上刷DAB（其貼圖圍繞DAB重建）659 → 3.7 ms；第一次刷新（所有地圖已建置）6.4 秒 → 0.39 秒。透過並行構建，CPU 超額認購約 14 倍（最少 7 次執行）：8.1 s → 0.33 s、9.9 s → 0.28 s、28 → 2.2 ms、1.7 s → 89 ms；移動文本層 7 px 36 → 89 ms；移動文本層 7 px 3 ms。`fx_bench --compare corpus/psd/…/*.psd`報告真實檔案上 GPU 與 CPU 的差異（31 個語料庫檔案中的 30 個在 GPU 上渲染效果，最差為 0.12/255）。
- 在 CPU 路徑上，`compose::effect_maps`快取每個圖層狀態的陰影、發光、斜面和緞面貼圖（LRU，768 MB 預算）。
- 大型文件的即時調整預覽使用下採樣代理程式 (`ui-egui/src/proxy.rs`)。
- **內容感知工具**（`algo::inpaint`、`algo::content_aware`：填滿、污點修復、修補、內容感知移動）。`cargo run --release -pphotocraft-engine--example content_aware_bench -- bench caf|heal|cancel`在沒有 UI 的情況下對 P21、P22 和 P49 行進行乘法，每行一個樣本，因此可以依序執行和合併兩個建置。`-- hash`以 8 位元、16 位元和 32 位元以及灰色、CMYK 和 Lab 格式列印每個工具輸出的雜湊值：填入適合Photoshop，因此加速必須使兩個建置的清單相同。解算器僅在按補片半徑增長的孔邊界上工作（沒有其他變化或匹配）。當條帶少於執行緒時，PatchMatch 在波前掃描其條帶；每個像元都看到與柵格順序相同的鄰居和隨機數，因此結果不相依性執行緒數。每行都會檢查取消：24 MP 填滿會在幾十毫秒內停止，其中大部分時間都花在釋放緩衝區上。
-`ui.inspect`返回`perf`計時（每幀 UI 毫秒、複合毫秒、上傳毫秒）。
- 切勿每格掃描整個表面。快取每個文件修訂版 (`PhotocraftApp::cached_bounds`)。 36 MP 層上的未快取`content_bounds()`曾經每幀花費 77 毫秒。

## 記分卡與績效預算

[`docs/scorecard.md`](scorecard.md) 以數字表示PhotoCraft代表每個區域。它是
生成；切勿手動編輯它。

```sh
cargo xtask scorecard            # regenerate docs/scorecard.md
cargo xtask scorecard --check    # what CI runs: fails if the committed file is stale
cargo xtask perf                 # full benchmark run (tens of minutes, release build)
cargo xtask perf --quick         # small synthetic documents (about a minute once built)
cargo xtask perf --update-baseline   # also write perf/baseline.json from this run
```

**來源。 ** 記分卡僅讀取已提交的文件，因此它是確定性的：
-`scorecard/*.toml`：每個區域一份清單（工具、檔案相容性、UI、類型、自動化、
  可靠性、分佈、未解決的錯誤）。每件商品都有一個`id`、一個短`target`、一個`status`
  （`done`、`partial`或`missing`）、`issue`和`note`以及證據（文件：行）。
`main`上的狀態必須為真：根據程式碼進行驗證，並在以下情況下註明`partial`並附上註釋
  不確定。翻轉 PR 中更改它的行。
-`perf/budgets.toml`：執行`xtask perf`的工作台和場景 P1…Pn，每個場景映射到一個
  長凳行，有預算（目標來自#209、#210、#211）。`enforce = false`標記目標
  程式碼尚未滿足（報告超出預算；只有回歸失敗）；將其翻轉到`true`
  符合它的公關。程式碼尚不支援`not_measurable`的場景和原因，永遠不會
  一個數字。
-`perf/baseline.json`：記分卡顯示的數字，僅由
`cargo xtask perf --update-baseline`，隨機（CPU、RAM、GPU轉接器、作業系統）、其機
  類、提交、日期和平均負載。
-`crates/io/tests/corpus.rs`：語料庫層（每個`Source { .. }`常數）。
- 首選項審核：沒有程式碼讀取的`Preferences`欄位（目標 0，#204），加上計數
  從樹（永不崩潰屬性覆蓋，`docs/parity-checklist.md`）。

**效能執行。 **`xtask perf`在版本中建置工作台（`perf_scenarios`、`interactive_bench`、
`fx_bench`、`bevel_bench`、`type_bench`、`large_image_bench`和`layout_bench`（一旦存在），執行每個
`--json`，並將報告合併到由場景 id 鍵控的`target/perf/results.json`中，使用 p50，
p95 和最大值（樣本上最接近的排名），過程中測量的峰值 RSS（`photocraft-testkit`的
`perf::RssSampler`），畫布所持有的GPU位元組數，以及每個之前和之後的負載平均值
長凳。`target/perf/summary.md`是 Markdown 表。當執行預算時，它退出非零
當場景的 p50 回歸超過 15% 時中斷（`--threshold`或`regression_pct`）
對照來自相同機器類別和模式的基線，或當工作台出現故障時。執行自
不同的機器類別從不進行比較，`--quick`編號僅與
快速基線。運作其他工作的機器會給出吵雜的數字：檢查
在相信回歸之前先進行摘要，並將其記錄在您引用的任何數字旁邊。

**新增場景。 ** 在長凳上添加一行（保持其名稱穩定：這是關鍵），給出
bench`--json`透過`photocraft_testkit::perf::{row, report, write_report}`支援，然後添加一個
`[[scenario]]`到`perf/budgets.toml`並執行`cargo xtask scorecard`。

**CI.**`ci.yml`執行`cargo xtask scorecard --check`。`perf-nightly.yml`執行`cargo xtask perf`
每天晚上（按需）在固定的 macOS 執行程式上，將表格發佈為工作摘要，然後上傳
`results.json`，並因預算破裂或回歸而失敗。它永遠不會在拉取請求上執行：
僅長凳的發佈建置就比 PR CI 應該花費的時間更長。給夜跑者
它自己的基線，使用`update_baseline`手動執行工作流程並提交`perf-baseline`
工件為`perf/baseline.json`。

## 網頁構建

`apps/photocraft-web`透過eframe的 web runner 在瀏覽器中執行相同的`PhotocraftApp`。渲染器為wgpu:WebGPU（瀏覽器有此渲染器），否則為 WebGL2。這只是 Rust。唯一的 JavaScript 是 wasm-bindgen 產生的黏合劑。

```sh
brew install trunk                 # or: cargo install trunk --locked
cd apps/photocraft-web
trunk build --release              # writes ../../dist/web (index.html, .js glue, .wasm)
trunk serve --release              # dev server on http://127.0.0.1:8765
```

任何靜態檔案伺服器都適用於`dist/web`，例如`python3 -m http.server 8765`在該目錄內運作。 Trunk 自行下載相符的`wasm-bindgen`和`wasm-opt`。`trunk build --release`使用`wasm-release`Cargo 設定檔（`index.html`中的`data-cargo-profile`：胖 LTO，用於 UI、引擎、I/O 和 Web shell 以及熱路徑之外的大型 crate 的選擇級別「z」（像素模型、解析器、J3）用於翻譯器、Jage） crate，保持級別3）。 UI 覆蓋範圍可減少瀏覽器程式碼大小，同時保留過濾器和合成的面向速度的設定。為`.wasm`提供壓縮服務；它的原始大小隨來源和編譯器的不同而變化。一些靜態主機拒絕這麼大的檔案（Cloudflare 的每個檔案上限為 25 MiB），因此發布網站不會將其作為一個檔案儲存：`packaging/web/package.sh`對它進行 gzip 壓縮，在`app/`下將其切成最多 20 MiB 的部分，並發送`app/`的頁面，這是一個獲取各個部分的小模組，讓瀏覽器將其膨脹（或膨脹它們）本身瀏覽器不能）並將流傳遞給應用程式自己的載入器（問題＃5302）。如果網站中的任何檔案超過 24 MiB，則指令碼將失敗； wasm 本身的大小不再受到限制，但它仍然是每個訪客下載的內容，所以請觀看。 Pull-request CI 也使用與發布作業相同的固定 Trunk 版本來建立實際的瀏覽器應用程序，並執行`packaging/web/package.sh`，包括相對 URL 和檔案大小檢查。`web build and wasm size gate`檢查在 main 和 PR 上執行，這些 PR 會更改 Web 建置編譯或打包的內容（`apps/photocraft-web`、`apps/photocraft-web-boot`、`packaging/web`、包源和清單、XZZPH488ZX/`packaging/web`、包源和清單、XZZPH488ZX/XZZ、389的大小和操作作業中站點的最大文件總結。網路建置從不嵌入工藝字體（請參閱上面的字體）。若要查看位元組的去向，請在`target/wasm32-unknown-unknown/wasm-release/photocraft-web.wasm`上執行`twiggy top -n 40`（在 wasm-opt 刪除名稱之前）。

URL 標誌：`?webgl`強制使用 WebGL2 後端，`?cpu`強制使用 CPU 畫布路徑。

要完全在 Docker 中建置和提供 Web 應用程序，請從儲存庫根執行：

```sh
docker build --load -t photocraft-web:local .
docker run --rm -p 8080:8080 photocraft-web:local
```

開啟http://localhost:8080/。請參閱 [Docker 託管](../packaging/web/README.md#docker)
HTTPS/反向代理部署和瀏覽器限制。

Web shell (`apps/photocraft-web/src/web.rs`) 與桌面有何不同：

- **檔案對話方塊**使用`rfd::AsyncFileDialog`。選定的文件內容透過`Services::file_dialog`非同步到達；這包括「檔案」>「開啟」和單一檔案命令，例如「嵌入」。
- **儲存/另存為/匯出**觸發瀏覽器下載編碼位元組。 shell 使用 Blob、物件 URL 和臨時`<a download>`來完成此操作，所有這些都由 Rust 建立。沒有儲存對話方塊，因此建議的名稱將成為下載名稱。
- **拖曳：**`WebShell`在應用程式看到框架的`dropped_files`之前取得它們。它使用`DroppedFile::bytes_async`讀取每個文件，並將位元組推送到`Services::inbox`中。
- **啟動失敗：**`index.html`顯示“正在載入PhotoCraft...”，直到應用程式將其刪除，或在 Web 執行程式無法啟動時將其替換為錯誤。如果`.wasm`從未下載或與頁面的`.js`（部署後陳舊的快取`index.html`）不匹配，則根本不會執行 Rust，因此頁面本身（僅 CSS，無指令碼）會在 20 秒後添加帶有重新加載連結的提示。
- **無控制伺服器：**瀏覽器無法偵聽 TCP。若要自動化 Web 構建，請使用`--remote-debugging-port`驅動無頭 Chrome。`Page.setInterceptFileChooserDialog`和`DOM.setFileInputFiles`覆蓋 Open，`Input.dispatchDragEvent`和`files`覆蓋 drop，`Browser.setDownloadBehavior`捕獲下載。
- macOS (`--headless=new --enable-unsafe-webgpu`) 上的 Headless Chrome 獲得真正的WebGPU轉接器。


## 索引色彩計時

`cargo run --release -pphotocraft-engine--example indexed_color_perf -- image.png`措施
調色板構造、Floyd–Steinberg 擴散和完整的全解析度 CPU 預覽
路徑（代理副本、引擎命令、結果組合；不包括 GPU 上傳/演示）。
它列印 8、16、32、64、128 和 256 種顏色的 JSON 行，其中包括 1 次預熱和 3 次測量
按計數執行，加上調色板和像素雜湊以進行精確的前後比較。輸入檔
留在當地；不要透過基準報告發布個人圖像、調色板或元資料。

合成 24 MP 查找比較可透過以下方式重現
`cargo test --release -pphotocraft-algoindexed_diffusion_24mp_release_comparison -- --ignored --nocapture`.
它交替線性/加速查找順序並檢查相同的索引和像素。
對於 2026 年 10 月 9 日本地計時，兩個二進位檔案均使用`CARGO_PROFILE_RELEASE_LTO=false`和
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`；比較使用相同設定建置的二進位。

在本機桌面會話中，索引顏色即時預覽會在一名工作人員上進行計算和展平。
拖曳時最後的預覽仍然可見；中間設定被合併，並且僅
上傳與目前文件修訂、活動層、對話方塊和參數相符的結果。
GPU 上傳/顯示轉換仍然在 UI 執行緒上執行。網路和`PHOTOCRAFT_INLINE_JOBS=1`
會話保留同步預覽。關閉對話方塊會使其結果無效；已經
執行計算完成，無需編輯會話。

## 螢幕外 UI 快照（無視窗）

使用真實的wgpu畫布無頭渲染完整的 UI，例如用於設計評審或應用程式時
視窗將被遮蔽（macOS 不會渲染被遮蔽的窗口，因此即時`ui.screenshot`必須引發
視窗並竊取焦點）：

```sh
cargo run --release -p photocraft-ui-egui --example snapshot -- \
    --out ui.png --size 1440x900 --scale 2 --open photo.jpg \
    --script '[["ui.set", {"tool": "type"}], ["ui.menu.invoke", {"id": "image.canvasSize"}]]'
```

`--script`是依序應用的控制協定呼叫 (`[method, params]`) 的清單。
輸入呼叫（`ui.key`、`ui.type`、`ui.click`）現在僅在應用程式處理事件後回复，
因此下面的`ui.inspect`觀察了它們的效果。


##Photoshop操作 (.atn)

`photocraft-psd`讀取和寫入Photoshop操作檔（`atn`，版本 16：Photoshop6 和
稍後）；`photocraft-engine`的`atn_cmds`將其事件對應到指令（`actions.load`、
`actions.save`；動作面板選單的載入/儲存動作…； `photocraft-cli批次 --actions
設定.atn`）。查看集合包含的內容（事件 ID、面板名稱、參數鍵Photoshop
記錄）向表中新增事件時：

```sh
cargo run -p photocraft-psd --example atn_dump -- "Default Actions.atn"
```

表外的事件會作為保留描述符的`actions.photoshopStep`步驟加載
逐字記錄（因此「儲存操作」會將其寫回），並且在播放時失敗（步驟名稱為Photoshop）。

## 刪除工具質量

`cargo run --release -pphotocraft-engine--example remove_quality -- <dir> [out dir]`填滿
`photocraft_algo::remove`的五張照片中出現孔洞（背景上的圓盤和細線）和
列印每個孔相對於原始`texture`的 PSNR（填滿的平均梯度
原始的：低於 1，它比它替換的更平滑）和`seam`（沿帶的相同）
孔的邊緣：在 1 上方顯示邊緣）。有了out dir，它就會寫下original |孔|填滿農作物。的
照片屬於公共領域，未經承諾；將它們以 1920 像素寬下載到`<dir>`中，如下所示：

|文件|照片|
|---|---|
|`01.jpg`| [提頓山脈與蛇河](https://commons.wikimedia.org/wiki/File:Adams_The_Tetons_and_the_Snake_River.jpg)，安塞爾亞當斯，1942 年（美國國家檔案館）|
|`02.jpg`| [Ansel Adams，國家檔案館 79-AAB-01](https://commons.wikimedia.org/wiki/File:Ansel_Adams_-_National_Archives_79-AAB-01.jpg)（美國國家檔案館）|
|`03.jpg`| [在 Crane Meadows 國家野生動物保護區健行](https://commons.wikimedia.org/wiki/File:Hiking_at_Crane_Meadows_National_Wildlife_Refuge_(49018614166).jpg) (USFWS) |
|`04.jpg`| [草甸海葵，安地斯湖國家野生動物保護區](https://commons.wikimedia.org/wiki/File:Meadow_Anemone_Lake_Andes_National_Wildlife_Refuge_South_Dakota_(53923253226).jpg) (USFWS) |
|`06.jpg`| [Line5070，NOAA 照片庫](https://commons.wikimedia.org/wiki/File:Line5070_-_Flickr_-_NOAA_Photo_Library.jpg) (NOAA) |

`https://commons.wikimedia.org/wiki/Special:FilePath/<file name>?width=1600`服務於每個
放置箱子的尺寸（Commons 將其四捨五入到 1920 年）。透過 24 MP 文件計時
指令：`cargo run --release -pphotocraft-engine--example remove_bench`。

## 測試語料庫

真實的文件是我們最好的預言機，但它們是大型二進位文件，所以它們永遠不會進入這個
儲存庫。它們位於`corpus/`（gitignored）中，並在 **固定提交** 處獲取並檢查
反對承諾的 **sha256 清單**。所有引腳都位於一處：
[`xtask/src/corpus_pins.rs`](../xtask/src/corpus_pins.rs).

|目錄 |什麼 |來源 |清單 |
|---|---|---|---|
|`corpus/photoshop/`|我們使用Photoshop創作的 256 個 PSD：智慧型濾鏡、圖層樣式效果形狀、文字引擎、每種模式和深度的調整 | https://github.com/storytold/photocraft-corpus（我們的調整，MIT 或 Apache-2.0）|
|`corpus/psd/`| 170 個小psd-tools和ag-psd文件，大多數 PSD 測試使用的組合 |psd-tools和ag-psd上游 (MIT) | XZPH581ZZX |
|`corpus/psd-tools/`|完整的psd-tools測試集（309 個檔案）|psd-tools上游 (MIT) |`xtask/psd-tools-corpus.sha256`|
|`corpus/heif/`|用於`heif`功能的 9 個小型 HEIC/HEIF 檔案（棋盤、RGB 條帶、帶有 EXIF/XMP 的網格平鋪照片，每個檔案的 Apple 解碼為`.ref.png`；帶有來源 PNG 的 10 位元 RG.和pillow-heif (BSD-3-Clause) 上游 |`xtask/heif-corpus.sha256`|
|`corpus/exr/`| 5 個深度 OpenEXR 測試影像（具有半色和 u32 ID 色版的掃描線深度資料；2.3 MB），對照其上游 sidecar 的 ID 清單進行檢查 | OpenEXR 上游 v3.5.2（BSD-3-Clause） | XZZPH593ZXXZPH |
|`corpus/affinity/`| 21 個公共Affinity文件和 #1606 的 18 個Affinity3 個範例，帶有導出的 PNG 參考 |向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和向量藝術和Affinity樣本 (CC0)、AFDesignLoad、Jac21/Branding 和pZXA 是 13953(XZplateXZ999953 (Jac21/Branding) |
|`corpus/pngsuite/`|PngSuite| schaik.com 發布檔案（公共領域）| （固定存檔）|

```sh
cargo xtask corpus                 # where each corpus lives, its pin, present or missing
cargo xtask corpus --all           # fetch everything missing or stale (cold: about 15 s; verified copies are left alone)
cargo xtask test-corpus            # fetch, then cargo test --release --features corpus (+ heif on codecs, io) on psd, codecs, io, engine
cargo xtask test-corpus -p io      # narrow to one crate (repeat -p for more)
cargo xtask test-corpus --changed  # only if psd, io, codecs, compose, gpu, text, format or affinity changed vs origin/main
cargo xtask test-corpus -- --nocapture   # pass arguments to the test binaries (per-file tables)
scripts/fetch-corpus.sh            # the same as cargo xtask corpus --all
```

**本地選擇加入，選擇加入時嚴格，始終在 CI 中。 **

- 語料庫測試位於`photocraft-psd`、`photocraft-codecs`的`corpus`貨物功能後面，
`photocraft-io`和`photocraft-engine`，因此普通的`cargo test`既不能編譯也不需要它們。
  HEIF 還需要`photocraft-codecs`/`photocraft-io`的`heif`功能（測試語料庫
  打開它）。
- 啟用此功能後，缺少語料就會失敗（「執行`cargo xtask corpus --all`」），永遠不會
  無聲跳過，每層樓強制執行。
- 如果您接觸 psd、io、編解碼器、撰寫、gpu、文字或格式，請先執行`cargo xtask test-corpus`
  你完成了。
- CI`corpus`作業在每個 PR 上執行`cargo xtask corpus --all`，然後執行`cargo xtask test-corpus`
  和主要的。`corpus/`與`actions/cache`一起快取，以`corpus_pins.rs`的雜湊為鍵，並且
  清單。只有當引腳變更時才會進行下載；否則快取會在幾秒鐘內恢復
  並在測試執行之前重新驗證（sha256）。 Fork PR 讀取基礎分支的快取。

**我們自己的語料庫：[photocraft-corpus](https://github.com/storytold/photocraft-corpus)。 **

- **創作克隆：** 一個普通的 git 克隆，`git clone git@github.com:storytold/photocraft-corpus.git`，
  放置在該結帳台旁 (`../photocraft-corpus`)。它的生成器在那裡寫入，然後你提交
  並推到那裡。
- **消費副本：**`corpus/photoshop/`此處。這是一個普通目錄：固定快照
  （作為固定提交的程式碼載入 tarball 下載，然後進行驗證）。
- **本地模式：**`cargo xtask corpus --photoshop --local`複製`photoshop/`
`../photocraft-corpus`（或`PHOTOCRAFT_CORPUS_REPO=<path>`）而不是下載。它會在什麼時候發出警告
  克隆的 HEAD 不是 pin，或其檔案與清單不同。用它來測試再生
  推送前的檔案：`cargo xtask test-corpus --local`複製它們並執行每個語料庫測試。
- **撞針：**提交並推送photocraft-corpus，然後在此處設定 PR
`PHOTOCRAFT_CORPUS_COMMIT`在`corpus_pins.rs`中，執行
`cargo xtask corpus --photoshop --update-manifest`，提交清單差異，執行
`cargo xtask test-corpus`，並提高了地板的改善。
- **不是子模組或子樹：** 子樹會將二進位檔案放回此儲存庫中
  歷史記錄和子模組會導致每個貢獻者的初始化和分離頭摩擦。一個針加一個
  清單給出了相同的再現性。
- 第三方集（psd-tools、ag-psd、PngSuite）以相同的方式從其上游獲取，
  並且永遠不會複製到photocraft-corpus中。

## 渲染保真度（PSD oracle）

`cargo xtask test-corpus -p io -- --nocapture`將我們每個語料庫 PSD 的組合與
Photoshop自己的合併影像（PASS ≤ 2/255）並檢查匯出→重新匯入渲染是否相同，
每個來源都有自己的樓層：`corpus/psd`，psd-tools集（也是截斷/損壞掃描
超過每個文件，絕對不能恐慌）和我們的Photoshop設定`corpus/photoshop`（每個功能組
總計：`smart-filters`、`effects`、`text`、`adjustments/<mode><bits>`）。智慧型物件和類型
層複合Photoshop的快取像素；`cargo xtask test-corpus -p engine -- --nocapture`
(`crates/engine/tests/photoshop_oracles.rs`) 使用我們的智慧型濾鏡堆疊和文字重新渲染它們
發動機，是兩者的故障圖。恐慌報告為`CRASH`並且執行失敗。沒有真正合併圖像的檔案（關閉“最大化相容性”）
而是根據嵌入的縮圖進行判斷（`PASS (thumbnail)`，嚴格的低分辨率
檢查）；只有當文件根本沒有預言時才會跳過。當`crates/io/tests/corpus.rs`中的地板升高時
改善；切勿降低它們。語料庫發現的合成複製品位於
`crates/io/tests/corpus_regressions.rs`（語料庫文件永遠不會提交）。要深入研究一個文件：

```sh
cargo run --release -p photocraft-io --example oracle_diff -- corpus/psd/<file>.psd 0 png /tmp/diff.png
```

寫我們的|Photoshop|並排的差異熱圖；`col`、`row`、`worst [n]`、`grid x0 y0 x1 y1 [ch]`、
`layerpx x y`和`DUMP_FX=1`列印樣本、最差像素、值網格、每層像素和原始像素
效果描述符。目前的發現：填滿層漸層由層的遮罩邊界構成；
Photoshop的漸層平滑度是 Catmull-Rom 混合，「感知」插值是 Oklab
（進口時烘烤成密集停止，`crates/io/src/gradient_bake.rs`）；形狀層的向量
筆畫繪製在其剪切圖層之上；來自圖層`fxrp`的連結效果圖案圖塊
參考點；筆畫距離遵循在子像素處播種的 5 × 5 倒角度量 (1, √2, √5)
邊緣偏移；內部效果保持圖層的 Alpha；外部筆觸融合到背景上
他們有自己的模式，上面的筆畫覆蓋著下面的筆畫。

## 系統列印

`file.print`和`file.printOneCopy`保留 PDF 佈局、縮放、標記和 ICC 轉換
PhotoCraft並透過`craft-print`在固定的 Git 修訂版處提交渲染的位元組
`Cargo.toml`。 CUPS 在 macOS/Linux 上串流 PDF； Windows 使用共用的 Windows PDF 後端。
驅動程式接收精確的實體圖面尺寸，因此它不會將PhotoCraft的佈局縮放兩次。
Windows 點陣列印無法建立實體印表機的色彩保真度；測試關鍵
使用實際印表機及其設定檔輸出照片。

桌面命令在後台工作人員上執行合成和提交。 UI 報告成功
僅當假脫機程式接受作業時；刪除系統列印佇列中提交的作業。無頭
`Session::execute`保持同步。傳回的`pdf`是僅 spool 作業的`null`，並且
`command`是診斷預覽，而不是具有 PDF 路徑的可執行指令。供應`output`
將 PDF 保留在該路徑中，並將`send`預設為 false。`dryRun`渲染 PDF 並報告
後台命令無需提交；僅當`output`明確時，它才會寫入 PDF。`send: false`
沒有輸出需要`dryRun: true`。記住的列印設定省略`output`、`dryRun`和
`send`，因此列印一份副本從自己的輸出選擇中獲得提交。共享假脫機傳輸
Windows 上的檔案會自動刪除； CUPS 不需要。
