# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4-alpha] - 2026-05-12

### PRD 003 — Web / desktop parity

Shipped scope: one compression core, one UI backend boundary, documented matrix and test entrypoints, and desktop save flows that respect the parity contract. Outstanding items stay explicitly **out of MVP** or tracked in **`docs/parity-matrix.md`**.

| ID | Summary | Changelog outcome |
|----|---------|-------------------|
| **FR-1** | Single compression source of truth (`crates/compression` for API + desktop) | **Done** — shared crate is the encoder path for Tauri commands and API. |
| **FR-2** | UI host abstraction (`AppBackend`, runtime `BackendProvider`) | **Done** — `src/backend/mod.rs`; UI uses provider instead of scattered invoke/fetch. |
| **FR-3** | Parity verification (`cargo test` + documented smoke) | **Done** for automated gate — see **`docs/testing.md`**; full release smoke still human-run per contributing. |
| **FR-4** | Contributor docs: parity + link to testing commands | **Done** — **`docs/contributing.md`** and **`docs/parity-matrix.md`**. |
| **FR-5** | Client-side ZIP in browser (JSZip-class bundling) | **Not in MVP** — web “Save All (.zip)” remains per matrix (download path / helper); desktop ZIP is native. |
| **FR-6** | Per-result download affordance | **Done** — `results_list.rs`: row click + download SVG affordance and MIME-aware blob download. |

**PRD 003 Bugs:**

- **BUG-001** — API vs Trunk port clash — **Fixed** (API default **3000**, Trunk **8080**).
- **BUG-002** — Browser picker not feeding compression — **Fixed** (`__BROWSER_DROPPED_FILES` wiring in file selector).
- **BUG-003** — Browser save “backend not available” — **Fixed** (Blob URL downloads, placeholder folder, no-op collision/manager where required).
- **BUG-004** — Multi-download blocked in browser — **Fixed** (UI: web shows **Save All (.zip)** only; desktop keeps folder save + ZIP).
- **BUG-005** — ZIP label vs individual downloads / status copy — **Partially addressed** (clearer labels and status by host; true single-file browser ZIP remains **FR-5**).
- **BUG-006** — Download affordance visibility — **Addressed** in current UI (inline SVG + clickable row); reopen in PRD if a host still hides the control.

### Added

- Tauri command **`resolve_unique_filenames`** and module **`src-tauri/src/filename_unique.rs`**: batch display-name disambiguation using real filesystem checks; same ` (1)` / `(2)` … rules reused for **ZIP member paths** when two entries would share the same name inside one archive.
- **`ResolveUniqueFilenamesRequest`** / **`resolve_unique_filenames`** on **`AppBackend`** (Tauri invoke; HTTP identity pass-through).
- Web-desktop parity backend abstraction in `src/backend/mod.rs` with `AppBackend` trait.
- `TauriBackend` implementation for desktop native operations via Tauri invoke.
- `HttpBackend` implementation for web HTTP API client with configurable base URL.
- `BackendProvider` for automatic runtime selection (Tauri vs HTTP) based on availability.
- `BackendCapabilities` struct for feature detection (native dialogs, folder picker, etc.).
- Parity matrix documentation in `docs/parity-matrix.md` tracking feature parity between platforms.
- Parity test harness and pre-release verification documented in `docs/testing.md` (contributing links there).
- Shared compression core validation: `crates/compression` is single source of truth for both Tauri and API.

### Fixed

