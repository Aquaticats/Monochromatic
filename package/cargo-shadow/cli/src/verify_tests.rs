//! The unit tests of the verification, ported from `update.unit.test.ts` (the `--verify` cases).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::http::RegistryError;
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;
use crate::verify::verify_rust_lockfiles;

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
const CRATE_DIR: &str = "package/v/one";

fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 0,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: true,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
    };
}

struct FakeRegistry;

impl Registry for FakeRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
    fn history(&self, _name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        let trusted = VersionRecord {
            num: "1.0.0".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            publisher_id: String::new(),
            trustpub_repository: "o/r".to_string(),
            repository: String::new(),
        };
        let downgraded = VersionRecord {
            num: "1.0.1".to_string(),
            created_at: "2026-02-01T00:00:00Z".to_string(),
            publisher_id: "9".to_string(),
            trustpub_repository: String::new(),
            repository: String::new(),
        };
        return Ok(vec![trusted, downgraded]);
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
}

fn lock_with(version: &str) -> String {
    return format!("version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n");
}

fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

#[test]
fn a_version_no_lockfile_held_at_the_base_is_checked() {
    let root = repository(&lock_with("1.0.1"));
    let base: String = lock_with("1.0.0");

    let problems = verify_rust_lockfiles(root.path(), 0, &settings(), &FakeRegistry, &|_dir| { return base.clone(); }).expect("verifies");

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("dep@1.0.1 has trust level 1"));
}

#[test]
fn a_version_the_base_already_locked_is_skipped_even_when_its_history_would_fail() {
    let root = repository(&lock_with("1.0.1"));
    let base: String = lock_with("1.0.1");

    let problems = verify_rust_lockfiles(root.path(), 0, &settings(), &FakeRegistry, &|_dir| { return base.clone(); }).expect("verifies");

    assert!(problems.is_empty());
}

#[test]
fn a_lockfile_absent_at_the_base_counts_as_new_in_full() {
    let root = repository(&lock_with("1.0.1"));

    let problems = verify_rust_lockfiles(root.path(), 0, &settings(), &FakeRegistry, &|_dir| { return String::new(); }).expect("verifies");

    assert_eq!(problems.len(), 1);
}
