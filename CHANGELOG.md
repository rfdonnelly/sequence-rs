# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes.

## [0.5.0]

### Added

* Added support for Windows (77fd4db)
* sv-bindings: Added `Rvs::next()` function to simplify one-time variable use (2d3cf52)

### Changed

* Changed the PRNG from XorShift to PCG to improve quality (1a8aec9)

### Misc

* Upgraded all dependencies
* Upgraded all packages to Rust 2018 edition

## [0.4.1]

### Fixed

* Fixed doneness for sampling without replacement. (6a5bfd6)

## [0.4.0]

### Added

* Added weighted sample without replacement. (0d06139)

### Changed

* Replaced `Sample` (sample with replacement), `Unique` (sample without
  replacement), and `{}` (weighted sample with replacement) with a unified
  sample syntax. (0d06139)

## [0.3.0]

### Added

* [systemverilog-bindings] Added `search_path_from_plusargs()` (36b47a4)
* [parser] Added support for variables that begin with an uppercase
  letter (2525d67)
* [development] Added code coverage collection (dd2b968, 47ef6fa)
* [development] Applied rustfmt (8461a9c)

### Fixed

* [c-api] Fixed `rvs_parse()` ignoring search path (d295337)
* [systemverilog-bindings] Fix line numbers in error messages (83bfcb8)

### Changed

* [rvs] Changed Sequence arguments to be re-evaluated on every cycle (ef1fabb)
* [grammar] Changed Sequence syntax
  * From: `Sequence(count)`, `Sequence(offset, count)`, `Sequence(offset,
    increment, count)`
  * To: `Sequence(last)`, `Sequence(first, last)`, `Sequence(first, last,
    increment)`
* [c-api] Changed `rvs_parse()` filename recognition heuristics to classify
  anything that ends with `.rvs` as a filename (d295337)
* [grammar] Removed optional leading underscore from hexadecimal
  literals (5d0d443)
* [grammar] Removed optional trailing comma for ranges (9b8370b)
* [rvs, ast] Renamed `WeightedSample` to `Weighted` (cb0f710)
* [ast] Replaced `VariableInst`, `EnumItemInst`, and `EnumItem` with
  `RIdentifier` (2525d67)

## [0.2.0] - 2017-06-20

First release.

[Unreleased]: https://github.com/rfdonnelly/rvs/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/rfdonnelly/rvs/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/rfdonnelly/rvs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/rfdonnelly/rvs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/rfdonnelly/rvs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/rfdonnelly/rvs/tree/v0.2.0
