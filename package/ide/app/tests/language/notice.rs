//! The entry that says why a displayed file has no hover,
//!  no hints,
//!  or no definitions:
//!  published with the file's diagnostics,
//!  only while something is missing or failing,
//!  and gone once the server provides everything.
//!
//! The scripted server's `FEATURES`,
//!  `INIT`,
//!  `HOVER`,
//!  and hint steps make each state happen at a named request,
//!  so no test depends on how busy the machine is.

use crate::support::{self, PRODUCT_TIMEOUT, Probe, SERVER};
use ide_app::language::{
    hints::HintWindow,
    reply::{RequestKind, RequestOutcome},
};

/// The lines every test that needs hints reports.
const WINDOW: HintWindow = HintWindow {
    first_line: 0,
    visible_lines: 20,
};

/// A server whose program is not installed explains itself,
///  and nothing is started.
#[test]
fn a_missing_program_gives_the_entry_with_its_remedy() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "notice::a_missing_program_gives_the_entry_with_its_remedy",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[], PRODUCT_TIMEOUT).replace(
        env!("CARGO_BIN_EXE_ide-scripted-lsp"),
        "definitely-not-installed-language-server",
    );
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("file.scripted"), "one\n");
    probe.wait_for("the entry for a missing program");
    while !(probe.notice().is_some()) {
        probe.step();
    }
    let entry = probe.notice().expect("entry");
    assert_eq!(
        entry,
        "Hover, inlay hints, and go to definition are not available for this file.\n\
         scripted-ls is not available: the program 'definitely-not-installed-language-server' for scripted-ls was not found on PATH.\n\
         Servers are looked up again when a file is displayed, so open the file again after fixing this."
    );
    assert!(support::children().is_empty());
}

/// A server that exits while starting explains itself;
///  while it is still starting nothing is shown.
#[test]
fn a_failed_start_gives_the_entry_and_a_server_still_starting_gives_none() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "notice::a_failed_start_gives_the_entry_and_a_server_still_starting_gives_none",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("INIT", "exit")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the failed-start entry");
    while !(probe.notice().is_some()) {
        probe.step();
    }
    let entry = probe.notice().expect("entry");
    assert!(
        entry.contains(
            "scripted-ls failed to start: the server process ended before it finished starting."
        ),
        "{entry}"
    );
    support::children_until_none();
}

/// A server that has not answered `initialize` yet gives no entry.
#[test]
fn a_server_that_is_still_starting_gives_no_entry() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "notice::a_server_that_is_still_starting_gives_no_entry",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("INIT", "hang")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the starting state");
    while !(probe.state(SERVER) == Some(&ide_app::language::status::ServerState::Starting)) {
        probe.step();
    }
    // Starting is published before anything else could be; wait out several polls to see no entry.
    for _ in 0..20 {
        probe.poll();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(probe.notice(), None);
}

/// A running server that offers every feature gives no entry.
#[test]
fn a_server_that_offers_everything_gives_no_entry() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "notice::a_server_that_offers_everything_gives_no_entry",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("file.scripted"), "alpha\nbeta\n");
    probe.until_ready();
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), WINDOW)
            .expect("hint window")
    );
    probe.wait_for("hints");
    while !(probe.hints.is_some()) {
        probe.step();
    }
    for _ in 0..10 {
        probe.poll();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(probe.notice(), None);
}

/// A running server that does not announce inlay hints is named,
///  and hover and definitions are not blamed on it.
#[test]
fn a_server_without_inlay_hints_is_named() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "notice::a_server_without_inlay_hints_is_named",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("FEATURES", "hover")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    probe.wait_for("the entry for a server without hints");
    while !(probe.notice().is_some()) {
        probe.step();
    }
    assert_eq!(
        probe.notice().as_deref(),
        Some("Inlay hints are not offered by scripted-ls, so none are shown.")
    );
}

/// A hint request the server answers with an error is named,
///  with the server's own error code and message.
#[test]
fn a_failed_hint_request_is_named_and_a_later_answer_clears_it() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "notice::a_failed_hint_request_is_named_and_a_later_answer_clears_it",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("HINT_STEPS", "error,answer")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\nbeta\n");
    probe.until_ready();
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), WINDOW)
            .expect("hint window")
    );
    probe.wait_for("the entry for a failed hint request");
    while !(probe.notice().is_some()) {
        probe.step();
    }
    assert_eq!(
        probe.notice().as_deref(),
        Some(
            "The inlay hint request to scripted-ls failed: the server answered with error -32603: scripted internal error."
        )
    );
    // The window moves, so the next request is the one the script answers.
    assert!(
        probe
            .worker
            .request_hints(
                probe.stamp(),
                HintWindow {
                    first_line: 1,
                    visible_lines: 20,
                },
            )
            .expect("hint window")
    );
    probe.wait_for("the entry to go once hints arrive");
    while !(probe.hints.is_some() && probe.notice().is_none()) {
        probe.step();
    }
}

/// A hover the server never answers is named after its timeout,
///  and a later answer clears it.
#[test]
fn a_hover_that_times_out_is_named() {
    let Some(root) = support::child_root() else {
        support::run_child("notice::a_hover_that_times_out_is_named", support::standard);
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[("HOVER", "silent")], 1));
    probe.open(&root.join("file.scripted"), "alpha\nbeta\n");
    probe.until_ready();
    let number = probe.request(RequestKind::Hover, 1);
    let answers = probe.answers(number);
    assert!(
        matches!(answers[0].outcome, RequestOutcome::Failed(_)),
        "{:?}",
        answers[0].outcome
    );
    probe.wait_for("the entry for a hover that timed out");
    while !(probe.notice().is_some()) {
        probe.step();
    }
    assert_eq!(
        probe.notice().as_deref(),
        Some("The hover request to scripted-ls failed: the server did not answer in time.")
    );
}
