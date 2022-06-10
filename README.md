# Heroku Cloud Native Procfile Buildpack

[![CircleCI](https://circleci.com/gh/heroku/procfile-cnb/tree/main.svg?style=svg)](https://circleci.com/gh/heroku/procfile-cnb/tree/main)

This is a [Cloud Native Buildpack](https://buildpacks.io/) that replicates the behavior of
`Procfile` in [Heroku Buildpacks](https://devcenter.heroku.com/articles/buildpacks).

It is written in Rust using the Cloud Native Buildpack framework [libcnb.rs](https://github.com/Malax/libcnb.rs).

## Deployment

### 0) Prerelease

- Ensure that the version in `buildpack.toml` is correct. The following deployment steps will create a release with the that version number.
- Ensure there's an entry for the same version in `CHANGELOG.md`.

### 1) Generate a release

- Visit the actions page https://github.com/heroku/procfile-cnb/actions,
- Click on "release" and then "Run workflow".

When the action is successful a release will be added to https://github.com/heroku/procfile-cnb/releases and docker hub https://hub.docker.com/r/heroku/procfile-cnb/tags.

### 2) Update builders

Heroku builders (github.com/heroku/builders) must be updated to the latest
version of the buildpack. A detailed procedure is available [here](github.com/heroku/languages-team/blob/main/languages/cnb/deploy.md#update-builder-images).

## Development

### Prerequisites

See [Development Environment Setup](https://github.com/Malax/libcnb.rs#development-environment-setup).

### Test

Run unit tests:

```
$ cargo test
```

Run integration tests:

```
$ cargo test -- --ignored
```

Or to run all of the tests at the same time:

```
$ cargo test -- --include-ignored
```

### Pack build example

```
$ cargo libcnb package \
&& pack build procfile_example_app --builder heroku/builder:22 --buildpack target/buildpack/debug/heroku_procfile --path tests/fixtures/app_with_procfile --verbose \
&& docker run -it --rm --entrypoint worker procfile_example_app
```

```
$ pack inspect procfile_example_app | grep -A10 Processes
Processes:
  TYPE                 SHELL        COMMAND              ARGS
  web (default)        bash         node index.js
  worker               bash         while true; do echo 'lol'; sleep 2; done
```
