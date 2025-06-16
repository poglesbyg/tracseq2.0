@echo off
title TracSeq 2.0 - Quick Start

echo.
echo =======================================
echo        TracSeq 2.0 - Quick Start
echo =======================================
echo.

REM Check if we're in the right directory
if not exist "run.ps1" (
    echo [ERROR] Please run this from the TracSeq 2.0 root directory
    pause
    exit /b 1
)

REM Try PowerShell first (best experience)
powershell -Command "Get-ExecutionPolicy" >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    for /f "tokens=*" %%a in ('powershell -Command "Get-ExecutionPolicy"') do set POLICY=%%a
    if not "!POLICY!"=="Restricted" (
        echo [INFO] Using PowerShell for best Windows experience...
        powershell -File "run.ps1" start-dev
        goto :end
    )
)

REM Fallback to batch file
echo [INFO] Using Windows batch implementation...
call run.bat start-dev

:end
echo.
echo [INFO] TracSeq 2.0 is now running!
echo [INFO] Check the URLs above to access the application
pause 
