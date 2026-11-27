//! Dropping the handle:
//!  every server is told to shut down,
//!  a server that never initialized is ended too,
//!  and a server that ignores `exit` is killed and reaped before the drop returns.

/// The scripted-server reader and the child processes it counts.
use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
    SERVER,
};
/// The state a displayed server is in.
use ide_app::language::status::ServerState;

/// Dropping the handle stops every server,
///  including one that never initialized.
#[test]
fn dropping_the_worker_leaves_no_child_process() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::shutdown::dropping_the_worker_leaves_no_child_process",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    assert_eq!(support::children().len(), 1);
    drop(probe);
    support::children_until_none();
    let received = support::received(&support::report(&root));
    assert!(
        received.contains(&"shutdown".to_string()) && received.contains(&"exit".to_string()),
        "the server was not asked to shut down: {received:?}"
    );
    // A server stuck in `initialize` cannot be asked. helix-lsp closes its input instead, and
    // keeps its handle inside the task that awaits `initialize` until the worker's runtime ends.
    let mut hanging = Probe::new(&root, support::scripted(&root, &[("INIT", "hang")], 30));
    hanging.open(&root.join("file.scripted"), "alpha\n");
    hanging.wait_for("the starting state");
    while !(hanging.state(SERVER) == Some(&ServerState::Starting)) {
        hanging.step();
    }
    assert_eq!(support::children().len(), 1);
    drop(hanging);
    support::children_until_none();
}

/// A server that ignores `exit` and the end of its input is killed after the grace,
///  and it is
/// reaped before the drop returns:
///  no child is left,
///  running or as a zombie.
#[test]
fn server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::shutdown::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("LINGER", "1")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    assert_eq!(support::children().len(), 1);
    let before = std::time::Instant::now();
    drop(probe);
    // Checked at once, without waiting: the drop joins the worker thread, and that thread ends
    // only when it has reaped what it killed.
    let left = support::children();
    assert!(
        left.is_empty(),
        "a child process was left when the drop returned: {left:?}"
    );
    assert!(
        before.elapsed() >= std::time::Duration::from_secs(1),
        "the server was not given its grace to exit: {:?}",
        before.elapsed()
    );
    let received = support::received(&support::report(&root));
    assert!(
        received.contains(&"shutdown".to_string()) && received.contains(&"exit".to_string()),
        "the server was not asked to shut down before it was killed: {received:?}"
    );
}
