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
cp "$here/analyze.py" "$target/analyze.py"
cat > "$target/run" <<'RUN'
#!/bin/sh
here="$(cd "$(dirname "$0")" && pwd)"
exec "$here/venv/bin/python" "$here/analyze.py" "$@"
RUN
chmod +x "$target/run"
echo "installed: $target/run (restart the server)"
