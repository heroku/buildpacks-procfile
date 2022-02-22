//! Integration tests using libcnb-test.
//!
//! All integration tests are skipped by default (using the `ignore` attribute),
//! since performing builds is slow. To run the tests use: `cargo test -- --ignored`

use libcnb_test::{BuildpackReference, IntegrationTest};

#[test]
#[ignore]
fn test() {
    IntegrationTest::new("heroku/buildpacks:20", "tests/fixtures/app_with_procfile")
        .buildpacks(vec![BuildpackReference::Crate])
        .run_test(|context| {
            // On failure, print the stdout
            println!("{}", context.pack_stdout);

            assert!(context.pack_stdout.contains("[Discovering process types]"));
            assert!(context
                .pack_stdout
                .contains("Procfile declares types -> web, worker"));

            assert!(context
                .pack_stdout
                .contains("Setting default process type 'web'"));

            context.start_container(&[8080], |container| {
                let result =
                    call_test_fixture_service(container.address_for_port(8080).unwrap(), "Aeluon")
                        .unwrap();

                assert!(result.contains("payload=Aeluon"))
            });
        });
}

fn call_test_fixture_service(addr: std::net::SocketAddr, payload: &str) -> Result<String, ()> {
    let req = ureq::get(&format!(
        "http://{}:{}/?payload={}",
        addr.ip(),
        addr.port(),
        payload
    ));
    Ok(req.call().unwrap().into_string().unwrap())
}
