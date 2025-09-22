//! Contains logic for parsing the `Procfile` format
use bullet_stream::style;
use linked_hash_map::LinkedHashMap;
use std::fmt::Display;
use winnow::{
    Parser,
    ascii::{line_ending, space0, till_line_ending},
    combinator::{alt, eof, opt, preceded, repeat, repeat_till, terminated, trace},
    error::{ContextError, ParseError, StrContext, StrContextValue},
    stream::{Offset, Stream},
    token::{one_of, take_while},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Procfile {
    pub(crate) processes: LinkedHashMap<String, String>,
    pub(crate) warnings: Vec<String>,
}

impl Procfile {
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

#[derive(Debug)]
pub(crate) enum ProcfileError {
    ParseError(ProcfileParseError),
}

impl Display for ProcfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcfileError::ParseError(error) => write!(f, "{error}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProcfileParseError {
    message: String,
    span: std::ops::Range<usize>,
    input: String,
}

impl ProcfileParseError {
    fn from_parse(error: &ParseError<&str, ContextError>, input: &str) -> Self {
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

impl std::str::FromStr for Procfile {
    type Err = ProcfileError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (processes, mut warnings) = parse_procfile
            .parse(input)
            .map_err(|e| ProcfileError::ParseError(ProcfileParseError::from_parse(&e, input)))?;

        if processes.is_empty() {
            warnings.push("Empty file, no processes defined".to_string());
        }

        Ok(Procfile {
            processes,
            warnings,
        })
    }
}

impl std::fmt::Display for ProcfileParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};

        let report = &[Level::ERROR.primary_title(&self.message).element(
            Snippet::source(&self.input)
                .annotation(AnnotationKind::Primary.span(self.span.clone())),
        )];

        formatter.write_str(&Renderer::plain().render(report))
    }
}

