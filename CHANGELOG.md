## Unreleased

- Remove incorrect error message shown in the case of internal buildpack regex errors ([#77](https://github.com/heroku/procfile-cnb/pull/77)).
- Update `libcnb` and `libherokubuildpack` from 0.5.0 to 0.9.0 ([#49](https://github.com/heroku/procfile-cnb/pull/49), [#60](https://github.com/heroku/procfile-cnb/pull/60), [#82](https://github.com/heroku/procfile-cnb/pull/82) and [#88](https://github.com/heroku/procfile-cnb/pull/88)).

## 1.0.1

- Added explicit support for heroku-* stacks.

## 1.0.0

- Initial release of Rust procfile buildpack, the old Go buildpack is now archived.
- Re-write logic of Procfile parsing to match Heroku's behavior, which has different behavior from the Go version (that assumed that a Procfile was YAML syntax).
