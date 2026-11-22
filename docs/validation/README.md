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

## Phase 3 – Latency, load and memory on Mac and Raspberry Pi (2026-11-21)

Environment: Mac: macOS (Apple silicon), release build of the server. Pi: Raspberry Pi 4 Model B (4 cores, 4 GB), Debian 12 (bookworm), kernel 6.12.47, governor `ondemand`, no throttling reported before and after; the release server was cross-built on the Mac (`cargo zigbuild`, target `aarch64-unknown-linux-gnu`, glibc 2.36) against the Pi's `libasound`. Audio output: Mac default device, Pi headphone jack (`alsa:plughw:CARD=Headphones,DEV=0`). Input: one synthetic recording (3 min, 44.1 kHz stereo WAV, quiet chords with noise). Load generator: two WebSocket clients on the same host (loopback), or on the Mac against the Pi.

| Check | Mac | Pi |
|---|---|---|
| Seek by one client until the other client receives the state, loopback, paused (500 runs) | p50 0.35 ms, p99 1.3 ms, max 2.3 ms | p50 0.30 ms, p99 1.0 ms, max 1.0 ms |
| Same, playing | p50 0.24 ms, p99 2.0 ms, max 9.8 ms | p50 0.61 ms, p99 1.1 ms, max 2.1 ms |
| Same, Mac client against the Pi server over the home network (500 runs each) | – | paused: p50 4.9 ms, p95 13.6 ms, p99 21.4 ms, max 35 ms; playing: p50 4.9 ms, p95 13.0 ms, p99 23.1 ms, max 43 ms |
| Ping round trip over the channel | p50 0.04 ms | loopback p50 0.2 ms; network p50 4.7–5.0 ms, p99 18–24 ms |
| Server CPU while playing (8 s window; 1×, 0.5×, 2×, 1× with +5 semitones, 0.75× with +3 semitones and loop), share of one core | 4.1–4.6 % | 15.3–15.8 % |
| Server CPU idle, no playback | 0.1 % | 1.4 % |
| Server memory while a 3 min recording is loaded (decoded to 32-bit float) | about 68–70 MB | 72 MB (RSS) |
| Server memory idle | – | 9 MB (RSS) |

Limits:

- Latency covers the path from one client's intent to another client's received state. The time from choosing a position to audible sound is not measured: it needs the audio output latency, which cannot be observed without a loopback or microphone measurement. Sound quality, the loop wrap and underruns were not judged by ear.
- CPU stays flat across speed and pitch settings, which suggests the stretcher runs continuously; whether this holds for other material was not examined. Headroom on the Pi is large for one stream; several clients, a browser on the same Pi and long recordings were not combined.
- The Mac's memory figure is the physical footprint (`footprint`); `ps` RSS under-reports it because macOS compresses pages. The Mac's idle footprint reading was not usable. Memory grows with recording length (about 21 MB per minute of stereo audio at 44.1 kHz after decoding); long recordings were not measured.
- The first Mac run reported a buffer underrun message on stream start; it did not recur in later runs. The Pi's default device (HDMI, nothing attached) produced continuous ALSA errors (`snd_pcm_avail_delay`); the headphone device did not, so the device choice matters on the Pi.
- Network figures come from one Mac on an unknown link type (Wi-Fi or Ethernet) at one time of day; the spread (p99 above 20 ms) is probably the link, not the server.
- One measurement run per setup; no repeated runs, no clock offset or drift measurement.

## Phase 4 – Analysis of tuning, key and tempo (2026-11-22)

Environment: macOS (Apple silicon), release build, native only. Not run on the Raspberry Pi.

