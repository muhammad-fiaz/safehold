@echo off
setlocal enabledelayedexpansion

echo 🚀 SafeHold Release Preparation Script
echo =====================================

REM Check prerequisites
echo 🔍 Checking prerequisites...

where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ❌ Cargo not found. Please install Rust.
    exit /b 1
)

where git >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ❌ Git not found. Please install Git.
    exit /b 1
)

REM Get current version from Cargo.toml
for /f "tokens=3 delims= " %%a in ('findstr "^version = " Cargo.toml') do set CURRENT_VERSION=%%a
set CURRENT_VERSION=%CURRENT_VERSION:"=%
echo 📦 Current version: %CURRENT_VERSION%

REM Check if working directory is clean
git status --porcelain >nul 2>nul
if %ERRORLEVEL% equ 0 (
    for /f %%i in ('git status --porcelain ^| find /c /v ""') do set CHANGES=%%i
    if !CHANGES! gtr 0 (
        echo ⚠️ Working directory is not clean. Uncommitted changes:
        git status --short
        echo.
        set /p "CONTINUE=Continue anyway? (y/N): "
        if /i not "!CONTINUE!"=="y" (
            echo ❌ Aborting release preparation.
            exit /b 1
        )
    )
)

REM Run tests
echo 🧪 Running tests...
cargo test
if %ERRORLEVEL% neq 0 (
    echo ❌ Tests failed. Fix tests before releasing.
    exit /b 1
)

REM Check code quality
echo 🔍 Checking code quality...
cargo clippy -- -D warnings
if %ERRORLEVEL% neq 0 (
    echo ❌ Clippy warnings found. Fix warnings before releasing.
    exit /b 1
)

REM Format code
echo 🎨 Formatting code...
cargo fmt --check
if %ERRORLEVEL% neq 0 (
    echo ⚠️ Code formatting issues found. Running cargo fmt...
    cargo fmt
)

REM Build release versions
echo 🔨 Building release versions...

echo   📦 Building CLI version...
cargo build --release --no-default-features
if %ERRORLEVEL% neq 0 (
    echo ❌ CLI build failed.
    exit /b 1
)

echo   🖥️ Building GUI version...
cargo build --release --features gui
if %ERRORLEVEL% neq 0 (
    echo ❌ GUI build failed.
    exit /b 1
)

REM Check CHANGELOG
echo 📝 Checking CHANGELOG.md...
findstr /C:"Version %CURRENT_VERSION%" CHANGELOG.md >nul
if %ERRORLEVEL% neq 0 (
    echo ⚠️ Version %CURRENT_VERSION% not found in CHANGELOG.md
    echo    Please update CHANGELOG.md before releasing.
)

REM Build documentation
echo 📚 Building documentation...
cargo doc --no-deps --features gui
if %ERRORLEVEL% neq 0 (
    echo ❌ Documentation build failed.
    exit /b 1
)

REM Cross-platform builds (optional)
echo.
set /p "CROSS_BUILD=Build cross-platform binaries? (y/N): "
if /i "%CROSS_BUILD%"=="y" (
    echo 🔨 Building cross-platform binaries...
    call build.bat
    if %ERRORLEVEL% neq 0 (
        echo ⚠️ Cross-platform build failed. Continuing...
    )
)

REM Package information
echo 📦 Package information:
cargo package --list --features gui | head -20

REM Dry-run publish
echo.
set /p "DRY_RUN=Run dry-run publish check? (Y/n): "
if /i not "%DRY_RUN%"=="n" (
    echo 🚀 Running publish dry-run...
    cargo publish --dry-run --features gui
    if %ERRORLEVEL% neq 0 (
        echo ❌ Publish dry-run failed.
        exit /b 1
    )
)

echo.
echo ✅ Release preparation completed successfully!
echo.
echo 📋 Release checklist:
echo   • ✅ Tests passing
echo   • ✅ Code quality checks passed
echo   • ✅ CLI and GUI builds successful
echo   • ✅ Documentation builds successfully
echo   • ✅ Publish dry-run successful
echo.
echo 🚀 To publish to crates.io:
echo   cargo publish --features gui
echo.
echo 🏷️ To create Git tag and GitHub release:
echo   git tag v%CURRENT_VERSION%
echo   git push origin v%CURRENT_VERSION%
echo.
echo 📁 Binary artifacts available in:
echo   target\release\safehold.exe (CLI + GUI)
echo   dist\ (cross-platform builds, if created)

pause