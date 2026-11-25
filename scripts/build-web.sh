#!/usr/bin/env bash
# Builds the Rust client core for the browser and the Flutter web app.
# Needs: Flutter, wasm-pack, flutter_rust_bridge_codegen 2.13.0, target wasm32-unknown-unknown.
#
# `flutter_rust_bridge_codegen build-web` requires nightly `-Z build-std` (threads);
# the client core needs no threads, so wasm-pack runs directly on stable.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root/app"

flutter_rust_bridge_codegen generate
RUSTFLAGS="--cfg=web_sys_unstable_apis" wasm-pack build \
  --target no-modules --release --no-typescript \
  --out-dir "$root/app/web/pkg" --out-name veltrunox_client_bridge \
  "$root/crates/client-bridge"
rm -f web/pkg/package.json
# The rehearsal room may have no internet, so CanvasKit and fonts are bundled instead of
# fetched from Google's CDN.
flutter build web --no-web-resources-cdn --no-source-maps \
  --dart-define=BUILD_TIME="$(date '+%Y-%m-%d %H:%M')" "$@"
# The app uses the CanvasKit renderer: browsers load only canvaskit/ (Safari, Firefox) or
# canvaskit/chromium/ (Chrome, Edge). Drop the other renderer variants and debug symbols.
cd build/web/canvaskit
rm -rf skwasm* wimp* webparagraph
find . -name '*.symbols' -delete
