//! Controls for lease formatting and validation against real records.

use super::*;
use crate::owner_lock::{OwnerLock, try_acquire_owner_lock};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// One environment pair.
fn pair(name: &str, value: &str) -> (OsString, OsString) {
    return (OsString::from(name), OsString::from(value));
}

/// Leases are `JSON.stringify` of their two fields,
///  in order,
///  with escapes.
#[test]
fn leases_have_the_incumbent_spelling() {
    assert_eq!(
        format_landing_lease("/repo/.git/cli-git-transactions/landing.lock", "t"),
        "{\"lockDirectory\":\"/repo/.git/cli-git-transactions/landing.lock\",\"token\":\"t\"}"
    );
}

/// A landing lease is valid only for its own lock while that lock carries its token.
#[test]
fn landing_leases_name_their_lock_and_token() {
    let root: PathBuf = fixture("lease-landing");
    let lock_directory: PathBuf = root.join("landing.lock");
    let lock: OwnerLock = try_acquire_owner_lock(lock_directory.as_path(), None)
        .expect("acquire")
        .expect("free lock");
    let text: &str = lock_directory.to_str().expect("UTF-8 fixture");
    let lease: String = format_landing_lease(text, lock.token());
    let environment: Vec<(OsString, OsString)> = vec![pair(LANDING_LEASE_VARIABLE, lease.as_str())];
    assert!(has_valid_landing_lease(
        environment.as_slice(),
        lock_directory.as_path()
    ));
    assert!(!has_valid_landing_lease(
        environment.as_slice(),
        root.join("other.lock").as_path()
    ));
    // The same lock spelled differently is another lock, as the incumbent compares text.
    let spelled: PathBuf = root.join(".").join("landing.lock");
    assert!(!has_valid_landing_lease(
        environment.as_slice(),
        spelled.as_path()
    ));
    let stale: String = format_landing_lease(text, "another-token");
    assert!(!has_valid_landing_lease(
        &[pair(LANDING_LEASE_VARIABLE, stale.as_str())],
        lock_directory.as_path()
    ));
    assert!(!has_valid_landing_lease(&[], lock_directory.as_path()));
    assert!(!has_valid_landing_lease(
        &[pair(LANDING_LEASE_VARIABLE, "[]")],
        lock_directory.as_path()
    ));
    let without_token: String =
        format!("{{\"lockDirectory\":{}}}", crate::json_record::quote(text));
    assert!(!has_valid_landing_lease(
        &[pair(LANDING_LEASE_VARIABLE, without_token.as_str())],
        lock_directory.as_path()
    ));
    lock.release().expect("release");
    // A released lock has no owner record, so the lease is gone with it.
    assert!(!has_valid_landing_lease(
        environment.as_slice(),
        lock_directory.as_path()
    ));
    std::fs::create_dir(&lock_directory).expect("ownerless lock");
    assert!(!has_valid_landing_lease(
        environment.as_slice(),
        lock_directory.as_path()
    ));
    std::fs::write(lock_directory.join("owner.json"), "{").expect("malformed owner");
    assert!(!has_valid_landing_lease(
        environment.as_slice(),
        lock_directory.as_path()
    ));
    remove(root.as_path());
}

/// A lease that is not Unicode text is no lease.
#[cfg(unix)]
#[test]
fn non_unicode_leases_are_ignored() {
    use std::os::unix::ffi::OsStringExt;
    let landing: Vec<(OsString, OsString)> = vec![(
        OsString::from(LANDING_LEASE_VARIABLE),
        OsString::from_vec(vec![0xff]),
    )];
    assert!(!has_valid_landing_lease(
        landing.as_slice(),
        Path::new("/x")
    ));
}

/// The token of a landing lease is its own field, and a missing, empty or malformed lease has none.
#[test]
fn the_landing_lease_token_is_its_own_field() {
    let lease: String = format_landing_lease("/r/landing.lock", "token-1");
    let named = |value: &str| { return vec![(OsString::from(LANDING_LEASE_VARIABLE), OsString::from(value))]; };
    assert_eq!(landing_lease_token(named(lease.as_str()).as_slice()), Some(String::from("token-1")));
    assert_eq!(landing_lease_token(&[]), None);
    assert_eq!(landing_lease_token(named("").as_slice()), None);
    assert_eq!(landing_lease_token(named("not json").as_slice()), None);
    assert_eq!(landing_lease_token(named("{\"lockDirectory\":\"/r\"}").as_slice()), None);
}
