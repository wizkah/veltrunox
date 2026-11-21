# Changelog

## Unreleased

- Project set up: architecture, ADRs (technology stack, GUI architecture, server-side audio output), Clean Code and GUI architecture guidelines, logo.
- Rust workspace with core, protocol, server and client crates; server lists recordings over HTTP, client core fetches them; CI with format, lint, tests and a `wasm32` build check.
- Flutter web app (`app/`) with a library feature that lists recordings through the Rust client core (`flutter_rust_bridge`).
- Real-time channel (WebSocket) with a server greeting; the client core turns link events into connection status with `rxrust`; the web app shows it.
- `scripts/build-web.sh` builds the client core for the browser and the Flutter web app; CI builds it and checks the generated bridge code.
- Licensed under GPL-3.0-only.
