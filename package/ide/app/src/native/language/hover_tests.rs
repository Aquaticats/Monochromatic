//! Hover by Ctrl+Q and by the resting pointer:
//!  content,
//!  placement,
//!  and every dismissal.

/// Fixtures and real key and pointer events.
use super::test_support::{
    LanguageReader, caret, caret_x, definitions, escape, hover, idle, move_to, point, popup,
    project, reader, ready,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Key events and a tree-row lookup shared with the other window tests.
use crate::native::{find_tests::key, navigation_tests::row};
/// Encoded keys and window points.
use slint::{LogicalPosition, platform::Key};

/// Forty lines:
///  words on even lines,
///  blank odd lines.
fn sample() -> String {
    let mut text = String::new();
    for index in 0..40 {
        if index % 2 == 0 {
            text.push_str(&format!("alpha{index:02} beta\n"));
        } else {
            text.push('\n');
        }
    }
    return text;
}

/// Character offset of the start of `line` in `sample()`:
///  even lines have 13 characters.
fn start(line: usize) -> usize {
    return (line / 2) * 14 + (line % 2) * 13;
}

/// A reader of `sample()` with the default scripted hover answers.
fn sample_reader() -> (super::test_support::Project, LanguageReader) {
    let fixture = project(&[("main.scripted", &sample()), ("other.scripted", "other\n")]);
    let opened = reader(&fixture, "main.scripted", definitions(&[], None));
    ready(&opened);
    return (fixture, opened);
}

/// Ctrl+Q shows the server's text below the caret line;
///  Escape removes it.
#[test]
fn control_q_shows_hover_below_the_caret_line_and_escape_dismisses_it() {
    let (_fixture, reader) = sample_reader();
    caret(&reader, start(2) + 8);
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader) == "line=alpha02 beta char=b") {
            waiting.step("the hover did not appear");
        }
    }
    assert!(!reader.window.get_language_popup_note());
    assert!(reader.window.get_language_popup_below());
    let top = reader.window.get_language_anchor_top();
    assert!(
        reader.window.get_language_popup_y() >= top + 24.0,
        "the hover covers its line"
    );
    escape(&reader);
    assert_eq!(popup(&reader), "");
    assert!(reader.window.get_source_has_focus());
}

/// Near the bottom of the view the hover goes above its line instead of covering it.
#[test]
fn hover_near_the_bottom_is_placed_above_its_line() {
    let (_fixture, reader) = sample_reader();
    let lines = (reader.window.get_viewport_height() / 24.0).floor() as usize;
    // The last fully visible even line.
    let line = (lines - 1) & !1;
    caret(&reader, start(line) + 1);
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).starts_with("line=alpha")) {
            waiting.step("the hover did not appear");
        }
    }
    assert!(!reader.window.get_language_popup_below());
    let bottom = reader.window.get_language_popup_y() + reader.window.get_language_popup_height();
    assert!(
        bottom <= reader.window.get_language_anchor_top(),
        "the hover covers its line"
    );
}

/// Caret movement,
///  scrolling,
///  and a file switch each dismiss the hover.
#[test]
fn hover_is_dismissed_by_caret_movement_scrolling_and_a_file_switch() {
    let (_fixture, reader) = sample_reader();
    {
        let mut waiting = Waiting::language();
        while !(row(&reader.window, "other.scripted").is_some()) {
            waiting.step("the tree did not show the files");
        }
    }
    caret(&reader, start(0) + 2);
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the hover did not appear");
        }
    }
    key(&reader.window, Key::RightArrow);
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("caret movement did not dismiss the hover");
        }
    }
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the hover did not appear again");
        }
    }
    reader.window.set_scroll_y(-48.0);
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("scrolling did not dismiss the hover");
        }
    }
    reader.window.set_scroll_y(0.0);
    caret(&reader, start(0) + 2);
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the hover did not appear a third time");
        }
    }
    reader
        .window
        .invoke_tree_activate(row(&reader.window, "other.scripted").expect("other row"));
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("the file switch did not dismiss the hover");
        }
    }
}

/// A reload of the displayed file dismisses the hover,
///  whose positions describe the old text.
#[test]
fn hover_is_dismissed_when_the_file_reloads() {
    let (fixture, reader) = sample_reader();
    caret(&reader, start(0) + 2);
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the hover did not appear");
        }
    }
    let revision = reader.source.borrow().document.revision();
    // The change lies after the caret, so neither the caret nor the view moves.
    std::fs::write(fixture.root.join("main.scripted"), sample() + "appended\n")
        .expect("external change");
    {
        let mut waiting = Waiting::language();
        while !(reader.source.borrow().document.revision() != revision) {
            waiting.step("the reload did not arrive");
        }
    }
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("the reload did not dismiss the hover");
        }
    }
}

