#!/usr/bin/env python3
"""
版本管理和發布工具
自動更新版本號、創建 Release Notes、發布到 GitHub
"""

import os
import re
import json
import subprocess
import sys
from pathlib import Path
from typing import List, Dict, Optional
from datetime import datetime

try:
    from rich.console import Console
    from rich.prompt import Prompt, Confirm
    from rich.panel import Panel
    from rich.table import Table
    from rich.markdown import Markdown
    from rich.progress import Progress, SpinnerColumn, TextColumn
    import requests
except ImportError:
    print("正在安裝必要的依賴...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", 
                          "rich", "requests", "--break-system-packages", "-q"])
    from rich.console import Console
    from rich.prompt import Prompt, Confirm
    from rich.panel import Panel
    from rich.table import Table
    from rich.markdown import Markdown
    from rich.progress import Progress, SpinnerColumn, TextColumn
    import requests

console = Console()


class VersionManager:
    """版本管理器"""
    
    def __init__(self, repo_path: str = "."):
        self.repo_path = Path(repo_path).resolve()
        self.version_files = []
        self.current_version = None
        self.new_version = None
        
    def detect_version_files(self) -> List[Dict[str, str]]:
        """偵測專案中的版本文件"""
        patterns = {
            "package.json": r'"version"\s*:\s*"([^"]+)"',
            "pyproject.toml": r'version\s*=\s*"([^"]+)"',
            "setup.py": r'version\s*=\s*["\']([^"\']+)["\']',
            "Cargo.toml": r'version\s*=\s*"([^"]+)"',
            "__init__.py": r'__version__\s*=\s*["\']([^"\']+)["\']',
            "version.txt": r'^(.+)$',
            "VERSION": r'^(.+)$',
        }
        
        found_files = []
        
        for filename, pattern in patterns.items():
            for file_path in self.repo_path.rglob(filename):
                # 跳過 node_modules, .git 等目錄
                if any(p in file_path.parts for p in ['node_modules', '.git', 'venv', 'dist']):
                    continue
                    
                try:
                    content = file_path.read_text(encoding='utf-8')
                    match = re.search(pattern, content, re.MULTILINE)
                    if match:
                        version = match.group(1)
                        found_files.append({
                            'path': str(file_path.relative_to(self.repo_path)),
                            'version': version,
                            'pattern': pattern
                        })
                        
                        # 設定當前版本（使用第一個找到的）
                        if self.current_version is None:
                            self.current_version = version
                except Exception as e:
                    console.print(f"[yellow]警告: 無法讀取 {file_path}: {e}[/yellow]")
        
        self.version_files = found_files
        return found_files
    
    def update_version_in_file(self, file_info: Dict, new_version: str) -> bool:
        """更新單個文件中的版本號"""
        file_path = self.repo_path / file_info['path']
        
        try:
            content = file_path.read_text(encoding='utf-8')
            old_version = file_info['version']
            
            # 替換版本號
            new_content = re.sub(
                file_info['pattern'],
                lambda m: m.group(0).replace(old_version, new_version),
                content
            )
            
            file_path.write_text(new_content, encoding='utf-8')
            return True
        except Exception as e:
            console.print(f"[red]錯誤: 更新 {file_path} 失敗: {e}[/red]")
            return False
    
    def update_all_versions(self, new_version: str) -> bool:
        """更新所有檔案中的版本號"""
        self.new_version = new_version
        success = True
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console
        ) as progress:
            task = progress.add_task("更新版本號...", total=len(self.version_files))
            
            for file_info in self.version_files:
                if self.update_version_in_file(file_info, new_version):
                    progress.console.print(f"✓ 更新 {file_info['path']}")
                else:
                    success = False
                progress.advance(task)
        
        return success
    
    def create_release_notes(self, version: str, notes_dir: str = "release_notes") -> Path:
        """創建 Release Notes 文件"""
        notes_path = self.repo_path / notes_dir
        notes_path.mkdir(exist_ok=True)
        
        release_file = notes_path / f"{version}_release.md"
        
        # 創建模板
        template = f"""# Release {version}

發布日期: {datetime.now().strftime('%Y-%m-%d')}

## ✨ 新功能

- 

## 🐛 Bug 修復

- 

## 📝 改進

- 

## ⚠️ 重要變更

- 

## 📦 依賴更新

- 

---
*此版本由版本管理工具自動創建*
"""
        
        release_file.write_text(template, encoding='utf-8')
        return release_file


