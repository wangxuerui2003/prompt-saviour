#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GUI_DIR="$ROOT/crates/ps-gui"

echo "==> Installing frontend dependencies"
cd "$GUI_DIR"
npm install --silent

echo "==> Building Tauri release bundle (.app + .dmg)"
npm run tauri build

BUNDLE_DIR="$GUI_DIR/src-tauri/target/release/bundle"
echo "==> Build artifacts:"
find "$BUNDLE_DIR" -maxdepth 3 -type f \( -name "*.dmg" -o -name "*.app" \) 2>/dev/null || true

echo "PASS: macOS app bundle build complete"
echo "Note: code signing requires Apple Developer credentials (see docs/development/setup.md)."
