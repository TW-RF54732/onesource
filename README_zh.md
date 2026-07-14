# onesource

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()
[![Crates.io](https://img.shields.io/crates/v/onesource.svg)](https://crates.io/crates/onesource)
[![Downloads](https://img.shields.io/crates/d/onesource.svg)](https://crates.io/crates/onesource)
[English](README.md)

把一個專案整理成一份可直接交給 AI 的上下文檔案。`onesource` 會輸出經過篩選的目錄樹，將每個選取的文字檔包在清楚的 `<file path="…">` 區塊中，並顯示預估 token 數。

它完全在本機執行。預設會遵守 ignore 規則，並略過常見的敏感檔案、產生檔與依賴目錄。

![onesource demo](medias/demo.gif)

## 安裝

### Cargo

```bash
cargo install onesource
```

### macOS / Linux

```bash
curl -sSL https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.sh | bash
```

安裝程式會下載最新 release，並把 `onesource` 安裝到 `/usr/local/bin`；可能會要求 `sudo` 權限。

### Windows（PowerShell）

```powershell
irm https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.ps1 | iex
```

安裝後請重新開啟終端機，讓更新後的使用者 `PATH` 生效。

### 可攜式執行檔

從 [Releases](https://github.com/TW-RF54732/onesource/releases) 下載對應平台的執行檔後，直接執行：

```bash
./onesource          # macOS / Linux
.\onesource.exe      # Windows PowerShell
```

### 更新

```bash
onesource update
```

此指令會下載最新的 GitHub release，並取代目前執行中的 `onesource` 執行檔。

## 快速開始

在專案目錄中執行：

```bash
onesource
```

它會掃描目前目錄，產生 `專案名稱.onesource`。檔案內容會先列出目錄樹，再接續選取檔案的內容。

```text
my-project/
├── Cargo.toml
└── src/
    └── main.rs

<file path="Cargo.toml">
[package]
name = "my-project"
</file>

<file path="src/main.rs">
fn main() {
    println!("Hello!");
}
</file>
```

常用指令：

```bash
# 只打包 Rust 原始碼。
onesource -i "*.rs"

# 打包指定目錄。
onesource ./my-project

# 排除產生檔或舊程式碼。多個模式以逗號分隔。
onesource -x "dist/,legacy/,*.log"

# 預覽選取檔案與完整輸出 token 估算，不建立輸出檔。
onesource --dry-run

# 將產生結果複製到剪貼簿，不建立檔案。
onesource --copy

# 指定輸出位置；相對路徑以目前終端機所在目錄為準。
onesource -o context/bug-report.onesource
```

## 會包含哪些內容？

`onesource` 會把不含 NUL byte 的檔案視為文字檔。預設行為為：

- 套用標準 ignore 篩選，包括 `.gitignore` 規則；
- 跳過大於 500 KiB 的檔案；
- 跳過常見的敏感檔案、產生檔、依賴與編輯器路徑：`.env`／`.env.*`、私鑰與 credential 檔、`*.onesource`、`.git`、`.onesourcerc`、`node_modules`、`target`、虛擬環境與常見 IDE／快取目錄；
- 隱藏檔預設會被掃描，除非被其他規則排除。

本次指定的輸出檔永遠不會被掃回內容或目錄樹，即使使用 `--no-blacklist` 也一樣。非 UTF-8 檔案會以 replacement character 轉換並顯示警告，不會再靜默改寫內容。
指向掃描根目錄外的 symbolic link 可出現在結構樹中，但不會讀取或附上其目標內容。

需要改變行為時，請謹慎使用以下選項：

```bash
# 不套用內容掃描的標準 ignore 篩選。
onesource --no-ignore

# 關閉內建安全黑名單；分享前請確認輸出內容。
onesource --no-blacklist

# 把單一檔案上限提高到 2 MiB。
onesource --max-size 2048
```

`--exclude` 永遠優先於 `--include`。模式是以逗號分隔的 glob，不是完整的 gitignore syntax，因此不支援 `!pattern` 否定規則；例如 `tests/` 這類目錄模式會套用到該目錄底下所有項目。無效 glob 會回傳錯誤而不會 panic。

## 目錄樹控制

預設情況下，目錄樹和檔案內容共用黑名單、ignore 與 include/exclude 規則。檔案大小、binary 與 UTF-8 狀態只決定是否附上內容，不會把路徑從樹中移除。當你只想傳送少量檔案、但仍要讓 AI 看到較完整的結構時，可以為目錄樹指定獨立規則：

```bash
# 只傳送 Rust 檔，但在樹中顯示 Rust、TOML 和 Markdown 檔。
onesource -i "*.rs" --tree-include "*.rs,*.toml,*.md"

# 輸出中不要包含目錄樹。
onesource --no-tree

# 只針對目錄樹覆寫從內容繼承的 ignore 設定。
onesource --tree-no-ignore
```

## 可重複使用的 Profile

可將常用掃描設定存到目標專案根目錄的 `.onesourcerc`。設定優先順序為：

```text
命令列選項 → 選取的 profile → 內建預設值
```

```bash
# 將本次明確輸入的設定存到預設 profile。
onesource -i "*.rs,*.toml" -x "target/" --save

# 建立並使用具名 profile。
onesource profile create backend -i "src/**/*.rs,Cargo.toml" --desc "Rust backend"
onesource -p backend

# 檢視或管理 profile。
onesource profile list
onesource profile show backend
onesource profile update backend -x "*.db"
onesource profile rename backend api
onesource profile delete api
```

`--save` 只會合併這次命令列明確傳入的選項。加入 `--replace` 可用這次選項完整重建目前 profile；`profile update --replace` 對指定 profile 的行為相同。

`.onesourcerc` 範例：

```json
{
  "profiles": {
    "default": {
      "include": "*.rs,*.toml",
      "exclude": "target/",
      "max_size": 300
    },
    "backend": {
      "description": "後端實作",
      "include": "src/backend/**,Cargo.toml"
    }
  }
}
```

單次執行想忽略 `.onesourcerc`，請加上 `--no-config`。腳本可使用 `profile list --json` 與 `profile show NAME --json` 取得 JSON。

## 找出檔案為何沒有出現

若某個路徑沒有出現在預期位置，使用 `explain`。它會分別顯示內容和目錄樹的判斷，包含本次輸出檔、掃描根目錄外路徑、黑名單、ignore、include/exclude、檔案大小、二進位檔、lossy UTF-8 與找不到路徑等結果。

```bash
onesource explain Cargo.toml README.md
onesource explain .env --no-blacklist
onesource explain Cargo.toml -p backend -i "*.rs" --tree-include "*.rs,*.toml"
```

請傳入實際路徑，不是 glob：使用 `onesource explain src/main.rs`，不要使用 `onesource explain "*.rs"`。`explain` 可使用和一般執行相同的掃描相關選項，但不會產生輸出、不會儲存 profile，也不會複製資料。

## 指令參考

```text
onesource [OPTIONS] [PATH]
onesource profile <COMMAND>
onesource explain [OPTIONS] <PATHS...>
onesource update
```

| 選項 | 縮寫 | 預設值 | 用途 |
|---|---|---:|---|
| `--output-path PATH` | `-o` | `<project>.onesource` | 輸出檔位置 |
| `--include PATTERNS` | `-i` | 所有符合條件的檔案 | 逗號分隔的 include glob |
| `--exclude PATTERNS` | `-x` | 無 | 逗號分隔的 exclude glob |
| `--no-ignore[=BOOL]` | — | `false` | 關閉內容的標準 ignore 篩選 |
| `--no-blacklist[=BOOL]` | — | `false` | 關閉內建安全黑名單 |
| `--max-size KiB` | `-m` | `500` | 每個內容檔案的大小上限 |
| `--tree-include PATTERNS` | `--ti` | 繼承 include | 僅用於目錄樹的 include glob |
| `--tree-exclude PATTERNS` | `--tx` | 繼承 exclude | 僅用於目錄樹的 exclude glob |
| `--tree-no-ignore[=BOOL]` | — | 繼承 `--no-ignore` | 覆寫目錄樹的標準 ignore 篩選 |
| `--no-tree[=BOOL]` | — | `false` | 不輸出目錄樹 |
| `--dry-run` | — | `false` | 預覽檔案和完整輸出 token 估算，不寫入 |
| `--copy` | `-c` | `false` | 複製結果到剪貼簿，不建立檔案 |
| `--profile NAME` | `-p` | `default` | 載入已儲存的 profile |
| `--save` | — | `false` | 將明確選項存到目前 profile |
| `--replace` | — | `false` | 儲存時取代而非合併 |
| `--desc TEXT` | — | 無 | 儲存時設定 profile 描述 |
| `--no-config` | — | `false` | 不讀取 `.onesourcerc` |
| `--show-arg[=BOOL]` | — | `false` | 印出已解析的參數以供除錯 |

可執行 `onesource --help`、`onesource profile --help` 或 `onesource explain --help` 查看內建說明。

輸出的 `<file path="…">` 是方便 AI 閱讀的 framing，不是完整 XML 或安全邊界。路徑屬性會被跳脫；有效 UTF-8 來源內容不做 delimiter escape，非 UTF-8 則依上述方式轉換。分享輸出前仍應審查內容與可能存在的提示注入。

## 從原始碼建置

```bash
cargo build --release
```

編譯後的執行檔位於 `target/release/onesource`（Windows 為 `onesource.exe`）。
