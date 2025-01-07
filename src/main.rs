mod error;
mod launch;
mod procfile;

use crate::error::{error_handler, ProcfileBuildpackError};
use crate::procfile::Procfile;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::log::{log_header, log_info};
use std::fs;
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
        log_header("Discovering process types");

        let procfile = fs::read_to_string(context.app_dir.join("Procfile"))
            .map_err(ProcfileBuildpackError::CannotReadProcfileContents)
            .and_then(|procfile_contents| {
                procfile_contents
                    .parse()
                    .map_err(ProcfileBuildpackError::InvalidProcfile)
            })?;

        log_info(format!(
            "Procfile declares types -> {}",
            format_processes_for_log(&procfile)
        ));

        for warning in &procfile.warnings {
            println!("WARNING: {warning}");
        }

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

fn format_processes_for_log(procfile: &Procfile) -> String {
    if procfile.processes.is_empty() {
        String::from("(none)")
    } else {
        procfile
            .processes
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ")
    }
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

    #[test]
    fn test_empty_names_from_processes() {
        let procfile = Procfile::new();

        let out = format_processes_for_log(&procfile);
        assert_eq!(out, "(none)");
    }

    #[test]
    fn test_valid_process_names_from_processes() {
        let mut procfile = Procfile::new();
        procfile.insert("web", "rails -s");

        let out = format_processes_for_log(&procfile);
        assert_eq!(out, "web");

        procfile.insert("worker", "rake sidekiq");

        let out = format_processes_for_log(&procfile);
        assert_eq!(out, "web, worker");
    }
}
