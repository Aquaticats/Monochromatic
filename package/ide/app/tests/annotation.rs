//! Which hints and diagnostics a frame paints:
//!  stale snapshots paint nothing,
//!  hints are grouped by line and
//! accumulated per revision,
//!  underline windows are bounded to the materialized rows,
//!  multi-line ranges above a
//! window are found,
//!  problems at the caret are listed worst first,
//!  and space is held across a reload.

/// What:
///  Helix's rope and the shared-ownership pointer the Language module hands snapshots out in.
/// Why:
///  Windows are computed from the displayed text's line starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The production selection logic,
///  the Language module's snapshot records,
///  and what assembly needs:
/// the displayed document,
///  the shaper that packs hint rows,
///  and the block record.
use ide_app::{
    annotation::{Annotations, Held, Mark, describe},
    document::Document,
    language::{
        diagnostics::{Diagnostic, DiagnosticsSnapshot, Freshness, Severity, SourceGroup},
        hints::{HintKind, HintsSnapshot, InlayHint},
        identity::{DocumentStamp, ServerIdentity},
    },
    shaped_text::TextShaper,
    virtual_row::{BLOCK_GAP, Block, ROW_HEIGHT},
};
/// What:
///  Snapshots and blocks are shared values;
///  `Duration` and `Instant` time held space.
/// Why:
///  Held space ends at a point in time the tests choose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = performance.now();
/// ```
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// The displayed text every test uses:
///  five lines,
///  the third empty,
///  the last without a terminator.
const TEXT: &str = "let a = 1;\nlet bb = 22;\n\nfn main() {\n}";

/// The stamp of the displayed text.
const SHOWN: DocumentStamp = DocumentStamp {
    file: 3,
    revision: 7,
};

/// The process every test record names.
fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted".to_string(),
        instance: 1,
    };
}

/// One hint at `position` with `label`.
fn hint(position: usize, label: &str) -> InlayHint {
    return InlayHint {
        position,
        label: label.to_string(),
        kind: Some(HintKind::Type),
        padding_left: true,
        padding_right: false,
        server: server(),
    };
}

/// What:
///  One diagnostic over `start..end`;
///  `Option<Severity>` lets a test omit the severity.
/// Why:
///  Records are built exactly as the Language module builds them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function problem(start: number, end: number, severity: Severity | undefined, message: string): Diagnostic;
/// ```
fn problem(start: usize, end: usize, severity: Option<Severity>, message: &str) -> Diagnostic {
    return Diagnostic {
        start,
        end,
        severity,
        code: None,
        message: message.to_string(),
        server: server(),
        freshness: Freshness::Versioned,
    };
}

/// Shared hint snapshot for `stamp` that answers for every line of the text.
fn hints(stamp: DocumentStamp, items: Vec<InlayHint>) -> Option<Arc<HintsSnapshot>> {
    return ranged(stamp, (0, 5), items);
}

/// Shared hint snapshot for `stamp` that answers for the lines `asked.0` up to,
///  not including,
///  `asked.1`.
fn ranged(
    stamp: DocumentStamp,
    asked: (usize, usize),
    items: Vec<InlayHint>,
) -> Option<Arc<HintsSnapshot>> {
    return Some(Arc::new(HintsSnapshot {
        stamp,
        first_line: asked.0,
        last_line: asked.1,
        hints: items,
    }));
}

/// What:
///  The blocks the store assembles for the text `stamp`,
///  packed by the production shaper at scale one.
///       `&mut Annotations` lends the store for change,
///  because assembly fills its packing cache.
/// Why:
///  Blocks are what the vertical mapping and every frame take from the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blocks(store: Annotations, stamp: DocumentStamp): Block[];
/// ```
fn blocks(store: &mut Annotations, stamp: DocumentStamp) -> Vec<Arc<Block>> {
    let document = Document::new(TEXT);
    let mut shaper = TextShaper::new();
    return store.assemble(stamp, &document, &mut shaper, 1.0, None, &(0..usize::MAX));
}

