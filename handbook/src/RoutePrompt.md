# Route Prompt

The route prompt is composed of space separated _token_ with the
following primary token categories:

- Performance data (cruise speed/level, wind)
- Airports
- Navigation aids

The following example shows a route from KSFO to KSAN in 7000 ft and 107 kt:

    N0107 A070 KSFO KSAN

## Performance Data

### Speed

Speed is entered in the FPL form's format (ICAO Doc. 4444 Annex 2):

- Kilometers per hour, expressed as K followed by 4 figures
  e.g. `K0830` for 830 km/h
- Knots, expressed as N followed by 4 figures e.g. `N0485` for 485 kt
- Mach, expressed as M followed by 3 figures to the nearest hundredth
  of unit Mach e.g. `M082` for 0.82 Mach

### Level

Level is entered in the FPL form's format (ICAO Doc. 4444 Annex 2):

- Flight level, expressed as F followed by 3 figures e.g. `F085` for FL085
- Standard metric level in tens of metres, expressed by S followed by
  4 figures e.g. `S1130` for 11300 m
- Altitude in hundreds of feet, expressed as A followed by 3 figures
  e.g. `A045` for 4500 ft
- Altitude in tens of metres, expressed as M followed by 4 figures
  e.g. `M0840` for 8400 m

### Wind

Wind is entered in the METAR's format:

- E.g. `23008KT` for wind from 230° with a speed of 8 kt

## Airports

- Enter the airport's ICAO identifier e.g. `KJFK`
- Append a runway designator to select a takeoff or landing runway at
  the airport e.g. `KJFK31L`

## Navigation Aids

The following navigation aids are supported within the route prompt:

- VFR Terminal Waypoints
- VFR Enroute Waypoints

### VFR Terminal Waypoints

- Terminal waypoints are referenced by the identifier provided in the
  navigation data
- Terminal waypoints are scoped to their terminal area to avoid ambiguity
- Terminal areas are delimited by the `DCT` via separator

- The `DCT` via can be omit, if the terminal waypoint's identifier is
  exclusive to **only one** of the terminal areas

- If terminal waypoint is entered between two terminal areas and **only
  one** terminal area features a waypoint with the entered identifier,
  the `DCT` via can be omit
- When crossing a terminal area via terminal waypoints **without**
  flying over the airport, the terminal area's scope is opened by
  `DCT` followed by the airport's identifier. For example, crossing
  `EDDV` via `N1`, `N2`, `W2` and `W1` would be `DCT EDDV N1 N2 W2 W1`
  and `EDDV` **will not** occur in the legs

### VFR Enroute Waypoints

- Enroute waypoints are entered by the identifier provided in the navigation data
- Enroute waypoints are ambiguous and the first matching point is selected

## Examples

- `N0107 A025 01005KT EDDH33 P2 EDHF02`
  - Cruise speed 107 kt
  - Cruise altitude 2500 ft
  - Wind 5 kt from 010°
  - Hamburg with takeoff runway 33
  - Compulsory VFR reporting point Papa 2
  - Itzehoe with landing runway 02