| Check | Result |
|---|---|
| Unit tests on synthetic inputs: tuning at 440, 432, 447.5 and 428 Hz found within 0.5 cent (peak lists); C major and D minor from chord sequences, key follows a given tuning; tempo 97.3, 120, 143.7 and 88 bpm within 0.35 bpm from pulse trains; 65 and 190 bpm fold to 130 and 95 | Pass |
| End to end on synthesised audio (40 s, chords struck on every beat, A4 435 Hz, C major, 100 bpm): tuning within 3 cents, C major, tempo within 1 bpm; silence gives "no tonal content", a fraction of a second "too short" | Pass |
| Background analysis, real server, in-memory store, a 30 s synthesised WAV: a client is told about the change, the library shows the result (C major, tempo within 1 bpm, tuning within 1 Hz), the stored result equals the shown one; a recording without audio reports why it failed; an unknown recording gives 404 | Pass (integration test) |
| Library over HTTP with the real server on the six bounces (release build, file database, no audio device): all six analysed during start-up, results as in the row below | Pass (manual, once) |
| Playback pitch follows the recording's own tuning; a tuning that arrives while the recording is loaded corrects the pitch without other commands | Pass (unit test of the player logic; not heard) |
| Flutter widget tests: analysis line, pending and failed states, refresh on a notice, re-analyse button, reset buttons, wanted tempo (valid, out of range, not a number) | Pass |
| Browser tests (Playwright, Chromium, rebuilt web app): library, two clients sharing the player, reconnect | Pass after adapting the recording locators to the new tile (it now contains a button) |
| Six real rehearsal bounces (MP3, 73–268 s) | Results plausible, no reference values: *Message in a Bottle* C♯ minor at 148 bpm, A4 440.7 Hz; *Stacy's Mom* E major at 122 bpm, A4 439.2 Hz; three variants of one 73 s take: G♯ minor at 100 bpm, A4 443.5 Hz. Decoding 0.1–0.2 s, analysis 0.1–0.4 s per file on the Mac |

Limits:

- The synthesised audio is one idealised case (clean harmonics, no noise, one tempo, one key). Noisy, dense or detuned mixes, minor keys end to end and other tunings are covered only by the real bounces, which have no known values.
- The real-recording results were judged from knowledge of the songs, not measured against a reference. Confidence ratios of the key were 1.01–1.03 for all files, so they separate nothing yet.
- The tempo of one 73 s take came out as exactly 100.0 with the lowest confidence; whether that is the true tempo or a rounding to the search grid is not examined. Tempo changes inside a recording are not covered.
- Timing on the Raspberry Pi is not measured. The analysis decodes the file a second time next to playback; memory with both at once is not measured.
- The browser tests do not look at analysis values: the browser server has no audio files. The analysis display is covered by widget tests only.
- That the tuning shift is audible and right (a recording analysed at 443 Hz played at 440 Hz) was not listened to.

## Phase 4 – Waveform and recording length (2026-11-22)

Environment: macOS (Apple silicon), debug build, native only. Not run on the Raspberry Pi, not run in a browser.

| Check | Result |
|---|---|
| Unit tests of the level builder: scaling and clamping (out-of-range and NaN samples), level count and block counts, coarse blocks hold the extremes of the finer blocks they cover, odd block counts | Pass |
| Storage in a real SQLite store: levels and length round-trip, a second save replaces the first, a missing recording is refused; recordings with a result but without length or waveform are scanned again | Pass |
| Real server, a 30 s synthesised WAV: length 30 000 ms, finest level of 1 024 frames per block with the expected block count, peaks over HTTP match the block count and are ordered, an unknown level gives 404, an unscanned recording gives 404 | Pass (integration test) |
| A scan whose analysis finds nothing (silence) still stores waveform and length | Pass (unit test with a fake analyser) |
| A seek or loop end beyond the stored length is refused; the length is known after a restart without decoding | Pass (unit tests) |

Limits:

- The waveform was not compared with a real rehearsal recording or drawn; the first consumer is the timeline control. Size and scan time on real files and on the Pi are not measured.
- The length comes from the decoded sample count, not from the container; for lossy formats it can differ from the player's value by the codec delay.

## Release package and scan on the Raspberry Pi (2026-11-22)

Environment: Raspberry Pi 4 Model B (4 GB), Debian 12, kernel 6.12.47, no throttling before and after (`get_throttled` 0x0), 45 °C. Package from `scripts/build-release.sh pi` (cross-built on the Mac), installed with `install.sh` as the `veltrunox` systemd service; configuration: `0.0.0.0:8080`, headphone device, import directory with the six real bounces (MP3, 73–268 s). Client for the timings below: the Mac over the home network.