/// The hint texts of the block above `line`,
///  or nothing when the line has no block.
fn hint_texts(assembled: &[Arc<Block>], line: usize) -> Vec<String> {
    let mut texts = Vec::new();
    for block in assembled {
        if block.line == line {
            for hint in &block.hints {
                texts.push(hint.text.clone());
            }
        }
    }
    return texts;
}

/// The hint snapshot a test expects to exist.
fn unwrapped(snapshot: Option<Arc<HintsSnapshot>>) -> Arc<HintsSnapshot> {
    return snapshot.expect("hint snapshot");
}

/// The diagnostics snapshot of one `rustc` problem with `message`.
fn problems(stamp: DocumentStamp, message: &str) -> Arc<DiagnosticsSnapshot> {
    return diagnostics(
        stamp,
        vec![("rustc", vec![problem(4, 5, Some(Severity::Error), message)])],
    )
    .expect("diagnostics snapshot");
}

/// The line each block stands above,
///  in order.
fn block_lines(assembled: &[Arc<Block>]) -> Vec<usize> {
    let mut lines = Vec::new();
    for block in assembled {
        lines.push(block.line);
    }
    return lines;
}

/// Shared diagnostics snapshot for `stamp` with one group per `(source, items)` pair.
fn diagnostics(
    stamp: DocumentStamp,
    groups: Vec<(&str, Vec<Diagnostic>)>,
) -> Option<Arc<DiagnosticsSnapshot>> {
    let mut built = Vec::new();
    for (source, items) in groups {
        built.push(SourceGroup {
            source: source.to_string(),
            items,
        });
    }
    return Some(Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: built,
    }));
}

/// Snapshots accepted for another file generation or another revision paint nothing for the displayed text:
/// no block,
///  no underline,
///  and no problem at the caret.
#[test]
fn stale_snapshots_paint_nothing_and_show_no_card() {
    let text = Rope::from(TEXT);
    for stamp in [
        DocumentStamp {
            file: 2,
            revision: 7,
        },
        DocumentStamp {
            file: 3,
            revision: 6,
        },
        DocumentStamp {
            file: 3,
            revision: 8,
        },
    ] {
        // The snapshots are accepted while their own text is displayed; then the displayed text moves on.
        let mut annotations = Annotations::default();
        annotations.replace(
            stamp,
            &text,
            hints(stamp, vec![hint(5, ": i32")]),
            diagnostics(stamp, vec![("rustc", vec![problem(4, 5, None, "unused")])]),
        );
        assert_eq!(
            blocks(&mut annotations, stamp).len(),
            1,
            "positive control: {stamp:?}"
        );
        let stale = blocks(&mut annotations, SHOWN);
        let mut hinted = 0;
        let mut messaged = 0;
        for block in &stale {
            hinted += block.hints.len();
            messaged += block.messages.len();
        }
        assert_eq!(hinted, 0, "a stale hint was painted: {stamp:?}");
        assert_eq!(messaged, 0, "a stale message was painted: {stamp:?}");
        assert!(
            annotations.marks(SHOWN, &text, 0, 5).is_empty(),
            "a stale mark was painted: {stamp:?}"
        );
        assert!(annotations.at(SHOWN, 4).is_empty());
    }
    let mut current = Annotations::default();
    current.replace(
        SHOWN,
        &text,
        hints(SHOWN, vec![hint(5, ": i32")]),
        diagnostics(SHOWN, vec![("rustc", vec![problem(4, 5, None, "unused")])]),
    );
    let shown = blocks(&mut current, SHOWN);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].hints.len(), 1);
    assert_eq!(shown[0].messages.len(), 1);
    assert_eq!(current.marks(SHOWN, &text, 0, 5).len(), 1);
    // A snapshot handed over for text that is not displayed is never grouped or painted.
    let mut refused = Annotations::default();
    refused.replace(
        SHOWN,
        &text,
        hints(
            DocumentStamp {
                file: 3,
                revision: 8,
            },
            vec![hint(5, ": i32")],
        ),
        None,
    );
    assert!(blocks(&mut refused, SHOWN).is_empty());
}

