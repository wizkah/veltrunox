# Veltrunox

**V**ersatile **E**ngine for **L**ooping, **T**uning, **R**ehearsal, **U**ploads, **N**avigation, **O**rganization & e**X**change.

<p align="center"><img src="docs/logo/veltrunox-logo.png" alt="Veltrunox" width="320"></p>

Veltrunox helps bands during rehearsals. Recordings live on a server together with bookmarks: labels mark points, regions mark sections. A client plays the whole recording, starts at a label, or loops a region, so nobody searches for the same passage again and again. All connected clients see and control the same playback in real time.

## Planned features

- Server-side library of recordings with labels and regions, editable from every client.
- Shared, real-time playback position across all clients, with scrubbing where possible.
- Waveform control for orientation and for setting and editing labels and regions.
- Detection of tuning (for example 412 Hz), key (for example A major) and tempo (bpm); tuning and key can be adjusted.
- Playback speed from 0.5× to 2× at constant pitch, also by entering a target tempo.
- Upload and management of recordings from the client.
- Roles: admin, user and an optional guest.

## Status

Design phase. There is no code yet. Architecture and coding rules are in `docs/`.

## Technology

Rust for server and client core, Flutter for the user interface. The browser (WebAssembly) is the first client target.

## Documentation

| Path | Content |
|---|---|
| `docs/architecture/Architecture.md` | Architecture |
| `docs/architecture/adr/` | Architecture decisions |
| `docs/development/Clean-Code.md` | Coding guidelines |
| `docs/development/GUI-Architecture.md` | GUI architecture |
| `docs/logo/` | Logo |
| `docs/validation/` | Verification results and their limits |
| `CHANGELOG.md` | Changes |

## License

Not yet decided.
