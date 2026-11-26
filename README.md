# Veltrunox

**V**ersatile **E**ngine for **L**ooping, **T**uning, **R**ehearsal, **U**ploads, **N**avigation, **O**rganization & e**X**change.

<p align="center"><img src="docs/logo/veltrunox-logo.png" alt="Veltrunox" width="320"></p>

Veltrunox helps bands during rehearsals. Recordings live on a server together with bookmarks: labels mark points, regions mark sections. A client plays the whole recording, starts at a label, or loops a region, so nobody searches for the same passage again and again. All connected clients see and control the same playback in real time.

## Features

- Server-side library of recordings, with upload, renaming and deletion from the client.
- Labels and regions, editable from every client on a zoomable waveform or in a list.
- One playback state shared by all connected clients. The audio comes out of the server, not the clients.
- Detection of tuning (for example 412 Hz), key (for example A major) and tempo (bpm); tuning and key can be adjusted.
- Playback speed from 0.5× to 2× at constant pitch, also by entering a target tempo.
- Stems with a mixer; an optional external program separates them and finds the song sections (see `tools/analysis/README.md` for the terms of its models).
- Accounts with the roles admin, user and an optional guest.
- Export and import of bookmarks as JSON or CSV, API tokens and an MCP server for LLMs.
- Interface in German and English.

## Status

Under development, no release yet. The server runs on Linux (including a 64-bit Raspberry Pi) and macOS, with the browser as client; there is also a macOS application with the server built in. What has been checked, on which hardware, and what has not is recorded in `docs/validation/`.

Limits to know before using it:

