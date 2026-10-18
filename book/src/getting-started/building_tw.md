＃ 建築

PhotoCraft使用儲存庫根目錄下的 Rust 工作區。安裝穩定的 Rust 工具鏈，克隆儲存庫，然後從該根執行命令。

在Linux上，先安裝windowing、GL和GTK開發套件。開髮指南的
[先決條件](https://github.com/storytold/photocraft/blob/main/docs/development.md#prerequisites)
列出 Debian/Ubuntu、Fedora、Arch 和 openSUSE 的它們。

```sh
cargo build --workspace
cargo run --release -p photocraft -- image.psd
cargo run -p photocraft-cli -- --help
```

標準存儲庫門是：

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm
```

`cargo xtask wasm`需要`wasm32-unknown-unknown`目標。也應使用[開發指出](https://github.com/storytold/photocraft/blob/main/docs/development.md) 中所述的離螢幕快照範例來檢查 UI 工作。

## 建立文件

安裝 mdBook，然後從儲存庫根目錄建置：

```sh
cargo install mdbook --locked
mdbook build book
```

產生的網站寫入`book/output/`，不是來源文件。
