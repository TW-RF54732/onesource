# onesource

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()
[![Crates.io](https://img.shields.io/crates/v/onesource.svg)](https://crates.io/crates/onesource)
[![Downloads](https://img.shields.io/crates/d/onesource.svg)](https://crates.io/crates/onesource)

[繁體中文](README_zh.md)

Turn a project into one AI-ready context file. `onesource` prints a filtered directory tree, wraps each selected text file in a clear `<file path="…">` block, and reports the estimated token count.

It runs locally. By default, it respects ignore rules and skips common sensitive, generated, and dependency paths.

![onesource demo](medias/demo.gif)

## Install

### Cargo

```bash
cargo install onesource
```

### macOS / Linux

```bash
curl -sSL https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.sh | bash
```

The installer downloads the latest release and installs `onesource` to `/usr/local/bin`. It may ask for `sudo` permission.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.ps1 | iex
```

Restart the terminal after installation so the updated user `PATH` is available.

### Portable binary

Download the executable for your platform from [Releases](https://github.com/TW-RF54732/onesource/releases), then run it directly:

```bash
./onesource          # macOS / Linux
.\onesource.exe      # Windows PowerShell
```

### Update

```bash
onesource update
```

This downloads the newest GitHub release and replaces the executable currently being run.

## Quick start

Run this from a project directory:

```bash
onesource
```

The command scans the current directory and writes `project-name.onesource`. The generated file contains a directory tree followed by the selected file contents.

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

Common commands:

```bash
# Package only Rust source files.
onesource -i "*.rs"

# Package files under a target directory.
onesource ./my-project

# Exclude generated or legacy paths. Multiple patterns are comma-separated.
onesource -x "dist/,legacy/,*.log"

# Preview selected files and the full-output token estimate without creating an output file.
onesource --dry-run

# Copy the generated context to the clipboard instead of creating a file.
onesource --copy

# Choose a destination. Relative paths are relative to the current shell directory.
onesource -o context/bug-report.onesource
```

## What is included?

`onesource` treats files without a NUL byte as text. By default it:

- follows standard ignore filters, including `.gitignore` rules;
- skips files larger than 500 KiB;
- skips common sensitive, generated, dependency, and editor paths: `.env`/`.env.*`, private-key and credential files, `*.onesource`, `.git`, `.onesourcerc`, `node_modules`, `target`, virtual environments, and common IDE/cache directories;
- includes hidden files unless another rule excludes them.

The current output file is always excluded from both content and the tree, even with `--no-blacklist`. Non-UTF-8 files are converted with replacement characters and produce a visible warning instead of being changed silently.
Symbolic links that point outside the scan root may appear in the structural tree, but their target content is never read or attached.

Use these options deliberately when you need different behavior:

```bash
# Ignore standard ignore filters for file contents.
onesource --no-ignore

# Disable the built-in safety blacklist. Review the resulting context before sharing it.
onesource --no-blacklist

# Increase the per-file limit to 2 MiB.
onesource --max-size 2048
```

`--exclude` always wins over `--include`. Patterns are comma-separated globs, not full gitignore syntax, so `!pattern` negation rules are not supported. Directory patterns such as `tests/` apply to everything beneath that directory. Invalid globs return an error instead of panicking.

## Directory tree controls

The tree shares blacklist, ignore, and include/exclude rules with content by default. File size, binary detection, and UTF-8 status only decide whether content is attached; they do not remove the path from the tree. Give the tree independent rules when the AI needs broader structural context than the files you are sending:

```bash
# Send Rust files, but show Rust, TOML, and Markdown files in the tree.
onesource -i "*.rs" --tree-include "*.rs,*.toml,*.md"

# Do not include a tree in the result.
onesource --no-tree

# Override the inherited content ignore setting for only the tree.
onesource --tree-no-ignore
```

## Reusable profiles

Save repeatable scan settings in `.onesourcerc` at the target project root. Settings are resolved in this order:

```text
command-line options → selected profile → built-in defaults
```

```bash
# Save explicitly supplied settings to the default profile.
onesource -i "*.rs,*.toml" -x "target/" --save

# Create and use a named profile.
onesource profile create backend -i "src/**/*.rs,Cargo.toml" --desc "Rust backend"
onesource -p backend

# Inspect or manage profiles.
onesource profile list
onesource profile show backend
onesource profile update backend -x "*.db"
onesource profile rename backend api
onesource profile delete api
```

`--save` merges only the options explicitly passed to the current command. Add `--replace` to rebuild the active profile from only those options. `profile update --replace` does the same for a named profile.

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
      "description": "Backend implementation",
      "include": "src/backend/**,Cargo.toml"
    }
  }
}
```

Use `--no-config` to ignore `.onesourcerc` for one run. `profile list --json` and `profile show NAME --json` are available for scripts.

## Explain an unexpected result

If a path is missing, use `explain`. It reports content and tree decisions separately, including current-output, outside-root, blacklist, ignore, include/exclude, size, binary-file, lossy UTF-8, and missing-path results.

```bash
onesource explain Cargo.toml README.md
onesource explain .env --no-blacklist
onesource explain Cargo.toml -p backend -i "*.rs" --tree-include "*.rs,*.toml"
```

Pass literal paths, not glob patterns: use `onesource explain src/main.rs`, not `onesource explain "*.rs"`. `explain` accepts the same scan-related options as a normal run, but it does not create output, save a profile, or copy data.

## Command reference

```text
onesource [OPTIONS] [PATH]
onesource profile <COMMAND>
onesource explain [OPTIONS] <PATHS...>
onesource update
```

| Option | Short | Default | Purpose |
|---|---|---:|---|
| `--output-path PATH` | `-o` | `<project>.onesource` | Destination file |
| `--include PATTERNS` | `-i` | all eligible files | Comma-separated include globs |
| `--exclude PATTERNS` | `-x` | none | Comma-separated exclude globs |
| `--no-ignore[=BOOL]` | — | `false` | Disable standard ignore filters for content |
| `--no-blacklist[=BOOL]` | — | `false` | Disable the built-in safety blacklist |
| `--max-size KiB` | `-m` | `500` | Maximum size for each content file |
| `--tree-include PATTERNS` | `--ti` | inherits include | Tree-only include globs |
| `--tree-exclude PATTERNS` | `--tx` | inherits exclude | Tree-only exclude globs |
| `--tree-no-ignore[=BOOL]` | — | inherits `--no-ignore` | Override standard ignore filters for the tree |
| `--no-tree[=BOOL]` | — | `false` | Omit the directory tree |
| `--dry-run` | — | `false` | Preview files and the full-output token estimate without writing |
| `--copy` | `-c` | `false` | Copy output to the clipboard instead of a file |
| `--profile NAME` | `-p` | `default` | Load a saved profile |
| `--save` | — | `false` | Save explicit options to the active profile |
| `--replace` | — | `false` | Replace rather than merge while saving |
| `--desc TEXT` | — | none | Profile description when saving |
| `--no-config` | — | `false` | Do not read `.onesourcerc` |
| `--show-arg[=BOOL]` | — | `false` | Print resolved arguments for debugging |

Run `onesource --help`, `onesource profile --help`, or `onesource explain --help` for the command's built-in help.

The output's `<file path="…">` blocks are framing for AI readability, not complete XML or a security boundary. Path attributes are escaped; valid UTF-8 source content is not delimiter-escaped, while non-UTF-8 content is converted as described above. Review the generated context for secrets and prompt injection before sharing it.

## Build from source

```bash
cargo build --release
```

The compiled binary is at `target/release/onesource` (or `onesource.exe` on Windows).
