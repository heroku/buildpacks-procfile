use linked_hash_map::LinkedHashMap;
use regex::Regex;
use std::str::FromStr;
use winnow::combinator::{alt, eof};
use winnow::error::{StrContext, StrContextValue};
use winnow::token::{one_of, take_while};
use winnow::Parser;
use winnow::{
    ascii::space0,
    combinator::{opt, preceded, repeat, terminated},
    error::{ContextError, ParseError},
};

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
// https://github.com/heroku/buildpacks-procfile/issues/73
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ProcfileParsingError {}

#[allow(unused_imports)]
use winnow::{
    ascii::{line_ending, till_line_ending},
    error::InputError,
    prelude::*,
    token::take_till,
};

// fn parse_procfile<'s>(input: &'s str) -> Result<Procfile, ParseError<&'s str>> {
//     let processes: LinkedHashMap<String, String> = parse_procfile_str
//         .parse(input)?
//         .iter()
//         .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
//         .collect();

//     Ok(Procfile { processes })
// }

#[derive(thiserror::Error, Debug)]
pub(crate) enum ProcfileError {
    #[error("Oops {0}")]
    ParseError(ProcfileParseError),
}

#[derive(Debug)]

pub(crate) struct ProcfileParseError {
    message: String,
    // Byte spans are tracked, rather than line and column.
    // This makes it easier to operate on programmatically
    // and doesn't limit us to one definition for column count
    // which can depend on the output medium and application.
    span: std::ops::Range<usize>,
    input: String,
}

pub(crate) struct ProcfileParsed {
    pub(crate) processes: LinkedHashMap<String, String>,
    pub(crate) warnings: Vec<String>,
}

impl ProcfileParseError {
    fn from_parse(error: ParseError<&str, ContextError>, input: &str) -> Self {
        // The default renderer for `ContextError` is still used but that can be
        // customized as well to better fit your needs.
        let message = error.inner().to_string();
        let input = input.to_owned();
        let start = error.offset();
        // Assume the error span is only for the first `char`.
        // Semantic errors are free to choose the entire span returned by `Parser::with_span`.
        let end = (start + 1..)
            .find(|e| input.is_char_boundary(*e))
            .unwrap_or(start);

        Self {
            message,
            span: start..end,
            input,
        }
    }
}

impl std::str::FromStr for ProcfileParsed {
    type Err = ProcfileError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (processes, warnings) = parse_da_procfile
            .parse(input)
            .map_err(|e| ProcfileError::ParseError(ProcfileParseError::from_parse(e, input)))?;

        Ok(ProcfileParsed {
            processes,
            warnings,
        })
    }
}

impl std::fmt::Display for ProcfileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = annotate_snippets::Level::Error
            .title(&self.message)
            .snippet(
                annotate_snippets::Snippet::source(&self.input)
                    .fold(true)
                    .annotation(annotate_snippets::Level::Error.span(self.span.clone())),
            );
        let renderer = annotate_snippets::Renderer::plain();
        let result = renderer.render(message);
        result.fmt(f)
    }
}

fn parse_da_procfile(input: &mut &str) -> PResult<(LinkedHashMap<String, String>, Vec<String>)> {
    let mut warnings: Vec<String> = Vec::new();
    let mut key_values: Vec<(String, String)> = Vec::new();
    let mut out = LinkedHashMap::new();

    while !input.is_empty() {
        match parse_strict_key_val(input) {
            Ok(kv) => {
                key_values.push(kv);
            }
            Err(err) => {
                println!("ONE");
                match parse_permissive_key(input).and_then(|original| {
                    parse_strict_key
                        .parse_next(
                            &mut format!("{}:", original.replace('_', "-").to_ascii_lowercase())
                                .as_str(),
                        )
                        .map(|fixed| (original, fixed))
                }) {
                    Ok((original, fixed)) => {
                        let value = preceded(space0, till_newline_or_eof)
                            .map(std::string::ToString::to_string)
                            .parse_next(input)?;

                        warnings.push(format!("WARNING: Procfile key `{original}` has been corrected to `{fixed}`. Please update your Procfile\n\n{fixed}: {value}"));
                        key_values.push((fixed, value));
                    }
                    Err(_) => return Err(err),
                }
            }
        }
    }

    for (key, value) in key_values {
        if out.contains_key(&key) {
            warnings.push(format!(
                "WARNING: Duplicate key `{key}` found. The value `{value}` will be used."
            ));
        }
        out.insert(key, value);
    }

    Ok((out, warnings))
}

