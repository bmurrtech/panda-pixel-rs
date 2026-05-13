# Testing Guide

## Overview

This guide covers **tests**, **quality gates**, **WASM checks**, and **smoke** workflows for the Rust monorepo.

**Repository layout, web vs desktop, and contributor workflow** are in [contributing.md](contributing.md).

The **primary automated correctness gate** is:

```bash
cargo test --workspace
```

Run it before every push, along with format, Clippy, and WASM checks below.

---

## Testing Strategy
- **Unit tests**: Co-located with code in `#[cfg(test)]` modules (preferred default)
- **Integration tests**: HTTP-level tests in `apps/api/tests/` only
- **WASM frontend**: Manual validation via `trunk serve` + API integration tests
- **Fixtures**: Centralized under `tests/fixtures/` (small, deterministic)

---

## One-Time Setup

### Step 1: Install the Rust Toolchain
The project uses a pinned Rust toolchain. Rust will automatically prompt you to install it when you run `cargo` commands, or you can install it explicitly:

```bash
# From the repository root
rustup toolchain install stable
```

The toolchain version is specified in `rust-toolchain.toml` at the repository root.

### Step 2: Install WASM Target
The web frontend compiles to WebAssembly. Install the target:

```bash
rustup target add wasm32-unknown-unknown
```

**Expected output**: `info: component 'rust-std' for target 'wasm32-unknown-unknown' is up to date`

### Step 3: Install Trunk (WASM Bundler)
Trunk is used to build and serve the web frontend:

```bash
cargo install trunk
```

**Expected output**: Installation completes without errors. Verify with `trunk --version`.

### Step 4: Install Node.js Dependencies for TailwindCSS
The web frontend uses TailwindCSS, which requires Node.js packages:

```bash
# Navigate to the web app directory
cd apps/web

# Install dependencies
npm install

# Return to repository root
cd ../..
```

**Expected output**: `npm` creates `node_modules/` and installs packages. You should see `added X packages` message.

### Step 5: Verify Setup (Optional)
Run a quick check to ensure everything is set up:

```bash
# From repository root
cargo check --workspace
```

**Expected output**: All packages compile successfully (may take a few minutes on first run).

---

## Primary quality gates (run before pushing)

Run these from the **repository root** unless noted.

### Step 1: Tests (main validator)

```bash
cargo test --workspace
```

**Expected output**: All packages’ tests pass. If something fails, fix or update tests in the same change.

### Step 2: Format

```bash
cargo fmt --check
```

If this fails:

```bash
cargo fmt
cargo fmt --check
```

### Step 3: Lint

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### Step 4: Workspace compile

```bash
cargo check --workspace
```

### Step 5: WASM compile checks

There are **two** Leptos frontends:

1. **Workspace member** `apps/web` (package `panda_pixel_rs_web`):

```bash
cargo check -p panda_pixel_rs_web --target wasm32-unknown-unknown
```

2. **Tauri UI** under `src/` (not a workspace member; package `padna_pixel_rs_frontend`):

```bash
cargo check --manifest-path src/Cargo.toml --target wasm32-unknown-unknown
```

### Step 6: Optional Trunk release build (`apps/web`)

```bash
cd apps/web && trunk build --release && cd ../..
```

Confirms Tailwind + Trunk for the workspace web app. The **desktop** bundle uses Trunk from `src/` via Tauri; validate that path with `cargo tauri build` when you change the embedded UI.

---

## Running Tests

### Run All Tests
Execute all tests across the entire workspace:

```bash
# From repository root
cargo test --workspace
```

**What this does**: Runs unit tests in all packages (domain, compression, api, desktop).

**Expected output**: Summary showing all tests passed.

### Run Tests for a Specific Package
Test individual packages:

```bash
# Test the domain crate (compression options, utilities)
cargo test -p domain

# Test the compression crate (image processing)
cargo test -p compression

# Test the API (HTTP endpoints)
cargo test -p api

# Test the desktop app
cargo test -p panda_pixel_rs_desktop
```

**Expected output**: Tests for that specific package run and pass.

> **Note**: Leptos frontends (`apps/web` as `panda_pixel_rs_web`, and `src/` as `padna_pixel_rs_frontend` for Tauri) rely on **WASM compile checks** and **manual smoke** more than workspace unit tests in those crates. Add tests in `crates/compression`, `crates/domain`, and `apps/api` for shared behavior.

### Run a Single Test or Pattern
Run specific tests by name:

