# 測試

測試是原始程式碼更改的大門。在開發過程中選擇最窄的相關套件測試，然後執行適合所涉及層的儲存庫門。

```sh
cargo test -p <package>
cargo clippy -p <package> --all-targets -- -D warnings
cargo xtask layers
```

對於 L6 以下或 L6 處的更改，也可以執行`cargo xtask wasm`。命令更改需要忽略對抗性測試：

```sh
cargo test -p photocraft-engine --test panic_hunt -- --ignored
```

格式變更應包括有效的往返、精確的邊界值、截斷、長度不一致、格式錯誤的結構以及分配/解壓縮限制情況。像素行為應在適用的 8 位元、16 位元和 32 位元深度下執行。

UI 變更需要從螢幕外快照範例或控制頻道螢幕截圖中呈現證據。成功的建置或 HTTP 回應不是視覺驗證。

文件變更透過以下方式驗證：

```sh
mdbook build book
```

還應檢查內部 Markdown 連結和每個`SUMMARY.md`目的地。文件工作流程在 CI 中執行乾淨的 mdBook 建置。
