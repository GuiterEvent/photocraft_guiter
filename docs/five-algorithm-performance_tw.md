# 五種成像演演算法：測量CPU效能

測量日期為 2026 年 10 月 10 日。這些更改消除了擠出塊的重複工作，
風/爆炸、旋轉模糊、鏡頭校正和印前陷阱。他們保留了現有的
輸出和參數行為。測量結果將完整的 CPU 操作與
`7722172585a01cbdb93c06f0f5ff2634fcb17999`，不只是核心。

## 發生了什麼變化

- **擠出區塊：**訪問每個單元格一次，計算其顏色和高度一次，然後
  柵格化其掃描範圍。保留原始候選範圍檢查、交集
  測試和細胞連接順序。預設的 30 px 區塊也能受益，而不僅僅是密集區塊。
- **風/爆炸：**將每個實際來源條紋分散一次。重複利用光源亮度和
  雜湊工作，保留最近來源綁定規則，然後每個輸出像素混合一次。
- **旋轉模糊：** 每個引腳準備一次所有 2..64 個樣本計數旋轉表。固定
  色版取樣器將雙線性累加器保留在暫存器中，無需更改抽頭或
  預乘 alpha 算術順序。
- **鏡頭校正：** 跨色版共享 Catmull-Rom 座標和權重
  使用固定色版累加器對同一點進行取樣。 RGB色差
  偏移量仍然使用獨立樣本。
- **陷阱：** 用可分離的前綴/後綴塊最大值取代寬視窗墨水掃描，
  與半徑無關。轉置連續列的中間輸出
  寫道。寬度 1 和 2 保留原始路徑。

＃＃ 方法

AWS m7i.4xlarge、16 個 vCPU、Intel Xeon Platinum 8488C、61 GiB RAM、Ubuntu 24.04.5。
Rust`1.99.0 (b940084d7 2026-09-28)`，發布設定文件，`RAYON_NUM_THREADS=16`。
建置使用的獨立目標目錄。計時期間沒有執行任何建置或測試。
最終的變基添加了上游畫布和自動化更改；定時演演算法
來源、固定裝置、相依性和 Cargo 設定均未變更。原始結果
記錄測量的修訂及其等效的重新基準修訂。

相同的已提交`bench_algorithms`範例在兩個版本中執行。它創造了
8位元、16位元或32位元浮點深度的原生RGB或CMYK圖塊，具有確定性
65536 等級繁忙紋理、可變 Alpha 和大約 20% 的零 Alpha 像素。界限開始於
（-17，-29）。 128 x 128 操作在計時之前預熱每個過程。

每個案例都有三個獨立的前後對，交替處理順序：
之前/之後，之後/之前，之前/之後。該表報告了每組的中位數
三個，加速比是根據未捨入的中位數計算的。夾具構造，
預熱、輸出序列化和結果銷毀都在定時器之外。

對於過濾器，定時器包括`apply_in`、圖塊讀取、處理、寫回和
修剪。鏡頭倍全`lens::correct`通話。陷阱包括表面讀數，
墨水膨脹、克隆、寫回和修剪。文件扁平化，撤銷簿記
和 UI 渲染不在這些 CPU 測量範圍內。

每對將每個輸出樣本依圖塊順序序列化為標準化 f32 位元。
SHA-256 之前/之後的所有 72 次比較完全匹配，包括 alpha。的
[原始結果](five-algorithm-performance.json) 包含所有時間、中位數和雜湊值。

擠出使用隨機深度區塊、種子 17、尺寸/深度 30/30 px 或 8/128 px
`extrude-dense`；實體正面、基於水平和不完整單元掩蔽已關閉。
風/疾風使用種子 17；`blast-right`改變方向。`spin`使用全片幅
橢圓銷，30 度模糊和 17 度方向；`spin-default`使用
預設引腳不變。鏡頭使用畸變 12、旋轉 2.5 度和暈影 20；
`lens-ca`也支援紅色/青色 12 和藍色/黃色 -8。陷阱寬度為 10、50
(`trap-wide`) 和 1 (`trap-default`) 像素。

## 完整執行結果

