# 模糊測試

模糊測試程序、目前目標、安全性屬性和語料庫規則均在[模糊測試和回歸測試](../security/fuzzing.md) 中維護。

現有的模糊工作區有意與主 Cargo 工作區分開，因為`cargo-fuzz`使用夜間 Rust 和消毒劑儀器。例子：

```sh
rustup toolchain install nightly
cargo install cargo-fuzz --locked
cd crates/codecs
cargo +nightly fuzz run decode_any
```

重現崩潰，將其最小化，添加確定性回歸測試，然後修復它。沒有結果的限時模糊執行只是該執行的證據；它並不能證明解析器是安全的。

目前，主要的 GitHub Actions 工作流程僅在使用`fuzz`輸入手動分派時才執行模糊測試。計劃或託管的連續模糊測試仍然是未來的工作。
