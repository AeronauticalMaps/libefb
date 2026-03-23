# Rust API Guidelines Checklist

Source: https://rust-lang.github.io/api-guidelines/checklist.html

## Naming

- **C-CASE**: Casing conforms to RFC 430 (`UpperCamelCase` for types/traits, `snake_case` for functions/methods/variables, `SCREAMING_SNAKE_CASE` for constants/statics)
- **C-CONV**: Ad-hoc conversions follow `as_` (cheap ref-to-ref), `to_` (expensive, or ref-to-owned), `into_` (owned-to-owned, consumes self) conventions
- **C-GETTER**: Getter names follow Rust convention — no `get_` prefix: `fn name(&self) -> &str`, not `fn get_name(&self) -> &str`
- **C-ITER**: Methods on collections producing iterators follow `iter` (&self), `iter_mut` (&mut self), `into_iter` (self)
- **C-ITER-TY**: Iterator type names match the methods that produce them (e.g. `Iter`, `IterMut`, `IntoIter`)
- **C-FEATURE**: Feature names are free of placeholder words (no `use-std`, just `std`)
- **C-WORD-ORDER**: Names use a consistent word order — type name first, then qualifier: `JoinHandle` not `HandleJoin`

## Interoperability

- **C-COMMON-TRAITS**: Types eagerly implement common traits: `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Display`, `Default`
- **C-CONV-TRAITS**: Conversions use standard traits `From`, `AsRef`, `AsMut`
- **C-COLLECT**: Collections implement `FromIterator` and `Extend`
- **C-SERDE**: Data structures implement Serde's `Serialize`, `Deserialize` (behind a feature flag)
- **C-SEND-SYNC**: Types are `Send` and `Sync` where possible
- **C-GOOD-ERR**: Error types are meaningful and well-behaved (`std::error::Error` + `Send` + `Sync`)
- **C-NUM-FMT**: Binary number types provide `Hex`, `Octal`, `Binary` formatting
- **C-RW-VALUE**: Generic reader/writer functions take `R: Read` and `W: Write` by value

## Predictability

- **C-SMART-PTR**: Smart pointers do not add inherent methods
- **C-CONV-SPECIFIC**: Conversions live on the most specific type involved
- **C-METHOD**: Functions with a clear receiver are methods
- **C-NO-OUT**: Functions do not take out-parameters
- **C-OVERLOAD**: Operator overloads are unsurprising
- **C-DEREF**: Only smart pointers implement `Deref` and `DerefMut`
- **C-CTOR**: Constructors are static, inherent methods

## Flexibility

- **C-INTERMEDIATE**: Functions expose intermediate results to avoid duplicate work
- **C-CALLER-CONTROL**: Caller decides where to copy and place data
- **C-GENERIC**: Functions minimize assumptions about parameters using generics
- **C-OBJECT**: Traits are object-safe if they may be useful as a trait object

## Type Safety

- **C-NEWTYPE**: Newtypes provide static distinctions
- **C-CUSTOM-TYPE**: Arguments convey meaning through types, not `bool` or `Option`
- **C-BITFLAG**: Types for a set of flags are `bitflags`, not enums
- **C-BUILDER**: Builders enable construction of complex values

## Dependability

- **C-VALIDATE**: Functions validate their arguments
- **C-DTOR-FAIL**: Destructors never fail
- **C-DTOR-BLOCK**: Destructors that may block have alternatives

## Debuggability

- **C-DEBUG**: All public types implement `Debug`
- **C-DEBUG-NONEMPTY**: Debug representation is never empty

## Future Proofing

- **C-SEALED**: Sealed traits protect against downstream implementations
- **C-STRUCT-PRIVATE**: Structs have private fields
- **C-NEWTYPE-HIDE**: Newtypes encapsulate implementation details
- **C-STRUCT-BOUNDS**: Data structures do not duplicate derived trait bounds

## Necessities

- **C-STABLE**: Public dependencies of a stable crate are stable
- **C-PERMISSIVE**: Crate and its dependencies have a permissive license
