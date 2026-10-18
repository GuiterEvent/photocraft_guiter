# 為網站寄存PhotoCraft

`photocraft-web-<version>.zip`（來自 GitHub 版本，或`packaging/web/package.sh`）擁有
`photocraft-web-<version>/`中的靜態站點：

|文件|它是什麼 |
|---|---|
|`index.html`|頁。它透過相對 URL 載入所有內容。 |
|`photocraft-web-boot-<hash>.js`、`photocraft-web-boot-<hash>_bg.wasm`|一個小模組（約 120 KB），使用 wasm-bindgen 膠水從其各個部分啟動應用程式 |
|`app/photocraft-web-<hash>.part-aa`、`-ab`、… |應用程式的WebAssembly，經過 gzip 壓縮並切成最多 20 MiB 的部分（請參閱[大小](#sizes)）|
|`app/parts.txt`|零件，依序 |
|`app/photocraft-web.js`|應用程式的 wasm-bindgen 膠水（生成，ES 模組） |
|`_headers`、`.htaccess`| Netlify/Cloudflare Pages 和 Apache 的範例標頭規則 |
|`fonts/`（可選）|按需載入的字體，在`fonts/manifest.txt`中列出（請參閱[字體](#fonts)）|

沒有伺服器端程式碼。將資料夾的內容上傳到提供靜態檔案的任何位置。

## 碼頭工人

> Docker 配方（`Dockerfile`、`.dockerignore`、`packaging/web/nginx.conf`）是
> **社群維護**：它不是在 CI 中建置或用於發布，因此它可能落後於
> 官方網站建置如上。歡迎修復。

從儲存庫根目錄（Docker 是唯一的建置先決條件）：

```sh
docker build --load -t photocraft-web:local .
docker run -d --name photocraft-web --restart unless-stopped \
  -p 8080:8080 photocraft-web:local
```

開啟 **http://localhost:8080/**。多階段`Dockerfile`建構現有的 Rust/Wasm
應用程式與 Trunk，包括 HEIF 支援，然後僅將靜態站點複製到 NGINX 中。的
執行階段以`nginx`使用者身分在連接埠 **8080** 上執行，具有`/healthz`端點和 Docker
健康檢查。它提供預先壓縮的 gzip 資產、Wasm MIME 類型、不可變的雜湊值
資產以及部署後重新驗證的 HTML 頁面。無需安裝 Rust、Node.js、
主機上需要資料庫、GPU 直通或文件磁碟區。

第一個建置下載 Rust 相依性和 Trunk 的 Wasm 工具，並且可能需要幾個
分鐘。後續建置重用 Cargo 快取。該建構器支援Linux amd64和arm64；
預設 Rust 版本是`Dockerfile`上次更新時使用的穩定版本 CI
（CI 和發布始終使用最新的穩定版），並且 Trunk 固定到發布工作流程的
`TRUNK_VERSION`。`RUST_VERSION`、`TRUNK_VERSION`和`NGINX_VERSION`可以被覆蓋
`--build-arg`。要使用提交和日期而不是“dev build”來標記“關於”對話方塊，請傳遞
`--build-arg PHOTOCRAFT_BUILD_SHA=$(git rev-parse HEAD) --build-arg PHOTOCRAFT_BUILD_DATE=$(date -u +%F)`.
Web 版本不嵌入選購的`craft-fonts`結帳。

Docker 建置使用精簡 LTO 和一個 Cargo 建置作業來減少峰值記憶體，同時保持
網路設定檔的大小最佳化和`wasm-opt -Oz`。它的 Wasm 大小可能與
發布下面的郵遞區號資料； NGINX 沒有 25 MiB 檔案限制。使用release zip的fat LTO
具有更多記憶體的構建器，通過`--build-arg CARGO_PROFILE_WASM_RELEASE_LTO=fat`。
`--build-arg CARGO_BUILD_JOBS=4`可在較大的建構器上實現更多並發編譯。編譯器被殺死
`SIGKILL`/`cannot allocate memory`表示 Docker/Podman VM 需要更多記憶體或交換空間。

對於公共部署，請放置 HTTPS 反向代理或託管平台的 TLS 端點
在容器連接埠 8080 前面。在同一主機上使用反向代理，僅綁定環回
(`-p 127.0.0.1:8080:8080`);使用代理容器，透過 Docker 網路連接兩者。
HTTPS 支援WebGPU和瀏覽器剪貼簿 API。如果託管在`/photocraft/`下，請重定向
`/photocraft`到`/photocraft/`並在代理到容器時刪除該前綴。相對
資產 URL 無需重建即可使用。例如，在現有的 HTTPS NGINX 伺服器中：

```nginx
location = /photocraft { return 301 /photocraft/; }
location /photocraft/ {
    proxy_pass http://127.0.0.1:8080/;
}
```

編輯器和影像處理在訪客的裝置上執行。開啟使用瀏覽器文件
選取器；儲存並匯出下載檔案。首選項使用瀏覽器本地儲存。沒有
伺服器端文件儲存、桌面 TCP 控制伺服器或 Web 自動儲存/崩潰復原；
在關閉或重新載入頁面之前儲存/下載工作。帶有WebGPU或 WebGL2 的瀏覽器
必需；`?webgl`在故障排除時強制回退。在不能的瀏覽器中
膨脹流本身（沒有`DecompressionStream`：在 Safari 16.4 和 Firefox 113 之前），
release zip 的網站在自己的程式碼中膨脹了應用程式的各個部分，速度稍慢一些。

若要驗證瀏覽器部署，請開啟頁面，開啟映像並儲存/匯出它。

## 尺寸

測量於 2026 年 10 月 13 日（0.8.2，`packaging/web/package.sh`）：

|文件|尺寸|
|---|---|
|`app/photocraft-web-<hash>.part-aa`（今天唯一的部分）| 11,401,527 位元組 (10.9 MiB) |
|`app/photocraft-web.js`|約 170 KB（使用 gzip 時為 26 KB）|
|`photocraft-web-boot-<hash>_bg.wasm`與`.js`|總共約 120 KB |



每個檔案的上傳限制：Cloudflare Pages 和 Workers 靜態資產拒絕任何檔案超過
**25 MiB**（26,214,400 位元組），應用程式的WebAssembly就是這個大小。 （發佈至 0.2.0
發送了 25.8 MiB wasm，但 Cloudflare 拒絕了；問題#198。 ）所以該網站不認為它是
一個檔案：打包 gzip 並將結果切割成最多 20 MiB 的部分，頁面將
下載它們時它們會重新組合在一起。如果`packaging/web/package.sh`中存在任何文件，則`packaging/web/package.sh`將失敗
站點超過 24 MiB，因此版本無法傳送常見主機拒絕的文件，並且應用程式可以增長
沒有達到該限制（問題＃5302）。直接從 Trunk 建置（`trunk build --release`
在`apps/photocraft-web`或上面的 Docker 映像中）仍然在一個`.wasm`中包含該應用程式（對於主機）
沒有這樣的上限。

該應用程式是使用`wasm-release`Cargo 設定檔（胖 LTO、大小最佳化程式碼以及
保持高速的像素和檔案解碼箱）加上`wasm-opt -Oz`，並儲存捆綁的字體
（PhotoCraft和egui）和翻譯目錄已壓縮，首次使用時解壓縮。

## 任何路徑都可以

`index.html`中的所有 URL 都是相對的（`apps/photocraft-web/Trunk.toml`中的`public_url = "./"`），
因此該網站在域名根 (`https://example.com/`) 的前綴下工作
(`https://example.com/tools/photocraft/`) 和來自 CDN 儲存桶。資產名稱帶有內容
hash，這樣它們就可以永久快取。`index.html`、`app/photocraft-web.js`和`app/parts.txt`保留
他們的名字並需要重新驗證。

## 所需的伺服器設置

- **MIME 類型：** 將`.wasm`用作`application/wasm`。瀏覽器拒絕對其進行串流編譯
  任何其他類型，然後應用程式加載緩慢或根本不加載。將`.js`用作`text/javascript`。
  大多數主機都已經這樣做了。對於 nginx，檢查`mime.types`是否具有`application/wasm wasm;`。
- **零件如下：** 逐位元組服務`app/*.part-*`，沒有`Content-Encoding`（它們
  已經是 gzip 了，而且頁面會膨脹它們）。他們的擴充是伺服器沒有的
  知道，所以通常的結果`application/octet-stream`並且沒有壓縮，是正確的。
- **遺失的檔案是 404：** 使用`index.html`回答每個未知路徑的主機（a
  單頁應用程式後備）隱藏失敗的上傳。PhotoCraft注意到並說下載
  失敗，但伺服器日誌中的 404 更清晰。
- **壓縮：**為`.wasm`、`.js`和`.html`開啟 gzip 或 Brotli。這對於
  下載，因為這些部分已經被壓縮；它縮小了兩個`.js`檔案和
  小引導模組。您也可以預先壓縮並讓伺服器發送`Content-Encoding: br`。
- **快取：**`Cache-Control: public, max-age=31536000, immutable`位於其名稱的檔案上
  攜帶散列（零件和`photocraft-web-boot-<hash>.*`），以及`index.html`上的`no-cache`，
`app/photocraft-web.js`和`app/parts.txt`。範例`_headers`和`.htaccess`執行此操作。
- **HTTPS:**WebGPU（和剪貼簿）僅在安全上下文中工作，這意味著`https://`
  或`http://localhost`。在其他地方透過普通 HTTP，應用程式會回退到 WebGL2。
- **沒有特殊的隔離接頭：**PhotoCraft不使用`SharedArrayBuffer`，因此它不
  需要`Cross-Origin-Opener-Policy`或`Cross-Origin-Embedder-Policy`。如果您的網站已經發送
COEP`require-corp`，也發送`Cross-Origin-Resource-Policy: same-origin`（或`cross-origin`
  當檔案位於 CDN 上時）在應用程式的檔案上。

nginx 範例：

```nginx
location /photocraft/ {
    types { application/wasm wasm; text/javascript js; text/html html; }
    gzip on;
    gzip_types application/wasm text/javascript text/html;
    # The first matching line wins, so the files that keep their names come first.
    location ~* (index\.html|app/photocraft-web\.js|app/parts\.txt)$ { add_header Cache-Control "no-cache"; }
    location ~* (\.(wasm|js)|\.part-[a-z]+)$ { add_header Cache-Control "public, max-age=31536000, immutable"; }
}
```

本機測試：`python3 -m http.server 8765`資料夾內，然後開啟http://localhost:8765/。

## 字體

該應用程式有兩種自己的字體：Inter 和 JetBrains Mono：瀏覽器不提供系統字體，並且
嵌入更多會將 wasm 推過尺寸門。主機可以在旁邊提供更多字體
應用程式。它們立即顯示在字體選單中，並且第一次下載家庭的文件
它被選中或文件的文本需要它，因此啟動應用程式需要花費一個小文本文件。

1. 將字型檔案（`.ttf`、`.otf`、`.ttc`）放置在`fonts/`下`index.html`旁的任一位置。
2.將它們列在`fonts/manifest.txt`中，每行一個文件，格式為
   [工藝字體](https://github.com/storytold/craft-fonts)'自己的`fonts/manifest.txt`：

   ```text
   # family | style | file (relative to the site root) | scripts | licence | licence file | sha256 | source
   Open Sans | Regular | fonts/open-sans/OpenSans[wdth,wght].ttf
   Open Sans | Italic | fonts/open-sans/OpenSans-Italic[wdth,wght].ttf
   Lobster | Regular | fonts/lobster/Lobster-Regular.ttf | Latn,Cyrl | OFL-1.1 | fonts/lobster/OFL.txt
   ```

應用程式讀取前三個字段；其他的在這裡是可選的。`family`必須是字體的
   自己的家族名稱（其`name`表中的印刷家族，其他家族），這就是
   一旦文件進入，字體選單就會顯示。該路徑是相對於網站根目錄的，不能離開它
   （啟動時沒有`..`，沒有`/`，沒有其他主機）。可變字體是一行：樣式選單提供
   其`wght`軸的重量。
3. 將每種字體的授權放在旁邊；SIL 開放字體授權就是其中之一。

工藝字體結帳已經以這種方式佈置，任何帶有
`fonts/manifest.txt`及其列出的文件：`PHOTOCRAFT_WEB_FONTS_DIR=<that folder>
Packaging/web/package.sh` 將清單、字型及其授權複製到 zip 中，同時保留
他們的道路。如果沒有`fonts/manifest.txt`（404），則不會發生任何變化。

服務：讓`fonts/manifest.txt`重新驗證 (`Cache-Control: no-cache`)，以便顯示編輯內容；快取
只有當字體檔案的名稱隨內容變更時，字體檔案才會長久儲存。將`.ttf`用作`font/ttf`
和`.otf`為`font/otf`，並壓縮它們（Brotli 大致將 TrueType 檔案減半）。該應用程式
使用`fetch()`取得它們，因此內容安全策略需要`connect-src`允許它們。

## 嵌入頁面（iframe）

```html
<iframe
  src="https://example.com/photocraft/"
  title="PhotoCraft image editor"
  style="width: 100%; height: 720px; border: 0;"
  allow="fullscreen; clipboard-read; clipboard-write"
  allowfullscreen>
</iframe>
```

- 應用程式填滿 iframe 並遵循其大小，因此調整 iframe 的大小而不是應用程式的大小。
- 使用者點擊 iframe 後，鍵盤快速鍵將轉到 iframe，就像任何嵌入式應用程式一樣。
- **跨源嵌入**工作。首選項保存在 iframe 的`localStorage`中。瀏覽器
  分區或阻止第三方存儲可能會在訪問之間忘記它們，並且應用程式
  然後從預設值開始。
- **沙盒 iframe** 至少需要
`sandbox="allow-scripts allow-same-origin allow-downloads allow-popups"`。沒有
`allow-same-origin`沒有儲存空間。不含`allow-downloads`，儲存並匯出（瀏覽器
  下載）被阻止。
- 請勿傳送不包含嵌入頁面的`X-Frame-Options: DENY`或`frame-ancestors`CSP。

## 渲染器選擇和後備標誌

PhotoCraft使用wgpu進行渲染。當瀏覽器擁有它並回退到時，它使用 **WebGPU**
**WebGL2** 本身。 URL 查詢標誌會覆寫這一點，而且它們也適用於 iframe`src`：

|旗幟|效果|
|---|---|
| *（無）* |WebGPU（如果可用），否則 WebGL2 |
|`?webgl`|強制 WebGL2 後端（當WebGPU驅動程式行為不當時很有用）|
|`?cpu`|強制CPU畫布路徑（最慢，最相容）|

例如：`<iframe src="https://example.com/photocraft/?webgl" ...>`。

既沒有WebGPU也沒有 WebGL2 的瀏覽器會收到一則訊息來取代應用程式。
