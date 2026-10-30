#!/usr/bin/env bash
# Build Mindsphere for the web and stage a GitHub Pages-ready directory.
#
# Usage: tools/build_web.sh [output-dir]   (default: build/web)
#
# Needs: rustup target add wasm32-unknown-unknown
#        wasm-bindgen-cli matching Cargo.lock
# Optional: wasm-opt (Binaryen) for a smaller bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${1:-$ROOT/build/web}"
WASM_BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
PROFILE_DIR="$ROOT/target/wasm32-unknown-unknown/wasm-release"
WASM="$PROFILE_DIR/mindsphere.wasm"

cd "$ROOT"

echo "==> building wasm (profile: wasm-release)"
cargo build --profile wasm-release --target wasm32-unknown-unknown --bin mindsphere

if [[ ! -f "$WASM" ]]; then
  echo "wasm artifact not found at $WASM" >&2
  exit 1
fi

rm -rf "$DIST"
mkdir -p "$DIST"

echo "==> wasm-bindgen"
"$WASM_BINDGEN" \
  --target web \
  --no-typescript \
  --out-name mindsphere \
  --out-dir "$DIST" \
  "$WASM"

if command -v wasm-opt >/dev/null 2>&1; then
  echo "==> wasm-opt -Oz"
  wasm-opt -Oz --output "$DIST/mindsphere_bg.opt.wasm" "$DIST/mindsphere_bg.wasm"
  mv "$DIST/mindsphere_bg.opt.wasm" "$DIST/mindsphere_bg.wasm"
else
  echo "==> wasm-opt not found; shipping the unoptimised wasm"
fi

echo "==> staging static shell and runtime assets"
cp "$ROOT/web/index.html" "$ROOT/web/style.css" "$ROOT/web/favicon.svg" "$DIST/"
mkdir -p "$DIST/assets"
cp -R "$ROOT/assets/fonts" "$DIST/assets/fonts"
cp -R "$ROOT/assets/textures" "$DIST/assets/textures"
touch "$DIST/.nojekyll"

echo
echo "built: $DIST"
du -sh "$DIST"
find "$DIST" -maxdepth 1 -type f -printf '  %-28f %10s bytes\n' | sort
