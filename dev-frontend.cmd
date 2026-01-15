@echo off
REM Cross-platform wrapper for dev frontend
REM On Windows, calls PowerShell script; on Unix, calls bash script
REM Port conflict handling (port 8080) is done in the scripts this dispatches to

if "%OS%"=="Windows_NT" (
    powershell -ExecutionPolicy Bypass -File "%~dp0dev-frontend.ps1"
) else (
    bash "%~dp0dev-frontend.sh"
)
