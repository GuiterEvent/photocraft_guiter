# 關於視窗中的貢獻者

**關於 ▸ 貢獻者** 歸功於為PhotoCraft做出貢獻的每個人，以及 **關於 ▸ 模型** 積分
`Co-Authored-By`預告片中命名的 AI 模型。這遵循共享工藝規則標準
[`standards/contributors.md`](https://github.com/storytold/craftrules/blob/main/standards/contributors.md);
此頁面是該決定的本機副本。

＃＃ 決定

- 兩個文件，均由 UI 板條箱的`build.rs`**編譯為二進位檔案**（未從其中讀取任何內容）
  執行階段的磁碟或網路；網路建置也有它們）：
  - [`contributors/contributors.json`](../contributors/contributors.json)：按 GitHub 提交統計信息
    用戶名。 **由`craftrules/scripts/contributors.py`產生**；切勿手動編輯。
  - [`contributors/people.toml`](../contributors/people.toml)：人們選擇的名字
    他們自己。 **每個貢獻者僅在自己的 PR 中新增自己的條目**。
- 每個貢獻者都會得到一行：GitHub 使用者名稱（永遠）、顯示名稱（選擇加入，或其同步的
  GitHub 個人資料名稱）、真實姓名（僅當他們告訴我們時）、合併的 PR、提交、新增的行、行
  已刪除、已新增二進位資產、已刪除二進位資產、第一個和最後一個提交日期。
- 此清單顯示為**抓包**（名稱以文字形式流動）或**表格**，順序相同。它
  按名稱、PR、提交、新增的行、刪除的行、行增量、新增/刪除的二進位資產排序，
  第一次或最後一次提交日期。名稱切換循環**使用者名稱→顯示名稱→真實姓名**；一個
  缺失的名稱回落為`@username`。按字母順序排序不區分大小寫並忽略`@`。

## 在PhotoCraft中

- The About window is`DialogKind::About`in`crates/ui-egui/src/dialogs.rs`(Help ▸ AboutPhotoCraft).
  Its tab is the dialog field`tab`(`about`,`contributors`,`models`), so agents can switch it with
`ui.dialog.set`。 Labels are translated like the rest of the shell (`tl!`+`crates/ui-egui/src/i18n/*.tsv`).

## 相信自己

將您的條目新增至您自己提交的 PR 中的`contributors/people.toml`（每個欄位都是可選的）：

```toml
[people.your-github-username]
real_name = "Your Name"
display_name = "Nickname"
sync_github_name = true   # use the name on your public GitHub profile as display name
```

或者，在`craft-apps/`旁邊查看工藝規則：

```sh
python3 ../../craftrules/scripts/contributors.py --add-me . --real-name "Your Name" --sync-github-name
```

下次刷新後顯示名稱：指令碼檢查每個條目是否由自行提交
使用者（或由具有`consent_source`的維護人員）。刪除您的條目即可刪除您的名字。

## 刷新（維護者）

```sh
python3 ../../craftrules/scripts/contributors.py .           # regenerate + verify; needs git, Python 3.11+, gh
python3 ../../craftrules/scripts/contributors.py --check .   # verify people.toml only
```

提交更新的`contributors/contributors.json`。 Git 作者姓名和電子郵件永遠不會被記錄。
