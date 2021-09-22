// Enable rustc and Clippy lints that are disabled by default.
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unused-crate-dependencies
#![warn(unused_crate_dependencies)]
// https://rust-lang.github.io/rust-clippy/stable/index.html
#![warn(clippy::pedantic)]
// Re-disable pedantic lints that are too noisy/unwanted.
#![allow(clippy::module_name_repetitions)]

use libcnb::{cnb_runtime, GenericErrorHandler};

mod build;
mod detect;
mod display;
mod error;

#[cfg(test)]
mod test_helper;

// Main entrypoint, the `cnb_runtime` produces a single binary
// that will call either `detect` or `build` functions based on the name of the
// binary file.

fn main() {
    cnb_runtime(detect::detect, build::build, GenericErrorHandler);
}
