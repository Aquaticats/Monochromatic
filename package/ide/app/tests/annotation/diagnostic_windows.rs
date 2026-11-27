//! Which diagnostics a frame shows:
//!  underline windows bounded to the materialized rows,
//!  a range that starts above the window is still found,
//!  problems at the caret listed worst first,
//!  and the text a problem spells out.

use crate::fixtures::{
    SHOWN,
    TEXT,
    blocks,
    diagnostics,
    problem,
};
use helix_core::Rope;
use ide_app::{
    annotation::{
        Annotations,
        Mark,
        describe,
    },
    language::diagnostics::Severity,
};

/// A range starting above the window is found through the reach index;
///  ranges ending before it are not;
/// groups of several sources merge into one start order.
#[test]
fn diagnostic_window_finds_ranges_starting_above_it() {
    let text = Rope::from(TEXT);
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &text,
        None,
        diagnostics(
            SHOWN,
            vec![
                (
                    "rustc",
                    vec![
                        problem(0, 3, Some(Severity::Error), "early"),
                        problem(4, 30, Some(Severity::Warning), "long"),
                    ],
                ),
                ("clippy", vec![problem(2, 6, Some(Severity::Hint), "short")]),
            ],
        ),
    );
    // Rows 3 and 4 start at character 25; only the long range reaches them.
    let below = annotations.marks(SHOWN, &text, 3, 5);
    assert_eq!(
        below,
        [Mark {
            start: 4,
            end: 30,
            severity: Severity::Warning,
        }]
    );
    let top = annotations.marks(SHOWN, &text, 0, 1);
    let mut starts: Vec<usize> = Vec::new();
    for mark in top.iter() {
        starts.push(mark.start);
    }
    assert_eq!(starts, [0, 2, 4]);
    // Every message row stands above the line its diagnostic starts on, here line 0, and nowhere else.
    let assembled = blocks(&mut annotations, SHOWN);
    assert_eq!(assembled.len(), 1);
    assert_eq!(assembled[0].line, 0);
    assert_eq!(assembled[0].messages.len(), 3);
}

/// The problems at the caret are every range touching it,
///  ends included,
///  worst first;
/// an omitted severity is a warning and a point range is found at its position.
#[test]
fn caret_problems_include_range_ends_and_order_by_severity() {
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &Rope::from(TEXT),
        None,
        diagnostics(
            SHOWN,
            vec![
                (
                    "rustc",
                    vec![
                        problem(4, 9, Some(Severity::Hint), "suggestion"),
                        problem(6, 9, None, "no severity"),
                        problem(9, 9, Some(Severity::Error), "point"),
                    ],
                ),
                (
                    "other",
                    vec![problem(12, 14, Some(Severity::Error), "elsewhere")],
                ),
            ],
        ),
    );
    let described_at_end = annotations.at(SHOWN, 9);
    let mut at_end: Vec<&str> = Vec::new();
    for item in described_at_end.iter() {
        at_end.push(item.message.as_str());
    }
    assert_eq!(at_end, ["point", "no severity", "suggestion"]);
    assert_eq!(annotations.at(SHOWN, 6)[0].mark.severity, Severity::Warning);
    let described_at_start = annotations.at(SHOWN, 4);
    let mut at_start: Vec<&str> = Vec::new();
    for item in described_at_start.iter() {
        at_start.push(item.message.as_str());
    }
    assert_eq!(at_start, ["suggestion"]);
    assert!(annotations.at(SHOWN, 10).is_empty());
    assert!(annotations.at(SHOWN, 3).is_empty());
}

/// A problem's text names the severity in words,
///  then the code and source,
///  then the trimmed message.
#[test]
fn problem_text_spells_out_severity_code_and_source() {
    let mut coded = problem(
        0,
        1,
        Some(Severity::Error),
        "mismatched types\nexpected `u32`\n",
    );
    coded.code = Some("E0308".to_string());
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &Rope::from(TEXT),
        None,
        diagnostics(
            SHOWN,
            vec![
                ("rustc", vec![coded]),
                (
                    "",
                    vec![problem(2, 3, Some(Severity::Information), "plain")],
                ),
            ],
        ),
    );
    assert_eq!(
        describe(annotations.at(SHOWN, 0)[0]),
        "Error E0308 (rustc): mismatched types\nexpected `u32`"
    );
    assert_eq!(describe(annotations.at(SHOWN, 2)[0]), "Information: plain");
}
