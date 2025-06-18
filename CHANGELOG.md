# Changelog

## [Unreleased]

## [0.8.2] 2025-06-18

Mostly just dependency updates.

### Added

- Development dependency updates:
  - Bump openssl from 0.10.71 to 0.10.73
  - Bump reqwest from 0.12.14 to 0.12.18
  - Bump tokio from 1.44.1 to 1.45.1
  - Bump axum from 0.8.1 to 0.8.4
  - Bump url 2.4.1 to 2.5.4
  - Bump reqwest from 0.12.14 to 0.12.15
  - Bump hyper from 1.5.2 to 1.6.0

### Fixed

- Remove unnecessary string copies
  ([#79](https://github.com/mjhoy/axum-inertia/pull/79)) thanks [@redzic](https://github.com/redzic)

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
