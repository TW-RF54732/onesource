# onesource

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()
[![GitHub Stars](https://img.shields.io/github/stars/TW-RF54732/onesource?style=social)](https://github.com/TW-RF54732/onesource/stargazers)


[中文文檔 Chinese README](https://github.com/TW-RF54732/onesource/blob/75be7d7d17a3797c7c251ae4265fbc734ce2e508/README_zh.md)
> Pack your entire project into a single context file — and paste it into your AI.  
> No Node.js. No Python. No cloud. Just download and run.

![OneSource Demo](./medias/demo.gif)
---

## The Story

I'm a first-year IM student. I break things constantly and ask AI to fix them — that's basically my workflow.

The problem: every time I wanted help from Claude or ChatGPT, I'd spend five minutes manually copying files, pasting them one by one, explaining the folder structure, forgetting a file, pasting again... Before the AI even saw my bug, I was already frustrated.

So I built a quick Python script to dump everything into one file. It worked. I used it every day.

Then I thought — *what if other people could use this?* I rewrote it in Rust. By hand. As a freshman. It killed brain cells I didn't know I had.

Is this the most powerful tool of its kind? No. Tools like [code2prompt](https://github.com/mufeedvh/code2prompt) exist and they're great. But they assume you know what you're doing. `onesource` assumes you just want it to work.

If you're a student, a beginner, or just someone who wants a dead-simple way to feed your project to an AI — this is for you.

---

## Why onesource?

There are already a few tools that do this. Here's an honest comparison:

| | **onesource** | **Repomix** | **Gitingest** | **code2prompt** |
|---|---|---|---|---|
| **Setup** | Download & run (one-line install) | Requires Node.js + npm | Web browser | Build from source |
| **Privacy** | 100% local | 100% local | Your code goes to their server | 100% local |
| **Dependencies** | Zero | `node_modules` (200MB+) | None (it's a website) | Zero |
| **Uninstall** | Delete the file. Done. | Good luck with `node_modules` | N/A | Delete the file |
| **Auto-install script** | Yes — one command for all platforms | No | No | No |
| **Offline** | Yes | Yes | No | Yes |

**vs Repomix:** It's a great tool, but I'm not installing a JavaScript runtime and 200MB of dependencies just to pack a text file.

**vs Gitingest:** Pushing your private, half-broken WIP code to the cloud to analyze it feels wrong. Everything here stays on your machine.

**vs code2prompt:** This is the honest one — code2prompt is powerful and aims at the same problem. If you're comfortable with Rust toolchains and want advanced features, check it out. If you just want something that installs in one line and gets out of your way, stay here.

---

## Installation

### macOS / Linux

```bash
curl -sSL https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.sh | bash
```

That's it. `onesource` will be available system-wide.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.ps1 | iex
```

Installs the latest release and adds it to your PATH automatically. Restart your terminal after.

### Portable Binary (no install)

Don't want to install anything? Grab the binary from the [Releases page](https://github.com/TW-RF54732/onesource/releases), drop it in your project folder, and run it directly:

```bash
./onesource          # macOS / Linux
.\onesource.exe      # Windows
```

---

## Usage

Run `onesource` from inside your project directory:

```bash
onesource [PATH] [OPTIONS]
```

If you run it with no arguments, it scans the current directory, respects your `.gitignore`, and writes everything to `{project name}.onesource`.

### Common workflows

**Just pack everything and save to a file:**
```bash
onesource
```

**Only pack your Rust source files:**
```bash
onesource -i "*.rs"
```

**Exclude a folder:**
```bash
onesource -x "tests/,legacy/"
```

**Preview what will be packed without actually writing anything:**
```bash
onesource --dry-run
```

**Show the directory tree separately from what gets packed:**  
*(Good for giving AI the full structure context, but only sending it specific files)*
```bash
onesource -i "*.rs" --tree-include "*.rs,*.toml,*.md"
```

---

## All Options

| Flag | Short | Default | Description |
|---|---|---|---|
| `path` | — | `.` (current dir) | Target directory to scan |
| `--output-path` | `-o` | `<folder-name>.onesource` | Output file path |
| `--include` | `-i` | all files | Only include files matching these comma-separated glob patterns |
| `--exclude` | `-x` | none | Exclude files matching these comma-separated glob patterns. Wins over `--include` on conflict. |
| `--no-ignore` | — | false | Ignore `.gitignore` rules when scanning file content |
| `--no-blacklist` | — | false | Disable the safety blacklist (allows scanning `.git/`, `node_modules/`, etc.) |
| `--tree-include` | `--ti` | inherits `-i` | Glob filter for the directory tree (enables independent tree mode) |
| `--tree-exclude` | `--tx` | inherits `-x` | Glob exclude for the directory tree |
| `--tree-no-ignore` | — | false | Ignore `.gitignore` rules only for the tree view |
| `--no-tree` | — | false | Disable the directory tree in output |
| `--max-size` | `-m` | 500 (KB) | Skip files larger than this size |
| `--dry-run` | — | false | Preview files that would be packed, without writing |
| `--save` | — | `default` | Save current flags to specified profile in `.onesourcerc` |
| `--desc` | — | none | Description to save with a profile when used with `--save` |
| `--show-arg` | — | false | Print resolved arguments for debugging |
| `--profile` | `-p` | `default` | Load a specific saved profile |
| `--no-config` | — | false | Ignore `.onesourcerc` and use only CLI flags |
| `--copy` | `-c` | false | Copy the result to the clipboard instead of writing a file |

---

## Configuration File

Running with `--save [NAME]` creates or updates a `.onesourcerc` file in your project directory. This is a basic optional workflow for saving repeated flag combinations. Profile management is intentionally small for now: save one, load one, and list what exists.

**Priority order:** CLI flags → Chosen Profile → Defaults

```bash
onesource -i "*.rs" -x "target/" --save backend
onesource -p backend
onesource profile ls
```

Example `.onesourcerc`:
```json
{
  "profiles": {
    "default": {
      "include": "*.rs,*.toml",
      "exclude": "target/",
      "max_size": 300
    },
    "backend": {
      "description": "Focus on backend logic",
      "include": "src/backend/*,Cargo.toml"
    }
  }
}
```

> Note: `path`, `--dry-run`, `--save`, `--show-arg`, and clipboard behavior are CLI-only. `.onesourcerc` is used as configuration input and is skipped by the default safety blacklist when packing project files.

---

## What the output looks like

```
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

Paste that into Claude, ChatGPT, or Gemini. The XML-style tags help the AI understand file boundaries clearly.

> **Tip:** Add `*.onesource` to your global `.gitignore` right now so you never accidentally commit an AI context file.

---

## Roadmap

This project started as a vibe-coded Python script. It's now a hand-written Rust binary that I actually understand (mostly). Here's what's next:

**Phase 1: Core Foundation (Fixes & Must-Haves)**
- [x] Hidden files support — reads hidden project files such as `.github/` while auto-blocking sensitive or noisy paths like `.git/`, `.env`, `node_modules/`, `target/`, and `.onesourcerc`
- [x] Safety blacklist — hardcoded block for `.git`, `node_modules`, `__pycache__`, `target` so you can't nuke your context window by accident
- [x] Smart output naming — output named after your project folder (`my-app.onesource` instead of `{project name}.onesource`)
- [X] Clipboard copy (`-c` flag) — write to clipboard instead of a file
- [x] Token counter — estimate how many tokens the output will use before you paste it

**Phase 2: Advanced Workflows (The Differentiators)**
- [ ] Profile polish — keep the existing basic save/load/list workflow, then consider small commands like `profile show`, `profile rm`, or `profile rename`
- [ ] Git Diff integration — incremental packing for modified files only, saving LLM context space

**Phase 3: Ecosystem & Integrations**
- [ ] Python bindings — import `onesource` directly in Python AI agents or CI/CD pipelines
- [ ] VSCode Extension — one-click context packing from the editor
- [ ] More install options (Homebrew, Scoop, Cargo)

If you have an idea or hit a bug, open an issue. I'm a student — I have time and I actually read them.

---

## Contributing

Pull requests are welcome. The codebase is small (~500 lines of Rust across 5 files) and not scary to navigate:

```
src/
├── main.rs         # Entry point, arg resolution, main flow
├── configs.rs      # CLI args + .onesourcerc config (clap + serde)
├── filter_utils.rs # Glob-based include/exclude logic
├── tree_utils.rs   # Directory tree builder and printer
└── io_utils.rs     # MultiWriter (write to file + stdout simultaneously)
```

To build locally:
```bash
git clone https://github.com/TW-RF54732/onesource.git
cd onesource
cargo build --release
```

---

## License

MIT — do whatever you want with it.

---

*Made by a first-year student who got tired of copy-pasting files into ChatGPT.*  
*If this saved you even five minutes, a star would mean a lot.*
