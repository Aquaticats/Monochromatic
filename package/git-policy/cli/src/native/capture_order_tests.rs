//! Controls for the capture under capture order:
//!  the lock, the identity, the sequence, the worktree listing and `captured.json`.

use super::*;
use crate::capture_records::read_captured_record;
use crate::capture_store::CAPTURE_STORE_NAME;
use crate::private_storage::take_directory_syncs;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::transaction_git::run_git_checked;

/// Fresh private index copied from the real one, so staging never touches the real index.
fn private_index(repo: &Path, name: &str) -> PathBuf {
    let path: PathBuf = repo.join(".git").join(name);
    std::fs::copy(repo.join(".git").join("index"), &path).expect("private index copy");
    return path;
}

/// A capture closure that always succeeds with no value.
fn nothing() -> Result<(), CaptureOrderError> {
    return Ok(());
}

/// Each capture takes the next sequence number, the identity stays stable,
///  and the lock directory is gone once the capture returns.
#[test]
fn captures_take_increasing_sequences_under_one_identity() {
    let root: PathBuf = fixture("capture-order-sequence");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let (first, ()) = capture_in_order(git_dir.as_path(), nothing).expect("first");
    let (second, ()) = capture_in_order(git_dir.as_path(), nothing).expect("second");
    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    assert_eq!(first.worktree_id, second.worktree_id);
    assert!(!first.worktree_id.is_empty());
    let store: PathBuf = git_dir.join(CAPTURE_STORE_NAME);
    let identity: String = std::fs::read_to_string(store.join(WORKTREE_ID_FILENAME)).expect("id");
    assert_eq!(identity, format!("{}\n", first.worktree_id));
    assert_eq!(std::fs::read_to_string(store.join(SEQUENCE_FILENAME)).expect("seq"), "2\n");
    assert!(!store.join(CAPTURE_LOCK_NAME).exists());
    assert_eq!(
        std::fs::read_dir(&store)
            .expect("store")
            .filter_map(Result::ok)
            .filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".tmp"); })
            .count(),
        0,
        "no temporary sequence file is left behind"
    );
    remove(root.as_path());
}

