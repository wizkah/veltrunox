#!/usr/bin/env bash
# Xcode build phase of the macOS app: builds the Rust library of the app and puts it into
# the bundle's Frameworks folder. Needs Rust with the host target; Apple silicon only.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
library=libveltrunox_client_bridge.dylib

if [[ "${CONFIGURATION:-Debug}" == Debug ]]; then
  profile=debug flags=()
else
  profile=release flags=(--release)
fi
cargo build --manifest-path "$root/Cargo.toml" -p veltrunox-client-bridge ${flags[@]+"${flags[@]}"}

destination="$TARGET_BUILD_DIR/$FRAMEWORKS_FOLDER_PATH"
mkdir -p "$destination"
cp "${CARGO_TARGET_DIR:-$root/target}/$profile/$library" "$destination/$library"
install_name_tool -id "@rpath/$library" "$destination/$library"
codesign --force --sign "${EXPANDED_CODE_SIGN_IDENTITY:--}" "$destination/$library"

# The web app the server offers to browsers on the network (scripts/build-web.sh).
web="$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH/web"
rm -rf "$web"
if [[ -f "$root/app/build/web/index.html" ]]; then
  cp -R "$root/app/build/web" "$web"
else
  echo "warning: no web app in app/build/web; browsers will get nothing"
fi
