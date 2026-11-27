//! A server that dies:
//!  the request pending at the crash fails,
//!  the crashed process leaves the display,
//!  and the next request or file starts a new one.

/// The process identifiers the report names.
use super::started;
/// The scripted-server reader and the answers it waits for.
use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
    SERVER,
};
/// What each request asked for and what became of it.
use ide_app::language::reply::{
    RequestFailure,
    RequestKind,
    RequestOutcome,
};
/// The state a displayed server is in.
use ide_app::language::status::ServerState;

/// A crash fails the pending request,
///  removes the server,
///  and the next open starts a new one.
#[test]
fn crash_fails_the_pending_request_and_the_next_open_restarts() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::crash_and_restart::crash_fails_the_pending_request_and_the_next_open_restarts",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("HOVER", "crash")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("first.scripted"), "alpha\n");
    probe.until_ready();
    probe.wait_for("diagnostics from the first process");
    while !(!probe.messages().is_empty()) {
        probe.step();
    }
    let number = probe.request(RequestKind::Hover, 1);
    let answers = probe.answers(number);
    assert_eq!(
        answers[0].outcome,
        RequestOutcome::Failed(RequestFailure::StreamClosed),
        "the request pending at the crash did not fail"
    );
    probe.wait_for("the exited state");
    while !(probe.state(SERVER) == Some(&ServerState::Exited)) {
        probe.step();
    }
    assert!(
        probe.messages().is_empty(),
        "diagnostics of an exited server stayed on display"
    );
    support::children_until_none();
    let after_exit = probe.request(RequestKind::Definition, 1);
    // An explicit request starts the server again; this request is answered by the starting state.
    let restarted = probe.answers(after_exit);
    assert_eq!(
        restarted[0].outcome,
        RequestOutcome::Starting,
        "an exited server could not be started again"
    );
    assert_eq!(support::instance_of(&restarted[0]), Some(2));
    probe.until_ready();
    assert_eq!(probe.status.servers.len(), 1);
    assert_eq!(probe.status.servers[0].server.instance, 2);
    let definition = probe.request(RequestKind::Definition, 1);
    assert!(matches!(
        probe.answers(definition)[0].outcome,
        RequestOutcome::Locations(_)
    ));
    // Crash again, then restart by displaying a file.
    let second_crash = probe.request(RequestKind::Hover, 1);
    probe.answers(second_crash);
    probe.wait_for("the second exit");
    while !(probe.state(SERVER) == Some(&ServerState::Exited)) {
        probe.step();
    }
    probe.open(&root.join("second.scripted"), "beta\n");
    probe.until_ready();
    assert_eq!(probe.status.servers[0].server.instance, 3);
    let pids = started(&support::report(&root));
    assert_eq!(pids.len(), 3, "three processes were expected: {pids:?}");
    assert!(pids[0] != pids[1] && pids[1] != pids[2]);
}
