//! Controlled watch slots prove generation filtering independently of subprocess scheduling luck.

/// Private fields expose only an in-memory worker fixture,
///  never an application debug callback.
use super::{SearchReply, SearchWorker};
/// Empty successful streams make reply identity,
///  rather than filesystem content,
///  the variable under test.
use crate::search::SearchResults;
/// Shared immutable replies keep their allocation alive across the watch-slot handoff.
use std::{path::PathBuf, sync::Arc};
/// Watch channels retain one value and a changed bit rather than an unbounded message queue.
use tokio::sync::watch;

/// Construct synthetic results without scanning or mutating any real directory.
fn reply(generation: u64) -> Arc<SearchReply> {
    return Arc::new(SearchReply {
        generation,
        query: "unchanged pattern".to_string(),
        scope: PathBuf::from("unused-scope"),
        results: SearchResults {
            paths: Ok(Vec::new()),
            contents: Ok(Vec::new()),
        },
    });
}

/// A queued old reply is consumed but rejected;
///  an otherwise identical current reply is accepted.
#[test]
fn unread_reply_identity_must_equal_the_current_generation() {
    // What: each watch pair separates a publisher from its latest-value receiver.
    // Why: Direct publication makes stale/current cases deterministic without sleeps or a fast-child assumption.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const replies = latestValueChannel();
    // const worker = { replies, generation: 2 };
    // ```
    let (requests, _request_receiver) = watch::channel(None);
    let (publisher, replies) = watch::channel(None);
    let mut worker = SearchWorker {
        requests: Some(requests),
        replies,
        cancellation: None,
        generation: 2,
        thread: None,
    };
    publisher.send_replace(Some(reply(1)));
    assert!(
        worker.try_take().expect("old reply poll").is_none(),
        "a stale reply crossed the current generation boundary"
    );
    assert!(
        worker
            .try_take()
            .expect("old reply consumed once")
            .is_none()
    );
    publisher.send_replace(Some(reply(2)));
    assert_eq!(
        worker
            .try_take()
            .expect("current reply poll")
            .expect("positive current-generation control")
            .generation,
        2
    );
}
