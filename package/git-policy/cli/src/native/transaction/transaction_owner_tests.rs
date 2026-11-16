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
    let mut child: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("60"))
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
        assert!(started.elapsed() < std::time::Duration::from_secs(60), "the killed sleeper still looks alive");
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
        Err(OwnerLivenessError::ForeignRecordRefused(text)) => panic!("a bare PID is no foreign record: {text}"),
    }
}

/// A record written by an owner that names its namespace round-trips with the field last;
/// one without it parses to none.
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
    // Past the patience the record is neither alive nor dead (owner, 2026-11-04, "Refuse and name it (Recommended)"): the
    // classifier refuses, and the error names the record, the owner and its namespace.
    let refused = classify_transaction_owner_record(&owner, old.as_path()).expect_err("an expired foreign record is refused");
    let text: String = format!("{refused:?}");
    assert!(text.contains("pid:[1]") && text.contains(old.to_string_lossy().as_ref()), "{text}");
}

/// A record of another PID namespace that is gone is a dead owner,
/// not a refusal:
/// its owner finished and removed the record,
/// so the recovery that read the parsed copy finds nothing in its way.
/// Why:
///  Reading the absence as an unreadable record stopped the command with an instruction to remove a file
///  that no longer exists,
///  after a concurrent merge had completed (cross-vendor review of 2026-11-09,
/// finding 3).
#[cfg(target_os = "linux")]
#[test]
fn an_owner_of_another_namespace_whose_record_is_gone_is_dead_and_not_refused() {
    let directory: std::path::PathBuf = crate::test_support::fixture("owner-namespace-gone");
    let mut owner: TransactionOwner = incumbent_owner();
    owner.owner_pid = i64::from(i32::MAX);
    owner.owner_pid_namespace = Some(String::from("pid:[1]"));
    let record: std::path::PathBuf = directory.join("owner.json");
    assert_eq!(
        classify_transaction_owner_record(&owner, record.as_path()).expect("a gone record is no refusal"),
        OwnerLiveness::Dead
    );
}

/// A record of this namespace,
/// or of none,
/// is still judged by its PID.
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

