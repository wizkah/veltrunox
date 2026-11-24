#!/bin/sh
# Installs the Veltrunox server as a systemd service on a Raspberry Pi (or any Debian system).
# Run as root from the folder that holds this script, veltrunox-server and veltrunox.toml.
# Existing configuration and data are kept, so running it again updates the binary only.
set -eu
[ "$(id -u)" -eq 0 ] || { echo "Run as root (sudo ./install.sh)." >&2; exit 1; }
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
[ -f "$here/veltrunox-server" ] || { echo "veltrunox-server not found next to install.sh." >&2; exit 1; }

user=veltrunox
data=/var/lib/veltrunox
config=/etc/veltrunox/veltrunox.toml

# The audio group gives the service access to the sound card.
id "$user" >/dev/null 2>&1 || useradd --system --home-dir "$data" --create-home --shell /usr/sbin/nologin "$user"
usermod -aG audio "$user"

install -m 0755 "$here/veltrunox-server" /usr/local/bin/veltrunox-server
install -d /etc/veltrunox
if [ ! -f "$config" ]; then
    install -m 0644 "$here/veltrunox.toml" "$config"
    # Absolute, so the command line finds the data from any directory.
    sed -i "s|^data_dir = .*|data_dir = \"$data/data\"|" "$config"
    echo "Wrote $config; set address and audio_device there."
fi

cat > /etc/systemd/system/veltrunox.service <<UNIT
[Unit]
Description=Veltrunox server
After=network-online.target sound.target
Wants=network-online.target

[Service]
User=$user
# Relative paths in the configuration (the database directory) are relative to this directory.
WorkingDirectory=$data
ExecStart=/usr/local/bin/veltrunox-server --config $config
Restart=on-failure

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable veltrunox.service
systemctl restart veltrunox.service
echo "Installed. Logs: journalctl -u veltrunox -f"
