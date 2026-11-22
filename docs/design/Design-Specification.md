# Veltrunox – Design Specification v0.1

**Status:** working draft for the first UI implementation
**Audience:** developers and AI coding agents
**Platform:** responsive browser client; a later native app is not anticipated
**UI language:** German at first; strings must not be hard-wired (see O-07)
**Companion file:** [`design-tokens.json`](design-tokens.json)

> **How binding is this?** Every rule carries one marker. **[B] confirmed** (product requirement or confirmed design decision), **[V] proposal v0.1** (concrete, testable starting values to be validated on a device), **[O] open** (must not be decided implicitly by an implementation). Where this text and a mockup disagree, this text wins.

## 1. Purpose, boundaries, goals

Veltrunox is a shared remote control for band rehearsals. One playback instance on the server drives the sound system of the rehearsal room. Browser clients **do not play audio themselves** [B]; they show the shared state and change it according to their rights [B].

### 1.1 UX goals [B]

1. A marked position or region plays with **at most two actions**, ideally one.
2. A first-time user finds the way to play a region without instructions.
3. Touch operation works in a loud rehearsal, often one-handed and in dim light.
4. All clients see what is playing and can follow changes made by others.
5. Allowed actions are clearly visible; **actions a user may not perform are hidden**, not merely greyed out.
6. Loading, offline, error and analysis progress have distinct, understandable states.

### 1.2 Out of scope [B]

Server architecture, audio processing, analysis algorithms, real authorization, backup, network latency, audio streaming to clients, cutting/mixing (no DAW features). This document defines UI expectations, not their technical implementation.

### 1.3 Scope of v0.1 [B]

- The library is a **large song list** (recordings). There are **no setlists** and no separate "Songs" screen.
- There is **no admin area** in the navigation for now. Upload and management actions appear where they apply (library, recording details) when the user has the right.

## 2. Principles

| ID | Rule | Status |
|---|---|---|
| P-01 | Waveform and transport are the visual centre of the player. | [B] |
| P-02 | The play button is immediately recognizable as the primary action. | [B] |
| P-03 | Label and region are different marker types. | [B] |
| P-04 | The waveform is never covered by full-area region overlays. | [B] |
| P-05 | Overlapping regions are shown in separate lanes. | [B] |
| P-06 | Non-overlapping regions may share a lane. | [B] |
| P-07 | Only the region selected for editing shows large end handles. | [B] |
| P-08 | No interrupting dialogs for ordinary playback actions. | [B] |
| P-09 | All essential functions work without hover and with touch. | [B] |
| P-10 | Local view control (zoom, scroll) and shared playback are separate concepts. | [V] |
| P-11 | State changes are conveyed by text or icon **and** colour. | [V] |
| P-12 | Deleting asks for confirmation; ordinary playback does not. | [B] |

### 2.1 Terminology [B]

- **Recording:** a server-side audio file; appears in the library.
- **Playhead:** the current global playback position.
- **Label:** a point marker with a single time position and an editable name. **No end value.**
- **Region:** a named time range with start and end.
- **Lane:** a display row for regions, outside the waveform.
- **Marker:** umbrella term for label or region.
- **Original values:** automatically analysed tuning, key and tempo; they stay visible as a reference where known.
- **Play mode:** what happens at the end of a region (section 5.3).

## 3. Foundations

### 3.1 Colour

Dark, logo-inspired: very dark background, teal/cyan for data and secondary emphasis, warm orange for primary interaction [B]. Exact hex values are [V] and live only in `design-tokens.json`.

| Semantics | Token | Start value [V] | Use |
|---|---|---|---|
| Background | `color.canvas` | `#071519` | page background |
| Surface | `color.surface` | `#102A30` | cards and panels |
| Raised surface | `color.surfaceRaised` | `#16353B` | menus, dialogs, popovers |
| Primary action | `color.action` | `#FF9D16` | play, primary actions, selected region |
| Waveform/accent | `color.accent` | `#14C5C9` | waveform, focus context, secondary information |
| Soft accent | `color.accentSoft` | `#65E2DC` | labels, detail markers |
| Text | `color.textPrimary` | `#F5F8F8` | body text and headings |
| Secondary text | `color.textSecondary` | `#B9CED1` | metadata and explanations |
| Success | `color.success` | `#34D399` | connected, positive feedback |
| Warning | `color.warning` | `#FDBA74` | hints and limit warnings |
| Error | `color.error` | `#FB7185` | errors, destructive feedback |
| On action | `color.onAction` | `#071519` | text/icons **on** orange or cyan fills |

