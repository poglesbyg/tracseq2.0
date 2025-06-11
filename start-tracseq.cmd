@echo off
echo.
echo ======================================
echo   TracSeq 2.0 - Quick Start Launcher
echo ======================================
echo.

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo ERROR: Please run this from the lab_manager directory
    echo Current directory: %CD%
    pause
    exit /b 1
)

echo Starting TracSeq 2.0...
echo This may take a few minutes on first run (Docker build)
echo.

REM Try PowerShell first (recommended)
where powershell >nul 2>nul
if %ERRORLEVEL% == 0 (
    echo Using PowerShell launcher...
    powershell -ExecutionPolicy Bypass -File "run.ps1" quick-start
) else (
    echo PowerShell not found, trying batch launcher...
    call run.bat start-dev
)

echo.
echo Press any key to close this window...
pause >nul 
