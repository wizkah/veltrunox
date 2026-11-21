#!/bin/sh
# Cross-builds the server for a 64-bit Raspberry Pi (Debian 12 or newer) on macOS or Linux.
# Extra arguments go to cargo, for example --features embedded-web (see scripts/build-release.sh).
# Output: target/aarch64-unknown-linux-gnu/release/veltrunox-server
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
for tool in cargo rustup curl ar tar shasum; do
    command -v "$tool" >/dev/null || { echo "Missing tool: $tool" >&2; exit 1; }
done
if ! command -v zig >/dev/null || ! command -v pkg-config >/dev/null; then
    command -v brew >/dev/null || { echo 'Homebrew is required to install Zig and pkg-config.' >&2; exit 1; }
    if ! command -v zig >/dev/null; then brew install zig; fi
    if ! command -v pkg-config >/dev/null; then brew install pkg-config; fi
fi
if ! cargo zigbuild --help >/dev/null 2>&1; then
    cargo install cargo-zigbuild --locked
fi
rustup target add aarch64-unknown-linux-gnu
target=aarch64-unknown-linux-gnu
arch=aarch64-linux-gnu
mirror=https://deb.debian.org/debian/pool/main/a/alsa-lib
work=$(mktemp -d "${TMPDIR:-/tmp}/veltrunox-sysroot.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
sysroot="$work/sysroot"
mkdir -p "$sysroot" "$work/package"
# ALSA headers and libasound for the audio output (cpal); checked against Debian 12 packages.
fetch() {
    curl -fsSL "$1" -o "$work/package/package.deb"
    printf '%s  %s\n' "$2" "$work/package/package.deb" | shasum -a 256 -c -
    (cd "$work/package" && ar -x package.deb && tar -xJf data.tar.xz -C "$sysroot" && rm -f control.tar.* data.tar.* debian-binary)
}
fetch "$mirror/libasound2_1.2.8-1+b1_arm64.deb" \
    9fa889400fcee4b92c8f4a2fafbb7f2cd33444d9ec1665a71002ab67c06114bb
fetch "$mirror/libasound2-dev_1.2.8-1+b1_arm64.deb" \
    263668e5ce831d3a8426bd342beba7e94925e893a4a5a285252b8f3ed6b3f5ee
cd "$root"
# The Stretch wrapper builds through bindgen and needs libclang; Xcode's or the system's is used.
PKG_CONFIG_ALLOW_CROSS=1 \
PKG_CONFIG_SYSROOT_DIR="$sysroot" \
PKG_CONFIG_LIBDIR="$sysroot/usr/lib/$arch/pkgconfig" \
cargo zigbuild --release --locked -p veltrunox-server --target "$target.2.36" "$@"
echo "Built: $root/target/$target/release/veltrunox-server"
