# Contributing

Veltrunox is a spare-time project. Bug reports, ideas and pull requests are welcome; answers may take a while.

## Issues

Search the existing issues first. For a bug, name the server system, the browser or application, what you did, what you expected and what happened. Report vulnerabilities as described in [SECURITY.md](SECURITY.md), not in an issue.

## Pull requests

- Open an issue before larger changes, so the direction is agreed before you do the work.
- Follow the [architecture](docs/architecture/Architecture.md), the [coding guidelines](docs/development/Clean-Code.md) and, for the interface, the [GUI architecture](docs/development/GUI-Architecture.md) and the [design specification](docs/design/). Decisions that change the architecture need a record in `docs/architecture/adr/`.
- Write code, comments and documentation in English. Add a line to the [changelog](CHANGELOG.md) for relevant changes.
- Add tests for changed behaviour.

Run before opening a pull request, as CI does:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && dart format --output=none --set-exit-if-changed lib test && flutter analyze && flutter test
```

Building and the browser tests are described in the [README](README.md#development).

## License

By contributing you agree that your contribution is licensed under the GNU General Public License v3.0 only, like the rest of the project (see [LICENSE](LICENSE)).
