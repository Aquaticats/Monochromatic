//! The language poll and the annotation renderer working together through real window events:
//! hints and diagnostics from the server are painted by the poll that stored them,
//!  the language popup
//! covers neither its line nor that line's virtual rows,
//!  and resting or Ctrl+clicking on a virtual row
//! asks the server nothing.

/// Fixtures,
///  the scripted server,
///  and real key and pointer events.
use super::test_support::{
    TEXT_TOP, above, address, caret, caret_x, control_click, definitions, hover, idle, location,
    move_to, point, popup, project, reader, ready, rows,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// The stamp of the displayed text and the production repaint.
use crate::native::{annotate::displayed, render};
/// One hint record and its snapshot,
///  exactly as the Language module builds them.
use ide_app::language::{
    hints::{HintKind, HintsSnapshot, InlayHint},
    identity::ServerIdentity,
};
/// Snapshots arrive behind shared pointers;
///  a scroll has a moment.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Two lines.
///  The scripted server warns about the first character of the first line
/// and returns one hint after `alpha`.
const TEXT: &str = "alpha beta\ngamma delta\n";

/// What:
///  The scripted server's setting that makes it push its warning on every publish;
///       `&[(&str, &str)]` is a borrowed list of borrowed name and value pairs.
/// Why:
///  The shared test definitions turn pushed diagnostics off unless a test names `PUSH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LATE: [string, string][] = [['PUSH', '1'], ['INIT_DELAY_MS', '3000']];
/// ```
///
/// The pushed warning comes from a server that starts answering only three seconds after it was started.
/// The source refresh repaints once,
///  when the file's first highlighting answer arrives.
///  A server that
/// answers after that leaves the language poll as the only thing that can paint its hints and warning.
const LATE: &[(&str, &str)] = &[("PUSH", "1"), ("INIT_DELAY_MS", "3000")];

/// Forty lines of two words each,
///  so a line near the bottom of the view exists.
fn tall() -> String {
    let mut text = String::new();
    for index in 0..40 {
        text.push_str(&format!("alpha{index:02} beta\n"));
    }
    return text;
}

/// The server's hints and diagnostics appear with no key,
///  pointer,
///  scroll,
///  reload,
///  or highlighting
/// answer after them:
///  the poll that stores a snapshot also repaints the source.
#[test]
fn server_hints_and_diagnostics_are_painted_by_the_poll_that_stored_them() {
    let fixture = project(&[("main.scripted", TEXT)]);
    let reader = reader(&fixture, "main.scripted", definitions(LATE, None));
    // What: `borrow()` lends the shared source state for reading; `Some(revision)` is the present
    //       value the highlighting revision is compared with.
    // Why: The source refresh asks for highlighting until an answer for the displayed revision was
    //      applied, and repaints when it applies one. After this wait it repaints nothing more.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await eventually(() => source.current.syntaxRevision === source.current.document.revision());
    // ```
    {
        let mut waiting = Waiting::language();
        loop {
            {
                let current = reader.source.borrow();
                if current.syntax_revision == Some(current.document.revision()) {
                    break;
                }
            }
            waiting.step("the first highlighting answer was not applied");
        }
    }
    // The order this test depends on: nothing from the delayed server was painted by that repaint.
    assert!(
        rows(&reader) == (0, 0),
        "the delayed server answered before highlighting settled; the repaints cannot be told apart"
    );
    // What: `Some(...)` records the reader as having scrolled at a moment a minute from now; a moment not
    //       yet reached counts as no time passed, so the repaints on the source refresh timer, which wait
    //       out 200 ms after a scroll, wait for a minute.
    // Why: Since `rows::due`, that timer also paints a store that holds rows no frame shows yet, a tick
    //      after the store changed. Left running it would paint what a poll stored without repainting,
    //      and removing the poll's repaint would go unnoticed. This comes before the server is ready
    //      because the first snapshots can be stored while that wait is still running. The poll's own
    //      repaint is not held back: the view is at the top, so the rows move nothing visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // source.current.scrolledAt = now() + 60_000;
    // ```
    reader.source.borrow_mut().scrolled_at = Some(Instant::now() + Duration::from_secs(60));
    ready(&reader);
    {
        let mut waiting = Waiting::language();
        loop {
            {
                let (hints, messages) = rows(&reader);
                if hints > 0 && messages > 0 {
                    break;
                }
            }
            waiting.step("accepted hints and diagnostics were stored but never painted");
        }
    }
    // Both stand above the first line, which therefore starts lower than a line without rows would.
    assert!(reader.source.borrow().row_map.code_top(0) > 0.0);
}

/// A popup placed above its line ends above that line's virtual rows too:
///  the hints and messages of the
/// line the popup is about stay readable beside it.
#[test]
fn hover_popup_above_a_line_does_not_cover_its_virtual_rows() {
    let fixture = project(&[("main.scripted", &tall())]);
    let reader = reader(&fixture, "main.scripted", definitions(&[], None));
    ready(&reader);
    {
        let mut waiting = Waiting::language();
        while !(rows(&reader).0 > 0) {
            waiting.step("the server's hint was not painted");
        }
    }
    // A hint for line 22, near the bottom of the view, through the poll's own entry point.
    let line = 22;
    let position = line * 13 + 7;
    let snapshot = Arc::new(HintsSnapshot {
        stamp: displayed(&reader.source.borrow()),
        first_line: line,
        last_line: line + 1,
        hints: vec![InlayHint {
            position,
            label: ": injected".to_string(),
            // `Some(...)` is a present kind.
            kind: Some(HintKind::Type),
            padding_left: false,
            padding_right: false,
            server: ServerIdentity {
                name: "test".to_string(),
                instance: 1,
            },
        }],
    });
    let accepted = {
        let mut source = reader.source.borrow_mut();
        let text = source.document.text().clone();
        let stamp = displayed(&source);
        source.annotations.accept_hints(stamp, &text, snapshot)
    };
    assert!(accepted);
    render(&reader.window, &reader.source);
    let (block_top, code_top) = {
        let source = reader.source.borrow();
        (
            source.row_map.block_top(line),
            source.row_map.code_top(line),
        )
    };
    assert!(
        code_top > block_top,
        "line {line} has no virtual rows (positive control)"
    );
    caret(&reader, position);
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the hover did not appear");
        }
    }
    assert!(
        !reader.window.get_language_popup_below(),
        "the popup fits below line {line}; the test needs a line near the bottom of the view"
    );
    let bottom = reader.window.get_language_popup_y() + reader.window.get_language_popup_height();
    let rows_top = TEXT_TOP + block_top + reader.window.get_scroll_y();
    assert!(
        bottom <= rows_top,
        "the hover covers the virtual rows of its line: it ends at {bottom}, the rows start at {rows_top}"
    );
}

