# ADR-016 – Streaming playback from disk

**Status:** Proposed  
**Date:** 2026-11-27  
**Amends (once accepted):** ADR-006 (decoded audio held in memory), ADR-011 (source of scrub pieces), ADR-015 (memory of stems)

## Context

The engine holds the loaded recording and, with stems on, every stem completely decoded as stereo `f32`: 384 kB per second and track at 48 kHz. A recording of 4:28 with four stems comes to about 515 MB (calculated), more than the 462 MB of a Raspberry Pi 3 A+. ADR-015 requires this to be solved before stems are used there.

The engine runs in the audio callback, which must not block, allocate or read files. It does not only read forwards: a seek reads the audio before the target to prime the stretcher, a loop jumps from its end to its start, and scrubbing reads short pieces anywhere (ADR-011). All tracks must stay aligned frame by frame.

## Decision

**Unpacked cache.** Every track that is played (original or stem) is first unpacked into a file of raw interleaved stereo `i16` at its own sample rate, named by the content hash, in `<data_dir>/cache/`. Frame *n* is at byte *4n*, so any position is read without a decoder. The cache is derived data: it is not backed up and may be deleted while the server is stopped.

- Stems are unpacked when a package is imported or an analysis finishes; originals after upload or import. A missing file is made when the recording is loaded, which then takes as long as decoding it.
- Unpacking runs block by block on a worker thread, writes a temporary file and renames it.
- The cache has a configurable size limit. The least recently used recordings are removed first, never the loaded one. If the disk is full, the recording does not load and clients are told why.

**Blocks in memory.** The engine reads from a fixed pool of blocks (starting value: 8192 frames each), allocated when the recording is loaded. A reader thread fills them and hands them to the callback without locks; used blocks go back the same way. It keeps ready:

- the next 4 seconds from the playhead, following the loop;
- the start of the loop or play range, including the audio needed to prime the stretcher;
- the surroundings of the last requested seek or scrub position.

One block covers the same frames of all tracks, so stems cannot drift apart and the mix stays instant.

**When a block is missing** (after a jump, or if the card is slow), the engine outputs silence and does not advance until the block arrives. It never waits in the callback. The reported position still comes from played samples. A scrub piece whose block arrives after a newer position was requested is dropped. Missing blocks during steady playback are counted and logged.

**One path for all platforms.** Server and desktop application use the same code (ADR-014); there is no second in-memory mode.

## Estimates

Calculated, not measured.

| 4:28 recording, original and four stems | Today | Proposed |
|---|---|---|
| Audio in memory | about 515 MB | about 6 MB, independent of length |
| Disk in addition | none | about 257 MB |
| Read rate while playing | none | about 1 MB/s |

## Alternatives considered

- **`i16` in memory:** small change, but about 257 MB for the example, still too much.
- **Decoding while playing:** no extra disk space, but five decoders at once on the Pi, and seeking in compressed files must be exact to the sample in every track.
- **Memory-mapped cache files:** little code, but a page fault can stall the audio callback on a slow card.
- **A smaller format** (mono or lower rate): audibly worse.

## Consequences

- Memory no longer limits the length of recordings or the use of stems.
- `i16` replaces `f32` for playback; sources with more than 16 bits are reduced. Samples beyond full scale, which lossy decoders can produce, are clipped.
- Disk space becomes the limit, and the SD card is written once per unpacked track.
- A seek costs a file read before sound starts; the 500 ms bound of ADR-006 must be measured again.
- The engine needs a way to say "not available yet", and its tests need a source that delivers late.
- The player must pass loop, seek and scrub targets to the reader as well as to the engine.

## Verification before acceptance as done

On a Raspberry Pi 3 A+, with a recording of at least 5 minutes and four stems: peak memory; time to sound after a seek; gap-free loop; scrubbing by ear; no missing blocks over a full playback, also while an upload is analysed; time to unpack.

## Steps

1. Unpacked cache with size limit; no change to playback.
2. Engine reads through a block source; first implementation from memory, behaviour unchanged.
3. Reader thread and block pool; remove the in-memory audio.
4. Measurements on the Pi; adjust block size and read-ahead.