/// What:
///  A record of a live owner whose birth identity is spelled another way is never a dead owner.
/// Why:
///  The wrapper writes one spelling of its own birth,
///  so a second spelling in a record was written by something that is not this release:
///  an older release,
///  another tool,
///  or a damaged file.
///  Read as another process,
///  it settles what a running command holds:
///  recovery discards a merge's record and drops the refs that keep its commit,
///  its kept copies are put back or removed beside its paths,
///  and its branch is read as one nobody is working on.
///  A spelling a reader cannot compare is no proof of a dead owner,
///  so the owner counts as one that may still run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const respelled of spellings(owner)) expect(classifyTransactionOwner(pid, respelled)).not.toBe('dead');
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_live_owners_record_in_another_spelling_is_never_dead() {
    let own: TransactionOwner = current_transaction_owner("a-live-owner", "2026-10-29T00:00:00.000Z").expect("this process has an identity");
    let spellings: [(&str, String); 3] = [
        ("a leading zero on the start tick", crate::process_identity_spellings::birth_with_a_leading_zero(own.owner_identity.as_str())),
        ("the platform's tag in capitals", crate::process_identity_spellings::birth_with_its_platform_tag_in_capitals(own.owner_identity.as_str())),
        ("a space before the birth", crate::process_identity_spellings::birth_behind_a_space(own.owner_identity.as_str())),
    ];
    let mut wrong: Vec<String> = Vec::new();
    for (what, respelled) in spellings {
        if classify_transaction_owner(own.owner_pid, respelled.as_str()).expect("a record of this namespace") == OwnerLiveness::Dead {
            wrong.push(format!("{what}: the reader called process {} a dead owner of {respelled}, and it runs", own.owner_pid));
        }
    }
    // The other answer of the same reader, so an arm that passes is not a reader that never answers "dead": a birth no
    // process has is a dead owner's.
    assert_eq!(
        classify_transaction_owner(own.owner_pid, "linux:1").expect("a number that runs"),
        OwnerLiveness::Dead,
        "a process with another birth is a dead owner"
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// What:
///  A record of a live owner whose PID namespace is spelled another way is never a dead owner either.
/// Why:
///  The namespace is a number,
///  and one number has one value however many zeros lead it;
///  read as another namespace,
///  the record is judged by its own age,
///  which keeps it or refuses the command and never takes it.
///  The arm is here so the report says what each part of the record does with a second spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(classifyTransactionOwnerRecord(paddedScope(owner), fresh)).not.toBe('dead');
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_live_owners_record_with_its_namespace_spelled_another_way_is_never_dead() {
    let directory: std::path::PathBuf = crate::test_support::fixture("owner-namespace-spelled-another-way");
    let mut owner: TransactionOwner = current_transaction_owner("a-live-owner", "2026-10-29T00:00:00.000Z").expect("this process has an identity");
    let namespace: String = owner.owner_pid_namespace.clone().expect("this host reports the PID namespace of a process");
    owner.owner_pid_namespace = Some(crate::process_identity_spellings::namespace_with_a_leading_zero(namespace.as_str()));
    let fresh: std::path::PathBuf = owner_file(directory.as_path(), std::time::Duration::from_secs(30));
    let verdict: Result<OwnerLiveness, OwnerLivenessError> = classify_transaction_owner_record(&owner, fresh.as_path());
    crate::test_support::remove(directory.as_path());
    assert_ne!(
        verdict.expect("a young record of another namespace is no refusal"),
        OwnerLiveness::Dead,
        "a namespace with a leading zero is the same namespace, so nothing proves the owner gone"
    );
}

/// What:
///  the answer a row of an owner table is due.
/// Why:
///  A record's holder can be shown to run,
///  shown to have ended,
///  or neither,
///  and the last has words a person reads: a row that says only "not alive" would let a silent wait pass as a pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Due {
    /// The pid names the owner.
    Alive,
    /// The process ended, or the number was reused by a process of another birth.
    Dead,
    /// Neither, and the record is named in the words a command stops with.
    Refused,
}

/// What:
///  A transaction record is read by the value its birth names, and a record whose birth names no value is bounded by
///  its own age.
/// Why:
///  The wrapper writes one spelling of a birth,
///  so a second one came from a writer that is not this release,
///  and a spelling is a value written another way: two spellings of one start tick are one process,
///  and two ticks are two.
///  A text that names no tick is neither a live owner's nor a gone one's,
///  and the record's own age is the only bound on it:
///  inside the patience the record stands for an owner that may run,
///  and past it the command stops with the record named and the command that removes it.
///  The namespace is a number too: one number written with a leading zero is that namespace, not another one.
#[cfg(target_os = "linux")]
#[test]
fn a_record_is_read_by_the_value_its_birth_names_and_by_its_own_age() {
    let directory: std::path::PathBuf = crate::test_support::fixture("transaction-owner-value-table");
    let own: TransactionOwner = current_transaction_owner("the-live-owner", "2026-10-29T00:00:00.000Z").expect("this process has an identity");
    let live: String = own.owner_identity.clone();
    let namespace: String = own.owner_pid_namespace.clone().expect("this host reports the PID namespace of a process");
    let young: std::time::Duration = std::time::Duration::from_secs(30);
    let aged: std::time::Duration = crate::pid_namespace::FOREIGN_OWNER_PATIENCE + std::time::Duration::from_secs(60);
    let capitalised: String = crate::process_identity_spellings::birth_with_its_platform_tag_in_capitals(live.as_str());
    let padded: String = crate::process_identity_spellings::birth_with_a_leading_zero(live.as_str());
    let behind_a_space: String = crate::process_identity_spellings::birth_behind_a_space(live.as_str());
    let padded_namespace: String = crate::process_identity_spellings::namespace_with_a_leading_zero(namespace.as_str());
    // What each row names: the birth, the namespace the record writes (none means the field is absent), the age of the
    // record, the answer due, and the words a refusal must say besides the record's path.
    let rows: [(&str, String, Option<String>, std::time::Duration, Due, &str); 10] = [
        ("the birth as the wrapper writes it", live.clone(), None, young, Due::Alive, ""),
        ("the platform's tag in capitals", capitalised.clone(), None, young, Due::Alive, ""),
        ("a leading zero on the start tick", padded.clone(), None, young, Due::Alive, ""),
        ("a start tick of another value", String::from("linux:1"), None, young, Due::Dead, ""),
        ("the tag in capitals over another tick", String::from("LINUX:1"), None, young, Due::Dead, ""),
        ("a birth behind a space", behind_a_space.clone(), None, young, Due::Alive, ""),
        ("a text naming the tag with no tick, young", String::from("linux:not-a-birth"), None, young, Due::Alive, ""),
        ("a text naming the tag with no tick, past the patience", String::from("linux:not-a-birth"), None, aged, Due::Refused, "birth identity"),
        ("a leading zero on the namespace, past the patience", live.clone(), Some(padded_namespace.clone()), aged, Due::Alive, ""),
        ("another namespace, past the patience", live.clone(), Some(String::from("pid:[1]")), aged, Due::Refused, "PID namespace"),
    ];
    // Every row is asked and every wrong answer is kept, so one run of the control names each row that was read another
    //  way instead of stopping at the first.
    let mut wrong: Vec<String> = Vec::new();
    for (what, birth, recorded_namespace, age, due, must_say) in rows {
        let mut owner: TransactionOwner = own.clone();
        owner.owner_identity = birth.clone();
        owner.owner_pid_namespace = recorded_namespace;
        let record: std::path::PathBuf = owner_file(directory.as_path(), age);
        match (classify_transaction_owner_record(&owner, record.as_path()), due) {
            (Ok(OwnerLiveness::Alive), Due::Alive) | (Ok(OwnerLiveness::Dead), Due::Dead) => {}
            (Err(OwnerLivenessError::ForeignRecordRefused(text)), Due::Refused) => {
                let named: bool = text.contains(record.to_string_lossy().as_ref()) && text.contains(own.owner_pid.to_string().as_str());
                if !named || !text.contains("rm -- ") || !text.contains(must_say) {
                    wrong.push(format!("{what}: the refusal does not name the record, the owner, the command that removes it and {must_say:?}: {text}"));
                }
            }
            (verdict, _) => {
                wrong.push(format!("{what}: the reader answered {verdict:?} for {birth}, and {due:?} is due"));
            }
        }
    }
    crate::test_support::remove(directory.as_path());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
