use crate::error::BuildpackError;
use libcnb::{
    data::launch::{Launch, Process},
    GenericBuildContext,
};
use std::path::Path;
use yaml_rust::YamlLoader;

/// `bin/build`
// https://github.com/Malax/libcnb.rs/issues/63
#[allow(clippy::needless_pass_by_value)]
pub fn build(context: GenericBuildContext) -> Result<(), libcnb::Error<BuildpackError>> {
    let mut launch = Launch::new();
    launch.processes = parse_procfile(&context.app_dir.join("Procfile"))?;

    context.write_launch(launch).map_err(BuildpackError::from)?;
    Ok(())
}

/// Parse processes from `Procfile`
fn parse_procfile(procfile: &Path) -> Result<Vec<Process>, BuildpackError> {
    let procfile_path = procfile.to_str().unwrap();
    let procfile_contents = std::fs::read_to_string(procfile_path)?;
    let contents = YamlLoader::load_from_str(&procfile_contents)?;

    let processes = contents[0]
        .as_hash()
        .ok_or(BuildpackError::Procfile("Not a valid YAML Hash"))?;

    processes
        .into_iter()
        .map(|(key, value)| {
            let process = Process::new(
                key.as_str().ok_or(BuildpackError::Procfile(
                    "process type name is an empty string",
                ))?,
                // TODO: Split this into separate args
                value.as_str().ok_or(BuildpackError::Procfile(
                    "process command is an empty string",
                ))?,
                Vec::<String>::new(),
                false,
            )?;

            Ok(process)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, str::FromStr};

    use libcnb::data::launch::ProcessType;

    use crate::test_helper::{procfile_fixture_path, TempContext};

    #[test]
    fn test_build() {
        let temp_context = TempContext::new(include_str!("../buildpack.toml"));
        let context = temp_context.build;
        let launch_toml_path = context.layers_dir.join("launch.toml");

        fs::write(context.app_dir.join("Procfile"), "web: bundle exec rails s").unwrap();
        build(context).unwrap();

        let layer_toml_string = fs::read_to_string(launch_toml_path).unwrap();
        let layer_toml = toml::from_str::<toml::value::Table>(&layer_toml_string).unwrap();
        let processes = layer_toml["processes"].as_array().unwrap();
        let process = &processes[0];

        assert_eq!("web", process["type"].as_str().unwrap());
        assert_eq!("bundle exec rails s", process["command"].as_str().unwrap());
        assert_eq!(1, processes.len());
    }

    #[test]
    fn test_parse_procfile() {
        let processes = parse_procfile(&procfile_fixture_path("app_with_procfile")).unwrap();

        assert_eq!(
            ProcessType::from_str("web").unwrap().as_str(),
            processes[0].r#type.as_str()
        );
        assert_eq!("node index.js", processes[0].command);
        assert_eq!(
            ProcessType::from_str("worker").unwrap().as_str(),
            processes[1].r#type.as_str()
        );

        assert_eq!(
            "while true; do echo 'lol'; sleep 2; done",
            processes[1].command
        );
    }
}
