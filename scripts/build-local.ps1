[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [string]$OutputDirectory = 'dist'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-PackageText {
    param(
        [string]$Path,
        [string]$Content,
        [System.Text.Encoding]$Encoding
    )

    $windowsText = $Content -replace '\r?\n', "`r`n"
    [System.IO.File]::WriteAllText($Path, $windowsText, $Encoding)
}

try {
    if ($env:OS -ne 'Windows_NT') {
        throw 'This packaging script requires Windows.'
    }

    $projectRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $manifestPath = Join-Path $projectRoot 'Cargo.toml'
    $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8
    $packageMatch = [regex]::Match($manifest, '(?ms)^\[package\]\s*(?<package>.*?)(?=^\[|\z)')
    $versionMatch = [regex]::Match($packageMatch.Groups['package'].Value, '(?m)^version\s*=\s*"([^"]+)"')
    $version = $versionMatch.Groups[1].Value
    if ($version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$') {
        throw "Cannot read a valid package version from $manifestPath."
    }

    if (-not $SkipBuild) {
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            throw 'Cargo was not found. Install the Rust MSVC toolchain and C++ Build Tools, then reopen the terminal.'
        }
        Push-Location -LiteralPath $projectRoot
        try {
            & cargo build --locked --release
            if ($LASTEXITCODE -ne 0) {
                throw "cargo build failed with exit code $LASTEXITCODE."
            }
        } finally {
            Pop-Location
        }
    }

    $targetRoot = Join-Path $projectRoot 'target'
    if ($env:CARGO_TARGET_DIR) {
        $targetRoot = $env:CARGO_TARGET_DIR
        if (-not [System.IO.Path]::IsPathRooted($targetRoot)) {
            $targetRoot = Join-Path $projectRoot $targetRoot
        }
    }
    $binaryPath = Join-Path $targetRoot 'release\game-accelerator.exe'
    if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
        throw "Release executable was not found: $binaryPath. Build the default Windows x64 target first."
    }

    # Verify the binary, rather than labelling an ARM or x86 build as x64.
    $binaryReader = [System.IO.BinaryReader]::new([System.IO.File]::OpenRead($binaryPath))
    try {
        if ($binaryReader.ReadUInt16() -ne 0x5A4D) {
            throw 'The release executable is not a Windows PE file.'
        }
        $binaryReader.BaseStream.Position = 0x3C
        $peOffset = $binaryReader.ReadUInt32()
        $binaryReader.BaseStream.Position = $peOffset
        if ($binaryReader.ReadUInt32() -ne 0x00004550 -or $binaryReader.ReadUInt16() -ne 0x8664) {
            throw 'The portable package requires a Windows x64 executable.'
        }
    } finally {
        $binaryReader.Dispose()
    }

    if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
        throw 'OutputDirectory must not be empty.'
    }
    $distRoot = $OutputDirectory
    if (-not [System.IO.Path]::IsPathRooted($distRoot)) {
        $distRoot = Join-Path $projectRoot $distRoot
    }
    $distRoot = [System.IO.Path]::GetFullPath($distRoot)
    $packageRoot = Join-Path $distRoot 'GameAccelerator'
    New-Item -ItemType Directory -Path $packageRoot -Force | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $packageRoot 'game-accelerator.exe') -Force

    $ascii = [System.Text.Encoding]::ASCII
    $utf8WithBom = [System.Text.UTF8Encoding]::new($true)
    $utf8NoBom = [System.Text.UTF8Encoding]::new($false)
    $startScript = @'
@echo off
setlocal
if not exist "%~dp0game-accelerator.exe" goto missing_exe
if exist "%~dp0data\" goto launch
mkdir "%~dp0data"
if errorlevel 1 goto data_error
:launch
start "" /D "%~dp0" "%~dp0game-accelerator.exe" --config-dir "%~dp0data" %*
if errorlevel 1 goto launch_error
exit /b 0
:missing_exe
echo The application EXE is missing. Extract the complete portable package.
pause
exit /b 1
:data_error
echo Cannot create the data directory. Move the package to a writable directory.
pause
exit /b 1
:launch_error
echo Failed to launch Game Accelerator.
pause
exit /b 1
'@
    Write-PackageText -Path (Join-Path $packageRoot 'Start.cmd') -Content $startScript -Encoding $ascii

    $diagnosticsScript = @'