/// Hints belong to the line their position lies on:
///  a hint at a line end stays on its line,
///  the end of the
/// text belongs to the last line,
///  labels lose their padding spaces,
///  and blank labels are dropped.
#[test]
fn hints_are_grouped_by_line_and_line_ends() {
    let text = Rope::from(TEXT);
    // Line starts: 0, 11, 24, 25, 37; the text has 38 characters.
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &text,
        hints(
            SHOWN,
            vec![
                hint(5, " : i32 "),
                hint(10, "end-of-line-0"),
                hint(11, "start-of-line-1"),
                hint(24, "   "),
                hint(38, "end-of-text"),
            ],
        ),
        None,
    );
    let assembled = blocks(&mut annotations, SHOWN);
    let lines: Vec<usize> = block_lines(&assembled);
    assert_eq!(
        lines,
        [0, 1, 4],
        "the blank label on line 2 must own no block"
    );
    assert_eq!(hint_texts(&assembled, 0), [": i32", "end-of-line-0"]);
    assert_eq!(hint_texts(&assembled, 1), ["start-of-line-1"]);
    assert_eq!(hint_texts(&assembled, 4), ["end-of-text"]);
    assert_eq!(assembled[2].hints[0].position, 38);
}

/// A hint snapshot replaces the hints of the lines it asked about and keeps those of other lines of the same
/// revision;
///  hints of another revision are dropped as a whole.
#[test]
fn hints_of_one_revision_accumulate_by_line_range() {
    let text = Rope::from(TEXT);
    let mut store = Annotations::default();
    // The view first shows lines 0 and 1, then lines 3 and 4.
    let upper = ranged(SHOWN, (0, 2), vec![hint(5, ": i32"), hint(17, ": i64")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(upper)));
    let lower = ranged(SHOWN, (3, 5), vec![hint(37, "end")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(lower)));
    let both = blocks(&mut store, SHOWN);
    let lines: Vec<usize> = block_lines(&both);
    assert_eq!(
        lines,
        [0, 1, 4],
        "an answer for other lines removed hints it did not ask about"
    );
    // A second answer for lines 0 and 1 without the hint on line 0 removes exactly that one.
    let again = ranged(SHOWN, (0, 2), vec![hint(17, ": u8")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(again)));
    let replaced = blocks(&mut store, SHOWN);
    let kept: Vec<usize> = block_lines(&replaced);
    assert_eq!(
        kept,
        [1, 4],
        "an answer did not replace the hints of its own lines"
    );
    assert_eq!(hint_texts(&replaced, 1), [": u8"]);
    // Hints of the next revision start from nothing.
    let next = DocumentStamp {
        file: 3,
        revision: 8,
    };
    let fresh = ranged(next, (0, 2), vec![hint(5, ": new")]);
    assert!(store.accept_hints(next, &text, unwrapped(fresh)));
    let renewed = blocks(&mut store, next);
    assert_eq!(renewed.len(), 1, "hints of the previous revision were kept");
    assert_eq!(hint_texts(&renewed, 0), [": new"]);
}

