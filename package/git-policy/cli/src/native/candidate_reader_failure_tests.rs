//! What:
//!  Failure controls for the object reader,
//!  using stand-in programs that misbehave on purpose.
//! Why:
//!  A reply that is cut short,
//!  names another object,
//!  or never arrives must be a typed
//!      failure that closes the reader,
//!  so that no later request can be answered from a
//!      stream that is out of step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await expect(reader.blob(oid)).rejects.toMatchObject({ failure: 'reply-truncated' });
//! ```
#![cfg(unix)]

/// Import the reader under test and shared fixtures.
use super::{ObjectReader, start_object_reader, start_object_reader_with};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{ObjectId, parse_object_id};
use crate::child_watch::WatchTerms;
use crate::test_support::{SharedSink, executable, fixture, remove};
use crate::wait_progress::Cadence;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A stand-in `git` that waits for one request line,
///  answers with fixed bytes,
///  and exits.
/// Waiting first keeps the request write from racing the stand-in's exit.
fn stand_in(root: &Path, name: &str, reply: &str) -> PathBuf {
    let path: PathBuf = root.join(name);
    executable(
        path.as_path(),
        format!("#!/bin/sh\nread -r request\nprintf '{reply}'\nexit 0\n").as_bytes(),
    );
    return path;
}

