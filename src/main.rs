use libcnb::data::build_plan::BuildPlan;
use libcnb::data::launch::{Launch, Process};
use std::path::PathBuf;

use yaml_rust::{YamlLoader};

use libcnb::{
    cnb_runtime, DetectOutcome, GenericBuildContext, GenericDetectContext, GenericErrorHandler,
    Result,
};

fn main() {
    cnb_runtime(detect, build, GenericErrorHandler);
}

fn detect(_context: GenericDetectContext) -> Result<DetectOutcome, std::io::Error> {
    let buildplan = BuildPlan::new();
    Ok(DetectOutcome::Pass(buildplan))
}

fn build(context: GenericBuildContext) -> Result<(), std::io::Error> {
    println!("===============================");
    let launch_vec = launch_vec_from_procfile(
        context.app_dir.join("Procfile")
    )?;

    for launch in launch_vec {
        context
            .write_launch(launch).unwrap();   
    }
    Ok(())
}

fn launch_vec_from_procfile(procfile: PathBuf) -> Result<Vec<libcnb::data::launch::Launch>, std::io::Error> {
    let mut launch_vec = vec![];
    let procfile_path = procfile.to_str().unwrap();
    
    let procfile_contents = std::fs::read_to_string(procfile_path).unwrap();
    let contents = YamlLoader::load_from_str(
        &procfile_contents
    ).unwrap();

    for (key, value) in &*contents[0].as_hash().unwrap(){
        let p= Process::new(
            key.as_str().unwrap(),
            value.as_str().unwrap(),
            Vec::<String>::new(),
            false,
        )?;
        launch_vec.push(Launch::default().process(p));
    }

    Ok(launch_vec)
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    fn procfile_fixture_path(file_name: &str) -> PathBuf {
        let path = env::current_dir().unwrap();
        path.join("../test/fixtures").join(file_name).join("Procfile")
    }

    #[test]
    fn test_lol() {
        let launch_vec = super::launch_vec_from_procfile(
            procfile_fixture_path("app_with_procfile")
        ).unwrap();

        assert_eq!(2, launch_vec.len());
    }
}


