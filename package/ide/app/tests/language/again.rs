//! Requests the worker makes on its own,
//!  asked again:
//!  an inlay-hint or pull-diagnostics request
//! the server leaves unanswered for its whole request timeout,
//!  or supersedes on every retry,
//! must not be the last one.
//!
//! The scripted server's request steps (`IDE_SCRIPTED_HINT_STEPS`,
//!  `IDE_SCRIPTED_PULL_STEPS`)
//! name exactly which request is left unanswered or superseded,
//!  and every other request is
//! answered at once.
//!  So the client's reaction does not depend on how busy the machine is:
//!  a
//! further timeout would need a stall of a whole request timeout,
//!  every time.

use crate::support::{self, PRODUCT_TIMEOUT, Probe};
use ide_app::language::{
    hints::HintWindow,
    reply::{RequestKind, RequestOutcome},
};
use serde_json::Value;
use std::{path::Path, time::Instant};

/// Request timeout of the scripted server in the tests where a request must time out,
///  in
/// seconds.
///  The silent request takes this long to fail;
///  the request sent again is answered at
/// once,
///  so it fails only if a stall of this length hits it,
///  and the two after it.
const TIMEOUT: u64 = 5;

/// The visible lines every test reports.
const WINDOW: HintWindow = HintWindow {
    first_line: 0,
    visible_lines: 20,
};

/// Text displayed first,
///  and the text an external change replaces it with.
const BEFORE: &str = "plain text line\nsecond\n";
const AFTER: &str = "longer plain text line\nsecond\nthird\n";

/// The answer to the first hint request is fine;
///  the one after the reload and its three retries
/// are superseded,
///  and the last of them is followed by a notification.
const SUPERSEDED_WITH_NOTIFICATION: &str = "answer,modified,modified,modified,modified-notify";

/// As `SUPERSEDED_WITH_NOTIFICATION`,
///  without the notification.
const SUPERSEDED: &str = "answer,modified,modified,modified,modified";

/// True when the server wrote the answer to the fifth `textDocument/inlayHint` request.
fn last_retry_answered(lines: &[Value]) -> bool {
    let requests = hint_requests(lines);
    // `get(4)` is the fifth request: the request after the reload plus three retries.
    let Some(last) = requests.get(4) else {
        return false;
    };
    for line in lines {
        if line["sent"]["id"] == last["id"] {
            return true;
        }
    }
    return false;
}

/// The `textDocument/inlayHint` requests the server received so far.
fn hint_requests(lines: &[Value]) -> Vec<&Value> {
    return support::all_received(lines, "textDocument/inlayHint");
}

/// Display `BEFORE`,
///  report the window,
///  and wait for the hints of that text.
fn display_with_hints(probe: &mut Probe, root: &Path) {
    probe.open(&root.join("main.scripted"), BEFORE);
    probe.until_ready();
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), WINDOW)
            .expect("hint window")
    );
    probe.wait_for("hints for the first text");
    while !(probe.hints.is_some()) {
        probe.step();
    }
}

/// Wait for hints of the reloaded text.
///  Every snapshot handed out after the reload must describe
/// the new text;
///  `what` names the wait in a failure.
fn wait_for_reloaded_hints(probe: &mut Probe, what: &str) {
    let stamp = probe.stamp();
    probe.wait_for(what);
    while probe.hints.is_none() {
        probe.step();
    }
    let hints = probe.hints.clone().expect("hints");
    assert_eq!(
        hints.stamp, stamp,
        "a hint snapshot for other text was handed out after the reload"
    );
    // The request that was answered named the lines of the new text, which has one line more.
    assert_eq!((hints.first_line, hints.last_line), (0, 4));
    assert_eq!(hints.hints.len(), 1);
}

/// Reload to `AFTER` and forget the snapshot of the previous text,
///  so anything seen from here
/// on was handed out after the reload.
fn reload(probe: &mut Probe) {
    probe.reload(AFTER);
    probe.hints = None;
}

/// The first hint request of a displayed file times out;
///  the hints still arrive.
#[test]
fn hints_are_asked_again_when_the_first_request_times_out() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "again::hints_are_asked_again_when_the_first_request_times_out",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("HINT_STEPS", "silent")], TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), BEFORE);
    probe.until_ready();
    let asked = Instant::now();
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), WINDOW)
            .expect("hint window")
    );
    probe.wait_for("hints for the first text after the first request timed out");
    while !(probe.hints.is_some()) {
        probe.step();
    }
    let hints = probe.hints.clone().expect("hints");
    assert_eq!(hints.stamp, probe.stamp());
    assert_eq!((hints.first_line, hints.last_line), (0, 3));
    // A request is sent again only after a whole request timeout passed without an answer.
    let allowed = 1 + usize::try_from(asked.elapsed().as_secs() / TIMEOUT).expect("small count");
    let requests = hint_requests(&support::report(&root)).len();
    assert!(
        (2..=allowed).contains(&requests),
        "{requests} hint requests were sent where at most {allowed} can follow from timeouts"
    );
}