@echo off
setlocal
if not exist "%~dp0game-accelerator.exe" goto missing_exe
if exist "%~dp0data\" goto diagnose
mkdir "%~dp0data"
if errorlevel 1 goto data_error
:diagnose
start "" /wait /D "%~dp0" "%~dp0game-accelerator.exe" --config-dir "%~dp0data" --diagnose "%~dp0data\diagnostics.toml" %*
set "diagnosticExit=%errorlevel%"
if not "%diagnosticExit%"=="0" goto diagnostic_error
if not exist "%~dp0data\diagnostics.toml" goto missing_report
echo Diagnostic report: "%~dp0data\diagnostics.toml"
pause
exit /b 0
:missing_exe
echo The application EXE is missing. Extract the complete portable package.
pause
exit /b 1
:data_error
echo Cannot create the data directory. Move the package to a writable directory.
pause
exit /b 1
:diagnostic_error
echo Diagnostics failed with exit code %diagnosticExit%.
pause
exit /b %diagnosticExit%
:missing_report
echo Diagnostics completed without creating the expected report.
pause
exit /b 1
'@
    Write-PackageText -Path (Join-Path $packageRoot 'Diagnostics.cmd') -Content $diagnosticsScript -Encoding $ascii

    $instructions = @"
Game Accelerator $version 本地便携版

1. 将整个目录解压到自己可写的文件夹，双击 Start.cmd 打开软件。
2. 在“设置”选择无畏契约、英雄联盟或穿越火线预设，先保留默认开关。
3. 点击“启动加速”后保持软件运行；游戏结束后点击“停止并恢复原设置”或正常关闭。
4. 停止/正常退出会尝试恢复本次会话修改前的电源计划和 Windows 游戏模式。
5. 默认不结束后台进程。全进程内存裁剪和自动游戏优先级调整已禁用。
6. 在系统/GPU 页手动修改的设置会持续保留；已关闭程序不能恢复。
7. 游戏进程识别接受文件名或完整路径；GPU 首选项需要现有 EXE 的绝对路径。
8. 配置保存在 data 文件夹。双击 Diagnostics.cmd 进行只读诊断，报告写入 data\diagnostics.toml。
9. 普通启动无需管理员权限；确需权限的操作可在界面中点击管理员按钮并查看 UAC。

该工具管理本机资源与 Windows 设置。FPS/Ping 效果需实测，未获得游戏厂商反作弊兼容认证。
强制结束软件、崩溃或断电可能中断恢复。请查看操作结果和 Windows 实际设置。
SHA256SUMS.txt 包含分发文件校验值。data 不包含在分发 ZIP 中。
项目与完整说明：https://github.com/zhang-forever/Game-Accelerator
"@
    Write-PackageText -Path (Join-Path $packageRoot '使用说明.txt') -Content $instructions -Encoding $utf8WithBom

    # Include only distribution files: never copy users' data into an archive.
    $packageNames = @('game-accelerator.exe', 'Start.cmd', 'Diagnostics.cmd', '使用说明.txt')
    $checksumLines = foreach ($name in $packageNames) {
        $hash = (Get-FileHash -LiteralPath (Join-Path $packageRoot $name) -Algorithm SHA256).Hash.ToLowerInvariant()
        '{0}  {1}' -f $hash, $name
    }
    Write-PackageText -Path (Join-Path $packageRoot 'SHA256SUMS.txt') -Content (($checksumLines -join "`n") + "`n") -Encoding $utf8NoBom

    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zipName = "GameAccelerator-$version-windows-x64.zip"
    $zipPath = Join-Path $distRoot $zipName
    $archiveStream = [System.IO.File]::Open($zipPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
    $archive = [System.IO.Compression.ZipArchive]::new($archiveStream, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($name in ($packageNames + @('SHA256SUMS.txt'))) {
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
                $archive,
                (Join-Path $packageRoot $name),
                "GameAccelerator/$name",
                [System.IO.Compression.CompressionLevel]::Optimal
            ) | Out-Null
        }
    } finally {
        $archive.Dispose()
        $archiveStream.Dispose()
    }

    $zipHash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-PackageText -Path "$zipPath.sha256" -Content ("$zipHash  $zipName`n") -Encoding $ascii
    Write-Host "Portable directory: $packageRoot"
    Write-Host "Archive: $zipPath"
    Write-Host "SHA256: $zipHash"
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
