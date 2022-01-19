use crate::ProcfileParsingError;
use linked_hash_map::LinkedHashMap;
use std::str::FromStr;
use yaml_rust::YamlLoader;

#[derive(Debug, Eq, PartialEq)]
pub struct Procfile {
    pub processes: LinkedHashMap<String, String>,
}

impl Procfile {
    pub fn new() -> Self {
        Procfile {
            processes: LinkedHashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    pub fn insert(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.processes.insert(k.into(), v.into());
    }
}

impl Default for Procfile {
    fn default() -> Self {
        Procfile::new()
    }
}

impl FromStr for Procfile {
    type Err = ProcfileParsingError;

    fn from_str(procfile_contents: &str) -> Result<Self, Self::Err> {
        let yaml = YamlLoader::load_from_str(procfile_contents)
            .map_err(|_| ProcfileParsingError::CannotParse)?;

        if let Some(first_yaml_entry) = yaml.first() {
            let first_yaml_entry_as_hash = first_yaml_entry
                .as_hash()
                .ok_or(ProcfileParsingError::CannotParse)?;

            first_yaml_entry_as_hash
                .iter()
                .map(|(key, value)| match (key.as_str(), value.as_str()) {
                    (Some(key), Some(value)) => Ok((String::from(key), String::from(value))),
                    (None, _) => Err(ProcfileParsingError::EmptyProcessName),
                    (_, None) => Err(ProcfileParsingError::EmptyProcessCommand),
                })
                .collect::<Result<LinkedHashMap<String, String>, ProcfileParsingError>>()
                .map(|processes| Procfile { processes })
        } else {
            Ok(Procfile::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_parse_procfile() {
        assert_eq!("".parse::<Procfile>(), Ok(Procfile::new()));
    }

    #[test]
    fn test_valid_parse_procfile() {
        let mut expected_procfile = Procfile::new();
        expected_procfile.insert("web", "rails s");

        assert_eq!("web: rails s".parse::<Procfile>(), Ok(expected_procfile));
    }

    #[test]
    fn test_multiple_valid_parse_procfile() {
        let mut expected_procfile = Procfile::new();
        expected_procfile.insert("web", "rails s");
        expected_procfile.insert("worker", "rake sidekiq");

        assert_eq!(
            "web: rails s\nworker: rake sidekiq".parse::<Procfile>(),
            Ok(expected_procfile)
        );
    }

    #[test]
    fn test_cannot_parse_procfile() {
        assert_eq!(
            "web ".parse::<Procfile>(),
            Err(ProcfileParsingError::CannotParse)
        );
    }

    #[test]
    fn test_missing_command_parse_procfile() {
        assert_eq!(
            "web: ".parse::<Procfile>(),
            Err(ProcfileParsingError::EmptyProcessCommand)
        );
    }

    #[test]
    fn test_missing_name_parse_procfile() {
        assert_eq!(
            ": rails -s".parse::<Procfile>(),
            Err(ProcfileParsingError::EmptyProcessName)
        );
    }
}