/// A resting pointer shows the hover of the character under it;
///  leaving the source hides it,
/// and so does resting where there is no character.
#[test]
fn resting_pointer_shows_hover_and_leaving_or_resting_on_nothing_hides_it() {
    let (_fixture, reader) = sample_reader();
    let x = caret_x(&reader, start(4) + 8) + 2.0;
    move_to(&reader.window, point(&reader, 4, x));
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader) == "line=alpha04 beta char=b") {
            waiting.step("the pointer hover did not appear");
        }
    }
    move_to(&reader.window, LogicalPosition::new(100.0, 300.0));
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("leaving the source did not hide the hover");
        }
    }
    move_to(&reader.window, point(&reader, 4, x));
    {
        let mut waiting = Waiting::language();
        while !(!popup(&reader).is_empty()) {
            waiting.step("the pointer hover did not appear again");
        }
    }
    move_to(&reader.window, point(&reader, 4, 600.0));
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("resting past the line's end did not hide the hover");
        }
    }
    // A blank line has no character: resting there asks nothing.
    move_to(&reader.window, point(&reader, 5, 4.0));
    idle(600);
    assert_eq!(popup(&reader), "");
}

/// A resting pointer on a file whose server program is missing explains why hover is unavailable,
///  once for the displayed file;
///  the explanation ends like hover content and is drawn as a note.
#[test]
fn resting_pointer_explains_a_missing_server_once_for_each_file() {
    let fixture = project(&[("main.scripted", &sample()), ("other.scripted", "other\n")]);
    let languages = definitions(&[], Some("/definitely/missing/ide-scripted-lsp"));
    let reader = reader(&fixture, "main.scripted", languages);
    let x = caret_x(&reader, start(4) + 8) + 2.0;
    move_to(&reader.window, point(&reader, 4, x));
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader)
            .starts_with("Hover information needs scripted-ls, which is not available:"))
        {
            waiting.step("the resting pointer did not explain the missing server");
        }
    }
    assert!(reader.window.get_language_popup_note());
    move_to(&reader.window, LogicalPosition::new(100.0, 300.0));
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader).is_empty()) {
            waiting.step("leaving the source did not end the explanation");
        }
    }
    // The same file is not explained again.
    move_to(&reader.window, point(&reader, 4, x));
    idle(800);
    assert_eq!(popup(&reader), "", "the same file was explained twice");
}

/// Ctrl+Q where the server has nothing explains that;
///  a resting pointer stays silent.
#[test]
fn empty_hover_is_explained_for_ctrl_q_only() {
    let (_fixture, reader) = sample_reader();
    caret(&reader, start(1));
    hover(&reader);
    {
        let mut waiting = Waiting::language();
        while !(popup(&reader) == "No hover information at this position.") {
            waiting.step("the empty hover was not explained");
        }
    }
    assert!(reader.window.get_language_popup_note());
    escape(&reader);
    assert_eq!(popup(&reader), "");
}

/// A resting pointer's late answer never replaces the location list opened meanwhile.
#[test]
fn pointer_hover_answer_never_replaces_the_location_list() {
    let fixture = project(&[("main.scripted", &sample())]);
    let main = super::test_support::address(&fixture.root.join("main.scripted"));
    let answer = format!(
        "[{},{}]",
        super::test_support::location(&main, 2, 0, 5),
        super::test_support::location(&main, 4, 0, 5)
    );
    let languages = definitions(&[("DEFINITION", &answer), ("HOVER_DELAY_MS", "700")], None);
    let reader = reader(&fixture, "main.scripted", languages);
    ready(&reader);
    let x = caret_x(&reader, start(6) + 2) + 2.0;
    move_to(&reader.window, point(&reader, 6, x));
    // The rest time passes and the delayed hover request is on its way.
    idle(450);
    super::test_support::definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_references_open()) {
            waiting.step("the definitions list did not open");
        }
    }
    idle(1200);
    assert!(
        reader.window.get_references_open(),
        "a pointer hover answer replaced the location list"
    );
    assert_eq!(
        popup(&reader),
        "",
        "a pointer hover answer replaced the location list"
    );
}
