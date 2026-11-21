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

## Phase 2 – Domain, real-time state, persistence (2026-11-21)

Environment: macOS (Apple silicon), Rust 1.99.1 stable, native only.

| Check | Result |
|---|---|
| `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace` | Pass |
| `cargo check --target wasm32-unknown-unknown` for core, protocol, client app, client adapters and bridge | Pass |
| Domain and protocol unit tests: transport operations, position extrapolation with speed and loop, value ranges, version check, wire formats, boundary validation | Pass |
| Persistence tests (in-memory and file database): settings and bookmarks survive reopening, stale versions conflict | Pass |
| Integration, real server and Rust clients: a seek by one client reaches another; a client joining during playback gets the current state first; concurrent intents give every client the same final state; a rejection reaches only its sender; invalid values are refused at the server | Pass |
| Reconnection, real server and Rust client (separate runtime dropped and restarted on the same port): the channel reports the loss, refuses sending meanwhile, reconnects and receives the current state; a channel started before the server exists keeps retrying until it appears | Pass (native, repeated runs) |
| Backoff policy unit tests: delays double up to the cap, a greeting starts them over, a failure and its close schedule one retry | Pass |
| Flutter: `dart format`, `flutter analyze`, `flutter test` (player bar and library with a fake player source) | Pass |
| Browser (Playwright, headless Chromium, real server, built web app): two clients share select, play and pause; a client joining during playback shows the running state | Pass (two runs) |
| GitHub Actions CI on `main` (Rust, Flutter, web build), ubuntu-latest | Pass (run of 2026-11-21; the first run failed on a race in one of the new tests, fixed) |
| Integration: label announced to other clients and loadable; stale edit yields 409 and loses nothing; restart keeps recordings, bookmarks and playback settings, the server starts idle | Pass |

Limits:

- The browser test covers one browser (Chromium), two clients on one machine and the play/pause path; speed, tuning and key controls are covered by widget tests only. Buttons are pressed with `dispatchEvent('click')` because another accessibility node covers them for real clicks.
- Latency (NFR-001) is not measured; the tests only show correctness.
- Not covered: server ping timeout, the periodic clock measurement (only its start at connection is exercised), clock offset against a real offset, any authentication.

## Phase 2 – Reconnection in the browser (2026-11-21)

Environment: macOS (Apple silicon), headless Chromium (Playwright), real server binary, web app built with `scripts/build-web.sh`.

| Check | Result |
|---|---|
| The page shows "Verbunden"; the server process is stopped; the page shows "Getrennt"; the server is started again on the same port and data directory; the page shows "Verbunden" without reload | Pass (repeated; the same test failed against a stale web build without the reconnect code) |

Limits: one browser, one outage of a few seconds (first backoff steps only); the 30 s cap and long outages are covered by the native unit tests only. CI integration of this test is not yet run on GitHub.

## Phase 3 – Playback engine (2026-11-21)

Environment: macOS (Apple silicon), Rust 1.99.1 stable, native only; synthetic sine signals, no recorded music, no listening test, not run on a Raspberry Pi.

| Check | Result |
|---|---|
| Engine at 0.5×, 1× and 2×: measured frequency of a 440 Hz sine stays within 8 Hz; progress of the position matches the speed within 30 ms | Pass |
| Pitch shift of +12 semitones doubles the frequency (within 16 Hz) without changing progress | Pass |
| Device rate 48 kHz with a 44.1 kHz file: pitch and duration unchanged | Pass |
| Loop over 500 ms, several wraps: no 10 ms window falls below 0.4 RMS (no gap or dropout), position stays inside the loop | Pass |
| End of recording: engine finishes and renders silence; seek lands on the new position | Pass |
| Player decisions (load, settings, seek only on jumps, silent scrubbing and resume, no audio file) with a recording sink | Pass |
| WAV file decodes; mono is doubled; a text file is refused | Pass |
| Directory import: formats, order, duplicates by content hash, titles, missing directory | Pass |
| Real server with a WAV in `VELTRUNOX_MEDIA_DIR`: the recording is listed, the default device opens at 48 kHz | Pass (manual, once; no sound played) |

Limits:

- Sound quality (NFR-003), the audible loop wrap (AC-003 tolerance still to be set) and latency from choosing a position to sound (NFR-002) are not measured. They need a listening test and measurement on the target hardware.
- Position is reported as the frames handed to the stretcher; the sound leaves it up to its input latency later. The latency is not yet measured or compensated.
- Whole files are held as `f32` in memory; memory use on the Pi is not measured. Only WAV is decoded in a test; MP3, FLAC, M4A, OGG and AIFF rely on `symphonia` and are not tested here.
- The server clock and the device clock are not compared; a long-running drift between them is not corrected.
