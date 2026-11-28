#!/usr/bin/env bash
# Installs the analysis program into a Veltrunox data directory, where the server finds it.
# Needs uv. Usage: install.sh DATA_DIR
# For the macOS app: install.sh "$HOME/Library/Application Support/Veltrunox"
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
target="${1:?usage: install.sh DATA_DIR}/analysis"
mkdir -p "$target"
uv venv --python 3.11 "$target/venv"
uv pip install --python "$target/venv/bin/python" -r "$here/requirements.txt"
cp "$here/analyze.py" "$here/openvino_demucs.py" "$target/"

# Intel's conversion of Demucs v4, at a fixed revision and checked against known sums.
models=https://huggingface.co/Intel/demucs-openvino/resolve/efb3d2bc54f93899b13b72b12c57001e19f09ae2
while read -r sum file; do
  path="$target/models/$file"
  if ! echo "$sum  $path" | shasum -a 256 -c --status 2>/dev/null; then
    mkdir -p "$(dirname "$path")"
    curl -fL --progress-bar -o "$path" "$models/$file"
    echo "$sum  $path" | shasum -a 256 -c --status || { echo "wrong checksum: $file" >&2; exit 1; }
  fi
done <<'SUMS'
74c4e1ebd68b648ea5a2aabefb65d5c3ba805658037fc97a37b84ce9c72f4eb1  htdemucs_v4/htdemucs_fwd.bin
1557f96f11cf1667beca60679926bf43b393d3ff71eb4b9a625c6a07bc23f4a4  htdemucs_v4/htdemucs_fwd.xml
5d66281099b5fdaba8844e306f3d6710b3e0b005fa9aad4514ce7604e24fbec9  htdemucs_v4_6s/htdemucs_fwd.bin
06c68a8a21a6cefb905575cbe689cfa3241416a4c263271804ae5690f8ba5cbf  htdemucs_v4_6s/htdemucs_fwd.xml
SUMS
cat > "$target/run" <<'RUN'
#!/bin/sh
here="$(cd "$(dirname "$0")" && pwd)"
exec "$here/venv/bin/python" "$here/analyze.py" "$@"
RUN
chmod +x "$target/run"
echo "installed: $target/run (restart the server)"