class GitHubReleaser:
    """GitHub 發布管理器"""
    
    def __init__(self, repo_path: str = "."):
        self.repo_path = Path(repo_path).resolve()
        self.token = None
        self.repo_owner = None
        self.repo_name = None
    
    def get_github_token(self) -> Optional[str]:
        """獲取 GitHub Token"""
        # 嘗試從環境變量獲取
        token = os.environ.get('GITHUB_TOKEN')
        if token:
            return token
        
        # 嘗試從 git config 獲取
        try:
            result = subprocess.run(
                ['git', 'config', '--get', 'github.token'],
                cwd=self.repo_path,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except:
            pass
        
        return None
    
    def get_repo_info(self) -> Optional[tuple]:
        """獲取倉庫信息（owner/repo）"""
        try:
            result = subprocess.run(
                ['git', 'remote', 'get-url', 'origin'],
                cwd=self.repo_path,
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                url = result.stdout.strip()
                # 解析 GitHub URL
                # git@github.com:owner/repo.git 或 https://github.com/owner/repo.git
                match = re.search(r'github\.com[:/]([^/]+)/([^/\.]+)', url)
                if match:
                    return match.group(1), match.group(2)
        except:
            pass
        
        return None
    
    def commit_and_push(self, version: str, files: List[str]) -> bool:
        """提交更改並推送"""
        try:
            # Git add
            subprocess.run(['git', 'add'] + files, cwd=self.repo_path, check=True)
            
            # Git commit
            subprocess.run(
                ['git', 'commit', '-m', f'chore: bump version to {version}'],
                cwd=self.repo_path,
                check=True
            )
            
            # Git push
            subprocess.run(['git', 'push'], cwd=self.repo_path, check=True)
            
            return True
        except subprocess.CalledProcessError as e:
            console.print(f"[red]Git 操作失敗: {e}[/red]")
            return False
    
    def create_tag(self, version: str) -> bool:
        """創建並推送 Git Tag"""
        tag = f"v{version}" if not version.startswith('v') else version
        
        try:
            # 創建 tag
            subprocess.run(
                ['git', 'tag', '-a', tag, '-m', f'Release {version}'],
                cwd=self.repo_path,
                check=True
            )
            
            # 推送 tag
            subprocess.run(['git', 'push', 'origin', tag], cwd=self.repo_path, check=True)
            
            return True
        except subprocess.CalledProcessError as e:
            console.print(f"[red]Tag 創建失敗: {e}[/red]")
            return False
    
    def create_github_release(self, version: str, release_notes: str) -> bool:
        """使用 GitHub API 創建 Release"""
        if not self.token:
            console.print("[red]錯誤: 未找到 GitHub Token[/red]")
            return False
        
        if not self.repo_owner or not self.repo_name:
            console.print("[red]錯誤: 無法獲取倉庫信息[/red]")
            return False
        
        tag = f"v{version}" if not version.startswith('v') else version
        
        url = f"https://api.github.com/repos/{self.repo_owner}/{self.repo_name}/releases"
        
        headers = {
            'Authorization': f'token {self.token}',
            'Accept': 'application/vnd.github.v3+json'
        }
        
        data = {
            'tag_name': tag,
            'name': f'Release {version}',
            'body': release_notes,
            'draft': False,
            'prerelease': False
        }
        
        try:
            response = requests.post(url, headers=headers, json=data)
            response.raise_for_status()
            
            release_url = response.json()['html_url']
            console.print(f"[green]✓ Release 創建成功: {release_url}[/green]")
            return True
        except requests.exceptions.RequestException as e:
            console.print(f"[red]GitHub API 錯誤: {e}[/red]")
            if hasattr(e, 'response') and e.response is not None:
                console.print(f"[red]回應: {e.response.text}[/red]")
            return False


def show_banner():
    """顯示歡迎橫幅"""
    banner = """
╔══════════════════════════════════════════╗
║     🚀 版本管理與發布工具 v1.0.0        ║
║     自動化版本更新和 GitHub 發布         ║
╚══════════════════════════════════════════╝
"""
    console.print(Panel(banner, style="bold cyan"))


def main():
    """主程序"""
    show_banner()
    
    # 初始化
    vm = VersionManager()
    gh = GitHubReleaser()
    
    # 1. 偵測版本文件
    console.print("\n[bold]📋 步驟 1: 偵測版本文件[/bold]")
    version_files = vm.detect_version_files()
    
    if not version_files:
        console.print("[red]❌ 未找到任何版本文件！[/red]")
        return
    
    # 顯示找到的文件
    table = Table(title="找到的版本文件")
    table.add_column("文件", style="cyan")
    table.add_column("當前版本", style="green")
    
    for f in version_files:
        table.add_row(f['path'], f['version'])
    
    console.print(table)
    console.print(f"\n[bold green]當前版本: {vm.current_version}[/bold green]")
    
    # 2. 輸入新版本
    console.print("\n[bold]📝 步驟 2: 設定新版本[/bold]")
    
    # 建議版本號
    if vm.current_version:
        parts = vm.current_version.split('.')
        if len(parts) == 3:
            major, minor, patch = parts
            suggestions = {
                '1': f"{major}.{minor}.{int(patch)+1}",  # Patch
                '2': f"{major}.{int(minor)+1}.0",        # Minor
                '3': f"{int(major)+1}.0.0",              # Major
            }
            
            console.print("[cyan]建議的版本號:[/cyan]")
            console.print(f"  1. Patch: {suggestions['1']}")
            console.print(f"  2. Minor: {suggestions['2']}")
            console.print(f"  3. Major: {suggestions['3']}")
            console.print("  4. 自訂版本號")
            
            choice = Prompt.ask("選擇", choices=['1', '2', '3', '4'], default='1')
            
            if choice in ['1', '2', '3']:
                new_version = suggestions[choice]
            else:
                new_version = Prompt.ask("請輸入新版本號")
        else:
            new_version = Prompt.ask("請輸入新版本號")
    else:
        new_version = Prompt.ask("請輸入新版本號", default="1.0.0")
    
    console.print(f"[bold green]新版本: {new_version}[/bold green]")
    
    # 3. 更新版本號
    console.print("\n[bold]🔄 步驟 3: 更新版本號[/bold]")
    if not Confirm.ask("確定要更新所有文件中的版本號嗎？"):
        console.print("[yellow]已取消[/yellow]")
        return
    
    if not vm.update_all_versions(new_version):
        console.print("[red]版本更新失敗！[/red]")
        return
    
    console.print("[green]✓ 版本號更新完成[/green]")
    
    # 4. 創建 Release Notes
    console.print("\n[bold]📄 步驟 4: 創建 Release Notes[/bold]")
    release_file = vm.create_release_notes(new_version)
    console.print(f"[green]✓ 創建 Release Notes: {release_file}[/green]")
    
    # 提示編輯
    console.print("\n[yellow]請編輯 Release Notes 文件，完成後按 Enter 繼續...[/yellow]")
    console.print(f"[cyan]文件位置: {release_file}[/cyan]")
    
    # 嘗試自動打開編輯器
    editor = os.environ.get('EDITOR', 'nano')
    try:
        subprocess.run([editor, str(release_file)])
    except:
        input("按 Enter 繼續...")
    
    # 讀取 Release Notes
    release_notes = release_file.read_text(encoding='utf-8')
    
    # 5. Git 操作
    console.print("\n[bold]📦 步驟 5: Git 提交和標籤[/bold]")
    
    # 獲取要提交的文件
    files_to_commit = [f['path'] for f in version_files]
    files_to_commit.append(str(release_file.relative_to(vm.repo_path)))
    
    console.print(f"[cyan]準備提交的文件:[/cyan]")
    for f in files_to_commit:
        console.print(f"  • {f}")
    
    if not Confirm.ask("\n提交更改到 Git？"):
        console.print("[yellow]跳過 Git 提交[/yellow]")
    else:
        if gh.commit_and_push(new_version, files_to_commit):
            console.print("[green]✓ Git 提交和推送完成[/green]")
        else:
            console.print("[red]Git 操作失敗[/red]")
            return
    
    # 6. 創建 Tag
    if Confirm.ask("\n創建 Git Tag？", default=True):
        if gh.create_tag(new_version):
            console.print("[green]✓ Tag 創建和推送完成[/green]")
        else:
            console.print("[red]Tag 創建失敗[/red]")
    
    # 7. GitHub Release
    console.print("\n[bold]🚀 步驟 6: 發布到 GitHub[/bold]")
    
    # 獲取 GitHub 配置
    gh.token = gh.get_github_token()
    repo_info = gh.get_repo_info()
    
    if repo_info:
        gh.repo_owner, gh.repo_name = repo_info
        console.print(f"[cyan]倉庫: {gh.repo_owner}/{gh.repo_name}[/cyan]")
    
    if not gh.token:
        console.print("[yellow]未找到 GitHub Token[/yellow]")
        console.print("[cyan]提示: 設定環境變量 GITHUB_TOKEN 或使用 git config github.token[/cyan]")
        
        if Confirm.ask("手動輸入 Token？"):
            gh.token = Prompt.ask("GitHub Token", password=True)
    
    if gh.token and Confirm.ask("\n創建 GitHub Release？", default=True):
        if gh.create_github_release(new_version, release_notes):
            console.print("[bold green]🎉 發布完成！[/bold green]")
        else:
            console.print("[yellow]GitHub Release 創建失敗，但本地更改已完成[/yellow]")
    else:
        console.print("[yellow]跳過 GitHub Release[/yellow]")
    
    # 完成
    console.print("\n" + "="*50)
    console.print(Panel(
        f"[bold green]✓ 版本 {new_version} 發布流程完成！[/bold green]",
        style="green"
    ))


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        console.print("\n[yellow]已取消操作[/yellow]")
    except Exception as e:
        console.print(f"\n[red]錯誤: {e}[/red]")
        import traceback
        console.print(traceback.format_exc())