# AGENTS.md

LibEFB is an Electronic Flight Bag (EFB) library for flight planning
and air navigation, written in Rust. The `efb` crate's Flight
Management System (`FMS`) ties together the navigation data, the
route and the flight planning; supporting crates parse navigation
data from source formats such as ARINC 424 and AIXM, and `bindings/`
expose `efb` to C, Python, WASM and Swift.

- **Conventions**: `.claude/rules/` -- architecture, memory and
  performance, naming, coding, and how to plan and hand off work.
- **Setup, workflow, tests, changelog**: `CONTRIBUTING.md`.
- **Commit messages**: `.gitmessage`.
- **Route prompt syntax**: `handbook/src/RoutePrompt.md`.
- **Using the FMS**: the crate docs in `efb/src/lib.rs`.
