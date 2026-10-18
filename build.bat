@echo off
rem ==============================================================================
rem PhotoCraft - Windows Build and Packaging Launcher (build.bat)
rem Targets:
rem   1. photocraft.exe and photocraft-cli.exe (static CRT + embedded icon resources)
rem   2. Installer bitmaps (24-bit installer-banner.bmp & installer-dialog.bmp)
rem   3. Windows setup installer (.msi)
rem   4. Portable zip package (.zip)
rem ==============================================================================
setlocal enabledelayedexpansion

cd /d "%~dp0"

echo ==============================================================================
echo   PhotoCraft Windows Build and Packaging Tool
echo ==============================================================================
echo.

rem 1. Auto-detect and configure Cargo
where.exe cargo.exe >nul 2>nul
if errorlevel 1 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set "PATH=%USERPROFILE%\.cargo\bin;!PATH!"
        echo [OK] Loaded Rust toolchain from %USERPROFILE%\.cargo\bin
    ) else (
        echo [ERROR] cargo.exe not found! Please install Rust from https://rustup.rs
        pause
        exit /b 1
    )
) else (
    echo [OK] Rust toolchain detected
)

rem 2. Auto-detect and configure .NET SDK and WiX toolset
if exist "%USERPROFILE%\.dotnet\dotnet.exe" (
    set "DOTNET_ROOT=%USERPROFILE%\.dotnet"
    set "PATH=%USERPROFILE%\.dotnet;%USERPROFILE%\.dotnet\tools;!PATH!"
    echo [OK] Loaded .NET SDK and WiX toolset
)
set "DOTNET_CLI_TELEMETRY_OPTOUT=1"

rem 3. Auto-detect and configure Windows SDK rc.exe (icon & VERSIONINFO resources)
where.exe rc.exe >nul 2>nul
if errorlevel 1 (
    for /f "delims=" %%I in ('dir /b /s "C:\Program Files (x86)\Windows Kits\10\bin\10.*\x64\rc.exe" 2^>nul') do (
        set "RC_DIR=%%~dpI"
        set "PATH=!RC_DIR!;!PATH!"
    )
)
echo [OK] Windows SDK resource compiler ready

set "PHOTOCRAFT_REQUIRE_WINRES=1"

rem 4. Architecture selection (default: x64)
set "ARCH=x64"
if not "%~1"=="" set "ARCH=%~1"

echo.
echo [1/3] Executing packaging pipeline for architecture: %ARCH%...
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0packaging\windows\package.ps1" -Arch %ARCH%
if errorlevel 1 (
    echo.
    echo [ERROR] Packaging pipeline failed!
    pause
    exit /b 1
)

echo.
echo [2/3] Staging standalone executables into dist\release...
set "TARGET=x86_64-pc-windows-msvc"
if /i "%ARCH%"=="x86" set "TARGET=i686-pc-windows-msvc"
if /i "%ARCH%"=="arm64" set "TARGET=aarch64-pc-windows-msvc"

if not exist "dist\release" mkdir "dist\release"
if exist "target\!TARGET!\release\photocraft.exe" (
    copy /y "target\!TARGET!\release\photocraft.exe" "dist\release\photocraft.exe" >nul
)
if exist "target\!TARGET!\release\photocraft-cli.exe" (
    copy /y "target\!TARGET!\release\photocraft-cli.exe" "dist\release\photocraft-cli.exe" >nul
)

echo.
echo ==============================================================================
echo [3/3] Build and Packaging Finished Successfully!
echo Artifacts staged in dist\release:
echo ==============================================================================
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Get-ChildItem dist\release | Format-Table Name, Length, LastWriteTime"

echo.
echo All artifacts are available in: %~dp0dist\release
echo.
pause
exit /b 0
