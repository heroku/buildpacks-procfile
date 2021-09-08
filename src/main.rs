use libcnb::{
    cnb_runtime,
    data::{
        build_plan::BuildPlan,
        launch::{Launch, Process},
    },
    DetectOutcome, GenericBuildContext, GenericDetectContext, GenericErrorHandler,
};
use std::path::PathBuf;
use yaml_rust::YamlLoader;

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

// Main entrypoint, the `cnb_runtime` produces a single binary
// that will call either `detect` or `build` functions based on the name of the
// binary file.
fn main() {
    cnb_runtime(detect, build, GenericErrorHandler);
}

// Code for `bin/detect`
fn detect(context: GenericDetectContext) -> Result<DetectOutcome, libcnb::Error<BuildpackError>> {
    let procfile_path = context.app_dir.join("Procfile");

    if procfile_path.exists() {
        let buildplan = BuildPlan::new();
        Ok(DetectOutcome::Pass(buildplan))
    } else {
        Ok(DetectOutcome::Fail)
    }
}

// Code for `bin/build`
fn build(context: GenericBuildContext) -> Result<(), libcnb::Error<BuildpackError>> {
    let launch = launch_from_procfile(context.app_dir.join("Procfile"))?;

    context
        .write_launch(launch)
        .map_err(|e| BuildpackError::from(e))?;
    Ok(())
}

// Bulk of logic extracted for testing
fn launch_from_procfile(procfile: PathBuf) -> Result<libcnb::data::launch::Launch, BuildpackError> {
    let procfile_path = procfile.to_str().unwrap();
    let procfile_contents = std::fs::read_to_string(procfile_path)?;
    let contents = YamlLoader::load_from_str(&procfile_contents)?;

    let mut launch = Launch::new();
    let processes = contents[0]
        .as_hash()
        .ok_or(BuildpackError::Procfile("Not a valid YAML Hash"))?;
    for (key, value) in processes {
        let p = Process::new(
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

        launch.processes.push(p);
    }

    Ok(launch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;

    use tempfile::{tempdir, TempDir};

    use libcnb::{BuildContext, GenericPlatform, Platform};

    use libcnb::data::{
        buildpack::BuildpackToml, buildpack_plan::BuildpackPlan, buildpack_plan::Entry,
        launch::ProcessType,
    };

    #[test]
    fn test_build() {
        let tmp_context = make_temp_context();
        let context = tmp_context.context;
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
    fn test_launch_from_procfile() {
        let launch = launch_from_procfile(procfile_fixture_path("app_with_procfile")).unwrap();

        assert_eq!(
            ProcessType::from_str("web").unwrap().as_str(),
            launch.processes[0].r#type.as_str()
        );
        assert_eq!("node index.js", launch.processes[0].command);
        assert_eq!(
            ProcessType::from_str("worker").unwrap().as_str(),
            launch.processes[1].r#type.as_str()
        );

        assert_eq!("node worker.js", launch.processes[1].command);
    }

    struct TempContext {
        // Hold reference to temp dirs so they're not cleaned off disk
        // https://heroku.slack.com/archives/CFF88C0HM/p1631124162001800
        _tmp_dirs: Vec<TempDir>,
        context: GenericBuildContext,
    }

    fn make_temp_context() -> TempContext {
        let bp_temp = tempdir().unwrap();
        let app_temp = tempdir().unwrap();
        let layers_temp = tempdir().unwrap();

        let bp_dir = bp_temp.path().to_owned();
        let app_dir = app_temp.path().to_owned();
        let layers_dir = layers_temp.path().to_owned();

        let context = BuildContext {
            layers_dir,
            app_dir,
            buildpack_dir: PathBuf::new(),
            stack_id: String::from("lol"),
            platform: GenericPlatform::from_path(bp_dir).unwrap(),
            buildpack_plan: BuildpackPlan {
                entries: Vec::<Entry>::new(),
            },
            buildpack_descriptor: toml::from_str::<BuildpackToml<Option<toml::value::Table>>>(
                r#"
    api = "0.4"

    [buildpack]
    id = "foo/bar"
    name = "Bar Buildpack"
    version = "0.0.1"

    [[stacks]]
    id = "io.buildpacks.stacks.bionic"

            "#,
            )
            .unwrap(),
        };
        TempContext {
            _tmp_dirs: vec![bp_temp, app_temp, layers_temp],
            context,
        }
    }

    fn procfile_fixture_path(fixture_name: &str) -> PathBuf {
        let path = env::current_dir().unwrap();
        path.join("../test/fixtures")
            .join(fixture_name)
            .join("Procfile")
    }
}
