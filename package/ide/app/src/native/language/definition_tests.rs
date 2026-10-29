//! Go to definition and references through real window events with the scripted server.

/// Fixtures and real key and pointer events.
use super::test_support::{
    LanguageReader, address, caret, caret_x, click, control_click, definition, definitions, escape,
    head, location, point, project, reader, ready,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Key events for the list and a tree-row lookup shared with the navigation tests.
use crate::native::{find_tests::key, navigation_tests::row};
/// Toolkit models,
///  encoded keys,
///  and window handles.
use slint::{LogicalPosition, Model, SharedString, platform::Key};

/// One hundred and twenty lines of 13 characters plus a newline:
///  `line 007 word`.
fn numbered() -> String {
    let mut text = String::new();
    for index in 0..120 {
        text.push_str(&format!("line {index:03} word\n"));
    }
    return text;
}

/// Character offset of `column` on `line` of `numbered()`.
fn at(line: usize, column: usize) -> usize {
    return line * 14 + column;
}

/// The label of list row `index`.
fn label(reader: &LanguageReader, index: usize) -> String {
    let rows = reader.window.get_references_entries();
    return rows.row_data(index).expect("list row").label.to_string();
}

/// Tree badge of the row labelled `name`.
fn badge(reader: &LanguageReader, name: &str) -> String {
    let index = row(&reader.window, name).expect("tree row") as usize;
    let rows = reader.window.get_tree_entries();
    return rows
        .row_data(index)
        .expect("tree entry")
        .recency
        .to_string();
}

/// A definition in the displayed file moves the caret there and scrolls it into view.
#[test]
fn definition_in_the_same_file_moves_the_caret_and_reveals_it() {
    let fixture = project(&[("main.scripted", &numbered())]);
    let main = address(&fixture.root.join("main.scripted"));
    let answer = format!("[{}]", location(&main, 100, 5, 8));
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(&[("DEFINITION", &answer)], None),
    );
    ready(&reader);
    definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(head(&reader) == at(100, 5)) {
            waiting.step("the caret did not reach the definition");
        }
    }
    let top = -reader.window.get_scroll_y();
    let bottom = top + reader.window.get_viewport_height();
    // The vertical mapping places line 100: the scripted server's hint takes a row above the first line.
    let row_top = reader.source.borrow().row_map.code_top(100);
    assert!(
        top <= row_top && row_top + 24.0 <= bottom,
        "line 100 at {row_top} is not in view: {top}..{bottom}"
    );
    assert!(reader.window.get_source_has_focus());
    assert_eq!(reader.window.get_language_popup_text(), "");
}

/// Ctrl+B at the definition lists its references;
///  arrows wrap,
///  Enter opens the chosen one with history.
#[test]
fn definition_at_the_caret_lists_references_and_enter_opens_the_chosen_one() {
    let other_text = "first\n  second line\n";
    let fixture = project(&[
        ("main.scripted", &numbered()),
        ("nested/other.scripted", other_text),
    ]);
    let main = address(&fixture.root.join("main.scripted"));
    let other = address(&fixture.root.join("nested/other.scripted"));
    let answer = format!("[{}]", location(&main, 0, 0, 4));
    let references = format!(
        "[{},{},{}]",
        location(&main, 0, 0, 4),
        location(&other, 1, 2, 8),
        location(&main, 50, 5, 8)
    );
    let languages = definitions(
        &[("DEFINITION", &answer), ("REFERENCES", &references)],
        None,
    );
    let reader = reader(&fixture, "main.scripted", languages);
    ready(&reader);
    {
        let mut waiting = Waiting::language();
        while !(row(&reader.window, "main.scripted").is_some()) {
            waiting.step("the tree did not show the file");
        }
    }
    definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_references_open()) {
            waiting.step("the references list did not open");
        }
    }
    assert_eq!(reader.window.get_references_title(), "3 references");
    assert_eq!(label(&reader, 0), "main.scripted:1");
    assert_eq!(label(&reader, 1), "nested/other.scripted:2");
    assert_eq!(label(&reader, 2), "main.scripted:51");
    assert!(reader.window.get_references_has_focus());
    assert_eq!(reader.window.get_references_selected(), 0);
    // The list never covers the caret line.
    let top = reader.window.get_language_anchor_top();
    let list_y = reader.window.get_references_y();
    assert!(list_y >= top + 24.0 || list_y + reader.window.get_references_height() <= top);
    key(&reader.window, Key::UpArrow);
    assert_eq!(reader.window.get_references_selected(), 2);
    key(&reader.window, Key::DownArrow);
    key(&reader.window, Key::DownArrow);
    assert_eq!(reader.window.get_references_selected(), 1);
    key(&reader.window, Key::Return);
    {
        let mut waiting = Waiting::language();
        while !(reader.source.borrow().file_path.as_deref()
            == Some(&fixture.root.join("nested/other.scripted")))
        {
            waiting.step("the chosen reference's file did not open");
        }
    }
    assert!(!reader.window.get_references_open());
    {
        let mut waiting = Waiting::language();
        while !(head(&reader) == 8) {
            waiting.step("the caret did not reach the reference");
        }
    }
    {
        let mut waiting = Waiting::language();
        while !(row(&reader.window, "other.scripted").is_some()) {
            waiting.step("the tree did not reveal the reference's file");
        }
    }
    assert_eq!(badge(&reader, "other.scripted"), "0");
    assert_eq!(badge(&reader, "main.scripted"), "1");
    assert!(!reader.window.get_file_outside_project());
}

