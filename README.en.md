# Game Accelerator

A Windows 10/11 tool for managing local resources and game settings, built with Rust and egui. The current local source version is **1.2.0**, with presets for VALORANT, League of Legends, and CrossFire.

[简体中文](README.md) · [GitHub Releases](https://github.com/zhang-forever/Game-Accelerator/releases) · [MIT](LICENSE)

Version 1.2.0 uses a consistent dark interface in blue and gray. Performance Overview brings game presets, session controls, and resource cards of equal width together. Process Management offers category and process list views. The Save and Restore Defaults buttons stay at the bottom of Preferences. The default window is 1080×720, with a minimum size of 880×560 and scrollable page content.

![Game Accelerator Performance Overview](docs/screenshot.png)

## Try it on your PC

If you have a portable package, extract all files into a directory you can write to, then double-click `GameAccelerator\Start.cmd`. Settings live in the adjacent `data` directory. The app starts with standard user privileges; use its administrator button when a specific operation needs UAC elevation.

To run from this repository:

1. Prepare Windows x64, the Rust MSVC toolchain, Visual Studio C++ Build Tools, and the Windows SDK.
2. Double-click `run-local.cmd` in the repository root. On the first run it builds and creates `dist\GameAccelerator`, then opens the GUI. You can also run it from PowerShell:

   ```powershell
   .\run-local.cmd
   ```

3. Choose a game preset in “性能概览” (Performance Overview) or “偏好设置” (Preferences) and keep the default options for the first trial. Click “启动加速” (Start Boost) and keep the app running during your game.
4. Click “停止并恢复原设置” (Stop and Restore Original Settings) or close the app normally after playing to restore the power plan and Windows Game Mode settings that were active before the session. Check the status messages for the result.

The local source can build a 1.2.0 portable package. For GitHub downloads, check the versions and attachments listed on Releases. The older v1.0.0 EXE does not include the session restoration and defaults described here. This workflow does not require an MSI installer.

## Defaults and restoration

Windows Game Mode and a high-performance power plan are enabled for a boost session by default. **Background process termination is disabled by default.** System-wide process memory trimming and automatic game priority changes are disabled. Presets identify game processes and can be edited for your installation.

| Operation | When it runs | After stopping or closing |
|-----------|--------------|---------------------------|
| Session power plan and Game Mode | After manually starting boost | Attempts to restore the previous settings; reports restoration failures |
| Background process termination | When enabled by the user, or manually on the process page | Closed applications are not restarted; unsaved work cannot be recovered |
| System-wide memory trimming | Disabled | Does not trim all system processes |
| Automatic game priority changes | Disabled | Does not change game process priority |
| Manual changes on System Optimization or GPU Settings | When the user clicks a control | Remain applied, sometimes requiring a restart; excluded from session restoration |

Normal stopping or closing runs session restoration. Forced termination, power loss, or a crash may prevent it from completing; check Windows power and Game Mode settings afterward. A high-performance power plan may increase power use, fan noise, and temperature, so choose according to your PC's workload.

This tool manages local resources and Windows settings. FPS and online latency effects require measurements on the actual PC and game. The project has no game-vendor anti-cheat compatibility certification. Keep the defaults for the first trial of these competitive games.

## Features and input requirements

- Performance Overview displays CPU, memory, GPU, and process information. NVIDIA GPU utilization, temperature, and VRAM readings require an available `nvidia-smi`; other GPUs may not provide these readings.
- Process Management offers category and process list views, with search, sorting, and confirmation before termination. Save application work first; the protected list does not establish that every third-party application can be closed safely.
- System Optimization offers manual power, Game Mode, and other settings. The GUI reports unavailable features or insufficient permissions.
- Game process identification accepts an **EXE filename or full path**, such as `VALORANT-Win64-Shipping.exe`.
- Windows per-application GPU preferences require the **absolute path of an existing EXE**, such as `E:\Games\Example\game.exe`. A filename alone cannot identify the application for this setting.
- Windows GPU preferences depend on the GPU, driver, and Windows support. Check the actual result in the GUI. CPU core parking changes, NVIDIA PowerMizer registry changes, and telemetry task disabling have been removed or disabled. The app does not launch games automatically.

## Local build and portable package

```powershell
git clone https://github.com/zhang-forever/Game-Accelerator.git
cd Game-Accelerator
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-local.ps1
```

The script runs `cargo build --locked --release`, does not install tools or change system settings, and preserves environment configuration such as `CARGO_HOME`. To package an existing release build:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-local.ps1 -SkipBuild
```

It reads `target\release\game-accelerator.exe` by default, or the equivalent path under `CARGO_TARGET_DIR` when set. Use `-OutputDirectory <directory>` to change the output location; relative paths are resolved from the repository root. Default outputs:

```text
dist/
├── GameAccelerator/
│   ├── game-accelerator.exe
│   ├── Start.cmd
│   ├── Diagnostics.cmd
│   ├── 使用说明.txt
│   └── SHA256SUMS.txt
├── GameAccelerator-1.2.0-windows-x64.zip
└── GameAccelerator-1.2.0-windows-x64.zip.sha256
```

Packaging preserves existing directories. The ZIP includes only the distribution files above and excludes runtime `data`, configuration, and diagnostic reports. Use `Get-FileHash -Algorithm SHA256` to compare ZIP and EXE hashes with the supplied checksum files.

Double-clicking the EXE also works and uses the user's configuration directory by default. `Start.cmd` passes `--config-dir "%~dp0data"` to keep portable configuration separate. Automatic tray behavior and automatic boost on startup are not implemented.

The repository retains `wix/main.wxs` for installer development. MSI builds need an additional WiX environment; the current CI and local delivery workflow validate the EXE and portable ZIP.

## Diagnostics and troubleshooting

Double-click `Diagnostics.cmd` in the portable directory and wait for completion. Success produces `data\diagnostics.toml`; failures display the exit code. Diagnostics read system state and write the report without starting boost or changing power, registry, service, or process settings. The CLI entry point is `game-accelerator.exe --diagnose <report-file>` and accepts `--config-dir <directory>`; normal GUI use requires no CLI options. The wrapper uses `start /wait` to wait for the GUI-subsystem EXE and read its actual exit code.

| Problem | Action |
|---------|--------|
| Cargo, linker, or SDK missing on the first run | Install the Rust MSVC toolchain and Visual Studio C++ Build Tools/Windows SDK, reopen the terminal, and retry; a prebuilt portable package needs no build tools |
| Cannot write configuration or the report | Move the entire portable directory to a writable location; extract the ZIP before use and avoid protected installation directories |
| An operation reports insufficient privileges | Check the specific failure, then use the app's administrator button if that operation requires elevation and review the UAC prompt |
| Game process not found | Launch the game and check its actual EXE; launchers and the running game may have different names |
| No GPU temperature or utilization | Check the NVIDIA driver and availability of `nvidia-smi`; CPU and memory monitoring remain available |
| A setting remains changed after stopping | Check restoration errors; manual page actions and closed applications are excluded from automatic restoration |

## Development and verification

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --release -- -W clippy::all
cargo test --locked --release
cargo build --locked --release
```

For a local GUI check, `game-accelerator.exe --smoke-test <directory>` uses the actual egui renderer to save six PNGs: the five pages plus the advanced process list view, along with `report.toml`. It filters interactive input while preserving the normal visual appearance. It does not start boost, save configuration, or change power, registry, service, or process settings. For example:

```powershell
.\target\release\game-accelerator.exe --smoke-test .\ui-smoke
.\target\release\game-accelerator.exe --window-size 880x560 --smoke-test .\ui-smoke-small
```

`--window-size <widthxheight>` sets the initial window dimensions. The second command checks the minimum window layout. Captures are named `dashboard.png`, `settings.png`, `gpu.png`, `system.png`, `processes.png`, and `process-list.png`.

CI runs these checks, read-only diagnostics, and portable packaging, then uploads the ZIP and checksum files. The version-tag workflow builds and publishes the EXE, ZIP, and checksums. Editing the workflow alone does not publish a release.

Key paths: `src/app.rs` manages the GUI and sessions, `src/config` persists settings, `src/core` wraps system operations, `src/monitor` collects metrics, `src/ui` implements pages, and `scripts/build-local.ps1` produces local deliverables.

When reporting a problem, include the app version, Windows version, failed action, and status message. A diagnostic report can help identify system and permission issues.

## License

[MIT](LICENSE) © 2026 mi
