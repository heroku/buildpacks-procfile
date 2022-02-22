# Procfile Cloud Native Buildpack (CNB) in Rust

This buildpack implements https://github.com/heroku/procfile-cnb in the Rust programming language. The goal in the re-write is improved confidence and maintenance through a stronger type system and unit tests.

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
