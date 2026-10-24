//! Hints a server places outside the lines it was asked about:
//!  an answer for other lines never adds a hint to a line it did not ask about,
//!  so the rows above that line,
//!  the height of the text,
//!  and the minimap's picture of it stay what the last answer for that line made them.

/// The production setter and the stamp of the displayed text.
use super::annotate::displayed;
/// The fixture text and the helpers that read a line's block.
use super::annotation_tests::{FIXTURE, block, hint};
/// The complete reader fixture.
use super::find_tests::{Reader, reader};
/// The production render.
use super::render;
/// Snapshot records exactly as the Language module builds them.
use ide_app::language::{hints::HintsSnapshot, identity::DocumentStamp};
/// Disposable fixtures and shared snapshot pointers.
use std::{fs, sync::Arc};

/// Hand a hints snapshot for the lines `asked.0` up to,
///  not including,
///  `asked.1` to the store the way the Language poll does,
///  then render once as the poll's caller does.
fn answer(reader: &Reader, stamp: DocumentStamp, asked: (usize, usize), items: Vec<(usize, &str)>) {
    let mut hints = Vec::new();
    for (position, label) in items {
        hints.push(hint(position, label));
    }
    let snapshot = Arc::new(HintsSnapshot {
        stamp,
        first_line: asked.0,
        last_line: asked.1,
        hints,
    });
    let accepted = {
        let mut source = reader.source.borrow_mut();
        let text = source.document.text().clone();
        source.annotations.accept_hints(stamp, &text, snapshot)
    };
    assert!(accepted, "the hints of the displayed text were refused");
    render(&reader.window, &reader.source);
}

/// How many hint texts the block above `line` holds,
///  as the state hands them to the window and to the minimap's request.
fn hints_above(reader: &Reader, line: usize) -> usize {
    return match block(reader, line) {
        Some(found) => found.hints.len(),
        None => 0,
    };
}

/// TypeScript 7 answers a request for later lines with the hints of a statement that began earlier:
///  line 0 of the fixture stands for such a statement.
///  Each answer must leave the hints of line 0,
///  the height of the text,
///  and the blocks the minimap is built from as the answer for line 0 made them.
#[test]
fn an_answer_for_other_lines_adds_no_hint_to_a_line_it_did_not_ask_about() {
    let fixture = tempfile::tempdir().expect("disposable range project");
    fs::write(fixture.path().join("main.rs"), FIXTURE).expect("range fixture");
    let reader = reader(fixture.path(), "main.rs");
    let stamp = displayed(&reader.source.borrow());
    // Line 0 is "let total = area(2, 3);": a type hint after `total` and the name of the first argument.
    let own = vec![(9, ": u32"), (17, "width:")];
    answer(&reader, stamp, (0, 4), own.clone());
    let before = reader.source.borrow().row_map.height();
    let blocks_before = reader.source.borrow().blocks.len();
    assert_eq!(
        hints_above(&reader, 0),
        2,
        "the answer for line 0 was not shown"
    );
    for round in 1..=4 {
        let mut later = own.clone();
        // Line 2 is "fn main() {}": the answer for lines 2 and 3 adds a hint there and repeats those of line 0.
        later.push((45, ": fn"));
        answer(&reader, stamp, (2, 4), later);
        assert_eq!(
            hints_above(&reader, 0),
            2,
            "round {round}: an answer for lines 2 and 3 added hints to line 0"
        );
        assert_eq!(
            hints_above(&reader, 2),
            1,
            "round {round}: the hint of line 2 was not shown once"
        );
        assert_eq!(
            reader.source.borrow().row_map.height(),
            before + block_rows(&reader, 2),
            "round {round}: the height of the text grew with each answer"
        );
    }
    assert_eq!(
        reader.source.borrow().blocks.len(),
        blocks_before + 1,
        "the blocks the minimap is built from changed beyond the one new line"
    );
    // An answer for line 0 again replaces line 0 whole.
    answer(&reader, stamp, (0, 2), vec![(9, ": u64")]);
    assert_eq!(hints_above(&reader, 0), 1);
}

/// The height of the block above `line`,
///  or nothing when it has none.
fn block_rows(reader: &Reader, line: usize) -> f32 {
    return match block(reader, line) {
        Some(found) => found.height(),
        None => 0.0,
    };
}
