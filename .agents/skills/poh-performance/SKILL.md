---
name: poh-performance
description: This skill should be used when the user provides one or more POH/AFM chart or table images and asks to transcribe them into libefb performance-type Rust literals (climb, cruise, descent, takeoff, or landing). It converts chart curves or tabular rows into `ClimbDescentBand`, `CumulativeClimbDescentEntry`, `PerformanceTableRow`, or `TakeoffLandingPerformance` table data using the correct measurement constructors.
---

# POH Performance Extraction

Transcribe aircraft performance data from POH / AFM chart or table images into libefb performance-type Rust literals. Output only the data literal(s) — no wrapping module, binary, or example scaffolding — so the user can paste into existing code.

## When to invoke

Invoke when the user:

- Provides an image of a POH/AFM **performance chart** (graphical nomogram with grid lines and guide curves) or **performance table** (discrete rows of values), and
- Asks to produce libefb performance data for one of: climb, cruise, descent, takeoff, or landing.

Do not invoke for aircraft weight & balance data, fuel planning scenarios, or navigation tables — those use different libefb types.

## Workflow

Follow these steps in order for every invocation.

### 1. Classify the chart

Determine which performance phase the image documents. Each maps to one libefb type:

| Phase              | libefb type                                                                         |
|--------------------|-------------------------------------------------------------------------------------|
| Climb              | `ClimbDescentPerformance` (via `ClimbDescentBand` or `CumulativeClimbDescentEntry`) |
| Cruise             | `Performance` (via `PerformanceTableRow`)                                           |
| Descent            | `ClimbDescentPerformance` (same shape as climb)                                     |
| Takeoff / Landing  | `TakeoffLandingPerformance` via `Vec<(VerticalDistance, Temperature, Length, Length)>` |

If ambiguous (e.g. a "cruise performance" chart that also lists climb data), ask the user which sub-chart to transcribe.

### 2. Identify the chart's axes and conditions

Before reading a single number, record:

- **Independent variables**: pressure altitude, outside air temperature, weight, wind, runway slope, flap setting, etc.
- **Dependent variables**: ground roll, distance to clear 50 ft, TAS, fuel flow, vertical rate, time/fuel/distance to climb.
- **Conditions** printed on the chart: ISA/non-ISA, gross weight, flap setting, runway condition, power setting. These go into the `notes: Option<String>` for takeoff/landing types, or as a Rust comment above the data for climb/cruise/descent.
- **Units** printed on the chart: knots vs. mph, ft vs. m, gal vs. l, °C vs. °F. Keep the POH units and use the matching `Speed::kt`, `Length::ft`, `Volume::gal`, `Temperature::c`, etc. constructor — do not convert manually.

### 3. Read values

For **tables**: Transcribe rows verbatim. Do not interpolate between table rows.

For **charts**: Load `references/chart-reading.md` for the guide-line-following technique. Key points:

- Enter the chart on the leftmost axis at a grid line (e.g. pressure altitude = 2 000 ft, 4 000 ft, …).
- Follow the nearest **guide curve** — usually sloped between grid lines — across each reference line.
- At each reference line, if the value lies between printed curves, follow the slope of the surrounding curves proportionally.
- Sample at regular intervals (typical: every 1 000 or 2 000 ft of pressure altitude, plus ISA and ISA+20 °C for takeoff/landing).
- **Round conservatively** — for takeoff/landing distance round *up*; for climb rate round *down*. Charts are already optimistic data; pick the pessimistic reading within ±1 grid-line precision.

### 4. Map to libefb types

Load `references/efb-types.md` for the exact struct signatures, field semantics, and measurement constructors. Always:

- Use `VerticalDistance::PressureAltitude(ft)` when the chart is indexed by pressure altitude (the usual case for POH charts).
- Use `VerticalDistance::Gnd` (not `PressureAltitude(0)`) for the sea-level baseline when it represents the aircraft on the ground.
- Use the macros `avgas!`, `diesel!`, `jet_a!` (from `efb::prelude::*`) to build `Fuel` from `Volume`.
- Wrap fuel flow in `FuelFlow::PerHour(...)`.
- Keep entries sorted ascending by `level` — libefb's reverse-find relies on this.

### 5. Emit the data

Produce a single Rust literal (no `fn main`, no `mod`, no `use` statements unless the user asks for them) in a fenced `rust` code block. Annotate with a brief leading comment capturing the POH's stated conditions.

Follow the code block with a short bulleted summary covering:

- The sample grid used (e.g. "pressure altitudes: 0, 2 000, 4 000, 6 000, 8 000 ft").
- Any cells where the chart was hard to read or the value was conservatively rounded.
- Any conditions the user should preserve in a surrounding `notes:` field or doc comment.

### 6. Worked examples per phase

For a fully worked transcription example in each of the five phases, load `references/examples.md`. Prefer matching a new task to the closest example rather than inventing a new shape.

## Constraints

- **No interpolation at transcription time.** Transcribe the values the chart actually shows. libefb's `between` / `at_level` methods handle lookup; they intentionally don't interpolate, and adding synthetic intermediate rows defeats that contract.
- **No unit conversion at transcription time.** Store values in the chart's own units. libefb measurement types hold the canonical SI value internally — the constructor handles the conversion.
- **Never invent data.** If a cell is unreadable (blurry image, cropped, annotation covers a value), call it out in the summary and omit that row. Do not extrapolate off the chart.
- **Takeoff/landing units.** POH takeoff/landing tables routinely print in *feet* for distance even when other charts use metres. Use `Length::ft` unless the image unambiguously says metres.
- **Altering factors** (`DecreaseHeadwind`, `IncreaseTailwind`, `IncreaseAltitude`, `IncreaseRWYCC`, `RunwaySlope`, `Mass`) come from *separate* correction tables or notes next to the main takeoff/landing chart. Only emit them if the image shows them; flag explicitly when the chart references factors but the image does not include them.
