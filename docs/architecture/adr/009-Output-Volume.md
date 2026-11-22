# ADR-009 – Output volume

**Status:** Accepted  
**Date:** 2026-11-22

## Context

The server plays for the whole room (ADR-003), so the loudness of the room is a decision of the room, not of a client. Everyone who may control playback must be able to change it and see the current value at once. A first listening test on the Raspberry Pi showed that the volume had to be set on the Pi itself. The Pi's headphone output has a hardware mixer; other devices (for example the default output of a development Mac) have none that the server can reach in a portable way.

## Decision

- **The volume is part of the shared playback state** (percent, 0–100) and changes through an intent like any other setting, so every client sees every change with the usual revision. It belongs to the room: it is not stored with a recording and survives selecting another one. It needs no selected recording.
- **System mixer first.** If the configured sound card has a mixer, the volume is applied to its playback volume control (ALSA on Linux; the card comes from the configured device name, the control is the first of Master, PCM, Headphone, Speaker, Digital that has a volume). Percent are spread evenly over at most 60 dB below the control's maximum, because a card with a range of 100 dB would otherwise be inaudible below about 60 %. A volume above 0 also switches the control on.
- **Engine gain as fallback.** Without a mixer (no card control, other platforms, a mixer that refuses the change) the render thread scales the audio by the squared fraction of the volume, ramped over one buffer so a change does not click.
- **Initial value.** At start-up the server reads the mixer and publishes that value, so the interface shows the real volume. The fallback starts at 100 %. The mixer keeps its own value across restarts; the fallback gain is not stored.
- Everyone who may control playback may change the volume. If roles restrict runtime changes later, the volume counts as one.

## Consequences

- The wire format gains a `volume_percent` field in the state and a `set_volume` message; the interface gets a slider that sends while it is dragged (throttled), so the room hears the change as it happens.
- A change made outside the server (for example with `alsamixer`) is not noticed until the next change through the server; the server does not watch the mixer.
- The mapping between percent and decibel is a convention of the server; the percentage shown does not match the percentage `alsamixer` shows.
- The server needs `libasound` at build time, as it already does for `cpal`.
