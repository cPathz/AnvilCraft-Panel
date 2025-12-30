# AnvilCraft Panel

![Status](https://img.shields.io/badge/Status-Beta-purple?style=flat-square)
![Version](https://img.shields.io/badge/Version-v0.1.1-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

## 💡 Note from the Author

> **Disclaimer:** This project was developed with the assistance of **Google Antigravity** code editor and **Google Gemini** for code generation. I mention this upfront for those who may have reservations about AI-assisted development.

This is a personal project born from the need to move away from using `.bat` files to start my Minecraft servers on Windows. My goal was to have all instances centralized and managed in a single, unified interface. Identifying a gap in existing solutions for my specific needs, I decided to build it myself. As a **Computer Engineer** (though not a professional programmer), I leveraged AI tools to bring this software to life.

I am completely open to feedback regarding **design, security, code structure, or functionality**. If you find this tool useful for your own personal use, please feel free to use it!

**AnvilCraft Panel** is a modern, high-performance Minecraft server manager built for power users and administrators. It provides a sleek, "premium" interface to create, manage, and monitor multiple Minecraft instances with ease.

Built with **Tauri v2** (Rust) and **SvelteKit**, AnvilCraft combines the performance of native code with the flexibility of modern web technologies.

## 🚀 Key Features

*   **Instance Management:** Create, start, stop, and kill server instances instantly.
*   **Multi-Version Support:**
    *   **Vanilla:** Download any version from the official manifest.
    *   **Forge:** Automated installer verification and execution.
    *   **Paper/Purpur:** (Planned/In-Progress) Support for optimized server software.
*   **Live Console:**
    *   Real-time stdout/stderr streaming.
    *   ANSI color support for readable logs.
    *   Command history and autocomplete support.
*   **Advanced Configuration:**
    *   Per-instance Java version selection.
    *   Configurable RAM allocation (Default 4GB).
    *   Automatic EULA management.
*   **Modern UI/UX:**
    *   Dark mode by default with glassmorphism effects.
    *   Responsive and fluid animations.
    *   No external CMD windows – everything is integrated.

## 🛠️ Tech Stack

*   **Frontend:** [SvelteKit](https://kit.svelte.dev/) + [TypeScript](https://www.typescriptlang.org/)
*   **Styling:** [TailwindCSS](https://tailwindcss.com/)
*   **Backend:** [Rust](https://www.rust-lang.org/) (via [Tauri](https://tauri.app/))
*   **State Management:** Svelte 5 Runes
*   **Build Tool:** Vite

## 📦 Installation

### Windows (MSI)
Download the latest `.msi` installer from the Releases section.
> **Note:** The installer is currently unsigned (Beta). You may need to verify the SHA-256 hash or bypass Windows SmartScreen initially.

### Manual Build / Development

Prerequisites:
- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (C++ workoad)

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/cPathz/anvil-craft-panel.git
    cd anvil-craft-panel
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Run in Development Mode:**
    ```bash
    npm run tauri dev
    ```

4.  **Build Release:**
    ```bash
    npm run tauri build
    ```

## 🛡️ Security

*   **CSP:** Strict Content Security Policy applied.
*   **Permissions:** Minimal capability set (ACL) configured in `default.json`.
*   **Isolation:** Instances run in isolated processes.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
**Developed by Luis Macias (cPathz)**
