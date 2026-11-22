# ADR-008 – Transport security

**Status:** Proposed  
**Date:** 2026-11-22

## Context

The server runs on a Raspberry Pi that moves between networks, some without internet access, and is reached by IP address or a local name. Accounts with passwords (Architecture, section 6) make TLS necessary outside a trusted local network. A certificate from a public authority needs a public domain name, internet access at issue and renewal, and short renewal intervals; none of this holds for an offline rehearsal room, and an expired certificate would lock users out exactly there.

## Decision

| Mode | Behaviour |
|---|---|
| `off` (default) | Plain HTTP, for a trusted local network only. Documented as the exception the architecture allows |
| `local-ca` | The server creates its own certificate authority on first start and issues a server certificate for its local name (`<hostname>.local`) and current IP addresses. It issues a new one when the addresses change or expiry is near; the authority stays the same. The authority certificate is offered for download so that users can install it once per device |
| `files` | Certificate and private key are read from configured paths. Obtaining and renewing them is the operator's task, with any tool |

- No built-in support for public certificate authorities (ACME, for example Let's Encrypt): it cannot work offline and needs a domain and DNS access.
- Without an installed authority certificate, browsers warn; the user can still confirm and proceed.
- The private keys of the authority and the server certificate stay in the data directory, readable by the service user only, and never appear in logs.
- Mode and paths are settings in the configuration file.

## Consequences

- A rehearsal room without internet works without any dependency on external services.
- Each device needs a one-time installation of the authority certificate to avoid warnings.
- Over plain HTTP, browsers do not treat a LAN address as a secure context; features that need one are unavailable in mode `off`.
- The server needs a TLS implementation (`rustls`) and certificate generation (`rcgen`), plus mDNS announcement of the local name if it is not provided by the system.
