//! Injected hint and diagnostic snapshots through real key and pointer events:
//!  reading results,
//!  copying,
//!  and
//! find rectangles are those of the text without annotations,
//!  virtual rows are never source text,
//!  the problems
//! at the caret are described in full,
//!  and stale snapshots paint nothing while their space is held.

/// The production setter and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// Anchor and head of the reading selection.
use super::caret_tests::position;
/// The complete reader fixture and its key helpers.
use super::find_tests::{Reader, chord, key, reader, status_for, type_text};
/// Bounded waits and tree-row lookup shared with the navigation tests.
use super::navigation_tests::row;
/// The production window.
use super::{AppWindow, render};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Snapshot records exactly as the Language module builds them.
use ide_app::language::{
    diagnostics::{Diagnostic, DiagnosticsSnapshot, Freshness, Severity, SourceGroup},
    hints::{HintKind, HintsSnapshot, InlayHint},
    identity::{DocumentStamp, ServerIdentity},
};
/// One line's block of virtual rows and the heights it is built from.
use ide_app::virtual_row::{BLOCK_GAP, Block, ROW_HEIGHT};
/// What:
///  Window events as a seat delivers them,
///  and a point in logical window pixels.
/// Why:
///  Presses and keys go through the real `TouchArea`,
///  `FocusScope`,
///  and callbacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LogicalPosition, Model, PointerEventButton, WindowEvent, Key } from 'slint';
/// ```
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
/// What:
///  Disposable fixtures,
///  shared snapshot pointers,
///  and `Duration` for held space.
/// Why:
///  Held space is given up after a time the test waits out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mkdtempSync } from 'node:fs';
/// ```
use std::{fs, sync::Arc, time::Duration};

/// Where source text starts in the window,
///  derived in `sidebar_tests` from the tree,
///  divider,
///  and gutter widths.
pub(super) use super::sidebar_tests::TEXT_LEFT;
/// Source rows start below the 32 px file label.
pub(super) const TEXT_TOP: f32 = 32.0;

/// Three lines:
///  a call with hints and an error,
///  a line with a tab,
///  CJK,
///  a combining mark,
///  and a ligature,
/// and a last line;
///  the last terminator ends the text.
pub(super) const FIXTURE: &str =
    "let total = area(2, 3);\n\tlet 猫 = e\u{301} != x;\nfn main() {}\n";

/// The process every injected record names.
fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted".to_string(),
        instance: 1,
    };
}

/// One diagnostic of `severity` over `start..end`.
pub(super) fn problem(
    start: usize,
    end: usize,
    severity: Severity,
    code: &str,
    message: &str,
) -> Diagnostic {
    return Diagnostic {
        start,
        end,
        severity: Some(severity),
        code: Some(code.to_string()),
        message: message.to_string(),
        server: server(),
        freshness: Freshness::Versioned,
    };
}

/// One hint at `position`.
pub(super) fn hint(position: usize, label: &str) -> InlayHint {
    return InlayHint {
        position,
        label: label.to_string(),
        kind: Some(HintKind::Type),
        padding_left: false,
        padding_right: true,
        server: server(),
    };
}

/// What:
///  The fixture's snapshots for `stamp`:
///  hints after `total`,
///  before `2`,
///  and after `猫`;
///  an error over the
///       call,
///  a warning on `猫`,
///  a hint over `!=`,
///  a warning from `x` onto the next line,
///  and a point at the
///       end of the last line.
///  The pair is (hints,
///  diagnostics).
/// Why:
///  Every test injects the same realistic mix of overlapping,
///  multi-line,
///  CJK,
///  and point ranges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshots(stamp: DocumentStamp): [HintsSnapshot, DiagnosticsSnapshot];
/// ```
pub(super) fn snapshots(
    stamp: DocumentStamp,
) -> (Option<Arc<HintsSnapshot>>, Option<Arc<DiagnosticsSnapshot>>) {
    // Line starts: 0, 24, 42, 55. Line 1: tab(24) l e t space 猫(29) space =(31) space e(33) U+0301 space !(36) =(37) space x(39) ;(40).
    let hints = HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: 4,
        hints: vec![hint(9, ": u32"), hint(17, "width:"), hint(30, ": char")],
    };
    let rustc = SourceGroup {
        source: "rustc".to_string(),
        items: vec![
            problem(
                12,
                22,
                Severity::Error,
                "E0308",
                "mismatched types\nexpected `u32`, found `&str`",
            ),
            problem(29, 30, Severity::Warning, "W1", "unused variable `猫`"),
            problem(36, 38, Severity::Hint, "H1", "consider `==`"),
            problem(39, 44, Severity::Warning, "W2", "statement continues"),
        ],
    };
    let analyzer = SourceGroup {
        source: "rust-analyzer".to_string(),
        items: vec![problem(
            54,
            54,
            Severity::Information,
            "I1",
            "missing trailing comment",
        )],
    };
    let diagnostics = DiagnosticsSnapshot {
        stamp,
        groups: vec![analyzer, rustc],
    };
    return (Some(Arc::new(hints)), Some(Arc::new(diagnostics)));
}

