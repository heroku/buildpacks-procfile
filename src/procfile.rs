use linked_hash_map::LinkedHashMap;
use regex::Regex;
use std::str::FromStr;

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
        let re_carriage_return_newline =
            Regex::new("\\r\\n?").map_err(ProcfileParsingError::InvalidRegex)?;
        let re_multiple_newline =
            Regex::new("\\n*\\z").map_err(ProcfileParsingError::InvalidRegex)?;

        // https://github.com/heroku/codon/blob/2613554383cb298076b4a722f4a1aa982ad757e6/lib/slug_compiler/slug.rb#L538-L545
        let re_procfile_entry = Regex::new("^[[:space:]]*([a-zA-Z0-9_-]+):?\\s+(.*)[[:space:]]*")
            .map_err(ProcfileParsingError::InvalidRegex)?;

        let procfile_contents = re_carriage_return_newline.replace_all(procfile_contents, "\n");
        let procfile_contents = re_multiple_newline.replace(&procfile_contents, "\n");

        Ok(Procfile {
            processes: procfile_contents
                .lines()
                .into_iter()
                .filter_map(|line| re_procfile_entry.captures(line))
                .filter_map(|cap| {
                    cap.get(1).and_then(|name| {
                        cap.get(2).map(|command| {
                            (String::from(name.as_str()), String::from(command.as_str()))
                        })
                    })
                })
                .collect::<LinkedHashMap<String, String>>(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcfileParsingError {
    InvalidRegex(regex::Error),
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
    fn test_nonsense_procfile() {
        assert_eq!("&&&&&".parse::<Procfile>(), Ok(Procfile::new()));
    }

    #[test]
    fn test_missing_command_parse_procfile() {
        assert_eq!("web:".parse::<Procfile>(), Ok(Procfile::new()));
    }

    #[test]
    fn test_missing_name_parse_procfile() {
        assert_eq!(": rails -s".parse::<Procfile>(), Ok(Procfile::new()));
    }
}
