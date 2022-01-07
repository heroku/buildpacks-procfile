// Enable rustc and Clippy lints that are disabled by default.
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unused-crate-dependencies
#![warn(unused_crate_dependencies)]
// https://rust-lang.github.io/rust-clippy/stable/index.html
#![warn(clippy::pedantic)]

mod error;
use crate::error::ProcfileError;
use crate::error::ProcfileParsingError;

use std::path::Path;
use std::str::FromStr;

use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{Launch, Process, ProcessType};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::{log_header, log_info};

use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

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
        let mut launch = Launch::new();

        let procfile_path = context.app_dir.join("Procfile");
        let procfile_contents =
            std::fs::read_to_string(procfile_path).map_err(ProcfileError::Io)?;

        launch.processes = parse_procfile(&procfile_contents)
            .map_err(ProcfileError::ProcfileParsingError)
            .and_then(|entries| entries.into_iter().map(Process::try_from).collect())?;

        log_info(format!(
            "Procfile declares types -> {}",
            format_processes_for_log(&launch.processes)
        ));

        BuildResultBuilder::new().launch(launch).build()
    }
}

fn dir_has_procfile(app_dir: impl AsRef<Path>) -> bool {
    app_dir.as_ref().join("Procfile").exists()
}

impl TryFrom<ProcfileEntry> for Process {
    type Error = ProcfileError;

    fn try_from(value: ProcfileEntry) -> Result<Process, Self::Error> {
        let process_type =
            ProcessType::from_str(&value.name).map_err(ProcfileError::ProcessType)?;

        Ok(Process {
            r#type: process_type,
            command: value.command,
            args: Vec::<String>::new(),
            direct: false,
            default: value.default,
        })
    }
}

fn format_processes_for_log(processes: &[Process]) -> String {
    let mut names = processes
        .iter()
        .map(|p| p.r#type.as_str())
        .collect::<Vec<&str>>();

    if names.is_empty() {
        names.push("(none)");
    }

    names.join(", ")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcfileEntry {
    name: String,
    default: bool,
    command: String,
}

fn parse_procfile(
    procfile_contents: impl AsRef<str>,
) -> Result<Vec<ProcfileEntry>, ProcfileParsingError> {
    let contents = YamlLoader::load_from_str(procfile_contents.as_ref())
        .map_err(|_| ProcfileParsingError::CannotParse)?;

    if contents.is_empty() {
        return Ok(vec![]);
    }

    let hash = contents[0]
        .as_hash()
        .ok_or(ProcfileParsingError::CannotParse)?;
    let len = hash.len();
    hash.into_iter()
        .map(|key_value| build_procfile_entry(key_value, len))
        .collect()
}

fn build_procfile_entry(
    key_value: (&Yaml, &Yaml),
    len: usize,
) -> Result<ProcfileEntry, ProcfileParsingError> {
    let name = key_value
        .0
        .as_str()
        .ok_or(ProcfileParsingError::EmptyProcessName)?;
    let command = key_value
        .1
        .as_str()
        .ok_or(ProcfileParsingError::EmptyProcessCommand)?;
    Ok(ProcfileEntry {
        name: name.to_string(),
        default: len == 1 || name == "web",
        command: command.to_string(),
    })
}

// Implements the main function and wires up the framework for the given buildpack.
buildpack_main!(ProcfileBuildpack);

#[cfg(test)]
mod tests {
    use super::*;
    use libcnb::data::process_type;

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
    fn test_empty_parse_procfile() {
        let entries = parse_procfile("");
        assert_eq!(entries, Ok(vec![]));
    }

    #[test]
    fn test_valid_parse_procfile() {
        let entries = parse_procfile("web: rails s");
        assert_eq!(
            entries,
            Ok(vec![ProcfileEntry {
                name: String::from("web"),
                command: String::from("rails s"),
                default: true,
            }])
        );
    }

    #[test]
    fn test_one_non_web_entry_is_default_parse_procfile() {
        let entries = parse_procfile("zweb: rails s");
        assert_eq!(
            entries,
            Ok(vec![ProcfileEntry {
                name: String::from("zweb"),
                command: String::from("rails s"),
                default: true,
            }])
        );
    }

    #[test]
    fn test_multiple_valid_parse_procfile() {
        let entries = parse_procfile("web: rails s\nworker: rake sidekiq");
        assert_eq!(
            entries,
            Ok(vec![
                ProcfileEntry {
                    name: String::from("web"),
                    command: String::from("rails s"),
                    default: true,
                },
                ProcfileEntry {
                    name: String::from("worker"),
                    command: String::from("rake sidekiq"),
                    default: false
                }
            ])
        );
    }

    #[test]
    fn test_cannot_parse_procfile() {
        let entries = parse_procfile("web ");
        assert_eq!(entries, Err(ProcfileParsingError::CannotParse));
    }

    #[test]
    fn test_missing_command_parse_procfile() {
        let entries = parse_procfile("web: ");
        assert_eq!(entries, Err(ProcfileParsingError::EmptyProcessCommand));
    }

    #[test]
    fn test_missing_name_parse_procfile() {
        let entries = parse_procfile(": rails -s");
        assert_eq!(entries, Err(ProcfileParsingError::EmptyProcessName));
    }

    #[test]
    fn test_valid_to_process() {
        let process: Result<Process, _> = ProcfileEntry {
            name: String::from("web"),
            command: String::from("rails -s"),
            default: false,
        }
        .try_into();
        assert_eq!(process.unwrap().r#type, process_type!("web"));
    }

    #[test]
    fn test_invalid_to_process() {
        let name = String::from("!nv@lid");
        let process: Result<Process, _> = ProcfileEntry {
            name,
            command: String::from("rails -s"),
            default: false,
        }
        .try_into();

        assert!(process.is_err());
    }

    #[test]
    fn test_empty_names_from_processes() {
        let out = format_processes_for_log(&[]);
        assert_eq!(out, "(none)");
    }

    #[test]
    fn test_valid_process_names_from_processes() {
        let web = Process {
            r#type: process_type!("web"),
            args: vec![],
            command: String::from("rails -s"),
            default: true,
            direct: false,
        };
        let worker = Process {
            r#type: process_type!("worker"),
            args: vec![],
            command: String::from("rake sidekiq"),
            default: true,
            direct: false,
        };

        let mut processes = vec![web];
        let out = format_processes_for_log(&processes);
        assert_eq!(out, "web");

        processes.push(worker);
        let out = format_processes_for_log(&processes);
        assert_eq!(out, "web, worker");
    }
}
