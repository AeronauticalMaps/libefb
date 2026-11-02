# Naming Conventions

Names follow the Rust API Guidelines; the `rust-api` skill has them
in depth. On top of those, a name speaks the pilot's language and
tells the cost of a call.

## Vocabulary

- One term per concept, the pilot's own: *route*, *leg*, *fix*,
  *navaid*, *navigation data*, *route prompt*, *flight planning*,
  *fuel planning*, *mass & balance*, *runway analysis*, *vertical
  profile*, *performance*. Reuse the term the code already has.
- Abbreviate as a pilot does, and spell out where a pilot would:
  `nd`, `fp`, `fms`, `mb`, `rwy`, `arpt`, `wp`, `tas`, `gs`, `wca`,
  `mc`, `mh`, `ete`, `ff`, `dist`, `qnh`, `pa`, `toc`, `tod`,
  `airac`. Add a new acronym to the acronym list in the `efb` crate
  docs (`efb/src/lib.rs`).
- Write acronyms in type names as aviation writes them: `FMS`, `CTA`,
  `RWYCC`, `NOTAM`.
- Let the context carry what it already says: `airport.name`,
  `Length::nm`.

## Per Crate

- `arinc424` speaks the spec. Field types and record fields take the
  spec's column abbreviation (`arpt_ident`, `cust_area`, `sec_code`,
  `bdry_via`, `frn`), and the doc comment cites the section:
  `/// 5.213 Controlled Airspace Type (ARSP TYPE)`. A reader checks a
  field against the spec, so the spec's words win over ours.
- `efb` speaks the pilot's domain (`Airport`, `Waypoint`, `Runway`).
  Parser names stay in the parser crate and in `nd/convert/`.
- A measurement constructor is the unit's symbol: `Length::nm`,
  `Speed::kt`, `Mass::kg`, `Duration::s`.

## Cost in the Name

- A cheap accessor is a noun: `legs`, `cycle`, `fuel_planning`.
- Work that is costly or can fail -- a search, a parse, a query --
  is a verb, so the caller sees the cost: `find`, `decode`,
  `accumulate`.
- A conversion's prefix states its cost, as the guidelines define
  it: `as_str` borrows, `to_msl` computes, `into_inner` consumes. A
  fallible constructor from a source format is `try_from_<format>`:
  `try_from_arinc424`.
- A builder is `XBuilder`, reached by `X::builder()`, finished by
  `build(self)`.

## Tests

A test name is a short sentence of the behaviour it checks:
`parses_latitude`, `div_length_by_speed`,
`usable_fuel_matches_tank_capacity`.