Rules:

- [V] Dark text and icons on fully orange or cyan buttons; no white text on cyan.
- [V] Selected region = orange; other regions = muted teal/blue tones. Colour is **not** a permanent region type.
- [V] Control state (on/off/inactive) is also conveyed by shape, icon and text.
- [V] Target **WCAG 2.2 AA** for text, focus and interactions; verify contrast in a real browser, not only on the palette.

### 3.2 Typography and icons [V]

| Element | Start value |
|---|---|
| Typeface | `Inter` (bundled with the app), then system-UI fallbacks |
| Body | 16 / 24 px, weight 400 |
| Secondary | 14 / 20 px, weight 400–500 |
| Section title | 20 / 28 px, weight 600 |
| Screen title | 24 / 32 px, weight 700 |
| Numeric transport values | 16–20 px, tabular figures |
| Icons | one plain outline set, mostly 20–24 px |

No essential label below 14 px in normal mobile use. Time values, labels and metadata stay usable at 200 % zoom.

### 3.3 Space, geometry, targets [V]

| Token or rule | Start value |
|---|---|
| Spacing grid | base 4 px; steps 4 / 8 / 12 / 16 / 24 / 32 |
| Radius small / medium / large | 8 / 12 / 16 px |
| Panel border | 1 px, low contrast but visible |
| Minimum touch target | 48 × 48 px for core controls |
| Primary play button | 64 × 64 px or larger |
| Region handle, visual | about 10–14 px wide; hit area at least 48 px |
| Status/overlay transitions | 120–180 ms, functional and subtle |
| Reduced motion | respect `prefers-reduced-motion` |

Hit area and visible handle are separate. Large invisible hit areas must not uncontrollably cover each other near closely spaced markers; use selection/precision mode then.

### 3.4 Responsive layout [V]

| Viewport | Width | Layout |
|---|---|---|
| Mobile | 0–599 px | One column: song, waveform, transport, "set label here" first; marker list as its own view or bottom sheet; tempo/pitch/volume condensed |
| Tablet | 600–1023 px | More room for the waveform; collapsible side list; keep large touch targets |
| Desktop | from 1024 px | Waveform/transport in the centre, markers on the right, tempo/pitch/volume below; library as list plus detail panel |

Rebuild the layout by content; do not merely scale desktop elements down. No horizontal page scrollbar for the main view; horizontal scrolling inside the time axis is fine.

## 4. Information architecture

### 4.1 Main paths [B]

```text
Login / first start
  ├── Sign in
  ├── Guest access (only when enabled)
  └── Set up first admin (initial state only)
        ↓
Library (song list)  ⇄  Player
    │                     └── Markers (labels + regions)
    └── Recording details (upload and management where permitted)

Administration of users and guest access: later phase, not in the v0.1 navigation
```

### 4.2 Navigation [V]

- Desktop: compact top navigation **Player / Library**; connection status always visible.
- Mobile: bottom navigation **Player / Library**, with enough clearance from the device edge.
- The library is the way to find a recording; a direct play entry jumps into the player in one step and plays, once the recording's state allows it.

### 4.3 Rights as UI capabilities [B]

| Action | User | Admin | Guest |
|---|---|---|---|
| Play / navigate | yes | yes | yes |
| Create and edit labels / regions | yes | yes | yes |
| Tempo / pitch / volume | yes | yes | yes |
| Upload a recording | yes | yes | yes |
| Manage recordings (rename, metadata, delete) | no | yes | no |
| Manage users, switch guest access | no | yes | no |

A guest has the rights of a user without signing in and exists only while the admin has enabled guest access. The row "manage recordings" is [V] (the product rules do not name it). The UI receives **effective capabilities**; disallowed actions do not appear. Real authorization is outside this document.

## 5. Player