```bash
# Run a test with a specific name
cargo test test_quality_range_parsing

# Run all tests matching a pattern
cargo test quality
cargo test compression
```

**Expected output**: Only matching tests run.

### Show Test Output (Including Passing Tests)
By default, passing tests don't print output. To see all output:

```bash
cargo test --workspace -- --nocapture
```

**Use case**: Debugging test failures or verifying test behavior.

### Run Ignored Tests
If you have intentionally marked slow tests as `#[ignore]`:

```bash
cargo test --workspace -- --ignored
```

**Use case**: Running comprehensive but slow tests before releases.

---

## Parity and pre-release checks

Behavioral parity between **desktop (Tauri)** and **web (WASM + API)** is tracked in [parity-matrix.md](parity-matrix.md). Automated regression is centered on the **shared Rust crates** the UI depends on.

### Focused test runs (after touching compression or API)

```bash
cargo test -p compression
cargo test -p domain
cargo test -p api
```

### API integration (HTTP contract)

```bash
cargo test -p api --test api_tests
```

### Pre-release (optional stricter pass)

```bash
cargo test --workspace
# Optional clean rebuild if you suspect stale artifacts:
# cargo clean && cargo test --workspace
```

Then run **manual smoke** below for the surfaces you changed (desktop, `apps/web`, and/or `src/` + API).

---

## Unit Tests

### Where Unit Tests Live
Unit tests are co-located with the code they test:

- **`crates/domain/src/lib.rs`**: Tests for `CompressionOptions`, quality parsing, compression level presets
- **`crates/compression/src/lib.rs`**: Tests for PNG/JPEG compression, format conversions
- **`apps/api/src/config.rs`**: Tests for configuration loading and validation (if present)

### Unit Test Template
Here's a standard unit test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Arrange: Set up test data
        let input = "50-80";

        // Act: Call the function being tested
        let result = parse_quality_range(input);

        // Assert: Verify the expected outcome
        assert_eq!(result, (50, 80));
    }
}
```

### Fixture vs Generated Images
- **Prefer programmatic generation** for simple test cases (e.g., creating a 100x100 red PNG in code)
- **Use fixtures** for edge cases that are hard to generate:
  - EXIF metadata oddities
  - Large dimension images
  - Corrupted files
  - Real-world format variations

Fixtures are stored in `tests/fixtures/` at the repository root.

---

## Integration Tests (API Only)

### What Integration Tests Cover
Integration tests validate the HTTP contract of the API:

- `POST /api/compress` - Single image compression
- `POST /api/compress/batch` - Multiple image compression
- Error responses (bad input, unsupported format, missing parts)
- CORS headers (if enabled)

### Run API Integration Tests
From the repository root:

```bash
cargo test -p api --test api_tests
```

**Expected output**: Integration tests run and pass.

### Run One Integration Test
Run a specific integration test:

```bash
cargo test -p api --test api_tests test_compress_endpoint
```

**Expected output**: Only that test runs.

### Integration Test Expectations
- Tests must spin up the app in-process (or bind to an ephemeral port)
- Tests must not rely on a `.env` file
- Tests should not hit external services
- Tests should use fixtures from `tests/fixtures/` when needed

---

## Fixtures

### Location (Single Source of Truth)
All test fixtures live here:

```
tests/fixtures/
```

### Recommended Structure
```
tests/fixtures/
├─ small.png          # Small test PNG image
├─ small.jpg          # Small test JPEG image
├─ odd_exif_rotation.jpg  # JPEG with unusual EXIF data
└─ corrupted.png      # Intentionally corrupted file for error testing
```

### Fixture Rules
- **Keep fixtures small**: Prefer files under 100KB
- **Prefer deterministic files**: Files that produce consistent results
- **Do not store large datasets**: Don't commit large "real world" images to git

### Loading Fixtures in Tests
Here's how to load fixtures in your tests:

```rust
use std::fs;

fn read_fixture(name: &str) -> Vec<u8> {
    fs::read(format!("tests/fixtures/{name}"))
        .expect("fixture missing or unreadable")
}

#[test]
fn test_with_fixture() {
    let image_data = read_fixture("small.png");
    // Use image_data in your test
}
```

---

## Manual smoke (browser + API)

Leptos in the browser needs a **running API** when using the HTTP backend. Use **two terminals**. Avoid port clashes: run the API on **8081** and Trunk on **8080** (or adjust consistently and update `CORS_ALLOWED_ORIGINS`).

### Step 1: API server

```bash
# From repository root
export APP_ENV=development
export PORT=8081
export CORS_ALLOWED_ORIGINS="http://localhost:8080"
export RUST_LOG=info

