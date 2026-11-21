# ADR-003 – Server-side audio output

**Status:** Accepted  
**Date:** 2026-11-21

## Context

A band listens together through one sound system. Clients are phones, tablets or laptops that only control playback. Playing locally on every client would need audio streaming or loading, and would make synchronous playback and time stretching harder.

## Decision

The server decodes, time-stretches, pitch-shifts and plays the audio on its own audio device. Clients send intents and render the shared playback state; they do not output audio. Speed, tuning and key are stored with the recording.

## Consequences

Playback is inherently synchronous for everyone in the room. The client core and the browser build need no audio engine. The server must process audio in real time on its target hardware (including a Raspberry Pi); the audio thread must not allocate or block. Audio output, device selection and latency are measured on the target hardware. Listening remotely (a client without access to the room) is not supported by this decision; it would need a new ADR.
