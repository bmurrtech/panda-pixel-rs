# Contributing to panda-pixel-rs

Thank you for your interest in contributing to panda-pixel-rs! This document provides detailed guidelines for developers.

## Table of Contents

- [Development Setup](#development-setup)
- [Building from Source](#building-from-source)
- [Development Workflow](#development-workflow)
- [Code Quality](#code-quality)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Development Setup

### Prerequisites

1. **Rust Toolchain**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install WASM target for web frontend
   rustup target add wasm32-unknown-unknown
   ```

2. **Tauri CLI**
   ```bash
   cargo install tauri-cli@2.5.0 --locked
   ```

3. **Trunk (WASM bundler)**
   ```bash
   cargo install trunk
   ```

4. **Pre-commit Hooks (Prek)**
```bash
# Install Prek
cargo install pre-commit

# Install hooks (sets up automatic quality checks on commit)
prek install --install-hooks
```

### Repository Setup

```bash
# Clone the repository
git clone https://github.com/bmurrtech/panda-pixel-rs.git
cd panda-pixel-rs

# Verify setup
cargo check --workspace
```

## Building from Source

### Monorepo Structure

This project uses a monorepo structure with strict target isolation:

```
panda-pixel-rs/
├── apps/
│   ├── web/          # Web deployment (WASM-only)
│   └── desktop/      # Desktop deployment (Tauri + native)
├── crates/           # Shared code
│   ├── domain/       # Platform-agnostic types
│   └── compression/  # Image processing algorithms
└── tests/            # Shared test fixtures
```

### Development Builds

#### Desktop Development
```bash
# Hot reload development with debugging console
unset CI && cargo tauri dev
```

**Key Features:**
- **Hot Reload**: Automatically rebuilds and restarts when Rust code changes
- **CSS Auto-build**: TailwindCSS compiles automatically via `beforeDevCommand`
- **Debug Console**: Built-in developer tools accessible via F12 or Ctrl+Shift+I
- **Window Management**: Auto-resizes based on content, supports drag & drop
- **File Watching**: Monitors changes across the entire monorepo

**Debugging with Console:**
1. **Open DevTools**: Press `F12` or `Ctrl+Shift+I` in the app window
2. **Console Logging**: View startup messages and debug output
3. **Network Tab**: Monitor API calls (when implemented)
4. **Application Tab**: Inspect local storage and app data
5. **Sources Tab**: Debug Rust-generated WASM code

**Console Messages You'll See:**
```
🚀 Panda Pixel starting (Tauri mode)
📦 drag-drop: 3 files (sample: image1.png, image2.jpg, image3.webp)
✅ Processed 3 dropped files
```

**Troubleshooting:**
- If CSS doesn't update: Run `cd apps/web && npm run build:css` manually
- If app doesn't start: Check for `CI=1` environment variable and unset it
- If dev tools don't open: Look for `Failed to open devtools` in console

#### Web Development (Optional)
```bash
# Install Node.js dependencies for TailwindCSS
cd apps/web && npm install

# Build CSS (one-time or when styles change)
cd apps/web && npm run build:css

# Serve with hot reload
cd apps/web && trunk serve
```

### Production Builds

#### Desktop Application
```bash
# Build web frontend CSS first (required for desktop bundle)
cd apps/web && npm run build:css

# Build desktop application with installer
unset CI && cargo tauri build
```

**Output locations:**
- **macOS**: `target/release/bundle/macos/Panda Pixel.app`
- **Binary**: `target/release/panda_pixel_rs_desktop`

#### Web Application (Optional)
```bash
# Build CSS
cd apps/web && npm run build:css

# Build WASM bundle
cd apps/web && trunk build --release

# Output: apps/web/dist/
```

## Development Workflow

### Daily Development Cycle

1. **Start development server**
   ```bash
   cargo tauri dev
   ```

2. **Make changes** to code

3. **Prek runs automatically** on commit (all quality checks enforced)
   ```bash
   git add .
   git commit -m "feat: add new compression option"
   # Prek automatically runs: format, lint, compile, test
   ```

4. **If pre-commit fails**, fix issues and try again
   ```bash
   # Fix any reported issues, then commit again
   git add .
   git commit -m "feat: add new compression option"
   ```

### Manual Quality Checks (if needed)

While prek handles automatic checks, you can run them manually:

```bash
# Run all checks
prek run --all-files

# Run specific checks
prek run rust-fmt
prek run rust-clippy
prek run rust-check
prek run rust-test
```

### Pre-commit Quality Gates

The project uses Prek for invariant enforcement. All checks must pass before committing:

```bash
# Run all pre-commit checks
prek run --all-files
```

Prek enforces:
- No trailing whitespace or mixed line endings
- Valid file formats (TOML, YAML, JSON)
- No large files or private keys
- Rust code formatting and linting

## Code Quality

### Rust Standards

Prek automatically enforces:

- **Formatting**: `cargo fmt --check --all` (auto-fix available)
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
- **Compilation**: `cargo check --workspace` (all targets)
- **WASM Compatibility**: `cargo check -p panda_pixel_rs_web --target wasm32-unknown-unknown`
- **Unit Tests**: `cargo test --workspace --lib` (fast tests only)

### Commit Messages

Follow conventional commit format:

```bash
# Good examples
git commit -m "feat: add AVIF output format"
git commit -m "fix: resolve memory leak in PNG compression"
git commit -m "docs: update build instructions"

# Bad examples
git commit -m "update code"
git commit -m "fix bug"
```

### Code Review Checklist

Prek automatically verifies:
- ✅ Code formatting (`cargo fmt`)
- ✅ Clippy linting (warnings as errors)
- ✅ Compilation on all targets
- ✅ WASM compatibility
- ✅ Unit tests pass

Manual verification needed:
- [ ] No `unwrap()` or `expect()` in production code
- [ ] Documentation updated for public APIs
- [ ] Breaking changes are clearly documented
- [ ] UI/UX changes tested manually
- [ ] Performance impact assessed for compression algorithms

## Testing

### Test Categories

- **Unit Tests**: Co-located with code in `#[cfg(test)]` modules (runs in prek)
- **Integration Tests**: HTTP-level tests in `apps/api/tests/` (manual, when needed)
- **WASM Tests**: Manual validation via `trunk serve`
- **Fixtures**: Centralized under `tests/fixtures/`

### Running Tests Manually

Unit tests run automatically in prek, but for manual testing:

```bash
# Run all tests (including integration)
cargo test --workspace

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Test specific package
cargo test -p compression
```

### Advanced Testing

For comprehensive testing beyond prek:

```bash
# Test coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --html --open

# E2E tests (if configured)
cargo nextest run --profile e2e

# Fuzz tests (if configured)
cargo test --release --package api --test fuzz_compression

# Benchmarks
cargo bench
```

See [docs/testing.md](testing.md) for comprehensive testing guidelines.

## Submitting Changes

### Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/add-webp-support
   ```

2. **Make changes** and ensure quality gates pass

3. **Update documentation** if needed

4. **Submit PR** with:
   - Clear title and description
   - Reference to related issues
   - Screenshots for UI changes
   - Test coverage for new features

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Screenshots (if applicable)
<!-- Add screenshots for UI changes -->

## Checklist
- [ ] Prek checks pass
- [ ] Code formatted
- [ ] Clippy warnings resolved
- [ ] Tests pass
- [ ] Documentation updated
```

## Release Process

### Version Bumping

Follow semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Steps

1. **Update version** in `Cargo.toml` files
2. **Update changelog** (if applicable)
3. **Run full test suite**
4. **Create release build**
   ```bash
   cargo tauri build
   ```
5. **Test the release build** on target platforms
6. **Create GitHub release** with built binaries
7. **Tag the release**
   ```bash
   git tag v1.2.3
   git push origin v1.2.3
   ```

### Platform Testing

Before releasing, test the built application on:
- macOS (Intel + Apple Silicon)
- Windows 10/11
- Linux (Ubuntu/Debian)

## Getting Help

- **Issues**: Use GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions and general discussion
- **Code of Conduct**: Be respectful and constructive in all interactions

## License

By contributing to this project, you agree that your contributions will be licensed under the GNU AGPLv3 license.