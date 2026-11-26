# ADR-015 – Stems and structure analysis

**Status:** Accepted  
**Date:** 2026-11-26  
**Amends:** ADR-004 (stem files), ADR-005 (mix intents), ADR-006 (the engine plays a mix)

## Context

For practising, single parts of a recording (drums, bass, vocals, the rest) shall be heard louder, quieter or not at all, and the song structure (verse, chorus …) shall appear as regions. A trial with `all-in-one-infer` and Demucs produced both, but needs Python, PyTorch and up to about 2.8 GB of memory, which the Raspberry Pi does not have. Server and desktop application share the server code (ADR-014).

## Decision

**Terms.** A *stem* is one audio track separated from a recording, as long as the recording. A recording has no stems or one *stem set* of at most eight; their names come with the set, so four or six stems work alike. A *section* is a part of the song structure and becomes an ordinary region.

**Storage.** Stem files are content-addressed files in the server's media storage, as ADR-004 stores recordings; the database holds name, order, hash and waveform of each. Audio is not stored inside the database. Stems are lossy compressed, Ogg Vorbis recommended (about 190 kbit/s), which has no encoder delay that would shift stems against each other.

**Package.** Stems travel as a ZIP file: `manifest.json` (format version, content hash and title of the recording, stem names and files, optionally tempo, beats, downbeats and sections) plus the audio files. Importing assigns it to the recording with that content hash, replaces an existing stem set as a whole, and adds the sections as regions. Everything is checked before anything is stored. Exporting writes the same format.

**Sections as regions.** Names are the section label with a running number ("Verse 1"). Start and end move to the nearest downbeat. A section is not added when a region with the same name or nearly the same range exists.

**Mix.** The shared playback state carries a mix: whether stems are used, and per stem volume, mute and solo. Mute wins over solo; with any solo only soloed stems sound. The mix is stored with the recording like speed, tuning and key (ADR-005). With stems switched off the original recording plays. The engine sums the stems before time stretching and pitch shifting, so there is one stretcher and the stems cannot drift apart.

**Analysis.** The separation and structure analysis is an external program configured in the server, not part of the build: it gets the audio file and writes a package. The server runs one such job at a time, reports its progress and imports the result. Without a configured program everything except starting an analysis works. Models and their weights are not distributed with Veltrunox; their licences (non-commercial or research only for the trial's models) are the operator's matter.

## Alternatives considered

- Audio as database blobs: one file to back up, but a large database and slow backups (ADR-004).
- MP3: widely known, but encoder delay and padding need correction per file.
- Porting the models to Rust or ONNX: no Python, but the structure model has no such port; may replace the external program later without changing the package.
- Mixing in the client: contradicts ADR-003.

## Consequences

Backups include the stem files in the media directory. A four-stem set of a 4.5-minute song takes about 25 MB on disk. The engine holds decoded audio in memory: four stems need four times the memory of a recording, which is too much for the Raspberry Pi until the engine reads audio in blocks; this must be solved before stems are used there. Results of the analysis are estimates and are editable as regions.