/// A server answers for every node of the text that reaches into the lines it was asked about,
///  so an answer for later lines can hold hints that stand on earlier ones
///  (TypeScript 7 answers a request for lines 100 to 200 of a test file with the four hints of the
///  `const integrationChildren = ...map(...)` statement on line 96,
///  whose body spans the whole file).
///  Those hints are not part of the answer:
///  the store replaces the hints of the lines asked about and keeps every other line as it was,
///  so an answer that repeats a hint of an unasked line adds nothing to that line.
#[test]
fn hints_outside_the_asked_lines_are_not_part_of_an_answer() {
    let text = Rope::from(TEXT);
    let mut store = Annotations::default();
    let upper = ranged(SHOWN, (0, 2), vec![hint(5, ": i32"), hint(17, ": i64")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(upper)));
    // The next answers ask for lines 3 and 4 and repeat the hint of line 0 and add one of line 1, as a server does.
    for _ in 0..3 {
        let lower = ranged(
            SHOWN,
            (3, 5),
            vec![hint(5, ": i32"), hint(20, ": extra"), hint(37, "end")],
        );
        assert!(store.accept_hints(SHOWN, &text, unwrapped(lower)));
    }
    let assembled = blocks(&mut store, SHOWN);
    assert_eq!(
        block_lines(&assembled),
        [0, 1, 4],
        "an answer put hints on lines it did not ask about"
    );
    assert_eq!(
        hint_texts(&assembled, 0),
        [": i32"],
        "an answer that repeated the hint of an unasked line added it again"
    );
    assert_eq!(
        hint_texts(&assembled, 1),
        [": i64"],
        "an answer added a hint to a line it did not ask about"
    );
    assert_eq!(hint_texts(&assembled, 4), ["end"]);
    // Asking for the lines again replaces them whole, as before.
    let again = ranged(
        SHOWN,
        (0, 2),
        vec![hint(5, ": i32"), hint(17, ": u8"), hint(18, ": u16")],
    );
    assert!(store.accept_hints(SHOWN, &text, unwrapped(again)));
    let replaced = blocks(&mut store, SHOWN);
    assert_eq!(hint_texts(&replaced, 0), [": i32"]);
    assert_eq!(hint_texts(&replaced, 1), [": u8", ": u16"]);
}

/// After an external change the previous rows' space is held above the lines it is mapped to:
///  hint space until
/// hints for those lines arrive,
///  message space until its time has passed,
///  and nothing after that.
#[test]
fn held_space_keeps_block_heights_until_hints_return_or_time_passes() {
    let text = Rope::from(TEXT);
    let start = Instant::now();
    let until = start + Duration::from_secs(1);
    let mut store = Annotations::default();
    let before = store.version();
    store.hold(
        SHOWN,
        vec![
            Held {
                line: 1,
                hints: ROW_HEIGHT,
                messages: 2.0 * ROW_HEIGHT,
            },
            Held {
                line: 3,
                hints: ROW_HEIGHT,
                messages: 0.0,
            },
        ],
        until,
    );
    assert_ne!(
        store.version(),
        before,
        "holding space must rebuild the map"
    );
    let held = blocks(&mut store, SHOWN);
    assert_eq!(held.len(), 2);
    assert_eq!(held[0].height(), BLOCK_GAP + 3.0 * ROW_HEIGHT);
    assert_eq!(held[1].height(), BLOCK_GAP + ROW_HEIGHT);
    assert!(
        held[0].hints.is_empty() && held[0].messages.is_empty(),
        "held space paints nothing"
    );
    // Held space belongs to one text only.
    let other = DocumentStamp {
        file: 3,
        revision: 8,
    };
    assert!(
        blocks(&mut store, other).is_empty(),
        "held space was applied to another text"
    );
    // Hints answering for lines 0 to 2 give up the hint space of line 1; its message space stays.
    let answer = ranged(SHOWN, (0, 3), vec![hint(5, ": i32")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(answer)));
    let answered = blocks(&mut store, SHOWN);
    let lines: Vec<usize> = block_lines(&answered);
    assert_eq!(lines, [0, 1, 3]);
    assert_eq!(
        answered[1].height(),
        BLOCK_GAP + 2.0 * ROW_HEIGHT,
        "hint space was not given up when hints for the line arrived"
    );
    assert_eq!(
        answered[2].height(),
        BLOCK_GAP + ROW_HEIGHT,
        "a line the answer did not cover lost its space"
    );
    // Diagnostics of the new text fill the held message space instead of adding to it.
    let found = diagnostics(
        SHOWN,
        vec![("rustc", vec![problem(11, 14, None, "one row")])],
    );
    assert!(store.accept_diagnostics(SHOWN, &text, found.expect("diagnostics snapshot")));
    let filled = blocks(&mut store, SHOWN);
    assert_eq!(filled[1].messages.len(), 1);
    assert_eq!(
        filled[1].height(),
        BLOCK_GAP + 2.0 * ROW_HEIGHT,
        "message space must stay held until its time has passed"
    );
    assert!(
        !store.expire(start + Duration::from_millis(999)),
        "space was given up early"
    );
    assert!(
        store.expire(until),
        "space was not given up when its time had passed"
    );
    let after = blocks(&mut store, SHOWN);
    let remaining: Vec<usize> = block_lines(&after);
    assert_eq!(remaining, [0, 1], "held space outlived its time");
    assert_eq!(after[1].height(), BLOCK_GAP + ROW_HEIGHT);
    assert!(!store.expire(until + Duration::from_secs(5)));
}

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

