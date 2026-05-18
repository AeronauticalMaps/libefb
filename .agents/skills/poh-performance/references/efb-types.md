# libefb Performance Types Reference

Canonical struct and constructor signatures for libefb performance types. All imports assumed from `efb::prelude::*`.

## Cruise — `Performance` / `PerformanceTableRow`

```rust
pub struct PerformanceTableRow {
    pub level: VerticalDistance,
    pub tas: Speed,
    pub ff: FuelFlow,
}

pub type PerformanceTable = Vec<PerformanceTableRow>;

pub struct Performance { /* wraps PerformanceTable */ }
impl Performance {
    pub fn new(table: PerformanceTable) -> Self;
}
```

- Rows must be **sorted ascending by `level`**; `at_level` uses reverse-find and returns the highest row ≤ the target.
- No interpolation — a value at 5 500 ft with rows at 5 000 and 7 000 returns the 5 000 ft row.
- Typical cruise chart source: power-setting table (e.g. 65 % / 75 % RPM) → one `Performance` instance per setting.

## Climb / Descent — `ClimbDescentPerformance`

Two construction paths; use whichever matches the POH format.

### Option A — band-per-row (`ClimbDescentBand`)

Use when the chart prints **rate of climb / TAS / fuel flow** directly per altitude band.

```rust
pub struct ClimbDescentBand {
    pub level: VerticalDistance,       // upper bound of this band
    pub tas: Speed,
    pub vertical_rate: VerticalRate,   // always positive
    pub ff: FuelFlow,
}

impl ClimbDescentPerformance {
    pub fn new(table: Vec<ClimbDescentBand>) -> Self;
}
```

- `level` is the **upper** bound of the band; the band starts at the previous row's level (or ground for the first row).
- `vertical_rate` is always positive — direction (climb vs. descent) is implied by which table the instance is passed into.

### Option B — cumulative time/fuel/distance (`CumulativeClimbDescentEntry`)

Use when the POH prints a **"Time, Fuel, and Distance to Climb"** table — the most common POH format.

```rust
pub struct CumulativeClimbDescentEntry {
    pub level: VerticalDistance,
    pub time: Duration,      // cumulative from baseline
    pub fuel: Volume,        // cumulative; Fuel type passed separately
    pub distance: Length,    // cumulative still-air distance
}

impl ClimbDescentPerformance {
    pub fn from_cumulative(
        entries: &[CumulativeClimbDescentEntry],
        fuel_type: FuelType,
    ) -> Option<Self>;
}
```

- First entry is the baseline (sea level, all values zero).
- Each subsequent row's `time`, `fuel`, `distance` are **cumulative from the baseline**, exactly as printed in the POH.
- Requires ≥ 2 rows and strictly increasing `level` and `time`.
- Fuel is transcribed as volume (e.g. `Volume::gal(0.9)`) — libefb converts to mass via `fuel_type`.

## Takeoff / Landing — `TakeoffLandingPerformance`

```rust
impl TakeoffLandingPerformance {
    pub fn new(
        table: Vec<(VerticalDistance, Temperature, Length, Length)>,
        factors: Option<AlteringFactors>,
        notes: Option<String>,
    ) -> Self;

    // Builder form:
    pub fn builder<I>(table: I) -> TakeoffLandingPerformanceBuilder
    where I: IntoIterator<Item = (VerticalDistance, Temperature, Length, Length)>;
}
```

Tuple is `(pressure_altitude, temperature, ground_roll, distance_to_clear_50ft)`.

- Rows cover a 2-D grid of pressure altitude × temperature. Include **every** printed (PA, temp) pair.
- Lookup is "next higher or equal" on both axes — so ordering within the `Vec` is not structurally required, but keep it sorted by PA ascending then temperature ascending for readability.
- If the chart shows only *ground roll*, repeat it in the fourth position so both columns are populated; flag in the summary.

### Altering factors — separate correction table

Only emit if the image includes the correction table.