### 5.1 Desktop structure [B for functions, V for arrangement]

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Logo   Player  Library                       ● Connected   4 clients │
├──────────────────────────────────────────────────────────────────────┤
│ Recording title + version / metadata        [subtle remote notice]   │
├──────────────────────────────────────────┬───────────────────────────┤
│ Time ruler + label row                   │ Markers                   │
│ ┌──────────────────────────────────────┐ │ ▶ Entry    00:25          │
│ │ Waveform / playhead / zoom / scroll  │ │ ▶ Verse    00:10–01:07    │
│ └──────────────────────────────────────┘ │ ▶ Chorus   00:40–01:32    │
│ Region lanes (overlaps stacked)          │ ▶ Break    01:25          │
│                                          │ [Filter: All/Labels/Reg.] │
├──────────────────────────────────────────┴───────────────────────────┤
│ [Back] [▶ Play] [Pause] [■ Stop] [Play mode] [Set label here]        │
├──────────────────────────────────────────────────────────────────────┤
│ Tempo 0.5× … 2.0× | target bpm · Tuning Hz | Key · Original · Volume │
└──────────────────────────────────────────────────────────────────────┘
```

The client count is shown at most as a number until O-01 is decided.

### 5.2 Mobile structure [V]

```text
┌──────────────────────────────┐
│ Logo             ● Connected │
│ Recording ▾        01:00/03:00│
│ [Remote notice, only if needed]│
│ Time ruler / label chips     │
│ Waveform + playhead          │
│ 1–2 region lanes     [+ more]│
│ [− Zoom] [+ Zoom] [Snap]     │
│ [Back][▶][Pause][Stop][Mode] │
│ [+ Set label here]           │
│ [Open markers]               │
│ Tempo 1.00× · Tuning 440 Hz  │
│ Volume                       │
│ Player  Library              │
└──────────────────────────────┘
```

With many regions, small displays show **at most two lanes by default**; "All regions" expands the view [B]. Other settings may sit below the immediate playback area [B].

### 5.3 Transport and direct actions

| Interaction | Behaviour | Status |
|---|---|---|
| Play / Pause / Stop | Large, clearly stateful buttons; synchronized across clients | [B] |
| Play mode | A three-state toggle next to the play button (below) | [B] |
| Tap a label in the waveform | **Immediately** jump there and play | [B] |
| Tap a region name / play icon | **Immediately** play the region from its start | [B] |
| Play in the marker list | The same action for both marker types | [B] |
| "Set label here" | **One action** creates a label at the playhead; the name can be edited later | [B] |
| Create a region | Two ways: choose the "Region" button, then drag a time range in the waveform (**two steps**); or open the region form (name, start, end) and confirm | [B] |
| Long-press label/region | Context actions (rename, edit, delete) according to rights | [B] |
| Open a recording from the library | Visible in the player; started directly on an explicit play | [V] |

**Play mode [B]** has three states and is part of the shared playback state:

| State | Behaviour |
|---|---|
| Play to end | Playback continues to the end of the recording, also after a region end |
| Play to region end | Playback of a region stops at the region end |
| Loop | A region plays repeatedly between its start and end |

The icon, a text label and the state colour must show the active state; tapping cycles through the states. The mode always starts as "play to end" (also after another recording is selected) and only has an effect when a region is played. Playing from a label, a free position or the play button always runs to the end of the recording. After "play to region end" has stopped at a region end, a new play runs on to the end of the recording; the region is played again by tapping it. The server side is described in ADR-010.

### 5.4 Tempo [B]

- Setting **0.5× to 2.0×**; initial value [V] 1.00×.
- Shown as a factor with an easily readable value, plus an optional target bpm.
- A target bpm derives a factor only when an analysed original bpm is available: `factor = target bpm / original bpm` [V].
- If the value falls outside `[0.5, 2.0]`, show an **explicit notice**. [V] In v0.1 an invalid target is not adopted; the entry stays visible for correction.
- Without a known original bpm, do not pretend a valid bpm conversion; the factor control stays standalone.

### 5.5 Pitch [B]

- Show and change the current **tuning in Hz** and the **key** separately.
- Show the analysed original values right away as a reference (for example "Original A = 440.0 Hz", "Original E major").
- If original values are missing, show "Not analysed yet" or "Unknown" instead of made-up values [V].
- The separate fields must not imply that tuning (Hz) and key are the same thing.

### 5.6 Volume [B]

A volume control belongs to the shared playback state. Every client shows the current value and any client may change it. It sits with the other playback settings, not inside the transport row.

## 6. Waveform – component contract

This section is normative for the implementation. The player is not a DAW; the waveform serves navigation and marker work.

### 6.1 Layers, top to bottom [B]

1. **Timeline/ruler:** time values and position.
2. **Label layer:** point markers **above** the waveform, with marker line and name chip. No end handle.
3. **Waveform layer:** amplitudes stay fully visible; no opaque region areas.
4. **Playhead layer:** global playback position, drawn through all relevant layers.
5. **Region lanes:** separate stacked rows below the waveform; each region with start, end and name.
6. **Interaction overlay:** selection handles, precise drag feedback, context menu and optional magnifier, only when needed.

The drawing order must allow collision avoidance: the active playhead is always recognizable; label chips must not permanently hide the ruler [V].

### 6.2 Semantic UI data model [B for the type split, V for fields]

```ts
// View-model contract only, not a backend or persistence rule.
type LabelMarker = {
  kind: 'label';
  id: string;
  title: string;
  positionMs: number;       // >= 0, within the recording
};

