use libcnb::{cnb_runtime, GenericErrorHandler};
use procfile_buildpack::{build, detect};

// Main entrypoint, the `cnb_runtime` produces a single binary
// that will call either `detect` or `build` functions based on the name of the
// binary file.
fn main() {
    cnb_runtime(detect, build, GenericErrorHandler);
}
