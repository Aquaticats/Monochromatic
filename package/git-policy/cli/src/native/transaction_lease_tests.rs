//! Controls for lease formatting and validation against real records.

use super::*;
use crate::owner_lock::{OwnerLock, try_acquire_owner_lock};
use crate::test_support::{fixture, remove};
use crate::transaction_owner::{current_transaction_owner, encode_transaction_owner};

/// One environment pair.
fn pair(name: &str, value: &str) -> (OsString, OsString) {
    return (OsString::from(name), OsString::from(value));
}

/// A transaction directory owned by `owner_bytes` whose plan carries `plan_lease`.
fn transaction(root: &Path, owner_bytes: &[u8], plan_lease: &str) -> PathBuf {
    let directory: PathBuf = root.join("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    std::fs::create_dir_all(directory.join("hooks")).expect("hooks directory");
    std::fs::write(directory.join(OWNER_FILENAME), owner_bytes).expect("owner record");
    let mut plan: ObjectWriter = ObjectWriter::new();
    plan.string("lease", plan_lease);
    std::fs::write(directory.join("hooks").join("plan.json"), plan.finish()).expect("plan");
    return directory;
}

/// The current process as a live transaction owner.
fn live_owner() -> Vec<u8> {
    let owner: crate::transaction_owner::TransactionOwner = current_transaction_owner(
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        "2026-10-06T00:00:00.000Z",
    )
    .expect("current owner");
    return encode_transaction_owner(&owner).into_bytes();
}

/// Leases are `JSON.stringify` of their two fields,
///  in order,
///  with escapes.
#[test]
fn leases_have_the_incumbent_spelling() {
    assert_eq!(
        format_preparation_lease("/repo/.git/cli-git-transactions/x", "t\"1"),
        "{\"directory\":\"/repo/.git/cli-git-transactions/x\",\"token\":\"t\\\"1\"}"
    );
    assert_eq!(
        format_landing_lease("/repo/.git/cli-git-transactions/landing.lock", "t"),
        "{\"lockDirectory\":\"/repo/.git/cli-git-transactions/landing.lock\",\"token\":\"t\"}"
    );
}

/// Only an exact plan lease of a live owner is valid.
#[test]
fn preparation_leases_need_the_plan_and_a_live_owner() {
    let root: PathBuf = fixture("lease-preparation");
    let directory_text: String = root
        .join("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10")
        .to_str()
        .expect("UTF-8 fixture")
        .to_string();
    let lease: String = format_preparation_lease(directory_text.as_str(), "token");
    let directory: PathBuf = transaction(root.as_path(), live_owner().as_slice(), lease.as_str());
    let valid: Vec<(OsString, OsString)> = vec![pair(PREPARATION_LEASE_VARIABLE, lease.as_str())];
    assert!(has_valid_preparation_lease(valid.as_slice()));
    // The last entry of a name wins.
    let overridden: Vec<(OsString, OsString)> = vec![
        pair(PREPARATION_LEASE_VARIABLE, "{}"),
        pair(PREPARATION_LEASE_VARIABLE, lease.as_str()),
    ];
    assert!(has_valid_preparation_lease(overridden.as_slice()));
    let shadowed: Vec<(OsString, OsString)> = vec![
        pair(PREPARATION_LEASE_VARIABLE, lease.as_str()),
        pair(PREPARATION_LEASE_VARIABLE, ""),
    ];
    assert!(!has_valid_preparation_lease(shadowed.as_slice()));
    assert!(!has_valid_preparation_lease(&[]));
    assert!(!has_valid_preparation_lease(&[pair(
        "OTHER",
        lease.as_str()
    )]));
    assert!(!has_valid_preparation_lease(&[pair(
        PREPARATION_LEASE_VARIABLE,
        "not json"
    )]));
    assert!(!has_valid_preparation_lease(&[pair(
        PREPARATION_LEASE_VARIABLE,
        "{\"token\":\"t\"}"
    )]));
    // Another token for the same directory is not the plan's lease.
    let other: String = format_preparation_lease(directory_text.as_str(), "other");
    assert!(!has_valid_preparation_lease(&[pair(
        PREPARATION_LEASE_VARIABLE,
        other.as_str()
    )]));
    // A plan that is not an object, or names no lease, validates nothing.
    std::fs::write(directory.join("hooks").join("plan.json"), "[]").expect("array plan");
    assert!(!has_valid_preparation_lease(valid.as_slice()));
    std::fs::write(directory.join("hooks").join("plan.json"), "{}").expect("empty plan");
    assert!(!has_valid_preparation_lease(valid.as_slice()));
    std::fs::write(directory.join("hooks").join("plan.json"), [0xff, 0xfe]).expect("binary plan");
    assert!(!has_valid_preparation_lease(valid.as_slice()));
    std::fs::remove_file(directory.join("hooks").join("plan.json")).expect("remove plan");
    assert!(!has_valid_preparation_lease(valid.as_slice()));
    remove(root.as_path());
}

/// A dead,
///  malformed or missing owner invalidates a lease whose plan still matches.
#[test]
fn preparation_leases_die_with_their_owner() {
    let root: PathBuf = fixture("lease-owner");
    let directory_text: String = root
        .join("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10")
        .to_str()
        .expect("UTF-8 fixture")
        .to_string();
    let lease: String = format_preparation_lease(directory_text.as_str(), "token");
    let environment: Vec<(OsString, OsString)> =
        vec![pair(PREPARATION_LEASE_VARIABLE, lease.as_str())];
    let mut dead: crate::transaction_owner::TransactionOwner = current_transaction_owner(
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        "2026-10-06T00:00:00.000Z",
    )
    .expect("current owner");
    dead.owner_identity = String::from("linux:1");
    let directory: PathBuf = transaction(
        root.as_path(),
        encode_transaction_owner(&dead).as_bytes(),
        lease.as_str(),
    );
    assert!(!has_valid_preparation_lease(environment.as_slice()));
    std::fs::write(directory.join(OWNER_FILENAME), "{}").expect("malformed owner");
    assert!(!has_valid_preparation_lease(environment.as_slice()));
    std::fs::remove_file(directory.join(OWNER_FILENAME)).expect("remove owner");
    assert!(!has_valid_preparation_lease(environment.as_slice()));
    std::fs::write(directory.join(OWNER_FILENAME), live_owner()).expect("live owner");
    assert!(has_valid_preparation_lease(environment.as_slice()));
    remove(root.as_path());
}

/// A lease whose owner lives in another PID namespace stays valid while the owner's record is fresh: this process
///  cannot interpret that PID (issues 1781 and 1784).
#[cfg(target_os = "linux")]
#[test]
fn preparation_leases_of_another_namespace_hold_until_the_record_expires() {
    let root: PathBuf = fixture("lease-foreign-namespace");
    let directory_text: String = root
        .join("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10")
        .to_str()
        .expect("UTF-8 fixture")
        .to_string();
    let lease: String = format_preparation_lease(directory_text.as_str(), "token");
    let environment: Vec<(OsString, OsString)> = vec![pair(PREPARATION_LEASE_VARIABLE, lease.as_str())];
    let mut foreign: crate::transaction_owner::TransactionOwner = current_transaction_owner(
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        "2026-10-06T00:00:00.000Z",
    )
    .expect("current owner");
    // A PID that names no process here: the old rule judged this owner dead.
    foreign.owner_pid = i64::from(i32::MAX);
    foreign.owner_pid_namespace = Some(String::from("pid:[1]"));
    let directory: PathBuf = transaction(root.as_path(), encode_transaction_owner(&foreign).as_bytes(), lease.as_str());
    assert!(has_valid_preparation_lease(environment.as_slice()));
    let expired: std::time::SystemTime =
        std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(directory.join(OWNER_FILENAME))
        .expect("open")
        .set_modified(expired)
        .expect("age");
    assert!(!has_valid_preparation_lease(environment.as_slice()));
    remove(root.as_path());
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
    let environment: Vec<(OsString, OsString)> = vec![(
        OsString::from(PREPARATION_LEASE_VARIABLE),
        OsString::from_vec(vec![0xff, b'{', b'}']),
    )];
    assert!(!has_valid_preparation_lease(environment.as_slice()));
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
