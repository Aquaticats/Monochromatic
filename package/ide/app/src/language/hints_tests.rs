//! Inlay request ranges and label shaping,
//!  without a server.

use super::{HintKind, HintWindow, request_lines, shape};
use crate::language::identity::ServerIdentity;
use helix_core::Rope;
use helix_lsp::{OffsetEncoding, lsp};

fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted".to_string(),
        instance: 3,
    };
}

fn hint(line: u32, character: u32, label: lsp::InlayHintLabel) -> lsp::InlayHint {
    return lsp::InlayHint {
        position: lsp::Position::new(line, character),
        label,
        kind: None,
        text_edits: None,
        tooltip: None,
        padding_left: None,
        padding_right: None,
        data: None,
    };
}

#[test]
fn request_covers_one_view_before_and_two_after_the_first_visible_line() {
    let window = HintWindow {
        first_line: 100,
        visible_lines: 30,
    };
    assert_eq!(request_lines(window, 1000), (70, 160));
}

#[test]
fn request_is_limited_to_the_text() {
    let near_start = HintWindow {
        first_line: 5,
        visible_lines: 30,
    };
    assert_eq!(request_lines(near_start, 40), (0, 40));
    let past_the_end = HintWindow {
        first_line: 500,
        visible_lines: 30,
    };
    assert_eq!(request_lines(past_the_end, 40), (40, 40));
    let huge = HintWindow {
        first_line: usize::MAX,
        visible_lines: usize::MAX,
    };
    assert_eq!(request_lines(huge, 7), (0, 7));
}

#[test]
fn label_parts_are_joined_and_edits_are_ignored() {
    let text = Rope::from_str("let value = make();\n");
    let mut with_parts = hint(
        0,
        9,
        lsp::InlayHintLabel::LabelParts(vec![
            lsp::InlayHintLabelPart {
                value: ": ".to_string(),
                ..Default::default()
            },
            lsp::InlayHintLabelPart {
                value: "Widget".to_string(),
                command: Some(lsp::Command {
                    title: "open".to_string(),
                    command: "scripted.command".to_string(),
                    arguments: None,
                }),
                ..Default::default()
            },
        ]),
    );
    with_parts.kind = Some(lsp::InlayHintKind::TYPE);
    with_parts.padding_left = Some(true);
    with_parts.text_edits = Some(vec![lsp::TextEdit::new(
        lsp::Range::new(lsp::Position::new(0, 9), lsp::Position::new(0, 9)),
        ": Widget".to_string(),
    )]);
    let shaped = shape(vec![with_parts], &text, OffsetEncoding::Utf16, &server());
    assert_eq!(shaped.len(), 1);
    assert_eq!(shaped[0].label, ": Widget");
    assert_eq!(shaped[0].position, 9);
    assert_eq!(shaped[0].kind, Some(HintKind::Type));
    assert!(shaped[0].padding_left);
    assert!(!shaped[0].padding_right);
    assert_eq!(shaped[0].server, server());
}

#[test]
fn hint_on_a_line_past_the_end_is_dropped() {
    let text = Rope::from_str("one\ntwo");
    let inside = hint(1, 3, lsp::InlayHintLabel::String("kept".to_string()));
    let past = hint(99, 0, lsp::InlayHintLabel::String("past-end".to_string()));
    let shaped = shape(vec![inside, past], &text, OffsetEncoding::Utf16, &server());
    assert_eq!(shaped.len(), 1, "a hint on a missing line was kept");
    assert_eq!(shaped[0].label, "kept");
    assert_eq!(shaped[0].position, 7);
}

#[test]
fn hint_position_uses_the_negotiated_column_unit() {
    let text = Rope::from_str("\u{e9}\u{1F600}x\n");
    for (encoding, column) in [
        (OffsetEncoding::Utf8, 6),
        (OffsetEncoding::Utf16, 3),
        (OffsetEncoding::Utf32, 2),
    ] {
        let mut parameter = hint(0, column, lsp::InlayHintLabel::String("name:".to_string()));
        parameter.kind = Some(lsp::InlayHintKind::PARAMETER);
        let shaped = shape(vec![parameter], &text, encoding, &server());
        assert_eq!(shaped[0].position, 2, "wrong offset for {encoding:?}");
        assert_eq!(shaped[0].kind, Some(HintKind::Parameter));
    }
}
