//! The states a request passes through:
//!  superseded and retried,
//!  unanswered before its timeout,
//!  the request timeout held while the start allowance is longer,
//!  and a reply overtaken by a reload or a file switch.

use super::SOURCE;
use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
};
use ide_app::language::reply::{
    RequestFailure,
    RequestKind,
    RequestOutcome,
};

/// `-32801` is retried while the text is unchanged,
///  and reported as superseded,
///  never as a failure.
#[test]
fn superseded_request_is_retried_then_reported_as_superseded() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::superseded_request_is_retried_then_reported_as_superseded",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("HOVER", "modified")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let number = probe.request(RequestKind::Hover, 3);
    let answers = probe.answers(number);
    assert_eq!(answers.len(), 1);
    assert_eq!(
        answers[0].outcome,
        RequestOutcome::Superseded,
        "a superseded request was shown as a failure"
    );
    let hovers = support::count_received(&support::report(&root), "textDocument/hover");
    assert_eq!(hovers, 4, "one request and three bounded retries");
    drop(probe);
    std::fs::remove_file(support::report_path(&root)).expect("previous report");
    let once = support::scripted(&root, &[("HOVER", "modified-once")], PRODUCT_TIMEOUT);
    let mut recovering = Probe::new(&root, once);
    recovering.open(&root.join("main.scripted"), SOURCE);
    recovering.until_ready();
    let retried = recovering.request(RequestKind::Hover, 3);
    assert!(matches!(
        recovering.answers(retried)[0].outcome,
        RequestOutcome::Hover(_)
    ));
}

/// A request the server never answers fails by the server's configured timeout.
#[test]
fn unanswered_request_times_out() {
    let Some(root) = support::child_root() else {
        support::run_child("requests::unanswered_request_times_out", support::standard);
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[("HOVER", "silent")], 1));
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let number = probe.request(RequestKind::Hover, 3);
    assert_eq!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Failed(RequestFailure::Timeout)
    );
}

/// A request keeps the request timeout although the server may take three request timeouts to
/// start:
///  an answer that comes after the request timeout is a timeout.
#[test]
fn request_keeps_the_request_timeout_while_the_start_allowance_is_longer() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::request_keeps_the_request_timeout_while_the_start_allowance_is_longer",
            support::standard,
        );
        return;
    };
    // A request timeout of 3 s gives helix-lsp a start allowance of 9 s; the hover answer comes
    // after 7 s. A stall of up to 4 s still ends in the timeout this test expects, and without
    // the worker's own bound the answer arrives 2 s before helix-lsp would give up.
    let definitions = support::scripted(&root, &[("HOVER_DELAY_MS", "7000")], 3);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let number = probe.request(RequestKind::Hover, 3);
    assert_eq!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Failed(RequestFailure::Timeout),
        "a request waited longer than its request timeout"
    );
}

/// A reply overtaken by a reload or a file switch is dropped when it is read.
#[test]
fn reply_overtaken_by_a_reload_or_file_switch_is_dropped() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::reply_overtaken_by_a_reload_or_file_switch_is_dropped",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("HOVER_DELAY_MS", "400")], PRODUCT_TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    // A file switch at equal revisions: only the file generation tells the reply apart.
    let before_switch = probe.request(RequestKind::Hover, 3);
    probe.open(&root.join("other.scripted"), "other file\n");
    probe.wait_for("the reply for the previous file to be dropped or delivered");
    while !(probe.worker.fence_counts().other_file == 1 || probe.has_reply(before_switch)) {
        probe.step();
    }
    assert!(
        !probe.has_reply(before_switch),
        "a reply for the previous file was accepted"
    );
    // A reload of the same file: only the revision tells the reply apart.
    let overtaken = probe.request(RequestKind::Hover, 3);
    probe.reload("changed text\n");
    probe.wait_for("the overtaken reply to be dropped or delivered");
    while !(probe.worker.fence_counts().stale_revision == 1 || probe.has_reply(overtaken)) {
        probe.step();
    }
    assert!(
        !probe.has_reply(overtaken),
        "a reply for the previous revision was accepted"
    );
    let current = probe.request(RequestKind::Hover, 3);
    let answers = probe.answers(current);
    let RequestOutcome::Hover(hover) = &answers[0].outcome else {
        panic!("hover produced {:?}", answers[0].outcome);
    };
    assert_eq!(hover.text, "line=changed text char=n");
}

/// A `-32801` answer for text that was reloaded meanwhile is not retried;
///  the fence drops it as stale.
#[test]
fn superseded_answer_for_reloaded_text_is_dropped_by_the_fence() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::superseded_answer_for_reloaded_text_is_dropped_by_the_fence",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(
        &root,
        &[("HOVER", "modified"), ("HOVER_DELAY_MS", "400")],
        PRODUCT_TIMEOUT,
    );
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let overtaken = probe.request(RequestKind::Hover, 3);
    probe.reload("changed text\n");
    probe.wait_for("the superseded answer to be dropped or delivered");
    while !(probe.worker.fence_counts().stale_revision == 1 || probe.has_reply(overtaken)) {
        probe.step();
    }
    assert!(
        !probe.has_reply(overtaken),
        "a superseded answer for reloaded text reached the interface"
    );
    std::thread::sleep(std::time::Duration::from_millis(600));
    let hovers = support::count_received(&support::report(&root), "textDocument/hover");
    assert_eq!(
        hovers, 1,
        "a request for text that is no longer displayed was retried"
    );
}
