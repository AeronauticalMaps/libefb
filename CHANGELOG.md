# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Physical quantity an unit measures

### Changed

- Don't take ownership of wrapped types passed from JavaScript

## [0.4.0] - 2025-11-10

### Added

- Accumulate distance, ETE and fuel of route
- Create navigation data per data source
- Add AIRAC and location to navigation aid
- Serialization support for navigation data
- Serialization support for aircraft builder
- WebAssembly bindings

### Changed

- Get route totals as one struct
- Modify route and navigation data from closure to reevaluate
  dependant FMS parts

### Fixed

- Fix naming conflict of Performance with From
- Inclusion of region code in airspace enums

### Removed

- Reading navigation data from file

## [0.3.3] - 2025-08-13

### Added

- Return the route as GeoJSON
- Implement std Error trait
- ICAO Doc. 8643 aircraft type designator

### Changed

- Let FMS take ownership of flight planning builder

### Fixed

- Create temperature with proper unit
- Flight planning gets updated on route change

## [0.3.2] - 2025-06-15

### Added

- Add takeoff/landing performance builder
- Add AvGas to fuels
- Add serialization support feature
- Add an aircraft builder

### Changed

- Reduce public struct fields to stabilize API
- Use symbols to create temperature
- Improve interoperability by deriving common traits

### Fixed

- Fix range check for ARINC 424 coordinates

## [0.3.1] - 2025-05-25

### Added

- Parse runway from ARINC 424 record

## [0.3.0] - 2025-05-25

### Added

- Add runway analysis
- Add origin and destination airport to route
- Add pressure measurement
- Add feet as length unit
- This CHANGELOG file

## [0.2.0] - 2025-03-24

### Added

- `measurements` module for physical quantities e.g. a length.

### Changed

- Use new measurement types wherever the core measurements were used.

### Removed

- Measurement types that were part of the `core` module.

## [0.1.0] - 2025-03-16

### Added

- Parser for ARINC 424 and OpenAir.
- FMS that reads navigation data and decodes a route.
- Aircraft performance entry.
- Fuel planning and Mass & Balance.

[unreleased]: https://github.com/pearjo/libefb/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/pearjo/libefb/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/pearjo/libefb/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/pearjo/libefb/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/pearjo/libefb/compare/0.3.0...v0.3.1
[0.3.0]: https://github.com/pearjo/libefb/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/pearjo/libefb/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/pearjo/libefb/releases/tag/0.1.0