/// The worktree identity syncs the store when it writes the identity and never when it only reads it.
#[test]
fn the_identity_syncs_the_store_only_when_it_writes() {
    let root: PathBuf = fixture("capture-order-identity-sync");
    let store: PathBuf = root.join("store");
    std::fs::create_dir(&store).expect("store");
    take_directory_syncs();
    let written: String = ensure_worktree_id(store.as_path()).expect("written");
    assert_eq!(take_directory_syncs(), vec![store.clone()]);
    assert_eq!(ensure_worktree_id(store.as_path()).expect("read"), written);
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// A first capture syncs what it created (the Git directory for the store, the store for the identity and for the
///  sequence); a later capture syncs only the store, for its sequence rename.
#[test]
fn only_the_first_capture_syncs_the_creations() {
    let root: PathBuf = fixture("capture-order-creation-sync");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let store: PathBuf = git_dir.join(CAPTURE_STORE_NAME);
    take_directory_syncs();
    capture_in_order(git_dir.as_path(), nothing).expect("first");
    assert_eq!(take_directory_syncs(), vec![git_dir.clone(), store.clone(), store.clone()]);
    capture_in_order(git_dir.as_path(), nothing).expect("second");
    assert_eq!(take_directory_syncs(), vec![store]);
    remove(root.as_path());
}

/// A failing capture releases the lock and still returns its failure.
#[test]
fn a_failing_capture_releases_the_lock() {
    let root: PathBuf = fixture("capture-order-failure");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let failed: Result<(CaptureStamp, ()), CaptureOrderError> = capture_in_order(git_dir.as_path(), || {
        return Err(CaptureOrderError::Write(String::from("boom")));
    });
    assert!(matches!(failed, Err(CaptureOrderError::Write(text)) if text == "boom"));
    assert!(!git_dir.join(CAPTURE_STORE_NAME).join(CAPTURE_LOCK_NAME).exists());
    let (stamp, ()) = capture_in_order(git_dir.as_path(), nothing).expect("lock is free");
    assert_eq!(stamp.sequence, 2, "the failed capture still allocated its number");
    remove(root.as_path());
}

/// An empty identity file is a malformed store, not a new identity.
#[test]
fn an_empty_identity_is_rejected() {
    let root: PathBuf = fixture("capture-order-empty-identity");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let store: PathBuf = git_dir.join(CAPTURE_STORE_NAME);
    std::fs::create_dir(&store).expect("store");
    std::fs::write(store.join(WORKTREE_ID_FILENAME), "  \n").expect("empty identity");
    let failed: Result<(CaptureStamp, ()), CaptureOrderError> = capture_in_order(git_dir.as_path(), nothing);
    let text: String = failed.expect_err("empty").to_string();
    assert!(text.starts_with("Capture store worktree identity is empty: "), "{text}");
    remove(root.as_path());
}

/// A sequence file that is not a canonical decimal stops the capture before it writes anything.
#[test]
fn a_malformed_sequence_stops_the_capture() {
    let root: PathBuf = fixture("capture-order-malformed-sequence");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let store: PathBuf = git_dir.join(CAPTURE_STORE_NAME);
    std::fs::create_dir(&store).expect("store");
    std::fs::write(store.join(SEQUENCE_FILENAME), "07\n").expect("malformed");
    let failed: Result<(CaptureStamp, ()), CaptureOrderError> = capture_in_order(git_dir.as_path(), nothing);
    assert!(matches!(failed, Err(CaptureOrderError::Store(_))));
    assert_eq!(std::fs::read_to_string(store.join(SEQUENCE_FILENAME)).expect("seq"), "07\n");
    remove(root.as_path());
}

/// `write_sequence` replaces the number in place and leaves no temporary file.
#[test]
fn write_sequence_replaces_the_number() {
    let root: PathBuf = fixture("capture-order-write-sequence");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let store: PathBuf = ensure_capture_store(repo.join(".git").as_path()).expect("store");
    write_sequence(store.as_path(), 9).expect("write");
    write_sequence(store.as_path(), 10).expect("rewrite");
    assert_eq!(std::fs::read_to_string(store.join(SEQUENCE_FILENAME)).expect("seq"), "10\n");
    remove(root.as_path());
}

/// A store path that cannot receive the sequence file reports the store path in its failure.
#[test]
fn write_sequence_failure_names_the_store() {
    let root: PathBuf = fixture("capture-order-write-missing");
    let error: CaptureError =
        write_sequence(root.join("absent").as_path(), 1).expect_err("missing store");
    assert!(error.0.contains("absent"), "{}", error.0);
    remove(root.as_path());
}

/// `captured.json` round-trips through the record reader and is written once.
#[test]
fn captured_record_round_trips_and_refuses_a_second_write() {
    let root: PathBuf = fixture("capture-order-record");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let directory: PathBuf = root.join("transaction");
    std::fs::create_dir(&directory).expect("transaction directory");
    let record: CapturedRecord = CapturedRecord {
        worktree_id: String::from("worktree-1"),
        sequence: 3,
        next_sequence_before_base: 3,
        worktree_paths: vec![String::from("a.txt")],
    };
    write_captured_record(directory.as_path(), &record).expect("write");
    let read: Option<CapturedRecord> = read_captured_record(directory.as_path()).expect("read");
    assert_eq!(read, Some(record.clone()));
    assert!(write_captured_record(directory.as_path(), &record).is_err());
    assert!(repo.exists());
    remove(root.as_path());
}

/// Explicit-path captures list the paths the private index changes against the base.
#[test]
fn explicit_path_lists_the_changed_paths_against_the_base() {
    let root: PathBuf = fixture("capture-order-explicit");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    let base: String = crate::test_support::git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: private_index(repo.as_path(), "commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    run_git_checked(
        &git_context(),
        &indexed_request(repo.as_path(), &["add", "a.txt"], Some(indexes.commit_index.as_path()), None),
    )
    .expect("stage a");
    let paths: Vec<String> = list_worktree_captured_paths(
        &git_context(),
        repo.as_path(),
        &indexes,
        TransactionMode::ExplicitPath,
        false,
        base.as_str(),
    )
    .expect("list");
    assert_eq!(paths, vec![String::from("a.txt")]);
    remove(root.as_path());
}

/// Index commits that do not stage worktree content read no worktree paths at all.
#[test]
fn index_commits_without_staging_read_no_worktree_paths() {
    let root: PathBuf = fixture("capture-order-no-stage");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: private_index(repo.as_path(), "commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    let paths: Vec<String> = list_worktree_captured_paths(
        &git_context(),
        repo.as_path(),
        &indexes,
        TransactionMode::Index,
        false,
        "HEAD",
    )
    .expect("list");
    assert!(paths.is_empty());
    remove(root.as_path());
}

/// Staging captures compare the captured index with the staged one, sorted by raw bytes:
///  `B.txt` precedes `a.txt`, which a case-insensitive sort would reverse.
#[test]
fn staging_lists_changed_paths_in_byte_order() {
    let root: PathBuf = fixture("capture-order-stage-order");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("B.txt"), "B\n").expect("B");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: private_index(repo.as_path(), "commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    run_git_checked(
        &git_context(),
        &indexed_request(repo.as_path(), &["add", "--all"], Some(indexes.commit_index.as_path()), None),
    )
    .expect("stage");
    let paths: Vec<String> = list_worktree_captured_paths(
        &git_context(),
        repo.as_path(),
        &indexes,
        TransactionMode::Index,
        true,
        "HEAD",
    )
    .expect("list");
    assert_eq!(paths, vec![String::from("B.txt"), String::from("a.txt")]);
    remove(root.as_path());
}

