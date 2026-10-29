//! Synthetic reply channels prove each identity component is checked,
//!  independent of thread timing.

/// Private fields build a worker handle around channels the test controls.
use super::{FindIdentity, FindReply, FindRequest, FindWorker};
/// Empty successful results make identity,
///  not match content,
///  the variable under test.
use crate::find::find_matches;
/// What:
///  `sync_channel` creates bounded sender and receiver ends;
///  `SyncSender` is the sending end.
/// Why:
///  The test publishes replies directly instead of racing a real scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const replies = boundedChannel<FindReply>(1);
/// ```
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

/// The identity every test wants;
///  each stale reply differs from it in exactly one component.
fn wanted() -> FindIdentity {
    return FindIdentity {
        file: 4,
        revision: 7,
        query: 9,
    };
}

/// A busy handle without a thread;
///  the returned ends let the test act as the worker.
fn handle(
    identity: Option<FindIdentity>,
) -> (FindWorker, SyncSender<FindReply>, Receiver<FindRequest>) {
    let (request_sender, request_receiver) = sync_channel(1);
    let (reply_sender, reply_receiver) = sync_channel(1);
    // What: `Some` stores the present sender; `thread: None` records that no OS thread exists.
    // Why: Drop then has nothing to join, and no real scan can answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const worker = { requests, replies, busy: true, waiting: undefined, wanted: identity, thread: undefined };
    // ```
    let worker = FindWorker {
        requests: Some(request_sender),
        replies: reply_receiver,
        busy: true,
        waiting: None,
        wanted: identity,
        thread: None,
    };
    return (worker, reply_sender, request_receiver);
}

/// A successful reply with no matches for the given tag.
fn reply(identity: FindIdentity) -> FindReply {
    return FindReply {
        identity,
        result: find_matches("", "", 1),
    };
}

/// Publish one stale reply,
///  then the exact wanted reply as the positive control.
fn rejects(stale: FindIdentity, message: &str) {
    let (mut worker, publisher, _requests) = handle(Some(wanted()));
    publisher.send(reply(stale)).expect("stale reply slot");
    // What: `expect` extracts the successful poll or fails the test; `is_none` checks nothing was returned.
    // Why: The stale reply must be consumed without reaching the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(worker.poll() === undefined, message);
    // ```
    assert!(
        worker.poll().expect("stale reply poll").is_none(),
        "{message}"
    );
    assert!(
        !worker.is_pending(),
        "a consumed reply must free the job slot"
    );
    worker.busy = true;
    publisher.send(reply(wanted())).expect("current reply slot");
    let accepted = worker
        .poll()
        .expect("current reply poll")
        .expect("positive control: the wanted identity is returned");
    assert_eq!(accepted.identity, wanted());
}

/// Navigation to another file must not show the previous file's matches.
#[test]
fn reply_for_another_file_generation_is_rejected() {
    let mut stale = wanted();
    stale.file = 3;
    rejects(stale, "a reply for another file generation was accepted");
}

/// An external reload must not show positions computed for the previous text.
#[test]
fn reply_for_another_content_revision_is_rejected() {
    let mut stale = wanted();
    stale.revision = 6;
    rejects(stale, "a reply for another content revision was accepted");
}

/// Typing must not show matches for the previous find text.
#[test]
fn reply_for_another_query_is_rejected() {
    let mut stale = wanted();
    stale.query = 8;
    rejects(stale, "a reply for another query was accepted");
}

/// Closing the bar wants nothing,
///  so even an identical late reply is dropped.
#[test]
fn reply_after_cancel_is_rejected() {
    let (mut worker, publisher, _requests) = handle(Some(wanted()));
    worker.cancel();
    publisher.send(reply(wanted())).expect("late reply slot");
    assert!(
        worker.poll().expect("late reply poll").is_none(),
        "a reply was accepted after cancellation"
    );
}

/// A reply frees the slot and the newest waiting request is sent exactly once.
#[test]
fn consumed_reply_dispatches_only_the_newest_waiting_request() {
    let (mut worker, publisher, requests) = handle(Some(wanted()));
    let mut older = wanted();
    older.query = 10;
    let mut newest = wanted();
    newest.query = 11;
    for identity in [older, newest] {
        worker
            .request(FindRequest {
                identity,
                source: helix_core::Rope::from_str("needle"),
                query: "needle".to_string(),
            })
            .expect("request while busy");
    }
    assert!(
        requests.try_recv().is_err(),
        "a request was sent while another job was running"
    );
    publisher.send(reply(wanted())).expect("running job reply");
    assert!(worker.poll().expect("superseded reply poll").is_none());
    let sent = requests.try_recv().expect("newest request dispatched");
    assert_eq!(sent.identity, newest);
    assert!(
        requests.try_recv().is_err(),
        "the replaced waiting request was also sent"
    );
}

/// A stopped worker reports once and releases its admission state.
#[test]
fn disconnected_worker_reports_and_releases_state() {
    let (mut worker, publisher, _requests) = handle(Some(wanted()));
    // What: `drop` ends the publisher's ownership, closing the reply channel.
    // Why: This is how the handle observes a worker thread that ended unexpectedly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // publisher.close();
    // ```
    drop(publisher);
    let error = worker.poll().expect_err("disconnected poll");
    assert!(error.to_string().contains("restart the application"));
    assert!(!worker.is_pending());
    let rejected = worker.request(FindRequest {
        identity: wanted(),
        source: helix_core::Rope::from_str(""),
        query: String::new(),
    });
    assert!(rejected.is_err(), "a closed worker accepted a request");
}
