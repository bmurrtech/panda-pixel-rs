#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ARM_TARGET="aarch64-apple-darwin"
X86_TARGET="x86_64-apple-darwin"
OUTPUT_DIR="$ROOT_DIR/target/macos-dmg"
PRODUCT_SLUG="panda-pixel-rs"
APP_NAME="Panda Pixel.app"

detect_version() {
  python3 - <<'PY'
import json
from pathlib import Path

for p in [
    Path("src-tauri/tauri.conf.json"),
    Path("apps/desktop/src-tauri/tauri.conf.json"),
]:
    if p.exists():
        data = json.loads(p.read_text())
        version = data.get("version")
        if version:
            print(version)
            raise SystemExit(0)
raise SystemExit("Unable to detect version from tauri.conf.json")
PY
}

find_dmg_path() {
  local target="$1"
  local base
  for base in "$ROOT_DIR/target" "$ROOT_DIR/src-tauri/target"; do
    if [[ -d "$base/$target/release/bundle/dmg" ]]; then
      local dmg
      dmg="$(ls -1t "$base/$target/release/bundle/dmg/"*.dmg 2>/dev/null | head -n 1 || true)"
      if [[ -n "${dmg:-}" ]]; then
        echo "$dmg"
        return 0
      fi
    fi
  done
  return 1
}

find_app_path() {
  local target="$1"
  local base
  for base in "$ROOT_DIR/target" "$ROOT_DIR/src-tauri/target"; do
    if [[ -d "$base/$target/release/bundle/macos/$APP_NAME" ]]; then
      echo "$base/$target/release/bundle/macos/$APP_NAME"
      return 0
    fi
  done
  return 1
}

build_arch_dmg() {
  local target="$1"
  local arch_slug="$2"
  local version="$3"
  local out_dmg="$OUTPUT_DIR/${PRODUCT_SLUG}-${version}-${arch_slug}.dmg"

  echo "==> Building DMG for $target"
  cargo tauri build --target "$target" --bundles dmg

  local source_dmg
  if ! source_dmg="$(find_dmg_path "$target")"; then
    echo "Missing DMG bundle for target: $target" >&2
    echo "Checked: target/$target/release/bundle/dmg and src-tauri/target/$target/release/bundle/dmg" >&2
    exit 1
  fi

  cp -f "$source_dmg" "$out_dmg"
  echo "Created: $out_dmg"
}

build_universal_dmg() {
  local version="$1"
  local universal_root="$ROOT_DIR/target/universal/release/bundle/macos"
  local universal_app="$universal_root/$APP_NAME"
  local universal_dmg="$OUTPUT_DIR/${PRODUCT_SLUG}-${version}-universal.dmg"

  echo "==> Building .app bundles for universal merge"
  cargo tauri build --target "$ARM_TARGET" --bundles app
  cargo tauri build --target "$X86_TARGET" --bundles app

  local arm_app
  local x86_app
  if ! arm_app="$(find_app_path "$ARM_TARGET")"; then
    echo "Missing ARM app bundle for universal build" >&2
    exit 1
  fi
  if ! x86_app="$(find_app_path "$X86_TARGET")"; then
    echo "Missing Intel app bundle for universal build" >&2
    exit 1
  fi

  mkdir -p "$universal_root"
  rm -rf "$universal_app"
  cp -R "$arm_app" "$universal_app"

  local arm_bin
  local x86_bin
  arm_bin="$(find "$arm_app/Contents/MacOS" -maxdepth 1 -type f | head -n 1)"
  x86_bin="$(find "$x86_app/Contents/MacOS" -maxdepth 1 -type f | head -n 1)"

  if [[ -z "${arm_bin:-}" || -z "${x86_bin:-}" ]]; then
    echo "Could not locate architecture binaries inside .app bundles" >&2
    exit 1
  fi

  local universal_bin="$universal_app/Contents/MacOS/$(basename "$arm_bin")"
  echo "==> Creating universal app binary with lipo"
  lipo -create "$arm_bin" "$x86_bin" -output "$universal_bin"
  lipo -info "$universal_bin"

  echo "==> Packaging universal DMG"
  rm -f "$universal_dmg"
  hdiutil create -volname "Panda Pixel" -srcfolder "$universal_app" -ov -format UDZO "$universal_dmg"
  echo "Created: $universal_dmg"
}

echo "==> Ensuring required Rust targets are installed"
rustup target add "$ARM_TARGET"
rustup target add "$X86_TARGET"

mkdir -p "$OUTPUT_DIR"
VERSION="$(detect_version)"
echo "==> Detected version: $VERSION"

build_arch_dmg "$ARM_TARGET" "aarch64" "$VERSION"
build_arch_dmg "$X86_TARGET" "x86_64" "$VERSION"
build_universal_dmg "$VERSION"

echo "==> Done. Three DMGs are in: $OUTPUT_DIR"
