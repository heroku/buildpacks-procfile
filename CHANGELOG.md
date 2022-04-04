## Unreleased

## 1.0.1
- Added explicit support for heroku-* stacks.

## 1.0.0

- Initial release of Rust procfile buildpack, the old Go buildpack is now archived.
- Re-write logic of Procfile parsing to match Heroku's behavior, which has different behavior from the Go version (that assumed that a Procfile was YAML syntax).