/// Handing back the same shared snapshots is recognized,
///  so a poll does not rebuild the index.
#[test]
fn identical_snapshots_are_recognized_by_identity() {
    let hint_snapshot = hints(SHOWN, vec![hint(5, ": i32")]);
    let problem_snapshot = diagnostics(SHOWN, vec![("rustc", vec![problem(4, 5, None, "x")])]);
    // `clone` on an `Option<Arc<...>>` copies the pointer, not the snapshot.
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &Rope::from(TEXT),
        hint_snapshot.clone(),
        problem_snapshot.clone(),
    );
    assert!(annotations.holds(&hint_snapshot, &problem_snapshot));
    let equal_copy = hints(SHOWN, vec![hint(5, ": i32")]);
    assert!(!annotations.holds(&equal_copy, &problem_snapshot));
    assert!(!annotations.holds(&None, &problem_snapshot));
    assert!(Annotations::default().holds(&None, &None));
}

/// The Language poll's entry points store only snapshots of the displayed text,
///  hand them out only for that
/// text,
///  keep what was held when a snapshot is refused,
///  and index accepted diagnostics for painting.
#[test]
fn accepting_stores_only_snapshots_of_the_displayed_text() {
    let text = Rope::from(TEXT);
    let older = DocumentStamp {
        file: 3,
        revision: 6,
    };
    let mut store = Annotations::default();
    assert!(
        !store.accept_hints(
            SHOWN,
            &text,
            unwrapped(hints(older, vec![hint(5, ": old")]))
        ),
        "hints for an earlier revision were stored"
    );
    assert!(
        !store.accept_diagnostics(SHOWN, &text, problems(older, "old")),
        "diagnostics for an earlier revision were stored"
    );
    assert!(store.hints(SHOWN).is_none() && store.diagnostics(SHOWN).is_none());
    assert!(store.accept_hints(
        SHOWN,
        &text,
        unwrapped(hints(SHOWN, vec![hint(5, ": i32")]))
    ));
    assert!(store.accept_diagnostics(SHOWN, &text, problems(SHOWN, "current")));
    assert!(
        store.hints(older).is_none(),
        "hints were handed out for another revision"
    );
    assert!(
        store.diagnostics(older).is_none(),
        "diagnostics were handed out for another revision"
    );
    assert_eq!(store.hints(SHOWN).expect("held hints").hints.len(), 1);
    assert_eq!(
        store
            .diagnostics(SHOWN)
            .expect("held diagnostics")
            .groups
            .len(),
        1
    );
    // Accepted diagnostics are indexed: the frame and the caret find them.
    assert_eq!(hint_texts(&blocks(&mut store, SHOWN), 0), [": i32"]);
    assert_eq!(
        store.marks(SHOWN, &text, 0, 5).len(),
        1,
        "accepted diagnostics were not indexed"
    );
    assert_eq!(store.at(SHOWN, 4)[0].message, "current");
    // A refused snapshot leaves the held ones, and their index, in place.
    assert!(!store.accept_diagnostics(SHOWN, &text, problems(older, "late")));
    assert!(!store.accept_hints(SHOWN, &text, unwrapped(hints(older, Vec::new()))));
    assert_eq!(store.at(SHOWN, 4)[0].message, "current");
    assert_eq!(hint_texts(&blocks(&mut store, SHOWN), 0), [": i32"]);
    // A newer accepted snapshot replaces the index and the message rows.
    assert!(store.accept_diagnostics(SHOWN, &text, problems(SHOWN, "replaced")));
    assert_eq!(store.at(SHOWN, 4).len(), 1);
    assert_eq!(store.at(SHOWN, 4)[0].message, "replaced");
    let rows = blocks(&mut store, SHOWN);
    assert_eq!(rows[0].messages.len(), 1);
    assert_eq!(rows[0].messages[0].text, "Error (rustc): replaced");
}

