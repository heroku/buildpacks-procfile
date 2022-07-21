//! Integration tests using libcnb-test.
//!
//! All integration tests are skipped by default (using the `ignore` attribute),
//! since performing builds is slow. To run the tests use: `cargo test -- --ignored`

// Enable Clippy lints that are disabled by default.
#![warn(clippy::pedantic)]

use indoc::indoc;
use libcnb_test::{assert_contains, BuildConfig, ContainerConfig, PackResult, TestRunner};

#[test]
#[ignore = "integration test"]
fn test_web_and_worker_procfile() {
    TestRunner::default().build(
        BuildConfig::new(
            "heroku/builder:22",
            "tests/fixtures/web_and_worker_procfile",
        ),
        |context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> web, worker
                "}
            );

            // When there is a web process type, it should be made the default even if there
            // are multiple process types declared.
            assert_contains!(context.pack_stdout, "Setting default process type 'web'");
            context.start_container(ContainerConfig::new(), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the web process!\n");
            });

            context.start_container(ContainerConfig::new().entrypoint(["worker"]), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the worker process!\n");
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_worker_only_procfile() {
    TestRunner::default().build(
        BuildConfig::new("heroku/builder:22", "tests/fixtures/worker_only_procfile"),
        |context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> worker
                "}
            );

            // When there is only one process type, it should be made the default process
            // type even when it doesn't have the name "web".
            assert_contains!(context.pack_stdout, "Setting default process type 'worker'");
            context.start_container(ContainerConfig::new(), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the worker process!\n");
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_multiple_non_web_procfile() {
    TestRunner::default().build(
        BuildConfig::new(
            "heroku/builder:22",
            "tests/fixtures/multiple_non_web_procfile",
        ),
        |context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> worker, console
                "}
            );

            // When there are multiple process types, and none of them has name "web",
            // then none of them should be set as the default process type.
            assert_contains!(context.pack_stdout, "no default process type");
            context.start_container(ContainerConfig::new(), |container| {
                let log_output = container.logs_wait();
                assert_contains!(
                    log_output.stdout,
                    "when there is no default process a command is required"
                );
            });

            context.start_container(ContainerConfig::new().entrypoint(["worker"]), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the worker process!\n");
            });

            context.start_container(
                ContainerConfig::new().entrypoint(["console"]),
                |container| {
                    let log_output = container.logs_wait();
                    assert_eq!(log_output.stdout, "this is the console process!\n");
                },
            );
        },
    );
}

#[test]
#[ignore = "integration test"]
// Tests a Procfile that happens to not be valid YAML, but is still valid according
// to the supported Procfile syntax.
fn test_not_yaml_procfile() {
    TestRunner::default().build(
        BuildConfig::new("heroku/builder:22", "tests/fixtures/not_yaml_procfile"),
        |context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> web
                "}
            );
            assert_contains!(context.pack_stdout, "Setting default process type 'web'");
            context.start_container(ContainerConfig::new(), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "foo: bar\n");
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_empty_procfile() {
    TestRunner::default().build(
        BuildConfig::new("heroku/builder:22", "tests/fixtures/empty_procfile"),
        |context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> (none)
                "}
            );
            assert_contains!(context.pack_stdout, "no default process type");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_missing_procfile() {
    TestRunner::default().build(
        BuildConfig::new("heroku/builder:22", "tests/fixtures/missing_procfile")
            .expected_pack_result(PackResult::Failure),
        |context| {
            assert_contains!(
                context.pack_stdout,
                "ERROR: No buildpack groups passed detection."
            );
        },
    );
}