/// Wait,
///  within a bound,
///  until a child has exited and is waiting to be collected.
fn wait_until_exited(process: u32) {
    for _attempt in 0..500 {
        let stat: String =
            std::fs::read_to_string(format!("/proc/{process}/stat")).expect("process entry");
        // The state letter follows the parenthesized program name; `Z` is an exited, uncollected child.
        if stat.contains(") Z ") {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the stand-in did not exit within the bound");
}

/// A process that reads a request and never answers is announced while the reader waits, stopped at the bound, and the
///  reader reports the bound and closes, so no later request waits on a dead stream.
#[test]
fn a_process_that_never_answers_is_announced_then_stopped_at_the_bound() {
    let root: PathBuf = fixture("reader-stalls");
    let program: PathBuf = root.join("git");
    executable(program.as_path(), b"#!/bin/sh\nread -r request\nexec sleep 30\n");
    let sink: SharedSink = SharedSink::default();
    let mut subject: ObjectReader = start_object_reader_with(
        program.as_path(),
        &[],
        &[],
        WatchTerms {
            limit: Duration::from_secs(1),
            cadence: Cadence { grace: Duration::from_millis(200), interval: Duration::from_millis(200) },
            notices: Box::new(sink.clone()),
        },
    )
    .expect("stand-in starts");
    let started: Instant = Instant::now();
    let error: CandidateError = subject.head_commit().expect_err("never answers");
    assert!(started.elapsed() < Duration::from_secs(20), "{:?}", started.elapsed());
    assert_eq!(error.failure, CandidateFailure::ReaderEnded);
    assert!(
        error.message.contains("git cat-file --batch did not finish within 1 seconds and was stopped"),
        "{error}"
    );
    assert!(error.message.contains("CLI_GIT_CHILD_WAIT_SECONDS"), "{error}");
    let text: String = sink.text();
    assert!(text.lines().count() >= 3, "{text}");
    assert!(text.starts_with("cli-git: still waiting for git cat-file --batch (process "), "{text}");
    let later: CandidateError = subject.head_commit().expect_err("closed");
    assert_eq!(later.failure, CandidateFailure::ReaderEnded);
    drop(subject);
    remove(root.as_path());
}

/// A `HEAD` that resolves to something other than a commit is refused instead of used as a baseline.
#[test]
fn head_that_is_not_a_commit_is_refused() {
    let root: PathBuf = fixture("reader-head-kind");
    let name: String = "a".repeat(40);
    let program: PathBuf = stand_in(
        root.as_path(),
        "git",
        format!("{name} tree 0\\n\\n").as_str(),
    );
    let mut subject: ObjectReader =
        start_object_reader(program.as_path(), &[], &[]).expect("stand-in starts");
    let error: CandidateError = subject.head_commit().expect_err("tree as HEAD");
    assert_eq!(error.failure, CandidateFailure::ObjectKindUnexpected);
    assert!(
        error.message.contains("HEAD does not name a commit"),
        "{error}"
    );
    drop(subject);
    remove(root.as_path());
}

/// A process that dies in the middle of a reply produces a typed failure,
///  and the reader refuses every later request.
#[test]
fn reply_cut_short_by_a_dying_process_closes_the_reader() {
    let root: PathBuf = fixture("reader-dies");
    let name: String = "a".repeat(40);
    let object: ObjectId = parse_object_id(name.as_bytes()).expect("name");
    let program: PathBuf = stand_in(
        root.as_path(),
        "git",
        format!("{name} blob 100\\nonly part").as_str(),
    );
    let mut subject: ObjectReader =
        start_object_reader(program.as_path(), &[], &[]).expect("stand-in starts");
    let error: CandidateError = subject.blob(&object).expect_err("truncated reply");
    assert_eq!(error.failure, CandidateFailure::ReplyTruncated);
    assert!(subject.pipes.is_none());
    assert!(subject.blobs.is_empty());
    assert_eq!(
        subject.blob(&object).expect_err("closed reader").failure,
        CandidateFailure::ReaderEnded
    );
    assert_eq!(
        subject.head_commit().expect_err("closed reader").failure,
        CandidateFailure::ReaderEnded
    );
    drop(subject);
    remove(root.as_path());
}

/// A reply naming another object is refused and closes the reader,
///  so its bytes are never attributed.
#[test]
fn reply_for_another_object_closes_the_reader() {
    let root: PathBuf = fixture("reader-mismatch");
    let requested: ObjectId = parse_object_id(&[b'a'; 40]).expect("name");
    let other: String = "b".repeat(40);
    let program: PathBuf = stand_in(
        root.as_path(),
        "git",
        format!("{other} blob 5\\nwrong\\n").as_str(),
    );
    let mut subject: ObjectReader =
        start_object_reader(program.as_path(), &[], &[]).expect("stand-in starts");
    assert_eq!(
        subject
            .blob(&requested)
            .expect_err("mismatched reply")
            .failure,
        CandidateFailure::ReplyMismatched
    );
    assert!(subject.pipes.is_none());
    assert!(subject.blobs.is_empty());
    drop(subject);
    remove(root.as_path());
}

/// A process that exits without answering is a reader failure.
#[test]
fn process_that_answers_nothing_is_a_reader_failure() {
    let root: PathBuf = fixture("reader-silent");
    let program: PathBuf = stand_in(root.as_path(), "git", "");
    let mut subject: ObjectReader =
        start_object_reader(program.as_path(), &[], &[]).expect("stand-in starts");
    let error: CandidateError = subject
        .blob(&parse_object_id(&[b'a'; 40]).expect("name"))
        .expect_err("no reply");
    assert_eq!(error.failure, CandidateFailure::ReaderEnded);
    drop(subject);
    remove(root.as_path());
}

/// A request written to a process that already exited is a reader failure that closes the reader.
#[test]
fn request_to_an_exited_process_is_a_reader_failure() {
    let root: PathBuf = fixture("reader-exited");
    let program: PathBuf = root.join("git");
    executable(program.as_path(), b"#!/bin/sh\nexit 0\n");
    let mut subject: ObjectReader =
        start_object_reader(program.as_path(), &[], &[]).expect("stand-in starts");
    wait_until_exited(subject.child.id());
    let error: CandidateError = subject
        .blob(&parse_object_id(&[b'a'; 40]).expect("name"))
        .expect_err("exited process");
    // Which step notices is not fixed: the write fails once the pipe has no reader, but a child
    // another test thread is starting can hold an inherited copy of the pipe for a moment, and
    // then the write succeeds and the read finds the stream ended. Both are the same failure.
    assert_eq!(error.failure, CandidateFailure::ReaderEnded);
    assert!(
        error.message.contains("stopped accepting requests")
            || error.message.contains("ended before answering a request"),
        "{error}"
    );
    assert!(subject.pipes.is_none());
    drop(subject);
    remove(root.as_path());
}

/// A program that cannot be started is its own failure,
///  naming the operation.
#[test]
fn unstartable_git_is_reported() {
    let error: CandidateError =
        match start_object_reader(Path::new("/nonexistent-directory/git"), &[], &[]) {
            Ok(_) => panic!("a missing program must not start"),
            Err(failure) => failure,
        };
    assert_eq!(error.failure, CandidateFailure::GitNotStarted);
    assert!(
        error
            .message
            .starts_with("cli-git could not start git cat-file --batch to read Git objects: "),
        "{error}"
    );
}