/// Another revision of the same file,
///  as an edit makes it.
const EDITED: DocumentStamp = DocumentStamp {
    file: 3,
    revision: 8,
};

/// Store the answer of `stamp` that asked about the lines `first` up to,
///  not including,
///  `last`.
fn answered(store: &mut Annotations, stamp: DocumentStamp, first: usize, last: usize) {
    let text = Rope::from(LONG);
    let items = vec![hint(text.line_to_char(first), ": T")];
    let snapshot = unwrapped(ranged(stamp, (first, last), items));
    assert!(store.accept_hints(stamp, &text, snapshot));
}

/// Store the answer of `stamp` that asked about the lines `first` up to,
///  not including,
///  `last`,
///  and put the hints `items` in it.
fn answered_with(
    store: &mut Annotations,
    stamp: DocumentStamp,
    asked: (usize, usize),
    items: Vec<InlayHint>,
) {
    let text = Rope::from(LONG);
    let snapshot = unwrapped(ranged(stamp, asked, items));
    assert!(store.accept_hints(stamp, &text, snapshot));
}

/// A hundred short lines,
///  so answers for the lines of a long file can hold hints.
const LONG: &str = "a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n";

/// The lines a stored answer asked about are covered,
///  the line before the first and the line the request ended before are not,
///  and an empty range is never covered.
#[test]
fn the_lines_a_stored_answer_asked_about_are_covered() {
    let mut store = Annotations::default();
    assert!(
        !store.answered_covers(SHOWN, 0..1),
        "lines were covered before any answer"
    );
    answered(&mut store, SHOWN, 35, 60);
    assert!(
        store.answered_covers(SHOWN, 35..60),
        "the asked lines are not covered"
    );
    assert!(
        store.answered_covers(SHOWN, 40..60),
        "lines inside the asked lines are not covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 34..60),
        "the line before the asked lines was covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 35..61),
        "the line the request ended before was covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 40..40),
        "an empty range was covered"
    );
}

/// Lines asked about by two answers are covered together when the answers overlap or touch,
///  and a gap between them is not covered.
#[test]
fn answers_that_overlap_or_touch_cover_their_union_but_not_a_gap() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 0, 30);
    answered(&mut store, SHOWN, 50, 80);
    assert!(
        !store.answered_covers(SHOWN, 20..60),
        "the gap between two answers was covered"
    );
    answered(&mut store, SHOWN, 30, 40);
    assert!(
        !store.answered_covers(SHOWN, 20..60),
        "a gap that two touching answers left was covered"
    );
    answered(&mut store, SHOWN, 38, 52);
    assert!(
        store.answered_covers(SHOWN, 20..60),
        "three overlapping answers do not cover the lines together"
    );
    assert!(store.answered_covers(SHOWN, 0..80));
    assert!(!store.answered_covers(SHOWN, 0..81));
}

/// Lines an answer of one revision asked about say nothing about another revision:
///  the other revision is not covered before any answer for it,
///  and its first answer drops what the earlier revision covered.
#[test]
fn covered_lines_belong_to_one_text_revision() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 35, 60);
    assert!(
        !store.answered_covers(EDITED, 40..60),
        "lines asked in one revision were covered in the next, before any answer for it"
    );
    answered(&mut store, EDITED, 0, 10);
    assert!(
        !store.answered_covers(EDITED, 40..60),
        "an answer for the next revision kept the lines the earlier revision had covered"
    );
    assert!(store.answered_covers(EDITED, 0..10));
    assert!(
        !store.answered_covers(SHOWN, 40..60),
        "the earlier revision stayed covered"
    );
}

