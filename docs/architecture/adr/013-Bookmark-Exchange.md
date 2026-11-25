# ADR-013 – Bookmark exchange and automation access

**Status:** Proposed  
**Date:** 2026-11-25

## Context

Labels and regions are created one by one in the interface and through the per-item endpoints (`/api/v1/recordings/{id}/labels`, `…/regions`, `/api/v1/labels/{id}`, `/api/v1/regions/{id}`). The owner wants to work out the bookmarks of many songs with an LLM, import them and also edit them afterwards. The library list (`GET /api/v1/recordings`) already carries title, length and analysis, but there is no way to move a whole set of bookmarks in or out, no overview over all recordings with their bookmarks, and no access that suits a script or an LLM (a login token expires, and a person's password does not belong in a script).

## Decision

**One rule set.** Import, export and the editor use the existing server operations and the domain validation of labels and regions. Nothing else writes bookmarks.

**Read.**
- `GET /api/v1/recordings` and the per-recording analysis stay the source for recording details. The bookmark count is added to each list entry.
- `GET /api/v1/export` returns every recording with its details (id, title, length, tempo, key, analysis sections where known) and its labels and regions in one JSON document. `GET /api/v1/recordings/{id}/export` returns the same for one recording. Both need no right beyond signing in.

**Write.**
- `PUT /api/v1/recordings/{id}/bookmarks` replaces all labels and regions of one recording in one transaction (`edit_bookmarks`). It validates every entry (range inside the length, start before end, non-empty name) and either applies all or none.
- `POST /api/v1/import` takes the export format for many recordings. Entries are matched by recording id; an id the server does not know is an error for that entry, a recording is never created. With `?dry_run=true` the server answers with what would change per recording (added, replaced, removed) and the errors per entry, and writes nothing. Without it, recordings are applied one transaction each and the answer lists the outcome per recording.
- Items carry the id and `version` they had at export time. On import, an entry whose recording changed since the export (version mismatch) is reported as a conflict and not applied unless `?overwrite=true` is given. Entries without ids are new.

**Format.**
- JSON is the exchange format, with a published schema (`format_version`, recordings, labels, regions). Times are numbers in milliseconds, as everywhere in the API.
- CSV is a convenience for spreadsheets: one flat table of bookmarks, columns `recording_id,type,name,start_ms,end_ms` (`end_ms` empty for a label). It has the same preview, validation and conflict rules and is converted to the JSON model on the server. The recording list with details is not offered as CSV.

**Access for scripts and LLMs.**
- A user can create named API tokens (`POST /api/v1/tokens`, list, revoke), stored like sessions (only the hash) but without the 30-day expiry and with a role limited to `read` or `read + edit_bookmarks`; they never carry upload, management or user administration. The token is shown once. A token's rights are the intersection of the user's rights and its limit.
- Tokens use `Authorization: Bearer`, so every endpoint above works unchanged.

**Interface.** The bookmark panel of a recording gets an editable table (the CRUD above), export and import buttons, and the import preview with its errors and conflicts. The design specification is extended on the owner's instruction before this is built.

**MCP, later.** A small separate MCP server calls the HTTP API with an API token. Tools: `list_recordings`, `get_recording`, `get_bookmarks`, `set_bookmarks`. It is not part of the server and not needed for the file-based path (export, edit with the LLM, import with preview).

## Alternatives considered

- CSV only: no types, no per-entry errors, no nesting for recording details; LLMs produce it less reliably than JSON.
- MCP inside the server: ties the server to a protocol that is still changing and adds a second route to the same rules.
- Reuse the login token for scripts: it expires and is bound to a password.
- Create missing recordings on import: would turn a bookmark import into an upload path with its own limits and rights.

## Consequences

- New endpoints, an import/export module in the server application, a token table (migration) and bridge calls; the existing per-item endpoints and the real-time notifications stay as they are, and a replace or import publishes the changed bookmarks like any other edit.
- An LLM can only place bookmarks as well as the data it sees: the export should carry the analysis (tempo, bars, sections) where present, and times for songs without analysis come from the owner's notes.
- API tokens are a new secret to protect; until ADR-008 is in use they travel in clear text like passwords.
- Architecture (server application, ports) and requirements are not changed by this ADR and stay as they are until the owner instructs it.
