# Changelog

## [Unreleased]

## [0.5.1] 2024-12-03

### Added

- Supports https for vite development (thanks @Dsaquel #32)

## [0.5.0] 2024-06-26

### Added

- Now supports precalculated integrity hash in the manifest file (via
  `integrity` field).

## [0.4.0] 2024-06-12

### Added

- Now supports vite development with React.
- A new `Props` trait is added for use with `Inertia::render`. Objects
  that implement `Props` know how to serialize themselves to json and
  are passed information about "partial" Inertia reloads to exclude
  certain fields. This trait can hopefully eventually be used to
  implement a derive macro for prop types that support things like
  lazily evaluated props.

### Fixed

- Now uses the original request url in the props response. This fixes
  nested routers.

## [0.3.0] 2024-02-12

- Split configuration to a new `InertiaConfig` struct

## [0.2.0] 2023-11-27

- Update to axum 0.7.

## [0.1.1] 2023-11-11

- Doc updates.

## [0.1.0] 2023-11-01

Initial release.
