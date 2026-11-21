# Validation

Verification results, the environment they were obtained in, and their remaining limits are recorded here.

## Phase 1 – Scaffold and browser feasibility (2026-11-21)

Environment: macOS (Apple silicon), Rust 1.99.1 stable, Flutter 3.50.1, `flutter_rust_bridge` 2.13.0, `wasm-pack`; browser: the built-in browser pane of the development app.

| Check | Result |
|---|---|
| `cargo fmt`, `cargo clippy -D warnings`, `cargo test` over the workspace | Pass |
| `dart format`, `flutter analyze`, `flutter test` (two widget tests with a fake recording source) | Pass |
| Native vertical slice: HTTP client adapter lists recordings served by the real server router (integration test) | Pass |
| Browser vertical slice: the Flutter web build loads the Rust client core as WASM through `flutter_rust_bridge`, fetches the recording list from the running server over HTTP and shows it | Pass (manual, and automated, see below) |
| Automated browser test (Playwright, headless Chromium): real server, built web app, expects both recordings and "Verbunden" and no console errors; fails when an expectation is wrong | Pass locally |
| Real-time channel: the server greets over WebSocket; the Rust client core (`ewebsock`) receives the greeting, an `rxrust` pipeline (`map`, `distinct_until_changed`) reduces events to status changes, and the Flutter UI shows "Verbunden" | Pass natively (integration test with the real router) and in the browser (manual, once) |
| GitHub Actions CI on `main` (Rust, Flutter, web build incl. check that generated bridge code is up to date), ubuntu-latest | Pass (run of 2026-11-21) |
| `rxrust` in the browser | Pass: the pipeline runs inside the WASM client core in the manual browser run above |

Limits:

- The browser test covers one browser (Chromium) and one happy path, with the server on another origin (exact origin allowed by a server setting). Other browsers, disconnect and reconnect are not tested.
- The WASM bundle is built without threads: `flutter_rust_bridge_codegen build-web` requires nightly `-Z build-std`, so `scripts/build-web.sh` calls `wasm-pack` on stable. The browser warns that buffers cannot be shared without cross-origin headers.
- Without threads, bridge functions must be `async`: a synchronous function is dispatched to a worker pool and panics in the browser ("fail to create WorkerPool"). Verified with `watch_connection`.
- The channel only carries the greeting. Reconnection, the real message set, throttling and clock synchronisation are not implemented or examined.
- The first CI web-build run took about 12 minutes because it compiled the codegen and `wasm-pack` from source; the tools are cached now (not yet measured).
