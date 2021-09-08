/// Buildpack Error Handling
#[derive(thiserror::Error, Debug)]
pub enum BuildpackError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Procfile YAML Parsing Error: {0}")]
    YamlScan(#[from] yaml_rust::scanner::ScanError),
    #[error("Procfile is not in a valid format: {0}")]
    Procfile(&'static str),
    #[error("Invalid ProcessType name: {0}")]
    ProcessType(#[from] libcnb::data::launch::ProcessTypeError),
    #[error("TOML Error: {0}")]
    Toml(#[from] libcnb::TomlFileError),
}

impl From<BuildpackError> for libcnb::Error<BuildpackError> {
    fn from(error: BuildpackError) -> Self {
        Self::BuildpackError(error)
    }
}
