# Testing Guide

## Overview
This guide explains how to run tests, quality gates, and smoke checks for the Rust monorepo.

It is intentionally practical: run the commands, confirm the expected outcomes, and ship.

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

## Prek Setup (Pre-Commit Quality Gates)

The repository uses Prek as the invariant enforcement layer and CI gate. Prek runs fast, deterministic checks that must pass before code can be committed or merged.

### Step 1: Install Prek
Prek is a modern pre-commit hook manager:

```bash
# Install via uv (recommended)
uv tool install prek

# Or via pip
pip install pre-commit

# Or via brew (macOS)
brew install pre-commit
```

### Step 2: Bootstrap Prek Configuration
Prek configuration is defined in `prek.toml` at the repository root:

```toml
# prek.toml
[[repos]]
repo = "builtin"
hooks = [
  { id = "trailing-whitespace", args = ["--markdown-linebreak-ext=md"] },
  { id = "end-of-file-fixer" },
  { id = "mixed-line-ending", args = ["--fix=auto"] },
  { id = "check-toml" },
  { id = "check-yaml", args = ["--allow-multiple-documents"] },
  { id = "check-json" },
  { id = "check-merge-conflict" },
  { id = "detect-private-key" },
  { id = "check-added-large-files", args = ["--maxkb", "1024", "--enforce-all"] },
  { id = "check-case-conflict" },
  { id = "check-symlinks" },
]

[[repos]]
repo = "local"
hooks = [
  { id = "rust-fmt", name = "Rust Format Check", entry = "cargo fmt --check --all", language = "system", pass_filenames = false, require_serial = true },
  { id = "rust-clippy", name = "Rust Clippy", entry = "cargo clippy --workspace --all-targets -- -D warnings", language = "system", pass_filenames = false, require_serial = true },
  { id = "rust-check", name = "Rust Compilation Check", entry = "cargo check --workspace", language = "system", pass_filenames = false, require_serial = true },
  { id = "rust-test", name = "Rust Unit Tests", entry = "cargo test --workspace --lib", language = "system", pass_filenames = false, require_serial = true },
]
```

### Step 3: Install Git Hooks
Enable automatic prek execution on commits:

```bash
# From repository root
prek install --install-hooks
```

**Expected output**: `pre-commit installed at .git/hooks/pre-commit`

### Step 4: Verify Prek Setup
Test that prek can run all checks:

```bash
# Run all checks on all files
prek run --all-files
```

**Expected output**: All hooks pass (some may auto-fix files). If files are modified, stage and commit them.

---

## Primary Quality Gates (Run Before Pushing)

These are the core "ready to push" checks. Run them in order before committing your changes.

### Step 1: Prek Invariant Checks
Run prek to enforce code quality invariants:

```bash
# From repository root
prek run --all-files
```

**If prek modifies files**, stage and commit those changes, then re-run prek.

**Expected output**: All hooks pass. Prek enforces:
- No trailing whitespace or mixed line endings
- Valid TOML/YAML/JSON syntax
- No large files or private keys committed
- Consistent Rust code formatting, linting, compilation, and testing

### Step 3: Format Check
Check that all code is properly formatted:

```bash
# From repository root
cargo fmt --check
```

**If formatting fails**, fix it automatically:

```bash
cargo fmt
```

Then run `cargo fmt --check` again to confirm.

**Expected output**: No output means formatting is correct. Errors indicate files that need formatting.

### Step 4: Lint Check
Run Clippy to catch common mistakes and style issues:

```bash
# From repository root
cargo clippy --workspace --all-targets -- -D warnings
```

**If warnings appear**, fix them. The `-D warnings` flag treats warnings as errors.

**Expected output**: Compilation completes with no warnings or errors.

### Step 5: Run All Tests
Execute the full test suite:

```bash
# From repository root
cargo test --workspace
```

**Expected output**: All tests pass. You'll see output like:
```
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored
```

### Step 6: Verify WASM Compilation
Ensure the web frontend can compile for WebAssembly (prevents native-only dependencies from creeping in):

```bash
# From repository root
cargo check -p rust_tinypng_clone_frontend --target wasm32-unknown-unknown
```

**Expected output**: Compilation succeeds without errors.

### Step 5: Build Web Frontend
Build the web frontend with Trunk to verify the TailwindCSS pipeline works:

```bash
# Navigate to web app directory
cd apps/web

# Build for release
trunk build --release

# Return to repository root
cd ../..
```

**Expected output**: 
- TailwindCSS generates `public/app.css`
- Trunk builds the WASM bundle
- Build completes successfully
- Output is in `dist/` directory (relative to repository root)

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
cargo test -p rust_tinypng_clone
```

**Expected output**: Tests for that specific package run and pass.

> **Note**: The web frontend (`rust_tinypng_clone_frontend`) does not have unit tests by design. It's validated through WASM compilation checks and manual smoke tests.

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

## Manual Smoke Tests (Required for WASM Frontend)

The web frontend requires manual testing since it runs in a browser. Follow these steps:

### Step 1: Start the API Server
The API must start without requiring a `.env` file. Set environment variables and run:

```bash
# From repository root
export APP_ENV=development
export PORT=8080
export CORS_ALLOWED_ORIGINS="http://localhost:8080"
export RUST_LOG=info

