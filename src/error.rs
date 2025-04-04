use crate::launch::ProcfileConversionError;
use crate::procfile::ProcfileError;
use bullet_stream::Print;
use indoc::formatdoc;

#[derive(Debug)]
pub(crate) enum ProcfileBuildpackError {
    CannotReadProcfileContents(std::io::Error),
    ProcfileParsingError(ProcfileError),
    ProcfileConversionError(ProcfileConversionError),
}

pub(crate) fn error_handler(buildpack_error: ProcfileBuildpackError) {
    let build_output = Print::new(std::io::stdout()).without_header();
    match buildpack_error {
        ProcfileBuildpackError::CannotReadProcfileContents(io_error) => {
            build_output.error(formatdoc! {"
                Cannot read Procfile contents

                Please ensure the Procfile in the root of your application is a readable UTF-8
                encoded file and try again.

                Underlying cause was: {io_error}
            "});
        }
        ProcfileBuildpackError::ProcfileParsingError(parsing_error) => {
            build_output.error(formatdoc! {"
                Invalid Procfile format

                The provided `Procfile` contains an invalid format and the buildpack cannot continue.

                To fix this problem please correct the following error and commit the results to git:

                {parsing_error}
            "});
        }
        ProcfileBuildpackError::ProcfileConversionError(conversion_error) => match conversion_error
        {
            ProcfileConversionError::InvalidProcessType(libcnb_error) => {
                build_output.error(formatdoc! {"
                    Cannot convert Procfile to CNB launch configuration

                    This is an unexpected internal error that occurs when a Procfile entry is not
                    compatible with the CNB launch configuration. At the time of writing, Procfile
                    process names are a strict subset of CNB launch process names and this should
                    never happen.

                    Please report this issue with the details below.

                    Details: {libcnb_error}
                "});
            }
        },
    }
}

impl From<ProcfileBuildpackError> for libcnb::Error<ProcfileBuildpackError> {
    fn from(error: ProcfileBuildpackError) -> Self {
        Self::BuildpackError(error)
    }
}