/// Ctrl+click asks for the definition under the pointer,
///  without the references fallback.
#[test]
fn control_click_opens_a_definition_in_another_project_file() {
    let fixture = project(&[
        ("main.scripted", &numbered()),
        ("nested/other.scripted", "first\n  second line\n"),
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
        while !(row(&reader.window, "main.scripted").is_some()) {
            waiting.step("the tree did not show the file");
        }
    }
    let x = caret_x(&reader, at(3, 6)) + 2.0;
    control_click(&reader.window, point(&reader, 3, x));
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_source_text() == "first\n  second line\n") {
            waiting.step("Ctrl+click did not open the definition's file");
        }
    }
    {
        let mut waiting = Waiting::language();
        while !(head(&reader) == 8) {
            waiting.step("the caret did not reach the definition");
        }
    }
    {
        let mut waiting = Waiting::language();
        while !(row(&reader.window, "other.scripted").is_some()) {
            waiting.step("the tree did not reveal the file");
        }
    }
    assert_eq!(badge(&reader, "other.scripted"), "0");
}

/// An outside-project target opens read-only and marked,
///  without a tree row or a history slot.
#[test]
fn outside_project_definition_opens_marked_without_tree_reveal_or_history() {
    let fixture = project(&[("main.scripted", &numbered())]);
    let library = fixture.outside.join("library.scripted");
    std::fs::write(&library, "pub library\n").expect("outside fixture");
    let answer = format!("[{}]", location(&address(&library), 0, 4, 11));
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(&[("DEFINITION", &answer)], None),
    );
    ready(&reader);
    {
        let mut waiting = Waiting::language();
        while !(row(&reader.window, "main.scripted").is_some()) {
            waiting.step("the tree did not show the file");
        }
    }
    definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_source_text() == "pub library\n") {
            waiting.step("the outside file did not open");
        }
    }
    assert!(reader.window.get_file_outside_project());
    assert_eq!(
        reader.window.get_file_label(),
        SharedString::from(library.display().to_string())
    );
    {
        let mut waiting = Waiting::language();
        while !(head(&reader) == 4) {
            waiting.step("the caret did not reach the outside target");
        }
    }
    assert_eq!(
        badge(&reader, "main.scripted"),
        "0",
        "the outside file entered the history"
    );
    let rows = reader.window.get_tree_entries();
    for index in 0..rows.row_count() {
        assert!(
            !rows.row_data(index).expect("tree entry").selected,
            "a tree row is selected"
        );
    }
    reader
        .window
        .invoke_tree_shortcut(SharedString::from("0"), true, false, false);
    {
        let mut waiting = Waiting::language();
        while !(!reader.window.get_file_outside_project()) {
            waiting.step("Ctrl+0 did not return to the project file");
        }
    }
    assert!(reader.window.get_source_text().starts_with("line 000"));
}

