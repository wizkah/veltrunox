# ADR-012 – Authentication, roles and upload

**Status:** Accepted  
**Date:** 2026-11-22

## Context

Until now every client may do everything. The server needs accounts, roles (admin, user, optional guest), a rights check for every action, and a way to add recordings. The ports `UserRepository`, `PasswordHasher` and `SessionTokens` are named in the architecture but not yet implemented. ADR-005 reserves the first client message of the real-time channel for authentication data. ADR-008 covers transport encryption separately; until it is in use, passwords travel in clear text and the server belongs in a trusted network only.

## Decision

**Accounts and passwords.**
- Users live in SQLite (migration): id, unique user name (case-insensitive), role, password hash, active flag, creation time.
- Passwords are hashed with Argon2id (`argon2` crate, parameters in the configuration, defaults suited to a Raspberry Pi). Verification and hashing run off the async runtime threads. Minimum length 10; no other composition rules.
- First start: while no user exists, `POST /api/v1/setup` creates the first admin. It is refused as soon as one user exists (checked in one transaction). There is no default password.

**Sessions.**
- `POST /api/v1/login` returns an opaque random token (32 bytes, hex). The database stores only its SHA-256 hash with user id, creation and last-use time. Tokens expire after 30 days without use; logout, deactivating or deleting a user, and a password change remove them.
- HTTP requests send `Authorization: Bearer <token>`. The real-time channel takes `{type: "auth", token}` as first client message (never in the URL); without a valid message within a few seconds the server closes. A client that wants guest access sends `{type: "auth", guest: true}`.
- Repeated failed logins for one user name or address are delayed (growing wait, capped); the response never says whether the name exists.

**Roles and rights.**
- The domain defines roles and capabilities; one function maps role to capabilities: `play_control` (position, run/pause, speed, tuning, key, volume, play range and mode), `edit_bookmarks`, `upload`, `manage_recordings` (rename, metadata, delete, re-analyse), `manage_users` (users and guest switch).
- User and guest hold `play_control`, `edit_bookmarks` and `upload`; the admin holds all. The guest role exists only while the admin has enabled guest access (setting in the database, off by default).
- Every HTTP handler and every intent checks the capability on the server. The server reports the effective capabilities to the client (`hello`/`whoami`); the interface hides what is not allowed. A missing or invalid token gives 401, a missing capability 403.
- While guest access is on, a request without token is a guest request, and `auth` with `guest: true` gives the same identity; an invalid token is never downgraded to guest. While it is off, a request without token gets 401.
- Unauthenticated access: only `setup` (when no user exists), `login`, a status call (set-up needed, guest enabled, protocol version) and the static web app.

**Upload.**
- `POST /api/v1/recordings` with the file as the raw request body and the original name in a header. The server streams it to a temporary file in the media directory while hashing, enforces `max_upload_bytes` (configuration) while reading, and checks the format by decoding it with the existing decoder (no extension list); the title is the name without extension. A file with a known hash is answered with the existing recording instead of a second copy.
- On success the file is renamed into the content-addressed store (ADR-004), the recording is created and analysis is queued; the response names the recording. Analysis progress arrives through the existing channel; the client shows bytes sent as upload progress.
- Unreadable or too large files are refused with a short, specific error; the temporary file is always removed.

**Administration.** Admin endpoints for users (create, change role and password, deactivate, delete), the guest switch, and recordings (rename, metadata, delete with removal of unreferenced media). The last active admin can be neither deleted, deactivated nor demoted.

## Alternatives considered

- JWT: stateless, but cannot be revoked without extra state; the server has a database anyway.
- Cookies: need CSRF protection and do not fit the bridge client that is not a browser.
- Multipart upload: needs a form parser and buffering for little gain; a raw body streams straight to disk.
- Extension or magic-byte list for formats: drifts from what the decoder can actually read.

## Consequences

- Without ADR-008 in use, credentials and tokens are readable on the network; the README states that clearly.
- Existing tests and browser runs authenticate (a test helper creates an admin) or run with guest access on.
- The bridge and client core carry the token and the capabilities; the shared state stays free of user data.
- Deleting a recording while it is loaded needs a rule (stop playback and unload); to be settled when the administration step is built.
