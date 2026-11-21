#!/bin/sh
# Builds the web app and a release server with the web app baked in.
# Usage: scripts/build-release.sh [host|pi]   (default: host)
# Output: dist/veltrunox-server-<target>/ with the binary, veltrunox.toml and, for the Pi, install.sh.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
target=${1:-host}
cd "$root"
"$root/scripts/build-web.sh"
case "$target" in
host)
    cargo build --release --locked -p veltrunox-server --features embedded-web
    binary="$root/target/release/veltrunox-server"
    ;;
pi)
    "$root/scripts/build-pi.sh" --features embedded-web
    binary="$root/target/aarch64-unknown-linux-gnu/release/veltrunox-server"
    ;;
*)
    echo "Usage: $0 [host|pi]" >&2
    exit 1
    ;;
esac
out="$root/dist/veltrunox-server-$target"
rm -rf "$out"
mkdir -p "$out"
cp "$binary" "$root/dist/veltrunox.toml" "$out/"
if [ "$target" = pi ]; then cp "$root/dist/install.sh" "$out/"; fi
echo "Release: $out"
