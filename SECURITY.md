# Security

## Reporting a vulnerability

Please report vulnerabilities privately through "Report a vulnerability" on the Security tab of the GitHub repository, not in a public issue. Veltrunox is a spare-time project; there is no guaranteed response time. Only the current state of `main` is maintained.

## Intended use

Veltrunox is built for a band's own, trusted network. Do not expose the server to the internet.

## Known limits

- Transport is plain HTTP by default: passwords, session tokens and API tokens can be read on the network. `tls = "local-ca"` or `tls = "files"` encrypts it (see the README).
- Until the first account exists, anyone who can reach the server can create the admin. Create it before opening the server to the network (`veltrunox-server users add`, or the setup page while `address` is still local).
- Repeated failed logins are delayed per user name, not per network address.
- An open real-time connection keeps its rights until it closes, also after the account was deactivated, changed or deleted, or the token revoked.
- With guest access on, anyone who can reach the server can control playback, edit bookmarks and upload files.
- New passwords need only 6 characters unless the admin selects the strict rules.
- The analysis program named in the configuration runs with the rights of the server and processes uploaded files.
