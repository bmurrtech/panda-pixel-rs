# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

(nothing yet)

## [0.1.2-alpha] - 2026-04-16

### Added

- File collision detection before save (`check_file_collisions`); modal with Cancel, Rename, Overwrite.
- ZIP export for multiple compressed results; timestamp-based ZIP name (`hh-mm-ss-yyyy-mm-dd-panda-pixel-app.zip`) to avoid generic name collisions.
- Selected files list with names and sizes; sticky footer so save actions stay visible.

### Changed

- Save actions: plain text labels, dynamic singular/plural (“Save File” vs “Save Files”), ZIP button only when multiple results; primary and ZIP buttons share bordered grayscale style with green hover.
- Collision modal primary action label: “Rename” (was longer wording).
- Single collision modal path: modal lives with save flow in `CompressButton` (removed duplicate modal from app shell).

### Fixed

- Leptos `Fn` vs `FnOnce` compile errors on nested save/collision handlers (`AppState` made `Copy` where appropriate).
- Rename-after-collision not applying when a second, non-functional modal was mounted; fixed by using one modal implementation wired to pending folder and save options.
- ZIP collision checks targeting a stale fixed filename; collision probe now uses the same timestamp base name as the save.
- **Tauri release build:** Stale permission index files under `target/release/build/tauri-*/` could still point at an old repo path after the project moved; `cargo tauri build` failed reading plugin permissions. Fix: `cargo clean` or `rm -rf target/release/build/tauri-*` then rebuild (see plan doc for full notes).
