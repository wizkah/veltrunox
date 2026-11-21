# ADR-001 – Technology stack and client core

**Status:** Accepted  
**Date:** 2026-11-21

## Context

An earlier prototype was written in C# (ASP.NET, SignalR, Avalonia). The new version is rebuilt with a different stack. The client runs in the browser first; other platforms follow. Business logic should not be duplicated between server and client, and the user interface should stay replaceable.

## Decision

- The server is written in Rust.
- The client core (application logic, protocol, session state, viewport arithmetic) is also written in Rust. It shares the domain crate with the server.
- The user interface is written in Flutter. The browser (WebAssembly) is the first target; other Flutter targets stay possible.
- The client core is connected to Flutter through `flutter_rust_bridge`, exposing a narrow API.
- Event streams in server and client core use `rxrust` where a stream is the natural model (see Architecture).

## Consequences

Domain rules exist once. Flutter holds only presentation. The client core must compile for `wasm32` and behave the same on later native targets; dependencies are chosen accordingly.

Not yet verified: that `flutter_rust_bridge`, the network client, `rxrust` and time handling work together in the browser build. The first implementation phase validates this with a vertical slice before more features are built. If a part fails, this ADR is superseded rather than silently bypassed.
