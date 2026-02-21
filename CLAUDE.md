# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when
working with code in this repository.

## Project Overview

LibEFB is an in Rust written Electronic Flight Bag (EFB) library for
flight planning and air navigation. The repository is home of the
primary `efb` crate and supporting crates to read navigation data from
various sources like ARINC 424. The `efb` crate provides a Flight
Management System (FMS) that bundles all loaded navigation data, route
planning, and flight calculations. It is the central element holding
all string together.

## Build and Development Commands

### Core Commands

- `cargo build` - Build the entire workspace
- `cargo test` - Run all tests
- `cargo clippy` - Run linting
- `cargo fmt` - Format code
- `cargo doc --open` - Generate and open documentation

### Workspace Structure

This is a Cargo workspace with the following main crates:

- `efb/` - Core library with FMS functionality
- `arinc424/` - ARINC 424 navigation data parser
- `aixm/` - AIXM (Aeronautical Information Exchange Model) parser
- `bindings/c/` - C bindings for FFI
- `bindings/python/` - Python bindings using PyO3
- `bindings/wasm/` - WebAssembly bindings
- `bindings/swift/` - Swift bindings

### Language Bindings

Each binding in `bindings/` has its own build system:

- **C bindings**: Uses cbindgen to generate headers, examples in
  `bindings/c/examples/`
- **Python bindings**: Uses maturin for building, examples in
  `bindings/python/examples/`
- **WASM bindings**: Uses wasm-pack, output in `bindings/wasm/pkg/`
- **Swift bindings**: Uses Package.swift, examples in Swift package

### Testing

- Run tests for specific crate: `cargo test -p efb`
- Run integration tests: `cargo test --test <test_name>`
- Test data files are typically in `tests/` directories

## Core Architecture

### Flight Management System (FMS)

The central component is `efb::fms::FMS` which integrates:

- **Navigation Data (`nd`)**: Holds all data relevant for navigation
  in a spatial index
- **Route**: Manages flight routes with legs and waypoints
- **Flight Planning (`fp`)**: Performs calculations for fuel,
  performance, runway analysis

### Key Modules

- `efb::measurements/` - Comprehensive measurement types (length,
  speed, mass, etc.) with unit conversions
- `efb::aircraft/` - Aircraft configuration including fuel tanks, CG
  envelopes, stations
- `efb::geom/` - Geometric calculations for coordinates, bounding
  boxes
- `efb::route/` - Route representation with legs and navigation
- `efb::fp/` - Flight planning with fuel calculations and performance
  analysis

### Data Flow

1. Load navigation data via e.g. `NavigationData::try_from_arinc424()`
   and append to `fms.modify_nd(|nd| nd.append(new_nd))`
2. Decode route string using `fms.decode()`
3. Configure aircraft using builders in `aircraft/` module
4. Generate flight planning with `FlightPlanningBuilder`

### Route Decoding

Routes use custom format:

    WIND SPEED ALTITUDE ORIGIN [WAYPOINTS] [VIA] DESTINATION

Terminal waypoints are resolved within the terminal area they occur
in. If this is ambiguous since the point occurs in both terminal areas
it is in between off, the terminal area context can be explicitly
closed by a `DCT` via.

- Example: `"29020KT N0107 A0250 EDDH N2 N1 EDHF"`
- Ambiguous terminal waypoint: `"EDHL W DCT W EDAH"`
- Supports runway specifications: `"EDDH33 EDHF20"`

### Measurement System

All physical quantities use the `measurements/` module with strong typing:

- Length: nautical miles, kilometers, feet, meters
- Speed: knots, km/h, m/s
- Mass: kilograms, pounds
- Volume: liters
- Temperature: Celsius, Fahrenheit, Kelvin

When doing calculations, they MUST be done by using measurements and
NOT by simply using numerical values to ensure that values are
calculated in compatible units. If multiplying two different
measurements like time and speed, the result will be length which is
ensured by sticking to the measurements.

### Error Handling

There SHOULD be as little assumptions as possible in the code. No
error in any computation go silently. Thus, the `efb` crate uses the
`efb::error::Error` enum for all error types across the library.
