//! External file updates are prepared in disposable fixtures and applied to current reading state.

/// Disposable fixture directories,
///  in memory when the system has one.
mod memory_fixture;

/// Application consumers share the same read/diff and bounded worker APIs.
use ide_app::{
    document::{Document, ReadingPosition},
    file_reload::read_reload,
    reload_worker::{ReloadReply, ReloadRequest, ReloadWorker},
};
/// Bound worker waits instead of hanging an unattended regression run.
use std::time::{Duration, Instant};

/// Longest wait for one reply:
///  a hang detector,
///  not a latency budget,
///  and the same bound as
/// `OPEN_PATIENCE` in `tests/file_open.rs` and the Language tests' `PATIENCE`.
///  Before its first
/// reply the worker builds the syntax engine,
///  processor work that takes far longer when the
/// machine is busy:
///  the reload worker was seen waiting 6 s for a processor while it ran for
/// 0.2 s,
///  and with a quarter processor 1 of 300 suite runs passed the 3 s bound this replaces.
/// No product bound exists for a reload.
const REPLY_PATIENCE: Duration = Duration::from_secs(20);

/// Wait for a single test reply while retaining an explicit failure deadline.
fn reply(worker: &mut ReloadWorker) -> ReloadReply {
    let start = Instant::now();
    loop {
        // What: expect fails this test on disconnect; Some extracts a completed response.
        // Why: A stopped worker must not look like an indefinitely pending operation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const response = worker.tryTake(); if (response !== undefined) return response;
        // ```
        if let Some(response) = worker.try_take().expect("poll source worker") {
            return response;
        }
        assert!(
            start.elapsed() < REPLY_PATIENCE,
            "source worker did not reply within {REPLY_PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// A byte-identical read produces no new revision or diff to apply.
#[test]
fn unchanged_disk_source_has_no_reload() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.txt");
    std::fs::write(&path, "a\r\n猫").expect("write fixture source");
    let document = Document::new("a\r\n猫");
    assert!(
        read_reload(&document, &path)
            .expect("read fixture")
            .is_none()
    );
    assert_eq!(document.revision(), 0);
}

/// An external replacement follows the selected region rather than a later literal match.
#[test]
fn disk_replacement_preserves_corresponding_selection() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.txt");
    std::fs::write(&path, "I was a big cat, but now I am a human!")
        .expect("external fixture update");
    let mut document = Document::new("I am a big cat");
    document.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let reload = read_reload(&document, &path)
        .expect("read updated source")
        .expect("changed source");
    assert!(document.apply_reload(reload));
    assert_eq!(document.selected_text(), "was a");
}

/// Read errors retain the caller's source instead of installing empty or lossy text.
#[test]
fn absent_non_regular_and_non_utf8_sources_fail_without_mutating_document() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.txt");
    let document = Document::new("retained source");
    assert!(read_reload(&document, &path).is_err());
    assert!(read_reload(&document, fixture.path()).is_err());
    std::fs::write(&path, [0xff, 0xfe]).expect("invalid UTF-8 fixture");
    assert!(read_reload(&document, &path).is_err());
    assert_eq!(document.text().to_string(), "retained source");
}

