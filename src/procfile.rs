use linked_hash_map::LinkedHashMap;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Procfile {
    pub(crate) processes: LinkedHashMap<String, String>,
}

impl Procfile {
    pub(crate) fn new() -> Self {
        Procfile {
            processes: LinkedHashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, k: impl Into<String>, v: impl Into<String>) {
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
        // Using `.expect()` since these can only fail if we've supplied invalid an invalid regex,
        // which would be caught by both the `invalid_regex` Clippy lint and the buildpack's tests.
        let re_carriage_return_newline = Regex::new("\\r\\n?").expect("Invalid Procfile regex");
        let re_multiple_newline = Regex::new("\\n*\\z").expect("Invalid Procfile regex");

        // https://github.com/heroku/codon/blob/2613554383cb298076b4a722f4a1aa982ad757e6/lib/slug_compiler/slug.rb#L538-L545
        let re_procfile_entry = Regex::new("^[[:space:]]*([a-zA-Z0-9_-]+):?\\s+(.*)[[:space:]]*")
            .expect("Invalid Procfile regex");

        let procfile_contents = re_carriage_return_newline.replace_all(procfile_contents, "\n");
        let procfile_contents = re_multiple_newline.replace(&procfile_contents, "\n");

        Ok(Procfile {
            processes: procfile_contents
                .lines()
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

// There are currently no ways in which parsing can fail, however we will add some in the future:
// https://github.com/heroku/procfile-cnb/issues/73
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ProcfileParsingError {}

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
