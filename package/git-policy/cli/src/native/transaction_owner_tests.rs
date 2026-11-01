//! Controls for the transaction owner record and owner liveness.

use super::*;

/// The record the incumbent's `encodeTransactionOwner` writes.
const INCUMBENT_OWNER: &str = "{\"schemaVersion\":2,\"transactionId\":\"0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10\",\
\"ownerPid\":4242,\"ownerIdentity\":\"linux:8423337\",\"createdAt\":\"2026-10-06T21:34:01.123Z\"}\n";

/// The fields of the incumbent record.
fn incumbent_owner() -> TransactionOwner {
    return TransactionOwner {
        transaction_id: String::from("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
        owner_pid: 4242,
        owner_identity: String::from("linux:8423337"),
        created_at: String::from("2026-10-06T21:34:01.123Z"),
        owner_pid_namespace: None,
    };
}

/// The incumbent's record parses,
///  and encoding reproduces its bytes.
#[test]
fn incumbent_owner_records_round_trip() {
    assert_eq!(
        parse_transaction_owner(INCUMBENT_OWNER.as_bytes()),
        Some(incumbent_owner())
    );
    assert_eq!(
        encode_transaction_owner(&incumbent_owner()),
        INCUMBENT_OWNER
    );
    let mut marked: Vec<u8> = b"\xef\xbb\xbf".to_vec();
    marked.extend_from_slice(INCUMBENT_OWNER.as_bytes());
    assert_eq!(
        parse_transaction_owner(marked.as_slice()),
        Some(incumbent_owner())
    );
}

/// Every shape `parseTransactionOwner` refuses is refused,
///  and its odd acceptances accepted.
#[test]
fn malformed_owner_records_are_refused() {
    for text in [
        "",
        "[]",
        "{\"schemaVersion\":1,\"transactionId\":\"t\",\"ownerPid\":1,\"ownerIdentity\":\"i\",\"createdAt\":\"c\"}",
        "{\"schemaVersion\":2,\"transactionId\":7,\"ownerPid\":1,\"ownerIdentity\":\"i\",\"createdAt\":\"c\"}",
        "{\"schemaVersion\":2,\"transactionId\":\"t\",\"ownerPid\":0,\"ownerIdentity\":\"i\",\"createdAt\":\"c\"}",
        "{\"schemaVersion\":2,\"transactionId\":\"t\",\"ownerPid\":1.5,\"ownerIdentity\":\"i\",\"createdAt\":\"c\"}",
        "{\"schemaVersion\":2,\"transactionId\":\"t\",\"ownerPid\":1,\"ownerIdentity\":\"\",\"createdAt\":\"c\"}",
        "{\"schemaVersion\":2,\"transactionId\":\"t\",\"ownerPid\":1,\"ownerIdentity\":\"i\"}",
        "{\"schemaVersion\":2,\"transactionId\":\"t\",\"ownerPid\":1,\"ownerIdentity\":\"i\",\"createdAt\":5}",
        "{\"schemaVersion\":2,\"ownerPid\":1,\"ownerIdentity\":\"i\",\"createdAt\":\"c\"}",
    ] {
        assert_eq!(parse_transaction_owner(text.as_bytes()), None, "{text}");
    }
    assert_eq!(
        parse_transaction_owner(b"{\"schemaVersion\":2,\"transactionId\":\"\xff\"}"),
        None,
        "invalid UTF-8 is refused by the fatal decoder"
    );
    let empty: TransactionOwner = parse_transaction_owner(
        b"{\"schemaVersion\":2,\"transactionId\":\"\",\"ownerPid\":1,\"ownerIdentity\":\"i\",\"createdAt\":\"\"}",
    )
    .expect("empty ID and time are text");
    assert_eq!(empty.transaction_id, "");
    assert_eq!(empty.created_at, "");
}

/// The current process's record names this process.
#[test]
fn the_current_owner_is_this_process() {
    let owner: TransactionOwner =
        current_transaction_owner("id", "2026-10-06T00:00:00.000Z").expect("own identity");
    assert_eq!(owner.owner_pid, i64::from(std::process::id()));
    assert_eq!(owner.transaction_id, "id");
    assert_eq!(owner.created_at, "2026-10-06T00:00:00.000Z");
    assert_eq!(
        classify_transaction_owner(owner.owner_pid, owner.owner_identity.as_str()).expect("own"),
        OwnerLiveness::Alive
    );
}

/// A reused PID,
///  an exited process and a zombie are dead owners.
#[cfg(target_os = "linux")]
#[test]
fn dead_owners_are_dead() {
    let own: i64 = i64::from(std::process::id());
    assert_eq!(
        classify_transaction_owner(own, "linux:1").expect("reused"),
        OwnerLiveness::Dead
    );
    assert_eq!(
        classify_transaction_owner(i64::from(i32::MAX), "linux:1").expect("gone"),
        OwnerLiveness::Dead
    );
    let mut child: std::process::Child = std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("sleeper");
    let pid: i64 = i64::from(child.id());
    let identity: String = crate::process_identity::process_birth_identity(pid)
        .expect("probe")
        .expect("alive");
    assert_eq!(
        classify_transaction_owner(pid, identity.as_str()).expect("alive"),
        OwnerLiveness::Alive
    );
    child.kill().expect("kill");
    let started: std::time::Instant = std::time::Instant::now();
    while classify_transaction_owner(pid, identity.as_str()).expect("zombie")
        == OwnerLiveness::Alive
    {
        assert!(started.elapsed() < std::time::Duration::from_secs(10));
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    child.wait().expect("reap");
    assert_eq!(
        classify_transaction_owner(pid, identity.as_str()).expect("reaped"),
        OwnerLiveness::Dead
    );
}

/// A refused existence probe is an error,
///  never a verdict.
#[cfg(target_os = "linux")]
#[test]
fn a_refused_probe_is_an_error() {
    // PID 1 belongs to another account in the gate and on most hosts.
    match classify_transaction_owner(1, "linux:1") {
        Err(OwnerLivenessError::Probe(error)) => {
            assert_eq!(error.raw_os_error(), Some(libc::EPERM));
        }
        Ok(verdict) => assert_eq!(
            verdict,
            OwnerLiveness::Dead,
            "a signalable PID 1 is not linux:1"
        ),
        Err(OwnerLivenessError::Identity(error)) => panic!("unexpected identity failure: {error}"),
    }
}

/// A record written by an owner that names its namespace round-trips with the field last; one without it parses to none.
#[test]
fn the_namespace_is_an_optional_last_field() {
    let mut named: TransactionOwner = incumbent_owner();
    named.owner_pid_namespace = Some(String::from("pid:[4026531836]"));
    let encoded: String = encode_transaction_owner(&named);
    assert!(encoded.ends_with(",\"ownerPidNamespace\":\"pid:[4026531836]\"}\n"), "{encoded}");
    assert_eq!(parse_transaction_owner(encoded.as_bytes()), Some(named));
    assert_eq!(incumbent_owner().owner_pid_namespace, None);
    assert_eq!(
        parse_transaction_owner(INCUMBENT_OWNER.as_bytes()).expect("old record").owner_pid_namespace,
        None
    );
    for ignored in ["\"\"", "5", "null", "[]"] {
        let text: String = INCUMBENT_OWNER.replace("}\n", &format!(",\"ownerPidNamespace\":{ignored}}}\n"));
        assert_eq!(
            parse_transaction_owner(text.as_bytes()).expect(ignored).owner_pid_namespace,
            None,
            "{ignored}"
        );
    }
}

/// A fresh owner record names the namespace of this process.
#[cfg(target_os = "linux")]
#[test]
fn the_current_owner_names_its_namespace() {
    let owner: TransactionOwner = current_transaction_owner("id", "2026-10-06T00:00:00.000Z").expect("identity");
    assert_eq!(owner.owner_pid_namespace, crate::pid_namespace::current_pid_namespace());
    assert!(owner.owner_pid_namespace.is_some());
}

/// The record file whose age and PID the classifier is given.
#[cfg(target_os = "linux")]
fn owner_file(directory: &std::path::Path, age: std::time::Duration) -> std::path::PathBuf {
    let path: std::path::PathBuf = directory.join("owner.json");
    let file: std::fs::File = std::fs::File::create(path.as_path()).expect("create");
    file.set_modified(std::time::SystemTime::now() - age).expect("age");
    return path;
}

/// An owner of another PID namespace is not judged by a PID this process cannot interpret (issues 1781 and 1784).
#[cfg(target_os = "linux")]
#[test]
fn an_owner_of_another_namespace_is_alive_until_its_record_expires() {
    let directory: std::path::PathBuf = crate::test_support::fixture("owner-namespace-1");
    // A PID that names no process here: the old rule judged this owner dead.
    let mut owner: TransactionOwner = incumbent_owner();
    owner.owner_pid = i64::from(i32::MAX);
    owner.owner_pid_namespace = Some(String::from("pid:[1]"));
    let fresh = owner_file(directory.as_path(), std::time::Duration::from_secs(30));
    assert_eq!(
        classify_transaction_owner_record(&owner, fresh.as_path()).expect("fresh"),
        OwnerLiveness::Alive
    );
    let old = owner_file(
        directory.as_path(),
        crate::pid_namespace::FOREIGN_OWNER_PATIENCE + std::time::Duration::from_secs(60),
    );
    assert_eq!(
        classify_transaction_owner_record(&owner, old.as_path()).expect("old"),
        OwnerLiveness::Dead
    );
}

/// A record of this namespace, or of none, is still judged by its PID.
#[cfg(target_os = "linux")]
#[test]
fn an_owner_of_this_namespace_or_none_is_judged_by_its_pid() {
    let directory: std::path::PathBuf = crate::test_support::fixture("owner-namespace-2");
    let fresh = owner_file(directory.as_path(), std::time::Duration::from_secs(30));
    let mut owner: TransactionOwner = incumbent_owner();
    owner.owner_pid = i64::from(i32::MAX);
    owner.owner_pid_namespace = crate::pid_namespace::current_pid_namespace();
    assert_eq!(
        classify_transaction_owner_record(&owner, fresh.as_path()).expect("same"),
        OwnerLiveness::Dead
    );
    owner.owner_pid_namespace = None;
    assert_eq!(
        classify_transaction_owner_record(&owner, fresh.as_path()).expect("none"),
        OwnerLiveness::Dead
    );
    let own: TransactionOwner = current_transaction_owner("id", "2026-10-06T00:00:00.000Z").expect("own");
    assert_eq!(
        classify_transaction_owner_record(&own, fresh.as_path()).expect("alive"),
        OwnerLiveness::Alive
    );
}