| Check | Result |
|---|---|
| Fresh install: service starts, imports six files, opens the audio output (48 000 Hz), serves the web app (HTTP 200) | Pass |
| All six bounces analysed after the first start (within about 40 s of the restart), results equal to the Mac values in kind (e.g. *Message in a Bottle* C♯ minor, 148 bpm; *Stacy's Mom* E major, 122 bpm) | Pass (manual, once) |
| Repeated scan of two files (268 s MP3 and a similar one), start to "done": 6.6 s and 6.2 s; process memory peaked at about 190 MB (about 35–55 MB otherwise; sampled every 0.25 s) | Measured, two runs |
| Waveform of the 268 s file: 7 levels, finest 12 550 blocks (25 100 B), coarsest 197 blocks (394 B) | Measured |
| Web app from the Mac: `main.dart.js` 0.8 MB compressed in 0.74 s, index 0.23 s | Measured, once |
| Idle load of the service after the scans | about 0 % CPU (`top`) |

Limits:

- Nothing was listened to; sound, scrubbing and loop on the Pi are not judged here. Latency to the audible tone is not measured (needs a loopback).
- Scan time and memory come from two runs on one file type; a scan next to playback, several clients and a browser on the Pi were not combined. The scan time includes the decode and the analysis, so the two are not separated.
- Web app timing was taken from a Mac over the network, not from a phone or from the Pi's own browser.

## Output volume and pause alignment (2026-11-22)

Environment: macOS (Apple silicon), debug build, native tests; Raspberry Pi 4 Model B (Debian 12), release build cross-built on the Mac, card `Headphones` (mixer control `PCM`, range −102.39 to +4.00 dB). The Pi checks used a small script as the client (WebSocket on the Pi itself), no browser and no listening.

| Check | Result |
|---|---|
| Unit tests: volume limits (0–100, 101 refused), volume changes without a recording and survives selecting, wire format and round trip, a volume command is validated in the client core, the service publishes a volume change without storing it with a recording | Pass |
| Unit tests of the mapping: percent to millibel and back for 0, 1, 25, 50, 99, 100; a 100 dB range is cut to 60 dB; card name from the device name; engine gain 0 at 0 %, 1 at 100 % | Pass |
| Unit tests of the player core: a volume change reaches the output also without a recording, also the first one after start-up; an unchanged volume sends nothing; pause puts the engine on the frozen position, play afterwards sends no seek | Pass |
| Gain ramp of the render thread: steady, moving and unity gain | Pass (unit tests) |
| Flutter widget tests: the slider shows the shared volume, a drag sends commands; throttle passes the first and the latest value and nothing after dispose; time until the next tenth at speed 1 and 2 | Pass |
| Pi, real mixer, one client: the state at start-up carries the mixer's value (−19.88 dB shown as 60 %); setting 30 / 100 / 0 / 60 % is broadcast to the client and read back on the mixer as −38.00 dB / +4.00 dB / minimum / −20.00 dB; 101 is refused with a message | Pass (manual, once) |
| Pi, real browser: two browser windows on the web app; the volume slider works and a change shows in the other window (reported by the user, not measured) | Pass (manual, once) |

Limits:

- The fallback gain on a device without a mixer was tested only as the gain ramp and the mapping; it was not listened to, and no second client watched the Pi's mixer value beyond the two browser windows above.
- Whether the slider steps feel even to the ear (the 60 dB span, the squared fallback curve) is not judged. Nothing was listened to.
- The pause alignment is checked at the level of the commands sent to the engine; whether repeated pausing no longer shifts the sound was not measured on the Pi.
- The evenness of the displayed tenths was judged by eye in a browser by the user, during playback and after pausing ("feels right"); it was not measured. The position is still read through the bridge call, so a slow call would still show.
- A volume changed outside the server (for example with `alsamixer`) is not noticed until the next change through the server.
