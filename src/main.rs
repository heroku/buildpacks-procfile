use libcnb::data::build_plan::BuildPlan;
use libcnb::data::launch::{Launch, Process};
use std::path::PathBuf;

use yaml_rust::YamlLoader;

use libcnb::{
    cnb_runtime, DetectOutcome, GenericBuildContext, GenericDetectContext, GenericErrorHandler,
    Result,
};

fn main() {
    cnb_runtime(detect, build, GenericErrorHandler);
}

fn detect(context: GenericDetectContext) -> Result<DetectOutcome, std::io::Error> {
    let procfile_path = context.app_dir.join("Procfile");

    if procfile_path.exists() {
        let buildplan = BuildPlan::new();
        Ok(DetectOutcome::Pass(buildplan))
    } else {
        Ok(DetectOutcome::Fail)
    }
}

fn build(context: GenericBuildContext) -> Result<(), std::io::Error> {
    let launch = launch_from_procfile(context.app_dir.join("Procfile"))?;

    context.write_launch(launch).unwrap();
    Ok(())
}

fn launch_from_procfile(procfile: PathBuf) -> Result<libcnb::data::launch::Launch, std::io::Error> {
    let procfile_path = procfile.to_str().unwrap();
    let procfile_contents = std::fs::read_to_string(procfile_path).unwrap();
    let contents = YamlLoader::load_from_str(&procfile_contents).unwrap();

    let mut launch = Launch::new();
    for (key, value) in &*contents[0].as_hash().unwrap() {
        let p = Process::new(
            key.as_str().unwrap(),
            value.as_str().unwrap(),
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

    use tempfile::tempdir;

    use libcnb::{
        Platform,
        GenericPlatform,
        BuildContext,
    };

    use libcnb::data::{
        buildpack_plan::BuildpackPlan, 
        buildpack_plan::Entry, 
        launch::ProcessType,
        buildpack::BuildpackToml,
    };

    #[test]
    fn test_build() {
        let bp_temp = tempdir().unwrap();
        let app_temp = tempdir().unwrap();
        let layer_temp = tempdir().unwrap();

        let bp_dir = bp_temp.path();
        let app_dir = app_temp.path();
        let layer_dir = layer_temp.path();

        fs::write(app_dir.join("Procfile"), "web: bundle exec rails s").unwrap();

        build(BuildContext {
            layers_dir: layer_dir.to_path_buf(),
            app_dir: app_dir.to_path_buf(),
            buildpack_dir: PathBuf::new(),
            stack_id: String::from("lol"),
            platform: GenericPlatform::from_path(bp_dir).unwrap(),
            buildpack_plan: BuildpackPlan { entries: Vec::<Entry>::new() },
            buildpack_descriptor: toml::from_str::<BuildpackToml<Option<toml::value::Table>>>(
                r#"
api = "0.4"

[buildpack]
id = "foo/bar"
name = "Bar Buildpack"
version = "0.0.1"

[[stacks]]
id = "io.buildpacks.stacks.bionic"

            "#).unwrap(),
        }).unwrap();

        let layer_toml = fs::read_to_string(layer_dir.join("launch.toml")).unwrap();
        let result = toml::from_str::<toml::value::Table>(&layer_toml).unwrap();
        let processes = result["processes"].as_array().unwrap();
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

    fn procfile_fixture_path(fixture_name: &str) -> PathBuf {
        let path = env::current_dir().unwrap();
        path.join("../test/fixtures")
            .join(fixture_name)
            .join("Procfile")
    }
}
