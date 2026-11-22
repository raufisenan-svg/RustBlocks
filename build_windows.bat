@echo off
setlocal
cd /d "%~dp0"
echo Building RustBlocks in release mode...
cargo build --release
if errorlevel 1 (
  echo.
  echo Build failed. Check that Rust stable and the Windows C++ build tools are installed.
  pause
  exit /b 1
)
echo.
echo Build complete: target\release\rustblocks.exe
pause
