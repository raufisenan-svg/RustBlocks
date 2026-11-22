@echo off
setlocal
cd /d "%~dp0"
if not exist "target\release\rustblocks.exe" (
  echo Game not built yet. Run build_windows.bat first.
  pause
  exit /b 1
)
start "RustBlocks" "target\release\rustblocks.exe"