/// Install the fixture's snapshots for the text the reader displays now.
pub(super) fn annotate(reader: &Reader) {
    let stamp = displayed(&reader.source.borrow());
    let (hints, diagnostics) = snapshots(stamp);
    set_annotations(&reader.window, &reader.source, hints, diagnostics);
}

/// What:
///  Window point over the code row of source line `line`,
///  `x` logical pixels into the text,
///  in the middle
///       of the row.
///  The row's place comes from the reader's vertical mapping.
/// Why:
///  With virtual rows above lines,
///  a line's code row is no longer at its number times one row height.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function point(reader: Reader, line: number, x: number): LogicalPosition;
/// ```
pub(super) fn point(reader: &Reader, line: usize, x: f32) -> LogicalPosition {
    let top = reader.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top + 12.0 + reader.window.get_scroll_y(),
    );
}

/// What:
///  Window point `rise` logical pixels above the top of line `line`'s code row:
///  inside its virtual rows.
/// Why:
///  Presses and pointer rests on hint and message rows are part of what the tests pin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function above(reader: Reader, line: number, x: number, rise: number): LogicalPosition;
/// ```
pub(super) fn above(reader: &Reader, line: usize, x: f32, rise: f32) -> LogicalPosition {
    let top = reader.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top - rise + reader.window.get_scroll_y(),
    );
}

/// The block of virtual rows above `line`,
///  or nothing;
///  `Option<Arc<Block>>` is a shared block or nothing.
pub(super) fn block(reader: &Reader, line: usize) -> Option<Arc<Block>> {
    let source = reader.source.borrow();
    for candidate in &source.blocks {
        if candidate.line == line {
            // `Some(Arc::clone(...))` hands out another pointer to the same block.
            return Some(Arc::clone(candidate));
        }
    }
    return None;
}

/// What:
///  How many hint texts,
///  message-row texts,
///  and underline runs the displayed frame paints;
///  a triple.
/// Why:
///  Hints and messages are pixels of the source image now;
///  the frame's positioned records say what was drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function painted(reader: Reader): [hints: number, messages: number, underlines: number];
/// ```
pub(super) fn painted(reader: &Reader) -> (usize, usize, usize) {
    let source = reader.source.borrow();
    // `let ... else` answers zeros when no frame or no annotations of it exist.
    let Some(view) = source.shaped.as_ref() else {
        return (0, 0, 0);
    };
    let Some(frame) = view.annotations.as_ref() else {
        return (0, 0, 0);
    };
    let mut hints = 0;
    let mut messages = 0;
    for text in &frame.texts {
        if text.severity.is_some() {
            messages += 1;
        } else {
            hints += 1;
        }
    }
    return (hints, messages, frame.underlines.len());
}

/// The top of the code row of each of the first three lines.
fn code_tops(reader: &Reader) -> Vec<f32> {
    let mut tops: Vec<f32> = Vec::new();
    for line in [0_usize, 1, 2] {
        tops.push(reader.source.borrow().row_map.code_top(line));
    }
    return tops;
}

/// Press at `start` after a click on line 1 (so the press is no double click),
///  move to `target`,
///  release,
///  and return the caret and selection end.
fn drag_to(reader: &Reader, start: LogicalPosition, target: LogicalPosition) -> (usize, usize) {
    let window = &reader.window;
    click(window, point(reader, 1, 300.0));
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: start,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: target });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: target,
            button: PointerEventButton::Left,
        });
    return position(reader);
}

/// Hand a hints snapshot to the store the way the poll does,
///  returning whether it was accepted.
fn accept_hints(reader: &Reader, stamp: DocumentStamp, snapshot: Arc<HintsSnapshot>) -> bool {
    let mut source = reader.source.borrow_mut();
    let text = source.document.text().clone();
    return source.annotations.accept_hints(stamp, &text, snapshot);
}

