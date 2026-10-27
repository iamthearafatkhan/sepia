<div align="center">
  <img src="public/logo.svg" alt="Sepia Logo" width="140" />
  <h1>Sepia — Your software, curated.</h1>
  <p><strong>A beautifully crafted, ultra-lightweight system utility and uninstaller for Windows.</strong></p>

  <a href="https://github.com/iamthearafatkhan/sepia/releases/latest">
    <img src="https://img.shields.io/badge/Download-Latest_Release-916B60?style=for-the-badge&logo=windows&logoColor=white" alt="Download Sepia" />
  </a>
  <a href="https://github.com/iamthearafatkhan/sepia/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-C1A094?style=for-the-badge" alt="MIT License" />
  </a>
</div>

<br />

**Sepia** is a modern, fast, and deeply integrated utility for Windows. Unlike traditional tools that only look at the Windows Registry, Sepia unifies all your software—including developer packages—into one stunning, warm-themed dashboard. 

Built with **Tauri (Rust + React)**, Sepia is incredibly lightweight (under 10 MB) and uses a fraction of the memory of other tools, while delivering a premium, lag-free experience. Whether you need to uninstall a heavy application or clean up gigabytes of hidden system caches, Sepia does it beautifully.

---

## ✨ Features

- 🎨 **Ultra Premium UI:** A calm, sophisticated dark theme based on warm sepia tones.
- 🧹 **Smart Cleaner (New!):** Safely scan and remove temporary files, Windows temp files, NPM caches, and Pip caches to instantly reclaim your hard drive space.
- 📦 **Multi-Ecosystem Support:** Uninstalls standard Windows apps, and developer packages installed via **Pip (Python)**, **NPM (Node.js)**, **Chocolatey**, and **Scoop**.
- ⚡ **Lightning Fast & Smooth:** Scans and size calculations run in the background, so the UI never freezes. Uninstalled apps disappear instantly from the list.
- 📏 **Accurate Disk Usage:** Calculates the exact folder size of your applications and Python packages so you know exactly what is eating your storage.
- 🛡️ **Smart Detection:** Automatically reads the Windows Registry, detects 32-bit/64-bit apps, and resolves missing folder paths using smart icon/uninstaller fallbacks.
- 🖥️ **Portable `.exe`:** No installation required. Download the file, double-click, and it runs instantly.

---
### ✨ New Features
*   **Smart Cleaner Tab:** A brand new interface to scan and clean temporary files.
*   **Deep Cache Cleaning:** Safely removes User Temp files, Windows Temp files, NPM Cache, and Pip (Python) Cache.
*   **One-Click Cleanup:** See exactly how much space each category is taking up and wipe it instantly.
*   **Tabbed Navigation:** Seamlessly switch between "Installed Software" and "Smart Cleaner".
---
### 🛠️ Improvements
*   Updated versioning to `v1.1.0`.
*   Further optimized background scanning so the UI never freezes, even when cleaning large caches.
*   Maintained the ultra-premium Sepia color palette and smooth modal confirmations.

---
## 🚀 How to Use

1. Go to the **[Latest Releases](https://github.com/iamthearafatkhan/sepia/releases/tag/v1.1)** page.
2. Download `sepia.exe`.
3. Double-click the `.exe` to open Sepia. 
4. Use the search bar to find any software or package.
5. Click **Uninstall** and confirm via the premium modal.

*(Note: Some Windows apps require Administrator privileges to uninstall. If a standard uninstaller prompts for UAC, simply accept it.)*

---

## 🛠️ Build from Source

If you are a developer and want to build Sepia yourself, ensure you have [Node.js](https://nodejs.org/) and [Rust](https://www.rust-lang.org/) installed.

```bash
# 1. Clone the repository
git clone https://github.com/iamthearafatkhan/sepia.git

# 2. Navigate into the directory
cd sepia

# 3. Install frontend dependencies
npm install

# 4. Run in development mode
npm run tauri dev

# 5. Build the final optimized .exe for production
npm run tauri build
```
The final .exe will be located at src-tauri/target/release/sepia.exe.

## 📄 License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---
<div align="center"> <i>Built with ❤️ by <a href="https://github.com/iamthearafatkhan">Arafat Khan</a></i> </div>
