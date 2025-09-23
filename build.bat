@echo off
setlocal enabledelayedexpansion

echo 🔨 SafeHold Cross-Platform Build Script
echo ========================================

REM Create dist directory
if not exist dist mkdir dist

REM Check if cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ❌ Cargo not found. Please install Rust.
    exit /b 1
)

REM Function to build for a target
:build_target
set target=%1
set name=%2

echo 🔨 Building for %target%...

REM Add target if not already installed
rustup target add %target% >nul 2>nul

REM Build CLI version
echo   📦 Building CLI version...
cargo build --release --target %target% --no-default-features
if %ERRORLEVEL% neq 0 (
    echo ❌ Failed to build CLI version for %target%
    exit /b 1
)

REM Build GUI version
echo   🖥️ Building GUI version...
cargo build --release --target %target% --features gui
if %ERRORLEVEL% neq 0 (
    echo ❌ Failed to build GUI version for %target%
    exit /b 1
)

REM Create target directory
if not exist "dist\%name%" mkdir "dist\%name%"

REM Copy binaries based on target OS
echo %target% | findstr "windows" >nul
if %ERRORLEVEL% equ 0 (
    copy "target\%target%\release\safehold.exe" "dist\%name%\safehold-cli.exe" >nul
    copy "target\%target%\release\safehold.exe" "dist\%name%\safehold-gui.exe" >nul
) else (
    copy "target\%target%\release\safehold" "dist\%name%\safehold-cli" >nul
    copy "target\%target%\release\safehold" "dist\%name%\safehold-gui" >nul
)

REM Copy documentation
copy README.md "dist\%name%\" >nul
copy CHANGELOG.md "dist\%name%\" >nul
if exist LICENSE copy LICENSE "dist\%name%\" >nul

echo ✅ Built %name%
goto :eof

REM Windows targets
echo 🖥️ Building Windows targets...
call :build_target "x86_64-pc-windows-msvc" "windows-x64"
call :build_target "x86_64-pc-windows-gnu" "windows-x64-gnu"

REM macOS targets (may not work on Windows without cross-compilation tools)
echo 🍎 Building macOS targets...
call :build_target "x86_64-apple-darwin" "macos-x64"
call :build_target "aarch64-apple-darwin" "macos-arm64"

REM Linux targets (may not work on Windows without cross-compilation tools)
echo 🐧 Building Linux targets...
call :build_target "x86_64-unknown-linux-gnu" "linux-x64"
call :build_target "aarch64-unknown-linux-gnu" "linux-arm64"
call :build_target "x86_64-unknown-linux-musl" "linux-x64-musl"

echo.
echo 🎉 All builds completed successfully!
echo 📦 Binaries are available in the 'dist' directory:
dir dist

echo.
echo 📝 Note: Some targets may require additional setup:
echo   • Windows: Install Visual Studio Build Tools or MinGW
echo   • macOS: macOS SDK (available on macOS only)
echo   • Linux ARM64: Cross-compilation tools
echo   • Linux MUSL: musl-tools package

pause