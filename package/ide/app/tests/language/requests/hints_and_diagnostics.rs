//! Hints and diagnostics through a reload:
//!  inlay hints shaped and following reloads,
//!  versioned diagnostics following the displayed version,
//!  an unversioned set pushed during the hold shown when the server answers or the hold expires,
//!  and pulled diagnostics following open and reload.

use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
    SERVER,
};
use ide_app::language::{
    diagnostics::{
        Freshness,
        HOLD_FALLBACK,
    },
    hints::{
        HintKind,
        HintWindow,
    },
    reply::RequestKind,
};
use std::time::{
    Duration,
    Instant,
};

/// Hints are shaped for drawing,
///  tagged with their text,
///  and asked for again after a reload.
#[test]
fn inlay_hints_are_shaped_and_follow_reloads() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::inlay_hints_are_shaped_and_follow_reloads",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("main.scripted"), "plain text line\nsecond\n");
    probe.until_ready();
    let window = HintWindow {
        first_line: 0,
        visible_lines: 20,
    };
    let asked = std::time::Instant::now();
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), window)
            .expect("hint window")
    );
    probe.wait_for("inlay hints");
    while !(probe.hints.is_some()) {
        probe.step();
    }
    let first = probe.hints.clone().expect("hints");
    assert_eq!(first.stamp, probe.stamp());
    assert_eq!((first.first_line, first.last_line), (0, 3));
    assert_eq!(
        first.hints.len(),
        1,
        "the hint on a missing line must be dropped"
    );
    assert_eq!(first.hints[0].label, "part-a-part-b");
    assert_eq!(first.hints[0].position, 5);
    assert_eq!(first.hints[0].kind, Some(HintKind::Type));
    assert!(first.hints[0].padding_left && !first.hints[0].padding_right);
    assert_eq!(first.hints[0].server.name, SERVER);
    probe.reload("longer plain text line\nsecond\nthird\n");
    let stamp = probe.stamp();
    probe.wait_for("hints for the reloaded text");
    while !(probe.hints_stamped(stamp)) {
        probe.step();
    }
    // Exactly one request per displayed text. Only a request the server left unanswered for its
    // whole timeout is sent again, so each further request needs one more elapsed timeout; on a
    // machine that did not stall for that long the count is exactly two.
    let timeouts = usize::try_from(asked.elapsed().as_secs() / PRODUCT_TIMEOUT);
    let allowed = 2 + timeouts.expect("small count");
    let requests = support::count_received(&support::report(&root), "textDocument/inlayHint");
    assert!(
        (2..=allowed).contains(&requests),
        "{requests} hint requests were sent where at most {allowed} can follow from timeouts"
    );
}

/// Versioned pushes are exact;
///  they replace each other across reloads without any hold.
#[test]
fn versioned_diagnostics_follow_the_displayed_version() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::versioned_diagnostics_follow_the_displayed_version",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("DIAG_VERSION", "1")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("main.scripted"), "first\n");
    probe.wait_for("diagnostics for the first text");
    while !(probe.messages() == ["TEXT:first\n"]) {
        probe.step();
    }
    let snapshot = probe.diagnostics.clone().expect("diagnostics");
    assert_eq!(snapshot.groups[0].source, "scripted");
    assert_eq!(snapshot.groups[0].items[0].freshness, Freshness::Versioned);
    assert_eq!(
        (
            snapshot.groups[0].items[0].start,
            snapshot.groups[0].items[0].end
        ),
        (0, 1)
    );
    probe.reload("second\n");
    let stamp = probe.stamp();
    probe.wait_for("diagnostics for the second text");
    while !(probe.messages() == ["TEXT:second\n"] && probe.diagnostics_stamped(stamp)) {
        probe.step();
    }
}

