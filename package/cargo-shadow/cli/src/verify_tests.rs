//! The unit tests of the check of every locked entry: what pnpm's install does to a lockfile unless `trustLockfile` is set.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::crates::LockSnapshot;
use crate::http::RegistryError;
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;
use crate::verify::{verify_added_since_base, verify_locked_entries};

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

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
        trust_lockfile: false,
    };
}

/// A registry that counts its history lookups.
struct CountingRegistry {
    lookups: Cell<u32>,
}

impl Registry for CountingRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
    fn history(&self, _name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        self.lookups.set(self.lookups.get() + 1);
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

fn snapshot(version: &str) -> LockSnapshot {
    return LockSnapshot {
        crate_dir: "package/v/one".to_string(),
        source: format!("version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n"),
    };
}

#[test]
fn every_locked_version_is_checked_not_only_the_ones_added_since_a_base() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    let problems = verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("dep@1.0.1 has trust level 1"));
}

#[test]
fn a_passing_lockfile_is_remembered_and_costs_no_lookup_the_next_time() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    assert!(verify_locked_entries(root.path(), &[snapshot("1.0.0")], &settings(), &registry, 0).expect("verifies").is_empty());
    let after_first: u32 = registry.lookups.get();
    assert!(verify_locked_entries(root.path(), &[snapshot("1.0.0")], &settings(), &registry, 0).expect("verifies").is_empty());

    assert!(after_first > 0);
    assert_eq!(registry.lookups.get(), after_first, "the second call asked the registry again");
}

#[test]
fn a_failing_lockfile_is_never_remembered() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");
    let after_first: u32 = registry.lookups.get();
    let again = verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");

    assert_eq!(again.len(), 1);
    assert!(registry.lookups.get() > after_first);
}

#[test]
fn a_changed_lockfile_or_a_changed_policy_is_checked_again() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };
    verify_locked_entries(root.path(), &[snapshot("1.0.0")], &settings(), &registry, 0).expect("verifies");
    let after_first: u32 = registry.lookups.get();

    verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");
    let after_new_lock: u32 = registry.lookups.get();
    let mut stricter: Settings = settings();
    stricter.trust_policy_exclude = vec!["other".to_string()];
    verify_locked_entries(root.path(), &[snapshot("1.0.0")], &stricter, &registry, 0).expect("verifies");

    assert!(after_new_lock > after_first);
    assert!(registry.lookups.get() > after_new_lock);
}

#[test]
fn since_the_base_only_versions_no_lockfile_held_there_are_checked() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };
    let current: LockSnapshot = snapshot("1.0.1");
    let base_with_it: String = snapshot("1.0.1").source;
    let base_without_it: String = snapshot("1.0.0").source;
    let _ = root;

    let held = verify_added_since_base(std::slice::from_ref(&current), &|_dir| { return base_with_it.clone(); }, &settings(), &registry, 0).expect("verifies");
    let lookups_when_held: u32 = registry.lookups.get();
    let new = verify_added_since_base(std::slice::from_ref(&current), &|_dir| { return base_without_it.clone(); }, &settings(), &registry, 0).expect("verifies");

    assert!(held.is_empty());
    assert_eq!(lookups_when_held, 0, "a version the base held costs no registry lookup");
    assert_eq!(new.len(), 1);
    assert!(new[0].contains("dep@1.0.1 has trust level 1"));
}

#[test]
fn since_the_base_a_lockfile_absent_there_counts_as_new_in_full() {
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    let problems = verify_added_since_base(&[snapshot("1.0.1")], &|_dir| { return String::new(); }, &settings(), &registry, 0).expect("verifies");

    assert_eq!(problems.len(), 1);
}
