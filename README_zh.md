# onesource

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()
[![GitHub Stars](https://img.shields.io/github/stars/TW-RF54732/onesource?style=social)](https://github.com/TW-RF54732/onesource/stargazers)

[English README 英文文檔](https://github.com/TW-RF54732/onesource/blob/683ce72337df6ef7ab3df7f7e37d8523344a14b2/README.md)

> 將你的整個專案打包成單一個上下文檔案 — 然後貼給你的 AI。  
> 不需要 Node.js。不需要 Python。不需要雲端。只需下載並執行。

```text
$ onesource ./my-project
my-project/
├── src/
│   ├── main.rs
│   └── utils.rs
└── Cargo.toml

  + src/main.rs
  + src/utils.rs
  + Cargo.toml
======File processing completed======
Files Processed: 3
Output saved to: /home/user/my-project/allCode.txt
```

-----

## 故事背景 (The Story)

我是一名大一資工系學生。我經常把東西弄壞，然後請 AI 幫我修復 — 這基本上就是我的工作流程。

問題是：每次我需要 Claude 或 ChatGPT 幫忙時，我都要花五分鐘手動複製檔案，一個一個貼上，解釋資料夾結構，漏掉一個檔案，再貼一次... 在 AI 看到我的 bug 之前，我已經感到很挫折了。

所以我寫了一個簡單的 Python 腳本，把所有東西都倒進一個檔案裡。它很有效，我每天都在用。

然後我想 — *如果其他人也能用這個呢？* 我用 Rust 重新寫了一遍。純手工。身為一個大一新生。這抹殺了我很多不知道自己擁有的腦細胞。

這是同類工具中最強大的嗎？不是。像 [code2prompt](https://github.com/mufeedvh/code2prompt) 這樣的工具已經存在，而且非常棒。但它們假設你知道自己在做什麼。`onesource` 假設你只想要它能直接運作。

如果你是學生、初學者，或者只是想要一個極度簡單的方法把專案餵給 AI 的人 — 這個工具就是為你準備的。

-----

## 為什麼選擇 onesource？ (Why onesource?)

市面上已經有幾個類似的工具了。這是一個誠實的比較：

| | **onesource** | **Repomix** | **Gitingest** | **code2prompt** |
|---|---|---|---|---|
| **安裝設定** | 下載並執行 (單行指令安裝) | 需要 Node.js + npm | 網頁瀏覽器 | 從原始碼編譯 |
| **隱私** | 100% 本地端 | 100% 本地端 | 你的程式碼會上傳到他們的伺服器 | 100% 本地端 |
| **依賴套件** | 零 | `node_modules` (200MB+) | 無 (它是一個網站) | 零 |
| **解除安裝** | 刪除檔案。搞定。 | 祝你處理 `node_modules` 順利 | 不適用 | 刪除檔案 |
| **自動安裝腳本** | 有 — 所有平台只需一行指令 | 無 | 無 | 無 |
| **離線使用** | 可以 | 可以 | 否 | 可以 |

  * **對比 Repomix：** 它是一個很棒的工具，但我不想為了一個打包文字檔的功能去安裝 JavaScript 執行環境和 200MB 的依賴套件。
  * **對比 Gitingest：** 把你私人的、做到一半有 bug 的程式碼推送到雲端進行分析感覺不對勁。這裡的所有東西都會留在你的機器上。
  * **對比 code2prompt：** 說實話 — code2prompt 很強大且旨在解決相同的問題。如果你對 Rust 工具鏈很熟悉且需要進階功能，去試試看它。如果你只想要一行指令就能安裝且不會干擾你的工具，請留在這裡。

-----

## 安裝方式 (Installation)

### macOS / Linux

```bash
curl -sSL https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.sh | bash
```

就這樣。`onesource` 將可以在整個系統中使用。

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.ps1 | iex
```

這會安裝最新版本並自動將其加入你的 PATH 中。之後請重新啟動你的終端機。

### 免安裝可攜式執行檔 (Portable Binary)

不想安裝任何東西？從 [Releases 頁面](https://github.com/TW-RF54732/onesource/releases) 取得執行檔，丟進你的專案資料夾，然後直接執行它：

```bash
./onesource          # macOS / Linux
.\onesource.exe      # Windows
```

-----

## 使用方法 (Usage)

在你的專案目錄內執行 `onesource`：

```bash
onesource [PATH] [OPTIONS]
```

如果你不帶任何參數執行它，它會掃描當前目錄，遵守你的 `.gitignore` 規則，並將所有內容寫入 `allCode.txt`。

### 常見工作流程

**直接打包所有東西並存成檔案：**

```bash
onesource
```

**只打包你的 Rust 原始碼檔案：**

```bash
onesource -i "*.rs"
```

**排除特定資料夾：**

```bash
onesource -x "tests/,legacy/"
```

**預覽會被打包的內容，而不實際寫入任何檔案：**

```bash
onesource --dry-run
```

**儲存你的設定，這樣就不用重新輸入：**

```bash
onesource -i "*.rs" -x "target/" --save
# 下次只需執行： onesource
```

**將目錄結構樹與打包內容分開顯示：**  
*(適合用來給 AI 完整的結構上下文，但只發送特定的檔案給它)*

```bash
onesource -i "*.rs" --tree-include "*.rs,*.toml,*.md"
```

-----

## 所有選項 (All Options)

| 標籤 (Flag) | 縮寫 (Short) | 預設值 (Default) | 描述 (Description) |
|---|---|---|---|
| `path` | — | `.` (當前目錄) | 要掃描的目標目錄 |
| `--output` | `-o` | `allCode.txt` | 輸出檔案路徑 |
| `--include` | `-i` | 所有檔案 | 只包含符合此 glob 模式的檔案 |
| `--exclude` | `-x` | 無 | 排除符合此 glob 模式的檔案。衝突時優先於 `--include`。 |
| `--no-ignore` | — | false | 忽略 `.gitignore` 規則並掃描所有內容 |
| `--tree-include` | `--ti` | 繼承 `-i` | 用於目錄樹的 Glob 過濾器 (啟用獨立的樹狀圖模式) |
| `--tree-exclude` | `--tx` | 繼承 `-x` | 用於目錄樹的 Glob 排除器 |
| `--tree-no-ignore` | — | false | 僅在樹狀檢視中忽略 `.gitignore` 規則 |
| `--no-tree` | — | false | 在輸出中停用目錄結構樹 |
| `--max-size` | `-m` | 500 (KB) | 跳過大於此大小的檔案 |
| `--dry-run` | — | false | 預覽將被打包的檔案，但不寫入 |
| `--save` | — | false | 將當前標籤儲存到目標目錄的 `.onesourcerc` 中 |
| `--no-config` | — | false | 忽略 `.onesourcerc`，只使用 CLI 標籤 |
| `--copy` | `-c` | false | 直接複製輸出至剪貼簿(不會有任何檔案被創建或複寫)|


-----

## 設定檔 (Configuration File)

加上 `--save` 執行會在你的專案目錄中建立一個 `.onesourcerc` 檔案。下次你在該處執行 `onesource` 時，它會自動讀取你儲存的設定。

**優先順序：** CLI 參數 → `.onesourcerc` → 預設值

範例 `.onesourcerc`：

```json
{
  "output_path": "context.txt",
  "include": "*.rs,*.toml",
  "exclude": "target/",
  "no_ignore": false,
  "max_size": 300
}
```

> 注意：`path`、`--dry-run`、`--save` 和 `--show-arg` 永遠不會被存入設定檔中 — 它們始終作為 CLI 參數傳遞。

-----

## 輸出內容長怎樣 (What the output looks like)

```text
my-project/
├── src/
│   └── main.rs
└── Cargo.toml

<file path="src/main.rs">
fn main() {
    println!("Hello, world!");
}
</file>

<file path="Cargo.toml">
[package]
name = "my-project"
version = "0.1.0"
</file>
```

把這些貼到 Claude、ChatGPT 或 Gemini 中。類似 XML 的標籤能幫助 AI 清楚理解檔案的邊界。

-----

## 開發藍圖 (Roadmap)

這個專案最初是一個靠感覺寫出來的 Python 腳本。現在它是一個我自己親手寫的、而且（大部分）能看懂的 Rust 執行檔。接下來的計畫如下：

**第一階段：核心基礎 (修復與必備功能)**

  * [x] 支援隱藏檔案 — 準確讀取 `.github/` 等，同時安全地忽略 `.git/`、`.env`。
  * [x] 複製到剪貼簿 (`-c` 標籤) — 寫入剪貼簿而不是檔案。
  * [x] Token 計數器 — 在你貼上之前估算輸出會使用多少 tokens。

**第二階段：進階工作流程 (差異化功能)**

  * [ ] 多個設定檔設定 — 瞬間切換設定 (例如：`onesource --profile backend`)。
  * [ ] Git Diff 整合 — 只對修改過的檔案進行增量打包，節省 LLM 的上下文空間。

**第三階段：生態系統與整合**

  * [ ] Python 綁定 — 在 Python AI Agents 或 CI/CD 流程中直接匯入 `onesource`。
  * [ ] VSCode 擴充套件 — 在編輯器中一鍵打包上下文。
  * [ ] 更多安裝選項 (Homebrew, Scoop, Cargo)。

如果你有任何想法或遇到 bug，歡迎開啟 Issue。我是個學生 — 我有時間，而且我真的會看。

-----

## 參與貢獻 (Contributing)

歡迎發起 Pull requests。程式碼庫很小 (大約 500 行 Rust 程式碼，分佈在 5 個檔案中)，瀏覽起來並不可怕：

```text
src/
├── main.rs         # 進入點，參數解析，主要流程
├── configs.rs      # CLI 參數 + .onesourcerc 設定 (clap + serde)
├── filter_utils.rs # 基於 Glob 的包含/排除邏輯
├── tree_utils.rs   # 目錄結構樹建立器和列印器
└── io_utils.rs     # MultiWriter (同時寫入檔案 + 標準輸出)
```

要在本地端編譯：

```bash
git clone https://github.com/TW-RF54732/onesource.git
cd onesource
cargo build --release
```

-----

## 授權條款 (License)

MIT — 你想拿它做什麼都可以。

-----

*由一個厭倦了把檔案複製貼上到 ChatGPT 的大一新生製作。*  
*如果這幫你省下了哪怕五分鐘的時間，給我一顆星星將對我意義重大。*
