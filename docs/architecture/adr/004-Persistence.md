# ADR-004 – Persistence

**Status:** Proposed  
**Date:** 2026-11-21

## Context

The server keeps recordings, bookmarks, users and per-recording playback settings across restarts. It is a single process, runs on Linux including a Raspberry Pi, and needs no external database service. Audio files can be large (a configurable upload limit applies).

## Decision

- **Metadata** is stored in SQLite through `rusqlite` (bundled library), in WAL mode, behind the `RecordingRepository` and `UserRepository` ports. Schema changes are versioned SQL migrations applied at start-up.
- **Audio files** are stored content-addressed by SHA-256 in a media directory (`data/media/<first two hex digits>/<hash>`). Identical uploads share one file. A file is written to a temporary name, flushed and then renamed. The database holds the original file name and format.
- **Derived data** (waveform peaks and similar) is stored beside the media file by hash and can be regenerated.
- **Bookmarks** carry a version counter per record; writes state the expected version (see ADR-005).
- **Playback settings** (speed, tuning, key) are columns of the recording. Analysis results are separate columns and are never overwritten by them.
- **Not persisted:** the running playback state (position, run/pause). After a restart the server is paused.
- **Identifiers** are integers assigned by the database, wrapped in dedicated types.
- Only an admin deletes recordings. Media files that no recording references are removed by a clean-up step.
- The data directory is configurable and excluded from version control.

## Alternatives considered

- `sqlx`: asynchronous and checks queries at compile time, but heavier to build and cross-compile; the domain operations are synchronous anyway.
- A key-value store (`redb`, `sled`): would need hand-written query logic for bookmarks and library views.
- Audio as database blobs: bloats the database and its backups.

## Consequences

One file plus a media directory make backup and restore simple (SQLite online backup plus a directory copy; to be implemented in the hardening phase). Database calls are blocking and run off the async runtime threads. Tests use in-memory SQLite. Cross-compiling for ARM needs a C toolchain for the bundled SQLite; to be checked in CI before the first ARM build.
