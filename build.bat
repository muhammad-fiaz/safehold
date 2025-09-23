@echo off
REM SafeHold Simple Build Script
REM For cross-platform builds, use build-universal.bat

echo � Building SafeHold (local development)...

REM Build CLI version
echo 📦 Building CLI version...
cargo build --release --features cli
if errorlevel 1 (
    echo ❌ CLI build failed!
    exit /b 1
)

REM Build GUI version
echo 🎨 Building GUI version...
cargo build --release --features gui
if errorlevel 1 (
    echo ❌ GUI build failed!
    exit /b 1
)

echo ✅ Build completed successfully!
echo 📁 Binaries available in: target\release\
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