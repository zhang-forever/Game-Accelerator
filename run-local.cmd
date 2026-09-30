@echo off
setlocal
if not exist "%~dp0dist\GameAccelerator\game-accelerator.exe" goto build
if not exist "%~dp0dist\GameAccelerator\Start.cmd" goto build
goto launch
:build
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-local.ps1"
if errorlevel 1 goto build_error
:launch
call "%~dp0dist\GameAccelerator\Start.cmd" %*
exit /b %errorlevel%
:build_error
echo Local build failed. See the error above and check the Rust/C++ build tools.
pause
exit /b 1
