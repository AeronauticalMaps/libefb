# Performance Profiles

## Overview

Performance profiles provide the data necessary to calculate fuel consumption, airspeeds, and takeoff/landing distances for an aircraft under various conditions. The library supports two types of performance profiles:

1. **Cruise Performance** - True airspeed and fuel flow at different altitudes
2. **Takeoff/Landing Performance** - Ground roll and obstacle clearance distances

## Cruise Performance

Cruise performance defines how the aircraft performs during enroute flight at different flight levels.

### Performance Table

A performance profile consists of a table with rows containing:

- **Level** - Altitude or flight level
- **True Airspeed (TAS)** - Cruise speed at this level
- **Fuel Flow (FF)** - Fuel consumption rate at this level

### Creating Performance Profiles

Performance profiles can be created in two ways:

#### Manual Table Definition

A performance table can be defined directly with specific values at different altitudes. Each entry specifies the altitude, true airspeed, and fuel flow at that altitude.

**Example:** A table might include entries for ground level, 1000 ft, 2000 ft, 3000 ft, continuing up to the aircraft's service ceiling, with corresponding TAS and fuel flow values from the POH.

#### Function-Based Generation

Performance tables can also be generated automatically using data from POH performance charts. The data is sampled at regular altitude intervals (typically 1000 ft) from ground level to the service ceiling.

**Use cases:**
- Interpolating from POH tables
- Applying environmental corrections
- Modeling theoretical performance

### Performance Lookup

When calculating leg performance, the system looks up the appropriate performance values for the leg's altitude.

**Lookup Method:** Conservative (non-interpolating)
- Finds the highest table entry at or below the target altitude
- Returns performance for that altitude
- Does NOT interpolate between table entries

**Rationale:** Using the next lower altitude's performance is conservative for fuel planning, as performance typically improves with altitude (lower fuel flow).

### Fuel Flow Retrieval

For a given altitude, the system retrieves the corresponding fuel flow from the performance table. This is used in leg fuel calculations:

$$
\text{Leg Fuel} = \text{Fuel Flow}(\text{altitude}) \times \text{ETE}
$$

### True Airspeed Retrieval

For a given altitude, the system retrieves the corresponding true airspeed from the performance table. While typically specified directly in the route, TAS can also be retrieved from the performance profile for standard configurations.

## Takeoff and Landing Performance

Takeoff and landing performance predicts required distances under various conditions. This is critical for runway analysis and safety margins.

### Performance Table Structure

A takeoff/landing performance table contains entries with:

- **Pressure Altitude** - Airport elevation corrected for pressure
- **Temperature** - Outside air temperature
- **Ground Roll** - Distance to liftoff/touchdown
- **Clear Obstacle** - Distance to clear a 50 ft obstacle

### Table Lookup Strategy

The system uses a **conservative lookup** approach:

1. **Pressure Altitude Selection:**
   - Finds the table entry at or above the actual pressure altitude
   - Selects the closest higher altitude if actual altitude is between entries

2. **Temperature Selection:**
   - From entries matching the pressure altitude criteria, finds temperature at or above actual temperature
   - Selects the closest higher temperature if actual temperature is between entries

**Rationale:** Since performance degrades with higher altitude and temperature, selecting the next higher value provides a conservative (longer) distance estimate.

### Example Lookup

Given a performance table with entries at sea level (0°C and 40°C) and at 8000 ft (0°C and 30°C):

For an airport at 1000 ft pressure altitude and 20°C:
- The system selects the 8000 ft, 30°C entry (next higher altitude and temperature)
- This returns the most conservative (longest) distances
- Example result: 2300 ft ground roll, 4800 ft over obstacle

This conservative approach ensures adequate runway length under actual conditions.

## Altering Factors

Altering factors modify the base performance to account for real-world conditions not captured in the performance table. These factors are multipliers applied to the ground roll and obstacle clearance distances.

### Types of Factors

#### Rated Factors

Applied at a fixed rate regardless of condition value:

**Example:** Grass runway increases distances by 20%
```
factor = 1.20
```

#### Value-Range Factors

Applied proportionally based on the magnitude of a condition:

**Example:** Tailwind increases distance by 10% per knot
```
factor = 1.0 + (0.10 × tailwind_knots)
```

### Common Altering Factors

From Pilot Operating Handbooks (POH), typical factors include:

#### Wind Factors

- **Headwind:** Decreases distances (factor < 1.0)
  - Typical: -10% per 5 knots headwind component

- **Tailwind:** Increases distances (factor > 1.0)
  - Typical: +10% per 2 knots tailwind component
  - **Critical:** Even small tailwinds significantly degrade performance

#### Surface Conditions

- **Grass runway:** +20% to +30%
- **Wet grass:** +25% to +40%
- **Soft field:** +25% to +45%
- **Snow/slush:** +40% to +60%

Surface effects are often combined with Runway Condition Code (RWYCC) for more precise estimation.

#### Runway Slope

- **Upslope (takeoff/landing uphill):** Increases distances
  - Typical: +10% per 1% gradient

