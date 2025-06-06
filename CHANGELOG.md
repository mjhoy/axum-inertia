# Changelog

## [Unreleased]

### Added

- Development dependency updates.

## [0.8.1] 2025-03-17

### Added

- Various dependency updates for local development.
- (ci) Configure dependabot.

## [0.8.0] 2025-03-08

### Added

- (Breaking) Update to axum v0.8.1. ([#44](https://github.com/mjhoy/axum-inertia/pull/44))
- Adds missing <!DOCTYPE html> tag in the vite config. ([#43](https://github.com/mjhoy/axum-inertia/pull/43))

## [0.7.0] 2025-03-03

### Fixed

- Include query params in the `url` field of the response. This was stripped out
  before and would result in weird client
  behavior. ([#40](https://github.com/mjhoy/axum-inertia/pull/40) thanks
  [@Dsaquel](https://github.com/Dsaquel))

## [0.6.0] 2024-12-05

### Added

- (Breaking) Remove static lifetime from `Page` struct
  ([#31](https://github.com/mjhoy/axum-inertia/pull/31) thanks
  [@KaioFelps](https://github.com/KaioFelps)). This allows for more flexible
  lifetimes in the `component` string reference, for e.g. dynamically generated
  component names.

- The `Vite::Development` struct now accepts a `base` param to set a path prefix
  on development vite script tags.
  ([#35](https://github.com/mjhoy/axum-inertia/pull/35) thanks
  [@Dsaquel](https://github.com/Dsaquel))

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