/// Returns a mapping of key/values and warnings from a Procfile
fn parse_procfile(
    input: &mut &str,
) -> winnow::Result<(LinkedHashMap<String, String>, Vec<String>)> {
    let mut warnings: Vec<String> = Vec::new();
    let mut key_values: Vec<(String, String)> = Vec::new();
    let mut out = LinkedHashMap::new();

    while !input.is_empty() {
        opt(parse_ignored_lines).parse_next(input)?;

        let checkpoint = input.checkpoint();
        // Strict path
        if let Ok(kv) = parse_key_value(input) {
            key_values.push(kv);
        } else {
            input.reset(&checkpoint);
            match parse_permissive_key_fixed(input) {
                Ok((original, fixed)) => {
                    let value = parse_value.parse_next(input)?;

                    warnings.push(format!(
                        "Procfile key {} has been corrected to {}. Please update your Procfile.",
                        style::value(&original),
                        style::value(&fixed)
                    ));
                    key_values.push((fixed, value));
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        opt(parse_ignored_lines).parse_next(input)?;
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

/// Extracts and transforms a semi-valid key or returns an error
///
/// Semi-valid key transformations
/// - Remove spaces at the start
/// - Transform `_` to `-`
/// - Transform uppercase to lowercase characters
///
/// Any other values will be invalid.
///
/// Returns (original, fixed) tuple on success
fn parse_permissive_key_fixed(input: &mut &str) -> winnow::Result<(String, String)> {
    let checkpoint = input.checkpoint();
    let original = parse_permissive_key(input)?;

    let fixed_input_string = format!("{}:", original.replace('_', "-").to_ascii_lowercase());
    let mut fixed_input = fixed_input_string.as_str();
    let before_strict = fixed_input.checkpoint();

    // Remove leading spaces
    opt(space0::<&str, ContextError<&str>>)
        .parse_next(&mut fixed_input)
        .expect("opt cannot fail");
    parse_key
        .parse_next(&mut fixed_input)
        .map(|fixed| (original.to_string(), fixed))
        .inspect_err(|_| {
            // In the event of an error, reset the input and determine what character caused the unrecoverable error
            input.reset(&checkpoint);
            // Determine how far the input moved before an error was encountered (in number of tokens)
            let error_offset = fixed_input.checkpoint().offset_from(&before_strict);
            // Advance the original input that number of tokens
            for _ in 0..error_offset {
                input.next_token();
            }
        })
}

/// A strictly validated single `key: value` pair in a `Procfile`
fn parse_key_value(input: &mut &str) -> winnow::Result<(String, String)> {
    let key: String = parse_key
        .context(StrContext::Label("key"))
        .parse_next(input)?;
    let val = parse_value.parse_next(input)?;

    Ok((key, val))
}

/// Takes all characters terminated by `:`
///
/// Used for transforming `_` to `-` and emitting warnings
fn parse_permissive_key<'s>(input: &mut &'s str) -> winnow::Result<&'s str> {
    terminated(take_while(0.., |c| c != ':'), ':').parse_next(input)
}

/// Pattern represents a strict Procfile key
/// <https://github.com/heroku/buildpacks-procfile/issues/251>
///
/// - Must start and end with a lowercase alphanumeric value ('a'..='z' or '0'..='9')
/// - Inner characters must be lowercase alphanumeric or a dash `-` (underscore `_` and other delimiters are not allowed)
fn parse_key(input: &mut &str) -> winnow::Result<String> {
    alt((
        terminated(parse_lower_alphanum1, ':').map(|key| key.to_string()),
        parse_two_or_more_char_key,
    ))
    .verify(|key: &str| key.chars().count() <= 63)
    .context(StrContext::Expected(StrContextValue::Description(
        "keys contain characters or fewer",
    )))
    .parse_next(input)
}

/// Value part of a `key: value` entry in procfile
fn parse_value(input: &mut &str) -> winnow::Result<String> {
    preceded(space0, till_newline_or_eof)
        .verify(|value: &str| !value.is_empty())
        .context(StrContext::Label("value"))
        .map(std::string::ToString::to_string)
        .parse_next(input)
}

/// Lowercase alphanumeric value
fn parse_lower_alphanum1(input: &mut &str) -> winnow::Result<char> {
    alt((one_of('a'..='z'), one_of('0'..='9')))
        .context(StrContext::Expected(StrContextValue::Description(
            "lowercase alphanumeric value (a-z0-9)",
        )))
        .parse_next(input)
}

/// Returns a key from `key: value` pair that has two or more characters
///
/// First and last character must be lowercase alphanumeric (a-z0-9)
/// Middle characters can be lowercase alphanumeric or `-`
fn parse_two_or_more_char_key(input: &mut &str) -> winnow::Result<String> {
    (
        parse_lower_alphanum1.context(StrContext::Label("first key character")),
        parse_middle_tail_key_chars,
    )
        .map(|(start, tail)| {
            let mut key = String::new();
            key.push(start);
            key.push_str(&tail);
            key
        })
        .parse_next(input)
}

/// Parses middle characters followed by a valid ending character
fn parse_middle_tail_key_chars(input: &mut &str) -> winnow::Result<String> {
    // The `repeat_till` will check the terminator matches before consuming
    // the first parser, this is needed because the ending character is a subset
    // of middle characters
    repeat_till(
        0..,
        alt((parse_lower_alphanum1, '-'))
            .context(StrContext::Label("inner key character"))
            .context(StrContext::Expected(StrContextValue::Description(
                "lowercase alphanum (a-z0-9) or `-`",
            ))),
        (
            parse_lower_alphanum1.context(StrContext::Label("last key character")),
            ':'.context(StrContext::Label("key delimiter"))
                .context(StrContext::Expected(StrContextValue::CharLiteral(':'))),
        )
            .map(|(last, _delimiter)| last),
    )
    .map(|(middle, last): (String, char)| {
        //
        let mut tail = String::new();
        tail.push_str(&middle);
        tail.push(last);
        tail
    })
    .parse_next(input)
}

/// Returns all characters up to (but not including) the newline or EOF
fn till_newline_or_eof<'s>(input: &mut &'s str) -> winnow::Result<&'s str> {
    terminated(till_line_ending, alt((line_ending, eof))).parse_next(input)
}

/// Consume comments and empty lines
fn parse_ignored_lines(input: &mut &'_ str) -> winnow::Result<()> {
    trace(
        "ignored lines",
        repeat(1.., alt((parse_comment, (terminated(space0, line_ending))))),
    )
    .parse_next(input)
}

/// A comment line in a Procfile
///
/// Starts with `#` optionally preceded with spaces
fn parse_comment<'s>(input: &mut &'s str) -> winnow::Result<&'s str> {
    preceded(space0, preceded("#", till_line_ending)).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use libcnb_test::assert_contains;

    #[test]
    fn test_parse_lower_alphanum1() {
        assert!(parse_lower_alphanum1.parse(" ").is_err());
    }

    #[test]
    fn test_replacing_upper_with_lowercase_warning() {
        let input = "IamAvalidKeyButNotStrictly: echo 'done'";
        let result: Procfile = input.parse().unwrap();
        assert_eq!(1, result.warnings.len());
        assert_eq!("Procfile key `IamAvalidKeyButNotStrictly` has been corrected to `iamavalidkeybutnotstrictly`. Please update your Procfile.".to_string(),  strip_ansi(result.warnings.last().unwrap()));
        assert_eq!(
            "echo 'done'",
            result.processes.get("iamavalidkeybutnotstrictly").unwrap()
        );
    }

    #[test]
    fn test_invalid_start() {
        let input = "-key: echo 'done'";
        assert!(input.parse::<Procfile>().is_err());
    }

    #[test]
    fn test_invalid_end() {
        let input = "key-: echo 'done'";
        assert!(input.parse::<Procfile>().is_err());
    }

    #[test]
    fn test_one_char_key() {
        let input = "a: echo 'done'";
        input.parse::<Procfile>().unwrap();
    }

    #[test]
    fn test_two_char_key() {
        let input = "aa: echo 'done'";
        input.parse::<Procfile>().unwrap();
    }

    #[test]
    fn test_strictly_valid_one_key_val_combo() {
        let input = "iamastrictly-validkey: echo 'done'";
        let result: Procfile = input.parse().unwrap();
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
        let (key, val) = parse_key_value.parse("web: rails s").unwrap();
        assert_eq!("web", key);
        assert_eq!("rails s", val);
    }

    #[test]
    fn test_empty_parse_procfile() {
        let procfile = "".parse::<Procfile>().unwrap();
        assert_eq!(0, procfile.processes.len());
        assert_eq!(
            &"Empty file, no processes defined".to_string(),
            procfile.warnings.first().unwrap()
        );
        assert_eq!(1, procfile.warnings.len());
    }

    #[test]
    fn test_valid_parse_procfile() {
        let mut expected_procfile = Procfile::new();
        expected_procfile
            .processes
            .insert("web".to_string(), "rails s".to_string());

        assert_eq!(
            expected_procfile,
            "web: rails s".parse::<Procfile>().unwrap()
        );
    }

    #[test]
    fn test_multiple_valid_parse_procfile() {
        let mut expected_procfile = Procfile::new();
        expected_procfile
            .processes
            .insert("web".to_string(), "rails s".to_string());
        expected_procfile
            .processes
            .insert("worker".to_string(), "rake sidekiq".to_string());

        assert_eq!(
            expected_procfile,
            "web: rails s\nworker: rake sidekiq"
                .parse::<Procfile>()
                .unwrap(),
        );
    }

    #[test]
    fn test_nonsense_procfile() {
        assert!("&&&&&".parse::<Procfile>().is_err());
    }

    #[test]
    fn test_missing_command_parse_procfile() {
        assert!("web:".parse::<Procfile>().is_err());
    }

    #[test]
    fn test_missing_name_parse_procfile() {
        assert!(": rails -s".parse::<Procfile>().is_err());
    }

    #[test]
    fn not_yaml_format_but_still_valid() {
        let input = r"
# Comment

   web: echo foo: bar
";
        let procfile = input.parse::<Procfile>().unwrap();
        assert_eq!(1, procfile.warnings.len());
    }

    #[test]
    fn invalid_procfile_key_points_at_the_correct_location_of_the_fatal_error() {
        let input = "is_w.e.b: echo hello";
        let result = input.parse::<Procfile>();
        assert!(result.is_err());
        match result {
            Ok(_) => panic!("Expected error, got {result:?}"),
            Err(e) => assert_contains!(
                &format!("{e}").trim(),
                r"
1 | is_w.e.b: echo hello
  |     ^
"
                .trim()
            ),
        }
    }

    #[test]
    fn max_length_key_is_63_chars() {
        let input = r"
# 63 chars ---------------------------------------------------v
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: fonz
";
        let result = input.parse::<Procfile>();
        assert!(result.is_ok());

        let input = r"
# 64 chars ----------------------------------------------------v
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: fonz
";
        let result = input.parse::<Procfile>();
        assert!(result.is_err());
    }
}