/// A second request cannot accumulate while the previous response remains unread.
#[test]
fn worker_bounds_requests_and_maps_the_latest_caret() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.txt");
    std::fs::write(&path, "I was a big cat.").expect("external fixture update");
    let mut document = Document::new("I am a big cat.");
    let mut worker = ReloadWorker::new().expect("start source worker");
    assert!(worker.try_take().expect("idle worker").is_none());
    // Clone retains the base rope/revision independently of subsequent UI movement.
    assert!(
        worker
            .request(ReloadRequest {
                path: path.clone(),
                snapshot: document.clone(),
                generation: 41,
                highlight_unchanged: false,
                require_quiet: None
            })
            .expect("first request")
    );
    assert!(
        !worker
            .request(ReloadRequest {
                path,
                snapshot: document.clone(),
                generation: 42,
                highlight_unchanged: false,
                require_quiet: None
            })
            .expect("bounded second request")
    );
    document.select(ReadingPosition {
        anchor: 9,
        head: 9,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let response = reply(&mut worker);
    assert_eq!(response.generation, 41);
    assert!(
        response
            .syntax
            .expect("changed source classification")
            .result
            .expect("plain source language")
            .is_none()
    );
    let reload = response
        .result
        .expect("background source read")
        .expect("changed source");
    assert!(document.apply_reload(reload));
    assert_eq!(document.position().head, 10);
}

/// Failed reads do not terminate the worker or prevent a later successful poll.
#[test]
fn worker_recovers_after_source_reappears() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.txt");
    let document = Document::new("same source");
    let mut worker = ReloadWorker::new().expect("start source worker");
    assert!(
        worker
            .request(ReloadRequest {
                path: path.clone(),
                snapshot: document.clone(),
                generation: 7,
                highlight_unchanged: false,
                require_quiet: None
            })
            .expect("missing-file request")
    );
    let failed = reply(&mut worker);
    assert!(failed.result.is_err());
    assert!(failed.syntax.is_none());
    std::fs::write(&path, "same source").expect("restore external fixture");
    assert!(
        worker
            .request(ReloadRequest {
                path,
                snapshot: document,
                generation: 7,
                highlight_unchanged: false,
                require_quiet: None
            })
            .expect("recovered-file request")
    );
    assert!(reply(&mut worker).result.expect("recovered read").is_none());
}

/// Initial and changed syntax describe the exact source revision;
///  unchanged accepted syntax is not reparsed.
#[test]
fn worker_classifies_initial_and_changed_source_revisions() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.rs");
    std::fs::write(&path, "fn main() {}\n").expect("initial Rust source");
    let mut document = Document::new("fn main() {}\n");
    let mut worker = ReloadWorker::new().expect("start source worker");
    assert!(
        worker
            .request(ReloadRequest {
                path: path.clone(),
                snapshot: document.clone(),
                generation: 1,
                highlight_unchanged: true,
                require_quiet: None,
            })
            .expect("initial syntax request")
    );
    let initial = reply(&mut worker);
    assert!(initial.result.expect("unchanged initial read").is_none());
    let syntax = initial.syntax.expect("initial syntax result");
    assert_eq!(syntax.revision, 0);
    assert!(
        !syntax
            .result
            .expect("initial Rust parsing")
            .expect("known Rust language")
            .is_empty()
    );
    assert!(
        worker
            .request(ReloadRequest {
                path: path.clone(),
                snapshot: document.clone(),
                generation: 1,
                highlight_unchanged: false,
                require_quiet: None,
            })
            .expect("unchanged poll")
    );
    assert!(reply(&mut worker).syntax.is_none());
    std::fs::write(&path, "fn next() { let cat = 42; }\n").expect("external Rust update");
    assert!(
        worker
            .request(ReloadRequest {
                path,
                snapshot: document.clone(),
                generation: 1,
                highlight_unchanged: false,
                require_quiet: None,
            })
            .expect("changed source poll")
    );
    let changed = reply(&mut worker);
    assert!(
        document.apply_reload(
            changed
                .result
                .expect("read changed source")
                .expect("prepared reload")
        )
    );
    let changed_syntax = changed.syntax.expect("changed syntax result");
    assert_eq!(changed_syntax.revision, document.revision());
    let styles = changed_syntax
        .result
        .expect("updated Rust parsing")
        .expect("known Rust language");
    let mut next_painted = false;
    for span in styles.iter() {
        if span.style == 6 && document.text().slice(span.start..span.end) == "next" {
            next_painted = true;
        }
    }
    assert!(next_painted);
}

/// Closing a worker with an unread job/reply joins it without a channel deadlock.
#[test]
fn worker_shutdown_does_not_require_consuming_the_last_reply() {
    let fixture = memory_fixture::directory("ide-file-reload-");
    let path = fixture.path().join("source.txt");
    std::fs::write(&path, "changed source").expect("write fixture");
    let mut worker = ReloadWorker::new().expect("start source worker");
    assert!(
        worker
            .request(ReloadRequest {
                path,
                snapshot: Document::new("old source"),
                generation: 1,
                highlight_unchanged: false,
                require_quiet: None
            })
            .expect("last request")
    );
    drop(worker);
}