- **Downslope (takeoff/landing downhill):** Decreases distances
  - Typical: -10% per 1% gradient
  - **Caution:** Downslope landings reduce stopping ability

#### Weight

- **Below reference weight:** Decreases distances
- **Above reference weight:** Increases distances

If POH performance is given for maximum takeoff weight (MTOW), lighter configurations require less distance.

**Typical:** ±5% per 100 kg deviation from reference weight

### Factor Application

Factors are applied sequentially to the base performance:

$$
\text{Final Distance} = \text{Base Distance} \times f_1 \times f_2 \times ... \times f_n
$$

where each $f_i$ is an altering factor.

**Order of application:** Factors are applied in the order configured, allowing predictable and repeatable calculations.

### Configuring Factors

Takeoff and landing performance is configured by specifying the base performance table and then adding each altering factor in sequence. Common factors include headwind/tailwind adjustments, surface condition factors, and slope corrections.

### Separate Factors for Ground Roll vs. Obstacle

Factors can be applied differently to:
- **Ground Roll** - Distance to liftoff/touchdown
- **Clear Obstacle** - Distance to clear 50 ft

Some factors (like headwind) affect both equally, while others (like certain flap settings) may primarily affect obstacle clearance.

## Flight Planning Factors

Beyond POH factors, additional safety factors can be applied during flight planning to meet regulatory or operational requirements.

### National Recommendations

Different countries mandate safety factors for general aviation:

**Example: Germany (FSM 3/75)**
- Takeoff: +15% for private operations
- Landing: +43% for private operations

These are applied AFTER POH altering factors:

$$
\text{Planning Distance} = \text{Influenced Distance} \times \text{Safety Factor}
$$

### Factor Configuration

Flight planning factors are configured separately from POH factors and applied in order after all POH factors.

**Typical sequence:**
1. Base performance from table (PA and temperature)
2. POH altering factors (wind, surface, slope, weight)
3. Flight planning safety factors (regulatory requirements)

## Influences

The `Influences` structure collects all conditions affecting performance:

- **Temperature** - Outside air temperature
- **Pressure Altitude** - Field elevation corrected for barometric pressure
- **Wind** - Wind speed and direction relative to runway
- **Runway Surface** - Type and condition
- **Runway Slope** - Gradient percentage
- **Aircraft Weight** - Actual weight vs. reference weight

These influences are passed to the performance system, which:
1. Looks up base performance for temperature and pressure altitude
2. Applies altering factors based on other influences
3. Returns predicted distances

## Minimum Distance Calculation

The complete calculation flow:

1. **Lookup base performance**
   - Find conservative entry from table based on PA and temperature
   - Retrieve base ground roll and obstacle clearance

2. **Apply POH factors**
   - Calculate factor for each influence (wind, surface, slope, etc.)
   - Multiply distances by each factor in sequence

3. **Apply planning factors** (if configured)
   - Apply regulatory safety margins
   - Apply operational safety margins

4. **Return predicted distances**
   - Minimum ground roll required
   - Minimum distance to clear 50 ft obstacle

### Formula

$$
\text{Distance}_{\text{final}} = \text{Distance}_{\text{base}}(PA, T) \times \prod_{i=1}^{n} f_{\text{POH},i} \times \prod_{j=1}^{m} f_{\text{planning},j}
$$

where:
- $\text{Distance}_{\text{base}}$ is from the performance table
- $f_{\text{POH},i}$ are POH altering factors
- $f_{\text{planning},j}$ are flight planning safety factors

## Performance Notes

Each performance profile can include notes documenting:

- **Reference source** - POH section and date
- **Reference conditions** - Weight, configuration, procedures
- **Limitations** - Any restrictions on applicability
- **Assumptions** - Pilot technique, aircraft condition

**Example notes:**
- Based on POH Section 5, Rev. 2023-04-15
- Reference weight: 1111 kg (MTOW)
- Short field technique, flaps 30°
- Assumes paved, level, dry runway
- Sea level ISA conditions for base performance

Notes ensure users understand the context and limitations of performance data.

## Practical Application

### For Cruise Performance

1. Create performance table from POH cruise performance charts
2. Enter data for relevant power settings (e.g., 75% power, economy cruise)
3. Include entries from ground level to service ceiling in 1000 ft intervals
4. Use in route planning for fuel calculations

### For Takeoff/Landing Performance

1. Create performance table from POH takeoff/landing charts
2. Configure altering factors per POH recommendations
3. Add flight planning factors per regulations
4. Use in runway analysis before flight

### Performance Validation

Always validate performance calculations:
- Compare results with manual POH calculations
- Verify factor application matches POH guidance
- Cross-check fuel calculations with actual consumption
- Update performance profiles based on operational experience

## Conservative Planning

The system uses conservative principles throughout:

- **Altitude lookup:** Next higher altitude if between entries
- **Temperature lookup:** Next higher temperature if between entries
- **Fuel planning:** Adequate reserves for leg plus alternates
- **Distance planning:** Safety margins for runway operations

This ensures safe operations even with minor deviations from planned conditions.
