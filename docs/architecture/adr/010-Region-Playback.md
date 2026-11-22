# ADR-010 – Region playback and play mode

**Status:** Accepted  
**Date:** 2026-11-22

## Context

The shared playback state holds a loop range (ADR-005). It is set by a separate intent and wraps the position at the sample level (ADR-006) while the position is inside it. The interface now offers three ways to play a region: to the end of the recording, to the end of the region, and looped. Playing to the end of a region has no counterpart in the state: nothing stops the audio at the region end. A region is also started by two intents today (set the range, then seek and play), which other clients' intents can interleave. The room must hear the stop exactly at the region end, not up to the 100 ms of the server's end-of-recording check.

## Decision

- **Play mode is part of the shared playback state:** `to_end` (default), `to_region_end`, `loop`. It belongs to the room, not to a recording; it is not stored. It starts as `to_end` after a server start and returns to `to_end` when another recording is selected. Everyone who may control playback may change it.
- **The loop range becomes the play range.** The state holds an optional play range (the region being played) and the play mode. Without a play range, the mode has no effect and playback runs to the end of the recording. Playing from anything but a region (a label, a free position, the play button) never sets a play range.
- **One intent starts a region:** `play_region(range)` sets the play range, moves the position to the range start and starts playback in one step, so concurrent intents cannot interleave. `set_play_mode(mode)` changes the mode, also during playback; the change applies from that moment on (the reference is rebased as for speed). The intent `set_loop` is replaced by these two.
- **Behaviour by mode while the position is inside the play range:**
  - `to_end`: the range is ignored; playback continues past the region end.
  - `to_region_end`: playback stops at the region end. The server pauses with the position at the range end and clears the play range. A new `play` then runs to the end of the recording; the region is played again with `play_region`.
  - `loop`: the position wraps from the range end to the range start, as the loop does today.
- **The play range ends with the position leaving it.** A `seek` outside the range, selecting another recording and pausing at the stop above clear it; a `seek` inside keeps it. Pausing by hand keeps it.
- **Exact stop in the engine.** The engine gets the stop position as a command, like the loop range, and ends playback at that sample with a short fade (a few milliseconds) so the cut does not click. The server computes the instant of the region end from the position reference and speed and publishes the paused state then, instead of polling. Clients clamp the extrapolated position at the range end, so the display never runs past it.
- **Validity.** `play_region` is rejected without a selected recording and for a range beyond the known length of the recording (as `seek` is).

## Consequences

- The wire format replaces `loop_range` with `play_range` and `play_mode` in the state and `set_loop` with `play_region` and `set_play_mode`. Server and web app ship together (the app is embedded in the server), so no compatibility layer is kept.
- The engine, the core transport functions and `position_at` change; the loop's sample-exact wrapping stays. The stop needs a test at the sample level (stop position, fade, behaviour at changed speed) and on the server (state published at the computed instant, two clients).
- The architecture text (playback state) and the requirements that name the loop have been updated to this model.
- A later region-specific tempo or a musical grid is not affected.
- A tap on a label could become a single intent as well (position and play); it stays with the existing seek and play for now.
