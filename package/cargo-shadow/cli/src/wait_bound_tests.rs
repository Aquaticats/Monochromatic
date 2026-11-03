//! The tests of the owner's rule of 2026-11-03, "nothing should ever wait forever or indefinitely", for the waits that are not
//! journal locks: a cargo call, the file-enforcer sync and a registry pause.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::cargo_run::{CargoRunner, RealCargo};
use crate::http::{RegistryError, TextFetcher};
use crate::registry::{CratesIo, Registry};
use crate::test_scratch::scratch;

/// Writes an executable shell script.
fn script(directory: &Path, name: &str, body: &str) -> PathBuf {
    let path: PathBuf = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    return path;
}

/// A cargo that outlives its bound is stopped, and the error names the call and the command to run again.
#[test]
fn a_cargo_that_outlives_its_bound_is_stopped_and_named() {
    let directory = scratch();
    let cargo = RealCargo { program: script(directory.path(), "cargo", "sleep 5"), toolchain: String::new(), bound: Duration::from_millis(300) };
    let started = Instant::now();

    let result = cargo.run(directory.path(), &["update".to_string(), "--workspace".to_string()]);

    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    let message: String = result.expect_err("a stopped cargo has no outcome");
    assert!(message.contains("cargo update --workspace") && message.contains("did not finish"), "{message}");
}

/// A fetcher that always answers HTTP 429 with a long `Retry-After`.
struct Throttled;

impl TextFetcher for Throttled {
    /// What:
    /// `fn get(&self, url) -> Result<String, RegistryError>` always answers "too many requests, retry in a day".
    /// Why:
    /// The registry must not sleep for what the server asks when that is longer than its own bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get(url: string): string  // always throws a 429 with Retry-After: 86400
    /// ```
    fn get(&self, url: &str) -> Result<String, RegistryError> {
        return Err(RegistryError { message: format!("{url} answered HTTP 429"), status: Some(429), retry_after_seconds: 86_400 });
    }
}

/// A server that asks for a pause longer than the registry's own bound ends the call with an error that says so; it does not sleep.
#[test]
fn a_retry_after_longer_than_the_bound_ends_the_call_instead_of_sleeping() {
    let root = scratch();
    let registry = CratesIo::new(root.path(), Box::new(Throttled), Duration::ZERO, Duration::from_millis(1));
    let started = Instant::now();

    let result = registry.owners("serde");

    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
    let message: String = result.expect_err("no answer").message;
    assert!(message.contains("429") && message.contains("run the command again"), "{message}");
}
