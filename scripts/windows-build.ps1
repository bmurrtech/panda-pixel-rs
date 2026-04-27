$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root
$ProductSlug = "panda-pixel-rs"
$OutputDir = Join-Path $Root "target\windows-artifacts"

function Get-AppVersion {
  $paths = @(
    (Join-Path $Root "src-tauri\tauri.conf.json"),
    (Join-Path $Root "apps\desktop\src-tauri\tauri.conf.json")
  )
  foreach ($p in $paths) {
    if (Test-Path $p) {
      $json = Get-Content -Path $p -Raw | ConvertFrom-Json
      if ($null -ne $json.version -and $json.version -ne "") {
        return [string]$json.version
      }
    }
  }
  throw "Unable to detect version from tauri.conf.json"
}

Write-Host "==> [windows-build] Environment"
rustc --version
cargo --version
trunk --version
cargo tauri --version

$Version = Get-AppVersion
Write-Host "==> [windows-build] Detected version: $Version"

Write-Host "==> [windows-build] Building frontend"
bash -lc "./scripts/build-frontend.sh"

Write-Host "==> [windows-build] Building Tauri bundles"
Push-Location "$Root\src-tauri"
cargo tauri build
Pop-Location

if (!(Test-Path $OutputDir)) {
  New-Item -Path $OutputDir -ItemType Directory | Out-Null
}

Write-Host "==> [windows-build] Collecting versioned artifacts"
$artifactFiles = Get-ChildItem -Path "$Root\target", "$Root\src-tauri\target" -Recurse -File -ErrorAction SilentlyContinue |
  Where-Object { $_.Extension -in ".exe", ".msi" }

if ($artifactFiles.Count -eq 0) {
  throw "No Windows artifacts were found (.exe/.msi)."
}

foreach ($file in $artifactFiles) {
  $destName = "$ProductSlug-$Version-windows-$($file.Name)"
  $destPath = Join-Path $OutputDir $destName
  Copy-Item -Path $file.FullName -Destination $destPath -Force
  Write-Host "Created: $destPath"
}
