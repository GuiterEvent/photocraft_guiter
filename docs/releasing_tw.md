# 釋放PhotoCraft

每次推送到`release`分支都會執行`.github/workflows/release.yml`。工作流程構建
適用於 macOS、Windows、Linux 和 Web 的簽章安裝程序，以及 FreeBSD tarball，然後建立
或更新名為`PhotoCraftv<version>`的 **草稿** GitHub 版本。沒有人會看到草稿，直到
維護者發布它。

這是PhotoCraft對共享的實現
[發布劇本](release-playbook.md)。面向使用者的名稱為 **PhotoCraft**。文件、二進位文件
id 保持小寫（`photocraft-<version>-<platform>-<arch>.<ext>`、`ai.storyteller.photocraft`）。

## 削減版本

1. **在`main`上升級版本**。它唯一居住的地方是`[workspace.package] version`
   在根`Cargo.toml`中：

   ```sh
   cargo xtask version                 # prints the current version, e.g. 0.1.0
   cargo xtask version set 0.2.0       # or 0.2.0-rc.1; updates Cargo.toml and Cargo.lock
   ```

透過正常審核流程提交變更 (`Cargo.toml`+`Cargo.lock`)。
2. **將`main`合併到`release`**（或快轉）並推送。工作流程自行啟動。
3. **等待草稿。 ** 大約30到45分鐘後（公證是較慢的部分），
   發布頁面有一個草稿`PhotoCraftv0.2.0`，在推送的提交上標記為`v0.2.0`，其中
   每一件神器和`SHA256SUMS.txt`。註釋是從合併的 PR 產生的。
4. **檢查一下。 ** 下載一兩個安裝程式並查看作業摘要。任意
`::warning::`意味著缺少簽名秘密並且該工件未簽名。
5. **將記分卡增量**加入到草稿註解中：移入的內容
   [`docs/scorecard.md`](scorecard.md) 自上一版本以來。比較總表
   以及兩個標籤的效能表（`git diff v<previous> v<this> -- docs/scorecard.md`）
   並列出每個區域已完成/部分/未完成的更改，新滿足其要求的場景
   預算或變慢，語料庫樓層提高，以及設定的變更無濟於事。
   引用基線機器和負載平均值的數字。
6. **在 GitHub UI 中發布**草稿。發布會創建`v0.2.0`標籤。版本帶有
   預發布後綴 (`-rc.1`) 標記為預發布。

在發布之前再次推送到`release`會重建相同的草稿並取代其資源。
草稿發布後，工作流程拒絕再次觸及該版本，因此請先進行修改。

**測試執行：** *操作→發布→執行工作流程*手動執行整個管線。可選的
`version`輸入（例如`0.2.0-rc.1`）僅針對此執行覆蓋`Cargo.toml`。職位申請
在建置之前將其與`cargo xtask version set`一起使用，因此二進位檔案也會報告它。奔跑依舊
需要`release`環境，只有`release`分支可以使用，所以選擇該分支
在對話方塊中。

## 建構了什麼

|平台|文物|建立在 |
|---|---|---|
| macOS 11+（通用：蘋果晶片 + 英特爾）|`photocraft-<v>-macos-universal.dmg`、`photocraft-cli-<v>-macos-universal.zip`|`macos-15`|
| Windows 10+ x64 |`photocraft-<v>-windows-x64.msi`、`photocraft-<v>-windows-x64-portable.zip`|`windows-latest`|
| Windows 10+ x86（32 位元）|`photocraft-<v>-windows-x86.msi`、`photocraft-<v>-windows-x86-portable.zip`|`windows-latest`|
| Linux x86_64 |`photocraft-<v>-linux-x86_64.{AppImage,AppImage.zsync,deb,rpm,tar.gz,flatpak}`|`ubuntu-22.04`（平板包裝：`ubuntu-24.04`）|
| Linux aarch64 | linux`photocraft-<v>-linux-aarch64.{AppImage,AppImage.zsync,deb,rpm,tar.gz,flatpak}`|`ubuntu-22.04-arm`（平板包裝：`ubuntu-24.04-arm`）|
| Linux riscv64（glibc 2.39+）|`photocraft-<v>-linux-riscv64.tar.gz`|在`ubuntu-24.04`（ports.ubuntu.com riscv64 函式庫）上交叉編譯；在 QEMU 下進行 CLI 冒煙測試 |
| FreeBSD 14 x86_64 | FreeBSD 14 x86_64`photocraft-<v>-freebsd-x86_64.tar.gz`|`ubuntu-latest`上的 FreeBSD 14.3 虛擬機 |
|網頁 |`photocraft-web-<v>.zip`（靜態網站；參見 [`packaging/web/README.md`](../packaging/web/README.md)）|`ubuntu-latest`|

