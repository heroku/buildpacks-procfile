//! Integration tests using libcnb-test.
//!
//! All integration tests are skipped by default (using the `ignore` attribute),
//! since performing builds is slow. To run the tests use: `cargo test -- --ignored`

// Enable Clippy lints that are disabled by default.
#![warn(clippy::pedantic)]

use indoc::indoc;
use libcnb_test::assert_contains;
use libcnb_test::{BuildpackReference, IntegrationTest};

#[test]
#[ignore = "integration test"]
fn test_web_and_worker_procfile() {
    IntegrationTest::new(
        "heroku/builder:22",
        "tests/fixtures/web_and_worker_procfile",
    )
    .buildpacks(vec![BuildpackReference::Crate])
    .run_test(|context| {
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
        context
            .prepare_container()
            .start_with_default_process(|container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the web process!\n");
            });

        context
            .prepare_container()
            .start_with_process(String::from("worker"), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the worker process!\n");
            });
    });
}

#[test]
#[ignore = "integration test"]
fn test_worker_only_procfile() {
    IntegrationTest::new("heroku/builder:22", "tests/fixtures/worker_only_procfile")
        .buildpacks(vec![BuildpackReference::Crate])
        .run_test(|context| {
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
            context
                .prepare_container()
                .start_with_default_process(|container| {
                    let log_output = container.logs_wait();
                    assert_eq!(log_output.stdout, "this is the worker process!\n");
                });
        });
}

#[test]
#[ignore = "integration test"]
fn test_multiple_non_web_procfile() {
    IntegrationTest::new(
        "heroku/builder:22",
        "tests/fixtures/multiple_non_web_procfile",
    )
    .buildpacks(vec![BuildpackReference::Crate])
    .run_test(|context| {
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
        context
            .prepare_container()
            .start_with_default_process(|container| {
                let log_output = container.logs_wait();
                assert_contains!(
                    log_output.stdout,
                    "when there is no default process a command is required"
                );
            });

        context
            .prepare_container()
            .start_with_process(String::from("worker"), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the worker process!\n");
            });

        context
            .prepare_container()
            .start_with_process(String::from("console"), |container| {
                let log_output = container.logs_wait();
                assert_eq!(log_output.stdout, "this is the console process!\n");
            });
    });
}

#[test]
#[ignore = "integration test"]
// Tests a Procfile that happens to not be valid YAML, but is still valid according
// to the supported Procfile syntax.
fn test_not_yaml_procfile() {
    IntegrationTest::new("heroku/builder:22", "tests/fixtures/not_yaml_procfile")
        .buildpacks(vec![BuildpackReference::Crate])
        .run_test(|context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> web
                "}
            );
            assert_contains!(context.pack_stdout, "Setting default process type 'web'");
            context
                .prepare_container()
                .start_with_default_process(|container| {
                    let log_output = container.logs_wait();
                    assert_eq!(log_output.stdout, "foo: bar\n");
                });
        });
}

#[test]
#[ignore = "integration test"]
fn test_empty_procfile() {
    IntegrationTest::new("heroku/builder:22", "tests/fixtures/empty_procfile")
        .buildpacks(vec![BuildpackReference::Crate])
        .run_test(|context| {
            assert_contains!(
                context.pack_stdout,
                indoc! {"
                    [Discovering process types]
                    Procfile declares types -> (none)
                "}
            );
            assert_contains!(context.pack_stdout, "no default process type");
        });
}

#[test]
#[ignore = "integration test"]
#[should_panic(expected = "ERROR: No buildpack groups passed detection.")]
fn test_missing_procfile() {
    IntegrationTest::new("heroku/builder:22", "tests/fixtures/missing_procfile")
        .buildpacks(vec![BuildpackReference::Crate])
        .run_test(|_| {});
}
