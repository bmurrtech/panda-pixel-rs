# Build from source

Use this guide when your OS or CPU architecture is not covered by [GitHub Releases](https://github.com/bmurrtech/panda-pixel-rs/releases) assets.

Repository layout, web vs desktop, and contributor workflow: [contributing.md](contributing.md).

---

## 1. Clone

```bash
git clone https://github.com/bmurrtech/panda-pixel-rs.git
cd panda-pixel-rs
```

---

## 2. Rust toolchain

If Rust is not installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The repo pins a toolchain in `rust-toolchain.toml`; `cargo` will prompt to install it when needed.

---

## 3. One-time tooling (desktop + Leptos `src/` frontend)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli@2.9.1 --locked
```

For the workspace web app (`apps/web`), also install Node dependencies for Tailwind (see [testing.md](testing.md#step-4-install-nodejs-dependencies-for-tailwindcss)).

---

## 4. Desktop release bundle

From the repository root:

```bash
cargo tauri build
```

Equivalent to `make build-desktop` (runs Trunk for `src/` via Tauri’s `beforeBuildCommand`).

---

## 5. Local development

| Goal | Command |
|------|---------|
| Desktop (dev) | `make dev-desktop` or `cargo tauri dev` |
| Web UI (`apps/web`, dev) | `make dev-web` (usually run `cargo run -p api` in another terminal; see [testing.md](testing.md#manual-smoke-browser--api) for port/CORS) |
| Desktop release | `make build-desktop` or `cargo tauri build` |

**Further reading**

- CI/path quirks: [troubleshooting.md](troubleshooting.md)
- Layout, releases, PRs: [contributing.md](contributing.md)
- Feature parity (desktop vs web): [parity-matrix.md](parity-matrix.md)
- Tests and quality gates: [testing.md](testing.md)
