# Memory and Performance

LibEFB loads whole navigation databases and runs on tablets, in the
browser through WASM and behind C, Python and Swift bindings. Every
value has a deliberate **ownership**: copied, shared, borrowed or
moved, and allocated at a known place. The signature makes that
ownership visible.

## Zero-Copy Parsing

`arinc424` is the reference for every parser:

- The parser borrows its input. `Records` yields
  `(RecordKind, &[u8])`, and a record's fields point into those 132
  bytes: `Alphanumeric<'a, N>` wraps `&'a [u8; N]`.
- A field decodes lazily, on access: `as_str`, `as_decimal`,
  `as_u32`. The caller pays only for the fields it reads.
- The field length is a const generic, checked once in `from_bytes`.
- A blank field is `Option<T>` through the blanket `FixedField` impl.
- Owned data appears where the caller asks for it, usually when
  `efb/src/nd/convert/` turns a record into a domain type.
- The success path stays allocation-free; only an error may copy the
  offending bytes (`bytes.to_vec()`).
- A nested format such as AIXM is read with a pull parser, one event
  at a time.

## Ownership

- **Copy** small values and pass them by value: measurements,
  `VerticalDistance`, `Fuel`, `Wind`, field enums. Keep them small:
  `f32` for measurements, the smallest integer that holds the domain
  (`u16` for feet and flight levels).
- **Inline** short codes of known length as byte arrays:
  `LocationIndicator([u8; 2])`, `Region::TerminalArea([u8; 4])`.
- **Share** large, immutable data with `Arc`. Airports, waypoints and
  airspaces live once in `NavigationData`; the route, the indexes and
  `NavAid` hold `Arc`s to them. Write `Arc::clone(&x)` so the reader
  sees a reference count. `Arc` keeps the `FMS` `Send + Sync`.
- **Move** what is consumed: a builder takes its parts by value and
  `build(self)` moves them into the result
  (`into_values().map(Arc::new)`).
- **Borrow** what is only read: arguments take `&str`, `&[T]`,
  `&Point<f64>`; a function takes ownership only to store the value
  (`set_alternate(Option<NavAid>)`).

## Returning Data

- A getter returns a borrow of what the type holds: `&str` from
  `Fix::ident`, `&[Leg]` from `Route::legs`, `Option<&T>` through
  `as_ref()`. The caller clones when it keeps the value.
- A borrow lives as long as its source: `LegPerformance<'a>::cruise`
  returns `Option<&'a Performance>`.
- Return `impl Iterator<Item = &T> + '_` where the caller may filter,
  stop early or count. Collect once, at the boundary that needs owned
  results (`Nearby`).

## Compute

- Size every algorithm for a full AIRAC database. A linear scan over
  all navaids inside a loop needs a justification in the plan.
- Filter coarse, then exact: the R-tree returns candidates by
  bounding box, and only those get the polygon or geodesic test.
- Build an index once, when the data is built (`RTree::bulk_load`).
- Re-evaluate only what changed: the `FMS` pipeline starts at the
  stage an input affects (`skip_until`).
- Reserve capacity (`Vec::with_capacity`) when the size is known
  before the loop.
- Reserve `#[inline]` for tiny hot accessors called across crates,
  as on the `arinc424` fields.

## Measurements

Compute physical quantities with `measurements` types, so units stay
compatible: `Length / Speed` is a `Duration`. When an operation
between two quantities is missing, implement it on the measurement
and keep `value()` for display and the bindings. The types are
`Copy` and `#[repr(C)]`, so the typing is free.

## Unsafe

`unsafe` is a workspace lint warning. It earns its place only where
it removes a measured cost and the invariant is checked right before
it, as in `Alphanumeric::from_bytes` after the length check. A
comment beside the block names the invariant.

## Hot Paths

Treat a loop over navigation data or records as a hot path. Each
clone, `to_string`, `collect` or `Box` in it goes to the human as a
question, with its cost and the reason a borrow falls short. Back a
performance claim with a measurement: `arinc424/benches/` holds
criterion benchmarks; propose a run for a change on a parsing hot
path.
