mod error;
mod launch;
mod procfile;

use crate::error::{ProcfileBuildpackError, error_handler};
use crate::procfile::Procfile;
use bullet_stream::{Print, style};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{Buildpack, buildpack_main};
use std::io::stdout;
use std::path::Path;

#[cfg(test)]
use libcnb_test as _;

struct ProcfileBuildpack;

impl Buildpack for ProcfileBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = ProcfileBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if dir_has_procfile(context.app_dir) {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let mut bullet = Print::new(stdout())
            .h2("Procfile Buildpack")
            .bullet(format!("Processes from {}", style::value("Procfile")));

        let procfile: Procfile = fs_err::read_to_string(context.app_dir.join("Procfile"))
            .map_err(ProcfileBuildpackError::CannotReadProcfileContents)
            .and_then(|procfile_contents| {
                procfile_contents
                    .parse()
                    .map_err(ProcfileBuildpackError::ProcfileParsingError)
            })?;

        let warning_prefix = style::important("WARNING:");
        for message in &procfile.warnings {
            bullet = bullet.sub_bullet(format!("{warning_prefix} {message}"));
        }

        for (name, command) in &procfile.processes {
            bullet = bullet.sub_bullet(format!("{name}: {cmd}", cmd = style::command(command)));
        }
        bullet.done().done();

        BuildResultBuilder::new()
            .launch(
                procfile
                    .try_into()
                    .map_err(ProcfileBuildpackError::ProcfileConversionError)?,
            )
            .build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        libherokubuildpack::error::on_error(error_handler, error);
    }
}

fn dir_has_procfile(app_dir: impl AsRef<Path>) -> bool {
    app_dir.as_ref().join("Procfile").exists()
}

// Implements the main function and wires up the framework for the given buildpack.
buildpack_main!(ProcfileBuildpack);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_detect() {
        let app_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/web_and_worker_procfile");
        assert!(dir_has_procfile(app_dir));
    }

    #[test]
    fn test_missing_procfile_detect() {
        let app_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/missing_procfile");
        assert!(!dir_has_procfile(app_dir));
    }
}