type RegionMarker = {
  kind: 'region';
  id: string;
  title: string;
  startMs: number;
  endMs: number;            // > startMs, within the recording
};

type Marker = LabelMarker | RegionMarker;
```

Empty titles at creation are allowed, but the display needs a sensible fallback name [V], for example "Label 3". A label has **no end time**, neither stored nor implied [B].

### 6.3 Overlapping regions and lanes [B]

- Two intervals that overlap in time never share a lane.
- Non-overlapping regions may share a lane.
- Lane height [V]: 36 px plus spacing; not to be read as the hit-area size.
- Keep the lane layout **as stable as possible during a drag**. If a change requires other assignments, reorder only **after** the drag ends.
- Highlight the selected region in orange; other regions in distinguishable, high-contrast secondary tones.
- Only the selected region gets larger end handles. Dragging a handle must not put a covering block over the waveform.
- Region titles are real actions: tapping starts playback. Editing uses a long press or an explicit edit action in the list [B].
- No fixed limit on the number of regions in the data. Visible lanes may be reduced on mobile [B].

**Layout algorithm [V]:** interval partitioning per recording with stable assignment and the selected region always visible. Touching at a shared end point is **not** an overlap, so intervals are `[start, end)`. The output is deterministic for the same input.

### 6.4 Labels (point in time only) [B]

- A label is a slim marker at `positionMs`, not a range.
- A short text chip carries the name, for example "Entry" or "Break".
- When chip texts collide, group or condense them, for example "+3". Never draw them illegibly on top of each other.
- Tapping a label plays immediately from its position.
- For a group: tapping opens a compact list; tapping the wanted label then plays (**two steps**) [V].
- A long press opens the permitted actions. Renaming changes the title, not the position.

### 6.5 Gestures: priority and collisions

| Context / gesture | Result | Status |
|---|---|---|
| Single tap on free waveform | Set the playhead target position **without starting playback** | [B] |
| Drag **directly on the playhead** | Scrubbing: move the playhead horizontally | [B] |
| Drag in free waveform area | Pan the zoomed section horizontally | [B] |
| Two-finger pinch | Zoom the time axis | [B] |
| Tap a label chip | Play from the label | [B] |
| Tap a region name | Play from the region start | [B] |
| Tap "Region", then drag over the waveform | Create a region; visible start/end preview | [B] |
| Drag a selected region's handle | Adjust start or end | [B] |
| Long press on label/region | Show context actions | [B] |
| Tap a `+N` marker group | Open the list of contained labels | [V] |
| Snap switch | Snap to markers on/off | [B] |
| `+` / `−` zoom buttons | Alternative to pinch, usable with mouse and keyboard | [V] |

**Gesture conflicts [V]:**

1. The active "create region" mode takes precedence for drags on free waveform; afterwards return to normal navigation.
2. A hit on a region handle beats a hit on the region body; a region body without a title must not be confused with scrubbing.
3. A hit on the playhead beats a free background drag.
4. A two-finger pinch beats a one-finger drag as soon as it is clearly recognized.
5. No hidden double-tap gestures for basic functions.
6. If a handle is hard to hit because of close spacing, offer an enlarged drag preview or precision magnifier [B as an option]; never switch the edited marker at random.

### 6.6 Snap and zoom [B/V]

- Snap to marker positions is **optional** and visibly switchable [B].
- [V] Snap first to labels, region starts and region ends; a musical beat grid is out of scope (O-02).
- [V] Snap is temporarily visible while dragging (line plus name/times) and never triggers a dialog.
- [V] The pinch zoom centre lies between the fingers; with `+`/`−` it lies at the visible playhead if in view, otherwise at the viewport centre.
- [V] "Fit whole recording" is its own clearly distinct action.
- [V] Scrolling and zooming change **only the local viewport**, not the shared playback position.

### 6.6a Region form [B]

The region form creates a region from a name, a start and an end time, and edits the same values of an existing region. It is reachable from the transport/marker area and from the context actions of a region, without a long press. Start and end default to the playhead and the end of the visible range [V]; invalid input (end not after start, outside the recording) shows a message at the field and is not adopted. A region created this way is selected afterwards, like one created by dragging.

### 6.7 Marker list [B]

- One shared list for **labels and regions**, sorted chronologically by start or point time.
- Each row shows type icon, title, time position or start–end, and a directly reachable play action.
- Edit/rename/delete are visible only with the matching capabilities.
- [V] Filter "All | Labels | Regions"; ties in the chronological order are broken stably by type and id.
- Opening a region from the list makes it visible in the waveform without changing the global position merely by highlighting it [V].

## 7. Shared state and feedback

### 7.1 Shared versus local state [V]

| Shared / authoritative for all clients | Local per client |
|---|---|
| Selected recording | Waveform zoom and visible time range |
| Playback state (play/pause/stop) | Open panel / bottom sheet |
| Playhead / playback position | Library search and filter input |
| Play mode | Hover, local focus, scroll position |
| Tempo, pitch and volume settings | Current selection / drag preview of a marker |
| Stored labels and regions | Provisional text editing until it is committed |

When another person changes the global position, the visible playback display follows the shared state [B]. A local zoom is not reset by this [V].

### 7.2 Showing remote actions [B]

- Short and non-blocking: "Ben moved the position to 01:12".
- No modal confirmation and no lasting disturbance.
- The playhead and the transport state update visibly.
- [V] If the actor is unknown: "Playback position was changed" rather than an invented name.
- [V] During a local drag, remote updates must not make the handle jump incomprehensibly; the rule for simultaneous editing stays open (O-05).

### 7.3 Connection status [B]

| State | Display | Action |
|---|---|---|
| Connected | Subtle signal / "Connected" | none |
| Connecting | "Connecting …" / activity indicator | no pretended live state |
| Reconnecting | "Restoring connection …" | visible, non-blocking where possible |
| Disconnected | Conspicuous but calm notice "Connection to the server lost" | "Reconnect" |
| After reconnect | Align the display with the current shared state | never keep showing a stale position silently |

[O] How actions taken locally during a disconnection are treated is not a v0.1 UI decision (O-08).

## 8. Library

### 8.1 Content [B]

The library is a large, searchable song list. Each recording shows at least its **title**, available metadata and analysed values (**tuning, key, bpm**) where known, and an understandable analysis state. Searching and sorting are central tasks. A recording must be quickly reachable in the player.

### 8.2 Desktop layout [V]

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Logo  Player  Library                              ● Connected       │
│ [Search: title, metadata …] [Sort ▾] [+ Upload]*                     │
│ [All] [Analysed] [Analysing] [Key ▾] [Tempo ▾]                       │
├──────────────────────────────────────────┬───────────────────────────┤
│ Recording       Length  Hz  Key  bpm     │ Selected recording        │
│ ▶ Back to …     03:00  440  E maj 120    │ Waveform thumbnail        │
│ ▶ Tight Verse   01:45  440  D maj 124    │ Details / tags / version  │
│ ◌ New Groove    —     Analysis: 64 %     │ [Open in player]          │
│                                          │ [Edit]** [Delete]**       │
└──────────────────────────────────────────┴───────────────────────────┘
* Only where uploading is permitted.   ** Only where managing is permitted.
```