fn parse_strict_procfile_str<'s>(input: &mut &'s str) -> PResult<Vec<(String, String)>> {
    repeat(0.., parse_strict_key_val).parse_next(input)
}

fn parse_strict_key_val<'s>(input: &mut &'s str) -> PResult<(String, String)> {
    opt(parse_ignored_lines).parse_next(input)?;

    let key: String = parse_strict_key
        .context(StrContext::Label("key"))
        .parse_next(input)?;

    let val = preceded(space0, till_newline_or_eof)
        .context(StrContext::Label("value"))
        .map(std::string::ToString::to_string)
        .parse_next(input)?;
    opt(parse_ignored_lines).parse_next(input)?;

    Ok((key, val))
}

fn parse_permissive_key<'s>(input: &mut &'s str) -> PResult<String> {
    trace(
        "permissive key",
        terminated(
            take_while(0.., |c| c != ':').context(StrContext::Expected(
                StrContextValue::Description(
                    "Key must contain only lowercase alphanumeric characters (a-z0-9) and `-`",
                ),
            )),
            ':',
        ),
    )
    .parse_next(input)
    .map(std::string::ToString::to_string)
}

fn parse_permissive_key_val<'s>(input: &mut &'s str) -> PResult<(String, String)> {
    opt(parse_ignored_lines).parse_next(input)?;
    let permissive_key: String = trace(
        "permissive key",
        terminated(
            take_while(0.., |c| c != ':').context(StrContext::Expected(
                StrContextValue::Description(
                    "Key must contain only lowercase alphanumeric characters (a-z0-9) and `-`",
                ),
            )),
            ':',
        ),
    )
    .parse_next(input)
    .map(std::string::ToString::to_string)?;

    let fixed_key = permissive_key.replace('_', "-").to_ascii_lowercase();
    let key = parse_strict_key.parse_next(&mut format!("{fixed_key}:").as_str())?;
    // println!("Re")

    let val = preceded(space0, till_newline_or_eof)
        .map(std::string::ToString::to_string)
        .parse_next(input)?;
    opt(parse_ignored_lines).parse_next(input)?;

    Ok((key, val))
}

/// Pattern represents a strict Procfile key
/// <https://github.com/heroku/buildpacks-procfile/issues/251>
///
/// - Must start and end with a lowercase alphanumeric value ('a'..='z' or '0'..='9')
/// - Inner characters must be lowercase alphanumeric or a dash `-` (underscore `_` and other delimiters are not allowed)
fn parse_strict_key<'s>(input: &mut &'s str) -> PResult<String> {
    alt((
        parse_one_char_key.map(|c| c.to_string()),
        parse_two_or_more_char_key,
    ))
    .parse_next(input)
}

fn parse_lower_alphanum1<'s>(input: &mut &'s str) -> PResult<char> {
    alt((one_of('a'..='z'), one_of('0'..='9'))).parse_next(input)
}

fn parse_one_char_key<'s>(input: &mut &'s str) -> PResult<char> {
    terminated(parse_lower_alphanum1, ':').parse_next(input)
}

use winnow::combinator::trace;

fn parse_two_or_more_char_key<'s>(input: &mut &'s str) -> PResult<String> {
    let key = trace(
        "two or more char keys",
        terminated(
            take_while(1.., |c: char| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'
            })
            .context(StrContext::Expected(StrContextValue::Description(
                "Key must contain only lowercase alphanumeric characters (a-z0-9) and `-`",
            ))),
            ':',
        ),
    )
    .parse_next(input)?;

    Ok(key.to_string())
}

/// Returns all characters up to (but not including) the newline or EOF
fn till_newline_or_eof<'s>(input: &mut &'s str) -> PResult<&'s str> {
    terminated(till_line_ending, alt((line_ending, eof))).parse_next(input)
}

