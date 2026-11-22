# Changelog

## Unreleased

- Holding the position handle still for a moment lets the recording play on from there (if it was playing); moving it again scrubs again.
- Audible scrubbing: dragging the position handle plays a short piece (120 ms, as recorded, with short fades) at each position instead of staying silent, and the shared position holds under the handle while it is dragged, then runs on from where it is let go.
- Design specification v0.1 and design tokens (`docs/design/`): player, waveform with labels and region lanes, library, states, accessibility and UI acceptance criteria.
- Player: a position slider with scrubbing (the length comes from the stored scan; the slider appears once a recording is analysed); the speed slider takes effect while it is dragged and shows the wanted tempo under the handle; the tuning slider is replaced by a field for the wanted tuning in Hz (300–500, with a message otherwise) that sets it back to the analysed value. Loops and regions are not part of this.
- Output volume (ADR-009): a slider in the player sets the volume of the server for everyone, and every client shows the current value. The server sets the sound card's mixer where there is one (ALSA, for example the Pi's headphone output) and scales the audio itself otherwise; at start-up it reads the mixer.
- Pausing puts the audio engine on the frozen position, so repeated pauses no longer move the sound ahead of the shared position.
- The position display reads the position when the displayed tenth changes while playing, so the tenths advance evenly.
- Release build of the server: settings come from a TOML configuration file (`--config <file>`), which replaces the `VELTRUNOX_*` environment variables (`address`, `data_dir`, `media_dir`, `audio_device`, `allowed_origin`, `web_dir`). The `embedded-web` feature bakes the web app into the binary; `scripts/build-release.sh` builds both, and `dist/install.sh` installs the Pi build as a systemd service. The web app bundles CanvasKit and fonts instead of loading them from Google's CDN (works without internet) and keeps only the CanvasKit variants it uses; the server compresses responses (gzip, Brotli).
- Analysis of tuning, key and tempo (ADR-007): runs in the background after start-up for every recording without a result and can be started again per recording; results are stored with the recording, clients are told when one changes, and the library shows key, tempo and tuning or the state of the analysis.
- Waveform overview and recording length: the background scan decodes each recording once and stores the length and min/max peaks in levels from coarse to fine; `GET /api/v1/recordings/{id}/waveform` describes the levels, `.../waveform/{level}` returns the peaks as bytes. Recordings stored without them are scanned again at start-up. Seeks and loop ends beyond the length are refused, also after a restart.
- The playback pitch uses the analysed tuning of the recording (concert pitch until it exists).
- The player shows the tempo the recording plays at, takes a wanted tempo (speed is derived from it) and sets speed, tuning and key back to the analysed values.

- Project set up: architecture, ADRs (technology stack, GUI architecture, server-side audio output), Clean Code and GUI architecture guidelines, logo.
- Rust workspace with core, protocol, server and client crates; server lists recordings over HTTP, client core fetches them; CI with format, lint, tests and a `wasm32` build check.
- Flutter web app (`app/`) with a library feature that lists recordings through the Rust client core (`flutter_rust_bridge`).
- Real-time channel (WebSocket) with a server greeting; the client core turns link events into connection status with `rxrust`; the web app shows it.
- `scripts/build-web.sh` builds the client core for the browser and the Flutter web app; CI builds it and checks the generated bridge code.
- Licensed under GPL-3.0-only.
- Automated browser test (Playwright) for the web app; CI caches the build tools.
- Domain model: recordings with playback settings, labels and regions with record versions, the shared playback state and the pure transport operations (select, play, pause, seek, speed, tuning, key, loop).
- Real-time protocol (ADR-005): complete playback state with revision on every change, client intents, clock ping, rejections and bookmark change notices; the server validates every value at the boundary.
- SQLite persistence (ADR-004): recordings, playback settings and bookmarks survive restarts; data directory via `VELTRUNOX_DATA_DIR`. The sample recordings are only added to an empty library.
- Bookmark endpoints with version checks (409 on conflict); client core with session view, clock offset estimate and throttled scrubbing.
- Flutter player bar: select a recording, play/pause, restart, speed, tuning and key, shared live between clients; the bridge exposes the session (`watch_session`, `send_command`, `playback_position_ms`). Browser test with two clients.
- Client core reconnects on its own with doubling, capped delays and measures the clock offset on connecting and every minute; the generated bridge no longer exposes a separate clock call.
- Browser test for reconnection: the web app shows "Getrennt" when the server stops and "Verbunden" again after it restarts (`e2e/playwright.reconnect.config.ts`).
- ADR-006: audio pipeline with `symphonia`, `cpal` and `signalsmith-stretch`.
- Playback on the server: audio files are decoded (`symphonia`), stretched and pitch-shifted (`signalsmith-stretch`) and played through the audio device (`cpal`); speed, tuning, key, loop and scrubbing follow the shared state. `VELTRUNOX_MEDIA_DIR` imports audio files as recordings (duplicates by content hash), `VELTRUNOX_AUDIO_DEVICE` picks the device.
- Target tempo to speed (with out-of-range hint) and tuning/key to pitch shift as pure operations.
- `scripts/build-pi.sh` cross-builds the server for a 64-bit Raspberry Pi.
- Playback pauses at the end of the recording; a seek beyond the end is rejected once the length is known; clients are told over the channel whether the server has an audio output.
- The server serves the built web app itself when `VELTRUNOX_WEB_DIR` is set.
- The web app talks to the origin it was loaded from, so the server's address and port are set with `VELTRUNOX_ADDRESS` without rebuilding; browser tests run against the server-hosted app.
