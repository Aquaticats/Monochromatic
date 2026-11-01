//! A real worker thread:
//!  newest request wins,
//!  replies keep their tags,
//!  and shutdown joins cleanly.

/// Reload correspondence supplies real content revisions for retagged requests.
use ide_app::document::Document;
/// Diagnostics name the documented query bound.
use ide_app::find::MAX_FIND_QUERY_CHARS;
/// The same handle the native find bar owns.
use ide_app::find_worker::{FindIdentity, FindReply, FindRequest, FindWorker};
/// Waits are bounded so a lost reply fails the test instead of hanging it.
use std::time::{Duration, Instant};

/// Build an owned request;
///  the rope clone shares the document's text chunks.
fn request(document: &Document, file: u64, query_generation: u64, query: &str) -> FindRequest {
    // What: `clone` on a rope copies a small handle; `to_string` copies the query into owned storage.
    // Why: A request must own everything it carries to another thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { identity, source: document.text, query };
    // ```
    return FindRequest {
        identity: FindIdentity {
            file,
            revision: document.revision(),
            query: query_generation,
        },
        source: document.text().clone(),
        query: query.to_string(),
    };
}

/// Poll until the wanted reply arrives;
///  stale replies are consumed inside `poll`.
fn reply(worker: &mut FindWorker) -> FindReply {
    let start = Instant::now();
    loop {
        // What: `expect` extracts the poll result; `if let Some(found)` runs only when a reply arrived.
        // Why: Polling is the native boundary; the test must not read the channel another way.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const found = worker.poll(); if (found !== undefined) return found;
        // ```
        if let Some(found) = worker.poll().expect("find worker remains available") {
            return found;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "in-file find did not complete"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Requests issued while a job runs replace each other;
///  only the newest reply is ever returned.
#[test]
fn newest_request_wins_over_running_and_waiting_requests() {
    let document = Document::new("alpha beta gamma beta");
    let mut worker = FindWorker::new().expect("find worker");
    assert!(worker.poll().expect("idle poll").is_none());
    assert!(!worker.is_pending());
    worker
        .request(request(&document, 1, 1, "alpha"))
        .expect("first request");
    worker
        .request(request(&document, 1, 2, "gamma"))
        .expect("replaced request");
    worker
        .request(request(&document, 1, 3, "beta"))
        .expect("newest request");
    assert!(worker.is_pending());
    let found = reply(&mut worker);
    assert_eq!(
        found.identity.query, 3,
        "a reply for a superseded query was returned"
    );
    let matches = found.result.expect("newest query matches");
    assert_eq!(matches.ranges.len(), 2);
    assert_eq!(matches.ranges[0].start, 6);
    let settled = Instant::now();
    while settled.elapsed() < Duration::from_millis(50) {
        assert!(
            worker.poll().expect("settled poll").is_none(),
            "a second reply followed the newest one"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!worker.is_pending());
}

/// An external reload changes the revision tag,
///  and matches describe the new text only.
#[test]
fn reload_retags_requests_with_the_new_revision() {
    let mut document = Document::new("needle one");
    let mut worker = FindWorker::new().expect("find worker");
    worker
        .request(request(&document, 1, 1, "needle"))
        .expect("base revision request");
    let reload = document.prepare_reload("moved needle and needle");
    assert!(document.apply_reload(reload));
    worker
        .request(request(&document, 1, 1, "needle"))
        .expect("reloaded revision request");
    let found = reply(&mut worker);
    assert_eq!(found.identity.revision, document.revision());
    assert_eq!(found.identity.revision, 1);
    let matches = found.result.expect("reloaded matches");
    assert_eq!(matches.ranges.len(), 2);
    assert_eq!(matches.ranges[0].start, 6);
    assert_eq!(matches.ranges[1].start, 17);
}

/// Navigation changes the file tag even when revision and query numbers repeat.
#[test]
fn navigation_retags_requests_with_the_new_file_generation() {
    let first = Document::new("needle in the first file");
    let second = Document::new("second file: no match here, then NEEDLE");
    assert_eq!(first.revision(), second.revision());
    let mut worker = FindWorker::new().expect("find worker");
    worker
        .request(request(&first, 1, 1, "needle"))
        .expect("first file request");
    worker
        .request(request(&second, 2, 1, "needle"))
        .expect("second file request");
    let found = reply(&mut worker);
    assert_eq!(
        found.identity.file, 2,
        "a reply for the previous file was returned"
    );
    let matches = found.result.expect("second file matches");
    assert_eq!(matches.ranges.len(), 1);
    assert_eq!(matches.ranges[0].start, 33);
}

/// A query the matcher refuses arrives as a tagged diagnostic;
///  the worker keeps serving.
#[test]
fn refused_query_is_a_tagged_diagnostic_and_the_worker_recovers() {
    let document = Document::new("needle");
    let mut worker = FindWorker::new().expect("find worker");
    let too_long = "n".repeat(MAX_FIND_QUERY_CHARS + 1);
    worker
        .request(request(&document, 1, 1, &too_long))
        .expect("over-long request");
    let refused = reply(&mut worker);
    assert_eq!(refused.identity.query, 1);
    assert!(
        refused
            .result
            .expect_err("over-long query diagnostic")
            .to_string()
            .contains("at most 1000 characters")
    );
    worker
        .request(request(&document, 1, 2, "needle"))
        .expect("recovered request");
    assert_eq!(
        reply(&mut worker).result.expect("recovered").ranges.len(),
        1
    );
}

/// Cancelling wants nothing;
///  dropping joins the thread whether idle,
///  running,
///  or holding a reply.
#[test]
fn cancel_discards_replies_and_drop_joins_the_thread() {
    let document = Document::new(&"needle ".repeat(1000));
    let mut cancelled = FindWorker::new().expect("find worker");
    cancelled
        .request(request(&document, 1, 1, "needle"))
        .expect("cancelled request");
    cancelled.cancel();
    let waited = Instant::now();
    while waited.elapsed() < Duration::from_millis(50) {
        assert!(
            cancelled.poll().expect("cancelled poll").is_none(),
            "a cancelled request produced a visible reply"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    // What: `drop` ends ownership now, running the worker's shutdown instead of waiting for scope end.
    // Why: A hang here would mean the thread was not joined after its channel closed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // worker.terminate();
    // ```
    drop(cancelled);
    let idle = FindWorker::new().expect("idle worker");
    drop(idle);
    let mut busy = FindWorker::new().expect("busy worker");
    busy.request(request(&document, 1, 1, "needle"))
        .expect("running request");
    busy.request(request(&document, 1, 2, "needle"))
        .expect("waiting request");
    drop(busy);
}