- **Desktop collision modal and saves:** duplicate / auto-rename no longer produced `name (1) (1).ext` when `name (1).ext` already existed; numeric suffix now strips the trailing copy index and picks the next free **` (n)`** on disk and within the save batch (**Tauri-authoritative**, not WASM `Path::exists`).
- **Partial rename in collision modal (desktop):** mix of edited and untouched rows resolves untouched names against disk and in-batch conflicts in one **`resolve_unique_filenames`** pass before save.
- **Collision modal UX:** tip text singular vs plural (“Edit file name” / “Edit file names”); **Duplicate** hidden while any name is edited (align with Overwrite); **Rename** uses green safe-action styling; post-rename collision errors no longer tell users to use Duplicate when that action is hidden.
- Port conflict between API (now 3000) and Trunk (8080): changed API default from 8080 to 3000 to avoid collision during local web dev.
- Backend not available error in browser mode (BUG-002): 
  - Implemented `compress_batch` for `HttpBackend` to properly send files to API for compression
  - Fixed file selection to store File objects in `window.__BROWSER_DROPPED_FILES` for later access during compression
  - Added debug logging to trace compression flow
- Backend not available error on save in browser mode (BUG-003):
  - Implemented browser-compatible save using download triggers via Blob URLs
  - `save_files_to_folder()`: Creates Blob URLs and triggers download via hidden anchor elements
  - `save_files_as_zip()`: Downloads individual files (ZIP creation requires JSZip library)
  - `select_output_folder()`: Returns placeholder path
  - `check_file_collisions()`: Returns empty (browser handles conflicts)
  - `open_in_file_manager()`: No-op

### Changed

- Removed browser mode banner from file selector UI (now fully functional in browser mode).
- Added browser drag & drop support with JavaScript glue code in `index.html`.
- **UI adapts to backend type (BUG-004):**
  - Web mode: Shows only "Save All (.zip)" button (browsers block multiple simultaneous downloads)
  - Desktop mode: Shows both "Save File(s)" and "Save All (.zip)" buttons
  - See `docs/parity-matrix.md` for complete save operation parity details
- **Improved save experience (BUG-005):**
  - Added download icon to each result item (click to download individual file)
  - Changed "Save as ZIP" button to "Save All (.zip)" for clarity
  - Browser mode now shows "Downloaded N files" instead of misleading "Saved ZIP" message
  - Browser auto-handles filename collisions (adds "(1)", "(2)", etc.)
  - Skip collision modal in browser mode (browser manages conflicts)

### Technical

- Desktop collision and auto-rename: WASM UI calls **`resolve_unique_filenames`** over Tauri instead of relying on `Path::exists` in the Trunk bundle; `panda_pixel_rs_desktop` includes **`filename_unique`** unit tests ( **`tempfile`** dev-dependency).
- Added `async-trait` dependency for backend trait implementation.
- Extended `web-sys` features for HTTP backend: `Request`, `RequestInit`, `RequestMode`, `Response`, `FormData`.
- Updated `compress_button.rs` to use `BackendProvider` for all save/compression operations.
- Updated `file_selector.rs` to use `BackendProvider` for file selection with capability detection.

## [0.1.3-alpha] - 2026-05-12

### Added

- Collision modal: editable filename inputs with per-file rename capability.
- Duplicate button: auto-rename collision behavior (previously labeled "Rename").
- Dynamic button visibility in collision modal: Cancel | Duplicate | (Overwrite|Rename) based on edit state.
- Manual rename validation: full collision check on edited names before save.
- Filename sanitization: strips path separators and dangerous characters from edited names.
- Browser Tier 0 support: FileSelector branches on Tauri availability; hidden file input for browser mode.
- Browser mode info banner: non-blocking notification when running without Tauri.
- `build_output_filenames` helper for mapping collision edits to result outputs.
- `prepare_files_payload_with_overrides` for injecting custom filenames into save payload.

### Changed

- Moved `pending_save_options` from local signal to `AppState` for cross-component access.
- Added `collision_name_edits` and `collision_initial_snapshot` signals to `AppState` for dirty detection.
- Collision modal copy: "Tip: Edit and rename file name above." (was "Would you like to overwrite...").
- Error copy for rename collisions: "File already exists. Try a different name or use Duplicate." (no overwrite mention).
- Overwrite button hidden when any filename is edited; Rename button shown instead.

### Removed

- Stub `collision_modal.rs` component (functionality merged into `CompressButton`).

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
