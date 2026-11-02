# Architecture

The boundaries below are fixed. A change that needs a new crate,
module boundary or dependency goes to the human as a question.

## Crates

- `arinc424` and `aixm` parse one source format each and stand
  alone, without `efb`. `arinc424-derive` generates `Record` impls
  from field attributes (`#[arinc424(skip(n))]`,
  `#[arinc424(field = n)]`).
- `efb` owns the domain model. `efb/src/nd/convert/` turns parser
  types into domain types with `TryFrom`, one submodule per format.
- `bindings/` wrap `efb` and carry no logic of their own; a binding
  follows the core change it exposes.

## The `efb` Crate

- `FMS` is the single entry point and owns the navigation data, the
  route and the flight planning. Every change goes through its
  `modify_*` and `set_*` methods, which re-run the evaluation
  pipeline from the affected stage.
- The layers build on each other: `measurements` and `core` give the
  value types; `nd` holds the navigation data with its R-tree
  indexes; `route` decodes the route prompt into legs against `nd`;
  `fp` plans on top of a route and an aircraft.
- A complex type is built with a builder (`AircraftBuilder`,
  `FlightPlanningBuilder`, `NavigationDataBuilder`).
- Absent data is `Option`. Every failure surfaces: a fallible call
  returns `efb::error::Error`, and data errors met while loading are
  collected in `NavigationData::errors`.
- `efb::prelude` re-exports the commonly used types.
- An optional dependency sits behind a feature (`serde`, `sqlite`,
  `geojson`, `handbook`), so the default build stays lean.
- A type that crosses the C boundary is `#[repr(C)]`.

## Logging

The `log` crate, by level: `info!` for an operation a user starts,
`debug!` for its outcome, `trace!` per item in a loop, `warn!` for
skipped data.