/// An unversioned set pushed right after a reload is kept back while the hold lasts and shown once
/// the server answers a request for the new text,
///  without any further push.
#[test]
fn unversioned_set_pushed_during_the_hold_is_shown_once_the_server_answers() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::unversioned_set_pushed_during_the_hold_is_shown_once_the_server_answers",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("main.scripted"), "first\n");
    probe.wait_for("diagnostics for the first text");
    while !(probe.messages() == ["TEXT:first\n"]) {
        probe.step();
    }
    let snapshot = probe.diagnostics.clone().expect("diagnostics");
    assert_eq!(
        snapshot.groups[0].items[0].freshness,
        Freshness::Unversioned
    );
    let reloaded = Instant::now();
    probe.reload("second\n");
    let stamp = probe.stamp();
    // The server pushes once, at once, for the change; that set arrives inside the hold.
    support::server_text_until(&root, "second\n");
    probe.wait_for("the previous revision's diagnostics to leave the display");
    while !(probe.diagnostics_stamped(stamp)) {
        probe.step();
    }
    std::thread::sleep(Duration::from_millis(300));
    probe.poll();
    // The hold ends by its fixed delay at the earliest `HOLD_FALLBACK` after the reload was sent,
    // because nothing is asked of the server after the reload. Only an observation made before
    // that can show that the set is kept back; a later one is skipped, never weakened.
    if reloaded.elapsed() < HOLD_FALLBACK {
        assert!(
            probe.messages().is_empty(),
            "an unversioned set that arrived during the hold was displayed before the hold ended: {:?}",
            probe.messages()
        );
    } else {
        eprintln!(
            "the check inside the hold was skipped: {:?} passed since the reload",
            reloaded.elapsed()
        );
    }
    // The hover answer ends the hold; the kept set is shown, and the server pushes nothing more.
    let number = probe.request(RequestKind::Hover, 1);
    probe.answers(number);
    probe.wait_for("the set pushed during the hold, shown once the server answered");
    while !(probe.messages() == ["TEXT:second\n"] && probe.diagnostics_stamped(stamp)) {
        probe.step();
    }
}

/// An unversioned set pushed right after a reload is shown when the hold's fixed delay passes,
/// although nothing is asked of the server after the reload and it pushes nothing else.
#[test]
fn unversioned_set_pushed_during_the_hold_is_shown_when_the_hold_expires() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::unversioned_set_pushed_during_the_hold_is_shown_when_the_hold_expires",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("main.scripted"), "first\n");
    probe.wait_for("diagnostics for the first text");
    while !(probe.messages() == ["TEXT:first\n"]) {
        probe.step();
    }
    probe.reload("second\n");
    let stamp = probe.stamp();
    probe.wait_for("the set pushed during the hold, shown when its fixed delay passed");
    while !(probe.messages() == ["TEXT:second\n"] && probe.diagnostics_stamped(stamp)) {
        probe.step();
    }
}

/// Pull diagnostics are requested after open and after every reload,
///  and fenced by stamp.
#[test]
fn pulled_diagnostics_follow_open_and_reload() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::pulled_diagnostics_follow_open_and_reload",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("PULL", "1"), ("PUSH", "0")], PRODUCT_TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), "first\n");
    probe.until_ready();
    assert!(
        probe.status.servers[0]
            .features
            .expect("features")
            .pull_diagnostics
    );
    probe.wait_for("pulled diagnostics");
    while !(probe.messages() == ["PULLED:first\n"]) {
        probe.step();
    }
    let snapshot = probe.diagnostics.clone().expect("diagnostics");
    assert_eq!(snapshot.groups[0].source, "scripted-pull");
    assert_eq!(snapshot.groups[0].items[0].freshness, Freshness::Pulled);
    probe.reload("second\n");
    probe.wait_for("pulled diagnostics for the reloaded text");
    while !(probe.messages() == ["PULLED:second\n"]) {
        probe.step();
    }
    assert_eq!(
        probe.diagnostics.clone().expect("diagnostics").stamp,
        probe.stamp()
    );
}
