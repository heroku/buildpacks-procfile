// Enable rustc and Clippy lints that are disabled by default.
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unused-crate-dependencies
#![warn(unused_crate_dependencies)]
// https://rust-lang.github.io/rust-clippy/stable/index.html
#![warn(clippy::pedantic)]

mod error;
mod launch;
mod procfile;

use crate::error::ProcfileError;
use crate::error::ProcfileParsingError;

use std::path::Path;

use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};

use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::{log_header, log_info};
use std::fs;

use crate::procfile::Procfile;

struct ProcfileBuildpack;

impl Buildpack for ProcfileBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = ProcfileError;

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
            .map_err(ProcfileError::Io)
            .and_then(|procfile_contents| {
                procfile_contents
                    .parse()
                    .map_err(ProcfileError::ProcfileParsingError)
            })?;

        log_info(format!(
            "Procfile declares types -> {}",
            format_processes_for_log(&procfile)
        ));

        BuildResultBuilder::new()
            .launch(procfile.try_into()?)
            .build()
    }
}

fn dir_has_procfile(app_dir: impl AsRef<Path>) -> bool {
    app_dir.as_ref().join("Procfile").exists()
}

fn format_processes_for_log(procfile: &Procfile) -> String {
    if procfile.is_empty() {
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
        let tmp_dir = tempfile::tempdir().unwrap();
        let procfile = tmp_dir.path().join("Procfile");
        std::fs::write(
            procfile,
            "julie_andrews: supercalifragilisticexpialidocious",
        )
        .unwrap();

        assert!(dir_has_procfile(tmp_dir));
    }

    #[test]
    fn test_missing_procfile_detect() {
        let tmp_dir = tempfile::tempdir().unwrap();
        assert!(!dir_has_procfile(tmp_dir));
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
