# Reading POH Performance Charts

POH performance *charts* (not tables) are graphical nomograms. This document describes how to read values off them reliably enough to transcribe into libefb types.

## Anatomy of a typical POH chart

A takeoff / landing chart usually has:

- **Entry axis** (left): pressure altitude, sometimes with overlaid OAT or density altitude curves.
- **Reference lines** (vertical or slanted): successive correction steps — e.g. "weight", "wind", "obstacle". Each reference line introduces a new family of guide curves.
- **Guide curves**: sloped families of curves between grid lines, each labelled with an input value (weight, headwind, etc.).
- **Exit axis** (right): the output — usually ground roll or distance to clear 50 ft.

A climb chart is simpler:

- **Y axis**: pressure altitude.
- **Curves**: one per temperature condition (e.g. ISA, ISA+20 °C). Sometimes a single curve with a small correction note.
- **X axis**: rate of climb, or (for "time/fuel/distance to climb" charts) minutes / gallons / NM.

A cruise chart typically has:

- **Y axis**: pressure altitude.
- **Families of curves**: one per power setting (55 %, 65 %, 75 % BHP).
- **X axis**: TAS and, on a parallel chart or second panel, fuel flow.

## Chart-reading procedure

### 1. Lock the known inputs

Before tracing, list every input the chart takes (altitude, OAT, weight, wind, …) and the value sought for each. If an input is missing from the image, pick the most conservative value printed on the chart (e.g. "no wind" on a wind reference line gives the longest distance).

### 2. Enter on a grid line, not between them

Always enter the chart at a printed grid-line value of the entry axis — e.g. PA = 2 000 ft, not 2 350 ft. Sample at a regular grid and let libefb's `at_level` reverse-find pick the conservative row for in-between altitudes.

### 3. Follow the guide curve, not a straight line

Between reference lines, follow the slope of the nearest guide curve, staying parallel to it. Never draw a straight horizontal line through a sloped family of curves — that loses the correction encoded in the slope.

If the entry value falls between two labelled guide curves (e.g. weight = 1 050 kg, curves labelled 1 000 and 1 100 kg), follow a curve that maintains the same proportional position between the two — 50 % of the way across for this example.

### 4. Cross each reference line; re-enter the next family

At each reference line, draw a vertical (or perpendicular) line onto the reference line, then pick up the next family of guide curves and follow them until the next reference line. Repeat until the exit axis.

### 5. Read the exit value at a grid line

At the exit axis, read the value to the nearest grid tick or 10 % of the finest division — whichever is coarser. Do not claim sub-tick precision that the chart does not support.

## Sampling strategy for transcription

To turn a chart into a libefb data table, sample at a grid that covers the operational envelope the user cares about.

**Climb / descent**: sample pressure altitude at 0, 2 000, 4 000, 6 000, 8 000 ft (and the ceiling, if below 10 000 ft). At each altitude record TAS, rate of climb, and fuel flow (or cumulative time/fuel/distance).

**Cruise**: for each power setting the user wants represented, sample pressure altitude at 2 000 ft intervals from sea level to the chart's top. Record TAS and fuel flow. Produce one `Performance` instance per power setting.

**Takeoff / landing**: sample pressure altitude at 0, 2 000, 4 000, 6 000, 8 000 ft × temperature at 0 °C, 10 °C, 20 °C, 30 °C, 40 °C (or the printed grid, whichever is coarser). Produce the full cartesian product — libefb expects the 2-D grid.

Reduce the grid only when the chart itself is printed at a coarser grid (e.g. PA every 4 000 ft).

## Conservative rounding

POH performance data is already optimistic (new aircraft, test pilot, clean wings). When the chart forces a choice:

- **Takeoff / landing distance**: round *up* to the next printed tick.
- **Rate of climb**: round *down*.
- **TAS**: round *down*.
- **Fuel flow**: round *up*.
- **Time / fuel / distance to climb**: round each *up*.

Call out in the output summary any cell where rounding moved the value by more than one tick.

## Image quality triage

Before starting, assess the image:

- **Grid lines unreadable** (blurry scan, compression artefacts): ask for a sharper image rather than guessing.
- **Cropped axis labels**: if the units or top of the scale is cut off, ask the user to confirm rather than inferring.
- **Overlaid annotations**: if a cell is obscured by someone's pen marks, skip the row and note it.

It's better to transcribe fewer, accurate rows than a complete table with guessed values.
