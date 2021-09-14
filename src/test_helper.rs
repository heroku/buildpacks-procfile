use libcnb::{
    data::buildpack::BuildpackToml, data::buildpack_plan::BuildpackPlan,
    data::buildpack_plan::Entry, BuildContext, DetectContext, GenericBuildContext,
    GenericDetectContext, GenericMetadata, GenericPlatform, Platform,
};
use std::{env, fs, path::PathBuf};

pub struct TempContext {
    pub detect: GenericDetectContext,
    pub build: GenericBuildContext,
    _tmp_dir: tempfile::TempDir,
}

impl TempContext {
    pub fn new(buildpack_toml_string: &str) -> Self {
        let tmp_dir = tempfile::tempdir().unwrap();
        let app_dir = tmp_dir.path().join("app");
        let layers_dir = tmp_dir.path().join("layers");
        let platform_dir = tmp_dir.path().join("platform");
        let buildpack_dir = tmp_dir.path().join("buildpack");

        for dir in [&app_dir, &layers_dir, &buildpack_dir, &platform_dir] {
            fs::create_dir_all(dir).unwrap();
        }

        let stack_id = String::from("heroku-20");
        let platform = GenericPlatform::from_path(&platform_dir).unwrap();
        let buildpack_descriptor: BuildpackToml<GenericMetadata> =
            toml::from_str(buildpack_toml_string).unwrap();

        let detect_context = DetectContext {
            platform,
            buildpack_descriptor,
            app_dir: app_dir.clone(),
            buildpack_dir: buildpack_dir.clone(),
            stack_id: stack_id.clone(),
        };

        let platform = GenericPlatform::from_path(&platform_dir).unwrap();
        let buildpack_descriptor: BuildpackToml<GenericMetadata> =
            toml::from_str(buildpack_toml_string).unwrap();
        let buildpack_plan = BuildpackPlan {
            entries: Vec::<Entry>::new(),
        };
        let build_context = BuildContext {
            layers_dir,
            app_dir,
            buildpack_dir,
            stack_id,
            platform,
            buildpack_plan,
            buildpack_descriptor,
        };

        TempContext {
            detect: detect_context,
            build: build_context,
            _tmp_dir: tmp_dir,
        }
    }
}

pub fn procfile_fixture_path(fixture_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/fixtures")
        .join(fixture_name)
        .join("Procfile")
}
