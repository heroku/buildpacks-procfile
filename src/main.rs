use libcnb::data::build_plan::BuildPlan;
use libcnb::data::launch::{Launch, Process, ProcessTypeError};

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
    println!("Build runs on stack {}!", context.stack_id);

    let process = Process::new(
        "web",
        String::from("while true; do echo 'lol'; sleep 2; done"),
        vec![String::from("")],
        false,
    )?;
    let launch = Launch::default().process(process);

    context
        .write_launch(launch).unwrap();
    Ok(())
}
