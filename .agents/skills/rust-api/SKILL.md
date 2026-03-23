---
name: rust-api
description: "Design Rust public APIs that are idiomatic, self-explanatory, and follow the official API guidelines. Use this skill when creating new public types, traits, functions, or modules in Rust — or when reviewing existing APIs for naming clarity, type safety, trait implementations, and ergonomics. Also trigger when the user asks about Rust naming conventions, newtype patterns, builder patterns, conversion traits, or asks to review/improve a Rust API surface. This skill covers API shape and naming; for documentation comments and doctests, defer to the rustdoc skill."
---

# Rust API Design

Design public Rust APIs that are idiomatic, self-explanatory, and consistent
with the [API Guidelines](https://rust-lang.github.io/api-guidelines/) and
[Rust Idioms](https://rust-unofficial.github.io/patterns/idioms/).

For documentation comments and doctests, defer to the **rustdoc** skill.

## Workflow

1. Read the code being designed or reviewed
2. Evaluate names, types, and signatures against the guidelines below
3. Suggest concrete improvements — propose actual names and signatures, not
   abstract advice
4. For the full checklist, read `references/api_checklist.md`

## Names that explain themselves

The best API needs minimal documentation because the names carry the meaning.
A reader should understand what a function does, what a type represents, and
what a parameter controls — all from the signature alone.

### Naming conventions (C-CASE, C-CONV, C-GETTER, C-WORD-ORDER)

| Item | Convention | Example |
|---|---|---|
| Types, traits | `UpperCamelCase` | `FlightPlan`, `IntoIterator` |
| Functions, methods | `snake_case` | `compute_fuel`, `as_bytes` |
| Constants, statics | `SCREAMING_SNAKE_CASE` | `MAX_ALTITUDE` |
| Type parameters | short `UpperCamelCase` | `T`, `K`, `V`, `E` |
| Lifetimes | short `lowercase` | `'a`, `'src` |

### Conversion method prefixes (C-CONV)

These prefixes tell the caller about cost and ownership at a glance:

| Prefix | Cost | Ownership | Example |
|---|---|---|---|
| `as_` | Free / cheap | `&self → &T` | `as_bytes`, `as_str` |
| `to_` | Expensive | `&self → T` | `to_string`, `to_vec` |
| `into_` | Variable | consumes `self` | `into_inner`, `into_bytes` |

If a conversion doesn't fit these, use `From`/`Into` traits instead.

### Getters drop the `get_` prefix (C-GETTER)

```rust
// Good — reads naturally
fn name(&self) -> &str
fn level(&self) -> VerticalDistance
fn is_empty(&self) -> bool

// Bad — the prefix adds noise without information
fn get_name(&self) -> &str
fn get_level(&self) -> VerticalDistance
```

Boolean getters use `is_` or `has_` — they read as questions:
`is_empty`, `has_waypoints`, `is_valid`.

### Word order: noun first, qualifier second (C-WORD-ORDER)

Put the type name (the primary concept) first:

```rust
FlightPlan        // not PlanFlight
JoinHandle        // not HandleJoin
FuelFlow          // not FlowFuel
VerticalDistance   // not DistanceVertical
```

### Iterator methods and types (C-ITER, C-ITER-TY)

| Method | Receiver | Yields | Iterator type |
|---|---|---|---|
| `iter()` | `&self` | `&T` | `Iter<'a>` |
| `iter_mut()` | `&mut self` | `&mut T` | `IterMut<'a>` |
| `into_iter()` | `self` | `T` | `IntoIter` |

### Choose precise names over short names

Spend time finding the right word. A longer but precise name beats a short
but ambiguous one:

```rust
// Precise — the reader knows exactly what this is
vertical_rate: VerticalRate
horizontal_distance_tas: Length

// Ambiguous — rate of what? distance in what frame?
rate: f32
dist: f32
```

When reviewing names, ask: *"If I saw this name with no surrounding context,
would I know what it means?"* If not, the name needs work.

That said, established domain abbreviations are fine when the target audience
knows them. In an aviation crate, `ff` (fuel flow), `tas` (true airspeed),
`fpm` (feet per minute), and `roc` (rate of climb) are clearer to pilots
than their verbose expansions — and overly long names can be just as
distracting as overly short ones. The test is whether the abbreviation is
standard in the domain, not whether a random reader would guess it.

## Type safety over primitive obsession (C-NEWTYPE, C-CUSTOM-TYPE)

Use the type system to make invalid states unrepresentable. Every time you
reach for a bare `f32`, `bool`, or `String`, ask whether a dedicated type
would prevent misuse.

### Newtypes for domain concepts

```rust
// Good — can't accidentally mix altitude with speed
struct Altitude(f32);
struct Speed(f32);

fn compute(alt: Altitude, spd: Speed) -> Duration { ... }

// Bad — f32 values are interchangeable by accident
fn compute(alt: f32, spd: f32) -> f32 { ... }
```

### Enums over booleans (C-CUSTOM-TYPE)

When a parameter selects between two modes, an enum is self-documenting:

```rust
// Good — the call site reads clearly
set_mode(Mode::Approach)
set_mode(Mode::Cruise)

// Bad — what does `true` mean here?
set_mode(true)
```

### Builder pattern for complex construction (C-BUILDER)

When a constructor would need more than 3-4 parameters, or has many optional
fields, use a builder:

```rust
let plan = FlightPlan::builder()
    .departure("EDDF")
    .destination("EDDM")
    .cruise_level(VerticalDistance::FlightLevel(100))
    .build()?;
```

## Accept borrowed types in arguments

Accept the most general borrowed form to maximize what callers can pass
without allocation:

```rust
// Good — accepts &str, &String, and string slices
fn find_waypoint(name: &str) -> Option<Waypoint>

// Unnecessarily restrictive
fn find_waypoint(name: &String) -> Option<Waypoint>

// Good — accepts &[T], &Vec<T>, and array slices
fn process_legs(legs: &[Leg]) -> Duration

// Unnecessarily restrictive
fn process_legs(legs: &Vec<Leg>) -> Duration
```

For owned data, use generics:

```rust
// Good — caller decides whether to clone or move
fn set_name(name: impl Into<String>)
```

## Trait implementations

### Eagerly implement common traits (C-COMMON-TRAITS)

Every public type should derive or implement the traits that make sense for
it. Ask for each type:

- **`Debug`** — always (required for good error messages and logging)
- **`Clone`** — unless the type manages a unique resource
- **`PartialEq`, `Eq`** — if equality comparison is meaningful
- **`Hash`** — if the type might be used as a map key
- **`Default`** — if there's a sensible "zero" or "empty" state
- **`Display`** — if the type has a natural human-readable representation
- **`Copy`** — if the type is small and cheap to copy (e.g. newtypes over
  numbers, enums without data)
- **`Send`, `Sync`** — ensured automatically unless interior mutability or
  raw pointers opt out

### Use standard conversion traits (C-CONV-TRAITS)

Prefer `From`/`Into` over ad-hoc conversion methods when the conversion is
infallible and value-to-value:

```rust
impl From<Knots> for Speed {
    fn from(kt: Knots) -> Self { ... }
}
```

Use `TryFrom`/`TryInto` when the conversion can fail.

### Error types (C-GOOD-ERR)

Error types should:
- Implement `std::error::Error` + `Send` + `Sync` + `'static`
- Implement `Display` with a human-readable message
- Be an enum when there are distinct failure modes the caller might match on

## API shape

### Functions with a clear receiver are methods (C-METHOD)

If a function's first argument is clearly "the thing being operated on",
make it a method:

```rust
// Good
impl FlightPlan {
    fn total_distance(&self) -> Length { ... }
}

// Bad — the receiver is obvious, so free function adds noise
fn total_distance(plan: &FlightPlan) -> Length { ... }
```

### No out-parameters (C-NO-OUT)

Return values instead of taking mutable references to fill:

```rust
// Good
fn compute(&self) -> ClimbDescentResult

// Bad
fn compute(&self, result: &mut ClimbDescentResult)
```

### Expose intermediate results (C-INTERMEDIATE)

When a computation produces useful intermediate values, expose them so
callers don't have to recompute:

```rust
// Good — caller gets time, fuel, AND distance from one call
struct ClimbDescentResult {
    time: Duration,
    fuel: Fuel,
    horizontal_distance_tas: Length,
}
```

### Constructors are static inherent methods (C-CTOR)

Use `new` for the primary constructor, and descriptive names for
alternatives:

```rust
impl Waypoint {
    fn new(ident: &str, position: Position) -> Self { ... }
    fn from_coordinates(lat: f64, lon: f64) -> Self { ... }
}
```

## Future proofing

### Keep struct fields private (C-STRUCT-PRIVATE)

Expose fields through methods so you can change internals later without
breaking callers. The exception is "data bag" structs (like results or
configuration) where all fields are part of the public contract.

### Use `#[non_exhaustive]` for extensibility

On enums and structs that might grow variants or fields:

```rust
#[non_exhaustive]
pub enum RouteError {
    InvalidWaypoint(String),
    NoRoute,
}
```

This lets you add new variants without a breaking change.

## Reviewing an existing API

When asked to review, check these in order of impact:

1. **Names** — Are types, methods, and parameters self-explanatory?
2. **Types** — Are there bare primitives that should be newtypes?
3. **Signatures** — Do functions accept borrowed types? Do they return values
   (not out-params)? Are receivers `&self` methods?
4. **Traits** — Are `Debug`, `Clone`, `PartialEq`, `Default` derived where
   appropriate? Are conversions using `From`/`Into`?
5. **Consistency** — Do similar operations follow the same patterns across
   the codebase?

For the full checklist, read `references/api_checklist.md`.