### 8.3 Mobile layout [V]

- Search field at the top; sort and filter as reachable touch targets.
- Recordings as readable rows/cards with title, length, key and bpm.
- The primary play action without a long sub-menu.
- Analysis progress directly on the affected recording.
- A detail view instead of the permanent desktop side panel.
- No table columns that force tiny text.

### 8.4 Library actions

| Action | Expectation | Status |
|---|---|---|
| Search | Directly findable, clear search text | [B] |
| Sort | Understandable current sort order | [B] |
| Filter by analysis, key, tempo | UI proposal for faster searching | [V] |
| Open a recording | Switch to the player; clearly distinct from "play now" | [B/V] |
| Running analysis | Progress/status per recording, not only a global spinner | [B] |
| Failed analysis | Understandable error message, no invented bpm/key values | [V] |
| Upload | Progress and errors shown | [B] |
| Metadata / rename | Only where permitted (section 4.3) | [B] |
| Delete | Confirm; name the recording being deleted | [B] |

[O-06] Whether a recording is playable while its analysis runs is up to the application. The UI always shows the analysis state and does not invent playability.

## 9. Sign-in and administration

### 9.1 Sign-in / first start [B]

- Normal login with **user name and password**.
- Show guest access only when the admin has enabled it.
- On an instance that is not set up yet, show a separate "Set up first admin" flow, not mixed with the normal login.
- Errors short and understandable; never show passwords in clear text permanently.

