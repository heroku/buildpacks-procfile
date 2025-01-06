use linked_hash_map::LinkedHashMap;
use winnow::combinator::{alt, eof, repeat_till};
use winnow::error::{StrContext, StrContextValue};
use winnow::token::{one_of, take_while};
use winnow::Parser;
use winnow::{
    ascii::space0,
    combinator::{opt, preceded, repeat, terminated},
    error::{ContextError, ParseError},
};
use winnow::{
    ascii::{line_ending, till_line_ending},
    prelude::*,
};

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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ProcfileParsed {
    pub(crate) processes: LinkedHashMap<String, String>,
    pub(crate) warnings: Vec<String>,
}

impl ProcfileParsed {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            processes: LinkedHashMap::new(),
            warnings: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, key: &str, value: &str) {
        self.processes.insert(key.to_string(), value.to_string());
    }
}

impl ProcfileParseError {
    fn from_parse(error: &ParseError<&str, ContextError>, input: &str) -> Self {
        // The default renderer for `ContextError` is still used but that can be
        // customized as well to better fit your needs.
        let message = error.inner().to_string();
        let input = input.to_owned();
        let start = error.offset();
        // Assume the error span is only for the first `char`.
        // Semantic errors are free to choose the entire span returned by `Parser::with_span`.
        let end = (start + 1..input.len())
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
        let (processes, mut warnings) = parse_da_procfile
            .parse(input)
            .map_err(|e| ProcfileError::ParseError(ProcfileParseError::from_parse(&e, input)))?;

        if processes.is_empty() {
            warnings.push("Empty file, no processes defined".to_string());
        }

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

                        warnings.push(format!("Procfile key `{original}` has been corrected to `{fixed}`. Please update your Procfile\n\n{fixed}: {value}"));
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
                "Duplicate key `{key}` found. The value `{value}` will be used."
            ));
        }
        out.insert(key, value);
    }

    Ok((out, warnings))
}

fn parse_strict_key_val(input: &mut &str) -> PResult<(String, String)> {
    opt(parse_ignored_lines).parse_next(input)?;

    let key: String = parse_strict_key
        .context(StrContext::Label("key"))
        .parse_next(input)?;

    let val = preceded(space0, till_newline_or_eof)
        .verify(|value: &str| !value.is_empty())
        .context(StrContext::Label("value"))
        .map(std::string::ToString::to_string)
        .parse_next(input)?;
    opt(parse_ignored_lines).parse_next(input)?;

    Ok((key, val))
}

fn parse_permissive_key(input: &mut &str) -> PResult<String> {
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

/// Pattern represents a strict Procfile key
/// <https://github.com/heroku/buildpacks-procfile/issues/251>
///
/// - Must start and end with a lowercase alphanumeric value ('a'..='z' or '0'..='9')
/// - Inner characters must be lowercase alphanumeric or a dash `-` (underscore `_` and other delimiters are not allowed)
fn parse_strict_key(input: &mut &str) -> PResult<String> {
    alt((
        terminated(parse_lower_alphanum1, ':').map(|c| c.to_string()),
        parse_two_or_more_char_key,
    ))
    .parse_next(input)
}

fn parse_lower_alphanum1(input: &mut &str) -> PResult<char> {
    alt((
        one_of('a'..='z').context(StrContext::Expected(StrContextValue::Description("alpha"))),
        one_of('0'..='9').context(StrContext::Expected(StrContextValue::Description("num"))),
    ))
    .context(StrContext::Expected(StrContextValue::Description(
        "alphanumeric value (a-z0-9)",
    )))
    .parse_next(input)
}

use winnow::combinator::trace;

fn parse_two_or_more_char_key(input: &mut &str) -> PResult<String> {
    (
        parse_lower_alphanum1
            .context(StrContext::Label("first key character"))
            .context(StrContext::Expected(StrContextValue::Description(
                "lowercase alphanumeric (a-z0-9)",
            ))),
        repeat_till(
            0..,
            alt((parse_lower_alphanum1, '-'))
                .context(StrContext::Label("inner key character"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "lowercase alphanumeric (a-z0-9) or `-`",
                ))),
            (
                parse_lower_alphanum1
                    .context(StrContext::Label("last key character"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "lowercase alphanumeric (a-z0-9)",
                    ))),
                ':'.context(StrContext::Label("key delimiter"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "colon `:`",
                    ))),
            )
                .map(|(last, _delimiter)| last),
        )
        .map(|(middle, last): (String, char)| {
            //
            let mut tail = String::new();
            tail.push_str(&middle);
            tail.push(last);
            tail
        }),
    )
        .map(|(start, tail)| {
            let mut key = String::new();
            key.push(start);
            key.push_str(&tail);
            key
        })
        .parse_next(input)
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
        assert_eq!(&"Procfile key `IamAvalidKeyButNotStrictly` has been corrected to `iamavalidkeybutnotstrictly`. Please update your Procfile\n\niamavalidkeybutnotstrictly: echo 'done'".to_string(), result.warnings.last().unwrap());
        assert_eq!(
            "echo 'done'",
            result.processes.get("iamavalidkeybutnotstrictly").unwrap()
        );
    }

    #[test]
    fn test_invalid_start() {
        let input = "-key: echo 'done'";
        assert!(input.parse::<ProcfileParsed>().is_err());
    }

    #[test]
    fn test_invalid_end() {
        let input = "key-: echo 'done'";
        assert!(input.parse::<ProcfileParsed>().is_err());
    }

    #[test]
    fn test_one_char_key() {
        let input = "a: echo 'done'";
        input.parse::<ProcfileParsed>().unwrap();
    }

    #[test]
    fn test_two_char_key() {
        let input = "aa: echo 'done'";
        input.parse::<ProcfileParsed>().unwrap();
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
        let (key, val) = parse_strict_key_val.parse("web: rails s").unwrap();
        assert_eq!("web", key);
        assert_eq!("rails s", val);
    }

    #[test]
    fn test_empty_parse_procfile() {
        let procfile = "".parse::<ProcfileParsed>().unwrap();
        assert_eq!(0, procfile.processes.len());
        assert_eq!(
            &"Empty file, no processes defined".to_string(),
            procfile.warnings.first().unwrap()
        );
        assert_eq!(1, procfile.warnings.len());
    }

    #[test]
    fn test_valid_parse_procfile() {
        let mut expected_procfile = ProcfileParsed {
            processes: LinkedHashMap::new(),
            warnings: Vec::new(),
        };
        expected_procfile
            .processes
            .insert("web".to_string(), "rails s".to_string());

        assert_eq!(
            expected_procfile,
            "web: rails s".parse::<ProcfileParsed>().unwrap()
        );
    }

    #[test]
    fn test_multiple_valid_parse_procfile() {
        let mut expected_procfile = ProcfileParsed {
            processes: LinkedHashMap::new(),
            warnings: Vec::new(),
        };
        expected_procfile
            .processes
            .insert("web".to_string(), "rails s".to_string());
        expected_procfile
            .processes
            .insert("worker".to_string(), "rake sidekiq".to_string());

        assert_eq!(
            expected_procfile,
            "web: rails s\nworker: rake sidekiq"
                .parse::<ProcfileParsed>()
                .unwrap(),
        );
    }

    #[test]
    fn test_nonsense_procfile() {
        assert!("&&&&&".parse::<ProcfileParsed>().is_err());
    }

    #[test]
    fn test_missing_command_parse_procfile() {
        assert!("web:".parse::<ProcfileParsed>().is_err());
    }

    #[test]
    fn test_missing_name_parse_procfile() {
        assert!(": rails -s".parse::<ProcfileParsed>().is_err());
    }
}
