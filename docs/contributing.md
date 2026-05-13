# Contributing to panda-pixel-rs

Guidelines for developers: **architecture**, **layout**, **web vs desktop**, and **day-to-day dev**.  
**Testing, parity checks, and pre-push validation** live in [docs/testing.md](testing.md).

## Table of contents

- [Development setup](#development-setup)
- [Repository layout](#repository-layout)
- [Web vs desktop](#web-vs-desktop)
- [Building](#building)
- [Parity (high level)](#parity-high-level)
- [Development workflow](#development-workflow)
- [Code quality](#code-quality)
- [Submitting changes](#submitting-changes)
- [Release process](#release-process)
- [Getting help](#getting-help)
- [License](#license)

---

## Development setup

### Prerequisites

1. **Rust** — [rustup](https://rustup.rs/). Install the toolchain; the repo may pin via `rust-toolchain.toml`.
2. **WASM target** (for Leptos frontends):

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Tauri CLI** (desktop):

   ```bash
   cargo install tauri-cli@2.9.1 --locked
   ```

4. **Trunk** (WASM bundler):

   ```bash
   cargo install trunk
   ```

5. **Node / npm** — only needed when working on Tailwind in `apps/web` (see [testing.md](testing.md) one-time setup).

### Clone and verify

```bash
git clone https://github.com/bmurrtech/panda-pixel-rs.git
cd panda-pixel-rs
cargo check --workspace
```

---

## Repository layout

Cargo workspace members are defined in root [Cargo.toml](../Cargo.toml). Excluded paths (e.g. `src/`) are built via `--manifest-path` or Trunk/Tauri, not `cargo check -p` from the workspace alone.

```
panda-pixel-rs/
├── Makefile         # `make dev-web`, `make dev-desktop`, `make build-desktop`
├── apps/
│   ├── api/           # Axum HTTP API (compression via shared crate)
│   └── web/           # Leptos CSR WASM (workspace member: panda_pixel_rs_web)
├── src-tauri/         # Tauri desktop shell; embeds UI from ../src
├── src/               # Leptos UI for desktop (Trunk); not a workspace member
├── crates/
│   ├── domain/        # Shared types / options
│   └── compression/   # Shared image pipeline (API + desktop should use this)
├── tests/fixtures/    # Shared binary fixtures for tests
└── docs/              # contributing, testing, parity-matrix, …
```

---

## Web vs desktop

| Surface | UI source | How it runs | Backend |
|--------|-----------|-------------|---------|
| **Desktop** | `src/` + Trunk | `make dev-desktop` or `cargo tauri dev` from repo root (hooks in [src-tauri/tauri.conf.json](../src-tauri/tauri.conf.json): Trunk in `src/`) | Tauri commands, native FS |
| **Web (workspace)** | `apps/web/` | `make dev-web` or `cd apps/web && trunk serve` (and usually `cargo run -p api` for full flow) | HTTP API when wired; see [parity-matrix.md](parity-matrix.md) |

See [troubleshooting.md](troubleshooting.md) if `CI=true` in your shell changes Tauri behavior.

### Critical Save Distinctions

**Web browsers cannot save multiple files individually** (blocked by browser security policy). The UI adapts:
- **Desktop**: Shows both "Save File(s)" and "Save as ZIP" buttons
- **Web**: Shows only "Save as ZIP" button; files download individually to browser's default folder

See [parity-matrix.md](parity-matrix.md) "Save Operations" and "Known Divergences → Save Behavior" for complete details.

---

## Building

### Desktop (dev)

```bash
make dev-desktop
```

Same as `cargo tauri dev` from the **repository root** (workspace `Cargo.toml`). Tauri runs Trunk for the Leptos UI in `src/` per [src-tauri/tauri.conf.json](../src-tauri/tauri.conf.json). DevTools: F12 / Ctrl+Shift+I in the app window.

### Desktop (release bundle)

```bash
cargo tauri build
```

Same with `NO_COLOR` cleared for nested Trunk: `make build-desktop` from repo root.

Outputs under `target/release/` and `target/release/bundle/` (platform-specific).

### Web (`apps/web`)

```bash
cd apps/web && npm install && npm run build:css
make dev-web
# or: cd apps/web && trunk serve
cd apps/web && trunk build --release
```

Tailwind / Trunk details: [testing.md](testing.md).

---

## Parity (high level)

The project aims at **behavioral parity** between desktop and web: one shared **`crates/compression`** story, one UI direction behind an **`AppBackend`**-style boundary (see [parity-matrix.md](parity-matrix.md)).

When you change compression or API contracts:

- Touch **`crates/compression`** (and **`crates/domain`** as needed), not duplicate encoder logic.
- Update **Tauri** (`src-tauri/`) and **API** (`apps/api/`) call sites together when behavior changes.
- Run tests and any manual smoke steps in [testing.md](testing.md).

---

## Development workflow

1. Create a branch; implement changes.
2. **Validate** with [testing.md](testing.md): primarily `cargo test --workspace`, plus fmt/clippy and WASM checks before push.
3. Open a PR with a clear description; link issues; add screenshots for UI changes.

---

## Code quality

Primary automation is **Rust’s own toolchain**, not optional hook frameworks:

| Check | Command |
|-------|---------|
| Tests | `cargo test --workspace` |
| Format | `cargo fmt --check` (fix: `cargo fmt`) |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Compile | `cargo check --workspace` |
| WASM | See [testing.md](testing.md) for `panda_pixel_rs_web` and `src/` (`padna_pixel_rs_frontend`) |

### Commits

Use [Conventional Commits](https://www.conventionalcommits.org/) when possible, e.g. `feat:`, `fix:`, `docs:`.

### Review (human)

- No silent `unwrap`/`expect` in production paths without justification.
- Public API / behavior changes reflected in docs or CHANGELOG as appropriate.
- Compression or parity-sensitive changes: see [testing.md](testing.md) and [parity-matrix.md](parity-matrix.md).

---

## Submitting changes

1. Branch from `main`: `git checkout -b feature/short-description`
2. Follow [testing.md](testing.md) before push.
3. PR should include: summary, test notes, screenshots if UI, breaking-change callouts if any.

---

## Release process

### SemVer

Tags use `vMAJOR.MINOR.PATCH[-PRERELEASE]` (e.g. `v1.2.3`, `v0.1.2-alpha`). Prereleases sort before the matching stable version.

### Version sync

Before tagging, align:

| Location | Field |
|----------|--------|
| [Cargo.toml](../Cargo.toml) | `[workspace.package] version` |
| [src-tauri/tauri.conf.json](../src-tauri/tauri.conf.json) | top-level `"version"` |

CI should fail if tag and manifests disagree.

### Steps

1. Bump version in both places above; update [CHANGELOG.md](../CHANGELOG.md) if needed.
2. Run validation from [testing.md](testing.md) (tests + recommended gates).
3. Optional: `cargo tauri build` locally.
4. Commit on `main`, then tag and push:

   ```bash
   git tag v0.1.3-alpha
   git push origin v0.1.3-alpha
   ```

Release workflow builds matrix artifacts and publishes a GitHub Release (see `.github/workflows/release.yml`).

---

## Getting help

- **Issues** — bugs and features  
- **Discussions** — questions  
- **Testing / CI failures** — [testing.md](testing.md)

---

## License

Contributions are licensed under the same terms as the project (GNU AGPLv3 — see repository [LICENSE](../LICENSE)).
