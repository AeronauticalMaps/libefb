---
name: rustdoc
description: "Write and improve Rust documentation comments following the rustdoc book and Rust API guidelines. Use this skill whenever documenting Rust code, adding doc comments, writing crate-level docs, creating doctests, or when the user asks to document a module/struct/enum/trait/function. Also trigger when the user mentions rustdoc, API guidelines, doc comments, doctests, or asks to check documentation quality. This skill covers both writing new documentation and auditing existing docs for completeness and API guideline compliance."
---

# Rustdoc Skill

Document Rust code following the rustdoc book conventions and ensure the public API complies with the Rust API Guidelines checklist.

## Workflow

1. Read the target code (file, module, or crate)
2. Identify all public items that need documentation
3. Write or improve doc comments following the structure below
4. Verify API guideline compliance using the checklist in `references/api_checklist.md`
5. Run `cargo doc --no-deps` to verify the docs build cleanly
6. Run `cargo test --doc` to verify all doctests pass

## Doc Comment Syntax

Use `///` for items and `//!` for crate/module-level docs. The attribute form `#[doc = include_str!("../README.md")]` is useful for pulling in external files.

## Doc Comment Structure

Every public item follows this structure. Not all sections apply to every
item — include only the relevant ones. Custom sections (like
`# Derivation` or `# Reading from a POH`) are fine when they genuinely
help the reader.

Do **not** add a `# Returns` section. Return types should be
self-explanatory from the signature and summary line. When extra context is
needed (e.g. what causes `None` or which error variants apply), weave it
into the prose description so the reader sees it alongside the behaviour
rather than in a separate heading.

```
/// One-line summary describing what this item does.
///
/// Longer explanation if the summary isn't sufficient. Explain behavior,
/// semantics, and any non-obvious design decisions. Don't repeat type
/// information that's already visible in the signature.
///
/// Returns `None` if the table is empty or the level cannot be expressed
/// in feet.
///
/// # Examples
///
/// ```
/// // A complete, compilable example demonstrating typical usage.
/// // Use `?` for error handling, not `unwrap()`.
/// ```
///
/// # Panics
///
/// Describe conditions under which this function panics.
///
/// # Errors
///
/// Describe the error variants returned and when each occurs.
///
/// # Safety
///
/// For `unsafe` functions: explain the invariants the caller must uphold.
```

### Section ordering

Use this order when multiple sections apply: Examples, Panics, Errors, Safety.

### Summary line

The first line is used in module-level listings and search results. Make it a single sentence in third person ("Returns the...", "Creates a new...", "Parses the..."). Don't start with "This function" or "This struct" — the context is already clear from where the doc appears.

## Doctests

Doctests are both documentation and tests. They prove the example compiles and works.

### Writing good doctests

- Make them complete and compilable — a reader should be able to copy-paste and run
- Use `?` operator for fallible operations, wrapping in `fn main() -> Result<...>` if needed
- Hide boilerplate with `# ` prefix (hidden from rendered docs but still compiled)
- Keep examples focused on the item being documented

### Doctest annotations

Use these fence attributes when a plain ```` ```rust ```` block doesn't fit:

| Annotation | When to use |
|---|---|
| `no_run` | Example is correct but has side effects (network, filesystem) |
| `should_panic` | Example demonstrates a panic case |
| `compile_fail` | Example shows what does NOT compile (useful for type safety docs) |
| `ignore` | Example can't be tested in CI (rare, needs justification) |

```rust
/// ```no_run
/// // This connects to a server, so we compile-check but don't run it.
/// let client = MyClient::connect("example.com:8080")?;
/// ```
```

### Hidden lines

Use `# ` to hide setup code that would distract from the point:

```rust
/// ```
/// # use my_crate::MyStruct;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let s = MyStruct::new();
/// assert_eq!(s.len(), 0);
/// # Ok(())
/// # }
/// ```
```

## Intra-Doc Links

Link to other items by path instead of URLs. This keeps links valid across versions and refactors.

```rust
/// Converts this into a [`Vec<u8>`].
/// See [`Option::unwrap`] for the panic behavior.
/// Related: [`Self::into_inner`], [`crate::errors`]
```

When a name exists in multiple namespaces, disambiguate:

```rust
/// See [`Foo`](struct@Foo) for the type, [`foo()`] for the function.
```

## Crate-Level Documentation

The crate root (`lib.rs`) uses `//!` comments and should include:

