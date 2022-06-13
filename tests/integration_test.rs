//! Integration tests using libcnb-test.
//!
//! All integration tests are skipped by default (using the `ignore` attribute),
//! since performing builds is slow. To run the tests use: `cargo test -- --ignored`

// Enable Clippy lints that are disabled by default.
#![warn(clippy::pedantic)]

use libcnb_test::assert_contains;
use libcnb_test::{BuildpackReference, IntegrationTest};
use std::io;
use std::thread;
use std::time::Duration;

#[test]
#[ignore]
fn test() {
    IntegrationTest::new("heroku/builder:22", "tests/fixtures/app_with_procfile")
        .buildpacks(vec![BuildpackReference::Crate])
        .run_test(|context| {
            assert_contains!(context.pack_stdout, "[Discovering process types]");
            assert_contains!(
                context.pack_stdout,
                "Procfile declares types -> web, worker"
            );
            assert_contains!(context.pack_stdout, "Setting default process type 'web'");

            context
                .prepare_container()
                .expose_port(8080)
                .start_with_default_process(|container| {
                    thread::sleep(Duration::from_secs(1));
                    let result = call_test_fixture_service(
                        container.address_for_port(8080).unwrap(),
                        "Aeluon",
                    )
                    .unwrap();

                    assert_contains!(result, "payload=Aeluon");
                });
        });
}

fn call_test_fixture_service(addr: std::net::SocketAddr, payload: &str) -> io::Result<String> {
    let req = ureq::get(&format!(
        "http://{}:{}/?payload={}",
        addr.ip(),
        addr.port(),
        payload
    ));
    req.call().unwrap().into_string()
}
