//! Controls for the PID namespace rule.

use super::*;
use std::fs::File;

/// A scratch record file whose modification time is `age` in the past.
fn record_aged(directory: &Path, age: Duration) -> std::path::PathBuf {
    let path: std::path::PathBuf = directory.join("owner.json");
    let file: File = File::create(path.as_path()).expect("create the record");
    file.set_modified(SystemTime::now() - age).expect("age the record");
    return path;
}

/// This process names its own namespace on Linux, in the kernel's spelling.
#[test]
#[cfg(target_os = "linux")]
fn the_current_namespace_is_named_the_way_the_kernel_names_it() {
    let name: String = current_pid_namespace().expect("a Linux process has a PID namespace");
    assert!(name.starts_with("pid:["), "{name}");
    assert!(name.ends_with(']'), "{name}");
}

/// Only two known and different names make a namespace foreign.
#[test]
fn only_two_known_and_different_names_are_another_namespace() {
    assert!(is_other_namespace(Some("pid:[1]"), Some("pid:[2]")));
    assert!(!is_other_namespace(Some("pid:[1]"), Some("pid:[1]")));
    assert!(!is_other_namespace(None, Some("pid:[2]")), "an old record names no namespace");
    assert!(!is_other_namespace(Some("pid:[1]"), None), "a reader without a namespace cannot tell");
    assert!(!is_other_namespace(None, None));
}

/// A fresh record has not outlived the patience; one past it has.
#[test]
fn a_foreign_record_expires_only_after_the_patience() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-1");
    let fresh = record_aged(directory.as_path(), Duration::from_secs(60));
    assert!(!foreign_record_has_expired(fresh.as_path()));
    let old = record_aged(directory.as_path(), FOREIGN_OWNER_PATIENCE + Duration::from_secs(60));
    assert!(foreign_record_has_expired(old.as_path()));
    let nearly = record_aged(directory.as_path(), FOREIGN_OWNER_PATIENCE - Duration::from_secs(60));
    assert!(!foreign_record_has_expired(nearly.as_path()), "just inside the patience stays alive");
}

/// A record that cannot be measured keeps its owner alive.
#[test]
fn a_record_that_cannot_be_measured_does_not_expire() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-2");
    assert!(!foreign_record_has_expired(directory.as_path().join("absent.json").as_path()));
    let future = record_aged(directory.as_path(), Duration::ZERO);
    File::options().write(true).open(future.as_path()).expect("open").set_modified(
        SystemTime::now() + Duration::from_secs(7200),
    ).expect("move the time ahead");
    assert!(!foreign_record_has_expired(future.as_path()), "a record from the future is not old");
}

/// The shared rule: unknown or same namespace leaves the decision to the PID; another one answers by age.
#[test]
#[cfg(target_os = "linux")]
fn a_foreign_owner_is_alive_until_its_record_expires() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-3");
    let fresh = record_aged(directory.as_path(), Duration::from_secs(60));
    let own: String = current_pid_namespace().expect("namespace");
    assert_eq!(foreign_owner_standing(Some(own.as_str()), fresh.as_path()), None);
    assert_eq!(foreign_owner_standing(None, fresh.as_path()), None);
    assert_eq!(foreign_owner_standing(Some("pid:[1]"), fresh.as_path()), Some(true));
    let old = record_aged(directory.as_path(), FOREIGN_OWNER_PATIENCE + Duration::from_secs(60));
    assert_eq!(foreign_owner_standing(Some("pid:[1]"), old.as_path()), Some(false));
}

/// Exactly the patience is still alive; one nanosecond more is expired.
#[test]
fn the_patience_boundary_is_exclusive() {
    assert!(!age_outlives_patience(FOREIGN_OWNER_PATIENCE));
    assert!(age_outlives_patience(FOREIGN_OWNER_PATIENCE + Duration::from_nanos(1)));
    assert!(!age_outlives_patience(FOREIGN_OWNER_PATIENCE - Duration::from_nanos(1)));
}
