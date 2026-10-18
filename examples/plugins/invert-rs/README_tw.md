# Invert：PhotoCraft外掛程式範例

一個完整的WebAssembly過濾器插件，大約 70 行`no_std`Rust：沒有分配器，沒有導入，
構建不到 1 KiB。它反相顏色色版並單獨保留 alpha，匹配
圖片 › 調整 › 反相。 ABI 記錄在 [`docs/plugins.md`](../../../docs/plugins.md) 中。

＃＃ 建造

```sh
rustup target add wasm32-unknown-unknown
./build.sh
```

`build.sh`執行`cargo build --release --target wasm32-unknown-unknown`並將模組複製到
`crates/plugins/tests/fixtures/invert.wasm`，主機測試使用的測試夾具。這個箱子不是
PhotoCraft工作區的成員。

＃＃ 安裝

```sh
# Into a running app (control channel) or the CLI, as a command:
plugin.install {"path": "examples/plugins/invert-rs/target/wasm32-unknown-unknown/release/photocraft_plugin_invert.wasm"}
plugin.run {"id": "org.photocraft.example.invert"}
```

或將`.wasm`複製到「編輯」>「首選項」>「外掛」中設定的資料夾（使用 *使用附加
插件資料夾* on);然後它會出現在“過濾器”>“插件”下。
