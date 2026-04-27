@echo off
setlocal enableextensions

set "SCRIPT_DIR=%~dp0"
set "PS_SCRIPT=%SCRIPT_DIR%windows-build.ps1"

if not exist "%PS_SCRIPT%" (
  echo ERROR: Missing PowerShell script: "%PS_SCRIPT%"
  exit /b 1
)

where pwsh >nul 2>nul
if %errorlevel%==0 (
  pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%PS_SCRIPT%" %*
  exit /b %errorlevel%
)

where powershell >nul 2>nul
if %errorlevel%==0 (
  powershell -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%PS_SCRIPT%" %*
  exit /b %errorlevel%
)

echo ERROR: Neither pwsh nor powershell was found on PATH.
exit /b 1