/// Hand a diagnostics snapshot to the store the way the poll does,
///  returning whether it was accepted.
fn accept_diagnostics(
    reader: &Reader,
    stamp: DocumentStamp,
    snapshot: Arc<DiagnosticsSnapshot>,
) -> bool {
    let mut source = reader.source.borrow_mut();
    let text = source.document.text().clone();
    return source
        .annotations
        .accept_diagnostics(stamp, &text, snapshot);
}

/// One complete left click at a window point.
pub(super) fn click(window: &AppWindow, position: LogicalPosition) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

/// What:
///  Drive one fixed script of clicks,
///  double clicks,
///  keys,
///  copying,
///  and find,
///  recording every resulting
///       selection,
///  copied text,
///  and find rectangle as text;
///  `Vec<String>` is the ordered record.
/// Why:
///  The same script on a reader with annotations and one without must produce the same record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function script(reader: Reader): string[];
/// ```
fn script(reader: &Reader) -> Vec<String> {
    let window = &reader.window;
    let mut record = Vec::new();
    // Rows alternate, so no two consecutive clicks form a double click; 600 px is past the end of every line.
    for x in [0.0, 4.0, 13.0, 37.5, 61.0, 120.0, 205.0, 260.0, 600.0] {
        for line in [0, 1, 2] {
            click(window, point(reader, line, x));
            record.push(format!("click {line} {x}: {:?}", position(reader)));
        }
    }
    // Double clicks on a word, on CJK, on the ligature, and past the end of the text.
    for (line, x) in [(1, 30.0), (1, 52.0), (1, 135.0), (0, 600.0)] {
        click(window, point(reader, 2, 1.0));
        click(window, point(reader, line, x));
        click(window, point(reader, line, x));
        record.push(format!("double {line} {x}: {:?}", position(reader)));
    }
    chord(window, Key::Control, Key::Home);
    for step in 0..30 {
        key(window, Key::RightArrow);
        record.push(format!("right {step}: {:?}", position(reader)));
    }
    for name in ["down", "up", "end", "down", "home", "word"] {
        if name == "down" {
            key(window, Key::DownArrow);
        } else if name == "up" {
            key(window, Key::UpArrow);
        } else if name == "end" {
            key(window, Key::End);
        } else if name == "home" {
            key(window, Key::Home);
        } else {
            chord(window, Key::Control, Key::RightArrow);
        }
        record.push(format!("{name}: {:?}", position(reader)));
    }
    // Shift+End selects to the line end; the copied text is the toolkit's clipboard source.
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(window, Key::End);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    record.push(format!("copy: {:?}", window.get_selected_text()));
    chord(window, Key::Control, "a");
    record.push(format!("copy all: {:?}", window.get_selected_text()));
    chord(window, Key::Control, "f");
    type_text(window, "e");
    status_for(window, "e", "1/4");
    let matches = window.get_source_matches();
    for index in 0..matches.row_count() {
        let rect = matches.row_data(index).expect("match rectangle");
        // A match rectangle is recorded by its line and its place inside the code row, which annotations
        // must not change; where the row is in the text is the vertical mapping's business.
        let place = reader.source.borrow().row_map.locate(rect.y);
        let within = rect.y - reader.source.borrow().row_map.code_top(place.line);
        record.push(format!(
            "match {}: line {} in block {} at {} + {} {} {}",
            index, place.line, place.in_block, rect.x, within, rect.width, rect.height
        ));
    }
    key(window, Key::Escape);
    return record;
}

