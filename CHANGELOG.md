# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Types of changes:

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

## [Unreleased]

## [0.9.3]

### Fixed

- Bump version to 0.9.3 in Cargo.toml

## [0.9.2]

### Fixed

- Remove redundant nested `unsafe` blocks in the macOS backend that triggered `unused_unsafe` warnings

## [0.9.1]

### Fixed

- Suppress `unexpected_cfgs` warnings caused by `cfg(cargo-clippy)` in `objc` 0.2.x macro expansions

## [0.9.0]

### Added

- Add opt-in macOS/iOS shuffle commands (#85. Thanks! @barissglc)
  - New `MediaControlEvent::SetShuffle(bool)` event
  - New `MediaControls::set_shuffle(Option<bool>)` on macOS/iOS
- Add CI workflow for auto-releases
- Add diagram in README
- Add diagram file

### Changed

- **Breaking:** Crate and Edition Updates (#78. Thanks! @101100)
  - Rust edition bumped to 2024.
  - MSRV bumped to 1.87
  - Dependency updates
  - CI will now run on stable and 1.87
- **Breaking:** `MediaControlEvent` is now `#[non_exhaustive]`. Matches on it need a wildcard (`_ =>`) arm
- Update README.md

### Fixed

- Add nil check for NSImage in macOS load_image_from_url (#76. Thanks! @teancom)
- Fix macOS build: stale Cocoa nil references from #76 merge (#79. Thanks! @animegolem)
- fix(zbus): Update `set_volume` in `PlayerInterface` to update internal volume (#81. Thanks! @UnknownSuperficialNight)
- fix: prevent macOS artwork flicker (#83. Thanks! @Waradu)
- fix: iOS panic when loading cover art from URL (#86 / #58. Thanks! @will3942)

## [0.8.3]

### Added

- rustfmt.toml to enforce formatting

### Changed

- Update README/docs to clarify MacOS Usage (#60 thank you! @ILikeTeaALot)

### Fixed

- Standardized line ending behavior in Git
- Implement error for all error types (#63 thank you! @nick42d)
- Exclude dbus on Android (#59 thank you! @will3942)

## [0.8.2]

### Fixed

- Failing iOS import (#57 Thank you @will3942!)

## [0.8.1]

### Fixed

- Android platform module error
- Linux CI failing due to missing dependency

## [0.8.0]

### Added

- iOS support (#55 Thank you @XMLHexagram !)
- Error handling for zbus, and unify dbus/zbus Error type (#52 Thank you @taoky !)

### Fixed

- Clippy lints (#51 Thank you @LucasFA !)

## [0.7.3]

### Added

- Documentation for `MediaControlEvent::SetVolume` #47

## [0.7.2]

### Changed

- Bumped MSRV to 1.67

## [0.7.1]

### Added

- MSRV in Cargo.toml and rust-toolchain (#46)
- CHANGELOG.md (#45)

### Changed

- Lowered MSRV back to 1.60 from 1.74 (#46)
- Updated CI to support MSRV (#46)
- Updated CI dependencies

## [0.7.0]

### Added

- Implemented volume control on Linux (#42)

### Changed

- Refactored D-Bus module (#42)
