//! What a reply may say:
//!  definition targets validated inside and outside the project,
//!  a failed request carrying the server's error code,
//!  hover positions and ranges in the negotiated column unit,
//!  an empty result on a blank line,
//!  and a missing capability reported per request.

use super::SOURCE;
use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
};
use ide_app::language::{
    hints::HintWindow,
    reply::{
        RequestFailure,
        RequestKind,
        RequestOutcome,
        Target,
    },
    target::TargetRefusal,
};

/// Definition answers are validated:
///  openable inside and outside the project,
///  or unavailable.
#[test]
fn definition_targets_are_validated_and_converted() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::definition_targets_are_validated_and_converted",
            support::standard,
        );
        return;
    };
    let outside = support::scratch(&root).join("library.scripted");
    std::fs::write(&outside, "library\n").expect("outside file");
    let outside_text = outside.display().to_string();
    let definitions = support::scripted(
        &root,
        &[("ENCODING", "utf-8"), ("OUTSIDE", outside_text.as_str())],
        PRODUCT_TIMEOUT,
    );
    let mut probe = Probe::new(&root, definitions);
    let file = root.join("main.scripted");
    probe.open(&file, SOURCE);
    probe.until_ready();
    let number = probe.request(RequestKind::Definition, 3);
    let answers = probe.answers(number);
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].stamp, probe.stamp());
    assert_eq!(answers[0].position, 3);
    let RequestOutcome::Locations(targets) = &answers[0].outcome else {
        panic!("definition produced {:?}", answers[0].outcome);
    };
    assert_eq!(targets.len(), 5);
    let Target::Open(same) = &targets[0] else {
        panic!("the same-document target is {:?}", targets[0]);
    };
    assert!(same.same_document && !same.outside_project);
    assert_eq!(same.path, file);
    assert_eq!(same.line, 2);
    // Six UTF-8 bytes on line 2 are the accented letter and the astral character: two characters.
    let line_start = SOURCE.chars().count() - "\u{e9}\u{1F600}cdef tail\n".chars().count();
    assert_eq!(same.range, Some((line_start, line_start + 2)));
    assert_eq!(
        targets[1],
        Target::Unavailable {
            uri: "untitled:Untitled-1".to_string(),
            refusal: TargetRefusal::UnsupportedScheme("untitled".to_string())
        },
        "a non-file target was not reported as unavailable"
    );
    assert!(matches!(
        &targets[2],
        Target::Unavailable { refusal: TargetRefusal::UnsupportedScheme(scheme), .. } if scheme == "jdt"
    ));
    assert_eq!(
        targets[3],
        Target::Unavailable {
            uri: "file:///definitely/missing/file.rs".to_string(),
            refusal: TargetRefusal::Missing
        }
    );
    let Target::Open(library) = &targets[4] else {
        panic!("the outside target is {:?}", targets[4]);
    };
    assert!(
        library.outside_project && !library.same_document,
        "a target outside the project root was not flagged outside project"
    );
    assert_eq!(library.path, outside);
    assert_eq!(library.range, Some((0, 1)));
}

/// A protocol error is a failed request with the server's code;
///  hover still works afterwards.
#[test]
fn failed_request_carries_the_servers_error_code() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::failed_request_carries_the_servers_error_code",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let number = probe.request(RequestKind::References, 3);
    assert_eq!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Failed(RequestFailure::Rpc {
            code: -32603,
            message: "scripted internal failure".to_string()
        })
    );
    let hover = probe.request(RequestKind::Hover, 3);
    assert!(matches!(
        probe.answers(hover)[0].outcome,
        RequestOutcome::Hover(_)
    ));
}

/// The same character offset reaches the server as the right column in every negotiated unit.
#[test]
fn hover_position_and_range_use_the_negotiated_column_unit() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::hover_position_and_range_use_the_negotiated_column_unit",
            support::standard,
        );
        return;
    };
    // Offset 13 is the `w` of `world`: after `café `, the astral character, and ` hello `.
    // Before it are 17 UTF-8 bytes, 14 UTF-16 units, and 13 characters.
    let position = 13;
    assert_eq!(SOURCE.chars().nth(position), Some('w'));
    for (encoding, column) in [("utf-8", 17), ("utf-16", 14), ("utf-32", 13), ("", 14)] {
        std::fs::remove_file(support::report_path(&root)).unwrap_or_default();
        let mut probe = Probe::new(
            &root,
            support::scripted(&root, &[("ENCODING", encoding)], PRODUCT_TIMEOUT),
        );
        probe.open(&root.join("main.scripted"), SOURCE);
        probe.until_ready();
        let number = probe.request(RequestKind::Hover, position);
        let answers = probe.answers(number);
        let RequestOutcome::Hover(hover) = &answers[0].outcome else {
            panic!("hover produced {:?}", answers[0].outcome);
        };
        assert_eq!(
            hover.text, "line=caf\u{e9} \u{1F600} hello world char=w",
            "the server saw another character in {encoding:?}"
        );
        assert!(hover.markdown);
        assert_eq!(hover.range, Some((position, position)));
        let lines = support::report(&root);
        let sent = support::first_received(&lines, "textDocument/hover").expect("hover request");
        assert_eq!(sent["params"]["position"]["character"], column);
        assert_eq!(sent["params"]["position"]["line"], 0);
    }
}

/// `null` is an empty successful result,
///  not a failure.
#[test]
fn hover_on_a_blank_line_is_an_empty_result() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::hover_on_a_blank_line_is_an_empty_result",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let mut blank = None;
    if let Some(bytes) = SOURCE.find("\n\n") {
        blank = Some(SOURCE[..bytes].chars().count() + 1);
    }
    let number = probe.request(RequestKind::Hover, blank.expect("blank line"));
    assert_eq!(probe.answers(number)[0].outcome, RequestOutcome::Empty);
    let outside = probe.request(RequestKind::Hover, 10_000);
    assert!(matches!(
        probe.answers(outside)[0].outcome,
        RequestOutcome::Failed(RequestFailure::Other(_))
    ));
}

/// A running server that lacks a feature answers "unsupported" for it and nothing else changes.
#[test]
fn missing_capabilities_are_reported_per_request() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "requests::missing_capabilities_are_reported_per_request",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[("FEATURES", "minimal")], PRODUCT_TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("main.scripted"), SOURCE);
    probe.until_ready();
    let features = probe.status.servers[0].features.expect("features");
    assert!(features.hover);
    assert!(!features.definition && !features.references && !features.inlay_hints);
    for kind in [RequestKind::Definition, RequestKind::References] {
        let number = probe.request(kind, 3);
        assert_eq!(
            probe.answers(number)[0].outcome,
            RequestOutcome::Unsupported
        );
    }
    let hover = probe.request(RequestKind::Hover, 3);
    assert!(matches!(
        probe.answers(hover)[0].outcome,
        RequestOutcome::Hover(_)
    ));
    let window = HintWindow {
        first_line: 0,
        visible_lines: 10,
    };
    assert!(
        probe
            .worker
            .request_hints(probe.stamp(), window)
            .expect("hint window")
    );
    std::thread::sleep(std::time::Duration::from_millis(300));
    probe.poll();
    assert!(probe.hints.is_none());
    let received = support::received(&support::report(&root));
    let mut unsupported_sent = false;
    for method in received.iter() {
        if method == "textDocument/definition"
            || method == "textDocument/references"
            || method == "textDocument/inlayHint"
        {
            unsupported_sent = true;
        }
    }
    assert!(
        !unsupported_sent,
        "an unsupported request was sent: {received:?}"
    );
}
