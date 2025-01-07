use crate::launch::ProcfileConversionError;
use crate::procfile::ProcfileParsingError;
use indoc::formatdoc;
use libherokubuildpack::log::log_error;

#[derive(Debug)]
pub(crate) enum ProcfileBuildpackError {
    CannotReadProcfileContents(std::io::Error),
    ProcfileConversionError(ProcfileConversionError),
    InvalidProcfile(ProcfileParsingError),
}

pub(crate) fn error_handler(buildpack_error: ProcfileBuildpackError) {
    match buildpack_error {
        ProcfileBuildpackError::CannotReadProcfileContents(io_error) => {
            log_error(
                "Cannot read Procfile contents",
                formatdoc! {"
                    Please ensure the Procfile in the root of your application is a readable UTF-8
                    encoded file and try again.

                    Underlying cause was: {io_error}
                "},
            );
        }
        ProcfileBuildpackError::ProcfileConversionError(conversion_error) => match conversion_error
        {
            ProcfileConversionError::InvalidProcessType(libcnb_error) => {
                log_error(
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
        ProcfileBuildpackError::InvalidProcfile(procfile_error) => log_error(
            "Invalid Procfile format",
            formatdoc! {"
                The provided `Procfile` contains an invalid format and the buildpack cannot continue.

                To fix this problem please correct the following error and commit the results to git:

                {procfile_error}
            "},
        ),
    }
}

impl From<ProcfileBuildpackError> for libcnb::Error<ProcfileBuildpackError> {
    fn from(error: ProcfileBuildpackError) -> Self {
        Self::BuildpackError(error)
    }
}
