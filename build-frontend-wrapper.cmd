@echo off
REM Cross-platform wrapper for Windows
REM Calls the PowerShell script
powershell -ExecutionPolicy Bypass -File "%~dp0build-frontend.ps1"
