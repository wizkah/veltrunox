# ADR-014 – Desktop application with the server inside

**Status:** Accepted  
**Date:** 2026-11-26

## Context

Besides the headless server with its browser client, a native application is wanted, macOS first. It must work on its own, with the audio played on that machine, and its settings must switch a server for the network on and off, including its port. While that server is on, it offers the web app as before. The architecture so far describes a server and browser clients.

## Decision

- The application is the existing Flutter interface built for macOS; the Rust client core is a native library in the application bundle. There is no second user interface.
- The server wiring is a library (`veltrunox-server`), used by the headless program and, through `veltrunox-client-bridge`, by the application. The bridge depends on it on native targets only.
- One core runs for as long as the application. The interface always reaches it through HTTP and the real-time channel on a loopback port chosen at start. There is no second, in-process path.
- The loopback endpoint accepts an owner token, made anew at each start and handed to the interface through the bridge. It has all rights and no account. No other endpoint accepts it.
- The network endpoint is separate, has a runtime of its own (switching it off ends every connection), requires sign-in as the headless server does, and can use the local certificate authority of ADR-008. It can only be switched on once a user account exists. Port, HTTPS and whether it is on are stored in the data directory.
- Data is kept in `~/Library/Application Support/Veltrunox`, apart from any headless installation. The built web app is copied into the bundle.
- The application is not sandboxed, and built for Apple silicon only.

## Alternatives considered

- A SwiftUI interface: native controls, but a second interface to keep in step.
- Calling the use cases in process instead of over loopback: no socket, but a second implementation of `ServerLink` and of the real-time behaviour.
- Running the headless program as a child process: a second binary to ship, supervise and talk to for settings.
- Opening the loopback endpoint without any token: any local program or web page could then control it.

## Consequences

- Local and networked use take the same path and are covered by the same tests.
- Closing the application stops the server.
- Distributing the application to others needs a developer signature and notarisation; neither is set up.