cargo run -p api
```

Keep this terminal open.

### Step 2: Trunk (pick one UI tree)

**Variant A — `apps/web` (workspace member)**

```bash
cd apps/web
trunk serve
```

Open the URL Trunk prints (often `http://localhost:8080`).

**Variant B — `src/` (same UI Tauri dev loads)**

```bash
cd src
trunk serve --port 8080
```

Use the same API base URL / CORS your build expects (if the UI defaults to `localhost:8081`, align env; otherwise configure per app).

### Step 3: Browser checklist

- [ ] App loads without console errors; styles look correct
- [ ] Select image(s), compress, results appear
- [ ] Invalid file (e.g. `.txt`) shows a clean error
- [ ] DevTools → Network: requests hit the expected API host/port; no secrets in payloads

### Step 4: Cleanup

`Ctrl+C` in each terminal.

---

## Desktop validation

Tauri loads the UI from `src/` (see `src-tauri/tauri.conf.json`). Compression and FS use **Tauri commands** unless your build routes through the HTTP backend; see [parity-matrix.md](parity-matrix.md).

### Build Desktop App
From the repository root:

```bash
cargo tauri build
```

Equivalent from `src-tauri/`:

```bash
cd src-tauri
cargo tauri build
```

Tauri's `beforeBuildCommand` (in `src-tauri/tauri.conf.json`) runs `trunk build --release` with working directory `src/`.

**Expected output**: 
- Compilation completes
- Tauri bundles the app
- Output binary is `target/release/panda_pixel_rs_desktop`
- Output bundles are in `target/release/bundle/`
- Platform-specific bundles are created (`.app` on macOS, `.exe` on Windows, etc.)

### Run Desktop App in Dev Mode
For development and testing:

```bash
# From repository root
cargo tauri dev
```

Equivalent from `src-tauri/`:

```bash
cd src-tauri
cargo tauri dev
```

**Expected output**:
- Desktop window opens
- App runs without deployment secrets
- Compression works via Tauri command channel
- DevTools open automatically in debug mode

---

## Tailwind + Trunk Checks (Web)

These checks catch common Tailwind/Trunk integration issues.

### Confirm Tailwind Scans Rust Sources
Check that `apps/web/tailwind.config.js` includes:

```javascript
content: [
  "./src/**/*.rs",    // Scans Rust source files
  "./index.html",     // Scans HTML file
],
```

**Why this matters**: Tailwind only includes CSS for classes it finds. If Rust files aren't scanned, classes in `view! { class="..." }` macros won't be included.

### Confirm Tailwind Output Location
Ensure Tailwind generates CSS to a single location:

- Tailwind config should output to: `apps/web/public/app.css`
- `apps/web/index.html` should link: `<link data-trunk rel="css" href="public/app.css"/>`

### Verify Trunk Build Includes CSS
Run a build and verify:

```bash
cd apps/web
trunk build --release
```

**Check**: The `dist/` directory should contain `app.css` with Tailwind utilities.

---

## Using the Justfile (Optional but Recommended)

The repository includes a `justfile` with convenient commands. If you have `just` installed:

```bash
# Install just (one-time)
cargo install just

# Then use these commands:
just fmt          # Format check
just lint         # Lint check
just test         # Run all tests
just check-web    # Check WASM compilation
just build-web    # Build web frontend
just serve-web    # Serve web frontend
just run-api      # Run API server
just validate     # Run all quality gates
```

**Benefits**: Shorter commands and consistent workflows across the team.

---

## Troubleshooting

### WASM Compilation Fails

**Symptoms**: `cargo check -p panda_pixel_rs_web --target wasm32-unknown-unknown` or `cargo check --manifest-path src/Cargo.toml --target wasm32-unknown-unknown` fails.

**Checklist**:

1. A native-only crate was added to the **failing** frontend `Cargo.toml`
2. A dependency enables native default features
3. A dependency does not support the WASM target

**Fix**:

- Remove or gate the dependency; prefer `default-features = false`
- Keep heavy codecs in `crates/compression` (native / server), not in WASM crates

### API Integration Tests Fail
**Checklist**:
1. Ensure tests do not depend on `.env` file
2. Ensure API can start with minimal env vars in test harness
3. Ensure fixtures exist in `tests/fixtures/` if tests reference them
4. Avoid fixed ports; prefer ephemeral ports for test servers

