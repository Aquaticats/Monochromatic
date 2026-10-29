//! A stopped reader produces one actionable failure instead of leaving the opener permanently pending.

/// The controller owns both its desired target and its reader's admission state.
use super::FileOpener;
/// Test-only transport construction uses real disconnected standard-library channels.
use crate::{reload_worker::ReloadWorker, workspace::Workspace};

/// Caller-side pending state is released together with the failed worker slot.
#[test]
fn disconnected_opener_clears_pending_work_after_reporting_failure() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    let mut opener = FileOpener {
        workspace,
        worker: ReloadWorker::disconnected_for_test(),
        generation: 1,
        pending: Some(fixture.path().join("source.txt")),
        pending_outside: false,
        submitted_outside: false,
    };
    assert!(opener.has_pending());
    assert!(opener.poll().is_err());
    assert!(
        !opener.has_pending(),
        "stopped opener remained eligible for continuous UI polling"
    );
}
