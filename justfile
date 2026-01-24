# Justfile for Rust TinyPNG Clone Monorepo

# Format code
fmt:
    cargo fmt --check

# Lint code
lint:
    cargo clippy --workspace --all-targets

# Run all tests
test:
    cargo test --workspace

# Check web compiles for WASM
check-web:
    cargo check -p rust_tinypng_clone_frontend --target wasm32-unknown-unknown

# Build web frontend
build-web:
    cd apps/web && trunk build --release

# Serve web frontend (dev)
serve-web:
    cd apps/web && trunk serve

# Build API
build-api:
    cargo build -p api --release

# Run API
run-api:
    cargo run -p api

# Build desktop app
build-desktop:
    mkdir -p dist
    cargo tauri build -p rust_tinypng_clone

# Run desktop app (dev)
run-desktop:
    mkdir -p dist
    cargo tauri dev -p rust_tinypng_clone

# Full validation (format, lint, test, check web)
validate:
    just fmt
    just lint
    just test
    just check-web
