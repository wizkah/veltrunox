# Changelog

## Unreleased

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