/// Caret placement,
///  clicks past the text,
///  word selection,
///  keys,
///  copying,
///  and find rectangles give the same
/// results with annotations as without them;
///  only where the code rows are differs.
#[test]
fn annotations_leave_reading_geometry_copy_and_find_unchanged() {
    let plain_fixture = tempfile::tempdir().expect("disposable plain project");
    fs::write(plain_fixture.path().join("main.rs"), FIXTURE).expect("plain fixture");
    let plain = reader(plain_fixture.path(), "main.rs");
    let annotated_fixture = tempfile::tempdir().expect("disposable annotated project");
    fs::write(annotated_fixture.path().join("main.rs"), FIXTURE).expect("annotated fixture");
    let annotated = reader(annotated_fixture.path(), "main.rs");
    annotate(&annotated);
    let (hints, messages, underlines) = painted(&annotated);
    assert!(
        underlines >= 3,
        "the injected diagnostics were not marked (positive control)"
    );
    assert_eq!(hints, 3, "the injected hints were not painted");
    assert_eq!(messages, 6, "one row per message line was expected");
    // Line 0: one hint row and two message rows; line 1: one hint row and three messages; line 2: one message.
    let mut heights: Vec<f32> = Vec::new();
    for line in [0, 1, 2] {
        heights.push(block(&annotated, line).expect("block").height());
    }
    assert_eq!(
        heights,
        [
            BLOCK_GAP + 3.0 * ROW_HEIGHT,
            BLOCK_GAP + 4.0 * ROW_HEIGHT,
            BLOCK_GAP + ROW_HEIGHT
        ]
    );
    let expected = script(&plain);
    let actual = script(&annotated);
    for (before, after) in expected.iter().zip(actual.iter()) {
        assert_eq!(before, after, "annotations changed a reading result");
    }
    assert_eq!(expected.len(), actual.len());
    // A click past the end of line 0 lands on that line's end, as it does without annotations.
    assert!(actual.contains(&"click 0 260: (23, 23)".to_string()));
    assert!(actual.contains(&"click 0 600: (23, 23)".to_string()));
    let mut copied = None;
    for line in &actual {
        if line.starts_with("copy all") {
            copied = Some(line);
            break;
        }
    }
    assert_eq!(
        copied,
        Some(&format!("copy all: {FIXTURE:?}")),
        "copying included hint or message text"
    );
    // Accessibility reads the source text only.
    assert_eq!(annotated.window.get_source_text(), FIXTURE);
}

/// Virtual rows are not source text.
///  A press or a drag on them acts on the code row they belong to,
///  the one
/// beneath them;
///  the caret and selection rectangles are only ever on code rows;
///  find does not match their text.
#[test]
fn virtual_rows_are_not_source_text() {
    let fixture = tempfile::tempdir().expect("disposable virtual-row project");
    fs::write(fixture.path().join("main.rs"), FIXTURE).expect("virtual-row fixture");
    let reader = reader(fixture.path(), "main.rs");
    let window = &reader.window;
    annotate(&reader);
    // A press anywhere in a line's block, its gap included, is a press on that line at the same x.
    for line in [0_usize, 1, 2] {
        let height = block(&reader, line).expect("block").height();
        for x in [0.0, 37.5, 120.0, 600.0] {
            // The neighboring line first, so no two presses form a double click.
            click(window, point(&reader, (line + 1) % 3, 300.0));
            click(window, point(&reader, line, x));
            let on_code = position(&reader);
            for rise in [1.0, ROW_HEIGHT / 2.0, height / 2.0, height - 1.0] {
                click(window, point(&reader, (line + 1) % 3, 300.0));
                click(window, above(&reader, line, x, rise));
                assert_eq!(
                    position(&reader),
                    on_code,
                    "a press {rise} px above line {line} at {x} did not act on that line"
                );
            }
        }
    }
    // A drag that ends on line 2's virtual rows extends the selection to line 2, not to line 1 above them.
    let start = point(&reader, 0, 13.0);
    let over_rows = above(&reader, 2, 61.0, ROW_HEIGHT / 2.0);
    let onto_code = drag_to(&reader, start, point(&reader, 2, 61.0));
    let onto_rows = drag_to(&reader, start, over_rows);
    assert_eq!(
        onto_rows, onto_code,
        "a drag onto virtual rows did not extend to their line"
    );
    assert!(
        onto_rows.1 >= 42,
        "the drag did not reach line 2: {onto_rows:?}"
    );
    // The selection is painted on code rows only: every rectangle is one code row of one line.
    chord(window, Key::Control, "a");
    let selections = window.get_source_selections();
    assert!(selections.row_count() >= 3);
    for index in 0..selections.row_count() {
        let rect = selections.row_data(index).expect("selection rectangle");
        let source = reader.source.borrow();
        let place = source.row_map.locate(rect.y);
        assert!(
            !place.in_block,
            "a selection rectangle starts in virtual rows at {}",
            rect.y
        );
        assert_eq!(rect.y, source.row_map.code_top(place.line));
        assert_eq!(rect.height, 24.0);
    }
    // The caret walks code rows: Down and Up move by source lines, however much lies between them.
    chord(window, Key::Control, Key::Home);
    for line in [0_usize, 1, 2, 3] {
        let source = reader.source.borrow();
        assert_eq!(
            window.get_caret_y(),
            source.row_map.code_top(line) + 2.0,
            "the caret is not on line {line}'s code row"
        );
        drop(source);
        key(window, Key::DownArrow);
    }
    // Find matches source text only: words that only hints and messages contain are not found.
    chord(window, Key::Control, "f");
    type_text(window, "mismatched");
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_find_status() == "No matches") {
            waiting.step("text of a message row was matched");
        }
    }
    assert_eq!(window.get_source_matches().row_count(), 0);
    key(window, Key::Escape);
}