```rust
pub enum AlteringFactor {
    DecreaseHeadwind(FactorOfEffect<Speed>),
    IncreaseTailwind(FactorOfEffect<Speed>),
    IncreaseAltitude(FactorOfEffect<VerticalDistance>),
    IncreaseRWYCC(HashMap<(Option<RunwayConditionCode>, Option<RunwaySurface>), f32>),
    RunwaySlope(FactorOfEffect<f32>),
    Mass(FactorOfEffect<Mass>),
}

pub enum FactorOfEffect<T> {
    Range(Vec<(RangeToInclusive<T>, f32)>),
    Rate { numerator: f32, denominator: T },
}
```

- `DecreaseHeadwind(Rate { numerator: 0.1, denominator: Speed::kt(9.0) })` = "decrease by 10 % per 9 kt headwind".
- `IncreaseTailwind(Rate { numerator: 0.1, denominator: Speed::kt(2.0) })` = "increase by 10 % per 2 kt tailwind".
- `Range(...)` models banded factors — each tuple is `(..=upper_bound, factor)`, covering all lower values.

## Measurement constructors cheat sheet

| Type              | Constructor                                                | Example                                        |
|-------------------|------------------------------------------------------------|------------------------------------------------|
| `VerticalDistance`| `::Gnd`, `::Altitude(u16)`, `::PressureAltitude(i16)`, `::Fl(u16)`, `::Msl(u16)`, `::Agl(u16)` | `VerticalDistance::PressureAltitude(6_000)`    |
| `Speed`           | `::kt(f32)`, `::mps(f32)`, `::mach(f32)`                   | `Speed::kt(107.0)`                             |
| `VerticalRate`    | `::fpm(f32)`, `::mps(f32)`                                 | `VerticalRate::fpm(650.0)`                     |
| `Length`          | `::ft(f32)`, `::nm(f32)`, `::m(f32)`, `::cm(f32)`, `::sm(f32)` | `Length::ft(1_500.0)`                      |
| `Temperature`     | `::c(f32)`, `::f(f32)`                                     | `Temperature::c(20.0)`                         |
| `Duration`        | `::s(u32)`, `::m(u32)`                                     | `Duration::m(8)`                               |
| `Volume`          | `::l(f32)`, `::gal(f32)`, `::cubic_m(f32)`                 | `Volume::gal(1.8)`                             |
| `Mass`            | `::kg(f32)`, `::lb(f32)`                                   | `Mass::kg(1_055.0)`                            |
| `Pressure`        | `::STD`, others as needed                                  | `Pressure::STD`                                |
| `Fuel`            | `Fuel::new(Mass, FuelType)`, `Fuel::from_volume(Volume, FuelType)`; macros `avgas!`, `diesel!`, `jet_a!` | `avgas!(Volume::gal(9.5))` |
| `FuelFlow`        | `FuelFlow::PerHour(Fuel)`                                  | `FuelFlow::PerHour(avgas!(Volume::l(20.0)))`   |

### Numeric literals

- Prefer underscored literals for large values: `6_000`, `1_500.0`.
- Keep one decimal even when the POH prints an integer TAS/fuel-flow (e.g. `Speed::kt(85.0)`) to match the `f32` constructors.
- `Duration::s` and `::m` take `u32`. `Duration::m(4)` = 4 minutes; don't pass `4.0`.
- `VerticalDistance::PressureAltitude` takes `i16` (negative values are legal for below-sea-level airports).

## Gotchas

- **`Duration::m` is minutes**, not metres — don't confuse with `Length::m`.
- **`Temperature::f` is Fahrenheit**, not a free `f32` field.
- **`VerticalDistance::Altitude` vs. `::PressureAltitude`**: POH charts almost always use pressure altitude. Only pick `::Altitude` if the chart explicitly says "indicated altitude" or "MSL".
- **`FuelFlow`** is an enum with only `PerHour` — every fuel flow must be wrapped, not bare `Fuel`.
- **`from_cumulative` requires `Volume`** for fuel even if the POH prints pounds — convert at transcription time to a `Volume::gal` / `Volume::l` value using the POH's own volume column, do *not* synthesise a volume from mass.
