//! Source-opening requests resolve in private project fixtures and retain only the latest user intent.

/// Disposable fixture directories,
///  in memory when the system has one.
mod memory_fixture;

/// Current read errors propagate while stale failures disappear from the consumer boundary.
use anyhow::Result;
/// Consumer-facing opener produces a document only after successful read and revision validation.
use ide_app::{
    file_open::{FileOpener, OpenedFile},
    workspace::Workspace,
};
/// Files and links are disposable;
///  every worker wait has a deadline.
use std::{
    fs,
    os::unix::fs::symlink,
    path::PathBuf,
    time::{Duration, Instant},
};

/// Longest wait for one open:
///  a hang detector,
///  not a latency budget,
///  and the same bound the
/// Language integration tests use for any expected state (`PATIENCE` in `tests/language/support.rs`).
/// The first open of every opener builds the syntax engine and compiles the language's queries,
/// once.
///  Measured in a debug build with fixtures in memory,
///  on a loaded machine,
///  that took 0.8 s
/// at the median and up to 4.2 s with two processors,
///  and 1.9 s at the median and up to 4.2 s
/// with half a processor;
///  later opens take milliseconds.
///  No product bound exists for opening a
/// file.
const OPEN_PATIENCE: Duration = Duration::from_secs(20);

/// Drain a pending open or cancellation without relying on filesystem-read timing.
fn finish(opener: &mut FileOpener) -> Result<Option<OpenedFile>> {
    let start = Instant::now();
    while opener.has_pending() {
        // Some means a ready latest open; ? propagates only current failures.
        if let Some(opened) = opener.poll()? {
            return Ok(Some(opened));
        }
        assert!(
            start.elapsed() < OPEN_PATIENCE,
            "file opener did not finish within {OPEN_PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    return Ok(None);
}

/// Real source classification and an empty initial reading state accompany the resolved target.
#[test]
fn opening_source_starts_at_zero_with_matching_syntax() {
    let fixture = memory_fixture::directory("ide-file-open-");
    fs::write(
        fixture.path().join("source.rs"),
        "fn main() { let 猫 = 1; }\n",
    )
    .expect("source fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let expected = workspace.root().join("source.rs");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    assert!(!opener.has_pending());
    assert!(opener.poll().expect("idle poll").is_none());
    opener
        .request(PathBuf::from("source.rs"))
        .expect("relative file request");
    let opened = finish(&mut opener)
        .expect("source read")
        .expect("opened source");
    assert_eq!(opened.path, expected);
    assert_eq!(
        opened.document.text().to_string(),
        "fn main() { let 猫 = 1; }\n"
    );
    assert_eq!(opened.document.position().anchor, 0);
    assert_eq!(opened.document.position().head, 0);
    assert_eq!(opened.document.position().viewport, 0);
    let syntax = opened.syntax.expect("initial classification");
    assert_eq!(syntax.revision, opened.document.revision());
    assert!(
        !syntax
            .result
            .expect("Rust parser")
            .expect("recognized Rust")
            .is_empty()
    );
    assert!(!opener.has_pending());
}

/// Newer waiting targets replace intermediate choices while an earlier read is executing or unread.
#[test]
fn latest_open_wins_without_a_queue_of_intermediate_files() {
    let fixture = memory_fixture::directory("ide-file-open-");
    for name in ["first.txt", "second.txt", "last.txt"] {
        fs::write(fixture.path().join(name), name).expect("source fixture");
    }
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    opener
        .request(PathBuf::from("first.txt"))
        .expect("first request");
    assert!(opener.poll().expect("submit first read").is_none());
    opener
        .request(PathBuf::from("second.txt"))
        .expect("intermediate request");
    opener
        .request(PathBuf::from("last.txt"))
        .expect("latest request");
    let opened = finish(&mut opener)
        .expect("latest read")
        .expect("latest source");
    assert_eq!(opened.document.text().to_string(), "last.txt");
    assert!(!opener.has_pending());
}

/// Choosing the already displayed file can invalidate queued or running opens without installing their result.
#[test]
fn cancellation_discards_both_waiting_and_running_opens() {
    let fixture = memory_fixture::directory("ide-file-open-");
    fs::write(fixture.path().join("source.txt"), "source").expect("source fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    opener
        .request(PathBuf::from("source.txt"))
        .expect("waiting request");
    opener.cancel().expect("cancel waiting request");
    assert!(!opener.has_pending());
    opener
        .request(PathBuf::from("source.txt"))
        .expect("running request");
    opener.poll().expect("submit read");
    opener.cancel().expect("cancel running request");
    assert!(finish(&mut opener).expect("drain cancelled read").is_none());
    assert!(!opener.has_pending());
}

/// A superseded read failure cannot replace the result or diagnostic of the latest successful open.
#[test]
fn stale_failure_is_discarded_and_current_failure_can_be_retried() {
    let fixture = memory_fixture::directory("ide-file-open-");
    fs::write(fixture.path().join("source.txt"), "source").expect("source fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    opener
        .request(PathBuf::from("missing.txt"))
        .expect("missing request");
    opener.poll().expect("submit missing read");
    opener
        .request(PathBuf::from("source.txt"))
        .expect("replacement request");
    assert!(
        finish(&mut opener)
            .expect("stale error suppressed")
            .is_some()
    );
    opener
        .request(PathBuf::from("missing.txt"))
        .expect("current missing request");
    assert!(finish(&mut opener).is_err());
    assert!(!opener.has_pending());
    fs::write(fixture.path().join("missing.txt"), "restored").expect("external restoration");
    opener
        .request(PathBuf::from("missing.txt"))
        .expect("retry request");
    let restored = finish(&mut opener)
        .expect("retry read")
        .expect("restored source");
    assert_eq!(restored.document.text().to_string(), "restored");
}

/// Empty sources and contained symlink aliases resolve to the same canonical file identity.
#[test]
fn empty_file_and_inside_alias_are_valid_targets() {
    let fixture = memory_fixture::directory("ide-file-open-");
    fs::write(fixture.path().join("empty.txt"), "").expect("empty source");
    symlink(
        fixture.path().join("empty.txt"),
        fixture.path().join("alias.txt"),
    )
    .expect("contained alias");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let expected = workspace.root().join("empty.txt");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    opener
        .request(PathBuf::from("alias.txt"))
        .expect("alias request");
    let opened = finish(&mut opener)
        .expect("empty source read")
        .expect("empty source opened");
    assert_eq!(opened.path, expected);
    assert_eq!(opened.document.text().len_chars(), 0);
    assert_eq!(opened.document.revision(), 0);
    assert_eq!(
        opened
            .syntax
            .expect("initial plain-text classification")
            .revision,
        0
    );
}

/// Project opens reject outside links,
///  directories,
///  and non-UTF-8 contents on the worker thread.
#[test]
fn project_boundary_and_source_kind_failures_do_not_produce_documents() {
    let fixture = memory_fixture::directory("ide-file-open-");
    let project = fixture.path().join("project");
    fs::create_dir(&project).expect("project directory");
    fs::write(fixture.path().join("outside.txt"), "outside").expect("outside fixture");
    symlink(
        fixture.path().join("outside.txt"),
        project.join("escape.txt"),
    )
    .expect("outside alias");
    fs::write(project.join("binary.txt"), [0xff]).expect("non-UTF-8 fixture");
    let workspace = Workspace::new(&project).expect("workspace");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    for target in ["escape.txt", "../outside.txt", ".", "binary.txt"] {
        opener
            .request(PathBuf::from(target))
            .expect("request validation stays on worker");
        assert!(
            finish(&mut opener).is_err(),
            "accepted invalid target {target}"
        );
        assert!(!opener.has_pending());
    }
}

/// An outside-project language target opens read-only and is marked;
///  a later project open replaces it.
#[test]
fn outside_target_opens_marked_and_a_later_project_open_wins() {
    let fixture = memory_fixture::directory("ide-file-open-");
    let project = fixture.path().join("project");
    fs::create_dir(&project).expect("project directory");
    let outside = fixture.path().join("library.rs");
    fs::write(&outside, "pub fn library() {}\n").expect("outside fixture");
    fs::write(project.join("main.rs"), "fn main() {}\n").expect("project fixture");
    let workspace = Workspace::new(&project).expect("workspace");
    let mut opener = FileOpener::new(workspace).expect("file opener");
    let canonical = outside.canonicalize().expect("canonical outside path");
    opener
        .request_outside(canonical.clone())
        .expect("outside request");
    let opened = finish(&mut opener)
        .expect("outside read succeeds")
        .expect("outside document");
    assert!(opened.outside_project);
    assert_eq!(opened.path, canonical);
    assert_eq!(opened.document.text().to_string(), "pub fn library() {}\n");
    opener
        .request_outside(canonical)
        .expect("second outside request");
    opener
        .request(PathBuf::from("main.rs"))
        .expect("project request replaces the outside one");
    let replaced = finish(&mut opener)
        .expect("project read succeeds")
        .expect("project document");
    assert!(!replaced.outside_project);
    assert_eq!(replaced.document.text().to_string(), "fn main() {}\n");
}
