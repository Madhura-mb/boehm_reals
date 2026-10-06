# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased 

### Tests
- Added few arithmetic operation based tests in `property_test.rs`

### Fixed
- Normalized denominator sign in `new` function.
- Fixed incorrect mantissa rounding behavior of `double_value` function. 
- Fixed range problem in `value_of_double` function.

## [0.1.0] - 2026-09-25

### Added
- Initial release
- Implemented the Bounded Rational component of Boehm's Reals.
- Added basic arithmatic operations (addition, subtraction, multiplication and division)
- Added conversion of bounded rational values to integer and floating-point representations.
- Added reduction and normalization of rational values.
- Added demo code demonstrating the usage of the Bounded Rational implementation