# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0] - 2024-12-20

- No changes.

## [3.2.0] - 2024-12-20

### Changed

- Build output style updated to `bullet_stream`. Output now also includes the commands that were pulled from the `Procfile` in the output. ([#252](https://github.com/heroku/buildpacks-procfile/pull/252))

## [3.1.2] - 2024-07-02

### Changed

- Upgraded Rust dependencies. ([#232](https://github.com/heroku/buildpacks-procfile/pull/232) and [#235](https://github.com/heroku/buildpacks-procfile/pull/235))

### Removed

- Removed `[[stacks]]` table workaround from `buildpack.toml`. ([#231](https://github.com/heroku/buildpacks-procfile/pull/231))

## [3.1.1] - 2024-05-02

### Fixed

- No changes to functionality. New release to fix incorrectly published artifacts in 3.1.0.

## [3.1.0] - 2024-05-02

### Added

- Support for `arm64` and multi-arch images. ([#225](https://github.com/heroku/buildpacks-procfile/pull/225))

## [3.0.1] - 2024-03-13

### Changed

- Switched Docker Hub repository from `docker.io/heroku/procfile-cnb` to `docker.io/heroku/buildpack-procfile`. ([#219](https://github.com/heroku/buildpacks-procfile/pull/219))
- Renamed GitHub repository from `heroku/procfile-cnb` to `heroku/buildpacks-procfile`. ([#216](https://github.com/heroku/buildpacks-procfile/pull/216))

## [3.0.0] - 2024-02-28

### Added

- Enabled tracing/telemetry via `libcnb`'s `trace` feature. ([#208](https://github.com/heroku/buildpacks-procfile/pull/208))

### Changed

- Updated to Buildpack API 0.10. ([#205](https://github.com/heroku/buildpacks-procfile/pull/205))
    - All launch processes are now wrapped in `bash -c` instead of using CNB's `direct = false` directive, which is no longer available.
    - `.profile` and `.profile.d` scripts will no longer be automatically sourced.
    - CNB Lifecycle 0.17 or newer is now required.

## [2.0.2] - 2023-10-24

### Changed

- Updated buildpack display name, description and keywords. ([#189](https://github.com/heroku/buildpacks-procfile/pull/189))

## [2.0.1] - 2023-08-21

### Changed

- Switched to new buildpack release automation. As a side-effect, the filename of the packaged buildpack attached to the GitHub release has changed from `heroku_procfile_X.Y.Z.cnb` to `heroku_procfile.cnb`. ([#156](https://github.com/heroku/buildpacks-procfile/pull/156) and [#170](https://github.com/heroku/buildpacks-procfile/pull/170))
- Updated buildpack dependencies.

## [2.0.0] - 2022-09-27

### Changed

- Buildpack now implements buildpack API version 0.8 and so requires `lifecycle` version 0.14.x or newer. ([#98](https://github.com/heroku/buildpacks-procfile/pull/98))
- Upgraded `libcnb` and `libherokubuildpack` to 0.11.0. ([#98](https://github.com/heroku/buildpacks-procfile/pull/98) and [#102](https://github.com/heroku/buildpacks-procfile/pull/102))

### Removed

- Removed explicitly named stacks from `[[stacks]]`, which were a workaround for Pack CLI <0.24.1 not supporting the wildcard stack. ([#103](https://github.com/heroku/buildpacks-procfile/pull/103))

## [1.0.2] - 2022-07-14

### Changed

- The buildpack binary is now stripped for reduced builder image size (thanks to [`libcnb-cargo` v0.5.0](https://github.com/heroku/libcnb.rs/releases/tag/libcnb-cargo%2Fv0.5.0)).
- Updated `libcnb` and `libherokubuildpack` from 0.5.0 to 0.9.0. ([#49](https://github.com/heroku/buildpacks-procfile/pull/49), [#60](https://github.com/heroku/buildpacks-procfile/pull/60), [#82](https://github.com/heroku/buildpacks-procfile/pull/82) and [#88](https://github.com/heroku/buildpacks-procfile/pull/88))

### Fixed

- Removed incorrect error message shown in the case of internal buildpack regex errors. ([#77](https://github.com/heroku/buildpacks-procfile/pull/77))

## [1.0.1] - 2022-04-05

### Fixed

- Fixed compatibility with older versions of Pack CLI that do not support the wildcard stack in `buildpack.toml`. ([#55](https://github.com/heroku/buildpacks-procfile/pull/55))

## [1.0.0] - 2022-04-05

### Changed

- Initial release of Rust procfile buildpack, the old Go buildpack is now archived.
- Re-write logic of Procfile parsing to match Heroku's behavior, which has different behavior from the Go version (that assumed that a Procfile was YAML syntax).

[unreleased]: https://github.com/heroku/buildpacks-procfile/compare/v4.0.0...HEAD
[4.0.0]: https://github.com/heroku/buildpacks-procfile/compare/v3.2.0...v4.0.0
[3.2.0]: https://github.com/heroku/buildpacks-procfile/compare/v3.1.2...v3.2.0
[3.1.2]: https://github.com/heroku/buildpacks-procfile/compare/v3.1.1...v3.1.2
[3.1.1]: https://github.com/heroku/buildpacks-procfile/compare/v3.1.0...v3.1.1
[3.1.0]: https://github.com/heroku/buildpacks-procfile/compare/v3.0.1...v3.1.0
[3.0.1]: https://github.com/heroku/buildpacks-procfile/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/heroku/buildpacks-procfile/compare/v2.0.2...v3.0.0
[2.0.2]: https://github.com/heroku/buildpacks-procfile/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/heroku/buildpacks-procfile/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/heroku/buildpacks-procfile/compare/v1.0.2...v2.0.0
[1.0.2]: https://github.com/heroku/buildpacks-procfile/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/heroku/buildpacks-procfile/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/heroku/buildpacks-procfile/releases/tag/v1.0.0