### 9.2 Upload [B]

- Visible progress per upload, file name and an unambiguous status.
- An understandable error message and, where sensible, a retry action.
- The analysis step is its own state; "upload succeeded" does not mean "analysis finished".

### 9.3 Managing recordings [B]

Rename, edit metadata, delete with confirmation. The confirmation names the **recording's title** and offers a clearly destructive action.

### 9.4 User administration [B, later phase]

Create users, change role, change password, deactivate, delete; switch guest access on and off. Visible to the admin only. [V] Critical changes get a confirmation or a clearly visible success/error message; ordinary field changes need no extra confirmation. Not part of the v0.1 navigation.

## 10. System states and errors

Common vocabulary in the player and the library [B]:

| State | Element | Example text | Expected behaviour |
|---|---|---|---|
| Empty | icon + explanation + next action | "No recording selected yet." | "Select a recording" |
| Loading | progress or spinner | "Loading recording …" | no unclear, seemingly usable waveform |
| Error | error icon + text + action | "The recording could not be loaded." | "Try again" |
| Disconnected | offline status + action | "Connection to the server lost." | "Reconnect" |
| Analysing | recording-specific status + progress | "Analysing (64 %)" | do not pretend analysis data yet |

Rules [V]:

- Show real percentages only when the UI receives a progress value; otherwise show indeterminate progress.
- An error text says **what** does not work and, where sensible, **what is possible next**. No bare error codes.
- No full-screen error page for a single faulty recording while the library as a whole stays usable.
- Status icons are not conveyed by colour alone; add meaningful ARIA status messages.

## 11. Component catalogue

| Component | Variants / states | Responsibility |
|---|---|---|
| `AppShell` | mobile/tablet/desktop | navigation, screen frame, global status area |
| `BrandMark` | compact/full | logo without distorted proportions |
| `ActionButton` | primary/secondary/danger; default/pressed/disabled/loading | simple triggering, focus/touch |
| `IconButton` | default/selected/disabled | clearly labelled icon button, accessible name |
| `TextField` | normal/focus/error/readonly | input and error messages |
| `SelectField` | normal/focus/disabled | options, clear current choice |
| `NumericControl` | normal/invalid/disabled | factor/bpm/Hz and limit errors |
| `SliderControl` | normal/dragging/disabled | tempo, volume; no too-small handles |
| `ConnectionStatus` | connected/connecting/reconnecting/disconnected | live connection display |
| `ActivityStatus` | idle/loading/progress/success/error | upload and analysis progress |
| `RecordingListItem` | normal/selected/analysing/unavailable | title, values, direct play action |
| `RecordingDetails` | read/edit | data and management actions |
| `TransportControls` | stopped/playing/paused | playback control |
| `PlayModeToggle` | to end / to region end / loop | three-state play mode |
| `WaveformViewport` | default/panning/zooming | time-axis view and zoom |
| `WaveformRenderer` | ready/loading/empty | amplitudes, no input logic |
| `TimelineRuler` | various zoom levels | time labels |
| `Playhead` | idle/dragging/remote-updated | position and scrubbing |
| `LabelLayer` | normal/clustered/selected | point markers and collisions |
| `RegionLaneLayout` | normal/editing/compact | overlap-free lane arrangement |
| `RegionItem` | normal/selected/dragging | region title and interaction |
| `RegionHandle` | normal/dragging/precision | start/end handles |
| `MarkerList` | all/labels/regions | shared chronological list |
| `ContextActions` | label/region, permissions | context actions |
| `ConfirmDialog` | destructive | explicit confirmation of deletion |
| `ToastNotice` | info/success/warning/error | non-blocking feedback |

