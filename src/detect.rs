use crate::BuildpackError;
use libcnb::{data::build_plan::BuildPlan, DetectOutcome, GenericDetectContext, Result};

/// `bin/detect`
pub fn detect(context: GenericDetectContext) -> Result<DetectOutcome, BuildpackError> {
    let procfile_path = context.app_dir.join("Procfile");

    if procfile_path.exists() {
        let buildplan = BuildPlan::new();
        Ok(DetectOutcome::Pass(buildplan))
    } else {
        Ok(DetectOutcome::Fail)
    }
}
