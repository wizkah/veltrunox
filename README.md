# Veltrunox

**V**ersatile **E**ngine for **L**ooping, **T**uning, **R**ehearsal, **U**ploads, **N**avigation, **O**rganization & e**X**change.

<p align="center"><img src="docs/logo/veltrunox-logo.png" alt="Veltrunox" width="320"></p>

Veltrunox helps bands during rehearsals. Recordings live on a server together with bookmarks: labels mark points, regions mark sections. A client plays the whole recording, starts at a label, or loops a region, so nobody searches for the same passage again and again. All connected clients see and control the same playback in real time.

## Planned features

- Server-side library of recordings with labels and regions, editable from every client.
- Shared, real-time playback position across all clients, with scrubbing where possible.
- Waveform control for orientation and for setting and editing labels and regions.
- Detection of tuning (for example 412 Hz), key (for example A major) and tempo (bpm); tuning and key can be adjusted.
- Playback speed from 0.5× to 2× at constant pitch, also by entering a target tempo.
- Upload and management of recordings from the client.
- Roles: admin, user and an optional guest.

## Status

Early scaffold. A Rust workspace with a first vertical slice (server lists recordings, client core fetches them) exists; a Flutter web app lists them. Architecture and coding rules are in `docs/`.

## Technology

Rust for server and client core, Flutter for the user interface. The browser (WebAssembly) is the first client target.

## Development

```sh
cargo test --workspace
VELTRUNOX_ADDRESS=127.0.0.1:8080 cargo run -p veltrunox-server
```

The server keeps its data in `VELTRUNOX_DATA_DIR` (default `data/`, ignored by Git).

Server for a 64-bit Raspberry Pi (Debian 12 or newer), cross-built on macOS or Linux (needs Rust, `curl` and Homebrew or ready-made Zig and `pkg-config`; the script fetches ALSA packages and checks their hashes):

```sh
scripts/build-pi.sh
```

The binary lands in `target/aarch64-unknown-linux-gnu/release/`. On the Pi, name the audio device (for example `VELTRUNOX_AUDIO_DEVICE=alsa:plughw:CARD=Headphones,DEV=0`); the default device may be an unconnected HDMI output.

Web app (needs Flutter, `wasm-pack`, `flutter_rust_bridge_codegen` 2.13.0 and the `wasm32-unknown-unknown` target):

```sh
scripts/build-web.sh
```

Browser test (after `scripts/build-web.sh`; needs Node):

```sh
cd e2e && npm ci && npx playwright install chromium && npx playwright test
```

To run the app by hand, serve `build/web` and start the server with `VELTRUNOX_ALLOWED_ORIGIN=<origin of the web app>`; the app talks to port 8080 on the host it was loaded from (`localhost` or `127.0.0.1`), or to `--dart-define=VELTRUNOX_SERVER_URL=...` if set.

## Documentation

| Path | Content |
|---|---|
| `docs/architecture/Architecture.md` | Architecture |
| `docs/architecture/adr/` | Architecture decisions |
| `docs/development/Clean-Code.md` | Coding guidelines |
| `docs/development/GUI-Architecture.md` | GUI architecture |
| `docs/logo/` | Logo |
| `docs/validation/` | Verification results and their limits |
| `CHANGELOG.md` | Changes |

## License

GNU General Public License v3.0 only. See `LICENSE`.
