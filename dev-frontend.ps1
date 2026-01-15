# PowerShell script to serve frontend with Trunk for Tauri dev mode
# Equivalent to dev-frontend.sh for Windows

$ErrorActionPreference = "Stop"

# Get the script directory (project root)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Unset all color-related environment variables
Remove-Item Env:NO_COLOR -ErrorAction SilentlyContinue
Remove-Item Env:CLICOLOR -ErrorAction SilentlyContinue
Remove-Item Env:CLICOLOR_FORCE -ErrorAction SilentlyContinue

# Change to src directory where Trunk.toml and Cargo.toml are
Set-Location "$ScriptDir\src"

# Check if port 8080 is in use and kill stale trunk processes
$PortProcess = Get-NetTCPConnection -LocalPort 8080 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique
if ($PortProcess) {
    $ProcessName = (Get-Process -Id $PortProcess -ErrorAction SilentlyContinue).ProcessName
    if ($ProcessName -like "*trunk*") {
        Write-Host "Killing stale trunk process on port 8080 (PID: $PortProcess)"
        Stop-Process -Id $PortProcess -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
    } else {
        Write-Host "Warning: Port 8080 is in use by another process (PID: $PortProcess)"
        Write-Host "Trunk will attempt to start anyway and may fail"
    }
}

# Run trunk serve from src directory
trunk serve --port 8080
