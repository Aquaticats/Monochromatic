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
