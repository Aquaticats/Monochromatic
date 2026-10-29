//! Unexpected channel closure releases reader state instead of leaving permanently busy UI polling.

/// Private transport fields let tests model an exited worker without panicking a real thread.
use super::{QueuedRead, ReloadReply, ReloadWorker};
/// Use actual bounded channel disconnection,
///  not a mocked try_recv return value.
use std::sync::mpsc::sync_channel;

/// Test-only construction is shared with the file-opener failure-liveness test.
impl ReloadWorker {
    /// Both worker-side endpoints are gone while the caller still believes a request is outstanding.
    pub(crate) fn disconnected_for_test() -> Self {
        let (requests, receiver) = sync_channel::<QueuedRead>(1);
        let (sender, replies) = sync_channel::<ReloadReply>(1);
        // Dropping worker-side endpoints produces the standard library's real Disconnected state.
        drop(receiver);
        drop(sender);
        return Self {
            requests: Some(requests),
            replies,
            busy: true,
            thread: None,
        };
    }
}

/// A failed transport cannot retain a busy slot or a sender that advertises another possible request.
#[test]
fn disconnected_reader_releases_its_outstanding_slot() {
    let mut reader = ReloadWorker::disconnected_for_test();
    assert!(reader.is_busy());
    assert!(reader.try_take().is_err());
    assert!(
        !reader.is_busy(),
        "disconnected reader retained its busy slot"
    );
    assert!(reader.requests.is_none());
}
