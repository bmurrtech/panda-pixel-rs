.PHONY: dev-web dev-desktop build-desktop

# Strip NO_COLOR for Trunk 0.21 (clap quirk); harmless if unset.
TRUNK_OK := env -u NO_COLOR -u FORCE_COLOR

# Workspace web UI (apps/web). With API: run `cargo run -p api` in another terminal.
dev-web:
	cd apps/web && $(TRUNK_OK) trunk serve

# Desktop shell; Tauri runs Trunk for src/.
dev-desktop:
	$(TRUNK_OK) cargo tauri dev

# Release desktop bundle (runs Trunk in src/ via Tauri).
build-desktop:
	$(TRUNK_OK) cargo tauri build