1. **What the crate does** — one paragraph summary
2. **Quick start example** — the minimum code to do something useful
3. **Module organization** — brief guide to where things live (for larger crates)
4. **Feature flags** — what each Cargo feature enables

```rust
//! A library for parsing ARINC 424 navigation data records.
//!
//! # Quick Start
//!
//! ```
//! use arinc424::Record;
//!
//! let record = Record::parse(line)?;
//! ```
//!
//! # Modules
//!
//! - [`records`] — Parsed record types (airports, waypoints, airways)
//! - [`fields`] — Individual field parsers and types
```

## Module Documentation

Each public module gets `//!` docs at the top explaining its purpose and how its types relate to each other. For modules that re-export items, mention which items are the main entry points.

## Documenting Specific Item Types

### Structs

Document the struct's purpose, its invariants, and how to construct it. If it has a builder or `new()`, the primary example belongs there. Document individual fields only if they're public — prefer private fields with accessor methods.

### Enums

Document the enum's purpose and when to use it. Each variant gets its own doc comment explaining when that variant applies. For enums with data, show how to construct and destructure.

### Traits

Explain what implementing the trait means semantically — not just the method signatures. Include an example implementation. If the trait is sealed, document that.

### Functions and Methods

Focus on behavior: what goes in, what comes out, what side effects occur. The signature already shows types, so don't repeat them in prose. Document panics and errors thoroughly since callers need this to write correct code.

### Macros

Macro docs need extra care because the signature doesn't convey usage patterns. Show the invocation syntax and multiple usage forms if the macro supports them.

## API Guidelines Compliance

After writing documentation, verify the public API against the Rust API Guidelines. Read `references/api_checklist.md` for the full checklist. The most common documentation-related items:

- **C-CRATE-DOC**: Crate-level docs are thorough with examples
- **C-EXAMPLE**: Every public item has an example
- **C-QUESTION-MARK**: Examples use `?`, not `unwrap()` or `try!`
- **C-FAILURE**: Error/panic/None conditions are documented
- **C-LINK**: Prose links to related items via intra-doc links
- **C-METADATA**: `Cargo.toml` has authors, description, license, repository
- **C-RELNOTES**: Release notes document significant changes
- **C-DEBUG**: All public types implement `Debug`
- **C-COMMON-TRAITS**: Types implement applicable standard traits (`Clone`, `PartialEq`, `Hash`, `Default`, etc.)
- **C-CONV**: Conversion method prefixes follow `as_`/`to_`/`into_` conventions
- **C-GETTER**: Getters omit the `get_` prefix
- **C-ITER**: Iterator methods use `iter()`/`iter_mut()`/`into_iter()`
- **C-GOOD-ERR**: Error types implement `std::error::Error` + `Send` + `Sync`

When you find a compliance issue, flag it and suggest the fix. Non-documentation items (like missing trait impls) should be reported but fixing them is optional — the user may want to address those separately.

## Lints

Suggest adding these lints to the crate root when they're missing:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
```

These catch undocumented public items and broken links at compile time.

## What NOT to do

- Don't add doc comments to private items unless the user asks — focus on the public API
- Don't document obvious getters with "Returns the X" boilerplate if the method name already says it (e.g., `fn name(&self) -> &str` doesn't need "Returns the name")
- Don't add `# Examples` sections that just repeat the type signature with no meaningful usage
- Don't add a `# Returns` section — return types should be self-explanatory from the signature, with `None`/error conditions woven into the prose when needed.
- Don't over-document: if a method is a trivial accessor or delegation, a one-line summary suffices
- Don't use `unwrap()` in doctests — use `?` with proper error handling
- Don't add HTML or raw links when intra-doc links work
