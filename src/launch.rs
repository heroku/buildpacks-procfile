use crate::Procfile;
use libcnb::data::launch::{Launch, Process, ProcessType};
use std::str::FromStr;

impl TryFrom<Procfile> for Launch {
    type Error = ProcfileConversionError;

    fn try_from(value: Procfile) -> Result<Self, Self::Error> {
        let mut launch = Launch::new();

        for (key, value) in value.processes {
            launch.processes.push(Process {
                r#type: ProcessType::from_str(&key)
                    .map_err(ProcfileConversionError::InvalidProcessType)?,
                command: value,
                args: Vec::<String>::new(),
                direct: false,
                default: key == "web",
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

#[derive(thiserror::Error, Debug)]
pub enum ProcfileConversionError {
    #[error("Incompatible process type")]
    InvalidProcessType(libcnb::data::launch::ProcessTypeError),
}

#[cfg(test)]
mod test {
    use crate::Procfile;
    use libcnb::data::launch::{Launch, Process};
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
                command: String::from("web_command"),
                args: vec![],
                direct: false,
                default: true
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
                command: String::from("xxx_command"),
                args: vec![],
                direct: false,
                default: true
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
                    command: String::from("web_command"),
                    args: vec![],
                    direct: false,
                    default: true
                },
                Process {
                    r#type: process_type!("foo"),
                    command: String::from("foo_command"),
                    args: vec![],
                    direct: false,
                    default: false
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
                    command: String::from("foo_command"),
                    args: vec![],
                    direct: false,
                    default: false
                },
                Process {
                    r#type: process_type!("bar"),
                    command: String::from("bar_command"),
                    args: vec![],
                    direct: false,
                    default: false
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
                    command: String::from("aaa_command"),
                    args: vec![],
                    direct: false,
                    default: false
                },
                Process {
                    r#type: process_type!("ccc"),
                    command: String::from("ccc_command"),
                    args: vec![],
                    direct: false,
                    default: false
                },
                Process {
                    r#type: process_type!("bbb"),
                    command: String::from("bbb_command"),
                    args: vec![],
                    direct: false,
                    default: false
                },
            ]
        );
    }
}
