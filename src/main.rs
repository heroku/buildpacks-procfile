use libcnb::data::build_plan::BuildPlan;
use libcnb::data::launch::{Launch, Process, ProcessTypeError};

use yaml_rust::{YamlLoader, YamlEmitter};

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
	// println("[INFO] Discovering process types")
    let procfile = context.app_dir.join("Procfile");
    let mut launch_vec = vec![];

    println!("===============================");

    if procfile.as_path().exists() {
        let procfile_path = procfile.to_str().unwrap();
        
        let procfile_contents = std::fs::read_to_string(procfile_path).unwrap();
        let contents = YamlLoader::load_from_str(
            &procfile_contents
        ).unwrap();

        // println!("{:?}", contents[0]);

        let first = contents[0].as_hash().unwrap();
        for (key, value) in &*first{
            let p= Process::new(
                key.as_str().unwrap(),
                value.as_str().unwrap(),
                Vec::<String>::new(),
                false,
            )?;
            launch_vec.push(Launch::default().process(p));
        }
    } else {
    }

    for launch in launch_vec {
        context
            .write_launch(launch).unwrap();   
    }
    Ok(())
}
