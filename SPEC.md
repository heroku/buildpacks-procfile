# CNB Procfile format specification

This document outlines the format of a `Procfile`, which defines names to process types. For example:

```
web: rails s
worker: bundle exec sidekiq
```

## Differences from Heroku "classic" Procfile

The classic `Procfile` has no formal specification. It is loosely defined based on a regex `"^[[:space:]]*([a-zA-Z0-9_-]+):?\\s+(.*)[[:space:]]*`. This specification is informed by the CNB specification for process names and [kubernetes](https://github.com/heroku/buildpacks-procfile/issues/251).

## Specification

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

- Spaces
  - The term spaces refers to non-newline whitespace characters such as tab `'\t'` or space `'\s'` characters.
- Each line MUST contain a comment, empty line, or key/value pair.
- Comments
  - A line MAY contain a comment
  - Comments MUST contain a `#` as the first visible character on each line.
  - A comment MAY be proceeded by one or more spaces.
- Empty line
  - A Procfile MAY contain empty lines
  - An empty line MUST be zero or more spaces followed by a line ending or end of file (EOF).
- Key/Value pairs
  - A line MAY contain a key/value pair where the key represents the name of a process and the value represents a command
  - A key MUST be separated from its value by a colon (`:`) followed by zero or more spaces.
- Key
  - A key's first and last character MUST be a lowercase alphanumeric (a-z0-9) character (but not `-`).
  - All other key (middle) characters MUST be lowercase alphanumeric (a-z0-9) characters or hyphen `-`.
  - Key length MUST be within the range `1..=63`
  - An implementation MAY accept `_` as a middle character provided it converts it to `-` and issues a warning.
  - An implementation MAY accept an uppercase character provided it is converted to lowercase characters and issues a warning.
  - A key MAY be preceded with zero or more spaces provided they are not included in the return key and a warning is issued.
- Value
  - A value MUST contain 1 or more non-whitespace characters.
  - A value MUST be terminated by a newline or EOF.