/// The hint request that follows a reload times out;
///  hints for the new text still arrive.
#[test]
fn hints_are_asked_again_after_a_reload_when_the_request_times_out() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "again::hints_are_asked_again_after_a_reload_when_the_request_times_out",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("HINT_STEPS", "answer,silent")], TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    display_with_hints(&mut probe, &root);
    let reloaded = Instant::now();
    reload(&mut probe);
    wait_for_reloaded_hints(
        &mut probe,
        "hints for the reloaded text after the request timed out",
    );
    // One request for the first text, one for the new text, and one more per elapsed timeout.
    let allowed = 2 + usize::try_from(reloaded.elapsed().as_secs() / TIMEOUT).expect("small count");
    let requests = hint_requests(&support::report(&root)).len();
    assert!(
        (3..=allowed).contains(&requests),
        "{requests} hint requests were sent where at most {allowed} can follow from timeouts"
    );
}

/// The pull request that follows a reload times out;
///  diagnostics for the new text still arrive.
#[test]
fn pulled_diagnostics_are_asked_again_after_a_reload_when_the_request_times_out() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "again::pulled_diagnostics_are_asked_again_after_a_reload_when_the_request_times_out",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(
        &root,
        &[
            ("PULL", "1"),
            ("PUSH", "0"),
            ("PULL_STEPS", "answer,silent"),
        ],
        TIMEOUT,
    );
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), "first\n");
    probe.wait_for("pulled diagnostics for the first text");
    while !(probe.messages() == ["PULLED:first\n"]) {
        probe.step();
    }
    probe.reload("second\n");
    let stamp = probe.stamp();
    probe.wait_for("pulled diagnostics for the reloaded text after the request timed out");
    while !(probe.messages() == ["PULLED:second\n"] && probe.diagnostics_stamped(stamp)) {
        probe.step();
    }
}

/// The request after a reload is superseded,
///  and so is every retry;
///  the notification the server
/// sends after the last one makes the worker ask once more.
#[test]
fn hints_are_asked_again_when_the_server_sends_a_notification_after_every_retry_was_superseded() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "again::hints_are_asked_again_when_the_server_sends_a_notification_after_every_retry_was_superseded",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(
        &root,
        &[("HINT_STEPS", SUPERSEDED_WITH_NOTIFICATION)],
        PRODUCT_TIMEOUT,
    );
    let mut probe = Probe::new(&root, definitions);
    display_with_hints(&mut probe, &root);
    reload(&mut probe);
    wait_for_reloaded_hints(
        &mut probe,
        "hints for the reloaded text after the server's notification followed the superseded retries",
    );
    // The first text, the request after the reload, three retries, and the one catch-up ask.
    assert_eq!(hint_requests(&support::report(&root)).len(), 6);
}

/// The request after a reload is superseded,
///  and so is every retry,
///  and the server pushes
/// nothing;
///  its answer to a later hover request makes the worker ask once more.
#[test]
fn hints_are_asked_again_when_the_server_answers_another_request_after_every_retry_was_superseded()
{
    let Some(root) = support::child_root() else {
        support::run_child(
            "again::hints_are_asked_again_when_the_server_answers_another_request_after_every_retry_was_superseded",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(
        &root,
        &[("PUSH", "0"), ("HINT_STEPS", SUPERSEDED)],
        PRODUCT_TIMEOUT,
    );
    let mut probe = Probe::new(&root, definitions);
    display_with_hints(&mut probe, &root);
    reload(&mut probe);
    // Wait until the server wrote its answer to the last retry. The hover is sent after that,
    // so its answer follows on the stream, and the worker has given up asking by then.
    let mut retry_wait = support::ReportWait::new(&root, "the answer to the last retry");
    while !last_retry_answered(retry_wait.lines()) {
        retry_wait.again();
    }
    // Nothing but the hover can make the worker ask again now: the server sends no other message.
    probe.poll();
    assert!(
        probe.hints.is_none(),
        "hints arrived although every request for the reloaded text was superseded"
    );
    let number = probe.request(RequestKind::Hover, 3);
    assert!(matches!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Hover(_)
    ));
    wait_for_reloaded_hints(
        &mut probe,
        "hints for the reloaded text after the server's hover answer followed the superseded retries",
    );
    assert_eq!(hint_requests(&support::report(&root)).len(), 6);
}