/// Git's raw path bytes are read as Latin-1 text, so UTF-8 `é` becomes two code points,
///  as the incumbent's `decodeLatin1` reads it.
#[test]
fn paths_decode_as_latin1_bytes() {
    let root: PathBuf = fixture("capture-order-latin1");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("é.txt"), "x\n").expect("é");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: private_index(repo.as_path(), "commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    run_git_checked(
        &git_context(),
        &indexed_request(repo.as_path(), &["add", "--all"], Some(indexes.commit_index.as_path()), None),
    )
    .expect("stage");
    let paths: Vec<String> = list_worktree_captured_paths(
        &git_context(),
        repo.as_path(),
        &indexes,
        TransactionMode::Index,
        true,
        "HEAD",
    )
    .expect("list");
    assert_eq!(paths, vec![String::from("\u{c3}\u{a9}.txt")]);
    remove(root.as_path());
}

/// NUL records drop their empty pieces, and a record without a tab cannot come from Git.
#[test]
fn nul_records_drop_empty_pieces() {
    let records: Vec<&[u8]> = nul_records(b"a\0\0b\0");
    assert_eq!(records, vec![&b"a"[..], &b"b"[..]]);
    assert!(nul_records(b"").is_empty());
}

/// Capture of content writes the record with the stamp, the listing and the pre-base sequence.
#[test]
fn capture_commit_content_journals_the_stamp_and_the_listing() {
    let root: PathBuf = fixture("capture-order-content");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let directory: PathBuf = root.join("transaction");
    std::fs::create_dir(&directory).expect("transaction directory");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: private_index(repo.as_path(), "commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    let record: CapturedRecord = capture_commit_content(
        &git_context(),
        repo.as_path(),
        repo.join(".git").as_path(),
        directory.as_path(),
        &indexes,
        TransactionMode::Index,
        false,
        "HEAD",
        1,
        nothing,
    )
    .expect("capture");
    assert_eq!(record.sequence, 1);
    assert_eq!(record.next_sequence_before_base, 1);
    assert!(record.worktree_paths.is_empty());
    assert_eq!(read_captured_record(directory.as_path()).expect("read"), Some(record));
    remove(root.as_path());
}
