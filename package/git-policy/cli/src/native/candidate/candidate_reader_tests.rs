//! What:
//!  Process controls for the object reader against real Git 2.56.0 and failing stand-ins.
//! Why:
//!  The reader must return exact stored bytes,
//!  read each blob once,
//!  stay usable after
//!      an ordinary "missing" answer,
//!  refuse everything after a broken reply,
//!  and never
//!      leave a process behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const reader = startObjectReader(git, ['-C', repo], []); expect(await reader.blob(oid)).toEqual(bytes);
//! ```
#![cfg(unix)]

/// Import the reader under test and shared fixtures.
use super::{ObjectReader, start_object_reader};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{ObjectId, parse_object_id};
use crate::git_metadata::strip_git_line;
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The global prefix selecting one repository:
///  `-C <directory>`.
fn prefix(directory: &Path) -> Vec<OsString> {
    return vec![OsString::from("-C"), directory.as_os_str().to_os_string()];
}

/// Start a reader over real Git in a fixture repository.
fn reader(directory: &Path) -> ObjectReader {
    return start_object_reader(Path::new(REAL_GIT), prefix(directory).as_slice(), &[])
        .expect("reader starts");
}

/// Store exact bytes as a blob by writing a file and hashing it,
///  and return its object name.
fn store_blob(repository: &Path, bytes: &[u8]) -> ObjectId {
    let file: PathBuf = repository.join("blob-source");
    std::fs::write(&file, bytes).expect("blob source");
    let output: std::process::Output = git(repository, &["hash-object", "-w", "blob-source"]);
    std::fs::remove_file(&file).expect("remove blob source");
    return parse_object_id(strip_git_line(output.stdout.as_slice())).expect("object name");
}

/// The object name a revision expression resolves to.
fn rev_parse(repository: &Path, revision: &str) -> ObjectId {
    let output: std::process::Output = git(repository, &["rev-parse", "--verify", revision]);
    return parse_object_id(strip_git_line(output.stdout.as_slice())).expect("object name");
}

