# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- EuroNav 7 compatibility for controlled and restrictive airspace types
- Restrictive airspace record

## [0.3.0] - 2026-01-28

### Added

- Controlled airspace record
- Record iterator (`Records`) for streaming raw ARINC 424 data
- Derive macro for generating record parsers with less boilerplate
- Benchmarks for record parsing performance

### Changed

- Refactored fields and records module structure
- Enhanced error handling with more descriptive error types
- Optimized parsing performance using byte-based parsing

## [0.2.0] - 2025-11-10

### Added

- Location indicator (ICAO code) field for records
- AIRAC cycle field for tracking data currency

### Fixed

- AIRAC field naming consistency

## [0.1.2] - 2025-06-15

### Fixed

- Range check validation for latitude coordinates

## [0.1.1] - 2025-05-25

### Added

- Runway record parsing

### Changed

- Updated package metadata for crates.io publishing

### Fixed

- Clippy warnings

## [0.1.0] - 2024-09-02

### Added

- ARINC 424-23 format parser
- Airport record parsing
- Waypoint record parsing
- Field types for coordinates, identifiers, and navigation data