### 11.1 Component contract form [V]

Each implemented component keeps a short contract:

```yaml
name: RegionItem
purpose: Show a named region in exactly one lane and make it directly playable
inputs: [region, selected, effectiveCapabilities, displayGeometry]
outputs: [onPlay(regionId), onSelectForEdit(regionId), onRequestContextMenu(regionId)]
visualStates: [default, selected, dragging, disabledByAvailability]
interaction: 'Tap title = play; long press = actions; end handles only when selected'
accessibility: 'Name, start/end, action and selection state are readable'
```

A contract describes behaviour and presentation only, not server or audio implementation.

## 12. GUI architecture

The structure and quality rules in [`GUI-Architecture.md`](../development/GUI-Architecture.md) apply. In addition [V]:

- Waveform and lane renderers work on **pure, deterministic data** and can be tested in isolation with test data.
- A presentation component does not know a concrete transport or persistence library.
- Shared and local state are separated explicitly (section 7.1).

Data flow:

```text
User action
  → feature intent (e.g. PlayMarker, MovePlayhead, ResizeRegion)
  → controller / use case (not part of the visual component)
  → new shared state or feedback
  → UI projection / view model
  → pure components

Local action (e.g. zoom)
  → local viewport state
  → waveform rendering
  (no change to shared playback)
```

## 13. Accessibility and practical usability [V]

1. Every primary action works by touch **and** keyboard; context actions are not available through a long press only.
2. No central operation requires a pinch; `+`/`−` and "fit view" complement it.
3. Visible keyboard focus on all interactive elements; a sensible focus order.
4. Screen reader: "Entry, 25 seconds, play"; region "Chorus, 40 seconds to 1 minute 32 seconds, play"; native semantics where possible.
5. Time and progress values are not shown as bars only; text is available.
6. Drag alternatives: a region can be created, and start/end of a selected region changed, in the region form with editable time fields.
7. Errors and offline messages use suitable status/alert semantics without stealing focus repeatedly.
8. Stay functional with `prefers-reduced-motion` and enlarged system fonts.
9. With a very dense waveform, avoid uncontrolled hit-area overlap: offer a selection/precision view.

**Touch test:** operation with a thumb, without hover, on a narrow portrait smartphone; test overlapping lanes and densely placed labels explicitly.

## 14. Acceptance criteria for v0.1

Scenarios for UI acceptance tests; test data suffices, no real audio or server integration is needed.

| ID | Test | Expected result |
|---|---|---|
| UI-AC-01 | Tap a label once | Play intent triggered for exactly that position |
| UI-AC-02 | Tap a region name once | Play intent from the correct region start |
| UI-AC-03 | Show three overlapping regions | No overlap inside a lane; titles and end points recognizable |
| UI-AC-04 | Enlarge a region by its handle | Only the active region has large handles; no lane jump mid-drag |
| UI-AC-05 | Tap "Set label here" | Label created exactly at the current position; editable later |
| UI-AC-06 | Tap free waveform | Playhead target set, **no** unwanted playback |
| UI-AC-07 | One-finger drag, free vs. on the playhead | Free: pan; playhead: scrub; clearly distinct |
| UI-AC-08 | Choose "Region", drag over the waveform | New interval created in at most two steps |
| UI-AC-21 | Create a region in the region form | Name, start and end are validated (end after start, within the recording); the region appears in its lane and in the marker list |
| UI-AC-09 | More than two region lanes on a narrow phone | Two lanes visible by default; "All regions" reachable |
| UI-AC-10 | Many nearby labels | Labels not illegibly overlapping; groups playable in at most two taps |
| UI-AC-11 | Simulate an external position change | Playhead updates; non-blocking notice about actor/position |
| UI-AC-12 | Cut the connection | Clear status and "Reconnect"; no misleading live display |
| UI-AC-13 | Analysis running for a recording | Unambiguous progress/status on that recording; missing values not invented |
| UI-AC-14 | User without management rights | No visible delete or management actions |
| UI-AC-15 | Enter a target bpm outside the 0.5×–2× factor | Visible limit notice, no silent adoption |
| UI-AC-16 | Operate primary functions by keyboard | Play, pause, marker selection, zoom alternatives reachable |
| UI-AC-17 | Switch responsively from desktop to phone | Waveform and transport keep priority; no shrunken desktop table |
| UI-AC-18 | Delete a recording (admin) | Confirmation with the concrete title and a destructive action |
| UI-AC-19 | Cycle the play mode and play a region | "To end" continues after the region end; "to region end" stops there; "loop" repeats |
| UI-AC-20 | Change the volume on one client | All clients show the new value without reloading |

