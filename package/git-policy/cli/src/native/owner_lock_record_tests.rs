//! Controls for the owner record:
//!  the incumbent's accepted and rejected shapes,
//!  its exact
//! encoding,
//!  reading a published lock,
//!  and owner liveness.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// The record the incumbent's `writeCandidate` writes for the hook lock,
///  byte for byte.
const INCUMBENT_HOOK_LOCK: &str = "{\"schemaVersion\":1,\"token\":\"0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10\",\
\"ownerPid\":4242,\"ownerBirthIdentity\":\"linux:8423337\"}\n";

/// The record the incumbent writes for the landing reservation,
///  with its transaction.
const INCUMBENT_RESERVATION: &str = "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":1,\
\"ownerBirthIdentity\":\"darwin:Mon Oct  6 16:34:01 2026\",\
\"transactionId\":\"11111111-2222-4333-8444-555555555555\"}\n";

/// The hook-lock record parses into its fields.
fn hook_lock_record() -> OwnerLockRecord {
    return OwnerLockRecord {
        token: String::from("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
        owner_pid: 4242,
        owner_birth_identity: String::from("linux:8423337"),
        transaction_id: None,
        owner_pid_namespace: None,
    };
}

/// Records written by the incumbent parse,
///  and re-encode to the same bytes.
#[test]
fn incumbent_records_round_trip_byte_for_byte() {
    assert_eq!(
        parse_owner_lock_record(INCUMBENT_HOOK_LOCK),
        Some(hook_lock_record())
    );
    assert_eq!(
        encode_owner_lock_record(&hook_lock_record()),
        INCUMBENT_HOOK_LOCK
    );
    let reservation: OwnerLockRecord =
        parse_owner_lock_record(INCUMBENT_RESERVATION).expect("reservation record");
    assert_eq!(
        reservation.transaction_id.as_deref(),
        Some("11111111-2222-4333-8444-555555555555")
    );
    assert_eq!(
        encode_owner_lock_record(&reservation),
        INCUMBENT_RESERVATION
    );
}

/// Every shape `parseOwnerLockRecord` refuses is refused.
#[test]
fn malformed_records_are_refused() {
    let refused: [&str; 17] = [
        "",
        "[]",
        "null",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":4242}",
        "{\"schemaVersion\":2,\"token\":\"t\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":\"1\",\"token\":\"t\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
        "{\"token\":\"t\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":\"\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":5,\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":0,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":-1,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":1.5,\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":\"1\",\"ownerBirthIdentity\":\"i\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":1,\"ownerBirthIdentity\":\"\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\",\"transactionId\":\"\"}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\",\"transactionId\":null}",
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":9007199254740992,\"ownerBirthIdentity\":\"i\"}",
    ];
    for text in refused {
        assert_eq!(parse_owner_lock_record(text), None, "{text}");
    }
}

/// Shapes `parseOwnerLockRecord` accepts beyond the written form are accepted.
#[test]
fn equivalent_spellings_are_accepted() {
    let spelled: OwnerLockRecord = parse_owner_lock_record(
        "{ \"ownerBirthIdentity\" : \"i\", \"ownerPid\" : 1.0, \"token\" : \"t\", \"schemaVersion\" : 1.0, \"extra\": [] }",
    )
    .expect("accepted spelling");
    assert_eq!(spelled.owner_pid, 1);
    assert_eq!(spelled.transaction_id, None);
    assert_eq!(
        parse_owner_lock_record(
            "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":9007199254740991,\"ownerBirthIdentity\":\"i\"}"
        )
        .expect("largest safe PID")
        .owner_pid,
        9_007_199_254_740_991
    );
}

/// Reading a lock:
///  a vanished lock is busy,
///  a valid record is its owner,
///  anything else fails.
#[test]
fn reading_a_published_lock() {
    let root: PathBuf = fixture("owner-record-read");
    let lock: PathBuf = root.join("hook.lock");
    assert_eq!(
        read_owner_lock_record(lock.as_path()).expect("absent lock"),
        PublishedOwner::Busy
    );
    std::fs::create_dir(&lock).expect("lock directory");
    assert_eq!(
        read_owner_lock_record(lock.as_path()).expect("no record yet"),
        PublishedOwner::Busy
    );
    std::fs::write(lock.join(OWNER_LOCK_RECORD_FILENAME), INCUMBENT_HOOK_LOCK).expect("record");
    assert_eq!(
        read_owner_lock_record(lock.as_path()).expect("owner"),
        PublishedOwner::Owner(hook_lock_record())
    );
    std::fs::write(
        lock.join(OWNER_LOCK_RECORD_FILENAME),
        "{\"schemaVersion\":1}",
    )
    .expect("bad");
    assert!(matches!(
        read_owner_lock_record(lock.as_path()),
        Err(OwnerRecordError::Malformed(_))
    ));
    // A lock path that is a file names no record: the lock is busy, as with `ENOTDIR`.
    let file_lock: PathBuf = root.join("file.lock");
    std::fs::write(&file_lock, b"x").expect("file");
    assert_eq!(
        read_owner_lock_record(file_lock.as_path()).expect("not a directory"),
        PublishedOwner::Busy
    );
    remove(root.as_path());
}

/// A record that is a link,
///  a directory or an oversized file is malformed,
///  never followed.
#[cfg(unix)]
#[test]
fn unsafe_records_are_malformed() {
    let root: PathBuf = fixture("owner-record-unsafe");
    let lock: PathBuf = root.join("landing.lock");
    std::fs::create_dir(&lock).expect("lock");
    let target: PathBuf = root.join("elsewhere.json");
    std::fs::write(&target, INCUMBENT_HOOK_LOCK).expect("target");
    std::os::unix::fs::symlink(&target, lock.join(OWNER_LOCK_RECORD_FILENAME)).expect("link");
    assert!(matches!(
        read_owner_lock_record(lock.as_path()),
        Err(OwnerRecordError::Malformed(_))
    ));
    std::fs::remove_file(lock.join(OWNER_LOCK_RECORD_FILENAME)).expect("unlink");
    std::fs::create_dir(lock.join(OWNER_LOCK_RECORD_FILENAME)).expect("record directory");
    assert!(matches!(
        read_owner_lock_record(lock.as_path()),
        Err(OwnerRecordError::Malformed(_))
    ));
    std::fs::remove_dir(lock.join(OWNER_LOCK_RECORD_FILENAME)).expect("rmdir");
    let oversized: String = format!("{}{}", INCUMBENT_HOOK_LOCK, " ".repeat(64 * 1024));
    std::fs::write(lock.join(OWNER_LOCK_RECORD_FILENAME), oversized).expect("oversized");
    assert!(matches!(
        read_owner_lock_record(lock.as_path()),
        Err(OwnerRecordError::Malformed(_))
    ));
    remove(root.as_path());
}

/// Invalid UTF-8 is replaced before parsing,
///  as Node's `readFile(path, 'utf8')` does.
#[test]
fn invalid_bytes_in_a_record_are_replaced() {
    let root: PathBuf = fixture("owner-record-bytes");
    let lock: PathBuf = root.join("capture.lock");
    std::fs::create_dir(&lock).expect("lock");
    std::fs::write(
        lock.join(OWNER_LOCK_RECORD_FILENAME),
        b"{\"schemaVersion\":1,\"token\":\"t\xff\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
    )
    .expect("record");
    let PublishedOwner::Owner(record) = read_owner_lock_record(lock.as_path()).expect("owner")
    else {
        panic!("the record has an owner");
    };
    assert_eq!(record.token, "t\u{fffd}");
    remove(root.as_path());
}

/// An unreadable record is reported as unreadable.
#[cfg(unix)]
#[test]
fn an_unreadable_record_is_reported() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("owner-record-unreadable");
    let lock: PathBuf = root.join("push.lock");
    std::fs::create_dir(&lock).expect("lock");
    let record: PathBuf = lock.join(OWNER_LOCK_RECORD_FILENAME);
    std::fs::write(&record, INCUMBENT_HOOK_LOCK).expect("record");
    std::fs::set_permissions(&record, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    if std::fs::read(&record).is_err() {
        assert!(matches!(
            read_owner_lock_record(lock.as_path()),
            Err(OwnerRecordError::Unreadable(_, _))
        ));
    }
    std::fs::set_permissions(&record, std::fs::Permissions::from_mode(0o600)).expect("restore");
    remove(root.as_path());
}

/// The owner is alive only with its PID and the identity that PID has now.
#[cfg(target_os = "linux")]
#[test]
fn liveness_needs_the_same_birth() {
    use crate::process_identity::current_birth_identity;
    let own: OwnerLockRecord = OwnerLockRecord {
        token: String::from("t"),
        owner_pid: i64::from(std::process::id()),
        owner_birth_identity: current_birth_identity().expect("own identity"),
        transaction_id: None,
        owner_pid_namespace: None,
    };
    assert!(owner_lock_holder_is_alive(&own, Path::new("/absent")).expect("own"));
    let reused: OwnerLockRecord = OwnerLockRecord {
        owner_birth_identity: String::from("linux:1"),
        ..own.clone()
    };
    assert!(!owner_lock_holder_is_alive(&reused, Path::new("/absent")).expect("reused PID"));
    let gone: OwnerLockRecord = OwnerLockRecord {
        owner_pid: i64::from(i32::MAX),
        ..own
    };
    assert!(!owner_lock_holder_is_alive(&gone, Path::new("/absent")).expect("no such process"));
}

/// The namespace is an optional last field that round-trips; an absent, empty or mistyped one reads as none.
#[test]
fn the_namespace_is_an_optional_last_field() {
    let mut named: OwnerLockRecord = hook_lock_record();
    named.owner_pid_namespace = Some(String::from("pid:[4026531836]"));
    let encoded: String = encode_owner_lock_record(&named);
    assert!(encoded.ends_with(",\"ownerPidNamespace\":\"pid:[4026531836]\"}\n"), "{encoded}");
    assert_eq!(parse_owner_lock_record(encoded.as_str()), Some(named));
    assert_eq!(
        parse_owner_lock_record(INCUMBENT_HOOK_LOCK).expect("old record").owner_pid_namespace,
        None
    );
    for ignored in ["\"\"", "5", "null", "[]"] {
        let text: String = INCUMBENT_HOOK_LOCK.replace("}\n", &format!(",\"ownerPidNamespace\":{ignored}}}\n"));
        assert_eq!(
            parse_owner_lock_record(text.as_str()).expect(ignored).owner_pid_namespace,
            None,
            "{ignored}"
        );
    }
}

/// An owner of another PID namespace is alive while its record is fresh and no PID probe decides (issues 1781 and 1784).
#[cfg(target_os = "linux")]
#[test]
fn a_holder_of_another_namespace_is_alive_until_its_record_expires() {
    let root: PathBuf = fixture("lock-foreign-namespace");
    let record_path: PathBuf = root.join(OWNER_LOCK_RECORD_FILENAME);
    std::fs::write(record_path.as_path(), INCUMBENT_HOOK_LOCK).expect("record");
    // A PID that names no process here: the old rule judged this owner dead.
    let mut foreign: OwnerLockRecord = hook_lock_record();
    foreign.owner_pid = i64::from(i32::MAX);
    foreign.owner_pid_namespace = Some(String::from("pid:[1]"));
    assert!(owner_lock_holder_is_alive(&foreign, root.as_path()).expect("fresh"));
    let expired: std::time::SystemTime =
        std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(record_path.as_path())
        .expect("open")
        .set_modified(expired)
        .expect("age");
    assert!(!owner_lock_holder_is_alive(&foreign, root.as_path()).expect("expired"));
    // The same PID without a foreign namespace is judged by the PID, as before.
    foreign.owner_pid_namespace = None;
    assert!(!owner_lock_holder_is_alive(&foreign, root.as_path()).expect("no namespace"));
    foreign.owner_pid_namespace = crate::pid_namespace::current_pid_namespace();
    assert!(!owner_lock_holder_is_alive(&foreign, root.as_path()).expect("same namespace"));
    remove(root.as_path());
}

/// The record limit is 64 KiB: a record far over a few hundred bytes still reads.
#[test]
fn a_record_of_a_few_kilobytes_is_read() {
    let root: PathBuf = fixture("lock-record-limit");
    let long_token: String = "t".repeat(4096);
    let record: String = INCUMBENT_HOOK_LOCK.replace("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10", long_token.as_str());
    std::fs::write(root.join(OWNER_LOCK_RECORD_FILENAME), record).expect("record");
    assert!(matches!(read_owner_lock_record(root.as_path()), Ok(PublishedOwner::Owner(_))));
    assert_eq!(OWNER_LOCK_RECORD_LIMIT, 65_536);
    remove(root.as_path());
}
