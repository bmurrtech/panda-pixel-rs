#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PRODUCT_SLUG="panda-pixel-rs"
OUTPUT_DIR="$ROOT_DIR/target/linux-artifacts"

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

echo "==> [linux-build] Environment"
uname -a || true
rustc --version
cargo --version
trunk --version
cargo tauri --version

VERSION="$(detect_version)"
echo "==> [linux-build] Detected version: $VERSION"

echo "==> [linux-build] Building frontend"
./scripts/build-frontend.sh

echo "==> [linux-build] Building Tauri bundles"
(
  cd src-tauri
  cargo tauri build
)

mkdir -p "$OUTPUT_DIR"

echo "==> [linux-build] Collecting versioned artifacts"
FOUND=0
for root in "$ROOT_DIR/target" "$ROOT_DIR/src-tauri/target"; do
  if [[ -d "$root" ]]; then
    while IFS= read -r src_file; do
      base_name="$(basename "$src_file")"
      dst_file="$OUTPUT_DIR/${PRODUCT_SLUG}-${VERSION}-linux-${base_name}"
      cp -f "$src_file" "$dst_file"
      echo "Created: $dst_file"
      FOUND=1
    done < <(find "$root" -type f \( -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" \) 2>/dev/null || true)
  fi
done

if [[ "$FOUND" -ne 1 ]]; then
  echo "No Linux artifacts were found (.deb/.rpm/.AppImage)." >&2
  exit 1
fi
