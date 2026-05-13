# Web-Desktop Parity Matrix

This document tracks feature parity between the desktop (Tauri) and web (WASM + HTTP API) implementations.

## Legend

- ✅ Full parity - feature works identically on both platforms
- ⚠️ Partial parity - feature works but with platform-specific differences
- ❌ No parity - feature only available on one platform
- 🚧 In progress - implementation pending

## Feature Matrix

| Feature | Desktop (Tauri) | Web (HTTP API) | Notes |
|---------|----------------|----------------|-------|
| **File Operations** ||||
| Native file picker | ✅ | ❌ | Web uses HTML file input |
| Drag & drop files | ✅ | ⚠️ | Web requires custom JS bridge |
| Folder picker | ✅ | ❌ | Web downloads to default location |
| Open file manager | ✅ | ❌ | Not available in browsers |
| **Compression** ||||
| PNG compression | ✅ | ✅ | Shared `crates/compression` |
| JPEG compression | ✅ | ✅ | Shared `crates/compression` |
| WebP conversion | ✅ | ✅ | Shared `crates/compression` |
| AVIF conversion | ✅ | ✅ | Shared `crates/compression` |
| TIFF conversion | ✅ | ✅ | Shared `crates/compression` |
| BMP conversion | ✅ | ✅ | Shared `crates/compression` |
| ICO conversion | ✅ | ✅ | Shared `crates/compression` |
| HEIC/HEIF input | ✅ | ✅ | Shared `crates/compression` |
| oxipng optimization | ✅ | ✅ | Shared `crates/compression` |
| PNG lossy compression | ✅ | ✅ | Shared `crates/compression` |
| **Save Operations** ||||
| Multi-file save (individual) | ✅ | ❌ | Browsers block multiple simultaneous downloads; use ZIP instead |
| ZIP export | ✅ | ⚠️ | Web downloads individual files sequentially (client-side ZIP creation future work) |
| Save button visibility | ✅ | ⚠️ | Web shows only "Save as ZIP"; Desktop shows both "Save File(s)" and "Save as ZIP" |
| Collision detection | ✅ | ❌ | Web relies on browser behavior |
| Auto-rename on collision | ✅ | ⚠️ | Web adds suffix automatically |
| Overwrite confirmation | ✅ | ❌ | Browser handles download conflicts |
| **UI Features** ||||
| Progress indication | ✅ | ✅ | Both show compression progress |
| Results list | ✅ | ✅ | Both display compression results |
| File size display | ✅ | ✅ | Both show original/compressed sizes |
| Savings percentage | ✅ | ✅ | Both calculate savings |

## Backend Capabilities

### Desktop (TauriBackend)

```rust
BackendCapabilities {
    supports_native_dialogs: true,
    supports_folder_picker: true,
    supports_collision_check: true,
    supports_file_manager: true,
    supports_drag_drop: true,
}
```

### Web (HttpBackend)

```rust
BackendCapabilities {
    supports_native_dialogs: false,
    supports_folder_picker: false,
    supports_collision_check: false,
    supports_file_manager: false,
    supports_drag_drop: false,
}
```

## Architecture

### Shared Compression Core

All compression algorithms are implemented in `crates/compression` and shared between:
- `src-tauri/src/commands.rs` - Desktop Tauri commands
- `apps/api/src/routes.rs` - HTTP API endpoints

### Backend Abstraction

The UI uses the `AppBackend` trait defined in `src/backend/mod.rs`:

```rust
#[async_trait(?Send)]
pub trait AppBackend: fmt::Debug {
    fn capabilities(&self) -> BackendCapabilities;
    fn is_available(&self) -> bool;
    async fn select_files(&self) -> Result<Vec<FileInfo>, BackendError>;
    async fn compress_batch(&self, request: CompressionRequest) -> Result<Vec<CompressionResult>, BackendError>;
    // ... other methods
}
```

Implementations:
- `TauriBackend` - Desktop native operations
- `HttpBackend` - Web HTTP API client
- `BackendProvider` - Runtime selection

## Single Source of Truth

The `crates/compression` crate is the single source of truth for:

1. **Image decoding** - All formats (PNG, JPEG, WebP, AVIF, etc.)
2. **Compression algorithms** - PNG quantization, JPEG encoding, WebP encoding
3. **Format conversion** - All supported input/output format combinations
4. **Quality settings** - Compression level mapping

Both the Tauri desktop app and the HTTP API use the same `compress_image_inproc()` function.

## Testing Parity

Run parity tests:

```bash
# Test compression crate
cargo test -p compression

# Test domain types
cargo test -p domain

# Test API
cargo test -p api

# All tests
cargo test
```

## Known Divergences

### File System Operations

Web browsers cannot:
- Access native file system paths
- Show native folder pickers
- Write to arbitrary folders
- Open native file managers

Mitigation: Web uses HTML file inputs and download triggers.

### Collision Handling

Desktop: Full collision detection with user modal
Web: Browser handles download conflicts automatically

### Save Behavior

Desktop: Users can save files individually or as ZIP. Full folder picker and collision handling.
Web: Browsers block multiple simultaneous downloads. UI shows only "Save as ZIP" button. Files download individually to browser's default download folder (client-side ZIP creation requires JSZip library - future work).

### Drag & Drop

Desktop: Native Tauri drag & drop with file paths
Web: Requires JavaScript bridge for file data transfer

## Development Workflow

### Adding New Features

1. Implement in `crates/compression` (if compression-related)
2. Update Tauri commands in `src-tauri/src/commands.rs`
3. Update API routes in `apps/api/src/routes.rs`
4. Update backend trait if new capability added
5. Update this parity matrix
6. Add tests to both paths

### Testing Changes

1. Run `cargo test` to verify shared core
2. Test desktop from repo root: `make dev-desktop` or `cargo tauri dev` (Trunk for `src/`; dev server on http://localhost:8080)
3. Test web (API=3000, Trunk in `apps/web`):
   - Terminal 1: `cargo run -p api`
   - Terminal 2: `make dev-web`
4. Verify feature works on both platforms
5. Update parity matrix if needed
