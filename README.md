# OneSource ⚡

> **The Local-First Project Packer for AI Context.**
>
> 🚫 **Escape the Node.js ecosystem.** No `npm install`. No file uploads.
> 🚀 **Just download and run.** (Or `pip install` if you prefer).

[![PyPI version](https://img.shields.io/pypi/v/onesource-cli.svg)](https://pypi.org/project/onesource-cli/)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Roadmap: Rust Rewrite](https://img.shields.io/badge/Roadmap-Rust%20Rewrite-orange?logo=rust)

![OneSource Demo](OneSource_demo.gif)

**OneSource** aggregates your entire project into a single, context-rich text file (or clipboard) for LLMs like Claude, ChatGPT, and Gemini.

It bridges the gap between **Windows users** who want a simple `.exe` and **Python developers** who want a native CLI tool.
---
## 🦀 The Rust Future (v2.0 Alpha)
OneSource is currently being rewritten in Rust to provide:

- Blazing Fast Speed: Near-instant processing for massive monorepos.

- Zero Dependencies: A truly single-binary experience for all platforms.

- Memory Safety: Guaranteed stability during deep directory recursion.
- Highly optimized for multi-core scalability.

Check out the `rust-dev` branch to see the progress!

---

## 🥊 Why OneSource? Comparison & Positioning
We don't aim to be the most complex tool; we aim to be the one that stays out of your way.

| Feature | **OneSource** ⚡ | **Repomix** | **Gitingest** | **code2prompt** |
| --- | --- | --- | --- | --- |
| **Setup Friction** | ✅ **Zero** (Single EXE) | ❌ High (Node.js) | ✅ Low (Web) | ⚠️ Med (Binary) |
| **Local Privacy** | ✅ **100% Local** | ✅ Local | ❌ **Cloud-based** | ✅ Local |
| **System Residue** | ✅ **Zero** (Delete = Gone) | ❌ `node_modules` | ✅ None | ✅ None |
| **Setup Convenience** | ✅ Instant (one line pip/powershell commend) | ❌ High Friction (NPM) | ✅ Instant (Web) | ⚠️ Manual build from rust|
| **Clipboard Copy** | ✅ **Built-in** | ✅ Yes | ❌ Manual | ✅ Yes |

* **vs Repomix:** Stop installing 200MB of `node_modules` just to pack a text file. OneSource is truly lightweight.
* **vs Gitingest:** Don't push your private secrets or messy WIP code to GitHub just to analyze it. OneSource works on your *local* disk, offline.
* **vs code2prompt:** OneSource focuses on the **Windows native experience** and automated installation. While code2prompt is a solid Rust alternative, we prioritize "Zero-Friction" deployment for every developer.
---

## 📥 Installation

Select your platform below to see the instructions.

<details>
<summary><strong>🪟 Windows Users - Packed up exe (Click to expand)</strong></summary>

We offer two ways to install OneSource on Windows. Choose the one that fits your style.

#### Option 1: The Network Installer (PowerShell) - Recommended
*Best for most users. Installs the latest version and adds it to PATH via one command.*

Open **PowerShell** and paste the following:

```powershell
irm https://raw.githubusercontent.com/TW-RF54732/OneSource/main/install.ps1 | iex
```

#### Option 2: The Portable EXE

*Best for USB drives or temporary use.*

1. Download the standalone `OneSource.exe` from the **[Releases Page](https://github.com/TW-RF54732/OneSource/releases)**.
2. Place it anywhere (e.g., inside your project folder).
3. Run it directly via terminal: `.\OneSource.exe`

</details>

<details>
<summary><strong>🐍 Python Developers / Every OS - python pip cli tool (Click to expand)</strong></summary>

If you have Python installed or want to integrate this into your CI/CD pipeline, use PyPI.

**Installation:**

```bash
pip install onesource-cli


```

**Upgrade:**

```bash
pip install --upgrade onesource-cli


```

</details>

## 🎮 Usage Scenarios

Run these commands in your project root.

### Scenario 1: The "Lazy" Mode (Bug Fixing) 🌟

You broke the code. You need AI help NOW.
This packs everything (respecting `.gitignore`) and copies it to your clipboard.

```bash
OneSource -c


```

*-> Ctrl+V into ChatGPT.*

### Scenario 2: Focused Backend Work

Don't confuse the AI with frontend assets. Only grab the Python logic.

```bash
OneSource -i "*.py" -c


```

### Scenario 3: "Will this fit in the context window?"

Check token count before pasting.

```bash
OneSource -t --dry-run


```

### Scenario 4: Set It and Forget It

Always exclude `tests/` and `legacy/` folders? Save your config.

```bash
OneSource -x "tests/**,legacy/**" --save


```

*Creates a hidden config file. Next time, just run `OneSource`.*

### Scenario 5: "Smart Isolation" Mode (Separate Tree & Content) 🧠

Want to see the full project structure (including docs and configs) to give AI context, but only feed it the actual Python code content to save tokens?

```bash
OneSource -i "*.py" -ti "*.py,*.md,*.json" -c


```

*Files processed: Only `.py`. Project Tree shown: `.py`, `.md`, and `.json`.*

---

## 📖 Command Reference

| Argument | Description | Default |
| --- | --- | --- |
| `path` | **(Positional)** Target project path. | Current folder (`.`) |
| `-o`, `--output` | Output filename. | `allCode.txt` |
| `-c`, `--copy` | **Auto-copy** result to clipboard. | `False` |
| `-i`, `--include` | Only include files matching this pattern (Applied **AFTER** `.gitignore`). | All non-ignored files |
| `-x`, `--exclude` | Extra patterns to ignore. **Wins over `-i**` if there is a conflict. | `None` |
| `--no-ignore` | **Unlock mode:** Force scan files even if listed in `.gitignore`. | `False` |
| `-ti`, `--tree-include` | Tree include patterns. **Triggers Independent Mode** (isolates tree from file filters). | `None` (Inherits `-i`) |
| `-tx`, `--tree-exclude` | Tree exclude patterns. **Triggers Independent Mode** (isolates tree from file filters). | `None` (Inherits `-x`) |
| `--tree-no-ignore` | Ignore `.gitignore` rules *only* for the project tree visualization. | `False` |
| `-t`, `--tokens` | Show token count (requires `tiktoken`). | `False` |
| `--no-tree` | Disable the directory tree visualization at the top. | `False` |
| `--max-size` | Skip files larger than this size (in KB). | `500` KB |
| `--marker` | Custom XML tag for wrapping code (e.g., use `code` instead of `file`). | `file` |
| `--dry-run` | Preview which files will be processed without writing/copying. | `False` |
| `--save` | Save current flags as default config (`.onesourcerc`). | `False` |

---

*Built for Vibe Coding. Privacy First. Local First.*
