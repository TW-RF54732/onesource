# onesource ⚡

> **The Local-First Project Packer for AI Context.**
>
> 🚫 **Escape the Node.js & Python ecosystem.** No `npm install`. No `pip install`.
> 🚀 **Just download and run.** A blazingly fast, single native binary.

**onesource** aggregates your entire project into a single, context-rich text file for LLMs like Claude, ChatGPT, and Gemini.

Rewritten entirely in **Rust** for v2.0, it bridges the gap between massive monorepos and AI context limits with near-instant processing, memory safety, and absolute zero-friction deployment.

-----

## 🦀 The Rust Rewrite (v2.0)

We left Python behind to bring you the ultimate developer experience:

  - **Blazing Fast Speed:** Near-instant directory traversal and filtering using highly optimized `ignore` and `globset` crates.
  - **Zero Dependencies:** A truly single-binary experience. No interpreters, no virtual environments.
  - **Strict Configuration:** Predictable parameter priority (`CLI -> .onesourcerc -> Defaults`) with a robust "Exclude-First" filtering logic.

-----

## 📥 Installation

Choose your platform below for a one-line installation.

**🪟 Windows (PowerShell):**
Installs the latest `.exe` and automatically adds it to your user PATH.

```powershell
irm https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.ps1 | iex
```

**🍎 macOS / 🐧 Linux (Terminal):**
Automatically detects your OS/Architecture (including Apple Silicon) and installs to `/usr/local/bin`.

```bash
curl -sSL https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.sh | bash
```

**🦀 Cargo (Rust Developers):**
Build directly from the source.

```bash
cargo install --git https://github.com/TW-RF54732/onesource.git
```

-----

## 🎮 Usage Scenarios

Run these commands in your project root.

### Scenario 1: The "Lazy" Mode (Bug Fixing) 🌟

You broke the code. You need AI help NOW.
This packs everything (respecting `.gitignore`) into `allCode.txt` instantly.

```bash
onesource
```

*-\> Open `allCode.txt` and paste it into ChatGPT.*

### Scenario 2: Focused Backend Work

Don't confuse the AI with frontend assets. Only grab the Rust/Python logic and output to a specific file.

```bash
onesource -i "*.rs,*.py" -o backend_logic.txt
```

### Scenario 3: "Smart Isolation" Mode (Separate Tree & Content) 🧠

Want to show the AI the full project structure (including docs and configs) for context, but *only* feed it the actual source code content to save tokens?

```bash
onesource -i "*.rs" -ti "*.rs,*.md,*.toml"
```

*Files processed: Only `.rs`. Project Tree shown: `.rs`, `.md`, and `.toml`.*

### Scenario 4: Preview Only

Check which files will be packed before actually writing to the disk.

```bash
onesource --dry-run
```

### Scenario 5: Set It and Forget It

Always exclude `tests/` and `target/` folders? Save your config.

```bash
onesource -x "tests/**,target/**" --save
```

*Creates a hidden `.onesourcerc` file. Next time, just run `onesource`.*

-----

## 📖 Command Reference

| Argument | Description | Default |
| --- | --- | --- |
| `path` | **(Positional)** Target project directory to scan. | `.` (Current directory) |
| `-o`, `--output-path` | Output filename and path. | `allCode.txt` |
| `-i`, `--include` | Only include files matching this pattern (e.g., `*.rs,src/`). | All non-excluded files |
| `-x`, `--exclude` | Extra patterns to ignore. **Wins over `-i`** (Exclude-First logic). | `None` |
| `--no-ignore` | **Unlock mode:** Force scan files even if listed in `.gitignore`. | `False` |
| `-t`, `--tree-include` | Tree include patterns. Overrides global `-i` for the tree view. | `None` (Inherits `-i`) |
| `-t`, `--tree-exclude` | Tree exclude patterns. Overrides global `-x` for the tree view. | `None` (Inherits `-x`) |
| `--tree-no-ignore` | Ignore `.gitignore` rules *only* for the project tree visualization. | `False` |
| `--no-tree` | Disable the directory tree visualization at the top of the output. | `False` |
| `--max-size` | Skip files larger than this size (in KB). | `500` KB |
| `--dry-run` | Preview mode: List files without generating the output file. | `False` |
| `--save` | Save current flags as the default config in `.onesourcerc` (JSON). | `False` |
| `--no-config` | Ignore the `.onesourcerc` configuration file and use raw defaults/CLI inputs. | `False` |
| `--show-arg` | Debug mode: Show parsed arguments before execution. | `False` |

-----

*Built for Vibe Coding. Privacy First. Local First.*
