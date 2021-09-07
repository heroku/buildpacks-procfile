# Basic Rust CNB

This basic rust CNB should create a web process that echos "lol" in a loop, but it does not.

Compare to a bash example that does the same thing: https://github.com/schneems/basic_bash_lol_cnb_buildpack, (but the bash version works).

## Pre-reqs

- Install rust - [rustup](https://rustup.rs/)
- Add musl target to rustup - `rustup target add x86_64-unknown-linux-musl`
- Install [cargo-make](https://github.com/sagiegurari/cargo-make): `cargo install cargo-make`
- On mac install [homebrew-musl-cross](https://github.com/FiloSottile/homebrew-musl-cross): `brew install FiloSottile/musl-cross/musl-cross`

## Dev

```
cargo make pack --profile "production" \
&& pack build procfile_example_app --path test/fixtures/app_with_procfile -B heroku/buildpacks:20 --buildpack ./target  -v \
&& docker run -it --init procfile_example_app
```