//! Unit tests of the network seam:
//! redirect resolution,
//! the https-only rule and diagnostics of the real client (against
//! loopback only),
//! and the fake used by every other test.

use super::fakes::FakeHttp;
use super::hash::sha256_hex;
use super::http::{HttpGet, HyperClient, resolve_location};
use crate::test_scratch::scratch;

#[test]
fn redirect_locations_resolve_against_the_request_url() {
    let base = "https://github.com/owner/repo/releases/download/v1/asset.tgz";
    assert_eq!(resolve_location(base, "https://objects.example/x").expect("absolute"), "https://objects.example/x");
    assert_eq!(resolve_location(base, "/other/path?q=1").expect("path"), "https://github.com/other/path?q=1");
    assert_eq!(resolve_location(base, "//cdn.example/y").expect("scheme-relative"), "https://cdn.example/y");
    assert!(resolve_location(base, "relative/path").is_err());
    let downgrade = resolve_location(base, "http://plain.example/x");
    assert!(downgrade.is_err(), "a redirect to plain http is refused: {downgrade:?}");
}

#[test]
fn the_real_client_refuses_plain_http_and_names_the_url() {
    let client = HyperClient::new().expect("the client builds on a host with root certificates");
    let error = client.get("http://registry.example/pkg", &[], 1024).expect_err("plain http is refused");
    assert_eq!(error.url, "http://registry.example/pkg");
    assert!(error.message.contains("only https"), "{error}");
    let download_error = client
        .download("http://registry.example/file.tgz", &scratch().path().join("file"), 1024)
        .expect_err("plain http is refused");
    assert!(download_error.to_string().starts_with("http://registry.example/file.tgz: "), "{download_error}");
}

#[test]
fn the_real_client_reports_a_refused_connection_with_the_url() {
    let client = HyperClient::new().expect("the client builds");
    let error = client.get("https://127.0.0.1:1/never-listening", &[], 1024).expect_err("nothing listens on port 1");
    assert!(error.url.contains("127.0.0.1:1"), "{error}");
    assert!(error.message.contains("failed"), "{error}");
}

#[test]
fn the_fake_serves_a_table_and_answers_404_for_anything_else() {
    let http = FakeHttp::new();
    http.serve("https://registry.example/a", b"{\"ok\": true}");
    let served = http.get("https://registry.example/a", &[], 1024).expect("served");
    assert_eq!(served.status, 200);
    assert_eq!(served.text(), "{\"ok\": true}");
    let missing = http.get("https://registry.example/b", &[], 1024).expect("a 404 is a response, not an error");
    assert_eq!(missing.status, 404);
    assert_eq!(http.requests(), vec!["https://registry.example/a".to_string(), "https://registry.example/b".to_string()]);
}

#[test]
fn the_fake_fails_a_url_on_request_and_downloads_with_the_files_hash() {
    let http = FakeHttp::new();
    http.fail("https://down.example/x", "connection reset");
    let error = http.get("https://down.example/x", &[], 1024).expect_err("failure configured");
    assert_eq!(error.to_string(), "https://down.example/x: connection reset");

    http.serve("https://files.example/a.tgz", b"payload");
    let directory = scratch();
    let target = directory.path().join("sub").join("a.tgz");
    let hash = http.download("https://files.example/a.tgz", &target, 1 << 20).expect("downloaded");
    assert_eq!(hash, sha256_hex(b"payload"));
    assert_eq!(std::fs::read(&target).expect("read"), b"payload");
    assert!(http.download("https://files.example/missing.tgz", &target, 1 << 20).is_err());
}
