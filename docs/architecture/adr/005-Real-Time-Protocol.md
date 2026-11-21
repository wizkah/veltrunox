# ADR-005 – Real-time protocol

**Status:** Proposed  
**Date:** 2026-11-21

## Context

All clients see and control one shared playback state held by the server. The channel must work in the browser, tolerate disconnects, and keep scrubbing responsive. The server also plays the audio (ADR-003).

## Decision

**Transport and format.** One WebSocket per client (`/api/v1/events`), messages as JSON text with a `type` tag. The first server message is `hello` with the protocol version; on a mismatch the server closes with a reason. A binary format is considered only if measurements require it. Request/response operations (library, bookmark edits, upload) use HTTP.

**State.** The server publishes the complete playback state with every change, tagged with a strictly increasing revision: recording, position reference, running, speed, tuning, key, loop range, scrubbing flag. Clients render the latest revision and ignore older ones. A joining or reconnecting client receives the current state first; there is no separate catch-up path.

**Position.** The state carries a reference point: position, server time at which it is valid, and the rate. Clients extrapolate while running. They estimate the clock offset to the server with ping round trips on connect and periodically.

**Intents.** Clients send intents: select recording, play, pause, seek, set speed, set tuning, set key, set loop. The server checks permission and validity, applies the intent in arrival order and publishes the new state. Concurrent intents need no locks; the arrival order decides and every client sees the same result. A rejected intent returns an error message to its sender only.

**Scrubbing.** Dragging sends `seek` intents with `scrubbing: true`, throttled on the client to about 20 per second, always including the last value; releasing sends a final `seek` with `scrubbing: false`. All clients follow continuously. Whether the room hears the scrubbed position is a property of the server's audio output, not of the protocol: the goal is audible scrubbing; if the measured audio latency on the target hardware does not allow it, the server holds the sound while the flag is set and resumes on release. This is decided and documented with the audio output measurements.

**Bookmarks.** Edits use HTTP with the expected record version; a mismatch yields a conflict response. After a successful edit the server sends a change notice over the channel so clients reload the affected recording's bookmarks.

**Connection.** The server pings periodically; a client that misses two pongs is considered gone. The client reconnects on its own with growing, capped delays and shows the connection status. Authentication data is sent in the first client message after the connection opens, never in the URL; roles are added with user management.

## Consequences

Sending the whole state keeps reconnecting, joining and lost messages trivial, at the cost of slightly larger messages; the state is a few dozen bytes. This replaces the "then increments" wording in `Architecture.md` section 4, which needs an explicit update. JSON makes traffic easy to inspect. Throttling and clock estimation live in the client core (`rxrust` operators). Message types and the ping interval are fixed in the implementation and covered by tests in `veltrunox-protocol`.
