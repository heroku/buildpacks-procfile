use crate::launch::ProcfileConversionError;
use crate::procfile::ProcfileParsingError;
use indoc::formatdoc;

#[derive(Debug)]
pub enum ProcfileBuildpackError {
    CannotReadProcfileContents(std::io::Error),
    Parsing(ProcfileParsingError),
    Conversion(ProcfileConversionError),
}

pub fn error_handler(buildpack_error: ProcfileBuildpackError) -> i32 {
    match buildpack_error {
        ProcfileBuildpackError::CannotReadProcfileContents(io_error) => {
            libherokubuildpack::log_error(
                "Cannot read Procfile contents",
                formatdoc! {"
                    Please ensure the Procfile in the root of your application is a readable UTF-8
                    encoded file and try again.

                    Underlying cause was: {io_error}
                "},
            );
        }
        ProcfileBuildpackError::Parsing(parsing_error) => match parsing_error {
            ProcfileParsingError::InvalidRegex(regex_error) => {
                libherokubuildpack::log_error(
                    "Cannot compile Procfile regex",
                    formatdoc! {"
                        This is an unexpected internal error that occurs when the regex used to parse
                        the Procfile cannot be compiled.

                        Please report this issue with the details below.

                        Details: {regex_error}
                    "},
                );
            }
        },
        ProcfileBuildpackError::Conversion(conversion_error) => match conversion_error {
            ProcfileConversionError::InvalidProcessType(libcnb_error) => {
                libherokubuildpack::log_error(
                    "Cannot convert Procfile to CNB launch configuration",
                    formatdoc! {"
                        This is an unexpected internal error that occurs when a Procfile entry is not
                        compatible with the CNB launch configuration. At the time of writing, Procfile
                        process names are a strict subset of CNB launch process names and this should
                        never happen.

                        Please report this issue with the details below.

                        Details: {libcnb_error}
                    "},
                );
            }
        },
    }

    1
}

impl From<ProcfileBuildpackError> for libcnb::Error<ProcfileBuildpackError> {
    fn from(error: ProcfileBuildpackError) -> Self {
        Self::BuildpackError(error)
    }
}
