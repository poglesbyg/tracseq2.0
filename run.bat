@echo off
REM TracSeq 2.0 Lab Manager Runner Batch Wrapper for Windows
REM This is a simple wrapper that calls the PowerShell script

echo [INFO] TracSeq 2.0 Lab Manager - Windows Batch Wrapper
echo [INFO] Delegating to PowerShell script for better functionality...
echo.

REM Check if PowerShell is available
powershell -Command "Write-Host 'PowerShell available'" >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] PowerShell is not available
    echo [INFO] Please use the PowerShell script directly: .\run.ps1 %*
    pause
    exit /b 1
)

REM Call the PowerShell script with all arguments
powershell -ExecutionPolicy Bypass -File "run.ps1" %* 