/// Several definitions are listed;
///  Escape and an outside click close the list and return focus.
#[test]
fn several_definitions_are_listed_and_closing_returns_focus_to_the_source() {
    let fixture = project(&[("main.scripted", &numbered())]);
    let library = fixture.outside.join("library.scripted");
    std::fs::write(&library, "pub library\n").expect("outside fixture");
    let main = address(&fixture.root.join("main.scripted"));
    let answer = format!(
        "[{},{}]",
        location(&main, 2, 0, 4),
        location(&address(&library), 0, 0, 3)
    );
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(&[("DEFINITION", &answer)], None),
    );
    ready(&reader);
    caret(&reader, at(10, 2));
    definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_references_open()) {
            waiting.step("the definitions list did not open");
        }
    }
    assert_eq!(reader.window.get_references_title(), "2 definitions");
    assert_eq!(label(&reader, 0), "main.scripted:3");
    let rows = reader.window.get_references_entries();
    assert_eq!(
        rows.row_data(1).expect("outside row").detail,
        "Outside project"
    );
    escape(&reader);
    assert!(!reader.window.get_references_open());
    assert!(reader.window.get_source_has_focus());
    assert_eq!(head(&reader), at(10, 2));
    definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_references_open()) {
            waiting.step("the definitions list did not open again");
        }
    }
    click(&reader.window, LogicalPosition::new(100.0, 400.0));
    assert!(!reader.window.get_references_open());
    assert!(reader.window.get_source_has_focus());
}

/// Unopenable targets are explained:
///  a single one at once,
///  a listed one when chosen.
#[test]
fn unavailable_targets_explain_why_they_cannot_be_opened() {
    let fixture = project(&[("main.scripted", &numbered())]);
    let missing = location("file:///definitely/missing/file.scripted", 0, 0, 1);
    let answer = format!("[{},{}]", location("untitled:Untitled-1", 0, 0, 1), missing);
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(&[("DEFINITION", &answer)], None),
    );
    ready(&reader);
    definition(&reader);
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_references_open()) {
            waiting.step("the definitions list did not open");
        }
    }
    assert_eq!(
        reader
            .window
            .get_references_entries()
            .row_data(0)
            .expect("row")
            .detail,
        "Cannot open"
    );
    key(&reader.window, Key::DownArrow);
    key(&reader.window, Key::Return);
    assert!(!reader.window.get_references_open());
    assert_eq!(
        reader.window.get_language_popup_text(),
        "Cannot open file:///definitely/missing/file.scripted: the file does not exist."
    );
    assert!(reader.window.get_language_popup_note());
    assert!(reader.window.get_source_has_focus());
}

/// A later open intent drops a language target still waiting for its file.
#[test]
fn a_later_open_drops_a_waiting_definition_target() {
    let fixture = project(&[
        ("main.scripted", &numbered()),
        ("b.scripted", "b0\nb1\nb2\n"),
        ("c.scripted", "c0 c1\n"),
    ]);
    let reader = reader(&fixture, "main.scripted", definitions(&[], None));
    {
        let mut waiting = Waiting::language();
        while !(row(&reader.window, "c.scripted").is_some()) {
            waiting.step("the tree did not show the files");
        }
    }
    // The state a definition answer leaves while its file `b` is still being read.
    reader.source.borrow_mut().pending_jump = Some(super::Jump {
        path: fixture.root.join("b.scripted"),
        outside: false,
        line: 0,
        range: Some((3, 4)),
    });
    reader
        .window
        .invoke_tree_activate(row(&reader.window, "c.scripted").expect("c row"));
    {
        let mut waiting = Waiting::language();
        while !(reader.window.get_source_text() == "c0 c1\n") {
            waiting.step("the chosen file did not open");
        }
    }
    assert_eq!(
        head(&reader),
        0,
        "a dropped definition target moved the caret in another file"
    );
    assert!(reader.source.borrow().pending_jump.is_none());
}