- Transport is plain HTTP by default. Use it in a trusted network only, or switch on [HTTPS](#https). Do not expose the server to the internet; see [SECURITY.md](SECURITY.md).
- Phones and tablets with touch have hardly been checked.
- The macOS application is not signed or notarized.
- Stems need much more memory than a Raspberry Pi has to spare.

## Technology

Rust for server and client core, Flutter for the user interface. The browser (WebAssembly) is the first client target.

## Development

```sh
cargo test --workspace
cargo run -p veltrunox-server -- --config veltrunox.toml
```

The server reads its settings from a TOML file (`--config <file>`; without the option `veltrunox.toml` in the working directory, else `/etc/veltrunox/veltrunox.toml`, else the defaults). All keys are optional; `dist/veltrunox.toml` documents them. By default the server keeps its data in `data/` (ignored by Git).

Users can be administered on the machine itself, with the server stopped or running (for example to set up an admin or after a forgotten password): `veltrunox-server [--config <file>] users list|add|passwd|role|enable|disable|delete|guest|policy`, see `veltrunox-server users` for the details. Passwords are asked for without echo, or read from standard input with `--password-stdin`. Run it as the user that owns the data directory, so the database files keep their owner: `sudo -u veltrunox veltrunox-server users passwd anna`. The installer sets `data_dir` to an absolute path, so the working directory does not matter; with a relative `data_dir` (the default) it does.

Server for a 64-bit Raspberry Pi (Debian 12 or newer), cross-built on macOS or Linux (needs Rust, `curl` and Homebrew or ready-made Zig and `pkg-config`; the script fetches ALSA packages and checks their hashes):

```sh
scripts/build-pi.sh
```

The binary lands in `target/aarch64-unknown-linux-gnu/release/`. On the Pi, name the audio device in the configuration (for example `audio_device = "alsa:plughw:CARD=Headphones,DEV=0"`); the default device may be an unconnected HDMI output.

Web app (needs Flutter, `wasm-pack`, `flutter_rust_bridge_codegen` 2.13.0 and the `wasm32-unknown-unknown` target):

```sh
scripts/build-web.sh
```

Browser test (after `scripts/build-web.sh`; needs Node):

```sh
cd e2e && npm ci && npx playwright install chromium && npx playwright test
```

Release build (web app baked into the server binary, configuration file, optional installer):

```sh
scripts/build-release.sh        # for this machine
scripts/build-release.sh pi     # for a 64-bit Raspberry Pi
```

The result lands in `dist/veltrunox-server-<target>/`. Copy the Pi folder to the Pi and run `sudo ./install.sh`: it installs the binary to `/usr/local/bin`, the configuration to `/etc/veltrunox/veltrunox.toml` (kept on later runs) and a `veltrunox` systemd service with its data in `/var/lib/veltrunox`. Without the installer, start `veltrunox-server --config veltrunox.toml` directly.

Without the `embedded-web` feature (the default for `cargo run`), the server serves a built web app only if `web_dir` is set (for example `web_dir = "app/build/web"`); `address` sets address and port, and the app uses the address it was loaded from, so nothing needs rebuilding. An app served from elsewhere (for example `flutter run`) needs `--dart-define=VELTRUNOX_SERVER_URL=...` and `allowed_origin = "<origin of the web app>"` in the configuration.

## macOS application

The interface and the server in one program (ADR-014). It needs Xcode and, for the web app it offers to browsers, a prior web build.

```sh
./scripts/build-web.sh
cd app && flutter build macos
```

The application lands in `app/build/macos/Build/Products/Release/Veltrunox.app` and keeps its data in `~/Library/Application Support/Veltrunox`. The server for the network is off until switched on in the settings (gear icon).

## HTTPS

Set `tls = "local-ca"` in the configuration and restart. The server then serves `https://<hostname>.local:<port>` (plain `http://` no longer answers) with a certificate from its own authority, which needs no internet. Install the authority once on each device: download `https://<hostname>.local:<port>/ca.crt` (confirm the browser's warning this one time) and compare its SHA-256 fingerprint with the one in the log (`journalctl -u veltrunox`).

- macOS: open the file, add it to the login keychain, then set it to "Always Trust".
- iOS/iPadOS: install the downloaded profile in Settings, then enable it under General → About → Certificate Trust Settings.
- Android: Settings → Security → Install a certificate → CA certificate.
- Windows: install into "Trusted Root Certification Authorities". Firefox keeps its own list (Settings → Certificates).

The authority can only certify `.local` names and local network addresses. Browsers treat plain `http://` on a network address as insecure and then run the app noticeably slower (for example when zooming).

## Bookmarks with an LLM

The server exports and imports bookmarks (labels and regions) as JSON or CSV, and a small program lets an LLM do it through the Model Context Protocol (MCP). Make an API token as a signed-in user (`POST /api/v1/tokens` with `{"name": "llm", "scope": "edit_bookmarks"}`; `read_only` only reads), then register `veltrunox-mcp` with the MCP client, for example in a `.mcp.json`:

```json
{
  "mcpServers": {
    "veltrunox": {
      "command": "/path/to/veltrunox-mcp",
      "env": {
        "VELTRUNOX_URL": "http://raspberrypi.local:8080",
        "VELTRUNOX_TOKEN": "vxt_..."
      }
    }
  }
}
```

The tools are `list_recordings`, `get_recording`, `get_bookmarks` and `set_bookmarks` (replaces all bookmarks of a recording; `dry_run` shows what would change). Without MCP, `GET /api/v1/export` and `POST /api/v1/import?dry_run=true` do the same with files; `GET /api/v1/import/schema` describes the format. Over plain HTTP the token is sent unencrypted. With `tls = "local-ca"` use the `https://` address and set `VELTRUNOX_CA` to the downloaded authority certificate.

## Documentation

| Path | Content |
|---|---|
| `docs/architecture/Architecture.md` | Architecture |
| `docs/architecture/adr/` | Architecture decisions |
| `docs/development/Clean-Code.md` | Coding guidelines |
| `docs/development/GUI-Architecture.md` | GUI architecture |
| `docs/design/` | Design specification and design tokens |
| `docs/logo/` | Logo |
| `docs/validation/` | Verification results and their limits |
| `CHANGELOG.md` | Changes |

## License

GNU General Public License v3.0 only. See `LICENSE`.
