# PowerShell script to build frontend with Trunk
# Equivalent to build-frontend.sh for Windows

$ErrorActionPreference = "Stop"

# Get the script directory (project root)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Unset all color-related environment variables
Remove-Item Env:NO_COLOR -ErrorAction SilentlyContinue
Remove-Item Env:CLICOLOR -ErrorAction SilentlyContinue
Remove-Item Env:CLICOLOR_FORCE -ErrorAction SilentlyContinue

# Change to src directory where Trunk.toml and Cargo.toml are
Set-Location "$ScriptDir\src"

# Run trunk from src directory - it will find Cargo.toml and Trunk.toml here
trunk build --release