/// A hint or message row is over no source character:
///  a pointer resting on it asks for no hover,
///  and
/// Ctrl+click on it asks for no definition,
///  while the code row beneath it answers both.
#[test]
fn resting_and_control_click_on_virtual_rows_ask_nothing() {
    let other_text = "first\n  second line\n";
    let fixture = project(&[
        ("main.scripted", TEXT),
        ("nested/other.scripted", other_text),
    ]);
    let other = address(&fixture.root.join("nested/other.scripted"));
    let answer = format!("[{}]", location(&other, 1, 2, 8));
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(&[("DEFINITION", &answer)], None),
    );
    ready(&reader);
    {
        let mut waiting = Waiting::language();
        while !(rows(&reader).0 > 0) {
            waiting.step("the server's hint was not painted");
        }
    }
    // Eight pixels above the first code row is inside the hint row; x is over `b` of `beta`.
    let x = caret_x(&reader, 6) + 2.0;
    move_to(&reader.window, above(&reader, 0, x, 8.0));
    idle(600);
    assert_eq!(popup(&reader), "", "resting on a hint row asked for hover");
    control_click(&reader.window, above(&reader, 0, x, 8.0));
    idle(600);
    assert_eq!(
        reader.window.get_source_text(),
        TEXT,
        "Ctrl+click on a hint row asked for a definition"
    );
    assert_eq!(popup(&reader), "");
    // Positive controls on the code row beneath: the same x answers both.
    move_to(&reader.window, point(&reader, 0, x));
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the pointer hover on the code row did not appear");
        }
    }
    control_click(&reader.window, point(&reader, 0, x));
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_source_text() == other_text) {
            waiting.step("Ctrl+click on the code row did not open the definition");
        }
    }
}