**Fix**: Update test code to set required environment variables programmatically.

### Tailwind Styles Missing
**Checklist**:
1. `npm install` completed in `apps/web/`
2. Tailwind build step runs before/with Trunk build (check `Trunk.toml` hooks)
3. `index.html` links the correct generated CSS path
4. Tailwind `content` includes Rust source globs (`./src/**/*.rs`)

**Fix**: 
```bash
cd apps/web
npm install
npm run build:css  # Manually build CSS to verify
```

### Clippy Fails with Warnings Treated as Errors
**Symptoms**: `cargo clippy` fails even though code compiles.

**Fix**: 
- Fix the warnings rather than allowing them
- If you must allow, scope `#[allow(clippy::warning_name)]` narrowly with justification
- Never use `#[allow(warnings)]` globally

### Tests Fail After Refactoring
**Symptoms**: Tests pass individually but fail when run together.

**Possible causes**:
- Shared mutable state between tests
- Tests depending on execution order
- File system or network state not cleaned up

**Fix**: Ensure tests are isolated and don't share state.

---

## Test Coverage Expectations (Minimal, High-Value)

The project focuses on high-value tests rather than 100% coverage:

- **Compression core**: Quality parsing, presets, invalid inputs, representative format cases
- **Domain types**: Validation + (de)serialization of `CompressionOptions`
- **API config**: Missing required env vars → startup failure
- **API endpoints**: Success + error cases for `/api/compress` and `/api/compress/batch`
- **WASM**: Compile guardrail + manual smoke test (Trunk + API)

---

## "Ready to Push" checklist

Before pushing your changes, verify all of these:

- [ ] **Tests**: `cargo test --workspace` passes
- [ ] **Format**: `cargo fmt --check` passes
- [ ] **Lint**: `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] **WASM**: Both checks pass:
  - `cargo check -p panda_pixel_rs_web --target wasm32-unknown-unknown`
  - `cargo check --manifest-path src/Cargo.toml --target wasm32-unknown-unknown`
- [ ] **Web build** (if you touched `apps/web`): `cd apps/web && trunk build --release`
- [ ] **Manual smoke** (if you touched WASM UI or API): API + Trunk as in [Manual smoke](#manual-smoke-browser--api); optional `cargo tauri dev` for desktop

### Test contribution notes (before PR)

- Add new cross-crate or scenario tests under the top-level `tests/` directory.
- Keep unit tests co-located in `#[cfg(test)]` modules for module-private logic.
- For bug fixes, include a regression test that would fail before the fix.
- Keep fixtures small and deterministic under `tests/fixtures/`.
- If a test is expensive, mark it `#[ignore]` with a clear reason and run instructions.

**Optional:** If the repo provides a `justfile`, `just validate` may bundle fmt, lint, test, and WASM checks—use it only if maintained alongside this document.

---

## Quick Reference

### Package names

- `domain` — domain types and utilities
- `compression` — image compression logic
- `api` — HTTP API server
- `panda_pixel_rs_web` — `apps/web` Leptos WASM (workspace member)
- `padna_pixel_rs_frontend` — `src/` Leptos WASM (Tauri UI; use `--manifest-path src/Cargo.toml`)
- `panda_pixel_rs_desktop` — Tauri desktop (`src-tauri/`)

### Key directories

- `apps/web/` — workspace web frontend
- `apps/api/` — API server
- `src-tauri/` — desktop shell
- `src/` — Leptos UI embedded by Tauri (Trunk)
- `crates/domain/`, `crates/compression/` — shared libraries
- `tests/fixtures/` — test binaries

### Common commands

```bash
cargo test --workspace
cargo test -p compression
cargo test -p api --test api_tests
cargo check -p panda_pixel_rs_web --target wasm32-unknown-unknown
cargo check --manifest-path src/Cargo.toml --target wasm32-unknown-unknown
cargo tauri dev
make dev-web
export PORT=8081 CORS_ALLOWED_ORIGINS="http://localhost:8080" && cargo run -p api
```

---

## Getting help

1. Read the error output (Rust is usually explicit).
2. Confirm [One-time setup](#one-time-setup) and that commands run from the repo root unless stated otherwise.
3. See [contributing.md](contributing.md) for layout and web vs desktop.
4. See [troubleshooting.md](troubleshooting.md) for shell/env quirks (`CI`, `NO_COLOR`, run directory), and [Troubleshooting](#troubleshooting) in this file for test/build flow.

For product questions, use GitHub Issues or Discussions.
