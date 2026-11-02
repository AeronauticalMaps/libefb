# Contributing to LibEFB

Thank you for your interest in contributing to LibEFB! This guide
covers the setup, the workflow and where the project's conventions
live. Be respectful, constructive and professional in all
interactions.

## Getting Started

### Prerequisites

- Rust toolchain, latest stable
- Ruby, for the pre-commit hook
- For the language bindings:
  - **Python**: pip (the build runs maturin through `pyproject.toml`)
  - **WASM**: wasm-pack (`cargo install wasm-pack`)
  - **Swift**: Xcode, for the XCFramework built by
    `bindings/swift/swift-efb/Makefile`
  - **C**: nothing extra; `build.rs` runs cbindgen and writes the
    header to `bindings/c/include/`

### Repository Setup

Enable the pre-commit hook, which keeps the copyright year of the
license headers current, and the commit message template:

```sh
git config core.hooksPath .githooks
git config commit.template .gitmessage
```

### Building and Testing

```sh
cargo build                       # the whole workspace
cargo build -p efb --all-features # efb with serde, sqlite, geojson, handbook
cargo test                        # all tests, as CI runs them
cargo test -p efb --test route_decoding_test
cargo bench -p arinc424           # criterion benchmarks of the parser
cargo doc --open
```

## Workspace

| Path               | Content                                            |
|--------------------|----------------------------------------------------|
| `efb/`             | Core library: FMS, navigation data, route, planning |
| `arinc424/`        | Zero-copy ARINC 424 parser                         |
| `arinc424-derive/` | Derive macro for ARINC 424 records                 |
| `aixm/`            | AIXM 5 streaming parser (in progress)              |
| `bindings/`        | C, Python, WASM and Swift bindings                 |
| `handbook/`        | mdBook user handbook, e.g. the route prompt syntax |

## Design and Code Conventions

LibEFB is designed with care for memory and compute: parsers borrow
their input instead of copying it, navigation data is shared rather
than duplicated, and every physical quantity is a typed measurement.
The conventions are written down once, in `.claude/rules/`, and apply
to human and AI contributors alike:

- [`architecture.md`](.claude/rules/architecture.md) -- the crate and
  module boundaries that stay fixed
- [`memory-and-performance.md`](.claude/rules/memory-and-performance.md)
  -- ownership, zero-copy parsing and hot paths
- [`naming-conventions.md`](.claude/rules/naming-conventions.md) --
  the pilot's vocabulary and names that tell the cost of a call
- [`coding-conventions.md`](.claude/rules/coding-conventions.md) --
  style, documentation and tests
- [`working-with-humans.md`](.claude/rules/working-with-humans.md) --
  how AI contributors plan and hand off their work

Read them before your first change; the Rust API Guidelines
(https://rust-lang.github.io/api-guidelines/) fill in the rest.

## Development Workflow

1. Fork the repository and branch from `main`.
2. Make small, focused commits; each one builds and passes the tests.
3. Add tests for new behaviour.
4. Run `cargo fmt`, `cargo clippy` and `cargo test`.
5. Add a line under `[unreleased]` in the changelog of the crate you
   changed: `CHANGELOG.md` for `efb`, `arinc424/CHANGELOG.md` for the
   parser. The changelogs follow [Keep a Changelog].
6. Update the handbook if you change what a user enters, such as the
   route prompt syntax.
7. Push to your fork and open a pull request.

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/

## Commit Messages

`.gitmessage` is the single source of truth: a module tag, a summary
of at most 50 characters, a body wrapped at 72 that explains what and
why, and a sign-off (`git commit -s`). With the template configured
as above, `git commit -s` opens it in your editor.

```
nd: Return &str from Fix::ident()

Let it be up to the caller to clone the identifier if really needed.
This should improve things when filtering navaids.

Signed-off-by: Jane Doe <jane@example.com>
```

## Testing

- **Unit tests** sit in a `#[cfg(test)] mod tests` at the end of the
  file they test.
- **Integration tests** live in `<crate>/tests/<topic>_test.rs`, e.g.
  `efb/tests/route_decoding_test.rs`.
- **Doc tests** are the examples in the rustdoc comments; they compile
  and run with `cargo test`.
- **Benchmarks** of the ARINC 424 parser live in `arinc424/benches/`.
  Run them for changes on a parsing hot path and add the numbers to
  the pull request.

Tests feed real data where they can: an ARINC 424 record line, a
route prompt, a table from a POH.

## Pull Requests

### Checklist

- [ ] `cargo test` passes
- [ ] `cargo fmt` and `cargo clippy` are clean
- [ ] New behaviour has tests, new public API has rustdoc
- [ ] The changelog has an entry under `[unreleased]`
- [ ] Commit messages follow `.gitmessage`

### Description

Describe what changed and why, any breaking change with its migration,
benchmark numbers for performance work, and related issues.

### Review

Address review feedback promptly, rebase on `main` when conflicts
arise, and squash or clean up commits if asked.

## Questions?

Open an issue, check the README for an overview, or read the existing
code for examples -- `arinc424` and `efb::measurements` show the
project's style well.