/// A snapshot the store refuses (another revision than the displayed one) covers nothing,
///  and forgetting what was asked keeps the stored hints.
#[test]
fn a_refused_snapshot_covers_nothing_and_forgetting_keeps_the_hints() {
    let text = Rope::from(TEXT);
    let mut store = Annotations::default();
    let late = unwrapped(ranged(EDITED, (0, 5), Vec::new()));
    assert!(!store.accept_hints(SHOWN, &text, late));
    assert!(
        !store.answered_covers(SHOWN, 0..5) && !store.answered_covers(EDITED, 0..5),
        "a refused snapshot covered lines"
    );
    let first = unwrapped(ranged(SHOWN, (0, 5), vec![hint(5, ": i32")]));
    assert!(store.accept_hints(SHOWN, &text, first));
    assert!(store.answered_covers(SHOWN, 0..5));
    store.forget_answered();
    assert!(
        !store.answered_covers(SHOWN, 0..5),
        "forgetting what was asked left the lines covered"
    );
    assert_eq!(
        hint_texts(&blocks(&mut store, SHOWN), 0),
        [": i32"],
        "forgetting what was asked removed the stored hints"
    );
}

/// An answer that put no hint on the lines it asked about covers nothing:
///  a server that is still loading answers with no hint or with some,
///  and the view that settles over the same lines must ask again.
#[test]
fn an_answer_with_no_hint_on_its_lines_covers_nothing() {
    let mut store = Annotations::default();
    answered_with(&mut store, SHOWN, (35, 60), Vec::new());
    assert!(
        !store.answered_covers(SHOWN, 35..60) && !store.answered_covers(SHOWN, 40..41),
        "an answer with no hint covered the lines it asked about"
    );
    // A hint that stands on a line the answer did not ask about does not make the asked lines covered.
    let text = Rope::from(LONG);
    answered_with(
        &mut store,
        SHOWN,
        (35, 60),
        vec![hint(text.line_to_char(90), ": far")],
    );
    assert!(
        !store.answered_covers(SHOWN, 35..60),
        "a hint on an unasked line made the asked lines covered"
    );
    // The same lines answered with a hint on one of them are covered.
    answered_with(
        &mut store,
        SHOWN,
        (35, 60),
        vec![hint(text.line_to_char(40), ": near")],
    );
    assert!(
        store.answered_covers(SHOWN, 35..60),
        "an answer with a hint on its lines does not cover them"
    );
}

/// An answer replaces the hints of the lines it asked about,
///  so one that put no hint there takes those lines out of what earlier answers covered,
///  and the lines around them stay covered.
#[test]
fn an_empty_answer_uncovers_the_lines_it_replaced() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 35, 60);
    answered_with(&mut store, SHOWN, (40, 50), Vec::new());
    assert!(store.answered_covers(SHOWN, 35..40));
    assert!(store.answered_covers(SHOWN, 50..60));
    assert!(
        !store.answered_covers(SHOWN, 40..50) && !store.answered_covers(SHOWN, 35..60),
        "lines an empty answer replaced stayed covered"
    );
}

/// An empty answer takes out only the lines it asked about,
///  and leaves answers that lie apart from them covered.
#[test]
fn an_empty_answer_leaves_answers_apart_from_its_lines_covered() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 0, 30);
    answered(&mut store, SHOWN, 50, 80);
    answered_with(&mut store, SHOWN, (35, 45), Vec::new());
    assert!(store.answered_covers(SHOWN, 0..30));
    assert!(store.answered_covers(SHOWN, 50..80));
    assert!(
        !store.answered_covers(SHOWN, 30..35),
        "an empty answer made the lines before its own covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 45..50),
        "an empty answer made the lines after its own covered"
    );
}