/// The problems at the caret are named in full,
///  worst first,
///  for the source view's accessible description;
/// the text is empty elsewhere and follows the caret into a CJK range and onto a point at a line end.
/// Nothing is drawn for it:
///  every message already stands above its line.
#[test]
fn caret_problems_follow_the_caret_and_are_spelled_out_in_full() {
    let fixture = tempfile::tempdir().expect("disposable problem project");
    fs::write(fixture.path().join("main.rs"), FIXTURE).expect("problem fixture");
    let reader = reader(fixture.path(), "main.rs");
    let window = &reader.window;
    annotate(&reader);
    chord(window, Key::Control, Key::Home);
    assert_eq!(window.get_caret_problems(), "");
    for _ in 0..12 {
        key(window, Key::RightArrow);
    }
    assert_eq!(position(&reader), (12, 12));
    assert_eq!(
        window.get_caret_problems(),
        "Error E0308 (rustc): mismatched types\nexpected `u32`, found `&str`"
    );
    key(window, Key::End);
    assert_eq!(
        window.get_caret_problems(),
        "",
        "the problems stayed after leaving the range"
    );
    key(window, Key::DownArrow);
    // Home goes to the first non-blank character of the line (past its tab) and a second Home to the line start.
    key(window, Key::Home);
    assert_eq!(position(&reader), (25, 25));
    key(window, Key::Home);
    for _ in 0..5 {
        key(window, Key::RightArrow);
    }
    assert_eq!(position(&reader), (29, 29));
    assert_eq!(
        window.get_caret_problems(),
        "Warning W1 (rustc): unused variable `猫`"
    );
    key(window, Key::DownArrow);
    key(window, Key::End);
    assert_eq!(position(&reader), (54, 54));
    assert_eq!(
        window.get_caret_problems(),
        "Information I1 (rust-analyzer): missing trailing comment"
    );
    // The rows above the lines say the same, whether or not the caret is there.
    let first = block(&reader, 0).expect("line 0 block");
    assert_eq!(
        first.messages[0].text,
        "Error E0308 (rustc): mismatched types"
    );
    assert_eq!(first.messages[1].text, "expected `u32`, found `&str`");
    assert!(first.messages[1].continued);
    let second = block(&reader, 1).expect("line 1 block");
    let mut order: Vec<&str> = Vec::new();
    for row in &second.messages {
        order.push(row.text.as_str());
    }
    assert_eq!(
        order,
        [
            "Warning W1 (rustc): unused variable `猫`",
            "Warning W2 (rustc): statement continues",
            "Hint H1 (rustc): consider `==`",
        ],
        "messages above a line are listed worst first, then in source order"
    );
}

