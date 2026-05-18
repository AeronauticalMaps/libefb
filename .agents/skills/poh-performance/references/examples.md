# Worked Examples

One fully transcribed example per performance phase. Use as a structural template when producing output for a new image — copy the shape, swap the numbers.

## 1. Climb (cumulative time/fuel/distance table)

**Source**: POH PA-28-181 Archer II, "Time, Fuel, and Distance to Climb" table, ISA, 2 550 lb gross weight, full throttle, 87 KIAS, 100LL.

```rust
// PA-28-181 climb — ISA, 2 550 lb, full throttle, 87 KIAS
let climb_entries = vec![
    CumulativeClimbDescentEntry {
        level: VerticalDistance::Gnd,
        time: Duration::m(0),
        fuel: Volume::gal(0.0),
        distance: Length::nm(0.0),
    },
    CumulativeClimbDescentEntry {
        level: VerticalDistance::PressureAltitude(2_000),
        time: Duration::m(4),
        fuel: Volume::gal(0.9),
        distance: Length::nm(5.0),
    },
    CumulativeClimbDescentEntry {
        level: VerticalDistance::PressureAltitude(4_000),
        time: Duration::m(8),
        fuel: Volume::gal(1.8),
        distance: Length::nm(11.0),
    },
    CumulativeClimbDescentEntry {
        level: VerticalDistance::PressureAltitude(6_000),
        time: Duration::m(13),
        fuel: Volume::gal(2.9),
        distance: Length::nm(18.0),
    },
    CumulativeClimbDescentEntry {
        level: VerticalDistance::PressureAltitude(8_000),
        time: Duration::m(18),
        fuel: Volume::gal(4.1),
        distance: Length::nm(27.0),
    },
];

let climb = ClimbDescentPerformance::from_cumulative(&climb_entries, FuelType::AvGas)
    .expect("valid cumulative climb table");
```

- Sample grid: SL, 2 000, 4 000, 6 000, 8 000 ft PA.
- Values transcribed verbatim; no rounding applied.
- Conditions (ISA, 2 550 lb, full throttle, 87 KIAS) should be preserved in a surrounding comment or doc line on the binding.

## 2. Cruise (power-setting table)

**Source**: POH C172N cruise table, 65 % BHP, standard temperature, 2 300 RPM.

```rust
// C172N cruise — 65 % BHP, 2 300 RPM, std temp
let cruise_65_bhp = Performance::new(vec![
    PerformanceTableRow {
        level: VerticalDistance::PressureAltitude(2_000),
        tas: Speed::kt(108.0),
        ff: FuelFlow::PerHour(avgas!(Volume::gal(7.4))),
    },
    PerformanceTableRow {
        level: VerticalDistance::PressureAltitude(4_000),
        tas: Speed::kt(110.0),
        ff: FuelFlow::PerHour(avgas!(Volume::gal(7.4))),
    },
    PerformanceTableRow {
        level: VerticalDistance::PressureAltitude(6_000),
        tas: Speed::kt(113.0),
        ff: FuelFlow::PerHour(avgas!(Volume::gal(7.4))),
    },
    PerformanceTableRow {
        level: VerticalDistance::PressureAltitude(8_000),
        tas: Speed::kt(115.0),
        ff: FuelFlow::PerHour(avgas!(Volume::gal(7.4))),
    },
]);
```

- Sample grid: 2 000, 4 000, 6 000, 8 000 ft PA.
- Fuel flow is flat at 7.4 gph for 65 % BHP in this POH — don't assume a bug if the column is constant.
- Produce a second `Performance` for 55 %, 75 % etc. if the user needs them.

## 3. Descent (band table)

Descents are rarely printed in full cumulative tables; most POHs state a single descent profile. Transcribe as bands when only TAS / rate / fuel flow are available.

**Source**: Hypothetical descent profile — constant 500 fpm at 130 KTAS, 10 gph avgas, valid up to 10 000 ft.

```rust
// Cruise-descent — 500 fpm, 130 KTAS, 10 gph avgas
let ff = FuelFlow::PerHour(avgas!(Volume::gal(10.0)));
let descent = ClimbDescentPerformance::new(vec![
    ClimbDescentBand {
        level: VerticalDistance::PressureAltitude(10_000),
        tas: Speed::kt(130.0),
        vertical_rate: VerticalRate::fpm(500.0),
        ff,
    },
]);
```

- A single-row table is valid — libefb applies the top row to any altitude ≤ 10 000 ft.
- For a banded descent (different rates per altitude), add more `ClimbDescentBand` rows in ascending `level` order.

## 4. Takeoff (pressure altitude × temperature table)

