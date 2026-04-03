import os
import sys
import shutil
import argparse
import json
from pathlib import Path
import pathspec
import pyperclip

# Optional: Precise Token Calculation
try:
    import tiktoken
    import tiktoken_ext.openai_public 
    HAS_TIKTOKEN = True
except ImportError:
    HAS_TIKTOKEN = False

VERSION = "v1.3.1"

BANNER = rf"""
==========================================================
  ____  _   _ _____   ____   ___  _   _ ____   ____ _____ 
 / __ \| \ | | ____| / ___| / _ \| | | |  _ \ / ___| ____|
| |  | |  \| |  _|   \___ \| | | | | | | |_) | |   |  _|  
| |__| | |\  | |___   ___) | |_| | |_| |  _ <| |___| |___ 
 \____/|_| \_|_____| |____/ \___/ \___/|_| \_\\____|_____|
                          
 >> OneSource {VERSION} | The Local-First Vibe Coding Tool <<
==========================================================
"""

CONFIG_FILE = ".onesourcerc"

class OneSource:
    def __init__(self):
        self.args = self._parse_args()
        self.root = Path(self.args.path).resolve()
        self.output_path = Path(self.args.output).resolve()
        
        # 1. File Content Filter Configuration
        self.spec = self._load_gitignore(self.args.no_ignore)
        self.include_spec = self._build_pathspec(self.args.include)
        self.exclude_spec = self._build_pathspec(self.args.exclude)
        
        # 2. Project Tree Filter Configuration (Smart Isolation Mechanism)
        # Determine if .gitignore rules apply to the tree view
        tree_no_ignore_flag = self.args.tree_no_ignore or self.args.no_ignore
        self.tree_spec = self._load_gitignore(tree_no_ignore_flag)

        # Check if the user has explicitly defined ANY tree-specific filter flags
        tree_is_independent = (self.args.tree_include is not None) or (self.args.tree_exclude is not None)

        if tree_is_independent:
            # Independent Mode: Disconnect inheritance. 
            # Tree rules are strictly controlled by -ti and -tx. Unset values default to None (no filter).
            t_inc = self.args.tree_include
            t_exc = self.args.tree_exclude
        else:
            # Inherited Mode: Fully replicate the file content filter settings for the tree view.
            t_inc = self.args.include
            t_exc = self.args.exclude

        self.tree_include_spec = self._build_pathspec(t_inc)
        self.tree_exclude_spec = self._build_pathspec(t_exc)
        
        # Initialize Tokenizer if available
        self.encoder = None
        if HAS_TIKTOKEN:
            try:
                self.encoder = tiktoken.get_encoding("cl100k_base")
            except Exception:
                pass

    def _build_pathspec(self, patterns_str):
        """Compile a comma-separated string of patterns into a PathSpec object."""
        if not patterns_str:
            return None
        patterns = [p.strip() for p in patterns_str.split(",") if p.strip()]
        return pathspec.PathSpec.from_lines('gitwildmatch', patterns)

    def _parse_args(self):
        """Parse CLI arguments with default values loaded from the local config file."""
        defaults = {}
        if os.path.exists(CONFIG_FILE):
            try:
                with open(CONFIG_FILE, "r") as f:
                    defaults = json.load(f)
            except: 
                pass

        parser = argparse.ArgumentParser(
            description=BANNER, 
            formatter_class=argparse.RawDescriptionHelpFormatter
        )
        
        # Core Parameters
        parser.add_argument("path", nargs="?", default=defaults.get("path", "."), help="Target project path")
        parser.add_argument("-o", "--output", default=defaults.get("output", "allCode.txt"), help="Output filename")
        
        # File Content Filters
        parser.add_argument("-i", "--include", default=defaults.get("include"), help="Include patterns (e.g., *.py,src/**/*.js)")
        parser.add_argument("-x", "--exclude", default=defaults.get("exclude"), help="Exclude patterns (e.g., venv/,**/*.log)")
        parser.add_argument("--no-ignore", action="store_true", help="Ignore .gitignore rules for files")
        
        # Project Tree Filters (Defaults to None to enable Smart Isolation detection)
        parser.add_argument("-ti", "--tree-include", default=None, help="Tree include patterns (independent mode)")
        parser.add_argument("-tx", "--tree-exclude", default=None, help="Tree exclude patterns (independent mode)")
        parser.add_argument("--tree-no-ignore", action="store_true", help="Ignore .gitignore rules for tree")

        # Output Modifiers & Flags
        parser.add_argument("-m", "--marker", default=defaults.get("marker", "file"), help="Custom XML tag name (default: file)")
        parser.add_argument("--no-tree", action="store_true", default=defaults.get("no_tree", False), help="Disable project structure tree")
        parser.add_argument("--max-size", type=int, default=defaults.get("max_size", 500), help="Max file size (KB)")
        parser.add_argument("--dry-run", action="store_true", help="Preview list without writing to disk")
        parser.add_argument("-c", "--copy", action="store_true", help="Copy output to clipboard")
        parser.add_argument("-t", "--tokens", action="store_true", help="Calculate token count")
        parser.add_argument("--save", action="store_true", help="Save current arguments as default config")
        parser.add_argument("-v", "--version", action="version", version=f"OneSource {VERSION}")

        args = parser.parse_args()

        # Save configuration logic
        if args.save:
            config_to_save = {
                "output": args.output,
                "include": args.include,
                "exclude": args.exclude,
                "no_ignore": args.no_ignore,
                "tree_include": args.tree_include, 
                "tree_exclude": args.tree_exclude,
                "tree_no_ignore": args.tree_no_ignore, 
                "marker": args.marker,
                "no_tree": args.no_tree,
                "max_size": args.max_size
            }
            with open(CONFIG_FILE, "w") as f:
                json.dump(config_to_save, f, indent=4)
            print(f"[*] Configuration saved to {CONFIG_FILE}")

        return args

    def _load_gitignore(self, disable_flag):
        """Load and parse the .gitignore file if present and not disabled."""
        if disable_flag: 
            return None
        gi = self.root / ".gitignore"
        if gi.exists():
            try:
                content = gi.read_text(encoding="utf-8", errors="ignore")
                return pathspec.PathSpec.from_lines('gitwildmatch', content.splitlines())
            except Exception as e:
                print(f"  ! Warning: Failed to read .gitignore: {e}")
                return None
        return None

    def _is_binary(self, path: Path):
        """Detect if a file is binary by attempting to read a chunk as UTF-8."""
        try:
            with open(path, 'r', encoding='utf-8') as f:
                f.read(1024)
                return False
        except UnicodeDecodeError:
            return True 
        except Exception: 
            return True 

    def _should_ignore_file(self, path: Path):
        """
        Filtering logic for FILE CONTENT. 
        Includes strict checks for symlinks, pathspec matches, file size, and binary detection.
        """
        # Base exclusions: symlinks, .git directory, and the output file itself
        if path.is_symlink() or ".git" in path.parts or path == self.output_path: 
            return True
        
        # Normalize paths for gitwildmatch compatibility
        rel_path = str(path.relative_to(self.root)).replace('\\', '/')
        
        # Check against .gitignore and custom exclude patterns
        if self.spec and self.spec.match_file(rel_path): 
            return True
        if self.exclude_spec and self.exclude_spec.match_file(rel_path):
            return True

        # File-specific checks (include pattern, size limit, binary detection)
        if path.is_file():
            if self.include_spec and not self.include_spec.match_file(rel_path):
                return True
            if path.stat().st_size > self.args.max_size * 1024 or self._is_binary(path): 
                return True
                
        return False

    def _should_ignore_tree(self, path: Path):
        """
        Filtering logic for the PROJECT TREE.
        Pure path matching. Ignores file content, size limits, and binary checks 
        so visual structure remains intact.
        """
        # Base exclusions
        if path.is_symlink() or ".git" in path.parts or path == self.output_path: 
            return True
        
        rel_path = str(path.relative_to(self.root)).replace('\\', '/')
        
        # Check against tree-specific exclude patterns
        if self.tree_spec and self.tree_spec.match_file(rel_path): 
            return True
        if self.tree_exclude_spec and self.tree_exclude_spec.match_file(rel_path):
            return True

        # Check against tree-specific include patterns
        if path.is_file():
            if self.tree_include_spec and not self.tree_include_spec.match_file(rel_path):
                return True
                
        return False

    def _generate_tree(self, dir_path, prefix=""):
        """Recursively generate a visual tree structure string."""
        tree_str = ""
        try:
            # Filter entries using tree-specific logic
            entries = sorted([e for e in dir_path.iterdir() if not self._should_ignore_tree(e)], 
                            key=lambda x: (x.is_file(), x.name))
        except PermissionError:
            return f"{prefix}[Permission Denied]\n"

        for i, entry in enumerate(entries):
            is_last = (i == len(entries) - 1)
            connector = "\\-- " if is_last else "|-- "
            tree_str += f"{prefix}{connector}{entry.name}\n"
            if entry.is_dir():
                tree_str += self._generate_tree(entry, prefix + ("    " if is_last else "|   "))
        return tree_str

    def run(self):
        """Main execution pipeline."""
        mode_label = "[DRY RUN]" if self.args.dry_run else "[PROCESSING]"
        print(f"{mode_label} Root: {self.root}")

        # Gather files for content aggregation using file-specific logic
        valid_files = [p for p in self.root.rglob("*") if p.is_file() and not self._should_ignore_file(p)]
        
        # Generate project tree (optional)
        project_tree = None
        if not self.args.no_tree:
            project_tree = f"{self.root.name}/\n{self._generate_tree(self.root)}"
            print("\nProject Structure Preview:")
            print("-" * 20)
            print(project_tree)
            print("-" * 20 + "\n")

        total_tokens = 0
        out_file = None
        
        # File Writing & Token Calculation
        if not self.args.dry_run:
            out_file = open(self.output_path, "w", encoding="utf-8")
            if project_tree:
                out_file.write(f"<project_structure>\n{project_tree}</project_structure>\n\n")

        marker = self.args.marker
        for p in valid_files:
            rel_path = str(p.relative_to(self.root)).replace('\\', '/')
            try:
                # Read with error replacement to prevent decoding crashes
                content = p.read_text(encoding="utf-8", errors="replace")
                
                if self.args.tokens and self.encoder:
                    total_tokens += len(self.encoder.encode(content))
                
                if out_file:
                    out_file.write(f'<{marker} path="{rel_path}">\n{content}\n</{marker}>\n\n')
                
                print(f"  + {rel_path}")
            except Exception as e:
                print(f"  ! Error reading {rel_path}: {e}")

        if out_file: 
            out_file.close()

        # Output Summary
        print("\n" + "="*40)
        print(f"Files Processed: {len(valid_files)}")
        if self.args.tokens:
            token_str = f"{total_tokens:,}" if self.encoder else "tiktoken error"
            print(f"Total Tokens:    {token_str}")
        
        if not self.args.dry_run:
            print(f"Output saved to: {self.output_path}")
            if self.args.copy:
                try:
                    pyperclip.copy(self.output_path.read_text(encoding="utf-8"))
                    print("Copied to clipboard.")
                except Exception as e:
                    print(f"Clipboard error: {e}")
        print("="*40)

def main():
    OneSource().run()

if __name__ == "__main__":
    main()