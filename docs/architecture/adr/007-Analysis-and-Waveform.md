# ADR-007 – Analysis and waveform

**Status:** Accepted  
**Date:** 2026-11-22

## Context

The server determines reference tuning, key and tempo of every recording and offers waveform data for the timeline control. Both start from decoded samples (ADR-006), must run on a Raspberry Pi and must not delay playback: a recording is playable before its analysis finishes. An earlier prototype estimated all three values from a short-time spectrum without a native audio-analysis library; the approach worked well in practice and is kept, with a finer tempo estimate (the prototype's resolution of one onset frame was too coarse to derive a speed from a target tempo).

## Decision

| Concern | Choice |
|---|---|
| FFT | `rustfft` (pure Rust, native server only) |
| Input | Mono mix of the decoded file at the file's sample rate; Hann window, FFT size 4096, hop 1024 |
| Spectral peaks | Local maxima with parabolic interpolation between 80 Hz and 4 kHz; frames without a clear tonal peak are skipped; the strongest 15 per frame, weighted by `ln(1 + magnitude)` |
| Tuning | Histogram of the peaks' cent residuals against a candidate A4 within ±50 cents of 440 Hz (coarse, then fine search). Whole-semitone offsets are not searched: they are a transposition of the recording and show up in the key. The result is the residual tuning near 440 Hz |
| Key | Chroma from the peaks, relative to the estimated tuning; cosine similarity against the 24 Krumhansl–Kessler major/minor profiles. The key is the sounding key |
| Tempo | Spectral flux (log-compressed) as onset strength; the score of a candidate tempo is the sum of autocorrelation values at its first four period multiples, searched in 60–200 bpm in steps of 0.1 bpm with interpolation between lags. A regular pulse also scores at slower tempi whose beats it contains, so among peaks scoring within 5 % of the best the fastest wins. Octave errors are folded into 80–180 bpm |
| Waveform | Min/max peak pairs per block over the mono mix, in levels from coarse to fine (each level has half the blocks of the next finer one); stored in the database and served per level |
| Execution | A background task per recording after import; results are stored with the recording, never overwritten by playback settings. Re-running replaces the stored result |

Confidence values (ratio of the best to the second-best score) are stored next to each result but are not shown as facts: they only mark estimates that are likely wrong.

**Domain.** `RecordingAnalysis` (tuning, key with mode, tempo, confidences) is a core type. The estimators are pure functions on samples in the server adapters; the use case reaches them through the `RecordingAnalyzer` port (Architecture, section 3).

**Pitch reference.** The stored tuning replaces the 440 Hz placeholder of ADR-006; the playback uses it as soon as it exists, also while the recording is playing. Until then the placeholder stays in effect.

**Target tempo.** The wanted tempo is turned into a speed in the client core (`speed_for_target_tempo`) and sent as an ordinary speed intent; the server needs no tempo intent. Resetting a setting to the analysed value is a plain intent with that value (speed 1×, key shift 0, tuning of the recording).

## Consequences

- Tempo is one value per recording; at fast tempi (about 190 bpm) the onset grid limits the resolution to under 1 %. Tempo changes within a recording (a live band, a rubato part) give an average or a wrong octave; per-region tempo stays a later option.
- Chroma from peaks is cheaper than a constant-Q transform but less robust for dense mixes; accuracy is measured on synthetic signals with known values and on real rehearsal recordings by plausibility (no reference values exist for those). Results and limits go to `docs/validation/`.
- The analysis decodes the file once more, in addition to playback. Memory use grows with the file length (about 21 MB per minute as stereo `f32`); besides the decoded file the analysis holds a mono copy (about 10 MB per minute at 44.1 kHz) and the peaks of all frames (about 1 MB per minute) while it runs.
- Waveform levels in the database avoid decoding when a client opens a recording; their size is a few kilobytes per minute at the finest level.
