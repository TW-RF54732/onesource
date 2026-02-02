# OneSource ⚡

> **為AI打造的本地端專題打包工具**
>
> 🚫 **不用再依賴於臃腫的Node.js生態** 不用 `npm install`. 不用上傳任何檔案.   
> 🚀 **下載後開箱及用** (或用 `pip install` 讓python幫你自動下載).

[![PyPI version](https://img.shields.io/pypi/v/onesource-cli.svg)](https://pypi.org/project/onesource-cli/)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Roadmap: Rust Rewrite](https://img.shields.io/badge/Roadmap-Rust%20Rewrite-orange?logo=rust)

**OneSource**能幫你把零散的專案內容打包成一個檔案，並且保留完整的檔案結構。專門為大語言模型(LLM)像是: Claude, ChatGPT, Gemini等設計。

這款工具**專為 Windows** 用戶與 **Python 開發者**量身打造，提供完善的一鍵安裝程式與**跨平台支援**；在主打**零上手成本**、讓新手享受極致簡單體驗的同時，更具備強大的自定義能力，確保簡單卻不失專業深度。
![OneSource Demo](OneSource_demo.gif)

## 📢 重要公告:
本專案正在使用Rust重寫，目前已完成基礎核心功能的alpha版本，如果想搶先試用，請到 `rust-dev`分支下載並編譯。
新版本將帶來:

- 百倍提升的執行速度，幾乎瞬間掃描大型專案
- 乾淨無依賴，不用擔心衝突且輕量
- 記憶體安全，確保深度掃描目錄期間的穩定性
- 多核心優化，解放CPU的所有效能
---
##　為何選擇 onesource? 比較與定位
> 目標不是做出最萬能做複雜的瑞士刀
> 而是做一個專精且好用，零上手成本的美工刀
> 
| 特性 | **本工具** | **Node.js 工具** | **網頁版 (Web)** | **Rust 執行檔** |
| --- | --- | --- | --- | --- |
| **檔案複雜度** | ✅ **低** (單一 EXE 檔) | ❌ 高 (需安裝 Node.js) | ✅ 低 (瀏覽器開啟) | ❌ 高 (需下載二進位檔) |
| **本地隱私** | ✅ **100% 本地運行** | ✅ 本地運行 | ❌ **雲端處理** | ✅ 本地運行 |
| **系統殘留** | ✅ **零** (刪除即清空) | ❌ 產生 `node_modules` | ✅ 無 | ✅ 無 |
| **部署便利性** | ✅ 極速 (一行指令安裝) | ❌ 繁瑣 (需透過 NPM) | ✅ 極速 (即開即用) | ❌ 需手動從 Rust 編譯 |
| **剪貼簿複製** | ✅ **內建功能** | ✅ 支援 | ❌ 需手動複製 | ✅ 支援 |
## 下載與部屬
