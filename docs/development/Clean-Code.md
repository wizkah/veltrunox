# Clean Code

These rules apply to all production and test code (Rust and Dart). GUI structure is governed additionally by `GUI-Architecture.md`. Deviations must have a concrete benefit and be justified.

## Readability

- Code and code-related texts are English throughout: identifiers, comments, API documentation, test descriptions, and log and technical error messages.
- Names describe purpose and meaning. Avoid unusual abbreviations.
- Functions stay small and fulfil one clearly named task.
- Use a consistent level of abstraction within a function (SLAP). Express domain flows and technical details separately.
- Comments explain reasons and constraints. Do not retell the code.
- Remove unused code and outdated comments.

## Responsibilities and dependencies

- Separation of Concerns: separate domain logic, flow control, presentation and external access.
- Single Responsibility: each unit has one coherent responsibility and one clear reason to change.
- Open/Closed: make foreseeable variants extensible in a targeted way. Do not build extension points in advance.
- Liskov Substitution: implementations honour the guaranteed contracts of their abstractions.
- Interface Segregation: interfaces contain only the capabilities their users need.
- Dependency Inversion: domain rules do not depend on concrete infrastructure implementations.
- Clean Architecture: source-code dependencies point inwards (adapters → application → domain). Ports are defined by the inner layer.
- Make dependencies and mutable state explicit. Avoid hidden global dependencies.

## Flow control and operations (IOSP, IODA)

- Apply IOSP where it improves readability and testability: integration coordinates operations; operations do the actual processing. An operation does not call other operations; an integration does not contain domain decisions.
- Apply IODA where it clarifies data flow: integrations compose operations, operations transform data, data types carry state without behaviour that depends on infrastructure, and APIs (entry points, adapters) translate between the outside world and the inner parts.
- Encapsulate domain decisions in operations. Integrations concentrate on how they work together.
- Express calculations as pure functions where possible. Keep input/output and other side effects clearly separated.
- Do not introduce additional layers or micro-functions merely to satisfy a principle formally.

## Reactive streams (rxrust)

- Use `rxrust` where a sequence of events over time is the natural model: incoming server messages, throttled or debounced user input, position ticks, connection status.
- Keep domain operations synchronous and free of stream types. Streams connect operations at the integration level.
- A stream has one owner. Other components receive values or narrow handles, not the subject.
- Make subscription lifetime explicit and release subscriptions on teardown. Do not leak subscriptions.
- Handle errors in the stream explicitly. Do not use streams for plain request/response calls.
- Keep operators readable: name non-trivial pipelines and extract steps into named functions.
- Verify that a stream-based design works on all targets, including `wasm32`, before building on it.

## Data and errors

- Express valid states through suitable types and clear contracts: `enum` for states, newtypes for identifiers, units in names or types (milliseconds, hertz, beats per minute).
- Validate data at system boundaries. Rely on defined contracts internally.
- Keep data immutable where possible; assign changes and resource ownership unambiguously.
- Handle errors explicitly or pass them on with their context. Use `Result` for expected errors; reserve panics for violated invariants. Do not swallow errors and do not fake success.
- Release resources also on errors and aborts.
- Do not put secrets in code or write them to logs.

## Rust specifics

- Format with `rustfmt` and lint with `clippy` without warnings in CI.
- No `unsafe` without a documented justification and a reviewed safety argument.
- Avoid `unwrap` and `expect` outside tests and start-up code with a documented invariant.
- Keep the library crates free of runtime and platform specifics; those belong to adapters and entry points.
- Pass time and randomness in as dependencies.
- Prefer borrowing over cloning in hot paths (audio, real-time messages); measure before optimising.
- Audio-thread code does not allocate, lock or block.
- Keep public APIs small; expose only what other crates need.

## Dart and Flutter specifics

- Format with `dart format`; analyse with the strictest agreed lint set without warnings in CI.
- Widgets contain presentation only. Logic with decisions, geometry or protocol knowledge belongs to the client core or to plain Dart units that do not import Flutter.
- Prefer immutable data classes and `const` widgets.
- Do not call infrastructure or server APIs from widgets.
- Structure, state boundaries and styling follow `GUI-Architecture.md`.

## Simplicity

- Choose the simplest solution that meets the current need.
- Do not duplicate knowledge and domain rules. Merge similar code only when it shares a meaning.
- Introduce abstractions only for a concrete need.
- Unify formatting and automatable rules through shared tools.

## Tests

- Test observable behaviour and contracts; do not couple tests to internal implementation details.
- Test domain logic in isolation and secure the interplay of external components with integration tests.
- Consider relevant edge cases, errors and aborts.
- Keep tests reproducible. Make time, randomness and external systems controllable where needed.
- Pure logic around time, pixels, snapping, regions, tempo and pitch calculations, and permissions is covered thoroughly and runs without network, audio or GUI.
- Widget tests cover component contracts: given inputs render as expected, user actions emit the expected events.
