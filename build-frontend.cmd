@echo off
REM Cross-platform wrapper for building frontend
REM On Windows, calls PowerShell script; on Unix, calls bash script

if "%OS%"=="Windows_NT" (
    powershell -ExecutionPolicy Bypass -File "%~dp0build-frontend.ps1"
) else (
    bash "%~dp0build-frontend.sh"
)
