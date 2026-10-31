//! What:
//!  Tests for the pinned-tarball cache:
//! cold fetch,
//! warm hit,
//! corrupted entries,
//! and tampered downloads.
//! Why:
//!  The cache must give a fresh machine and a warm one the same bytes,
//! and must never trust unverified ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('loadData', () => { /* cold, warm, corrupt cache, tampered registry */ });
//! ```

use crate::error::BrowserslistError;
use crate::fake_registry::FakeRegistry;
use crate::fixture::{fixture_lock, fixture_registry, scratch};
use crate::integrity::cache_key;
use crate::source::{DataSource, load_data, pinned_tarball, tarball_url, write_atomically};
use std::fs;

/// Build a source over a scratch cache and a fake registry.
fn source_for(registry: &FakeRegistry, cache: &std::path::Path) -> DataSource {
    return DataSource { cache_dir: cache.join("cache"), registry_url: registry.url() };
}

/// A cold cache fetches exactly the two pinned tarballs;
/// a warm cache fetches nothing.
#[test]
fn cold_then_warm() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = source_for(&registry, directory.path());
    let lock = fixture_lock();
    let cold = load_data(&lock, &source).expect("cold load");
    assert_eq!(
        registry.requests(),
        vec![
            "/caniuse-lite/-/caniuse-lite-1.0.30001820.tgz".to_string(),
            "/node-releases/-/node-releases-2.0.61.tgz".to_string(),
        ]
    );
    let warm = load_data(&lock, &source).expect("warm load");
    assert_eq!(registry.requests().len(), 2);
    assert_eq!(cold, warm);
    assert_eq!(cold.firefox_esr_major, 153);
    let key = cache_key(&lock.caniuse_lite.integrity).expect("key");
    assert!(source.cache_dir.join(format!("{key}.tgz")).is_file());
}

/// The URL is derived from registry,
/// name and version only.
#[test]
fn tarball_urls() {
    let lock = fixture_lock();
    assert_eq!(
        tarball_url("https://registry.npmjs.org", &lock.node_releases),
        "https://registry.npmjs.org/node-releases/-/node-releases-2.0.61.tgz"
    );
}

/// A cached file that no longer hashes correctly is replaced by a fresh verified download.
#[test]
fn corrupted_cache_entry_is_refetched() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = source_for(&registry, directory.path());
    let lock = fixture_lock();
    load_data(&lock, &source).expect("cold load");
    let path = source.cache_dir.join(format!("{}.tgz", cache_key(&lock.caniuse_lite.integrity).expect("key")));
    fs::write(&path, b"tampered").expect("corrupt the entry");
    load_data(&lock, &source).expect("recovers");
    assert_eq!(registry.requests().len(), 3);
    assert_ne!(fs::read(&path).expect("entry"), b"tampered".to_vec());
}

/// A registry that serves different bytes than the lock pins is rejected,
/// and nothing is cached.
#[test]
fn tampered_download_is_rejected_and_not_cached() {
    let registry = fixture_registry();
    registry.set_route("/caniuse-lite/-/caniuse-lite-1.0.30001820.tgz", 200, b"evil bytes".to_vec());
    let directory = scratch();
    let source = source_for(&registry, directory.path());
    let lock = fixture_lock();
    let error = pinned_tarball(&lock.caniuse_lite, &source).expect_err("rejected");
    assert!(matches!(error, BrowserslistError::Integrity { .. }), "{error}");
    let cached = fs::read_dir(&source.cache_dir).map(|entries| return entries.count()).unwrap_or(0);
    assert_eq!(cached, 0);
}

/// A registry failure surfaces as a network error naming the pinned URL.
#[test]
fn missing_tarball_is_a_network_error() {
    let registry = FakeRegistry::start(Vec::new());
    let directory = scratch();
    let source = source_for(&registry, directory.path());
    let message = load_data(&fixture_lock(), &source).expect_err("404").to_string();
    assert!(message.contains("caniuse-lite-1.0.30001820.tgz"), "{message}");
}

/// Verified bytes that are not a usable package fail in the decoder,
/// naming what is missing.
#[test]
fn unusable_package_contents_are_reported() {
    let registry = fixture_registry();
    let directory = scratch();
    let source = source_for(&registry, directory.path());
    let mut lock = fixture_lock();
    let junk = crate::fixture::build_tgz(&[("package/readme", b"hi")]);
    lock.caniuse_lite.integrity = crate::integrity::integrity_of(&junk);
    registry.set_route("/caniuse-lite/-/caniuse-lite-1.0.30001820.tgz", 200, junk);
    let message = load_data(&lock, &source).expect_err("missing files").to_string();
    assert!(message.contains("package/data/agents.js is missing"), "{message}");
}

/// Atomic writes create parent directories and replace existing files.
#[test]
fn atomic_write_replaces_files() {
    let directory = scratch();
    let path = directory.path().join("a").join("b.bin");
    write_atomically(&path, b"one").expect("first write");
    write_atomically(&path, b"two").expect("second write");
    assert_eq!(fs::read(&path).expect("file"), b"two".to_vec());
    let leftovers = fs::read_dir(path.parent().expect("parent")).expect("dir").count();
    assert_eq!(leftovers, 1);
}

/// The production source points at an `npmjs.org` registry and a per-user cache directory.
#[test]
fn user_default_source() {
    let source = DataSource::user_default().expect("home directory exists in tests");
    assert_eq!(source.registry_url, "https://registry.npmjs.org");
    assert!(source.cache_dir.to_string_lossy().contains("browserslist"));
}