/// Consume comments and empty lines
fn parse_ignored_lines(input: &mut &'_ str) -> PResult<()> {
    trace(
        "ignored lines",
        repeat(1.., alt((parse_comment, (terminated(space0, line_ending))))),
    )
    .parse_next(input)
}

fn parse_comment<'s>(input: &mut &'s str) -> PResult<&'s str> {
    preceded(space0, preceded("#", till_line_ending)).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replacing_upper_with_lowercase_warning() {
        let input = "IamAvalidKeyButNotStrictly: echo 'done'";
        let result: ProcfileParsed = input.parse().unwrap();
        assert_eq!(1, result.warnings.len());
        assert_eq!(&"WARNING: Procfile key `IamAvalidKeyButNotStrictly` has been corrected to `iamavalidkeybutnotstrictly`. Please update your Procfile\n\niamavalidkeybutnotstrictly: echo 'done'".to_string(), result.warnings.last().unwrap());
        assert_eq!(
            "echo 'done'",
            result.processes.get("iamavalidkeybutnotstrictly").unwrap()
        );
    }

    #[test]
    fn test_strictly_valid_one_key_val() {
        let input = "iamastrictly-validkey: echo 'done'";
        let result: ProcfileParsed = input.parse().unwrap();
        assert_eq!(0, result.warnings.len());
        assert_eq!(
            "echo 'done'",
            result.processes.get("iamastrictly-validkey").unwrap()
        );
    }

    #[test]
    fn test_till_line_ending() {
        let mut input = "a\nb\nc";
        let result = till_newline_or_eof(&mut input).unwrap();
        assert_eq!("a", result);
        assert_eq!("b\nc", input);
        let result = till_newline_or_eof(&mut input).unwrap();
        assert_eq!("b", result);
        let result = till_newline_or_eof(&mut input).unwrap();
        assert_eq!("c", result);

        let mut input = " a\n b\n c";
        let result = till_newline_or_eof(&mut input).unwrap();

        assert_eq!(" a", result);
    }

    #[test]
    fn comment_test() {
        let mut input = "# I ama comment";
        let comment = parse_comment(&mut input).unwrap();
        assert_eq!(" I ama comment", comment);

        let mut input = " # Comment with spaces";
        let comment = parse_comment(&mut input).unwrap();
        assert_eq!(" Comment with spaces", comment);

        let mut input = "I am not a comment # <--";
        let result = parse_comment(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn process_key_value() {
        let mut input = "web: rails s\n";
        let (key, val) = parse_strict_key_val(&mut input).unwrap();
        assert_eq!("web", key);
        assert_eq!("rails s", val);
    }

    // #[test]
    // fn test_empty_parse_procfile() {
    //     assert_eq!("".parse::<Procfile>(), Ok(Procfile::new()));
    // }

    // #[test]
    // fn test_valid_parse_procfile() {
    //     let mut expected_procfile = Procfile::new();
    //     expected_procfile.insert("web", "rails s");

    //     assert_eq!("web: rails s".parse::<Procfile>(), Ok(expected_procfile));
    // }

    // #[test]
    // fn test_multiple_valid_parse_procfile() {
    //     let mut expected_procfile = Procfile::new();
    //     expected_procfile.insert("web", "rails s");
    //     expected_procfile.insert("worker", "rake sidekiq");

    //     assert_eq!(
    //         "web: rails s\nworker: rake sidekiq".parse::<Procfile>(),
    //         Ok(expected_procfile)
    //     );
    // }

    // #[test]
    // fn test_nonsense_procfile() {
    //     assert_eq!("&&&&&".parse::<Procfile>(), Ok(Procfile::new()));
    // }

    // #[test]
    // fn test_missing_command_parse_procfile() {
    //     assert_eq!("web:".parse::<Procfile>(), Ok(Procfile::new()));
    // }

    // #[test]
    // fn test_missing_name_parse_procfile() {
    //     assert_eq!(": rails -s".parse::<Procfile>(), Ok(Procfile::new()));
    // }
}
