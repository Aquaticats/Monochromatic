//! Hint windows:
//!  an answer for lines the reader no longer shows never replaces the current hints.
//!
//! The scripted server holds back its answer to the first hint request and writes it right
//! after the answer to the second,
//!  so the earlier window is answered last on the same stream.
//! The test waits until the report shows both answers written,
//!  so nothing depends on when the
//! worker reads them,
//!  and a hover round trip afterwards proves the worker has handled both.

use crate::support::{self, Probe};
use ide_app::language::{
    hints::HintWindow,
    reply::{RequestKind, RequestOutcome},
};
use serde_json::Value;

/// Request timeout of the scripted server,
///  in seconds:
///  Helix's default,
///  the product's bound.
const TIMEOUT: u64 = 20;

/// The request number of the first `textDocument/inlayHint` request in the report,
///  once it is there.
fn first_hint_request(lines: &[Value]) -> Option<Value> {
    let line = support::first_received(lines, "textDocument/inlayHint")?;
    return Some(line["id"].clone());
}

/// How many answers to `textDocument/inlayHint` requests the server has written so far.
fn hint_answers_sent(lines: &[Value]) -> usize {
    // `filter` keeps the received hint requests; `map` lends each one's id.
    let mut asked: Vec<&Value> = Vec::new();
    for line in support::all_received(lines, "textDocument/inlayHint") {
        asked.push(&line["id"]);
    }
    // An answer is a written message with an id and no method.
    let mut answers = 0;
    for line in lines {
        if line["sent"]["method"].is_null() && asked.contains(&&line["sent"]["id"]) {
            answers += 1;
        }
    }
    return answers;
}

/// The hints of an earlier window,
///  answered last,
///  do not replace those of the window shown now.
#[test]
fn late_answer_for_an_earlier_window_does_not_replace_the_current_hints() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "windows::late_answer_for_an_earlier_window_does_not_replace_the_current_hints",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("HINT_STEPS", "hold")], TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    let mut text = String::new();
    for line in 0..120 {
        text.push_str(&format!("line {line}\n"));
    }
    probe.open(&root.join("main.scripted"), &text);
    probe.until_ready();
    // The first window asks for lines 0 to 20; its answer is held back by the server.
    let earlier = HintWindow {
        first_line: 0,
        visible_lines: 10,
    };
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), earlier)
            .expect("first window")
    );
    {
        let mut wait = support::ReportWait::new(&root, "the first hint request");
        while !(first_hint_request(wait.lines()).is_some()) {
            wait.again();
        }
        wait.into_lines()
    };
    // The reader scrolled: the second window asks for lines 50 to 80 and is answered at once.
    let current = HintWindow {
        first_line: 60,
        visible_lines: 10,
    };
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), current)
            .expect("second window")
    );
    // The server writes the late answer right after the current one.
    {
        let mut wait = support::ReportWait::new(&root, "both hint answers written");
        while !(hint_answers_sent(wait.lines()) == 2) {
            wait.again();
        }
        wait.into_lines()
    };
    // Both answers were written before the server reads this hover, so the hover answer follows
    // them on the stream, and the worker handles both hint answers first.
    let number = probe.request(RequestKind::Hover, 0);
    assert!(matches!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Hover(_)
    ));
    probe.poll();
    let hints = probe.hints.clone().expect("hints");
    assert_eq!(
        (hints.first_line, hints.last_line),
        (50, 80),
        "an answer for an earlier window replaced the hints of the current one"
    );
    assert_eq!(hints.stamp, probe.stamp());
}
