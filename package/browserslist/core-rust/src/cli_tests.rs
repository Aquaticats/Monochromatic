//! What:
//!  Tests for the command surface against a fake registry and disposable directories.
//! Why:
//!  `update` writes the committed lock,
//! `export` feeds the conformance harness,
//! and `resolve` is the engine's
//!  call as a command;
//! their option handling must refuse typos instead of ignoring them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('cli.run', () => { /* update, export, resolve, usage errors */ });
//! ```

use crate::cli::run;
use crate::fake_registry::FakeRegistry;
use crate::fixture::{caniuse_tgz, fixture_lock, fixture_registry, scratch};
use crate::lock::DataLock;
use std::fs;

/// Turn literals into the owned argument list `run` takes.
fn args(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| return (*item).to_string()).collect();
}

/// A fake registry ready for `update`:
/// version documents for both packages plus a Mozilla document.
fn updating_registry() -> FakeRegistry {
    let registry = fixture_registry();
    let lock = fixture_lock();
    let document = |integrity: &str| -> Vec<u8> {
        return format!("{{\"version\":\"1.0.30001820\",\"dist\":{{\"integrity\":\"{integrity}\"}}}}").into_bytes();
    };
    registry.set_route("/caniuse-lite/latest", 200, document(&lock.caniuse_lite.integrity));
    registry.set_route(
        "/node-releases/latest",
        200,
        format!("{{\"version\":\"2.0.61\",\"dist\":{{\"integrity\":\"{}\"}}}}", lock.node_releases.integrity).into_bytes(),
    );
    registry.set_route("/mozilla.json", 200, b"{\"FIREFOX_ESR\":\"153.6.0esr\"}".to_vec());
    assert!(!caniuse_tgz().is_empty());
    return registry;
}

/// `update` writes a parseable lock and reports the pinned versions.
#[test]
fn update_writes_the_lock() {
    let registry = updating_registry();
    let directory = scratch();
    let lock_file = directory.path().join("lock.json");
    let cache = directory.path().join("cache");
    let mozilla = format!("{}/mozilla.json", registry.url());
    let output = run(&args(&[
        "update", "--lock", lock_file.to_str().expect("utf8"), "--cache-dir", cache.to_str().expect("utf8"),
        "--registry", &registry.url(), "--mozilla", &mozilla,
    ]))
    .expect("updates");
    assert!(output.contains("caniuse-lite 1.0.30001820, node-releases 2.0.61, Firefox ESR 153.6.0esr"), "{output}");
    let lock = DataLock::from_json(&fs::read_to_string(&lock_file).expect("lock written")).expect("lock parses");
    assert_eq!(lock, fixture_lock());
}

/// `export` writes both verified tarballs named by package and version.
#[test]
fn export_writes_the_tarballs() {
    let registry = fixture_registry();
    let directory = scratch();
    let lock_file = directory.path().join("lock.json");
    fs::write(&lock_file, fixture_lock().to_json()).expect("write lock");
    let out = directory.path().join("out");
    let cache = directory.path().join("cache");
    run(&args(&[
        "export", "--lock", lock_file.to_str().expect("utf8"), "--cache-dir", cache.to_str().expect("utf8"),
        "--registry", &format!("{}/", registry.url()), "--out", out.to_str().expect("utf8"),
    ]))
    .expect("exports");
    assert_eq!(fs::read(out.join("caniuse-lite-1.0.30001820.tgz")).expect("caniuse"), caniuse_tgz());
    assert!(out.join("node-releases-2.0.61.tgz").is_file());
}

/// `resolve` prints the rendered file for a config path.
#[test]
fn resolve_prints_the_rendered_file() {
    let registry = fixture_registry();
    let directory = scratch();
    let lock_file = directory.path().join("lock.json");
    fs::write(&lock_file, fixture_lock().to_json()).expect("write lock");
    let config = directory.path().join(".browserslistrc");
    fs::write(&config, "last 1 Chrome versions\n").expect("write config");
    let cache = directory.path().join("cache");
    let output = run(&args(&[
        "resolve", "--config", config.to_str().expect("utf8"), "--lock", lock_file.to_str().expect("utf8"),
        "--cache-dir", cache.to_str().expect("utf8"), "--registry", &registry.url(),
    ]))
    .expect("resolves");
    assert_eq!(output, "[\n  \"chrome 156\"\n]\n");
}

/// Missing commands,
/// unknown commands,
/// unknown options,
/// missing values and missing files are all usage or I/O errors.
#[test]
fn rejects_bad_invocations() {
    for invocation in [
        args(&[]),
        args(&["frobnicate"]),
        args(&["update", "--bogus", "x"]),
        args(&["update", "stray"]),
        args(&["update", "--lock"]),
        args(&["export"]),
        args(&["resolve"]),
        args(&["resolve", "--out", "x"]),
    ] {
        let message = run(&invocation).expect_err("rejected").to_string();
        assert!(message.starts_with("command line: "), "{invocation:?}: {message}");
    }
    let directory = scratch();
    let missing = directory.path().join("missing.json");
    let message = run(&args(&["export", "--out", "x", "--lock", missing.to_str().expect("utf8")])).expect_err("no lock").to_string();
    assert!(message.contains("missing.json"), "{message}");
}
