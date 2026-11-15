# Coding Conventions

> Programs must be written for people to read, and only incidentally
> for machines to execute. — Harold Abelson and Gerald Jay Sussman

Code reads top to bottom like an explanation in conversation. Names
carry the meaning, so a reader understands what happens without
jumping around. Comments describe the present code, and only where
the context leaves it unclear: a magic number, a non-obvious
constraint, a spec quirk. Why something changed belongs in the
commit message.

Structure follows `architecture.md`, data handling
`memory-and-performance.md`, names `naming-conventions.md`. Before
writing a helper, check the existing API -- measurements, `core`
types, builders, `Fix` -- and extend it if a small change covers the
task.

## Style

- Every source file starts with the Apache 2.0 license header; copy
  it from a neighbouring file. The pre-commit hook keeps the
  copyright year current.
- Comments wrap at 72 columns, narrower than the code.
- Write the em dash as `--` in comments.
- A long file separates its major sections with a banner:

  ```rust
  /////////////////////////////////////////////////////////////////////////////
  // Numeric Field
  /////////////////////////////////////////////////////////////////////////////
  ```

## Documentation

Document by visibility, with the `rustdoc` skill: every public item
has a doc comment, with one `# Examples` section and `# Errors` or
`# Panics` where they apply; a private helper relies on its name; a
module gets one overview of its components. In `arinc424`, the doc
comment cites the spec section the item implements.

Write for the developer who searches the docs for how to use an item,
in the tone of the existing docs: short, plain sentences with a
pilot's example (`EDDH` for Hamburg), citing a spec instead of
repeating its wording. Each fact sits where its reader looks for it:

- A module overview names the document it implements (`ICAO Doc
  7910`, `ARINC 424`, `AIXM 5.1`), its components and how they
  relate, nothing more.
- A doc comment says what the item is and how to call it.
- A corner case is a comment beside the code that handles it.
- A detail of a source format belongs to its parser or its
  `nd/convert/` submodule, not to the domain type.

## Tests

- Unit tests sit in a `#[cfg(test)] mod tests` at the end of the file
  they test; integration tests in `<crate>/tests/<topic>_test.rs`.
- A test feeds real data where it can: an ARINC 424 record line, a
  route prompt, a POH table.
- Unwrap with `expect("... should ...")`, so a failure says what was
  expected.
