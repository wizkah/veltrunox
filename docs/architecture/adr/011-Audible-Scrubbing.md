# ADR-011 – Audible scrubbing

**Status:** Accepted  
**Date:** 2026-11-22  
**Replaces:** the scrubbing paragraph of ADR-005 (the fallback "hold the sound while scrubbing")

## Context

ADR-005 left open whether the room hears the scrubbed position and named a fallback: hold the sound while the scrubbing flag is set. After a seek, the time stretcher (ADR-006) needs about 55 ms before it produces sound. With around 20 position intents per second, almost nothing was audible. The listening test on the target hardware (Raspberry Pi) confirmed this and showed short pieces of the plain recording to work well.

## Decision

- **Scrubbing is audible.** For every new position of a drag the engine plays a short piece of the recording (120 ms) taken directly from the decoded audio, without the stretcher, with short fades in and out. Consecutive pieces overlap without a gap.
- **The pieces ignore speed, tuning and key.** Scrubbing is for finding a place, not for judging the result.
- **The shared position holds while dragging.** All clients follow the scrubbed position; the position reference stays under the handle during the drag and runs on from the release position when the handle is let go.
- **Holding still plays on.** If the handle rests for a moment (130 ms), a recording that was playing continues from there; a paused recording also plays while the handle is held and stops when it is let go.
- **The protocol does not change.** The intents (`seek` with `scrubbing: true`, throttled to about 20 per second, and a final `seek` with `scrubbing: false`) stay as ADR-005 describes them.

## Consequences

- The engine gets a command for scrub pieces; the player derives it from the shared state (position changes while the flag is set).
- The fallback of ADR-005 no longer applies; ADR-005 points to this ADR.
- The sound of scrubbing is not the sound of playback; a future change (for example pieces through the stretcher) would need a new measurement on the Pi.
