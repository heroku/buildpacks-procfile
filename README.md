# Heroku Cloud Native Procfile Buildpack

[![CI](https://github.com/heroku/buildpacks-procfile/actions/workflows/ci.yml/badge.svg)](https://github.com/heroku/buildpacks-procfile/actions/workflows/ci.yml)

This is a [Cloud Native Buildpack](https://buildpacks.io/) that replicates the behavior of
[`Procfile`](https://devcenter.heroku.com/articles/procfile) from non-CNB Heroku Builds.

It is written in Rust using the Cloud Native Buildpack framework [libcnb.rs](https://github.com/heroku/libcnb.rs).

## Development

### Prerequisites

See [Development Environment Setup](https://github.com/heroku/libcnb.rs#development-environment-setup).

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
&& pack build procfile_example_app --builder heroku/builder:22 --buildpack target/buildpack/debug/heroku_procfile --path tests/fixtures/web_and_worker_procfile --verbose \
&& docker run -it --rm --entrypoint worker procfile_example_app
```

```
$ pack inspect procfile_example_app | grep -A10 Processes
Processes:
  TYPE                 SHELL        COMMAND                                   ARGS        WORK DIR
  web (default)        bash         echo 'this is the web process!'                       /workspace
  worker               bash         echo 'this is the worker process!'                    /workspace
```

## Releasing

[Deploy Cloud Native Buildpacks](https://github.com/heroku/languages-team/blob/main/languages/cnb/deploy.md)
