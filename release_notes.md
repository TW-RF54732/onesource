### OneSource v1.3.0 Changelog

**Release Date**: 2026-01-25
**Highlight**: 🚀 **"Smart Isolation" Mechanism for File Content vs. Project Tree**

This release resolves a major pain point by decoupling the "file content aggregation" from the "project tree visualization." You can now show large or binary files (like `.exe` or `docs/`) in the project structure to give AI the full context, without wasting tokens by injecting their actual content.

#### ✨ New Features

* **Tree-Specific Filters**: Added dedicated CLI flags to control the project tree structure independently.
* `-ti` / `--tree-include`: Include specific patterns *only* in the tree view.
* `-tx` / `--tree-exclude`: Exclude specific patterns *only* in the tree view.
* `--tree-no-ignore`: Ignore `.gitignore` rules *only* for the tree view.


* **Smart Isolation & Fallback Mechanism**:
* **Independent Mode**: Triggering any `-ti` or `-tx` flag automatically isolates the tree logic from the file content logic.
* **Inherited Mode**: If no tree-specific flags are provided, the tree automatically inherits the `-i` and `-x` settings, ensuring a clean CLI experience and backward compatibility.


* **Extended Configuration Persistence**: The `--save` flag now correctly persists `--no-ignore` and all new `--tree-*` configurations to the `.onesourcerc` file.

#### 🛠️ Refactoring & Optimizations

* **Decoupled Filtering Logic**: Split the singular `_should_ignore` method into `_should_ignore_file` and `_should_ignore_tree`.
* **`_should_ignore_tree` (Path Matching Only)**: No longer checks file size or binary status. This ensures that large/compiled files remain visible in the tree structure, even if they aren't written to the output file.
* **`_should_ignore_file` (Deep Inspection)**: Retains strict size limits and binary detection to ensure only valid text content is processed.


#### 🐛 Bug Fixes

* Fixed an issue where large files or binaries were erroneously hidden from the top-level project structure preview due to `--max-size` or binary checks.
