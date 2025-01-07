use crate::procfile::Procfile;
use libcnb::data::launch::{Launch, Process, ProcessType, WorkingDirectory};
use std::str::FromStr;

impl TryFrom<Procfile> for Launch {
    type Error = ProcfileConversionError;

    fn try_from(value: Procfile) -> Result<Self, Self::Error> {
        let mut launch = Launch {
            labels: vec![],
            processes: vec![],
            slices: vec![],
        };

        for (key, value) in value.processes {
            launch.processes.push(Process {
                r#type: ProcessType::from_str(&key)
                    .map_err(ProcfileConversionError::InvalidProcessType)?,
                command: vec![String::from("bash"), String::from("-c")],
                args: vec![value],
                default: key == "web",
                working_directory: WorkingDirectory::App,
            });
        }

        if launch.processes.len() == 1 {
            if let Some(process) = launch.processes.first_mut() {
                process.default = true;
            }
        }

        Ok(launch)
    }
}

#[derive(Debug)]
pub(crate) enum ProcfileConversionError {
    InvalidProcessType(libcnb::data::launch::ProcessTypeError),
}

#[cfg(test)]
mod test {
    use super::*;
    use libcnb::data::launch::{Launch, Process, WorkingDirectory};
    use libcnb::data::process_type;

    #[test]
    fn test_single_web_process() {
        let mut procfile = Procfile::new();
        procfile.insert("web", "web_command");

        let launch: Launch = procfile.try_into().unwrap();

        assert_eq!(
            launch.processes,
            vec![Process {
                r#type: process_type!("web"),
                command: vec![String::from("bash"), String::from("-c")],
                args: vec![String::from("web_command")],
                default: true,
                working_directory: WorkingDirectory::App,
            }]
        );
    }

    #[test]
    fn test_single_non_web_process() {
        let mut procfile = Procfile::new();
        procfile.insert("xxx", "xxx_command");

        let launch: Launch = procfile.try_into().unwrap();

        assert_eq!(
            launch.processes,
            vec![Process {
                r#type: process_type!("xxx"),
                command: vec![String::from("bash"), String::from("-c")],
                args: vec![String::from("xxx_command")],
                default: true,
                working_directory: WorkingDirectory::App,
            }]
        );
    }

    #[test]
    fn test_web_and_additional_process() {
        let mut procfile = Procfile::new();
        procfile.insert("web", "web_command");
        procfile.insert("foo", "foo_command");

        let launch: Launch = procfile.try_into().unwrap();

        assert_eq!(
            launch.processes,
            vec![
                Process {
                    r#type: process_type!("web"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("web_command")],
                    default: true,
                    working_directory: WorkingDirectory::App,
                },
                Process {
                    r#type: process_type!("foo"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("foo_command")],
                    default: false,
                    working_directory: WorkingDirectory::App,
                }
            ]
        );
    }

    #[test]
    fn test_multiple_non_web_processes() {
        let mut procfile = Procfile::new();
        procfile.insert("foo", "foo_command");
        procfile.insert("bar", "bar_command");

        let launch: Launch = procfile.try_into().unwrap();

        assert_eq!(
            launch.processes,
            vec![
                Process {
                    r#type: process_type!("foo"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("foo_command")],
                    default: false,
                    working_directory: WorkingDirectory::App,
                },
                Process {
                    r#type: process_type!("bar"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("bar_command")],
                    default: false,
                    working_directory: WorkingDirectory::App,
                }
            ]
        );
    }

    #[test]
    fn test_no_processes() {
        let procfile = Procfile::new();
        let launch: Launch = procfile.try_into().unwrap();

        assert_eq!(launch.processes, vec![]);
    }

    #[test]
    fn test_process_order() {
        let mut procfile = Procfile::new();
        procfile.insert("aaa", "aaa_command");
        procfile.insert("ccc", "ccc_command");
        procfile.insert("bbb", "bbb_command");

        let launch: Launch = procfile.try_into().unwrap();

        assert_eq!(
            launch.processes,
            vec![
                Process {
                    r#type: process_type!("aaa"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("aaa_command")],
                    default: false,
                    working_directory: WorkingDirectory::App,
                },
                Process {
                    r#type: process_type!("ccc"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("ccc_command")],
                    default: false,
                    working_directory: WorkingDirectory::App,
                },
                Process {
                    r#type: process_type!("bbb"),
                    command: vec![String::from("bash"), String::from("-c")],
                    args: vec![String::from("bbb_command")],
                    default: false,
                    working_directory: WorkingDirectory::App,
                },
            ]
        );
    }
}
