# 🚀 Game Accelerator

> A lightweight, open-source game booster for Windows, written in Rust. Single-file, zero-dependency, ready to run.

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg)]()
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)]()

中文文档：[README.md](README.md)

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
|--------|--------------|:--:|
| **One-click Boost** | Clean memory, close background processes, switch to high-performance power, boost game priority | Memory/processes: yes; priority: resets on game restart |
| **System Optimization** | High-performance power, Windows Game Mode, Hardware-accelerated GPU Scheduling, Xbox Game Bar toggle, pause disk indexing | ✅ Permanent (system settings) |
| **Process Management** | Batch-close background apps by category, or fine-grained control in advanced mode; hide small processes (<50MB) | ✅ |
| **GPU Settings** | NVIDIA max-performance mode, disable background telemetry, force games to use the discrete GPU | ✅ |
| **Live Monitoring** | CPU (per-core), RAM, GPU usage & temperature, process count | — |

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

Requires the [Rust toolchain](https://rustup.rs/) (MSVC).

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

---

## 🧱 Tech Stack

- **Language**: Rust 2021
- **GUI**: [egui](https://github.com/emilk/egui) / eframe (immediate-mode GUI)
- **System info**: sysinfo
- **Windows API**: windows-sys (`EmptyWorkingSet`, `SetPriorityClass`, etc.)
- **Highlights**: single-file executable, no runtime dependencies

---

## 🤝 Contributing

Issues and PRs are welcome. Please make sure `cargo build` passes before submitting.

## 📄 License

[MIT](LICENSE) © 2026 mi
