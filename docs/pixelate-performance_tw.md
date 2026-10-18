# 刻面與彩色半色調效能

2026年10月10日。兩個變更消除了現有 Pixelate 濾鏡中的重複工作：

- **Facet：** 每個來源像素計算一次亮度，並計算每個 3x3 象限一次。三行環與其四個輸出像素共享此象限。九樣本相加順序、變異數計算、象限順序和嚴格平局比較保持不變。
- **彩色半色調：** 分別為每個色版記住旋轉螢幕網格中的五次墨水計算和點半徑。像素仍然使用相同的候選點、距離和抗鋸齒來覆蓋範圍。實驗室轉換、CMYK 墨水方向和 Alpha 處理保持不變。

Facet 對於 256x256 輸出圖塊使用約 291 KiB 的額外刮痕。亮度平面的上限為 1,048,576 個樣本。每個色版的半色調表上限為 2 MiB。臨時分配是容易出錯的；不完整的光環、過大的表格或不合適的螢幕座標均使用原始計算。預設平鋪過濾器路徑在本機目標上保持並行，在 wasm 上保持單執行緒。

## 完整的過濾計時

EC2 m7i.4xlarge、16 個 vCPU (Intel Xeon Platinum 8488C)、Ubuntu 24.04、rustc 1.99.0、預設版本設定檔、`RAYON_NUM_THREADS=16`。底座：`7722172585a01cbdb93c06f0f5ff2634fcb17999`。更改後的演演算法位於`1bc3a9f1`中。之前和之後分別建置在不同的目標目錄中；測量期間沒有執行其他建置或基準測試。

每個條目是三個交替對的中位數：之前/之後、之後/之前、之前/之後。定時器覆蓋`apply_in`，包括tile讀取、過濾、組裝、深度轉換、寫回和剪枝。夾具構造、小型預熱和輸出序列化都在定時器之外。該裝置是一個密集的確定性本色圖像，具有不同的 Alpha、負文件原點和透明像素。這些是 CPU 過濾器應用時間；視窗呈現和撤銷簿記不在此測量範圍內。

|過濾|尺寸|模型+阿爾法|深度 |之前（毫秒）|之後（毫秒）|加速|
|---|---:|---|---|---:|---:|---:|
|刻面| 24 MP | RGB | u8 | 684.4 | 684.4 311.5 | 311.5 2.20 倍 |
|半色調 r8 | 24 MP | RGB | u8 | 1137.3 | 1137.3 603.8 | 603.8 1.88 倍 |
|刻面| 36 MP | RGB | u8 | 1027.5 | 1027.5 454.5 | 454.5 2.26 倍 |
|半色調 r8 | 36 MP | RGB | u8 | 1707.8 | 1707.8 898.0 | 1.90 倍 |
|刻面| 24 MP | RGB | u16 | 16 756.9 | 756.9 370.1 | 370.1 2.04 倍 |
|半色調 r8 | 24 MP | RGB | u16 | 16 1198.9 | 1198.9 664.7 | 664.7 1.80 倍 |
|刻面| 24 MP | RGB | f32 | f32 | 806.9 | 806.9 435.7 | 435.7 1.85 倍 |
|半色調 r8 | 24 MP | RGB | f32 | f32 | 1272.0 | 725.6 | 725.6 1.75 倍 |
|半色調 r2 | 24 MP | RGB | u8 | 1417.8 | 1417.8 716.4 | 716.4 1.98 倍 |
|半色調 r64 | 24 MP | RGB | u8 | 1138.9 | 1138.9 587.5 | 587.5 1.94 倍 |
|刻面| 24 MP |四色色 | u8 | 7888.4 | 574.2 | 574.2 13.74 倍 |
|半色調 r8 | 24 MP |實驗室| u8 | 3588.7 | 3588.7 718.3 | 718.3 5.00 倍 |

每對在完整輸出上與`cmp`進行比較，依圖塊順序序列化為小端 f32 樣本位元。所有 36 次比較都完全匹配，包括 16 位元和浮點情況。 [原始三輪計時和輸出 SHA-256 雜湊值](pixelate-performance.json) 與此表一起提交。

## 重現

針對基礎樹和更改後的樹建立相同的範例。將 Cargo 目標目錄分開：

```sh
git worktree add --detach ../photocraft-pixelate-base 7722172585a01cbdb93c06f0f5ff2634fcb17999
cp crates/algo/examples/bench_pixelate.rs ../photocraft-pixelate-base/crates/algo/examples/
(cd ../photocraft-pixelate-base && CARGO_TARGET_DIR=target cargo build --release -p photocraft-algo --example bench_pixelate)
CARGO_TARGET_DIR=target cargo build --release -p photocraft-algo --example bench_pixelate

RAYON_NUM_THREADS=16 ../photocraft-pixelate-base/target/release/examples/bench_pixelate facet 6000 4000 u8 1 before.bin rgb
RAYON_NUM_THREADS=16 target/release/examples/bench_pixelate facet 6000 4000 u8 1 after.bin rgb
cmp before.bin after.bin
sha256sum before.bin after.bin
```

每對重複三次，中間對的順序相反。使用`7200 5000`為 36 MP；其他深度為`u16`或`f32`；螢幕殼為`halftone`、`halftone-small`或`halftone-large`；其他顏色型號為 XZZPH13ZXX 或 XZPH14ZXZXZ。此範例將定時操作與文件寫入分開列印。

## 正確性與儲存庫檢查

獨立的標量預言機保留了原始核心。測試比較所有八種顏色模型、Alpha 開/關、恆定顏色和邊緣、隨機樣本、HDR/負值、非有限樣本、薄區域、部分光暈、極端螢幕偏移、擴展色版數和划痕抑制的每個樣本位。平鋪表面測試增加了 8/16/32 位元儲存、軟選擇、平鋪尺寸 1/7/17/256 以及透明和重複邊緣。

通過：401 項演演算法測試；警告被拒絕的 Clippy；格式化；相依性分層；完整的`cargo xtask wasm`套件； llvmpipe 上的`cargo xtask perf --quick`；和記分卡產生。命令、UI 或格式合約均未更改。
