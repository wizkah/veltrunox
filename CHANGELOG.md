# Changelog

## Unreleased

- Project set up: architecture, ADRs (technology stack, GUI architecture, server-side audio output), Clean Code and GUI architecture guidelines, logo.
- Rust workspace with core, protocol, server and client crates; server lists recordings over HTTP, client core fetches them; CI with format, lint, tests and a `wasm32` build check.
- Flutter web app (`app/`) with a library feature that lists recordings through the Rust client core (`flutter_rust_bridge`).
