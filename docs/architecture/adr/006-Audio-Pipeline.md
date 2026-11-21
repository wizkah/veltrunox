# ADR-006 – Audio pipeline

**Status:** Accepted  
**Date:** 2026-11-21

## Context

ADR-003 puts decoding, time stretching, pitch shifting and output on the server, including a Raspberry Pi. Requirements: speed 0.5×–2× without pitch change, tuning and key shifts, a gap-free loop over a region, audible start within 500 ms of choosing a position. Audio must run in real time without allocating or blocking on the audio thread.

An earlier prototype played through LibVLC (output module and ALSA device chosen by environment variables, `--audio-time-stretch`) and, on macOS, through `AVAudioUnitTimePitch`. On the Raspberry Pi it ran as a systemd service with mpv (VLC as fallback) on ALSA, outputting to the headphone jack; no latency, load or quality figures were recorded. It had no device list, no defined behaviour without a device, and a loop that depended on the player's seek accuracy.

## Decision

| Concern | Choice | Reason |
|---|---|---|
| Decoding | `symphonia` (pure Rust) | No native dependency; covers WAV, MP3, FLAC, AIFF, M4A (AAC), OGG; also feeds waveform and analysis later |
| Output | `cpal` (ALSA on Linux, CoreAudio on macOS) | Device list and selection from Rust; callback model fits a non-blocking audio thread |
| Time stretch and pitch | `signalsmith-stretch` (MIT) | Good quality at 0.5×–2× and ±12 semitones; one engine for speed and pitch; light enough for ARM (to be measured) |
| Sample-rate conversion | None of its own: the stretcher's input/output length ratio absorbs the difference between file and device rate, and the pitch factor compensates it | One resampling step instead of two; covered by a test |

**Decoded audio is held in memory** for the loaded recording (PCM, `f32`), so seeking and loops are index arithmetic, not decoder seeks. The playback thread reads ahead of the audio callback through a lock-free ring buffer; commands (position, speed, pitch, loop, run/pause) reach it through a bounded channel. The loop wraps at the sample level inside the engine, so there is no gap by construction.

**Port.** `AudioOutput` (Architecture, section 3) is extended by the engine behind it: the use case sends the shared playback state; the adapter renders it. Position reported back is derived from played samples, not from wall-clock time.

**Tuning and key** become one pitch shift in semitones: key shift plus `12·log2(tuning / analysed tuning)`; before analysis exists (phase 4) the reference is 440 Hz.

**Speed from target tempo** is a pure core operation: `target bpm / recording bpm`, with a hint when the result lies outside 0.5×–2×.

**Device.** Listed and chosen by the admin, stored persistently. The device list shows the Pi's headphone jack first. The service user needs the `audio` group. Without a device (none present or vanished) the server keeps the shared state running silently and reports "no audio device" to clients; it never refuses transport commands.

**Source of audio until upload exists (phase 6).** The server reads a configured directory at start and adds one recording per file not yet known (by content hash). Upload later writes to the same store.

## Consequences

- No `ffmpeg`, VLC or mpv on the Pi any more; `symphonia` replaces them. All Rust, no C++ toolchain on the Pi; Signalsmith's wrapper compiles C++ and needs a C++ compiler for builds; to be checked in CI and for ARM before it is relied on. Fallback: SoundTouch.
- Whole-file PCM costs memory (a 10-minute stereo 44.1 kHz recording is about 210 MB as `f32`); to be measured on the Pi, with `i16` or streamed decoding as fallback. The 200 MB upload limit is a different bound.
- Latency, CPU load and audible quality at the extremes are measured on target hardware and recorded in `docs/validation/`; audible scrubbing is decided from these numbers (ADR-005).
- Supported formats follow the decoder; the open question on formats in the project memory is answered by it.
