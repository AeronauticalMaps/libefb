# Aircraft Configuration

## Overview

The aircraft configuration provides all information necessary for fuel and mass & balance planning. The aircraft is defined in its empty configuration, and additional masses are loaded at specific stations during flight planning.

## Aircraft Components

### Basic Information

Each aircraft is defined by:

- **Registration** - Unique aircraft registration (tail number)
- **ICAO Type** - Aircraft type designator according to ICAO Doc. 8643
- **Empty Mass** - Mass of the empty aircraft from the last mass and balance report
- **Empty Balance** - Center of gravity (CG) of the empty aircraft
- **Fuel Type** - Type of fuel used (Avgas 100LL, Diesel, Jet A1, etc.)
- **Notes** - Optional notes about the aircraft configuration

### Stations

Stations are positions in the aircraft where mass can be loaded. Each station is defined by:

- **Arm** - Distance from the reference datum (in meters or inches)
- **Description** - Optional label (e.g., "front seats", "baggage compartment")

Common stations include:
- Pilot and passenger seats
- Baggage compartments
- Cargo areas

**Important:** Fuel tanks are NOT defined as stations. They are handled separately through the fuel tank system.

### Fuel Tanks

Fuel tanks represent the aircraft's fuel capacity. Each tank is defined by:

- **Capacity** - Usable fuel volume (in liters or US gallons)
- **Arm** - Distance from reference datum to the tank's center

During mass and balance calculations, fuel tanks are automatically converted to stations with appropriate mass based on fuel volume and density.

### Center of Gravity Envelope

The CG envelope defines the safe operating limits for the aircraft's center of gravity. It is represented as a polygon in a mass-versus-balance coordinate system.

The envelope is defined by a series of CG limits, where each limit specifies:
- **Mass** - Aircraft mass at this limit point
- **Distance** - Allowed CG position at this mass

The aircraft is considered balanced if the CG on ramp AND after landing both fall within this envelope.

## Mass and Balance Calculations

### Basic Principles

The mass and balance system calculates:

1. **Total Mass** - Sum of all loaded stations
2. **Center of Gravity** - Balance point based on moments

These calculations are performed for two critical phases:
- **On Ramp** - Before flight with full passenger load and starting fuel
- **After Landing** - After flight with remaining fuel

### Moment Calculation

The moment for each station is:

$$
\text{Moment} = \text{Mass} \times \text{Arm}
$$

### Total Mass

Total mass is the sum of all station masses:

$$
\text{Total Mass} = \sum_{i=1}^{n} \text{Mass}_i
$$

This includes:
- Empty aircraft mass
- Passenger and cargo masses at stations
- Fuel masses in all tanks

### Center of Gravity (Balance)

The CG is calculated from the total moment divided by total mass:

$$
\text{CG} = \frac{\sum_{i=1}^{n} (\text{Mass}_i \times \text{Arm}_i)}{\text{Total Mass}}
$$

This calculation is performed separately for the on-ramp and after-landing conditions.

### Loading Stations

When performing mass and balance calculations, you must provide:

1. **Mass at each station** - Vector of masses mapped to stations by index
2. **Fuel in each tank** - Vector of fuel quantities mapped to tanks by index

**Critical Requirement:** The number of masses must match the number of stations, and the number of fuel quantities must match the number of tanks. If these don't match, the calculation will fail with an error.

## Mass and Balance Methods

The library provides three methods for mass and balance calculations with varying levels of detail:

### Full Mass and Balance

The `mb()` method provides complete control:

```
mb(mass_on_ramp, mass_after_landing, fuel_on_ramp, fuel_after_landing)
```

**Parameters:**
- `mass_on_ramp` - Vector of masses at each station before flight
- `mass_after_landing` - Vector of masses at each station after flight
- `fuel_on_ramp` - Vector of fuel in each tank before flight
- `fuel_after_landing` - Vector of fuel in each tank after landing

**Use when:** Fuel is distributed unevenly across tanks, or passenger/cargo mass changes during flight.

### Equally Distributed Fuel

The `mb_from_equally_distributed_fuel()` method simplifies fuel distribution:

```
mb_from_equally_distributed_fuel(mass_on_ramp, mass_after_landing, total_fuel_on_ramp, total_fuel_after_landing)
```

This method automatically distributes the total fuel equally across all tanks.

**Use when:** The aircraft has multiple tanks but fuel distribution is symmetric or doesn't need detailed tracking.

