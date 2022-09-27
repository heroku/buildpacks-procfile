## Unreleased

- Remove redundant explicitly named stacks from `[[stacks]]`. ([#103](https://github.com/heroku/procfile-cnb/pull/103))
- Upgrade `libcnb` and `libherokubuildpack` to `0.11.0`. ([#98](https://github.com/heroku/procfile-cnb/pull/98) and [#102](https://github.com/heroku/procfile-cnb/pull/102))
- Buildpack now implements buildpack API version `0.8` and so requires lifecycle version `0.14.x` or newer. ([#98](https://github.com/heroku/procfile-cnb/pull/98))

## 1.0.2

- Strip buildpack binary for reduced builder image size (thanks to [`libcnb-cargo` v0.5.0](https://github.com/heroku/libcnb.rs/releases/tag/libcnb-cargo%2Fv0.5.0)).
- Remove incorrect error message shown in the case of internal buildpack regex errors ([#77](https://github.com/heroku/procfile-cnb/pull/77)).
- Update `libcnb` and `libherokubuildpack` from 0.5.0 to 0.9.0 ([#49](https://github.com/heroku/procfile-cnb/pull/49), [#60](https://github.com/heroku/procfile-cnb/pull/60), [#82](https://github.com/heroku/procfile-cnb/pull/82) and [#88](https://github.com/heroku/procfile-cnb/pull/88)).

## 1.0.1

- Added explicit support for heroku-* stacks.

## 1.0.0

- Initial release of Rust procfile buildpack, the old Go buildpack is now archived.
- Re-write logic of Procfile parsing to match Heroku's behavior, which has different behavior from the Go version (that assumed that a Procfile was YAML syntax).
