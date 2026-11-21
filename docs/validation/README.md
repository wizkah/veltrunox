# Validation

Verification results, the environment they were obtained in, and their remaining limits are recorded here.

## Phase 1 – Scaffold and browser feasibility (2026-11-21)

Environment: macOS (Apple silicon), Rust 1.99.1 stable, Flutter 3.50.1, `flutter_rust_bridge` 2.13.0, `wasm-pack`; browser: the built-in browser pane of the development app.

| Check | Result |
|---|---|
| `cargo fmt`, `cargo clippy -D warnings`, `cargo test` over the workspace | Pass |
| `dart format`, `flutter analyze`, `flutter test` (two widget tests with a fake recording source) | Pass |
| Native vertical slice: HTTP client adapter lists recordings served by the real server router (integration test) | Pass |
| Browser vertical slice: the Flutter web build loads the Rust client core as WASM through `flutter_rust_bridge`, fetches the recording list from the running server over HTTP and shows it | Pass (manual, once) |
| `rxrust` operator chain (`distinct_until_changed`) | Pass in a native unit test; compiles for `wasm32`, not run in the browser |

Limits:

- The browser run was manual and visual, in one browser, with the server on another origin (exact origin allowed by a server setting). It is not automated and not part of CI.
- The WASM bundle was built without threads: `flutter_rust_bridge_codegen build-web` requires nightly `-Z build-std`, so `wasm-pack` was called directly on stable. The browser warns that buffers cannot be shared without cross-origin headers; this does not affect the current call.
- A WebSocket client under `wasm32`, clock handling and `rxrust` at runtime in the browser are not examined. The real-time channel is not implemented.
- The CI workflow has not run on a CI host.
