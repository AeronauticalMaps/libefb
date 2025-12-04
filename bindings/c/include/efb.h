/* SPDX-License-Identifier: Apache-2.0 */
/* Copyright 2024 Joe Pearson
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef EFB
#define EFB

#include <stdbool.h>
#include <stdlib.h>

/// Angle unit with _rad_ as SI unit.
typedef enum {
  TrueNorth,
  MagneticNorth,
  Radian,
} EfbAngleUnit;

/// Duration unit with s as SI unit.
typedef enum {
  Seconds,
} EfbDurationUnit;

/// Type of fuel used by an aircraft.
///
/// Represents different fuel types that can be used in aircraft. Each fuel type
/// has an associated density at ISA conditions used for mass/volume
/// conversions.
typedef enum {
  /// Aviation gasoline (100LL) with density of 0.75 kg/L at ISA.
  AvGas,
  /// Diesel fuel with density of 0.838 kg/L at ISA.
  Diesel,
  /// Jet-A with density of 0.8 kg/L at ISA.
  JetA,
} EfbFuelType;

typedef enum {
  Arinc424,
  OpenAir,
} EfbInputFormat;

/// Length unit with _m_ as SI unit.
typedef enum {
  Centimeters,
  Meters,
  Kilometers,
  Inches,
  Feet,
  NauticalMiles,
  StatuteMiles,
} EfbLengthUnit;

/// Mass unit with _kg_ as SI unit.
typedef enum {
  Kilograms,
  Pounds,
} EfbMassUnit;

/// Speed unit with _m/s_ as SI unit.
typedef enum {
  MetersPerSecond,
  Knots,
  Mach,
} EfbSpeedUnit;

/// Volume with _m³_ as SI unit.
typedef enum {
  CubicMeters,
  Liter,
  USGallon,
} EfbVolumeUnit;

typedef struct EfbAircraftBuilder EfbAircraftBuilder;

/// A point that spawns the CG envelope.
typedef struct EfbCGLimit EfbCGLimit;

/// The Flight Management System (FMS).
///
/// This type wraps the [FMS] which is the integral system of this library. The
/// FMS holds all information like the navigation data or the route.
typedef struct EfbFMS EfbFMS;

/// The [Route] to fly.
///
/// This type is a wrapper around the [Route] with an initial cruise speed,
/// level and all legs along the route.
///
/// The [`efb_route_legs_first`] and [`efb_route_legs_next`] functions return a
/// leg of the route and can be used to iterate over the route:
///
/// ```
/// for (const EfbLeg *leg = efb_route_legs_first(route);
///      leg != NULL;
///      leg = efb_route_legs_next(route))
/// ```
typedef struct EfbRoute EfbRoute;

typedef struct EfbFlightPlanning EfbFlightPlanning;

/// Flight planning factory, which is used to build a flight planning.
typedef struct EfbFlightPlanningBuilder EfbFlightPlanningBuilder;

typedef struct EfbFuelPlanning EfbFuelPlanning;

/// An aircraft's fuel tank.
typedef struct EfbFuelTank EfbFuelTank;

/// A leg `from` one point `to` another.
typedef struct EfbLeg EfbLeg;

/// The mass & balance on ramp and after landing.
///
/// The mass and balance of the [`Aircraft`] is computed from [`Station`]s
/// loaded on the aircraft. The mass is computed as sum of all station's mass
/// and the balance is the sum of all moment divided by the total mass.
///
/// [`Aircraft`]: crate::aircraft::Aircraft
/// [`Station`]: crate::aircraft::Station
typedef struct EfbMassAndBalance EfbMassAndBalance;

typedef struct EfbPerformanceTable EfbPerformanceTable;

/// A row of the performance table presenting a performance up to a specific
/// level.
typedef struct EfbPerformanceTableRow EfbPerformanceTableRow;

/// A position within the aircraft that can be loaded with a payload.
///
/// The payload if an aircraft is loaded to defined _stations_ e.g. a
/// seat. Thus, the station defines where in reference to the aircraft's datum a
/// payload can be placed. The [`LoadedStation`] provides a station with it's
/// actual payload.
///
/// [`Aircraft`]: super::Aircraft
typedef struct EfbStation EfbStation;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
typedef struct {
  float value;
  EfbAngleUnit unit;
} EfbMeasurementf32AngleUnit;

typedef EfbMeasurementf32AngleUnit EfbAngle;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
typedef struct {
  float value;
  EfbLengthUnit unit;
} EfbMeasurementf32LengthUnit;

typedef EfbMeasurementf32LengthUnit EfbLength;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
typedef struct {
  uint32_t value;
  EfbDurationUnit unit;
} EfbMeasurementu32DurationUnit;

typedef EfbMeasurementu32DurationUnit EfbDuration;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
typedef struct {
  float value;
  EfbMassUnit unit;
} EfbMeasurementf32MassUnit;

typedef EfbMeasurementf32MassUnit EfbMass;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
typedef struct {
  float value;
  EfbSpeedUnit unit;
} EfbMeasurementf32SpeedUnit;

typedef EfbMeasurementf32SpeedUnit EfbSpeed;

/// The wind with a speed and direction.
///
/// The wind can be split into headwind (or tailwind) and crosswind components
/// for a direction. This provides e.g. information of the crosswind component
/// on landing.
///
/// # Examples
///
/// ```
/// # use std::str::FromStr;
/// # use efb::error::Error;
/// # use efb::measurements::{Angle, Speed};
/// # use efb::Wind;
/// #
/// # fn main() -> Result<(), Error> {
/// // the wind as reported from our destinations METAR
/// // blowing from the south
/// let wind = Wind::from_str("00010KT")?;
///
/// // we land on runway 09 pointing to the east so we have full 10 knots
/// // crosswind from the right
/// assert_eq!(wind.crosswind(&Angle::t(90.0)), Speed::kt(-10.0));
/// #     Ok(())
/// # }
/// ```
typedef struct {
  /// The direction from which the wind comes.
  EfbAngle direction;
  /// The wind speed.
  EfbSpeed speed;
} EfbWind;

/// Fuel quantity with a specific type and mass.
///
/// Represents a quantity of fuel, tracking both the fuel type and mass.
/// Fuel quantities can be created from either mass or volume, and converted
/// between the two at ISA conditions.
///
/// # Examples
///
/// ```
/// # use efb::prelude::*;
/// # use efb::measurements::{Mass, Volume};
/// // Create fuel from volume
/// let fuel = Fuel::from_volume(Volume::l(100.0), FuelType::Diesel);
///
/// // Create fuel from mass
/// let fuel = Fuel::new(Mass::kg(83.8), FuelType::Diesel);
///
/// // Add fuel quantities
/// let total = Fuel::from_volume(Volume::l(50.0), FuelType::Diesel)
///     + Fuel::from_volume(Volume::l(50.0), FuelType::Diesel);
/// assert_eq!(total.volume(), Volume::l(100.0));
/// ```
typedef struct {
  EfbFuelType fuel_type;
  EfbMass mass;
} EfbFuel;

typedef enum {
  PerHour,
} EfbFuelFlow_Tag;

typedef struct {
  EfbFuelFlow_Tag tag;
  union {
    struct {
      EfbFuel per_hour;
    };
  };
} EfbFuelFlow;

/// A vertical distance.
typedef enum {
  /// Absolute Altitude as distance above ground level in feet.
  Agl,
  /// Altitude in feet with reference to a local air pressure.
  Altitude,
  /// Pressure altitude in feet.
  PressureAltitude,
  /// Flight level in hundreds of feet as altitude at standard air pressure.
  Fl,
  /// Ground level.
  Gnd,
  /// True Altitude as distance above mean sea level.
  Msl,
  /// An unlimited vertical distance.
  Unlimited,
} EfbVerticalDistance_Tag;

typedef struct {
  EfbVerticalDistance_Tag tag;
  union {
    struct {
      uint16_t agl;
    };
    struct {
      uint16_t altitude;
    };
    struct {
      int16_t pressure_altitude;
    };
    struct {
      uint16_t fl;
    };
    struct {
      uint16_t msl;
    };
  };
} EfbVerticalDistance;

/// A measurement of a physical quantity.
///
/// The measurement has a value of type `T` and a unit `U` that implements a
/// [`UnitOfMeasure`]. For measurements of the same unit the operator `+`, `-`,
/// `*` and `/` are implemented. Differing units that have a value in a third
/// unit as result if divided or multiplied (e.g. length divided by duration is
/// speed) can implement those operations.
typedef struct {
  float value;
  EfbVolumeUnit unit;
} EfbMeasurementf32VolumeUnit;

typedef EfbMeasurementf32VolumeUnit EfbVolume;

/// Policy for determining fuel to load.
///
/// Represents different approaches to fuel planning. The [`fuel planning`] is
/// based on this policy.
///
/// [`fuel planning`]: `FuelPlanning`
typedef enum {
  /// Load minimum required fuel only.
  MinimumFuel,
  /// Fill tanks to capacity.
  MaximumFuel,
  /// Total fuel to load (includes required fuel).
  ManualFuel,
  /// Desired fuel remaining after landing.
  FuelAtLanding,
  /// Additional fuel beyond minimum requirements.
  ExtraFuel,
} EfbFuelPolicy_Tag;

typedef struct {
  EfbFuelPolicy_Tag tag;
  union {
    struct {
      EfbFuel manual_fuel;
    };
    struct {
      EfbFuel fuel_at_landing;
    };
    struct {
      EfbFuel extra_fuel;
    };
  };
} EfbFuelPolicy;

typedef enum {
  Manual,
} EfbReserve_Tag;

typedef struct {
  EfbReserve_Tag tag;
  union {
    struct {
      EfbDuration manual;
    };
  };
} EfbReserve;

/// The aircraft performance at a specific level and configuration.
typedef struct {
  /// The true airspeed.
  EfbSpeed tas;
  /// The fuel flow at the level.
  EfbFuelFlow ff;
} EfbPerformanceAtLevel;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/// Frees the string `s`.
///
/// # Safety
///
/// The caller must make sure that only strings that are allocated by the libefb
/// are passed to this function. It is unsafe to try freeing any string that was
/// returned by a function of this library.
void
efb_string_free(char *s);

/// Returns the angle formatted as string.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_angle_to_string(const EfbAngle *angle);

/// Returns the length formatted as string.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_length_to_string(const EfbLength *length);

/// Returns the duration formatted as string.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_duration_to_string(const EfbDuration *duration);

/// Returns the mass formatted as string.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_mass_to_string(const EfbMass *mass);

/// Returns the wind formatted as string.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_wind_to_string(const EfbWind *wind);

/// Returns the speed formatted as string.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_speed_to_string(const EfbSpeed *speed);

/// Returns an angle with reference to true north.
EfbAngle
efb_angle_true_north(float radians);

/// Returns an angle with reference to magnetic north.
EfbAngle
efb_angle_magnetic_north(float radians);

/// Returns a length in meter.
EfbLength
efb_length_m(float m);

/// Returns a length in feet.
EfbLength
efb_length_ft(float ft);

/// Returns a length in nautical miles.
EfbLength
efb_length_nm(float nm);

/// Returns the seconds `s` as duration.
EfbDuration
efb_duration(uint32_t s);

/// Returns the hours of the duration.
uint32_t
efb_duration_hours(const EfbDuration *duration);

/// Returns the minutes of the duration.
uint32_t
efb_duration_minutes(const EfbDuration *duration);

/// Returns the seconds of the duration.
uint32_t
efb_duration_seconds(const EfbDuration *duration);

/// Returns `l` liter of Diesel.
EfbFuel
efb_fuel_diesel_l(float l);

/// Returns a fuel flow of `fuel` per hour.
EfbFuelFlow
efb_fuel_flow_per_hour(EfbFuel fuel);

/// Returns a mass in kilogram.
EfbMass
efb_mass_kg(float kg);

/// Returns a speed in knots.
EfbSpeed
efb_speed_knots(float kt);

/// Returns a speed in m/s.
EfbSpeed
efb_speed_mps(float mps);

/// Returns a speed in mach.
EfbSpeed
efb_speed_mach(float mach);

/// Returns true if `a == b`.
bool
efb_vertical_distance_eq(const EfbVerticalDistance *a,
                         const EfbVerticalDistance *b);

/// Returns true if `a != b`.
bool
efb_vertical_distance_neq(const EfbVerticalDistance *a,
                          const EfbVerticalDistance *b);

/// Returns true if `a < b`.
bool
efb_vertical_distance_lt(const EfbVerticalDistance *a,
                         const EfbVerticalDistance *b);

/// Returns true if `a <= b`.
bool
efb_vertical_distance_lte(const EfbVerticalDistance *a,
                          const EfbVerticalDistance *b);

/// Returns true if `a > b`.
bool
efb_vertical_distance_gt(const EfbVerticalDistance *a,
                         const EfbVerticalDistance *b);

/// Returns true if `a >= b`.
bool
efb_vertical_distance_gte(const EfbVerticalDistance *a,
                          const EfbVerticalDistance *b);

/// Returns a vertical distance in feet.
EfbVerticalDistance
efb_vertical_distance_altitude(uint16_t ft);

/// Returns a volume in liter.
EfbVolume
efb_volume_l(float l);

/// Returns the limit's mass.
const EfbMass *
efb_cg_limit_mass(const EfbCGLimit *limit);

/// Returns the limit's distance in reference to the aircraft's datum.
const EfbLength *
efb_cg_limit_distance(const EfbCGLimit *limit);

/// Returns the tanks arm in reference to the aircraft's datum.
const EfbLength *
efb_fuel_tank_arm(const EfbFuelTank *tank);

/// Returns the tanks capacity.
const EfbVolume *
efb_fuel_tank_capacity(const EfbFuelTank *tank);

/// Returns the stations arm in reference to the aircraft's datum.
const EfbLength *
efb_station_arm(const EfbStation *station);

/// Returns the stations description or null if undefined.
///
/// # Safety
///
/// The returned value, if not null, needs to be freed by [`efb_string_free`].
char *
efb_station_description(const EfbStation *station);

/// Creates and returns a new FMS.
///
/// # Safety
///
/// The caller is responsible to free the allocated FMS by calling efb_fms_free.
EfbFMS *
efb_fms_new(void);

/// Frees the memory of the allocated FMS.
void
efb_fms_free(EfbFMS *fms);

/// Reads the string which is in the fmt into the navigation database.
///
/// # Safety
///
/// It is up to the caller to guarantee that `s` points to a valid string.
void
efb_fms_nd_read(EfbFMS *fms, const char *s, EfbInputFormat fmt);

/// Decodes the route and enters it into the FMS.
///
/// # Safety
///
/// It is up to the caller to guarantee that `route` points to a valid string.
void
efb_fms_decode(EfbFMS *fms, const char *route);

/// Returns a reference to the FMS route.
///
/// # Safety
///
/// It's up to the caller to unref the returned pointer.
EfbRoute *
efb_fms_route_ref(EfbFMS *fms);

/// Decreases the reference count of the route.
void
efb_fms_route_unref(EfbRoute *route);

/// Returns the flight planning.
///
/// The planning is created by the builder returned by
/// [`efb_flight_planning_builder_new`].
const EfbFlightPlanning *
efb_fms_flight_planning(const EfbFMS *fms);

/// Sets the flight planning.
///
/// The planning is created by the builder returned by
/// [`efb_flight_planning_builder_new`].
void
efb_fms_set_flight_planning(EfbFMS *fms,
                            const EfbFlightPlanningBuilder *builder);

/// Prints the route and planning of the FMS.
///
/// # Safety
///
/// The returned string needs to be freed by [`efb_string_free`].
char *
efb_fms_print(EfbFMS *fms, size_t line_length);

/// Returns a new aircraft builder.
///
/// Use the builder to gradually provide all the different inputs required to
/// define an aircraft.
///
/// # Safety
///
/// The memory allocated for the builder needs to be freed by calling
/// [`efb_aircraft_builder_free`].
EfbAircraftBuilder *
efb_aircraft_builder_new(void);

/// Frees the aircraft builder.
void
efb_aircraft_builder_free(EfbAircraftBuilder *builder);

void
efb_aircraft_builder_registration(EfbAircraftBuilder *builder,
                                  const char *registration);

/// Pushes a new station to the stations and returns it.
const EfbStation *
efb_aircraft_builder_stations_push(EfbAircraftBuilder *builder, EfbLength arm,
                                   const char *description);

void
efb_aircraft_builder_stations_remove(EfbAircraftBuilder *builder, size_t at);

/// Returns the first station.
///
/// To iterate over all stations, call [`efb_aircraft_builder_stations_next`]
/// until `NULL` is returned:
///
/// ```c
/// for (const EfbStation *station =
/// efb_aircraft_builder_stations_first(builder);
///      station != NULL;
///      station = efb_aircraft_builder_stations_next(builder))
/// ```
const EfbStation *
efb_aircraft_builder_stations_first(EfbAircraftBuilder *builder);

/// Returns the next station.
///
/// When the end of the stations is reached, this function returns a null
/// pointer.
const EfbStation *
efb_aircraft_builder_stations_next(EfbAircraftBuilder *builder);

void
efb_aircraft_builder_empty_mass(EfbAircraftBuilder *builder, EfbMass mass);

void
efb_aircraft_builder_empty_balance(EfbAircraftBuilder *builder,
                                   EfbLength distance);

void
efb_aircraft_builder_fuel_type(EfbAircraftBuilder *builder,
                               EfbFuelType fuel_type);

/// Pushes a new tank to the tanks and returns it.
const EfbFuelTank *
efb_aircraft_builder_tanks_push(EfbAircraftBuilder *builder, EfbVolume capacity,
                                EfbLength arm);

void
efb_aircraft_builder_tanks_remove(EfbAircraftBuilder *builder, size_t at);

/// Returns the first tank.
///
/// To iterate over all tanks, call [`efb_aircraft_builder_tanks_next`]
/// until `NULL` is returned:
///
/// ```c
/// for (const EfbTank *tank = efb_aircraft_builder_tanks_first(builder);
///      tank != NULL;
///      tank = efb_aircraft_builder_tanks_next(builder))
/// ```
const EfbFuelTank *
efb_aircraft_builder_tanks_first(EfbAircraftBuilder *builder);

/// Returns the next tank.
///
/// When the end of the tanks is reached, this function returns a null pointer.
const EfbFuelTank *
efb_aircraft_builder_tanks_next(EfbAircraftBuilder *builder);

/// Pushes a new CG limit into the envelope and returns a pointer to the new
/// limit.
const EfbCGLimit *
efb_aircraft_builder_cg_envelope_push(EfbAircraftBuilder *builder, EfbMass mass,
                                      EfbLength distance);

void
efb_aircraft_builder_cg_envelope_remove(EfbAircraftBuilder *builder, size_t at);

/// Returns the first CG limit.
///
/// To iterate over all CG limits, call
/// [`efb_aircraft_builder_cg_envelope_next`] until `NULL` is returned:
///
/// ```c
/// for (const EfbCGLimit *limit =
/// efb_aircraft_builder_cg_envelope_first(builder);
///      limit != NULL;
///      limit = efb_aircraft_builder_cg_envelope_next(builder))
/// ```
const EfbCGLimit *
efb_aircraft_builder_cg_envelope_first(EfbAircraftBuilder *builder);

/// Returns the next CG limit.
///
/// When the end of the CG limits is reached, this function returns a null
/// pointer.
const EfbCGLimit *
efb_aircraft_builder_cg_envelope_next(EfbAircraftBuilder *builder);

void
efb_aircraft_builder_notes(EfbAircraftBuilder *builder, const char *notes);

const EfbFuelPlanning *
efb_flight_planning_fuel_planning(const EfbFlightPlanning *planning);

const EfbMassAndBalance *
efb_flight_planning_mb(const EfbFlightPlanning *planning);

bool
efb_flight_planning_is_balanced(const EfbFlightPlanning *planning);

/// Returns a new flight planning builder.
///
/// # Safety
///
/// The memory allocated for the builder needs to be freed by calling
/// [`efb_flight_planning_builder_free`].
EfbFlightPlanningBuilder *
efb_flight_planning_builder_new(void);

/// Frees the flight planning builder.
void
efb_flight_planning_builder_free(EfbFlightPlanningBuilder *builder);

void
efb_flight_planning_builder_set_aircraft(
    EfbFlightPlanningBuilder *builder,
    const EfbAircraftBuilder *aircraft_builder);

void
efb_flight_planning_builder_set_mass(EfbFlightPlanningBuilder *builder,
                                     const EfbMass *mass, size_t len);

void
efb_flight_planning_builder_set_policy(EfbFlightPlanningBuilder *builder,
                                       EfbFuelPolicy policy);

void
efb_flight_planning_builder_set_taxi(EfbFlightPlanningBuilder *builder,
                                     EfbFuel taxi);

void
efb_flight_planning_builder_set_reserve(EfbFlightPlanningBuilder *builder,
                                        EfbReserve reserve);

void
efb_flight_planning_builder_set_perf(
    EfbFlightPlanningBuilder *builder,
    EfbPerformanceAtLevel (*perf)(const EfbVerticalDistance *),
    EfbVerticalDistance ceiling);

const EfbFuel *
efb_fuel_planning_taxi(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_climb(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_trip(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_alternate(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_reserve(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_total(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_min(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_extra(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_on_ramp(const EfbFuelPlanning *planning);

const EfbFuel *
efb_fuel_planning_after_landing(const EfbFuelPlanning *planning);

const EfbMass *
efb_mass_and_balance_mass_on_ramp(const EfbMassAndBalance *mb);

const EfbMass *
efb_mass_and_balance_mass_after_landing(const EfbMassAndBalance *mb);

const EfbLength *
efb_mass_and_balance_balance_on_ramp(const EfbMassAndBalance *mb);

const EfbLength *
efb_mass_and_balance_balance_after_landing(const EfbMassAndBalance *mb);

/// Returns a new performance table
///
/// Use the table to define the performance at different level.
///
/// # Safety
///
/// The memory allocated for the table needs to be freed by calling
/// [`efb_performance_table_free`].
EfbPerformanceTable *
efb_performance_table_new(void);

/// Frees the performance table.
void
efb_performance_table_free(EfbPerformanceTable *table);

const EfbPerformanceTableRow *
efb_performance_table_push(EfbPerformanceTable *table,
                           EfbVerticalDistance level, EfbSpeed tas,
                           EfbFuelFlow ff);

void
efb_performance_table_remove(EfbPerformanceTable *table, size_t at);

/// Returns the first performance.
///
/// To iterate over the table, call [`efb_performance_table_next`]
/// until `NULL` is returned:
///
/// ```c
/// for (const EfbPerformanceTableRow *row = efb_performance_table_first(table);
///      row != NULL;
///      row = efb_performance_table_next(table))
/// ```
const EfbPerformanceTableRow *
efb_performance_table_first(EfbPerformanceTable *table);

/// Returns the next performance.
///
/// When the end of the table is reached, this function returns a null pointer.
const EfbPerformanceTableRow *
efb_performance_table_next(EfbPerformanceTable *table);

const EfbSpeed *
efb_performance_table_row_tas(const EfbPerformanceTableRow *row);

void
efb_performance_table_row_set_tas(EfbPerformanceTableRow *row, EfbSpeed tas);

const EfbFuelFlow *
efb_performance_table_row_ff(const EfbPerformanceTableRow *row);

void
efb_performance_table_row_set_ff(EfbPerformanceTableRow *row, EfbFuelFlow ff);

/// Returns the routes total length.
///
/// If the route has no legs, a NULL pointer is returned.
const EfbLength *
efb_route_totals_dist(EfbRoute *route);

/// Returns the estimated time enroute.
///
/// If the ETE can't be calculated, a NULL pointer is returned.
const EfbDuration *
efb_route_totals_ete(EfbRoute *route);

/// Returns the first leg in the route.
const EfbLeg *
efb_route_legs_first(EfbRoute *route);

/// Returns the next leg in the route.
///
/// When the end of the legs is reached, this function returns a null pointer.
const EfbLeg *
efb_route_legs_next(EfbRoute *route);

/// Returns the ident from where the leg starts.
///
/// # Safety
///
/// The returned value needs to be freed by [`efb_string_free`].
char *
efb_leg_get_from(const EfbLeg *leg);

/// Returns the ident to where the leg ends.
///
/// # Safety
///
/// The returned value needs to be freed by [`efb_string_free`].
char *
efb_leg_get_to(const EfbLeg *leg);

/// Returns the leg's level or null if unknown.
const EfbVerticalDistance *
efb_leg_get_level(const EfbLeg *leg);

/// Returns the wind along the leg or null if unknown.
const EfbWind *
efb_leg_get_wind(const EfbLeg *leg);

/// Returns the leg's true airspeed or null if unknown.
const EfbSpeed *
efb_leg_get_tas(const EfbLeg *leg);

/// Returns the true heading considering the WCA or null if unknown.
const EfbAngle *
efb_leg_get_heading(const EfbLeg *leg);

/// Returns the magnetic heading considering the variation at the start of the
/// leg or null if unknown.
const EfbAngle *
efb_leg_get_mh(const EfbLeg *leg);

/// Returns the bearing between the two points.
const EfbAngle *
efb_leg_get_bearing(const EfbLeg *leg);

/// Returns the magnetic course taking the magnetic variation from the starting
/// point into consideration.
const EfbAngle *
efb_leg_get_mc(const EfbLeg *leg);

/// Returns the distance between the leg's two points.
const EfbLength *
efb_leg_get_dist(const EfbLeg *leg);

/// Returns the ground speed in knots or null if unknown.
const EfbSpeed *
efb_leg_get_gs(const EfbLeg *leg);

/// Returns the estimated time enroute the leg or null if unknown.
const EfbDuration *
efb_leg_get_ete(const EfbLeg *leg);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* EFB */
