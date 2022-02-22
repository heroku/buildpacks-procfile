# Heroku Cloud Native Procfile Buildpack

[![CircleCI](https://circleci.com/gh/heroku/procfile-cnb/tree/main.svg?style=svg)](https://circleci.com/gh/heroku/procfile-cnb/tree/main)

This is a [Cloud Native Buildpack](https://buildpacks.io/) that replicates the behavior of
`Procfile` in [Heroku Buildpacks](https://devcenter.heroku.com/articles/buildpacks).

It is written in Rust using the Cloud Native Buildpack framework [libcnb.rs](https://github.com/Malax/libcnb.rs).

## Dev

### Pre-reqs

- Install rust - [rustup](https://rustup.rs/)
- Add musl target to rustup - `rustup target add x86_64-unknown-linux-musl`
- Install [cargo-make](https://github.com/sagiegurari/cargo-make): `cargo install cargo-make`
- On mac install [homebrew-musl-cross](https://github.com/FiloSottile/homebrew-musl-cross): `brew install FiloSottile/musl-cross/musl-cross`

### Test

Run unit tests:

```
$ cargo test
```

### Pack build Example

```
cargo make pack --profile "production" \
&& pack build procfile_example_app --path test/fixtures/app_with_procfile -B heroku/buildpacks:20 --buildpack ./target  -v \
&& docker run -it --entrypoint worker procfile_example_app
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