### Constant Mass with Equal Fuel Distribution

The `mb_from_const_mass_and_equally_distributed_fuel()` method assumes passenger/cargo mass doesn't change:

```
mb_from_const_mass_and_equally_distributed_fuel(mass, fuel_on_ramp, fuel_after_landing)
```

**Use when:** No passengers/cargo are added or removed during flight (typical general aviation scenario).

## Fuel Capacity Validation

The system enforces fuel capacity limits:

- **On Ramp:** Fuel loaded must not exceed tank capacity
- **After Landing:** Remaining fuel must not exceed tank capacity

If either limit is exceeded, the calculation fails with an appropriate error:
- `ExceededFuelCapacityOnRamp`
- `ExceededFuelCapacityAfterLanding`

## Fuel Types and Density

Different fuel types have different densities, affecting mass calculations:

| Fuel Type | Density |
|-----------|---------|
| Avgas 100LL | ~0.72 kg/L |
| Diesel/Jet A1 | ~0.82 kg/L |

The fuel type is specified in the aircraft configuration and automatically used to convert fuel volume to mass for CG calculations.

## Balance Verification

After calculating mass and balance, verify the aircraft is within limits:

```
is_balanced(mass_and_balance)
```

This checks that **both** the on-ramp and after-landing CG points fall within the CG envelope.

**Important:** An aircraft must be balanced throughout the entire flight. An aircraft balanced on ramp but unbalanced after landing (or vice versa) is NOT safe to fly.

## Example Configuration

A typical Cessna 172 with diesel engine:

**Stations:**
- Front seats: 0.94 m from datum
- Back seats: 1.85 m from datum
- Cargo compartment 1: 2.41 m from datum
- Cargo compartment 2: 3.12 m from datum

**Aircraft Data:**
- Empty mass: 807 kg
- Empty CG: 1.00 m
- Fuel type: Diesel

**Fuel Tanks:**
- Single tank: 168.8 L capacity at 1.22 m arm

**CG Envelope:**
- Point 1: 0 kg at 0.89 m
- Point 2: 885 kg at 0.89 m
- Point 3: 1111 kg at 1.02 m
- Point 4: 1111 kg at 1.20 m
- Point 5: 0 kg at 1.20 m

## CG Envelope Shape

The CG envelope typically has these characteristics:

- **Forward limit** - Often constant or slightly forward-sloping with increasing mass
- **Aft limit** - Usually constant, representing the rearmost safe CG
- **Lower mass corner** - May restrict CG range at light weights
- **Upper mass corner** - Reflects maximum certificated weight

The envelope shape is aircraft-specific and comes from the aircraft's type certificate data sheet (TCDS) or pilot's operating handbook (POH).

## Common Errors

### Station Count Mismatch

Error: `UnexpectedMassesForStations`

**Cause:** Number of mass values doesn't match number of defined stations.

**Solution:** Ensure you provide exactly one mass value per station, even if the mass is zero.

### Fuel Tank Count Mismatch

Error: `UnexpectedNumberOfFuelStations`

**Cause:** Number of fuel quantities doesn't match number of tanks.

**Solution:** Provide fuel quantity for each tank defined in the aircraft.

### Exceeded Capacity

Errors: `ExceededFuelCapacityOnRamp`, `ExceededFuelCapacityAfterLanding`

**Cause:** Fuel quantity exceeds tank capacity.

**Solution:** Reduce fuel load to fit within usable capacity, or check for data entry errors.

## Reference Datum

The reference datum is an arbitrary point from which all arms are measured. It is typically:

- The firewall
- The propeller spinner
- A point ahead of the aircraft

**Important:** All arms must be measured from the same datum point. The datum location doesn't affect calculations as long as consistency is maintained.

## Practical Application

When planning a flight:

1. **Define aircraft configuration** with stations, tanks, and envelope
2. **Enter passenger/cargo masses** for each station
3. **Enter fuel quantities** for departure and expected landing
4. **Calculate mass and balance** using appropriate method
5. **Verify balance** using `is_balanced()`
6. **Check mass limits** against maximum takeoff weight (MTOW) and maximum landing weight (MLW)

Only proceed with the flight if:
- ✓ CG is within envelope on ramp
- ✓ CG is within envelope after landing
- ✓ Takeoff mass ≤ MTOW
- ✓ Landing mass ≤ MLW (if applicable)