These are target criteria, not statements about existing code.

## 15. Decisions and open points

### 15.1 Confirmed design decisions [B]

- **D-01:** dark, logo-inspired surface with teal/cyan and orange.
- **D-02:** touch-first, high contrast, large targets, responsive layouts.
- **D-03:** labels = points in time; regions = time intervals.
- **D-04:** label chips above a freely visible waveform.
- **D-05:** overlapping regions in separate lanes; non-overlapping ones may share a lane.
- **D-06:** only the active region has large handles; lanes stay stable during a drag.
- **D-07:** a tap on a label or region title starts playback; a long press opens permitted actions.
- **D-08:** a label at the current position in one action; a region in two steps.
- **D-09:** on mobile at most two region lanes at first; the rest expandable; dense labels grouped.
- **D-10:** one shared chronological marker list.
- **D-11:** server-based shared playback and unobtrusive display of others' position changes.
- **D-12:** missing rights mean hidden actions.
- **D-13:** the library is a large song list; no setlists; no admin area in the v0.1 navigation.
- **D-14:** rights follow the product's role model (section 4.3).
- **D-15:** play mode is a three-state toggle next to the play button: play to end (always the initial state), play to region end, loop; it only affects region playback.

### 15.2 Proposals to validate [V]

- **V-01:** hex values, type scale, spacing and breakpoints (section 3).
- **V-02:** mobile/desktop navigation patterns and library filters.
- **V-03:** marker clusters `+N` with a selection list and the fixed two-step rule.
- **V-04:** precise gesture priority, half-open intervals `[start,end)` for the lane layout, local viewport.
- **V-05:** component model, interaction intents and module boundaries.
- **V-06:** accessibility and UI test criteria (sections 13 and 14).

### 15.3 Open decisions [O]

| ID | Question | Meaning for the UI |
|---|---|---|
| O-01 | Show connected clients by name? | Placement and level of detail of the client display |
| O-02 | Musical grid (bars/beats)? | Extra grid layer and snap modes |
| O-04 | Separate bpm per region? | Extra region parameters and editing UI |
| O-05 | Simultaneous editing of one marker: conflict display? | Feedback, versioning, locking or merging |
| O-06 | Playback while analysis is still running? | Enabled/unavailable state of recording actions |
| O-07 | Multiple languages / language switch? | String catalogue, space requirements, locale behaviour |
| O-08 | Exact semantics of unconfirmed input while offline? | UI feedback, pending actions, error handling |
| O-09 | Stored or temporary zoom/view preferences? | Locality and lifetime of the viewport |
| O-10 | Default name and name editing right after creating a label? | One-tap flow and immediate editability |

Open points are not blockers for an isolated, interactive design-system prototype, but they must not be hidden as undocumented assumptions.

## 16. Implementation order

Two isolated layers, first without server and audio:

1. Foundations from `design-tokens.json` and the visual base components (button, slider, input, status, modal) with all states.
2. `WaveformViewport` and `RegionLaneLayout` with deterministic test data: three overlapping regions, two labels, one marker group, region creation, handles, pan/zoom/scrub, direct play intent.
3. Compose the library and the player from the same components.
4. Responsive, contrast and touch tests against UI-AC-01 to UI-AC-21.
5. Carry validated numbers and behaviour into the next version; close open points with an explicit decision.
