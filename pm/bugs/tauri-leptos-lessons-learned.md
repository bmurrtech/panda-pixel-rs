# Tauri + Leptos Migration: Lessons Learned

This document captures issues, bugs, fixes, and technical considerations encountered during the migration from Axum web server to Tauri + Leptos desktop application.

## Table of Contents

- [Build & Configuration Issues](#build--configuration-issues)
- [CI/CD & GitHub Actions Issues](#cicd--github-actions-issues)
- [Release Management & Local Testing](#release-management--local-testing)
- [Required Build Scripts](#required-build-scripts)
- [Tauri API & Integration Issues](#tauri-api--integration-issues)
- [Leptos Reactive System Issues](#leptos-reactive-system-issues)
- [File System & Dialog Issues](#file-system--dialog-issues)
- [UI/UX Issues](#uiux-issues)
- [Performance & Optimization](#performance--optimization)
- [Static Assets & Image Rendering](#static-assets--image-rendering)
- [Developer Tools & Debugging](#developer-tools--debugging)
- [Application Icon Setup](#application-icon-setup)

---

## Build & Configuration Issues

### Issue: Tauri v2 Configuration Changes

**Problem**: `tauri.conf.json` used deprecated `devPath` and `distDir` keys, causing build failures.

**Error**:
```
Additional properties are not allowed ('devPath', 'distDir' were unexpected)
```

**Fix**: Tauri v2 uses `frontendDist` in the `build` section:
```json
{
  "build": {
    "beforeDevCommand": "./dev-frontend.sh",
    "beforeBuildCommand": "./build-frontend.sh",
    "frontendDist": "../dist"
  }
}
```

**Lesson**: Tauri v2 has significant breaking changes. Always check version-specific documentation.

---

### Issue: Trunk Build Script Environment Variables

**Problem**: Tauri sets `NO_COLOR=1` environment variable, which Trunk interprets as a flag, causing build failures.

**Error**:
```
error: invalid value '1' for '--no-color'
```

**Fix**: Required wrapper scripts that unset color-related environment variables and change to the correct directory.

**Solution**: Create platform-specific wrapper scripts at project root:
- **Unix:** `dev-frontend.sh`, `build-frontend.sh`
- **Windows:** `dev-frontend.ps1`, `build-frontend.ps1`
- **Optional dispatchers:** `dev-frontend.cmd`, `build-frontend.cmd`

**Key requirements for all scripts:**
1. Resolve repo root using script's own location
2. Unset `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE` environment variables
3. Change to `src/` directory where `Trunk.toml` and `Cargo.toml` are located
4. Run trunk commands from the correct directory
5. (Dev scripts only) Handle port 8080 conflicts automatically

**See [Required Build Scripts](#required-build-scripts) section for complete script contents and implementation details.**

**Lesson**: Wrapper scripts are required because Tauri passes incompatible environment variables (`NO_COLOR=1` is interpreted as a flag by Trunk) and Trunk must run from the `src/` directory. Always test scripts locally before relying on them in CI.

---

### Issue: Trunk Configuration Location

**Problem**: `cargo metadata` failed with "can't find library rust_tinypng_clone_frontend".

**Fix**:
1. Move `Trunk.toml` to `src/` directory
2. Add explicit `[lib]` section to `src/Cargo.toml`:
```toml
[lib]
name = "rust_tinypng_clone_frontend"
path = "lib.rs"
crate-type = ["cdylib", "rlib"]
```

**Lesson**: Trunk must be run from the directory containing both `Cargo.toml` and `Trunk.toml` for the frontend crate.

---

### Issue: Trunk WebSocket Placeholder Spam

**Problem**: Console shows `ws://{{__trunk_address__}}...` errors during Tauri dev mode.

**Fix**: Disable live-reload in `Trunk.toml`:
```toml
[serve]
reload = false  # Disable for Tauri builds
```

**Lesson**: Trunk's live-reload is designed for browser development, not Tauri. Disable it for desktop apps.

---

## CI/CD & GitHub Actions Issues

### Issue: Missing WASM Target in CI

**Problem**: GitHub Actions workflow failed to build frontend because `wasm32-unknown-unknown` target was not installed.

**Error**:
```
error[E0463]: can't find crate for `core`
  = note: the `wasm32-unknown-unknown` target may not be installed
  = help: consider downloading the target with `rustup target add wasm32-unknown-unknown`
```

**Fix**: Add step to install WASM target before building frontend:
```yaml
- name: Install WASM target
  run: rustup target add wasm32-unknown-unknown
```

**Lesson**: Leptos frontend requires WASM target. Always install it in CI before building.

---

### Issue: Missing NASM on Linux for AVIF Support

**Problem**: Linux builds failed because `rav1e` (dependency of `ravif` for AVIF encoding) requires `nasm` (Netwide Assembler) for assembly optimizations.

**Error**:
```
NASM build failed. Make sure you have nasm installed or disable the "asm" feature.
Unable to run nasm: No such file or directory (os error 2)
```

**Fix**: Add `nasm` to Linux dependencies:
```yaml
- name: Install Linux dependencies
  if: matrix.platform == 'ubuntu-latest'
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev \
      libgtk-3-dev \
      libappindicator3-dev \
      librsvg2-dev \
      patchelf \
      nasm
```

**Lesson**: Check all dependencies' build requirements. Some Rust crates have native dependencies that must be installed separately.

---

### Issue: Windows Can't Execute Shell Scripts

**Problem**: Windows runners can't execute `.sh` scripts, causing `beforeBuildCommand` to fail.

**Error**:
```
'.' is not recognized as an internal or external command,
operable program or batch file.
```

**Fix**: 
1. Create PowerShell equivalent (`build-frontend.ps1`)
2. Build frontend separately in workflow before `cargo tauri build`
3. Remove `beforeBuildCommand` from `tauri.conf.json` (workflow handles it)

**Workflow approach**:
```yaml
- name: Build frontend (Unix)
  if: matrix.platform != 'windows-latest'
  run: ./build-frontend.sh

- name: Build frontend (Windows)
  if: matrix.platform == 'windows-latest'
  shell: pwsh
  run: ./build-frontend.ps1

- name: Build Tauri app
  working-directory: src-tauri
  run: cargo tauri build
```

**Lesson**: Cross-platform CI requires platform-specific build steps. Build frontend separately in workflow rather than relying on Tauri's `beforeBuildCommand` in CI.

---

### Issue: Path Resolution for beforeDevCommand

**Problem**: Tauri's `beforeDevCommand` path resolution differs between dev and build modes. Initially used `../dev-frontend.sh` assuming it runs from `src-tauri/`, but it actually runs from project root.

**Error**:
```
bash: ../dev-frontend.sh: No such file or directory
```

**Fix**: 
1. **For local dev:** Use project root relative path in `tauri.conf.json`:
```json
{
  "build": {
    "beforeDevCommand": "./dev-frontend.sh",
    "frontendDist": "../dist"
  }
}
```

2. **For CI:** Build frontend explicitly in workflow steps (do not rely on `beforeBuildCommand`):
```yaml
- name: Build frontend (Unix)
  if: matrix.platform != 'windows-latest'
  run: ./build-frontend.sh

- name: Build frontend (Windows)
  if: matrix.platform == 'windows-latest'
  shell: pwsh
  run: ./build-frontend.ps1
```

**Lesson**: `beforeDevCommand` executes from **project root**, not `src-tauri/`. Use `./dev-frontend.sh` (relative to root), not `../dev-frontend.sh`. For CI reliability, build frontend explicitly in workflow steps rather than using `beforeBuildCommand`.

---

### Issue: Artifact Discovery Failures (Shell Quoting + Path Resolution)

**Problem**: Multiple issues with artifact discovery:
1. Shell quoting breaks find expressions when passed as strings
2. Workspace builds may output to `target/` or `src-tauri/target/`
3. macOS `.app` bundles are directories (can't upload to GitHub Releases)
4. Artifact discovery can fail silently, leading to empty releases

**Error Examples**:
```
ERROR: No artifacts found for platform macos-latest
Found artifacts:  (empty)
```

**Root Cause**: Passing find expressions as quoted strings causes shell to treat parentheses and globs as literal characters:
```bash
# BROKEN: String interpolation breaks find parsing
collect "\( -name \"*.dmg\" -o -name \"*.app.zip\" \)"
# Shell receives: '\(' and '"*.dmg"' as literal strings
```

**Fix**: Use direct find commands per platform (no string interpolation):

1. **Search both workspace output locations:**
```yaml
ROOTS=("target/release/bundle" "src-tauri/target/release/bundle")
```

2. **Use direct find commands (not helper functions with string params):**
```yaml
- name: Find bundle artifacts (fail-fast if empty)
  id: find-artifacts
  shell: bash
  run: |
    set -euxo pipefail
    ROOTS=("target/release/bundle" "src-tauri/target/release/bundle")
    ARTIFACTS=""
    
    for root in "${ROOTS[@]}"; do
      if [[ ! -d "$root" ]]; then
        continue
      fi
      
      if [[ "${{ matrix.platform }}" == "macos-latest" ]]; then
        # Direct find command - no string interpolation
        found=$(find "$root" -type f \( -name "*.dmg" -o -name "*.app.zip" \) -maxdepth 5 2>/dev/null || true)
      elif [[ "${{ matrix.platform }}" == "windows-latest" ]]; then
        found=$(find "$root" -type f \( -name "*.exe" -o -name "*.msi" \) -maxdepth 5 2>/dev/null || true)
      else
        found=$(find "$root" -type f \( -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" \) -maxdepth 5 2>/dev/null || true)
      fi
      
      if [[ -n "$found" ]]; then
        ARTIFACTS="${ARTIFACTS}${found}"$'\n'
      fi
    done
    
    # Fail-fast with debug output
    if [[ -z "$ARTIFACTS" ]]; then
      echo "ERROR: No artifacts found"
      # List bundle contents for debugging
      for root in "${ROOTS[@]}"; do
        [[ -d "$root" ]] && find "$root" -type f -maxdepth 5 | head -20
      done
      exit 1
    fi
```

3. **Package macOS .app as .zip (GitHub Releases only accept files):**
```yaml
- name: Package macOS .app as zip
  if: matrix.platform == 'macos-latest'
  shell: bash
  run: |
    APP_FOUND=false
    DMG_FOUND=false
    for root in target/release/bundle src-tauri/target/release/bundle; do
      [[ -d "$root" ]] || continue
      # Check for .dmg
      find "$root" -type f -name "*.dmg" -maxdepth 5 | head -1 | grep -q . && DMG_FOUND=true
      # Package .app as .zip
      APP_DIR=$(find "$root" -type d -name "*.app" -maxdepth 5 | head -1 || true)
      [[ -n "$APP_DIR" ]] && ditto -c -k --sequesterRsrc --keepParent "$APP_DIR" "${APP_DIR}.zip" && APP_FOUND=true
    done
    # Fail if neither found
    [[ "$APP_FOUND" == "true" || "$DMG_FOUND" == "true" ]] || exit 1
```

4. **Use multiline output format for paths with spaces:**
```yaml
{
  echo "artifacts<<EOF"
  echo "$ARTIFACTS"
  echo "EOF"
} >> "$GITHUB_OUTPUT"
```

**Lesson**: Never pass find expressions as quoted strings. Use direct find commands per platform. Always search both workspace output locations. Package directory artifacts (`.app`) as archives. Use multiline `$GITHUB_OUTPUT` format for paths with spaces. Implement fail-fast checks with debug output.

---

### Issue: Duplicate Step IDs in GitHub Actions

**Problem**: Multiple steps with the same `id` caused workflow validation errors.

**Error**:
```
Invalid workflow file: The identifier 'find-artifacts' may not be used more than once within the same scope.
```

**Fix**: Combine platform-specific steps into one with conditional logic:
```yaml
- name: Find bundle artifacts
  id: find-artifacts
  shell: bash
  run: |
    if [[ "${{ matrix.platform }}" == "macos-latest" ]]; then
      # macOS logic
    elif [[ "${{ matrix.platform }}" == "windows-latest" ]]; then
      # Windows logic
    else
      # Linux logic
    fi
```

**Lesson**: Each step `id` must be unique within a job. Use conditional logic within a single step instead of multiple steps with the same ID.

---

### Issue: Build Time Optimization

**Problem**: Initial CI builds were slow due to LTO and single codegen unit.

**Solution**: Optimize for CI speed while maintaining release quality:

1. **Disable LTO in Cargo.toml** (can enable via CI flags for final releases):
```toml
[profile.release]
opt-level = 3
# lto = true  # Disabled for faster CI builds
# codegen-units = 1  # Disabled for parallel compilation
strip = true
panic = "abort"
```

2. **Add sccache for compiler output caching**:
```yaml
- name: Install sccache
  uses: mozilla-actions/sccache-action@v0.0.4
```

3. **Enhance rust-cache configuration**:
```yaml
- name: Rust cache
  uses: swatinem/rust-cache@v2
  with:
    cache-all-crates: true
```

**Lesson**: CI builds should prioritize speed. Use caching (sccache + rust-cache) and disable expensive optimizations (LTO, codegen-units=1) unless specifically needed for final releases. Keep `Cargo.toml` release profile fast by default; apply ThinLTO only in CI via `.cargo/config.toml` for tagged releases.

---

### Issue: tauri-action Compatibility Issues

**Problem**: `tauri-apps/tauri-action@v0` tried to run `tauri init --ci`, overwriting `tauri.conf.json` and failing to detect package name/version.

**Error**:
```
running tauri [ 'init', '--ci' ]
Updating tauri.conf.json file according to these configurations
Could not determine package name and version.
```

**Fix**: Build manually instead of using `tauri-action`:
```yaml
- name: Build frontend
  run: ./build-frontend.sh

- name: Build Tauri app
  working-directory: src-tauri
  run: cargo tauri build

- name: Upload artifacts
  uses: softprops/action-gh-release@v1
```

**Lesson**: `tauri-action` may not work well with existing Tauri v2 projects. Manual build + artifact upload is more reliable.

---

### Issue: CI Tool Installation Performance and Reliability

**Problem**: Installing CLI tools (`trunk`, `tauri-cli`) from scratch on every CI run is slow, flaky (network issues, crates.io hiccups), and wastes CI minutes.

**Solution**: Pin tool versions and cache their installation:

1. **Pin tool versions** for reproducible builds:
```yaml
- name: Install Trunk (cached, pinned version)
  uses: taiki-e/cache-cargo-install-action@v2
  with:
    tool: trunk@0.21.14

- name: Install Tauri CLI (cached, pinned version)
  uses: taiki-e/cache-cargo-install-action@v2
  with:
    tool: tauri-cli@2.5.0
```

2. **Pin Rust toolchain** for determinism:
```yaml
- name: Setup Rust (pinned toolchain)
  uses: dtolnay/rust-toolchain@stable
  with:
    toolchain: 1.89.0
```

3. **Ensure sccache is actually used** by setting `RUSTC_WRAPPER`:
```yaml
env:
  RUSTC_WRAPPER: sccache
```

**Lesson**: Always pin tool versions in release CI for reproducibility. Cache tool installations to avoid network-dependent failures and reduce build times. Set `RUSTC_WRAPPER=sccache` at job level to ensure Cargo actually uses sccache.

---

### Issue: Windows ThinLTO Configuration and Shell Compatibility

**Problem**: ThinLTO configuration using bash heredoc syntax fails on Windows runners (PowerShell by default). Artifact discovery also breaks if using PowerShell `find.exe` instead of Unix `find`.

**Fix**: Use `shell: bash` consistently for all artifact-related steps:
```yaml
- name: Build Tauri app (release with ThinLTO)
  working-directory: src-tauri
  shell: bash  # Force bash for cross-platform heredoc
  run: |
    mkdir -p .cargo
    cat > .cargo/config.toml << 'EOF'
    [profile.release]
    lto = "thin"
    EOF
    cargo tauri build

- name: Find bundle artifacts
  shell: bash  # Avoid PowerShell find.exe confusion
  run: |
    # Direct find commands work consistently across all platforms
```

**Lesson**: Windows runners have bash via Git for Windows. Use `shell: bash` for all steps using bash syntax (heredoc, find, etc.) to ensure cross-platform compatibility. PowerShell's `find.exe` is different from Unix `find` and will break artifact discovery.

---


---

## Release Management & Local Testing

### Local-First Release Gate Strategy

**Principle**: Always test builds locally before pushing to CI. If `cargo tauri build` fails locally, CI will almost certainly fail too (just slower and with worse feedback).

**Local "clean room" validation checklist:**

1. **Clean the repo outputs:**
```bash
cargo clean  # Workspace-wide
rm -rf dist target src-tauri/target  # Frontend and Tauri build outputs
```

2. **Confirm Rust toolchain sanity:**
```bash
rustc -V
cargo -V
# Ensure toolchain matches CI pinned version
```

3. **Confirm WASM target exists:**
```bash
rustup target list --installed | grep wasm32-unknown-unknown
# If missing: rustup target add wasm32-unknown-unknown
```

4. **Build frontend explicitly:**
```bash
./build-frontend.sh  # or ./build-frontend.ps1 on Windows
```

5. **Run the real local proof:**
```bash
cd src-tauri
cargo tauri build
```

**Pass criteria:** Build completes successfully and produces bundles in either:
- `target/release/bundle/` OR
- `src-tauri/target/release/bundle/`

**Lesson**: Never push changes that affect build tooling, dependencies, or packaging without first validating a clean local build. CI should mirror local builds, not discover fundamental issues.

---

### Version Pinning and Update Cadence

**What to pin in release CI:**
- **Rust toolchain** (e.g., 1.89.0)
- **Build tools** (e.g., trunk@0.21.14, tauri-cli@2.5.0)
- **Actions versions** (at least major tags)

**Update cadence:**
- **Every time you prepare a release:**
  - Run `cargo audit` for security vulnerabilities
  - Update pinned versions in `.github/workflows/release.yml` unless a valid blocker prevents update
  - Test locally with new versions before tagging

- **Additionally update if:**
  - A security advisory affects pinned tooling
  - You need a platform/build fix for shipping

**Upgrade procedure:**
1. Update pinned versions in `.github/workflows/release.yml`
2. Run local gate rehearsal: `cargo clean && ./build-frontend.sh && cd src-tauri && cargo tauri build`
3. Run `cargo audit` and resolve vulnerabilities
4. Cut a pre-release tag (e.g., `-rc.1`) to validate CI across OS matrix
5. Promote to stable tag once CI is green and audits pass

**Lesson**: Pinning prevents "surprise breakage" from upstream tool updates. Pins must be reviewed and refreshed before each release. Never use floating versions in release CI.

---

### CI Release Workflow (Tag-Triggered)

**Trigger:** Pushing a git tag matching `v*` (e.g., `v0.1.0-alpha`, `v1.2.3`)

**Purpose:** Produce distributable artifacts for macOS/Windows/Linux and publish to GitHub Releases.

**Key principles:**
- **CI should be boring:** Install prerequisites → run same local build commands → collect artifacts
- **No beforeBuildCommand in CI:** Build frontend explicitly in workflow steps
- **Apply optimizations only in release:** ThinLTO via `.cargo/config.toml`, not in `Cargo.toml` defaults
- **Fail-fast on missing artifacts:** Never publish empty releases

**Workflow structure:**
1. Install prerequisites (Rust, WASM target, native deps)
2. Install and cache tools (Trunk, Tauri CLI) with pinned versions
3. Build frontend explicitly (platform-specific scripts)
4. Configure ThinLTO for release build
5. Build Tauri app
6. Package artifacts (macOS .app → .zip)
7. Find artifacts with fail-fast checks
8. Upload to GitHub Releases

**Lesson**: Keep CI simple and predictable. It should mirror local builds exactly. Use explicit steps rather than relying on Tauri hooks for reliability.

---

### Tagging and Release Process

**1. Prepare release notes (annotated tag message):**

Create a tag message file with sections that apply:
- **Highlights:** 2–4 bullets; highest-impact user-facing changes
- **What's new:** Feature additions and new capabilities
- **Improvements:** Performance, stability, UX polish
- **Breaking changes / required actions:** Explicit user actions, config changes
- **Known issues:** Only include real known issues
- **Install notes:** Signing/notarization status, OS warnings, installer formats

**2. Create annotated tag:**
```bash
git status
git pull --rebase
git tag -a vX.Y.Z[-label] -F /tmp/tag_message.txt
git show vX.Y.Z[-label]  # Verify
```

**3. Push tag to trigger CI release:**
```bash
git push origin vX.Y.Z[-label]
```

**Versioning guidance:**
- Use `vMAJOR.MINOR.PATCH` for stable releases (e.g., `v1.2.3`)
- Use `-alpha`, `-beta`, `-rc.N` for pre-releases (e.g., `v0.1.0-alpha`)
- Prefer monotonic tags (do not re-use or move tags)

**Lesson**: Use annotated tags with release notes for better GitHub Releases. Always verify tag locally before pushing. Pre-release tags (`-rc.N`) allow CI validation before stable release.

---

## Required Build Scripts

### Script Architecture Overview

**Required scripts:**
- **Unix (macOS/Linux):** `dev-frontend.sh`, `build-frontend.sh`
- **Windows:** `dev-frontend.ps1`, `build-frontend.ps1`
- **Cross-platform dispatchers (optional):** `dev-frontend.cmd`, `build-frontend.cmd`

**Design principles:**
1. **Resolve repo root** using script's own location
2. **Unset color-related env vars** (Tauri may pass `NO_COLOR=1` which Trunk interprets as a flag)
3. **Change to `src/` directory** where `Trunk.toml` and `Cargo.toml` are located
4. **Run trunk commands** from the correct directory

**Lesson**: Wrapper scripts are required because Tauri passes incompatible environment variables and Trunk must run from the `src/` directory. Keep scripts simple and focused.

---

### dev-frontend.sh (Unix Development Server)

**Purpose:** Serve frontend with Trunk for `cargo tauri dev` mode.

**Location:** Project root

**Contents:**
```bash
#!/bin/bash
set -e
# Wrapper script to serve frontend with Trunk for Tauri dev mode
# Filters out invalid --no-color flag that Tauri may pass

# Get the script directory (project root)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Unset all color-related environment variables
unset NO_COLOR
unset CLICOLOR
unset CLICOLOR_FORCE

# Change to src directory where Trunk.toml and Cargo.toml are
cd "$SCRIPT_DIR/src" || exit 1

# Check if port 8080 is in use and kill stale trunk processes
if command -v lsof >/dev/null 2>&1; then
  PORT_PID=$(lsof -ti:8080 2>/dev/null || true)
  if [[ -n "$PORT_PID" ]]; then
    # Check if it's a trunk process
    if ps -p "$PORT_PID" -o comm= 2>/dev/null | grep -q trunk; then
      echo "Killing stale trunk process on port 8080 (PID: $PORT_PID)"
      kill "$PORT_PID" 2>/dev/null || true
      sleep 1
    else
      echo "Warning: Port 8080 is in use by another process (PID: $PORT_PID)"
      echo "Trunk will attempt to start anyway and may fail"
    fi
  fi
fi

# Run trunk serve from src directory
exec trunk serve --port 8080
```

**Key features:**
- Handles port 8080 conflicts automatically (kills stale trunk processes)
- Unsets color env vars to prevent Trunk errors
- Uses `exec` to replace shell process with trunk

**Lesson**: Always handle port conflicts in dev scripts. Stale processes from previous `cargo tauri dev` sessions can block new starts.

---

### build-frontend.sh (Unix Production Build)

**Purpose:** Build frontend for production with Trunk.

**Location:** Project root

**Contents:**
```bash
#!/bin/bash
set -e
# Wrapper script to build frontend with Trunk
# Filters out invalid --no-color flag that Tauri may pass

# Get the script directory (project root)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Unset all color-related environment variables
unset NO_COLOR
unset CLICOLOR
unset CLICOLOR_FORCE

# Change to src directory where Trunk.toml and Cargo.toml are
cd "$SCRIPT_DIR/src" || exit 1

# Run trunk from src directory - it will find Cargo.toml and Trunk.toml here
trunk build --release
```

**Lesson**: Keep build scripts simple and focused. They should only handle environment setup and directory navigation.

---

### dev-frontend.ps1 (Windows Development Server)

**Purpose:** PowerShell equivalent of `dev-frontend.sh` for Windows local dev.

**Location:** Project root

**Contents:**
```powershell
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
```

**Lesson**: Provide native PowerShell scripts for Windows contributors who prefer not to use Git Bash. Match functionality of Unix scripts exactly.

---

### build-frontend.ps1 (Windows Production Build)

**Purpose:** PowerShell equivalent of `build-frontend.sh` for Windows CI and local builds.

**Location:** Project root

**Contents:**
```powershell
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
```

**Lesson**: Windows CI requires PowerShell scripts. Keep them simple and match Unix script functionality.

---

### Cross-Platform Dispatchers (Optional)

**dev-frontend.cmd** and **build-frontend.cmd** are optional convenience wrappers that auto-detect platform and call the appropriate script:

```batch
@echo off
REM Cross-platform wrapper for dev frontend
REM On Windows, calls PowerShell script; on Unix, calls bash script
REM Port conflict handling (port 8080) is done in the scripts this dispatches to

if "%OS%"=="Windows_NT" (
    powershell -ExecutionPolicy Bypass -File "%~dp0dev-frontend.ps1"
) else (
    bash "%~dp0dev-frontend.sh"
)
```

**Lesson**: Dispatchers are optional but provide convenience for cross-platform workflows. They don't need port conflict handling since they delegate to scripts that handle it.

---

### Tauri Configuration for Scripts

**tauri.conf.json** configuration:

```json
{
  "build": {
    "beforeDevCommand": "./dev-frontend.sh",
    "frontendDist": "../dist"
  }
}
```

**Key points:**
- `beforeDevCommand` runs from **project root** (not `src-tauri/`), so use `./dev-frontend.sh`
- Do **not** hardcode `bash` in the config (let shebang handle it)
- Do **not** use `beforeBuildCommand` in CI (build frontend explicitly in workflow)
- `frontendDist` is relative to `src-tauri/` directory

**Lesson**: Tauri's `beforeDevCommand` executes from project root. Use relative paths from root, not from `src-tauri/`. For CI, build frontend explicitly rather than relying on `beforeBuildCommand`.

---

## Tauri API & Integration Issues

### Issue: Tauri API Not Available in Frontend

**Problem**: `window.__TAURI__` is `undefined` when frontend tries to call Tauri commands.

**Fix**: 
1. Add to `tauri.conf.json`:
```json
{
  "app": {
    "withGlobalTauri": true
  }
}
```

2. Support both Tauri 1.x and 2.x API paths:
```rust
let invoke = window.__TAURI__?.core?.invoke || 
             window.__TAURI__?.tauri?.invoke || 
             window.__TAURI__?.invoke;
```

**Lesson**: Always enable `withGlobalTauri` and check for both API paths (core.invoke vs tauri.invoke). Centralize Tauri API interactions in a helper module for easier maintenance.

---

### Issue: Tauri Command Argument Serialization

**Problem**: Commands receive `"missing required key filePaths"` error.

**Root Cause**: Arguments were serialized as JSON strings, not JavaScript objects. Tauri expects camelCase keys that map to Rust snake_case.

**Fix**:
1. Parse JSON string into JavaScript object:
```rust
let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
    .unwrap_or_else(|_| JsValue::NULL);
```

2. Use camelCase in frontend, snake_case in Rust with `#[serde(rename)]`:
```rust
#[derive(Serialize, Deserialize)]
struct CompressBatchArgs {
    #[serde(rename = "filePaths")]
    file_paths: Vec<String>,
}
```

**Lesson**: Tauri automatically maps camelCase (JS) to snake_case (Rust), but arguments must be JavaScript objects, not JSON strings.

---

### Issue: Tauri Dialog API Changes

**Problem**: `FileDialogBuilder` methods changed between Tauri versions.

**Fix**: Tauri v2 uses callback-based methods:
```rust
app.dialog()
    .file()
    .add_filter("Images", &["png", "jpg", "jpeg", ...])
    .pick_files(move |paths| {
        // Handle paths
    });
```

**Lesson**: Tauri v2 dialog API uses callback-based methods instead of builder pattern for some operations.

---

### Issue: Tauri Native Drag & Drop

**Problem**: HTML drag & drop events don't work in Tauri window.

**Fix**:
1. Listen for Tauri native events:
```javascript
const eventApi = window.__TAURI__.event || window.__TAURI__.core?.event;
eventApi.listen('tauri://drag-drop', (event) => {
    const filePaths = event.payload?.paths || [];
    // Process files
});
```

2. Grant event permissions in `capabilities/main.json`:
```json
{
  "permissions": [
    "core:event:allow-listen",
    "core:event:allow-emit"
  ]
}
```

**Lesson**: Use Tauri's native drag & drop events instead of HTML5 drag & drop for better security and cross-platform compatibility.

---

## Leptos Reactive System Issues

### Issue: "Signal Accessed Outside Reactive Tracking" Warnings

**Problem**: Leptos warns about accessing signals outside reactive contexts.

**Fix**:
1. Use `get_untracked()` in event handlers and async contexts:
```rust
on:click=move |_| {
    if state.is_compressing.get_untracked() {
        return;
    }
}
```

2. Use closures in `view!` macros:
```rust
view! {
    <button>
        {move || button_text.get()}
    </button>
}
```

**Lesson**: Always use `get_untracked()` in event handlers and async contexts. Use closures (`move || signal.get()`) in view macros for reactive updates.

---

### Issue: Move Errors in Closures

**Problem**: "cannot move out of captured variable" errors in Leptos components.

**Fix**: Clone before the closure captures it:
```rust
let format_str_for_click_cloned = format_str_for_click.clone();
view! {
    <button
        on:click=move |_| {
            state.output_format.set(format_str_for_click_cloned.clone());
        }
    >
        {label}
    </button>
}
```

**Lesson**: Clone values before move closures capture them, and clone again inside if needed.

---

### Issue: Deprecated `create_rw_signal`

**Problem**: Using deprecated `create_rw_signal()` function.

**Fix**: Use `RwSignal::new()` instead:
```rust
let files = RwSignal::new(Vec::new());
```

**Lesson**: Always use `RwSignal::new()` instead of `create_rw_signal()` in Leptos 0.7+.

---

## File System & Dialog Issues

### Issue: Save Dialog Shows Full Path / FilePath Type Conversion

**Problems**: 
1. Save dialog filename field shows full path instead of just filename
2. `FilePath` enum can't be used directly with `std::path::Path`

**Fixes**:
1. Extract only filename for dialog:
```rust
let file_name_only = Path::new(&default_name)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or(&default_name)
    .to_string();

app.dialog()
    .file()
    .set_file_name(&file_name_only)
    .save_file(move |path| { ... });
```

2. Convert `FilePath` to `PathBuf`:
```rust
let path_buf = save_path.as_path()
    .ok_or_else(|| "Invalid path".to_string())?
    .to_path_buf();
fs::write(&path_buf, data)?;
```

**Lesson**: Save dialogs should only receive filenames. Always convert `FilePath` to `PathBuf` before using standard library path operations.

---

### Issue: Batch Save Dialog UX

**Problem**: "Download All" opened individual save dialogs for each file.

**Fix**: Implement single folder picker + batch save:
```rust
let output_folder = select_output_folder().await?;
save_files_to_folder(output_folder, files).await?;
```

**Lesson**: For batch operations, use a single folder picker instead of multiple file dialogs.

---

## UI/UX Issues

### Issue: Progress Bar Invisible on Black Background

**Problem**: Progress bar uses light colors that don't contrast with dark background.

**Fix**: Use subtle, semi-transparent colors with glow:
```css
.progress {
    background: rgba(255, 255, 255, 0.10);
}

.progress-bar {
    background: rgba(96, 165, 250, 0.65);
    box-shadow: 0 0 8px rgba(96, 165, 250, 0.25);
}
```

**Lesson**: Use rgba colors with low opacity and subtle glows for UI elements on dark backgrounds.

---

### Issue: Advanced Options Dropdown Not Opening

**Problem**: Clicking "Advanced Options" doesn't toggle the dropdown.

**Fix**: CSS class binding must match actual CSS classes:
```rust
<div class="advanced-options" class:show=move || state.advanced_open.get()>
    <Show when=move || state.advanced_open.get()>
        {/* Content */}
    </Show>
</div>
```

**Lesson**: Leptos class bindings must match the actual CSS classes.

---

### Issue: Window Auto-Resize

**Problem**: Window doesn't automatically resize to fit content when dropdown opens.

**Fix**: Use `ResizeObserver` in JavaScript:
```javascript
const resizeObserver = new ResizeObserver(entries => {
    const rect = entries[0].contentRect;
    window.__TAURI__.core.invoke('resize_window', {
        width: rect.width,
        height: rect.height
    });
});
resizeObserver.observe(container);
```

**Lesson**: Use browser APIs (ResizeObserver) for UI-driven window resizing, not Leptos signals.

---

### Issue: File Selection Disabled During Compression

**Problem**: Users could select new files while compression was in progress.

**Fix**:
1. Add disabled state CSS:
```css
.upload-section.disabled {
    opacity: 0.55;
    filter: grayscale(1);
    pointer-events: none;
}
```

2. Bind disabled state:
```rust
<div class="upload-section" class:disabled=move || state.is_compressing.get()>
    <button disabled=move || state.is_compressing.get()>
        "Select Images"
    </button>
</div>
```

**Lesson**: Always provide visual feedback and disable interactive elements during async operations. Reset related state when user input changes to prevent UI inconsistencies.

---

## Performance & Optimization

### Issue: Verbose Debug Logging

**Problem**: Console logs contained full file paths and large JSON payloads.

**Fix**:
1. Create utility functions for path redaction:
```rust
pub fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}
```

2. Gate logging by build mode:
```rust
if cfg!(debug_assertions) {
    web_sys::console::log_1(&JsValue::from_str("Debug message"));
}
```

3. Log summaries instead of full data:
```rust
web_sys::console::log_1(&JsValue::from_str(&format!(
    "compress_batch: {} files, format={}, sample={:?}",
    count, format, sample_names
)));
```

**Lesson**: Always redact paths in logs, gate debug logging by build mode, and log summaries instead of full payloads.

---

### Issue: Drag-Over Event Spam

**Problem**: `drag-over` events fire constantly, spamming the console.

**Fix**: Only log `drag-enter` and `drag-drop`, not `drag-over`.

**Lesson**: Be selective about which events to log. High-frequency events like `drag-over` should be silent.

---

## Static Assets & Image Rendering

### Issue: Images Don't Render in Tauri Production Builds

**Problem**: Images work in development (`cargo tauri dev`) but fail to load in production builds (`cargo tauri build`). Console shows 404 errors for image requests.

**Root Cause**: Tauri serves the frontend from a non-root base path in production builds. Absolute paths like `/assets/icon.png` break because they resolve from the wrong base URL. Even relative paths can fail if the `public-url` in Trunk.toml is set incorrectly.

**Solution**: Use the following setup for reliable image rendering in both dev and production:

### 1. File Structure

Place static assets (images, fonts, etc.) at the **project root**, not inside `src/`:

```
project-root/
├── assets/              # Static assets here (at root)
│   ├── icon_32x32.png
│   └── logo.png
├── src/                 # Leptos frontend code
│   ├── Trunk.toml
│   ├── index.html
│   ├── app.rs
│   └── ...
└── src-tauri/           # Tauri backend
```

**Why**: Trunk's asset handling works best when assets are outside the source directory. This also keeps assets separate from code.

### 2. Trunk Configuration

Set `public-url = "./"` in `src/Trunk.toml` (critical for Tauri builds):

```toml
[build]
target = "index.html"
dist = "../dist"
public-url = "./"  # Must be "./" not "/" for Tauri production builds
```

**Why**: The `./` base URL ensures paths resolve correctly in Tauri's production environment, where the app is served from a non-root path.

### 3. Declare Assets in index.html

Use `data-trunk` directives to tell Trunk to copy assets:

```html
<!DOCTYPE html>
<html>
<head>
    <link data-trunk rel="css" href="style.css"/>
    <link data-trunk rel="copy-dir" href="../assets"/>
</head>
<body>
    <div id="root"></div>
</body>
</html>
```

**Options**:
- `copy-dir`: Copies entire directory (e.g., `../assets` copies all files in `assets/`)
- `copy-file`: Copies a single file (e.g., `../assets/icon.png`)

**Path Note**: Since `Trunk.toml` is in `src/`, reference the root `assets/` folder as `../assets`.

### 4. Reference Images in Leptos Components

Use **relative paths with `./` prefix** in Leptos `view!` macros:

```rust
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="header">
            <h1>
                <img 
                    src="./assets/icon_32x32.png" 
                    alt="App Logo" 
                    class="header-logo"
                />
                "App Name"
            </h1>
        </div>
    }
}
```

**Key Points**:
- Always use `./assets/...` (relative path with `./` prefix)
- Never use `/assets/...` (absolute path - breaks in production)
- Never use `assets/...` without `./` (can break with nested routes)

### 5. CSS Background Images

For CSS background images, use the same relative path pattern:

```css
.header-logo {
    width: 32px;
    height: 32px;
    background-image: url('./assets/icon_32x32.png');
    background-size: contain;
}
```

### 6. Verify Build Output

After building, verify assets are copied correctly:

```bash
# Check that assets exist in dist/
ls -la dist/assets/

# Should show:
# icon_32x32.png
# logo.png
# etc.
```

### Complete Example

**Project Structure**:
```
rust-tinypng-clone/
├── assets/
│   └── icon_32x32.png
├── src/
│   ├── Trunk.toml        # public-url = "./"
│   ├── index.html        # <link data-trunk rel="copy-dir" href="../assets"/>
│   └── app.rs            # <img src="./assets/icon_32x32.png" />
└── dist/                 # Generated by Trunk
    └── assets/
        └── icon_32x32.png
```

**Trunk.toml**:
```toml
[build]
target = "index.html"
dist = "../dist"
public-url = "./"  # Critical for Tauri
```

**index.html**:
```html
<link data-trunk rel="copy-dir" href="../assets"/>
```

**app.rs**:
```rust
view! {
    <img src="./assets/icon_32x32.png" alt="Logo" class="header-logo"/>
}
```

### Common Mistakes

1. **Using absolute paths** (`/assets/...`):
   - Works in dev, breaks in production
   - Fix: Use `./assets/...`

2. **Wrong `public-url`** (`public-url = "/"`):
   - Causes path resolution issues in Tauri builds
   - Fix: Use `public-url = "./"`

3. **Assets in `src/` directory**:
   - Can work but is not recommended
   - Fix: Move to project root `assets/` folder

4. **Missing `copy-dir` or `copy-file`**:
   - Assets not copied to `dist/`
   - Fix: Add `<link data-trunk rel="copy-dir" href="../assets"/>` to `index.html`

5. **Relative path without `./`**:
   - Can break with nested routes or base href changes
   - Fix: Always use `./assets/...`

### Debugging Image Loading

If images still don't load:

1. **Check browser devtools Network tab**:
   - Look for 404 errors
   - Note the requested URL vs. actual file location

2. **Verify file exists in dist/**:
   ```bash
   ls -la dist/assets/
   ```

3. **Check Trunk build output**:
   - Look for "Copying" messages during build
   - Verify no errors about missing files

4. **Test path resolution**:
   - In devtools console: `new URL('./assets/icon.png', window.location.href)`
   - Should resolve to the correct path

**Lesson**: For Tauri + Trunk builds, always use `public-url = "./"`, place assets at project root, declare them with `copy-dir` in `index.html`, and reference them with `./assets/...` relative paths. This ensures images work in both development and production builds.

---

## Developer Tools & Debugging

### Best Practice: Enable Devtools in Dev, Disable in Production

**Problem**: Devtools should be available during development but not in production builds.

**Solution**: Use Rust's `#[cfg(debug_assertions)]` attribute to conditionally enable devtools.

**Implementation in `src-tauri/src/main.rs`**:
```rust
.setup(|app| {
    #[cfg(debug_assertions)]
    {
        use tauri::Manager;
        if let Some(webview) = app.get_webview_window("main") {
            // Auto-open devtools in debug mode only
            std::thread::spawn({
                let app_handle = app.handle().clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Some(webview) = app_handle.get_webview_window("main") {
                        webview.open_devtools();
                    }
                }
            });
        }
    }
    Ok(())
})
```

**How it works**:
- `#[cfg(debug_assertions)]` is a Rust conditional compilation attribute
- When `cargo tauri dev` runs, `debug_assertions` is `true` → devtools open automatically
- When `cargo tauri build --release` runs, `debug_assertions` is `false` → code is completely removed from the binary
- This ensures zero overhead and no devtools in production builds

**Alternative: Check at Runtime** (if you need more control):
```rust
.setup(|app| {
    if cfg!(debug_assertions) {
        // Dev-only setup
    }
    Ok(())
})
```

**Frontend Keyboard Shortcut** (optional, for manual devtools):
```javascript
// In index.html - always available, but only works if devtools are enabled
document.addEventListener('keydown', (e) => {
    if (e.key === 'F12' || (e.ctrlKey && e.shiftKey && e.key === 'I')) {
        if (window.__TAURI__) {
            const invoke = window.__TAURI__.core?.invoke || 
                          window.__TAURI__.tauri?.invoke;
            if (invoke) {
                invoke('open_devtools').catch(() => {});
            }
        }
    }
});
```

**Lesson**: Always use `#[cfg(debug_assertions)]` for dev-only features. This ensures production builds are clean, secure, and have zero performance overhead from debug code.

---

## Application Icon Setup

### Icon Requirements

For best results when building the application:

1. **Format**: PNG only (1024×1024px recommended)
2. **Transparency**: No transparency on the edges (macOS hates soft alpha)
3. **Design**: Centered design with padding (~10–15% margin)
4. **Location**: Place source icon in `/assets/icon.png`

### Setup Process

1. **Create source icon**: Design a 1024×1024px PNG with no edge transparency
2. **Generate platform icons**: Use `cargo tauri icon` to generate all required sizes:
```bash
cargo install cargo-tauri-icon
cargo tauri icon assets/icon.png
```

This generates all required sizes in `src-tauri/icons/`:
- `icon.icns` (macOS)
- `icon.ico` (Windows)
- Various PNG sizes (32x32, 128x128, etc.)

3. **Verify configuration**: Icons are already configured in `tauri.conf.json`:
```json
{
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

4. **Build**: The icon will be included automatically:
```bash
cargo tauri build
```

### Common Issues

- **Soft alpha edges**: macOS may reject icons with transparency on edges. Use solid backgrounds or ensure transparency is fully opaque.
- **Wrong size**: Source icon should be 1024×1024px for best quality across all platforms.
- **Missing icons**: Ensure `cargo tauri icon` completes successfully and all files are generated.

---

## Key Takeaways

1. **Tauri v2 Breaking Changes**: Many APIs changed from v1. Always check version-specific documentation.
2. **Leptos Reactive Tracking**: Use `get_untracked()` in event handlers, closures in views.
3. **Argument Serialization**: Tauri expects JavaScript objects, not JSON strings. Use camelCase in JS, snake_case in Rust.
4. **Native Drag & Drop**: Use Tauri events, not HTML5 drag & drop.
5. **Path Redaction**: Always redact full paths in logs for privacy.
6. **State Management**: Reset related state when user input changes.
7. **Build Scripts**: Required wrapper scripts handle Tauri's environment variables and directory context. Always test locally before pushing to CI.
8. **Release Management**: Always validate clean local builds before tagging. Pin tool versions in CI for reproducibility. Use fail-fast artifact checks.
9. **Port Conflicts**: Dev scripts should automatically handle port conflicts (kill stale processes) to prevent "Address already in use" errors.
8. **Devtools**: Auto-open in debug mode for better DX.
9. **Error Handling**: Distinguish between user cancellations (silent) and actual errors (show).
10. **UI Feedback**: Always provide visual feedback during async operations (disabled states, progress bars).
11. **Icon Setup**: Use 1024×1024px PNG with no edge transparency, centered design with padding.
12. **Static Assets**: Use `public-url = "./"` in Trunk.toml, place assets at project root, declare with `copy-dir` in index.html, and reference with `./assets/...` relative paths.

---

*Last Updated: After CI workflow optimization and release management implementation*
