use crate::error::BuildpackError;
use libcnb::{data::build_plan::BuildPlan, DetectOutcome, GenericDetectContext, Result};

/// `bin/detect`
// https://github.com/Malax/libcnb.rs/issues/63
#[allow(clippy::needless_pass_by_value)]
// https://github.com/Malax/libcnb.rs/issues/86
#[allow(clippy::unnecessary_wraps)]
pub fn detect(context: GenericDetectContext) -> Result<DetectOutcome, BuildpackError> {
    let procfile_path = context.app_dir.join("Procfile");

    if procfile_path.exists() {
        let buildplan = BuildPlan::new();
        Ok(DetectOutcome::Pass(buildplan))
    } else {
        Ok(DetectOutcome::Fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::fs;

    use crate::test_helper::TempContext;
    use libcnb::DetectOutcome;

    #[test]
    fn it_fails_if_no_procfile() {
        let temp_context = TempContext::new(include_str!("../buildpack.toml"));
        let result = detect(temp_context.detect);

        assert_matches!(result.unwrap(), DetectOutcome::Fail);
    }

    #[test]
    fn it_passes_detect_if_finds_procfile() {
        let temp_context = TempContext::new(include_str!("../buildpack.toml"));
        fs::write(temp_context.detect.app_dir.join("Procfile"), "").unwrap();
        let result = detect(temp_context.detect);

        assert_matches!(result.unwrap(), DetectOutcome::Pass(_));
    }
}
