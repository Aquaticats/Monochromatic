//! What:
//!  Tests for the update step against a fake registry and a fake Mozilla document.
//! Why:
//!  The newest-version pick, integrity requirement, ESR shape check and decode-before-pin behavior are the
//!  contract of the update task, and none of it may run against the real network in tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('buildLock', () => { /* latest lookup, ESR shape, tampered tarball */ });
//! ```

use crate::error::BrowserslistError;
use crate::fake_registry::FakeRegistry;
use crate::fixture::{caniuse_tgz, fixture_lock, fixture_registry, node_releases_tgz, scratch};
use crate::integrity::integrity_of;
use crate::source::DataSource;
use crate::update::{build_lock, latest_firefox_esr, latest_package};

/// A registry version document for a package.
fn version_document(version: &str, integrity: &str) -> Vec<u8> {
    return format!("{{\"name\":\"x\",\"version\":\"{version}\",\"dist\":{{\"integrity\":\"{integrity}\",\"shasum\":\"0\"}}}}").into_bytes();
}

/// A Mozilla-style document holding an ESR value.
fn mozilla_document(esr: &str) -> Vec<u8> {
    return format!("{{\"FIREFOX_ESR\":\"{esr}\",\"FIREFOX_ESR115\":\"115.44.0esr\",\"LATEST_FIREFOX_VERSION\":\"159.0\"}}").into_bytes();
}

/// A registry that answers `latest` for both packages and serves their tarballs.
fn updating_registry(caniuse_version: &str) -> FakeRegistry {
    let registry = fixture_registry();
    let lock = fixture_lock();
    registry.set_route("/caniuse-lite/latest", 200, version_document(caniuse_version, &lock.caniuse_lite.integrity));
    registry.set_route("/node-releases/latest", 200, version_document("2.0.61", &lock.node_releases.integrity));
    registry.set_route(&format!("/caniuse-lite/-/caniuse-lite-{caniuse_version}.tgz"), 200, caniuse_tgz());
    registry.set_route("/mozilla.json", 200, mozilla_document("153.6.0esr"));
    return registry;
}

/// The update pins the newest versions, their integrities, and the Mozilla ESR, and caches the verified tarballs.
#[test]
fn builds_a_lock_from_the_newest_data() {
    let registry = updating_registry("1.0.30009999");
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    let lock = build_lock(&source, &format!("{}/mozilla.json", registry.url())).expect("builds");
    assert_eq!(lock.caniuse_lite.version, "1.0.30009999");
    assert_eq!(lock.node_releases.version, "2.0.61");
    assert_eq!(lock.caniuse_lite.integrity, integrity_of(&caniuse_tgz()));
    assert_eq!(lock.node_releases.integrity, integrity_of(&node_releases_tgz()));
    assert_eq!(lock.firefox_esr, "153.6.0esr");
    assert!(registry.requests().contains(&"/caniuse-lite/-/caniuse-lite-1.0.30009999.tgz".to_string()));
}

/// A tarball that does not hash to the registry's integrity stops the update.
#[test]
fn tampered_tarball_stops_the_update() {
    let registry = updating_registry("1.0.30009999");
    registry.set_route("/caniuse-lite/-/caniuse-lite-1.0.30009999.tgz", 200, b"not the right bytes".to_vec());
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    let error = build_lock(&source, &format!("{}/mozilla.json", registry.url())).expect_err("rejected");
    assert!(matches!(error, BrowserslistError::Integrity { .. }), "{error}");
}

/// A bad ESR shape stops the update with the format diagnostic.
#[test]
fn bad_esr_shape_stops_the_update() {
    let registry = updating_registry("1.0.30009999");
    registry.set_route("/mozilla.json", 200, mozilla_document("153.6esr"));
    let directory = scratch();
    let source = DataSource { cache_dir: directory.path().join("cache"), registry_url: registry.url() };
    let message = build_lock(&source, &format!("{}/mozilla.json", registry.url())).expect_err("rejected").to_string();
    assert!(message.contains("<major>.<minor>.<patch>esr"), "{message}");
}

/// Malformed registry and Mozilla answers are each refused with the URL named.
#[test]
fn malformed_answers_are_refused() {
    let registry = FakeRegistry::start(vec![
        ("/p/latest", 200, b"[]".to_vec()),
        ("/q/latest", 200, b"{\"version\":\"1.0.0\"}".to_vec()),
        ("/r/latest", 200, b"{\"version\":\"1.0.0\",\"dist\":{\"integrity\":\"sha1-AAAA\"}}".to_vec()),
        ("/s/latest", 200, b"{\"dist\":{}}".to_vec()),
        ("/t/latest", 200, b"nope".to_vec()),
        ("/m1", 200, b"{}".to_vec()),
        ("/m2", 200, b"{\"FIREFOX_ESR\":153}".to_vec()),
    ]);
    for name in ["p", "q", "r", "s", "t"] {
        let message = latest_package(&registry.url(), name).expect_err("refused").to_string();
        assert!(message.contains(&format!("/{name}/latest")) || message.contains("sha512"), "{name}: {message}");
    }
    for path in ["/m1", "/m2"] {
        let message = latest_firefox_esr(&format!("{}{path}", registry.url())).expect_err("refused").to_string();
        assert!(message.contains("FIREFOX_ESR"), "{path}: {message}");
    }
    assert!(latest_package(&registry.url(), "p").is_err());
}