**Source**: POH takeoff distance table, flaps 10°, gross weight, hard surface, zero wind. Distances in feet (ground roll / distance to clear 50 ft).

```rust
// Takeoff — gross weight, flaps 10°, hard runway, zero wind
let takeoff = TakeoffLandingPerformance::new(
    vec![
        (VerticalDistance::PressureAltitude(0),     Temperature::c(0.0),  Length::ft(735.0),  Length::ft(1_385.0)),
        (VerticalDistance::PressureAltitude(0),     Temperature::c(20.0), Length::ft(825.0),  Length::ft(1_510.0)),
        (VerticalDistance::PressureAltitude(0),     Temperature::c(40.0), Length::ft(920.0),  Length::ft(1_645.0)),
        (VerticalDistance::PressureAltitude(2_000), Temperature::c(0.0),  Length::ft(810.0),  Length::ft(1_530.0)),
        (VerticalDistance::PressureAltitude(2_000), Temperature::c(20.0), Length::ft(910.0),  Length::ft(1_670.0)),
        (VerticalDistance::PressureAltitude(2_000), Temperature::c(40.0), Length::ft(1_015.0), Length::ft(1_825.0)),
        (VerticalDistance::PressureAltitude(4_000), Temperature::c(0.0),  Length::ft(895.0),  Length::ft(1_690.0)),
        (VerticalDistance::PressureAltitude(4_000), Temperature::c(20.0), Length::ft(1_005.0), Length::ft(1_855.0)),
        (VerticalDistance::PressureAltitude(4_000), Temperature::c(40.0), Length::ft(1_125.0), Length::ft(2_030.0)),
    ],
    None, // altering factors omitted — add if the POH provides them
    Some("Gross weight, flaps 10°, hard runway, zero wind".to_string()),
);
```

- Sample grid: PA = 0, 2 000, 4 000 ft × temp = 0, 20, 40 °C (9 rows total).
- Extend the grid if the source image covers higher altitudes or more temperature columns.
- The fourth column (distance to clear 50 ft) is mandatory. If the POH shows only ground roll, repeat it and flag in the summary.

## 5. Landing (with altering factors for wind and runway condition)

**Source**: POH landing distance table plus correction notes. Dry paved runway, flaps full, 2 550 lb.

```rust
// Landing — flaps full, 2 550 lb, dry paved
let landing = TakeoffLandingPerformance::new(
    vec![
        (VerticalDistance::PressureAltitude(0),     Temperature::c(0.0),  Length::ft(520.0), Length::ft(1_250.0)),
        (VerticalDistance::PressureAltitude(0),     Temperature::c(20.0), Length::ft(560.0), Length::ft(1_335.0)),
        (VerticalDistance::PressureAltitude(2_000), Temperature::c(0.0),  Length::ft(560.0), Length::ft(1_340.0)),
        (VerticalDistance::PressureAltitude(2_000), Temperature::c(20.0), Length::ft(605.0), Length::ft(1_430.0)),
        (VerticalDistance::PressureAltitude(4_000), Temperature::c(0.0),  Length::ft(605.0), Length::ft(1_435.0)),
        (VerticalDistance::PressureAltitude(4_000), Temperature::c(20.0), Length::ft(650.0), Length::ft(1_535.0)),
    ],
    Some(AlteringFactors::new([
        AlteringFactor::DecreaseHeadwind(FactorOfEffect::Rate {
            numerator: 0.1,
            denominator: Speed::kt(9.0),
        }),
        AlteringFactor::IncreaseTailwind(FactorOfEffect::Rate {
            numerator: 0.1,
            denominator: Speed::kt(2.0),
        }),
    ])),
    Some("Flaps full, 2 550 lb, dry paved runway".to_string()),
);
```

- Wind corrections are quoted as "decrease 10 % per 9 kt headwind" and "increase 10 % per 2 kt tailwind" — the canonical POH phrasing; map directly to `FactorOfEffect::Rate`.
- Add further factors (`IncreaseRWYCC`, `RunwaySlope`, `Mass`) only if the image includes them.

## Output shape reminder

When emitting data for the user:

1. Leading one-line comment naming the aircraft and POH conditions.
2. A single Rust literal expression (no `fn main`, no `use` lines unless requested).
3. A short bullet summary below the code block listing sample grid, any rounding, and any missing factors.

Example output skeleton:

````markdown
```rust
// <aircraft> <phase> — <conditions>
let <binding> = <Type>::new(vec![
    // …rows…
]);
```

- Sample grid: …
- Rounding: …
- Omitted: altering factors (image did not include the correction table).
````