/// Blob bytes come back exactly:
///  text,
///  binary with NUL and non-UTF-8 bytes,
///  an empty blob,
///  and a large one.
#[test]
fn blobs_are_returned_byte_for_byte() {
    let root: PathBuf = fixture("reader-bytes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut large: Vec<u8> = Vec::new();
    for index in 0..300_000_u32 {
        large.extend_from_slice(&index.to_le_bytes());
    }
    let contents: Vec<Vec<u8>> = vec![
        b"plain text\n".to_vec(),
        b"no final newline".to_vec(),
        b"\0\xff\xfe\n\n binary \r\n".to_vec(),
        Vec::new(),
        large,
    ];
    let mut names: Vec<ObjectId> = Vec::new();
    for content in &contents {
        names.push(store_blob(repo.as_path(), content.as_slice()));
    }
    let mut subject: ObjectReader = reader(repo.as_path());
    for (name, content) in names.iter().zip(contents.iter()) {
        let bytes: Rc<[u8]> = subject.blob(name).expect("blob");
        assert_eq!(&bytes[..], content.as_slice());
    }
    assert_eq!(subject.blobs.len(), contents.len());
    drop(subject);
    remove(root.as_path());
}

/// A blob already read is served from memory:
///  it survives the reader being closed,
///  and an unread one does not.
#[test]
fn each_blob_is_read_once() {
    let root: PathBuf = fixture("reader-memo");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: ObjectId = store_blob(repo.as_path(), b"first\n");
    let second: ObjectId = store_blob(repo.as_path(), b"second\n");
    let mut subject: ObjectReader = reader(repo.as_path());
    let read: Rc<[u8]> = subject.blob(&first).expect("first read");
    // Closing the pipes makes any further request fail, so a success below cannot have asked Git.
    subject.pipes = None;
    let again: Rc<[u8]> = subject.blob(&first).expect("remembered read");
    assert_eq!(&again[..], b"first\n");
    // The same storage is shared, not copied.
    assert!(Rc::ptr_eq(&read, &again));
    let error: CandidateError = subject.blob(&second).expect_err("closed reader");
    assert_eq!(error.failure, CandidateFailure::ReaderEnded);
    assert!(
        error
            .message
            .contains("was closed after an earlier failure"),
        "{error}"
    );
    drop(subject);
    remove(root.as_path());
}

/// A missing object and a non-blob object are failures that leave the reader in step for the next request.
#[test]
fn missing_and_non_blob_objects_are_failures_the_reader_survives() {
    let root: PathBuf = fixture("reader-missing");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let present: ObjectId = store_blob(repo.as_path(), b"present\n");
    let absent: ObjectId = parse_object_id(&[b'1'; 40]).expect("name");
    let commit: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let tree: ObjectId = rev_parse(repo.as_path(), "HEAD^{tree}");
    let mut subject: ObjectReader = reader(repo.as_path());
    let missing: CandidateError = subject.blob(&absent).expect_err("missing object");
    assert_eq!(missing.failure, CandidateFailure::ObjectMissing);
    assert!(missing.message.contains(absent.as_str()), "{missing}");
    for object in [&commit, &tree] {
        let wrong_kind: CandidateError = subject.blob(object).expect_err("not a blob");
        assert_eq!(wrong_kind.failure, CandidateFailure::ObjectKindUnexpected);
        assert!(wrong_kind.message.contains(object.as_str()), "{wrong_kind}");
    }
    // Neither failure was remembered as content, and the stream is still in step.
    assert!(subject.blobs.is_empty());
    assert_eq!(
        &subject.blob(&present).expect("still usable")[..],
        b"present\n"
    );
    drop(subject);
    remove(root.as_path());
}

/// `HEAD` resolves to the current commit,
///  and to nothing in a repository without commits.
#[test]
fn head_commit_is_resolved_or_absent() {
    let root: PathBuf = fixture("reader-head");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut subject: ObjectReader = reader(repo.as_path());
    assert_eq!(
        subject.head_commit().expect("born"),
        Some(rev_parse(repo.as_path(), "HEAD"))
    );
    drop(subject);
    let unborn: PathBuf = root.join("unborn");
    std::fs::create_dir(&unborn).expect("unborn directory");
    git(
        unborn.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    let mut unborn_subject: ObjectReader = reader(unborn.as_path());
    assert_eq!(unborn_subject.head_commit().expect("unborn"), None);
    // The missing answer left the reader usable.
    let blob: ObjectId = store_blob(unborn.as_path(), b"later\n");
    assert_eq!(
        &unborn_subject.blob(&blob).expect("later blob")[..],
        b"later\n"
    );
    drop(unborn_subject);
    remove(root.as_path());
}

/// Objects written after the reader started are still found,
///  as a fix that stages new content requires.
#[test]
fn objects_written_after_start_are_found() {
    let root: PathBuf = fixture("reader-late");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut subject: ObjectReader = reader(repo.as_path());
    assert!(subject.head_commit().expect("head").is_some());
    let late: ObjectId = store_blob(repo.as_path(), b"written later\n");
    assert_eq!(
        &subject.blob(&late).expect("late blob")[..],
        b"written later\n"
    );
    drop(subject);
    remove(root.as_path());
}

/// Dropping the reader ends and collects its process:
///  no live or zombie entry remains.
#[test]
fn dropping_the_reader_collects_its_process() {
    let root: PathBuf = fixture("reader-drop");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut subject: ObjectReader = reader(repo.as_path());
    assert!(subject.head_commit().expect("head").is_some());
    let entry: PathBuf = PathBuf::from(format!("/proc/{}", subject.child.id()));
    assert!(entry.exists(), "the reader process is running");
    drop(subject);
    // A process that ended but was never waited for keeps its entry as a zombie.
    assert!(!entry.exists(), "the reader process was collected");
    remove(root.as_path());
}

/// The environment overlay reaches the reader:
///  a private object directory hides the repository's own objects.
#[test]
fn overlay_selects_the_object_directory() {
    let root: PathBuf = fixture("reader-overlay");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let blob: ObjectId = store_blob(repo.as_path(), b"in the real store\n");
    let private: PathBuf = root.join("private-objects");
    std::fs::create_dir(&private).expect("private object directory");
    let overlay: Vec<(OsString, OsString)> = vec![(
        OsString::from("GIT_OBJECT_DIRECTORY"),
        private.clone().into_os_string(),
    )];
    let mut hidden: ObjectReader = start_object_reader(
        Path::new(REAL_GIT),
        prefix(repo.as_path()).as_slice(),
        overlay.as_slice(),
    )
    .expect("reader starts");
    assert_eq!(
        hidden.blob(&blob).expect_err("hidden object").failure,
        CandidateFailure::ObjectMissing
    );
    drop(hidden);
    let mut visible: ObjectReader = reader(repo.as_path());
    assert_eq!(
        &visible.blob(&blob).expect("visible object")[..],
        b"in the real store\n"
    );
    drop(visible);
    remove(root.as_path());
}
