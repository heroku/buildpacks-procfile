# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.1] - 2023-08-21

### Changed

- Switched to new buildpack release automation. As a side-effect, the filename of the packaged buildpack attached to the GitHub release has changed from `heroku_procfile_X.Y.Z.cnb` to `heroku_procfile.cnb`. ([#156](https://github.com/heroku/procfile-cnb/pull/156) and [#170](https://github.com/heroku/procfile-cnb/pull/170))
- Updated buildpack dependencies.

## [2.0.0] - 2022-09-27

### Changed

- Buildpack now implements buildpack API version 0.8 and so requires `lifecycle` version 0.14.x or newer. ([#98](https://github.com/heroku/procfile-cnb/pull/98))
- Upgraded `libcnb` and `libherokubuildpack` to 0.11.0. ([#98](https://github.com/heroku/procfile-cnb/pull/98) and [#102](https://github.com/heroku/procfile-cnb/pull/102))

### Removed

- Removed explicitly named stacks from `[[stacks]]`, which were a workaround for Pack CLI <0.24.1 not supporting the wildcard stack. ([#103](https://github.com/heroku/procfile-cnb/pull/103))

## [1.0.2] - 2022-07-14

### Changed

- The buildpack binary is now stripped for reduced builder image size (thanks to [`libcnb-cargo` v0.5.0](https://github.com/heroku/libcnb.rs/releases/tag/libcnb-cargo%2Fv0.5.0)).
- Updated `libcnb` and `libherokubuildpack` from 0.5.0 to 0.9.0. ([#49](https://github.com/heroku/procfile-cnb/pull/49), [#60](https://github.com/heroku/procfile-cnb/pull/60), [#82](https://github.com/heroku/procfile-cnb/pull/82) and [#88](https://github.com/heroku/procfile-cnb/pull/88))

### Fixed

- Removed incorrect error message shown in the case of internal buildpack regex errors. ([#77](https://github.com/heroku/procfile-cnb/pull/77))

## [1.0.1] - 2022-04-05

### Fixed

- Fixed compatibility with older versions of Pack CLI that do not support the wildcard stack in `buildpack.toml`. ([#55](https://github.com/heroku/procfile-cnb/pull/55))

## [1.0.0] - 2022-04-05

### Changed

- Initial release of Rust procfile buildpack, the old Go buildpack is now archived.
- Re-write logic of Procfile parsing to match Heroku's behavior, which has different behavior from the Go version (that assumed that a Procfile was YAML syntax).

[unreleased]: https://github.com/heroku/procfile-cnb/compare/v2.0.1...HEAD
[2.0.1]: https://github.com/heroku/procfile-cnb/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/heroku/procfile-cnb/compare/v1.0.2...v2.0.0
[1.0.2]: https://github.com/heroku/procfile-cnb/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/heroku/procfile-cnb/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/heroku/procfile-cnb/releases/tag/v1.0.0
