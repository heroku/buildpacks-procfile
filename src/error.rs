use crate::launch::ProcfileConversionError;
use crate::procfile::ProcfileParsingError;
use indoc::formatdoc;

#[derive(thiserror::Error, Debug)]
pub enum ProcfileBuildpackError {
    #[error("Cannot read Procfile contents: {0}")]
    CannotReadProcfileContents(std::io::Error),
    #[error("Procfile parsing error: {0}")]
    ProcfileParsingError(#[from] ProcfileParsingError),
    #[error("Procfile conversion error: {0}")]
    ProcfileConversionError(#[from] ProcfileConversionError),
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
                ",
                io_error = io_error
                },
            );
        }
        ProcfileBuildpackError::ProcfileParsingError(parsing_error) => {
            libherokubuildpack::log_error(
                "Cannot parse Procfile",
                formatdoc! {"
                    Please ensure the Procfile in the root of your application is a readable UTF-8
                    encoded Procfile as specified on Heroku DevCenter and try again:

                    https://devcenter.heroku.com/articles/procfile

                    Underlying cause was: {parsing_error}
                ",
                parsing_error = parsing_error
                },
            );
        }
        ProcfileBuildpackError::ProcfileConversionError(conversion_error) => {
            libherokubuildpack::log_error(
                "Cannot convert Procfile to CNB launch configuration",
                formatdoc! {"
                    This is an unexpected internal error that occurs when a Procfile entry is not
                    compatible with the CNB launch configuration. At the time of writing, Procfile
                    process names are a strict subset of CNB launch process names and this should
                    never happen.

                    Please report this issue with the details below.

                    Details: {conversion_error}
                ",
                conversion_error = conversion_error
                },
            );
        }
    }

    1
}

impl From<ProcfileBuildpackError> for libcnb::Error<ProcfileBuildpackError> {
    fn from(error: ProcfileBuildpackError) -> Self {
        Self::BuildpackError(error)
    }
}
