# 🚀 Game Accelerator

<p align="center">
  <strong>A lightweight, open-source game booster for Windows, written in Rust. Single-file, zero-dependency, ready to run.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg" alt="Platform"></a>
  <a href="#"><img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust"></a>
  <a href="https://github.com/zhang-forever/Game-Accelerator/actions/workflows/ci.yml"><img src="https://github.com/zhang-forever/Game-Accelerator/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/zhang-forever/Game-Accelerator/releases"><img src="https://img.shields.io/github/v/release/zhang-forever/Game-Accelerator?include_prereleases" alt="Release"></a>
  <a href="https://github.com/zhang-forever/Game-Accelerator/releases"><img src="https://img.shields.io/github/downloads/zhang-forever/Game-Accelerator/total" alt="Downloads"></a>
</p>

<p align="center">
  中文文档：<a href="README.md">README.md</a>
</p>

---

## ⚠️ Important Safety Notice (Read Before Use)

> **This tool modifies system settings, terminates processes, and adjusts process priorities. Understand the risks before using it.**

- **Anti-cheat / ban risk**: The "memory cleanup" and "boost game priority" features operate on the running game's process. In competitive games with strict anti-cheat systems (e.g. **VALORANT (Vanguard)**, **League of Legends**, **CS2**, **Apex Legends**), such external process manipulation **may be flagged as suspicious and can result in being kicked from a match or even a ban**.
  - **Recommendation**: For these games, run the boost **before launching the game** (clean memory, close background apps, apply system tweaks), then start the game. **Do not** click boost while an anti-cheat-protected game is already running.
  - You use this tool at your own risk. The author is not responsible for any bans or account loss.
- **Administrator privileges**: This tool requires admin rights to modify power plans, registry keys, and system services. A UAC prompt appears automatically when you run it.
- **System stability**: A protected-process whitelist prevents killing critical system processes. However, some "system optimization" actions (e.g. pausing disk indexing) change system behavior — use them as needed.

---

## ✨ Features

| Module | What it does | Persists after closing? |
|:------:|--------------|:--:|
| 🚀 **One-click Boost** | Clean memory, close background processes, switch to high-performance power, boost game priority | Memory/processes: yes; priority: resets on game restart |
| ⚙️ **System Optimization** | High-performance power, Windows Game Mode, Hardware-accelerated GPU Scheduling, Xbox Game Bar toggle, pause disk indexing | ✅ Permanent (system settings) |
| 📋 **Process Management** | Batch-close background apps by category, or fine-grained control in advanced mode; hide small processes (<50MB) | ✅ |
| 🎮 **GPU Settings** | NVIDIA max-performance mode, disable background telemetry, force games to use the discrete GPU | ✅ |
| 📊 **Live Monitoring** | CPU (per-core), RAM, GPU usage & temperature, process count | — |

### Additional Highlights

- 🖥️ **Cyberpunk Dark Theme**: A sleek, eye-friendly UI with a high-tech aesthetic
- 🔋 **Smart Power Plan Switching**: Automatically selects Ultimate Performance or High Performance mode
- 🛡️ **Safety Mechanisms**: Built-in system-critical process whitelist to prevent accidental system crashes
- 📂 **Process Categorization**: Auto-classifies browsers, chat apps, office tools, cloud sync, and more for one-click batch closure
- 🎯 **Priority Boosting**: Automatically detects game processes and promotes them to high priority
- 💾 **Persistent Config**: All settings are saved to a TOML config file and restored on next launch

---

## 📥 Installation & Usage

### Option 1: Download the installer (recommended)

1. Go to the [Releases](../../releases) page
2. Download the latest `GameAccelerator-x.x.x.msi`
3. Double-click to install, then launch from the Start Menu
4. Click "Yes" on the UAC prompt on first run

### Option 2: Download the portable exe

1. Download `game-accelerator.exe` from [Releases](../../releases)
2. Double-click to run (no installation needed)

### How to use

1. **First time**: Toggle the switches you want on the "System Optimization" page (set once, stays applied)
2. **Before each gaming session**: Open the app → click "Boost" → close the app → launch your game
3. No need to keep it running in the background; only keep it open if you want live monitoring

> 💡 **Game path**: Just enter the game's EXE name in Settings — no full path needed. For example, VALORANT is `VALORANT-Win64-Shipping.exe`.

---

## 🛠️ Building from Source

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (MSVC)
- Windows 10/11
- **Recommended**: Visual Studio Build Tools or full Visual Studio

### Quick Start

```bash
# Clone
git clone https://github.com/zhang-forever/Game-Accelerator.git
cd Game-Accelerator

# Debug build
cargo build

# Release build (size-optimized)
cargo build --release
# Output: target/release/game-accelerator.exe
```

### Building the MSI installer (optional)

```bash
cargo install cargo-wix
cargo wix
# Output: target/wix/GameAccelerator-x.x.x.msi
```

### Build Optimization Parameters

The release build uses these optimization flags to minimize binary size:

| Parameter | Value | Description |
|-----------|-------|-------------|
| `opt-level` | `"z"` | Minimize binary size |
| `lto` | `true` | Link-Time Optimization for further compression |
| `codegen-units` | `1` | Single compilation unit for better optimization |
| `strip` | `true` | Remove debug symbols |
| `panic` | `"abort"` | Use abort instead of unwind to reduce size |

---

## 🧱 Tech Stack

| Component | Technology | Description |
|-----------|------------|-------------|
| **Language** | Rust 2021 | Safe, high-performance systems programming |
| **GUI** | [egui](https://github.com/emilk/egui) / eframe 0.29 | Immediate-mode GUI, lightweight and efficient |
| **System Info** | sysinfo 0.31 | Cross-platform system metrics collection |
| **Windows API** | windows-sys 0.59 | Low-level Windows API bindings |
| **Config** | serde + toml | TOML configuration file support |
| **Concurrency** | parking_lot | High-performance mutex implementation |

### Project Architecture

```
src/
├── main.rs           # Entry point, font/theme initialization
├── app.rs            # Main app struct, page routing
├── config/           # Configuration management (TOML persistence)
├── core/             # Core acceleration logic
│   ├── elevation.rs       # UAC elevation management
│   ├── memory_cleaner.rs  # Memory cleanup
│   ├── process_manager.rs # Process management
│   ├── cpu_optimizer.rs   # CPU optimization
│   ├── power_manager.rs   # Power plan management
│   ├── game_mode.rs       # Windows Game Mode
│   ├── gpu_manager.rs     # GPU settings
│   ├── disk_optimizer.rs  # Disk optimization
│   └── process_category.rs # Process classification
├── monitor/          # System monitoring (background thread)
└── ui/               # UI modules
    ├── dashboard.rs       # Dashboard
    ├── process_page.rs    # Process management page
    ├── gpu_page.rs        # GPU settings page
    ├── system_opt_page.rs # System optimization page
    ├── settings_page.rs   # Settings page
    ├── theme.rs           # Theme constants
    └── widgets.rs         # Shared UI widgets
```

---

## 🤝 Contributing

Issues and PRs are welcome. Before submitting, please make sure:

```bash
cargo build
cargo clippy
```

### Development Guide

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

---

## 📄 License

[MIT](LICENSE) © 2026 mi

---

<p align="center">
  <sub>Made with ❤️ and Rust</sub>
</p>
