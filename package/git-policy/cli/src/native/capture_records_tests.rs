//! Controls for capture records,
//!  landed records,
//!  recording and pruning.

use super::*;
use crate::capture_store::{SEQUENCE_FILENAME, WORKTREE_ID_FILENAME};
use crate::private_storage::take_directory_syncs;
use crate::test_support::{fixture, remove};
use crate::transaction_registry::{OWNER_FILENAME, TRANSACTION_ROOT_NAME};

/// A capture record with every field set.
fn captured(worktree_id: &str, next: i64) -> CapturedRecord {
    return CapturedRecord {
        worktree_id: String::from(worktree_id),
        sequence: 3,
        next_sequence_before_base: next,
        worktree_paths: vec![
            String::from("a.txt"),
            String::from("d\u{e9}j\u{e0} \"q\".txt"),
        ],
    };
}

/// `captured.json` has the incumbent's field order and round-trips;
///  malformed fields fail.
#[test]
fn captured_records_round_trip() {
    let record: CapturedRecord = captured("w", 2);
    let encoded: String = encode_captured_record(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"w\",\"sequence\":3,\
         \"nextSequenceBeforeBase\":2,\"worktreePaths\":[\"a.txt\",\"d\u{e9}j\u{e0} \\\"q\\\".txt\"]}\n"
    );
    assert_eq!(parse_captured_record(encoded.as_str()), Ok(record));
    let cases: [(&str, &str); 9] = [
        ("[]", "captured.json is not a JSON object."),
        ("{", "captured.json is not a JSON object."),
        (
            "{\"schemaVersion\":1,\"state\":\"captured\"}",
            "captured.json is not a schema-version-2 captured record.",
        ),
        (
            "{\"schemaVersion\":2,\"state\":\"prepared\"}",
            "captured.json is not a schema-version-2 captured record.",
        ),
        (
            "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"\"}",
            "captured.json has a malformed worktreeId field.",
        ),
        (
            "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"w\",\"sequence\":0}",
            "captured.json has a malformed sequence field.",
        ),
        (
            "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"w\",\"sequence\":1,\"nextSequenceBeforeBase\":1.5}",
            "captured.json has a malformed nextSequenceBeforeBase field.",
        ),
        (
            "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"w\",\"sequence\":1,\"nextSequenceBeforeBase\":1,\"worktreePaths\":[\"\"]}",
            "captured.json has a malformed worktreePaths field.",
        ),
        (
            "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"w\",\"sequence\":1,\"nextSequenceBeforeBase\":1,\"worktreePaths\":\"a\"}",
            "captured.json has a malformed worktreePaths field.",
        ),
    ];
    for (text, message) in cases {
        assert_eq!(
            parse_captured_record(text),
            Err(CaptureError(String::from(message))),
            "{text}"
        );
    }
    let mixed: &str = "{\"schemaVersion\":2,\"state\":\"captured\",\"worktreeId\":\"w\",\"sequence\":1,\"nextSequenceBeforeBase\":1,\"worktreePaths\":[\"a\",7]}";
    assert!(parse_captured_record(mixed).is_err());
}

/// Landed records have the incumbent's field order and round-trip.
#[test]
fn landed_records_round_trip() {
    let record: LandedCaptureRecord = LandedCaptureRecord {
        commit: String::from("c0"),
        transaction_id: String::from("t"),
        worktree_id: String::from("w"),
        sequence: 3,
        next_sequence_after_landing: 9,
        worktree_paths: vec![String::from("a")],
    };
    let encoded: String = encode_landed_capture_record(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":1,\"commit\":\"c0\",\"transactionId\":\"t\",\"worktreeId\":\"w\",\"sequence\":3,\
         \"nextSequenceAfterLanding\":9,\"worktreePaths\":[\"a\"]}\n"
    );
    assert_eq!(
        parse_landed_capture_record(encoded.as_str(), "n"),
        Ok(record)
    );
    assert_eq!(
        parse_landed_capture_record("{\"schemaVersion\":2}", "n"),
        Err(CaptureError(String::from(
            "n is not a schema-version-1 landed-capture record."
        )))
    );
    assert_eq!(
        parse_landed_capture_record("{\"schemaVersion\":1,\"commit\":7}", "n"),
        Err(CaptureError(String::from(
            "n has a malformed commit field."
        )))
    );
    assert_eq!(
        parse_landed_capture_record("null", "n"),
        Err(CaptureError(String::from("n is not a JSON object.")))
    );
    assert_eq!(
        landed_record_directory(Path::new("/repo/.git")),
        PathBuf::from("/repo/.git/cli-git-captures/landed")
    );
}

