# Rust API Guidelines Checklist

Reference: https://rust-lang.github.io/api-guidelines/checklist.html

When auditing a public API, check each applicable item below. Report violations with the guideline ID (e.g., C-CASE) so the user can look up the full rationale.

## Naming

- **C-CASE**: Types use `UpperCamelCase`, functions/methods use `snake_case`, constants use `SCREAMING_SNAKE_CASE`. Acronyms count as one word: `Uuid`, not `UUID`.
- **C-CONV**: Conversion methods use correct prefixes:
  - `as_` — free, borrowed-to-borrowed (e.g., `str::as_bytes()`)
  - `to_` — expensive or value-producing (e.g., `str::to_lowercase()`)
  - `into_` — ownership-consuming (e.g., `String::into_bytes()`)
- **C-GETTER**: Getters omit `get_` prefix. Use bare names: `len()`, not `get_len()`.
- **C-ITER**: Iterator-producing methods:
  - `iter()` — iterates over `&T`
  - `iter_mut()` — iterates over `&mut T`
  - `into_iter()` — iterates over `T`, consuming `self`
- **C-ITER-TY**: Iterator type names match their method: `into_iter()` → `IntoIter`.
- **C-FEATURE**: Feature names are direct nouns/adjectives, no `use-` or `with-` prefixes.
- **C-WORD-ORDER**: Related names follow consistent word order (e.g., `ParseIntError`, `ParseFloatError`).

## Documentation

- **C-CRATE-DOC**: Crate root has thorough docs with examples.
- **C-EXAMPLE**: Every public item has a runnable example.
- **C-QUESTION-MARK**: Examples use `?` for error handling, not `unwrap()` or `try!`.
- **C-FAILURE**: Functions document their failure modes:
  - `# Errors` section for `Result`-returning functions
  - `# Panics` section for functions that can panic
  - `# Safety` section for `unsafe` functions
- **C-LINK**: Doc prose hyperlinks to related types/functions via intra-doc links.
- **C-METADATA**: `Cargo.toml` includes: authors, description, license, repository, keywords, categories.
- **C-HTML-ROOT**: If publishing pre-rendered docs, set `html_root_url`.
- **C-RELNOTES**: Release notes document all significant changes per version.

## Interoperability

- **C-COMMON-TRAITS**: Types implement applicable standard traits:
  - `Copy`, `Clone` (if semantically appropriate)
  - `Eq`, `PartialEq`, `Ord`, `PartialOrd` (if ordering makes sense)
  - `Hash` (if `Eq` is implemented)
  - `Debug` (almost always)
  - `Display` (for user-facing types)
  - `Default` (if a sensible default exists)
  - `Send`, `Sync` (if thread-safe)
- **C-CONV-TRAITS**: Use `From`/`TryFrom`/`AsRef`/`AsMut` for conversions. Never implement `Into` or `TryInto` directly.
- **C-COLLECT**: Collections implement `FromIterator` and `Extend`.
- **C-SERDE**: Data structures implement Serde traits behind an optional `serde` feature.
- **C-SEND-SYNC**: Types are `Send`/`Sync` where possible.
- **C-GOOD-ERR**: Error types implement `std::error::Error` + `Send` + `Sync`. Use specific types, never `()`. Error messages are lowercase without trailing punctuation.

## Type Safety

- **C-NEWTYPE**: Use newtypes for static distinctions (e.g., `Miles` vs `Kilometers`).
- **C-CUSTOM-TYPE**: Prefer custom types over `bool`/integer arguments for clarity.
- **C-BITFLAG**: Use `bitflags` crate for sets of independent flags, not enums.
- **C-BUILDER**: Use builder pattern for types with many optional parameters.

## Dependability

- **C-VALIDATE**: Validate arguments. Prefer compile-time enforcement via types; use `debug_assert!` or `_unchecked` variants for runtime.
- **C-DTOR-FAIL**: Destructors must never fail. Provide explicit `close()` methods for fallible cleanup.
- **C-DTOR-BLOCK**: Destructors that may block should have non-blocking alternatives.

## Debuggability

- **C-DEBUG**: All public types implement `Debug`.
- **C-DEBUG-NONEMPTY**: `Debug` output is never empty — use `""` for empty strings, `[]` for empty collections.
- **C-DISPLAY**: Implement `Display` for types that have a natural user-facing representation.

## Future-Proofing

- **C-SEALED**: Seal traits that should not be implemented downstream.
- **C-STRUCT-PRIVATE**: Keep struct fields private; use accessors.
- **C-STRUCT-BOUNDS**: Don't put trait bounds on struct definitions — put them on impls instead.

## Necessities

- **C-STABLE**: All public dependencies must be stable (≥ 1.0).
- **C-PERMISSIVE**: Use permissive licensing (MIT/Apache-2.0 dual license recommended).
