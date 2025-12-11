# Route

## Overview

A route defines the path from an origin to a destination and is composed of individual legs between waypoints. Each leg represents the flight segment from one navigation aid (NavAid) to another.

A route can be composed of the following elements:

- **Wind along route** - Wind direction and speed affecting each leg
- **Cruise Speed & Altitude** - True airspeed (TAS) and flight level
- **Origin** - Departure airport (with optional runway designation)
- **Waypoints** - Intermediate navigation points
- **Destination** - Arrival airport (with optional runway designation)

The wind and cruise speed/altitude can be changed from leg to leg, and the last entered values are applied to all following legs.

## Route Format

Routes are entered as space-separated elements in ICAO format:

```
WIND SPEED ALTITUDE DEPARTURE [WAYPOINTS] DESTINATION
```

### Example Routes

Simple route with basic parameters:
```
29020KT N0107 A0250 EDDH DHN2 DHN1 EDHF
```
This creates a route from Hamburg (EDDH) to Heide (EDHF) via waypoints DHN2 and DHN1, with:
- Wind: 290° at 20 knots
- True airspeed: 107 knots
- Altitude: 2,500 feet

Route with runway specifications:
```
29020KT N0107 A0250 EDDH RWY33 EDHF RWY20
```
This specifies takeoff from runway 33 at EDDH and landing on runway 20 at EDHF.

Route with changing wind conditions:
```
13509KT N0107 EDDH DHD 18009KT HLW EDHL
```
Wind changes from 135° at 9 knots for the first leg to 180° at 9 knots for subsequent legs.

## Route Editing

### Adding Elements

Route elements can be added to the end of the route or inserted at specific positions between existing waypoints.

### Route Elements

The following element types can be added:
- **Speed** - True airspeed (e.g., N0107 for 107 knots)
- **Level** - Flight level or altitude (e.g., A0250 for 2,500 feet, FL065 for flight level 65)
- **Wind** - Wind direction and speed in METAR format (e.g., 29020KT)
- **NavAid** - Navigation aid identifier (waypoint or airport)
- **RunwayDesignator** - Runway identifier following an airport (e.g., RWY33)

### Setting Cruise Parameters

Cruise speed and altitude can be set or modified at any point along the route. Setting either parameter to blank removes it from the route.

### Alternates

An alternate destination can be specified, and the system automatically creates a leg from the destination to the alternate using the final leg's performance parameters.

## Leg Calculations

Each leg between two navigation points is computed with the following parameters:

### Basic Parameters

- **Bearing (BRG)** - True bearing from start to end point
- **Magnetic Course (MC)** - Bearing adjusted for magnetic variation at the starting point
- **Distance (DIST)** - Great circle distance between points in nautical miles

### Wind Correction Calculations

When wind and true airspeed are provided, the following values are calculated:

#### Wind Correction Angle (WCA)

The wind correction angle compensates for wind drift to maintain the desired track. It is calculated using the law of sines:

$$
\sin(\text{WCA}) = \frac{WS}{TAS} \times \sin(\alpha)
$$

where:
- $WS$ = wind speed
- $TAS$ = true airspeed
- $\alpha$ = angle between wind direction and bearing

The angle $\alpha$ is calculated as:
$$
\alpha = \text{BRG} - (\text{Wind Direction} + 180°)
$$

**Sign Convention:**
- Positive WCA: correction to the right (wind from left)
- Negative WCA (displayed as 360° - WCA): correction to the left (wind from right)

#### True Heading (HDG)

The true heading is the bearing corrected for wind drift:

$$
\text{HDG} = \text{BRG} + \text{WCA}
$$

#### Magnetic Heading (MH)

The magnetic heading accounts for magnetic variation at the departure point:

$$
\text{MH} = \text{HDG} + \text{Magnetic Variation}
$$

This is the heading to fly on the magnetic compass.

#### Ground Speed (GS)

Ground speed is calculated using the law of cosines:

$$
GS = \sqrt{TAS^2 + WS^2 - 2 \times TAS \times WS \times \cos(\beta)}
$$

where:
$$
\beta = \text{BRG} - \text{Wind Direction} + \text{WCA}
$$

**Physical Interpretation:**
- Headwind component reduces ground speed
- Tailwind component increases ground speed
- Crosswind component has minimal effect on ground speed but requires heading correction

### Time and Fuel Calculations

#### Estimated Time Enroute (ETE)

Time for each leg is calculated from distance and ground speed:

$$
\text{ETE} = \frac{\text{DIST}}{GS}
$$

#### Fuel Consumption

Fuel consumed on a leg depends on the fuel flow at the cruise altitude and the time enroute:

$$
\text{Fuel} = \text{Fuel Flow}(\text{altitude}) \times \text{ETE}
$$

The fuel flow is retrieved from the performance profile for the leg's altitude.

## Totals and Accumulation

The route provides progressive totals through the `accumulate_legs()` method, which yields cumulative values for each leg:

- **Total Distance** - Sum of all leg distances from start
- **Total Time** - Sum of all leg ETEs (if all legs have ETE data)
- **Total Fuel** - Sum of all leg fuel consumption (if performance profile provided)

**All-or-Nothing Principle:** If any leg is missing ETE or fuel data, cumulative ETE/fuel will be `None` for that leg and all subsequent legs to ensure data consistency.

### Route Totals

The `totals()` method returns the final accumulated values for the entire route, providing:
- Total route distance
- Total estimated time
- Total fuel required (if performance data available)

These totals are essential for flight planning and checking fuel requirements against aircraft capacity.

## Wind

Wind is entered in METAR format:

```
DDDddKT
```

where:
- `DDD` = wind direction in degrees (true)
- `dd` = wind speed in knots
- `KT` = knots unit identifier

Examples:
- `36010KT` - Wind from north (360°) at 10 knots
- `09005KT` - Wind from east (090°) at 5 knots
- `18020KT` - Wind from south (180°) at 20 knots

## Magnetic Variation

Magnetic variation is automatically retrieved from the World Magnetic Model (WMM) at the starting point of each leg. The variation is added to true values to obtain magnetic values:

- Easterly variation (positive): adds to true heading
- Westerly variation (negative): subtracts from true heading

This ensures all magnetic headings and courses reflect the local magnetic environment.
