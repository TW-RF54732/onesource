import PyInstaller.__main__

if __name__ == "__main__":
    PyInstaller.__main__.run([
        'onesource/main.py', 
        '--name=OneSource',
        '--onefile',
        '--console',
        '--clean',
        '--collect-all=tiktoken',
        '--collect-all=pathspec',
        '--noconfirm',
    ])