|案例 |尺寸|模式/深度|前|之後|加速|
|---|---|---|---:|---:|---:|
|`extrude`| 6000×4000 | RGB/u8 | 2.0641 | 2.0641 0.3296 | 0.3296 6.26 倍 |
|`extrude-dense`| 6000×4000 | RGB/u8 | 160.3010 | 0.3866 | 414.69x |
|`extrude`| 7200×5000 | RGB/u8 | 3.1539 | 3.1539 0.4954 | 0.4954 6.37 倍 |
|`extrude`| 6000×4000 | RGB/u16 | 2.1515 | 2.1515 0.4004 | 5.37 倍 |
|`extrude`| 6000×4000 | RGB/f32 | 2.2541 | 2.2541 0.4774 | 0.4774 4.72 倍 |
|`blast`| 6000×4000 | RGB/u8 | 1.4413 | 1.4413 0.4114 | 0.4114 3.50 倍 |
|`blast`| 7200×5000 | RGB/u8 | 2.1646 | 2.1646 0.6327 | 0.6327 3.42 倍 |
|`blast`| 6000×4000 | CMYK/u8 | 6.2310 | 0.7092 | 0.7092 8.79 倍 |
|`blast-right`| 6000×4000 | RGB/u16 | 1.5043 | 1.5043 0.4821 | 3.12 倍 |
|`wind`| 6000×4000 | RGB/f32 | 0.7678 | 0.4866 | 1.58 倍 |
|`wind`| 6000×4000 | CMYK/u8 | 2.5829 | 2.5829 0.6568 | 0.6568 3.93 倍 |
|`spin`| 6000×4000 | RGB/u8 | 6.3307 | 6.3307 3.9093 | 3.9093 1.62 倍 |
|`spin-default`| 6000×4000 | RGB/u8 | 1.5961 | 1.5961 1.1203 | 1.1203 1.42 倍 |
|`spin`| 7200×5000 | RGB/u8 | 9.4388 | 5.8694 | 1.61 倍 |
|`spin`| 6000×4000 | RGB/u16 | 6.3066 | 3.9617 | 3.9617 1.59 倍 |
|`spin`| 6000×4000 | RGB/f32 | 6.3749 | 6.3749 4.0619 | 1.57 倍 |
|`lens`| 6000×4000 | RGB/u8 | 0.6208 | 0.4448 | 1.40 倍 |
|`lens`| 6000×4000 | CMYK/u16 | 0.8080 | 0.5649 | 0.5649 1.43 倍 |
|`lens`| 6000×4000 | RGB/f32 | 0.6847 | 0.6847 0.5103 | 0.5103 1.34 倍 |
|`lens-ca`| 6000×4000 | RGB/u8 | 0.6214 | 0.6214 0.6312 | 0.98 倍 |
|`trap`| 6000×4000 | CMYK/u8 | 0.9710 | 0.8082 | 1.20 倍 |
|`trap-wide`| 7200×5000 | CMYK/u16 | 4.6265 | 4.6265 1.3331 | 3.47 倍 |
|`trap`| 6000×4000 | CMYK/f32 | 1.1924 | 1.0273 | 1.0273 1.16 倍 |
|`trap-default`| 6000×4000 | CMYK/u8 | 0.6363 | 0.6363 0.6506 | 0.6506 0.98 倍 |

## 正確性與有界工作

407 項演演算法測試已通過，其中 14 項已被忽略。新的直接引用
比較涵蓋所有八種顏色模型、Alpha 開/關、所有三種深度、邊緣
模式、不尋常的瓷磚尺寸和適用的軟選擇。他們也鍛鍊身體
關係、部分單元格、極端座標、非有限樣本和後備路徑。
自旋旋轉表與原始三角結果逐位匹配。
陷阱測試包括剪切視窗、紙張保存、額外色版和帶符號的零。

Extrude 將其額外獲勝平面限制為 4,194,304 像素 (16 MiB)、單元訪問
262,144，座標+/-2^22；正常的平鋪執行符合這些界限。
Wind 獲勝者的上限為每行 1,048,576 個條目 (8 MiB)。旋轉台是
小於 2 MiB，最多 120 個引腳。這些臨時分配是容易出錯的，並且使用
如果準備失敗或超出界限，則原始直接實施。
Extrude Pyramids 和 Wind Stagger 保留原始實作。陷阱是新的
每行後綴緩衝區是容易出錯的，有直接後備；它不添加完整圖像
超出現有膨脹工作空間的平面。包含負零的行使用
原始掃描保留浮點位模式。

`cargo fmt --all -- --check`，包含所有目標和拒絕警告的演演算法 Clippy，
`cargo xtask layers`、完整`cargo xtask wasm`、`cargo xtask perf --quick`和
記分卡產生/檢查已通過。快速效能執行使用 llvmpipe 和
仍然報告現有的超出預算的 UI 場景。這些操作測量
不要改變這些預算宣告或建立Photoshop像素奇偶校驗。

## 重現

從 PR 結帳中，建立一個基礎工作樹並僅複製基準夾具：

```sh
base=7722172585a01cbdb93c06f0f5ff2634fcb17999
git worktree add --detach ../photocraft-five-base "$base"
cp crates/algo/examples/bench_algorithms.rs ../photocraft-five-base/crates/algo/examples/
CARGO_TARGET_DIR="$PWD/target/five-after" cargo build --release -p photocraft-algo --example bench_algorithms
(cd ../photocraft-five-base && CARGO_TARGET_DIR="$PWD/target/five-before" cargo build --release -p photocraft-algo --example bench_algorithms)
before="$PWD/../photocraft-five-base/target/five-before/release/examples/bench_algorithms"
after="$PWD/target/five-after/release/examples/bench_algorithms"
export RAYON_NUM_THREADS=16

# Each invocation prints its full-operation time. Keep the machine otherwise idle.
# Use the case names, dimensions, depth and mode from the table.
case=extrude-dense
w=6000
h=4000
depth=u8
mode=rgb
for pair in 1 2 3; do
    if [ "$pair" = 2 ]; then
        "$after" "$case" "$w" "$h" "$depth" 1 after.bin "$mode"
        "$before" "$case" "$w" "$h" "$depth" 1 before.bin "$mode"
    else
        "$before" "$case" "$w" "$h" "$depth" 1 before.bin "$mode"
        "$after" "$case" "$w" "$h" "$depth" 1 after.bin "$mode"
    fi
    cmp before.bin after.bin || exit 1
    sha256sum before.bin after.bin
done
```

取每個二進制的三個時間的中位數。輸出檔可達 720 MB
每面 36 MP CMYK+alpha；在不同情況之間重複使用兩個檔案名稱。硬體，
工人數量和圖像內容會影響時間，因此在使用時請保留固定裝置
比較修訂版。
