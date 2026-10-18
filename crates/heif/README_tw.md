#photocraft-heif

PhotoCraft可選的 HEIF/HEIC 解碼器：iPhone 和 Mac 照片。薄薄的、恐慌的包裝紙
圍繞 [`heic-rs`](https://github.com/tbraun96/heic-rs)，一個純 Rust HEVC 靜態圖片解碼器
（無`unsafe`、MIT 或 Apache-2.0）：單張圖片和網格平鋪照片，8 位元和 10 位元（10 位元）
解碼為 16 位元）、alpha 輔助影像、容器的旋轉/鏡像/裁剪、ICC、EXIF 和 XMP。
只讀：寫入需要 HEVC 編碼器。

```rust
let info = photocraft_heif::probe(&bytes)?;             // size, depth, alpha: check limits first
let img = photocraft_heif::decode(&bytes, &photocraft_heif::Options::default())?;
// img.width, img.height, img.has_alpha, img.sixteen_bit, img.data (RGB/RGBA), img.icc, img.exif, img.xmp
```

錯誤為`Error::Unsupported`（影像序列、疊加…）、`Error::Limit`或
`Error::Malformed`。它永遠不會恐慌：每次對 heic-rs 的呼叫都在`catch_unwind`下執行，並且會出現恐慌
在它裡面（模糊測試在0.1.1中發現了兩個）變成了`Error::Malformed`。

## 為什麼它是一個單獨的、可選的 crate

- **一個年輕的解碼器。 ** heic-rs 是新的並且只有一個維護者。它被精確固定
  (`=0.1.1`);移動引腳是經過審查的更改（重新執行`cargo fuzz run decode_heif`
`crates/codecs`和`cargo xtask test-corpus`）。
- **HEVC 專利由經銷商決定。 ** HEVC 在某些司法管轄區受專利保護，因此
  構建是否包含 HEVC 解碼器是構建時的選擇。`photocraft-codecs`用的是這個
  僅在其非預設`heif`功能後面使用 crate；應用程式轉送它
  （`cargo build -pphotocraft--features heif`）。官方PhotoCraft建置和 CI 啟用它。
  沒有它，HEIC 文件仍然可以被識別，並且打開一個文件會明顯“不支援 HEIC/HEIF”
  包含在此版本中”錯誤。

此板條箱不相依性其他PhotoCraft板條箱，並且對`photocraft-codecs`的類型一無所知；
`codecs`→`heif`是獨立套件 (`cargo xtask layers`) 之間允許的一種相依性關係。
在`photocraft-codecs`中即時測試真實檔案（`tests/heif.rs`、功能`corpus,heif`、文件
由`cargo xtask corpus --heif`取得）。
