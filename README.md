# 🚀 Game Accelerator · 游戏加速器

<p align="center">
  <strong>轻量、开源的 Windows 游戏加速工具，用 Rust 编写。单文件、零依赖、即开即用。</strong>
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
  English version: <a href="README.en.md">README.en.md</a>
</p>

---

## ⚠️ 重要安全提示（使用前必读）

> **本工具会修改系统设置、结束进程、调整进程优先级。请理解风险后再使用。**

- **反作弊封号风险**：本工具的"内存清理"和"提升游戏优先级"功能会对游戏进程进行操作。在带有严格反作弊系统的竞技游戏中（如**无畏契约 / VALORANT（Vanguard）**、**英雄联盟**、**CS2**、**APEX** 等），这类外部程序操作**有被判定为可疑行为、导致踢出对局甚至封号的风险**。
  - **建议**：对这类游戏，请在**游戏启动前**完成加速（清理内存、关闭后台、设置系统优化），然后再启动游戏；**不要**对正在运行的反作弊游戏点击加速。
  - 你需要自行承担使用风险，作者不对任何封号或账号损失负责。
- **管理员权限**：本工具需要管理员权限才能修改电源计划、注册表、系统服务。双击运行时会自动弹出 UAC 授权框。
- **系统稳定性**：本工具内置受保护进程白名单，不会结束系统关键进程。但"系统优化"中的部分操作（如关闭磁盘索引）会改变系统行为，请按需使用。

---

## ✨ 功能特性

| 模块 | 功能 | 关闭软件后是否持续有效 |
|:---:|------|:--:|
| 🚀 **一键加速** | 清理内存、关闭后台进程、切换高性能电源、提升游戏优先级 | 内存/进程：是；优先级：游戏重启后失效 |
| ⚙️ **系统优化** | 高性能电源模式、Windows 游戏模式、硬件加速 GPU 调度、Xbox Game Bar 开关、暂停磁盘索引 | ✅ 永久有效（改系统设置） |
| 📋 **进程管理** | 按类别批量关闭后台程序，或在高级模式下精细管理；可隐藏小进程（<50MB） | ✅ |
| 🎮 **GPU 设置** | NVIDIA 最大性能模式、关闭后台遥测、强制游戏使用独立显卡 | ✅ |
| 📊 **实时监控** | CPU（各核心）、内存、GPU 使用率与温度、进程数 | — |

### 更多功能亮点

- 🖥️ **赛博朋克暗色主题**：精致的 UI 界面，护眼的同时充满科技感
- 🔋 **电源计划智能切换**：自动选择 Ultimate Performance 或 High Performance 模式
- 🛡️ **安全保护机制**：内置系统关键进程白名单，防止误杀系统进程
- 📂 **进程分类管理**：自动识别浏览器、聊天、办公、云同步等类别，一键批量关闭
- 🎯 **进程优先级提升**：自动识别游戏进程并提升至高优先级
- 💾 **配置持久化**：所有设置自动保存到 TOML 配置文件，下次启动自动恢复

---

## 📥 安装与使用

### 方式一：下载安装包（推荐普通用户）

1. 前往 [Releases](../../releases) 页面
2. 下载最新的 `GameAccelerator-x.x.x.msi`
3. 双击安装，从开始菜单启动
4. 首次运行点击 UAC 授权框中的"是"

### 方式二：下载绿色版 exe

1. 在 [Releases](../../releases) 下载 `game-accelerator.exe`
2. 双击运行即可（无需安装）

### 使用流程

1. **首次**：在「系统优化」页打开需要的开关（设一次即永久生效）
2. **每次玩游戏前**：打开软件 → 点「启动加速」→ 关掉软件 → 启动游戏
3. 不需要常驻后台；只有想看实时监控时才需保持开启

> 💡 **关于游戏路径**：在「设置」中填游戏 EXE 名即可，无需完整路径。例如无畏契约填 `VALORANT-Win64-Shipping.exe`。

---

## 🛠️ 从源码构建

### 环境要求

- [Rust 工具链](https://rustup.rs/)（MSVC 工具集）
- Windows 10/11
- **推荐**：Visual Studio Build Tools 或完整的 Visual Studio

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/zhang-forever/Game-Accelerator.git
cd Game-Accelerator

# 开发构建
cargo build

# 发布构建（体积优化）
cargo build --release
# 产物：target/release/game-accelerator.exe
```

### 构建 MSI 安装包（可选）

```bash
cargo install cargo-wix
cargo wix
# 产物：target/wix/GameAccelerator-x.x.x.msi
```

### 构建参数说明

发布构建使用以下优化参数以最小化二进制体积：

| 参数 | 值 | 说明 |
|------|-----|------|
| `opt-level` | `"z"` | 最小化体积优化 |
| `lto` | `true` | 链接时优化，进一步压缩体积 |
| `codegen-units` | `1` | 单编译单元，更好的优化 |
| `strip` | `true` | 去除调试符号 |
| `panic` | `"abort"` | 使用 abort 而非 unwind，减少体积 |

---

## 🧱 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| **语言** | Rust 2021 | 安全、高性能的系统编程语言 |
| **GUI** | [egui](https://github.com/emilk/egui) / eframe 0.29 | 即时模式 GUI，轻量高效 |
| **系统信息** | sysinfo 0.31 | 跨平台系统信息采集 |
| **Windows API** | windows-sys 0.59 | 底层 Windows API 调用 |
| **配置管理** | serde + toml | TOML 格式配置文件 |
| **并发** | parking_lot | 高性能互斥锁 |

### 架构概览

```
src/
├── main.rs           # 程序入口，字体/主题初始化
├── app.rs            # 应用主结构，页面路由
├── config/           # 配置管理 (TOML 持久化)
├── core/             # 核心加速逻辑
│   ├── elevation.rs       # UAC 提权管理
│   ├── memory_cleaner.rs  # 内存清理
│   ├── process_manager.rs # 进程管理
│   ├── cpu_optimizer.rs   # CPU 优化
│   ├── power_manager.rs   # 电源计划管理
│   ├── game_mode.rs       # Windows 游戏模式
│   ├── gpu_manager.rs     # GPU 设置
│   ├── disk_optimizer.rs  # 磁盘优化
│   └── process_category.rs # 进程分类
├── monitor/          # 系统监控 (后台线程)
└── ui/               # 界面模块
    ├── dashboard.rs       # 仪表盘
    ├── process_page.rs    # 进程管理页
    ├── gpu_page.rs        # GPU 设置页
    ├── system_opt_page.rs # 系统优化页
    ├── settings_page.rs   # 设置页
    ├── theme.rs           # 主题常量
    └── widgets.rs         # 通用 UI 组件
```

---

## 🤝 贡献

欢迎提 Issue 和 PR。提交代码前请确保：

```bash
cargo build
cargo clippy
```

### 开发指南

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'Add amazing feature'`
4. 推送到分支：`git push origin feature/amazing-feature`
5. 创建 Pull Request

---

## 📄 许可证

[MIT](LICENSE) © 2026 mi

---

<p align="center">
  <sub>Made with ❤️ and Rust</sub>
</p>
