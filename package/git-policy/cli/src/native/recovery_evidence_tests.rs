//! Controls for attempt evidence and the release of recorded locks.

use super::*;
use crate::recovery_files::tests::lock_identity_of;
use crate::test_support::{fixture, remove};
use crate::transaction_journal::{IndexLockRecord, LockIdentity};
use crate::transaction_journal_encode::encode_index_lock;

/// Owned names from string literals.
fn names(items: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = Vec::new();
    for item in items {
        owned.push(String::from(*item));
    }
    return owned;
}

/// Attempts are positive decimal numbers between the prefix and the suffix,
///  ascending.
#[test]
fn attempts_are_positive_decimals() {
    let found: Vec<i64> = attempt_numbers(
        names(&[
            "landing-10.json",
            "landing-2.json",
            "landing-0.json",
            "landing-.json",
            "landing-x.json",
            "landing-3.jsonl",
            "landing-4",
            "index-lock-5.json",
            "landing-007.json",
            "landing-9007199254740992.json",
            "landing-9007199254740991.json",
            "landing- 6.json",
            "landing-+7.json",
        ])
        .as_slice(),
        LANDING_RECORD_PREFIX,
    );
    assert_eq!(found, vec![2, 7, 10, 9_007_199_254_740_991]);
    assert_eq!(decimal_attempt("1"), Some(1));
    assert_eq!(decimal_attempt("0"), None);
    assert_eq!(decimal_attempt(""), None);
    assert_eq!(decimal_attempt("-1"), None);
    assert_eq!(decimal_attempt("99999999999999999999"), None);
}

/// Landing evidence is a landing or index-lock record.
#[test]
fn landing_evidence_is_found_by_name() {
    let root: PathBuf = fixture("recovery-evidence-landing");
    assert!(!has_landing_record(root.as_path()).expect("empty"));
    assert!(!entered_landing(root.as_path()).expect("empty"));
    std::fs::write(root.join("preparing.json"), b"{}").expect("preparing");
    std::fs::write(root.join("index-lock-1.json"), b"{}").expect("lock record");
    assert!(!has_landing_record(root.as_path()).expect("lock only"));
    assert!(entered_landing(root.as_path()).expect("lock record"));
    std::fs::remove_file(root.join("index-lock-1.json")).expect("remove");
    std::fs::write(root.join("landing-1.json"), b"{}").expect("landing");
    assert!(has_landing_record(root.as_path()).expect("landing"));
    assert!(entered_landing(root.as_path()).expect("landing"));
    let mut listed: Vec<String> = directory_names(root.as_path()).expect("names");
    listed.sort();
    assert_eq!(listed, names(&["landing-1.json", "preparing.json"]));
    assert!(
        directory_names(root.join("missing").as_path())
            .expect_err("missing")
            .0
            .starts_with("listing ")
    );
    assert!(has_landing_record(root.join("missing").as_path()).is_err());
    assert!(entered_landing(root.join("missing").as_path()).is_err());
    remove(root.as_path());
}

/// Git's PID file is removed only while it names the dead owner exactly.
#[test]
fn pid_files_are_removed_only_for_the_owner() {
    let root: PathBuf = fixture("recovery-evidence-pid");
    let index: PathBuf = root.join("index");
    let pid: PathBuf = root.join("index~pid.lock");
    remove_dead_pid_file(index.as_path(), 42).expect("absent");
    std::fs::write(&pid, b"pid 43\n").expect("other owner");
    remove_dead_pid_file(index.as_path(), 42).expect("kept");
    assert!(pid.exists());
    std::fs::write(&pid, b"pid 42").expect("no newline");
    remove_dead_pid_file(index.as_path(), 42).expect("kept");
    assert!(pid.exists());
    std::fs::write(&pid, b"pid 42\n").expect("owner");
    remove_dead_pid_file(index.as_path(), 42).expect("removed");
    assert!(!pid.exists());
    std::fs::create_dir(&pid).expect("directory");
    assert!(remove_dead_pid_file(index.as_path(), 42).is_err());
    remove(root.as_path());
}

/// Recorded locks are released when owned and kept when foreign;
///  the PID file follows.
#[test]
fn recorded_locks_are_released_by_identity() {
    let root: PathBuf = fixture("recovery-evidence-release");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let index: PathBuf = root.join("index");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let owned: LockIdentity = lock_identity_of(lock.as_path());
    let mut stale: LockIdentity = owned.clone();
    stale.file.inode = String::from("1");
    std::fs::write(
        transaction.join("index-lock-1.json"),
        encode_index_lock(&IndexLockRecord {
            attempt: 1,
            lock: stale,
        }),
    )
    .expect("first attempt");
    std::fs::write(root.join("index~pid.lock"), b"pid 77\n").expect("pid file");
    let listed: Vec<String> = directory_names(transaction.as_path()).expect("names");
    release_recorded_locks(
        transaction.as_path(),
        listed.as_slice(),
        index.as_path(),
        77,
    )
    .expect("foreign kept");
    assert!(lock.exists());
    assert!(!root.join("index~pid.lock").exists());
    std::fs::write(
        transaction.join("index-lock-2.json"),
        encode_index_lock(&IndexLockRecord {
            attempt: 2,
            lock: owned,
        }),
    )
    .expect("second attempt");
    let both: Vec<String> = directory_names(transaction.as_path()).expect("names");
    release_recorded_locks(transaction.as_path(), both.as_slice(), index.as_path(), 77)
        .expect("owned released");
    assert!(!lock.exists());
    std::fs::write(transaction.join("index-lock-3.json"), b"{}").expect("malformed");
    let malformed: Vec<String> = directory_names(transaction.as_path()).expect("names");
    assert!(
        release_recorded_locks(
            transaction.as_path(),
            malformed.as_slice(),
            index.as_path(),
            77
        )
        .is_err()
    );
    // A listed record that vanished is a failure, not a skipped attempt.
    assert!(
        release_recorded_locks(
            transaction.as_path(),
            names(&["index-lock-9.json"]).as_slice(),
            index.as_path(),
            77
        )
        .is_err()
    );
    remove(root.as_path());
}