每個二進位檔案都會報告其版本、提交和建置日期：`photocraft--version`，
`photocraft-cli--version`，和*幫助 › 關於PhotoCraft*。 CI 設定`PHOTOCRAFT_BUILD_SHA`和
`PHOTOCRAFT_BUILD_DATE`和`crates/engine/src/build_info.rs`在編譯時讀取它們。一個平原
`cargo build`未設定它們並報告`0.2.0 (dev build)`。

每個桌面建置作業（macOS、Windows、Linux、FreeBSD）也會檢查 [craft-fonts](https://github.com/storytold/craft-fonts)
在`.github/workflows/release.yml`中的`craft-fonts-ref`提交處並使用`CRAFT_FONTS_DIR`構建
`CRAFT_FONTS_REQUIRED=1`，因此桌面版本嵌入了日文字體（網頁版本不嵌入任何字體：請參閱
`docs/development.md`› 字體）並且失敗而不是在沒有它們的情況下發貨。這些軟體包帶有每種字體的許可證：
`OFL-<family>.txt`（`packaging/env.sh`中的`copy_font_licences`；Windows 上的便攜式 zip；
macOS 應用程式中的`Contents/Resources/Licenses`）。故意將銷釘與那個一起撞
在`ci.yml`。規則：`../craftrules/standards/fonts.md`；建置選項：`docs/development.md`› 字型。

### macOS

`packaging/macos/package.sh`建構`aarch64-apple-darwin`和`x86_64-apple-darwin`
`MACOSX_DEPLOYMENT_TARGET=11.0`，將它們與`lipo`連接，並組裝`PhotoCraft.app`：

-`Info.plist`是從`Info.plist.in`產生的。捆綁包 ID 為`ai.storyteller.photocraft`。
  plist 設定`LSMinimumSystemVersion`11.0、`NSHighResolutionCapable`和文件類型：
`.pcraft`（所有者），加上 PSD/PSB 和PhotoCraft讀取的圖像格式（備用，因此它永遠不會
  接管預覽的預設設定）。圖示是`assets/app-icon/photocraft.icns`。
- **簽名**透過強化的執行階段間和安全時間戳進行由內而外的處理。執行檔
  首先簽名，然後捆綁。最終簽名上沒有`--deep`。的
  權利 (`entitlements.plist`) 故意為空。
- **公證：**應用程式被壓縮並使用`xcrun notarytool submit --wait`發送，然後
  票證已釘在應用程式上。該應用程式在 DMG 上執行（`hdiutil`，帶有`Applications`鏈接
  拖到）。 DMG 也經過簽名、公證和裝訂。該指令碼檢查結果
  與`codesign --verify --strict`、`stapler validate`和`spctl -a -vvv`。
  它的Finder視窗（背景、圖標大小和位置）來自
  [`packaging/macos/dmg/`](../packaging/macos/dmg/README.md)，其磁碟區名為`PhotoCraft`
  沒有視窗背景所需的版本； DMG 檔案名稱保留版本。
- **CLI:** 通用`photocraft-cli`使用相同的開發者 ID 進行簽名，即硬化的
  執行階段和安全時間戳記（標識符`ai.storyteller.photocraft-cli`），壓縮，以及 zip
  發送至`notarytool`。僅`.app`、`.dmg`和`.pkg`可容納裝訂票，而不是裸票
  Mach-O，因此 Gatekeeper 在第一次下載時會在線上找到 CLI 的票證
  （隔離）複製執行。離線時，可以拒絕第一次執行，直到 Mac 再次上線。
- **驗證：**`packaging/macos/verify.sh`在使用者下載工件時檢查工件。它
  解壓縮 CLI zip 並要求內部二進位檔`codesign --verify --strict`
  架構、強化執行階段標誌、`Developer ID Application`權威
`APPLE_TEAM_ID`、時間戳記和`spctl --assess --type install`報告
`source=Notarized Developer ID`（與 Gatekeeper 的線上查找相同）。对于 DMG 它运行
`codesign --verify`、`stapler validate`和`spctl --type open`。發布工作流程執行它
  作為包裝後自己的步驟。對於簽名秘密，任何失誤都會使工作失敗；沒有
  它們僅檢查簽名完整性並發出警告，例如`package.sh`。之后在本地运行它
  型号：`packaging/macos/verify.sh --arch aarch64`。

在本地，如果沒有證書，指令碼會臨時簽署 (`codesign -s -`) 並跳過公證。
這足以在您自己的 Mac 上檢查捆綁包和 DMG：

```sh
packaging/macos/package.sh                    # universal; needs both rustup targets
packaging/macos/package.sh --arch aarch64     # quicker, host-only
open dist/release/photocraft-*-macos-*.dmg
```

### 視窗

`packaging/windows/package.ps1 -Arch x64|x86`使用`-C target-feature=+crt-static`建置。
靜態 C 執行階段表示 MSI 和可移植 zip 都不需要 Visual C++
可重新分發，這對於獨立安裝程序很重要，並且成本僅為約 100 KB。旗幟
進入`CARGO_TARGET_<TRIPLE>_RUSTFLAGS`，因此主機建置指令碼不受影響。

-`apps/photocraft/build.rs`和`apps/photocraft-cli/build.rs`嵌入圖標
  (`assets/app-icon/photocraft.ico`) 和 VERSIONINFO 以及`winresource`箱
  （建構相依性`0.1`）。他們僅在針對 Windows 時才這樣做。其他地方各
  即使設定了`PHOTOCRAFT_REQUIRE_WINRES`，指令碼也是無操作的，並且 Web 構建
  不碰那個板條箱。兩者都將 ProductName 設定為PhotoCraft，將 CompanyName 設為
  Learning Machines LLC 與版權線相同。圖形使用者介面描述是
  「PhotoCraft影像編輯器」（`photocraft.exe`）。 CLI 描述是
  “PhotoCraft命令列介面”，原始檔案名稱為`photocraft-cli.exe`
  和內部名稱`photocraft-cli`。版本字串是 Cargo 套件版本，
  因此保留預發布標籤；數位檔案和產品版本是
  winresource 的`major.minor.patch.0`。缺少資源編譯器會發出警告，因此
  可選的交叉編譯靜態連結。`package.ps1`套裝`PHOTOCRAFT_REQUIRE_WINRES=1`。
  然後，CLI 建置指令碼傳回一個命名為`rc.exe`或`windres`的錯誤，並執行
  不要驚慌。
- 發布版本使用 GUI 子系統，因此「開始」功能表啟動不會開啟控制台視窗。
  CLI 保留在控制台子系統 3 中，因此其輸出到達終端。
-`photocraft.wxs`(WiX v5) 是按機器安裝到 Program Files 中，並附有廣告
  開始功能表快捷方式。PhotoCraft成為`.pcraft`的預設應用程序，並列在下面
  PSD/PSB 和影像檔案的「開啟方式」。它還註冊應用程式路徑 (Win+R`photocraft`)。
  MSI 版本是數位`X.Y.Z`，因為 MSI 沒有預發布欄位。同版本
允許升級，以便候選版本可以相互替換。
- 雙擊 MSI 開啟安裝精靈（沒有許可證頁面的 WixUI_InstallDir）：
  歡迎，安裝資料夾（記住`HKLM\Software\PhotoCraft\InstallDir`中的升級），
  就緒、進度頁面和完成頁面，其中顯示在哪裡可以找到該應用程序，並勾選“啟動”
PhotoCraft」盒子。這些頁面命名為完整版本（`photocraft.en-us.wxl`）。`package.ps1`抽獎
  嚮導的橫幅和側面位圖來自應用程式圖標，因此沒有 WiX 庫存藝術作品。`msiexec /qn`
  仍然靜默安裝。建置需要 WiX UI 和 Util 擴充：
`wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`.
- 捷徑圖示標識符保留可執行檔的`.exe`副檔名：MSI 使用識別符
  作為快取的圖示文件名，無副檔名的檔名可以呈現為空白文件
  圖標。`packaging/windows/check-icons.ps1`檢查CI中的引用與擴充；
`package.ps1`在簽署之前也使用 ICE50 驗證建置的 MSI。
- **便攜式拉鍊：**`packaging/windows/portable.txt`與`photocraft.exe`一起發貨。那
  標記（或`PhotoCraft.portable`檔案）開啟便攜式模式：首選項、預設、
  恢復自動儲存和 GPU 啟動標記轉至 exe 旁的`PhotoCraftData\`
`%APPDATA%\Photocraft`。如果該資料夾不可寫，應用程式會發出警告並使用`%APPDATA%`。
  MSI 沒有標記。邏輯位於`apps/photocraft/src/app_dirs.rs`中且工作原理相同
  在 macOS 和 Linux 上。
- **簽名：**`packaging/windows/sign.ps1`對`.exe`檔案和`.msi`進行簽名
`signtool`，使用 SHA-256 和 RFC 3161 時間戳記。它使用任何存在的材料：
  1.`.pfx`檔案（`WINDOWS_CERTIFICATE`、`WINDOWS_CERTIFICATE_PASSWORD`），時間戳記為
     數位證書，或
  2. Azure 可信任簽章 (`AZURE_*`)：指令碼下載
`Microsoft.Trusted.Signing.Client`dlib 和 Microsoft 伺服器的時間戳記。

當 Windows 簽章設定發生變更時，該指令碼是唯一需要變更的位置。

Windows 本機：`dotnet tool install -g wix --version 5.0.2`，
`wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`，則
`pwsh packaging/windows/package.ps1 -Arch x64`.

### Linux

`packaging/linux/package.sh`暫存一棵 FHS 樹並從中產生每種格式。樹
包含兩個二進位檔案、`ai.storyteller.photocraft.desktop`、16 px 到 512 px 的 hicolor 圖標
加上可擴展的 SVG、AppStream 元資訊以及`.pcraft`、`.psb`和的共享 mime-info 文件
`.qoi`.

為什麼要採用這些格式：

- **AppImage** 可以在任何發行版上執行，無需安裝任何東西。的
  下載即用選項以及軟體包未涵蓋的發行版的後備。已建成
  與維護的`AppImage/appimagetool`相比，其靜態執行階段不需要 libfuse2。
  打包指令碼驗證固定的 AppImageTool 1.9.1 和 type2 執行階段 20251108 二進位。
  對於離線構建，請預先播種`$CARGO_TARGET_DIR/appimagetool-1.9.1-$ARCH.AppImage`和
`$CARGO_TARGET_DIR/type2-runtime-20251108-$ARCH`與配套的官方發布資產；
  該指令碼檢查 SHA-256 摘要，並在驗證後使快取的 AppImageTool 可執行；
  執行階段僅作為資料讀取。
`APPIMAGETOOL`覆寫必須指向相同的經過驗證的 AppImageTool 二進位文件，並且已經是
  可執行檔（打包前的`chmod +x`）。
  每個 AppImage 都嵌入更新資訊（`gh-releases-zsync|…|latest|…`，#349）並發布
  旁邊有一個`.zsync`，所以AppImageUpdate、AppImageLauncher和類似的工具可以找到
  下一個版本並僅下載已更改的區塊。
  其`AppRun`(`packaging/linux/AppRun`) 在發佈時也與桌面整合：
  Wayland 的停靠圖示來自合成器，將視窗的 app_id 解析為
  *已安裝*`.desktop`文件，僅 AppImage 永遠不會提供該文件 - 如果沒有這個文件
  AppImage 執行顯示通用 Wayland 標誌 (#593)。盡力：安裝
`~/.local/share/applications/ai.storyteller.photocraft.desktop`與`Exec`改寫為
  AppImage 的路徑（`TryExec`已刪除，二進位檔案不在主機路徑上）和
  旁邊有hicolor圖標，只有內容改變時才重寫，任何失敗都會離開
發射未受影響。`PHOTOCRAFT_NO_DESKTOP_INTEGRATION=1`選擇退出；要卸載，`rm
  〜/.local/share/applications/ai.storyteller.photocraft.desktop` and `find
  〜/.local/share/icons/hicolor -name 'ai.storyteller.photocraft*' -delete`。指令碼是
  由`packaging/linux/apprun-test.sh`在包裝棉絮工作流程中涵蓋。
- **.deb** 涵蓋 Debian、Ubuntu、Mint、Pop!_OS 和elementary。 **.rpm** 涵蓋 Fedora、RHEL
  及其克隆，以及 openSUSE。兩者都與選單、MIME 和圖示快取整合（
`postinst.sh`掛鉤）並卸載乾淨。兩者都是由
  [nfpm](https://nfpm.goreleaser.com/)：一種維護的、無相依性性的工具和一種設定
  （`nfpm.yaml`）。我們的板條箱中不需要cargo-deb和cargo-generate-rpm元數據，並且
  它可以在 Ubuntu 上建立這兩種格式。
- **.tar.gz** 適用於管理自己的`/opt`或`~/.local`的人員。
- **Flatpak 捆綁包** (`.flatpak`) 是一個安裝到 Flatpak、沙盒和
  對於那些更喜歡 Flatpak 而不是 AppImage 的人來說，可以透過安裝更新的捆綁包進行更新。
`packaging/linux/flatpak-bundle.sh`將作業的`.tar.gz`重新打包為
  免費桌面上的`packaging/linux/flatpak/ai.storyteller.photocraft.bundle.yml`26.08
  執行階段。它不進行 Rust 構建，並且`flatpak-builder`內部不需要網路，因此該捆綁包
  擁有與其他格式相同的二進位。每個架構有一個單獨的`flatpak`作業
  （`ubuntu-24.04`和`ubuntu-24.04-arm`，適用於 flatpak-builder 1.4）下載 Linux 作業的
  工件，執行指令碼，安裝捆綁包並在
  沙箱作為冒煙測試。該捆綁包將 Flathub 命名為其執行階段存儲庫，因此用戶安裝
它與：

  ```sh
  flatpak install --user photocraft-<v>-linux-x86_64.flatpak   # pulls org.freedesktop.Platform//26.08 from Flathub if missing
  flatpak run ai.storyteller.photocraft
  ```

沙盒權限（在清單中合理）：具有 X11 回退功能的 Wayland、IPC (X11
  共享記憶體）、用於 GPU 的`dri`以及對圖片和文件的讀取/寫入存取。每個
  其他文件透過文件選擇器和文件入口網站。沒有網路，所以
`--control`伺服器只能從沙箱內部訪問，直到用戶執行
`flatpak override --user --share=network ai.storyteller.photocraft`。主機字體讀取自
`/run/host/fonts`和`/run/host/user-fonts`。本地（在 Linux 上，使用`flatpak`和
`flatpak-builder`):`packaging/linux/package.sh --formats tar && packaging/linux/flatpak-bundle.sh`.
- **Flathub**：`packaging/linux/flatpak/ai.storyteller.photocraft.yml`從原始碼建置並
  已準備好提交 Flathub（它與捆綁包保持相同的執行階段間和`finish-args`
  清單，由 Packaging-lint 檢查）。 CI 不建置它。真正的建造需要供應的板條箱
  源（來自`flatpak-cargo-generator.py`的`cargo-sources.json`），每個增加 20 分鐘以上
  架構。使用清單標題中的命令手動建立它。

所有 Linux 二進位檔案均基於 Ubuntu 22.04 構建，並且需要 **glibc ≥ 2.35**：Ubuntu 22.04+，
Debian 12+、Fedora 36+、RHEL 10、openSUSE Tumbleweed。它們僅連結 glibc 和 libgcc_s。 X11,
Wayland、xkbcommon、Vulkan和 EGL 在執行階段從系統加載，這也是
GPU驅動程式必須來自。這就是 .deb 和 .rpm 將它們聲明為相依性的原因
（完整清單以及每個清單的原因位於`packaging/linux/nfpm.yaml`中）和 AppImage
不捆綁它們。 Flatpak 從 freedesktop 執行階段獲取它們。因為AppImage
並且 tarball 無法聲明相依性，`photocraft`檢查其會話中的庫
（X11 或 Wayland）需要在開啟視窗（`apps/photocraft/src/linux_libs.rs`）之前，並且，如果
缺少一個，列印要安裝的套件並以狀態 1 退出而不是崩潰。
`PHOTOCRAFT_SKIP_LIB_CHECK=1`跳過檢查。

本地（在 Linux 上）：安裝 [nfpm](https://nfpm.goreleaser.com/install/)，然後
`packaging/linux/package.sh`（或`--formats "deb tar"`）。

### FreeBSD

GitHub 沒有 FreeBSD 執行程序，因此`freebsd`作業在
FreeBSD 14.3 VM（`vmactions/freebsd-vm`，由提交固定），具有與
FreeBSD CI 工作流程 (`.github/workflows/freebsd.yml`) 加上`bash`。該指令碼構建了兩者
具有`CARGO_BUILD_JOBS=4`的二進位（更多執行記憶體不足的 12 GB VM）並寫入
`photocraft-<v>-freebsd-x86_64.tar.gz`。 FreeBSD 的`uname -m`表示`amd64`；檔案名稱使用
`x86_64`和其他神器一樣。 tarball 是`/usr/local`風格的樹：`bin/photocraft`，
`bin/photocraft-cli`和`share/`下具有相同的桌面條目、MIME 類型、AppStream 元訊息
和 hicolor 圖示為 Linux，以及授權、`NOTICE`和`ATTRIBUTION.md`
`share/doc/photocraft/`。用戶安裝它：

```sh
pkg install libxkbcommon wayland libX11 libXcursor libXrandr libXi libxcb mesa-libs vulkan-loader gtk3 fontconfig freetype2 alsa-lib
tar -xzf photocraft-<v>-freebsd-x86_64.tar.gz --strip-components 1 -C /usr/local
```

該作業不進行任何簽名，因此它在`release`環境之外執行並且不會獲取任何秘密。的
版本、建置日期和提交通過操作的`envs:`到達虛擬機器（按名稱，從不
模板化到指令碼中），並且構建發生在`/var/tmp`中，因此只有`dist/`被複製回來
脫離虛擬機器。二進位檔案未簽章：FreeBSD 沒有鬆散的程式碼簽章方案
二進位文件，因此請根據`SHA256SUMS.txt`檢查 tarball。

FreeBSD CI 工作流程在建置和測試後執行相同的指令碼（在推送到`main`時，以及
在觸及`packaging/freebsd/`、`freebsd.yml`或`release.yml`的 PR 上），因此發布作業
不是它運作的第一個地方。在本地，在任何作業系統上，`packaging/freebsd/package.sh --dry-run`從存根二進位檔案暫存樹
並列出 tarball，它可以在沒有 FreeBSD 機器的情況下檢查佈局。

### 網路

`packaging/web/package.sh`為應用程式執行`trunk build --release`（`apps/photocraft-web`，進入
`dist/web`）以及啟動它的模組（`apps/photocraft-web-boot`，進入`dist/web-boot`）。
它對應用程式的 wasm 進行 gzip 壓縮，將其切成最多 20 MiB 的部分，並使用以下命令壓縮啟動模組的頁面
零件、範例`_headers`和`.htaccess`檔案以及託管指南。本站僅使用
相對 URL，因此它可以在任何路徑下和 iframe 中工作。 wasm 建於
尺寸最佳化的`wasm-release`Cargo 型材（在`apps/photocraft-web/index.html`中設定），以及
如果網站中的任何檔案超過 24 MiB（低於 Cloudflare 的每個檔案 25 MiB 限制），指令碼將失敗；
應用程式的 wasm 本身可能更大，因為它作為部件提供。
[`packaging/web/README.md`](../packaging/web/README.md) 涵蓋 MIME 類型、壓縮、
快取、iframe 片段和`?webgl`/`?cpu`標誌。

## 秘密

所有秘密都位於儲存庫的**`release`環境**中（*設定→環境→
釋放*）。將其部署分支限制為`release`。`release.yml`中的每項工作
簽署或發布聲明`environment: release`（Flatpak 和 FreeBSD 作業未簽署任何內容
並且不），因此只有推送到該分支，或在其上手動執行，才能讀取秘密。每個秘密都是可選的。如果缺少一個，則該平台的工件未簽名，並且
執行顯示`::warning::`。

|秘密|用於 |
|---|---|
|`APPLE_CERTIFICATE`|帶有「開發者 ID 應用程式：Learning Machines LLC (DJ6XS33FX8)」的 base64`.p12`|
|`APPLE_CERTIFICATE_PASSWORD`|`.p12`的密碼 |
|`KEYCHAIN_PASSWORD`|臨時 CI 鑰匙圈的密碼（若未設定則為隨機）|
|`APPLE_ID`|`notarytool`使用的 Apple ID |
|`APPLE_PASSWORD`|該 Apple ID 的應用程式專用密碼 |
|`APPLE_TEAM_ID`|`DJ6XS33FX8`|
|`WINDOWS_CERTIFICATE`| base64`.pfx`程式碼簽署憑證 |
|`WINDOWS_CERTIFICATE_PASSWORD`|`.pfx`的密碼 |
|`AZURE_TENANT_ID`、`AZURE_CLIENT_ID`、`AZURE_CLIENT_SECRET`| Azure 可信任簽署的服務主體（`.pfx`的替代方案）|
|`AZURE_SIGNING_ENDPOINT`|例如`https://eus.codesigning.azure.net`|
|`AZURE_SIGNING_ACCOUNT`、`AZURE_CERT_PROFILE`|受信任的簽章帳號與憑證設定檔名稱 |

`GITHUB_TOKEN`建立版本。只有最終作業才能獲得`contents: write`。

在 macOS 上，`packaging/macos/import-cert.sh`將`.p12`解碼為臨時鑰匙串
（`security create-keychain`，然後`security import`，然後`set-key-partition-list`所以共同設計
可以在沒有提示的情況下使用該密鑰）。它將身份的 SHA-1 匯出為`MACOS_SIGN_IDENTITY`。
鑰匙圈在作業結束時被刪除。

## 圖示

`assets/app-icon/photocraft.svg`是規範的圖示：擁有者的ArtCraft繪圖
九尾 kitsune，向量化（MIT 或 Apache-2.0，請參閱`LICENSE.txt`；調色板和
`README.md`中的幾何形狀）。`packaging/icons.sh`重新產生 1024 px PNG、`.icns`、`.ico`（由
`cargo xtask ico`）和其中的 hicolor PNG。它需要`resvg`，以及 macOS 上的`iconutil`。
輸出是承諾的，因此包裝永遠不需要這些工具。

## 檢查

`.github/workflows/packaging-lint.yml`對`packaging/`的任何更改都會在幾秒鐘內執行，
工作流程或圖示。它執行 actionlint、shellcheck、PowerShell 解析、xmllint、
`desktop-file-validate`、`appstreamcli validate`和 YAML 檢查兩個 Flatpak
清單就執行階段和權限達成協議。