/// A landed record is written once from `captured.json`,
///  with the next sequence after landing.
#[test]
fn landings_record_their_capture_once() {
    let root: PathBuf = fixture("capture-records-record");
    let git_dir: PathBuf = root.join("git");
    std::fs::create_dir(&git_dir).expect("git dir");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    assert_eq!(read_captured_record(transaction.as_path()), Ok(None));
    assert_eq!(
        record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c0"),
        Ok(false)
    );
    std::fs::write(
        transaction.join(CAPTURED_FILENAME),
        encode_captured_record(&captured("w", 2)),
    )
    .expect("captured");
    assert_eq!(
        read_captured_record(transaction.as_path()),
        Ok(Some(captured("w", 2)))
    );
    let store: PathBuf =
        crate::capture_store::ensure_capture_store(git_dir.as_path()).expect("store");
    std::fs::write(store.join(SEQUENCE_FILENAME), b"6\n").expect("sequence");
    assert_eq!(
        record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c0"),
        Ok(true)
    );
    let path: PathBuf = landed_record_directory(git_dir.as_path()).join("c0.json");
    let written: String = std::fs::read_to_string(&path).expect("landed record");
    let parsed: LandedCaptureRecord =
        parse_landed_capture_record(written.as_str(), "c0").expect("parsed");
    assert_eq!(parsed.next_sequence_after_landing, 7);
    assert_eq!(parsed.sequence, 3);
    assert_eq!(parsed.worktree_paths, captured("w", 2).worktree_paths);
    // Repeating keeps the existing record.
    std::fs::write(store.join(SEQUENCE_FILENAME), b"8\n").expect("later sequence");
    assert_eq!(
        record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c0"),
        Ok(true)
    );
    assert_eq!(std::fs::read_to_string(&path).expect("kept"), written);
    assert!(record_landed_capture_or_warn(
        git_dir.as_path(),
        transaction.as_path(),
        "t",
        "c0"
    ));
    // A malformed capture is warned about, not returned.
    std::fs::write(transaction.join(CAPTURED_FILENAME), b"{}").expect("malformed");
    assert!(record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c1").is_err());
    assert!(!record_landed_capture_or_warn(
        git_dir.as_path(),
        transaction.as_path(),
        "t",
        "c1"
    ));
    // A malformed sequence file fails the record.
    std::fs::write(
        transaction.join(CAPTURED_FILENAME),
        encode_captured_record(&captured("w", 2)),
    )
    .expect("captured");
    std::fs::write(store.join(SEQUENCE_FILENAME), b"x\n").expect("malformed sequence");
    assert!(record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c2").is_err());
    remove(root.as_path());
}

/// Recording a landing syncs the directories it creates and the record directory it writes into;
///  recording the same landing again finds everything in place and syncs nothing.
#[test]
fn a_repeated_landing_record_syncs_nothing() {
    let root: PathBuf = fixture("capture-records-record-sync");
    let git_dir: PathBuf = root.join("git");
    std::fs::create_dir(&git_dir).expect("git dir");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    std::fs::write(
        transaction.join(CAPTURED_FILENAME),
        encode_captured_record(&captured("w", 2)),
    )
    .expect("captured");
    let store: PathBuf = git_dir.join(crate::capture_store::CAPTURE_STORE_NAME);
    let landed_directory: PathBuf = landed_record_directory(git_dir.as_path());
    take_directory_syncs();
    assert_eq!(
        record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c0"),
        Ok(true)
    );
    assert_eq!(
        take_directory_syncs(),
        vec![git_dir.clone(), store.clone(), landed_directory.clone()]
    );
    assert_eq!(
        record_landed_capture(git_dir.as_path(), transaction.as_path(), "t", "c0"),
        Ok(true)
    );
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// A published transaction directory with a capture record.
fn published(registry: &Path, id: &str, capture: Option<&CapturedRecord>) {
    let directory: PathBuf = registry.join(id);
    std::fs::create_dir_all(&directory).expect("transaction");
    std::fs::write(directory.join(OWNER_FILENAME), b"{}").expect("owner placeholder");
    if let Some(record) = capture {
        std::fs::write(
            directory.join(CAPTURED_FILENAME),
            encode_captured_record(record),
        )
        .expect("captured");
    }
}

/// A landed record file.
fn landed(git_dir: &Path, commit: &str, worktree_id: &str, next: i64) {
    let directory: PathBuf = landed_record_directory(git_dir);
    std::fs::create_dir_all(&directory).expect("landed directory");
    let record: LandedCaptureRecord = LandedCaptureRecord {
        commit: String::from(commit),
        transaction_id: String::from("t"),
        worktree_id: String::from(worktree_id),
        sequence: 1,
        next_sequence_after_landing: next,
        worktree_paths: Vec::new(),
    };
    std::fs::write(
        directory.join(format!("{commit}.json")),
        encode_landed_capture_record(&record),
    )
    .expect("landed");
}

/// The landed record names left in a directory,
///  sorted.
fn left(git_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = list_landed_record_names(git_dir).expect("names");
    names.sort();
    return names;
}

/// Pruning keeps records a published transaction may replay over and removes the rest.
#[test]
fn pruning_follows_the_pinning_bound() {
    let root: PathBuf = fixture("capture-records-prune");
    let git_dir: PathBuf = root.join("git");
    std::fs::create_dir(&git_dir).expect("git dir");
    let registry: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        0
    );
    let store: PathBuf =
        crate::capture_store::ensure_capture_store(git_dir.as_path()).expect("store");
    std::fs::write(store.join(WORKTREE_ID_FILENAME), b"w\n").expect("identity");
    landed(git_dir.as_path(), "c1", "w", 2);
    landed(git_dir.as_path(), "c2", "w", 5);
    landed(git_dir.as_path(), "c3", "other", 9);
    let directory: PathBuf = landed_record_directory(git_dir.as_path());
    std::fs::write(directory.join("broken.json"), b"{").expect("malformed");
    std::fs::write(directory.join("stray.tmp"), b"").expect("not a record");
    // A published transaction that has not captured yet keeps everything.
    published(
        registry.as_path(),
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        None,
    );
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        0
    );
    assert_eq!(left(git_dir.as_path()).len(), 5);
    // Once it captured with bound 3: c1 (2 < 3), c3 (other store), broken and stray go.
    published(
        registry.as_path(),
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        Some(&captured("w", 3)),
    );
    // A transaction of another store generation does not pin anything.
    published(
        registry.as_path(),
        "1b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        Some(&captured("old", 1)),
    );
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        4
    );
    assert_eq!(left(git_dir.as_path()), vec![String::from("c2.json")]);
    // With no transaction left every record goes.
    std::fs::remove_dir_all(&registry).expect("no transactions");
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        1
    );
    assert!(left(git_dir.as_path()).is_empty());
    // Failures are reported and change nothing.
    landed(git_dir.as_path(), "c4", "w", 1);
    std::fs::write(&registry, b"").expect("registry is a file");
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        0
    );
    assert_eq!(left(git_dir.as_path()), vec![String::from("c4.json")]);
    std::fs::remove_file(&registry).expect("restore registry");
    std::fs::create_dir(directory.join("nested.json")).expect("directory record");
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        0
    );
    remove(root.as_path());
}

/// A store without identity prunes every record as another generation's.
#[test]
fn records_without_a_store_identity_are_pruned() {
    let root: PathBuf = fixture("capture-records-no-identity");
    let git_dir: PathBuf = root.join("git");
    std::fs::create_dir(&git_dir).expect("git dir");
    landed(git_dir.as_path(), "c1", "w", 9);
    let registry: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    published(
        registry.as_path(),
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        Some(&captured("w", 1)),
    );
    assert_eq!(
        prune_landed_captures(git_dir.as_path(), registry.as_path()),
        1
    );
    std::fs::write(landed_record_directory(git_dir.as_path()), b"").expect_err("directory exists");
    std::fs::remove_dir_all(landed_record_directory(git_dir.as_path())).expect("remove");
    std::fs::write(landed_record_directory(git_dir.as_path()), b"").expect("landed is a file");
    assert!(list_landed_record_names(git_dir.as_path()).is_err());
    remove(root.as_path());
}