# Start the API
cargo run -p api
```

**Expected output**:
- API starts successfully
- You see: `Starting API server on 0.0.0.0:8080`
- Config validation happens at startup
- If required env vars are missing, you get a clear error and the process exits

**Keep this terminal open** - the API needs to keep running.

### Step 2: Start the Web Dev Server
In a **new terminal**, start the web frontend:

```bash
# Navigate to web app directory
cd apps/web

# Start Trunk dev server
trunk serve
```

**Expected output**:
- Trunk builds the WASM bundle
- TailwindCSS generates the CSS
- Server starts on `http://localhost:8080` (or another port if 8080 is taken)
- Browser automatically opens (or navigate manually)

### Step 3: Browser Checklist
In the browser, verify:

- [ ] **Web app loads**: The UI appears with no console errors
- [ ] **Tailwind styling works**: Buttons, layout, colors are styled correctly
- [ ] **Upload single image**: Select an image and compress it
- [ ] **Batch compression**: Upload multiple images and compress them
- [ ] **Error handling**: Try uploading an invalid file (e.g., `.txt`) and confirm a clean error message
- [ ] **Network tab check**: Open browser DevTools → Network tab:
  - No secrets exposed in requests
  - API calls go to correct endpoint (`/api/compress`)
  - No unexpected 4xx/5xx errors
  - CORS headers are present (if applicable)

### Step 4: Cleanup
When done testing:

1. Stop the Trunk server: Press `Ctrl+C` in the terminal running `trunk serve`
2. Stop the API server: Press `Ctrl+C` in the terminal running `cargo run -p api`

---

## Desktop Validation (Optional)

The desktop app is local-only and uses Tauri commands (no HTTP calls in MVP).

### Build Desktop App
From the repository root:

```bash
cargo tauri build --bundles app
```

Equivalent from `src-tauri/`:

```bash
cd src-tauri
cargo tauri build --bundles app
```

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
**Symptoms**: `cargo check -p rust_tinypng_clone_frontend --target wasm32-unknown-unknown` fails.

**Checklist**:
1. A native-only crate was accidentally added to `apps/web/Cargo.toml`
2. A dependency enables native default features
3. A dependency doesn't support WASM target

**Fix**:
- Remove the problematic dependency from `apps/web/Cargo.toml`
- Use `default-features = false` where appropriate
- Split native functionality into separate crates (e.g., `crates/compression`)

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

## "Ready to Push" Checklist

Before pushing your changes, verify all of these:

- [ ] **Format**: `cargo fmt --check` passes
- [ ] **Lint**: `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] **Tests**: `cargo test --workspace` passes
- [ ] **WASM**: `cargo check -p rust_tinypng_clone_frontend --target wasm32-unknown-unknown` passes
- [ ] **Web Build**: `cd apps/web && trunk build --release` succeeds
- [ ] **Manual Smoke Test**: `trunk serve` + `cargo run -p api` works in browser

### Test Contribution Notes (Before PR)

- Add new cross-crate or scenario tests under the top-level `tests/` directory.
- Keep unit tests co-located in `#[cfg(test)]` modules for module-private logic.
- For bug fixes, include a regression test that would fail before the fix.
- Keep fixtures small and deterministic under `tests/fixtures/`.
- If a test is expensive, mark it `#[ignore]` with a clear reason and run instructions.

**Quick validation command** (if using `just`):

```bash
just validate
```

This runs format, lint, test, and WASM check in sequence.

---

## Quick Reference

### Package Names
- `domain` - Domain types and utilities
- `compression` - Image compression logic
- `api` - HTTP API server
- `rust_tinypng_clone_frontend` - Web frontend (WASM)
- `rust_tinypng_clone` - Desktop app (Tauri)

### Key Directories
- `apps/web/` - Web frontend source
- `apps/api/` - API server source
- `apps/desktop/src-tauri/` - Desktop app source
- `crates/domain/` - Shared domain types
- `crates/compression/` - Shared compression logic
- `tests/fixtures/` - Test image files

### Common Commands
```bash
# From repository root
cargo test --workspace                    # Run all tests
cargo test -p domain                     # Test domain crate
cargo test -p compression                 # Test compression crate
cargo test -p api                         # Test API
cargo check -p rust_tinypng_clone_frontend --target wasm32-unknown-unknown  # Check WASM
cd apps/web && trunk serve                # Serve web app
cargo run -p api                          # Run API server
```

---

## Getting Help

If you encounter issues not covered here:

1. Check the error message carefully - Rust compiler errors are usually very helpful
2. Verify your setup matches the "One-Time Setup" section
3. Ensure you're running commands from the repository root (unless specified otherwise)
4. Check that all dependencies are installed (`rustup`, `trunk`, `npm` packages)
5. Review the troubleshooting section above

For project-specific questions, consult the main `README.md` or project documentation.
