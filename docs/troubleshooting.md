# Troubleshooting

## Wrong directory for Tauri

Run `cargo tauri dev` / `cargo tauri build` from the **repository root** (workspace `Cargo.toml` + `src-tauri/`). There is no `tauri.conf.json` inside `src/`.

## `CI=true` in your shell

Tauri treats a CI environment differently (non-interactive, stricter). If a terminal or tool exports `CI=true` and behavior looks off, run `unset CI` (Unix) or clear `CI`, then retry.

## Trunk: `invalid value '1' for '--no-color'`

Some environments set `NO_COLOR=1`; Trunk 0.21 can mis-parse it. From the Trunk crate directory (`src/` or `apps/web`), on Unix: `env -u NO_COLOR -u FORCE_COLOR trunk serve` (or `trunk build --release`). On Windows, clear `NO_COLOR` and `FORCE_COLOR` for that session before `trunk`.

The repo **Makefile** runs `make dev-web`, `make dev-desktop`, and `make build-desktop` with those variables unset so Trunk is invoked safely.

## `apps/web` Trunk.toml: hook parse errors

Trunk **0.21** expects `[[hooks]]` with `stage` and `command` (and optional `command_arguments`). The old `[hooks.build]` table is invalid; see `apps/web/Trunk.toml`.