/// After an external change and after a file switch,
///  the old snapshots paint nothing and name no problem until
/// snapshots for the new text arrive.
///  After a change the rows' space stays open,
///  so lines keep their place;
///  it
/// is given up when its time has passed without new annotations.
#[test]
fn stale_snapshots_disappear_after_reload_and_file_switch() {
    let fixture = super::find_tests::memory_project("ide-stale-annotations-");
    let path = fixture.path().join("main.rs");
    fs::write(&path, FIXTURE).expect("stale fixture");
    fs::write(fixture.path().join("other.rs"), "fn other() {}\n").expect("second fixture");
    let reader = reader(fixture.path(), "main.rs");
    let window = &reader.window;
    // The external changes below reach the reader through its live watch, not the sweep.
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().refresh.is_watched()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    annotate(&reader);
    for _ in 0..12 {
        key(window, Key::RightArrow);
    }
    assert!(painted(&reader).1 > 0);
    assert_ne!(window.get_caret_problems(), "");
    let tops = code_tops(&reader);
    let revision = reader.source.borrow().document.revision();
    // The change appends a line, so every annotated line keeps its number.
    super::find_tests::replace_file(&path, &format!("{FIXTURE}// changed\n"));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().document.revision() > revision) {
            waiting.step("the external change was not reloaded");
        }
    }
    let (hints, messages, underlines) = painted(&reader);
    assert_eq!(underlines, 0, "stale markers were painted after reload");
    assert_eq!(messages, 0, "stale messages were painted after reload");
    assert_eq!(hints, 0, "stale hints were painted after reload");
    assert_eq!(
        window.get_caret_problems(),
        "",
        "stale problems were named after reload"
    );
    let held = code_tops(&reader);
    assert_eq!(held, tops, "the rows' space was not held across the reload");
    annotate(&reader);
    assert!(painted(&reader).1 > 0, "current snapshots were not painted");
    let returned = code_tops(&reader);
    assert_eq!(returned, tops, "returning annotations moved a line");
    // A second change with no annotations for its text: the space is held, then given up.
    let second = reader.source.borrow().document.revision();
    super::find_tests::replace_file(&path, &format!("{FIXTURE}// changed again\n"));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().document.revision() > second) {
            waiting.step("the second external change was not reloaded");
        }
    }
    assert_eq!(reader.source.borrow().row_map.code_top(2), tops[2]);
    let since = std::time::Instant::now();
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().row_map.code_top(2) == 48.0) {
            waiting.step("held space was never given up");
        }
    }
    let waited = since.elapsed();
    assert!(
        waited >= Duration::from_millis(700) && waited <= Duration::from_millis(2000),
        "held space was given up after {waited:?}, not after about one second"
    );
    annotate(&reader);
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "other.rs").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    let generation = reader.source.borrow().file_generation;
    window.invoke_tree_activate(row(window, "other.rs").expect("second file row"));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().file_generation > generation) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(
        painted(&reader),
        (0, 0, 0),
        "annotations of the previous file were painted"
    );
    assert_eq!(
        reader.source.borrow().row_map.height(),
        48.0,
        "rows of the previous file kept space in another file"
    );
}

/// The Language poll's path:
///  snapshots accepted into the state's store,
///  hints and diagnostics independently,
/// show after one render,
///  and a snapshot for other text is refused and changes nothing.
#[test]
fn accepted_snapshots_show_after_one_render() {
    let fixture = tempfile::tempdir().expect("disposable accept project");
    fs::write(fixture.path().join("main.rs"), FIXTURE).expect("accept fixture");
    let reader = reader(fixture.path(), "main.rs");
    let window = &reader.window;
    let stamp = displayed(&reader.source.borrow());
    let (hints, diagnostics) = snapshots(stamp);
    let stale = DocumentStamp {
        file: stamp.file,
        revision: stamp.revision + 1,
    };
    let (stale_hints, stale_diagnostics) = snapshots(stale);
    // What: `borrow_mut` lends the state for writing, as the poll holds it while accepting; the closure
    //       takes the store and the displayed text out of it together.
    // Why: The store's answer tells the poll whether a render is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const accepted = state.current.annotations.acceptHints(stamp, text, hints);
    // ```
    let refused = accept_hints(&reader, stamp, stale_hints.expect("stale hints"));
    assert!(!refused, "hints for another revision were accepted");
    assert!(accept_hints(&reader, stamp, hints.expect("hints")));
    assert_eq!(
        painted(&reader).0,
        0,
        "accepting must not draw before the render the poll requests"
    );
    render(window, &reader.source);
    assert_eq!(painted(&reader).0, 3, "accepted hints were not drawn");
    assert_eq!(painted(&reader).2, 0);
    assert!(accept_diagnostics(
        &reader,
        stamp,
        diagnostics.expect("diagnostics")
    ));
    render(window, &reader.source);
    assert!(
        painted(&reader).2 >= 3,
        "accepted diagnostics were not drawn"
    );
    assert_eq!(painted(&reader).1, 6, "accepted messages were not drawn");
    assert_eq!(
        painted(&reader).0,
        3,
        "accepting diagnostics dropped the hints"
    );
    let late = accept_diagnostics(
        &reader,
        stamp,
        stale_diagnostics.expect("stale diagnostics"),
    );
    assert!(!late, "diagnostics for another revision were accepted");
    render(window, &reader.source);
    assert!(
        painted(&reader).2 >= 3,
        "a refused snapshot removed the marks"
    );
}
