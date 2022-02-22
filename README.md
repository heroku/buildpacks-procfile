# Procfile Cloud Native Buildpack (CNB) in Rust

This buildpack implements https://github.com/heroku/procfile-cnb in the Rust programming language. The goal in the re-write is improved confidence and maintenance through a stronger type system and unit tests.

## Development

### Prerequisites

See [Development Environment Setup](https://github.com/Malax/libcnb.rs#development-environment-setup).

### Test

Run unit tests:

```
$ cargo test
```

### Pack build example

```
$ cargo libcnb package \
&& pack build procfile_example_app --builder heroku/buildpacks:20 --buildpack target/buildpack/debug/heroku_procfile --path tests/fixtures/app_with_procfile --verbose \
&& docker run -it --rm --entrypoint worker procfile_example_app
```

```
$ pack inspect procfile_example_app | grep -A10 Processes
Processes:
  TYPE                 SHELL        COMMAND              ARGS
  web (default)        bash         node index.js
  worker               bash         while true; do echo 'lol'; sleep 2; done
```

### Structure

The code produces a single binary that contain both the "detect" and "build" interfaces:

```rs
fn main() {
    cnb_runtime(detect, build, GenericErrorHandler);
}
```

The function `cnb_runtime` changes behavior based on the name of the calling file. In the `Makefile.toml` the binary is symlinked with different names (`detect` versus `build`):

```rs
fs::hard_link(destination.join("bin/build"), destination.join("bin/detect")).unwrap();
```

The package can be complied for linux (musl) by running:

```
cargo make pack --profile "production"
```

This produces a `target/bin/detect` and `target/bin/build` that can be used for pack.

The functionality of those two binaries are come from the functions with the same name:

```rs
fn detect(context: GenericDetectContext) -> Result<DetectOutcome, E> {
  //...
}
fn build(context: GenericDetectContext) -> Result<(), E> {
  // ...
}
```
