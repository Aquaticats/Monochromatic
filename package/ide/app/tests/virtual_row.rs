//! The rows a line shows above its code row,
//!  without fonts or a window:
//!  how messages become rows,
//!  how a pile
//! of them is capped,
//!  and how tall a block is.

/// What:
///  Import the production row rules and the diagnostic records they consume.
/// Why:
///  The vertical mapping and every frame take row counts from exactly these functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Block, messageRows, wrap } from 'ide-app/virtual-row';
/// ```
use ide_app::{
    annotation::{Mark, Problem},
    language::diagnostics::Severity,
    virtual_row::{
        BLOCK_GAP, Block, HintPlace, LINE_MESSAGES, MESSAGE_ROWS, MessageRow, ROW_HEIGHT,
        WRAP_COLUMNS, message_rows, wrap,
    },
};

/// What:
///  One diagnostic of `severity` starting at character `start`;
///  `&str` lends the message.
/// Why:
///  Rows are derived from problems exactly as the annotation store indexes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function problem(start: number, severity: Severity, message: string): Problem;
/// ```
fn problem(start: usize, severity: Severity, message: &str) -> Problem {
    return Problem {
        mark: Mark {
            start,
            end: start + 1,
            severity,
        },
        source: "rustc".to_string(),
        // `Some(...)` is a present code; the rows show it after the severity word.
        code: Some("E1".to_string()),
        message: message.to_string(),
    };
}

/// The texts of `rows`,
///  in order.
fn texts(rows: &[MessageRow]) -> Vec<&str> {
    let mut result = Vec::new();
    for row in rows {
        result.push(row.text.as_str());
    }
    return result;
}

/// A short message is one row that starts with the severity word,
///  the code,
///  and the source.
#[test]
fn a_message_row_starts_with_severity_code_and_source() {
    let found = problem(7, Severity::Error, "mismatched types");
    let rows = message_rows(&[&found]);
    assert_eq!(texts(&rows), ["Error E1 (rustc): mismatched types"]);
    assert_eq!(rows[0].start, 7);
    assert!(!rows[0].continued);
    assert_eq!(rows[0].severity, Severity::Error);
}

/// Line breaks in a message start new rows,
///  blank lines are dropped,
///  indentation is kept,
///  and every row after
/// the first is marked as continued.
#[test]
fn line_breaks_start_rows_and_indentation_is_kept() {
    let found = problem(
        0,
        Severity::Error,
        "Type 'A' is not assignable to type 'B'.\n  Types of property 'a' are incompatible.\n\n\tType 'string' is not assignable to type 'number'.\n",
    );
    let rows = message_rows(&[&found]);
    assert_eq!(
        texts(&rows),
        [
            "Error E1 (rustc): Type 'A' is not assignable to type 'B'.",
            "  Types of property 'a' are incompatible.",
            "  Type 'string' is not assignable to type 'number'.",
        ]
    );
    let mut continued: Vec<bool> = Vec::new();
    for row in rows.iter() {
        continued.push(row.continued);
    }
    assert_eq!(continued, [false, true, true]);
}

/// A long line wraps at blanks to the column limit;
///  a word longer than a row is cut;
///  wide characters count two.
#[test]
fn long_lines_wrap_at_the_column_limit() {
    assert_eq!(wrap("one two three", 7), ["one two", "three"]);
    assert_eq!(
        wrap("one two three", 8),
        ["one two", "three"],
        "a word that does not fit starts the next row"
    );
    assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    assert_eq!(
        wrap("  indented text here", 12),
        ["  indented", "text here"]
    );
    assert_eq!(
        wrap("猫猫猫猫 猫", 8),
        ["猫猫猫猫", "猫"],
        "four wide characters fill eight columns"
    );
    assert!(wrap("", 10).is_empty());
    assert!(wrap("   ", 10).is_empty());
    // A message of 400 characters without a line break never produces a row past the limit.
    let long = "word ".repeat(80);
    let found = problem(0, Severity::Warning, &long);
    let rows = message_rows(&[&found]);
    assert!(
        rows.len() >= 5,
        "a long message must wrap, got {}",
        rows.len()
    );
    for row in &rows {
        assert!(
            row.text.chars().count() <= WRAP_COLUMNS,
            "a row has {} columns",
            row.text.chars().count()
        );
    }
}

/// A message with more lines than the cap shows the first lines and counts the rest on its last row.
#[test]
fn a_runaway_message_is_cut_and_counts_what_is_left_out() {
    let mut message = String::new();
    for index in 0..30 {
        message.push_str(&format!("line {index}\n"));
    }
    let found = problem(3, Severity::Information, &message);
    let rows = message_rows(&[&found]);
    assert_eq!(rows.len(), MESSAGE_ROWS);
    assert_eq!(rows[0].text, "Information E1 (rustc): line 0");
    assert_eq!(
        rows[MESSAGE_ROWS - 2].text,
        format!("line {}", MESSAGE_ROWS - 2)
    );
    assert_eq!(
        rows[MESSAGE_ROWS - 1].text,
        format!("… {} more lines", 30 - (MESSAGE_ROWS - 1))
    );
    // A message of exactly the cap is shown whole.
    let mut exact = String::new();
    for index in 0..MESSAGE_ROWS {
        exact.push_str(&format!("line {index}\n"));
    }
    let whole = message_rows(&[&problem(3, Severity::Information, &exact)]);
    assert_eq!(whole.len(), MESSAGE_ROWS);
    assert_eq!(
        whole[MESSAGE_ROWS - 1].text,
        format!("line {}", MESSAGE_ROWS - 1)
    );
}

/// Past the cap on messages per line,
///  one row counts the rest and names their worst severity;
///  given worst
/// first,
///  the cap therefore never hides an error behind a milder problem.
#[test]
fn a_pile_of_messages_is_capped_with_a_count() {
    let mut pile = Vec::new();
    for index in 0..LINE_MESSAGES {
        pile.push(problem(index, Severity::Error, "wrong"));
    }
    pile.push(problem(40, Severity::Warning, "suspicious"));
    pile.push(problem(41, Severity::Hint, "maybe"));
    pile.push(problem(42, Severity::Hint, "perhaps"));
    let lent: Vec<&Problem> = pile.iter().collect();
    let rows = message_rows(&lent);
    assert_eq!(rows.len(), LINE_MESSAGES + 1);
    for row in rows.iter().take(LINE_MESSAGES) {
        assert_eq!(row.severity, Severity::Error);
    }
    let last = &rows[LINE_MESSAGES];
    assert_eq!(last.text, "3 more on this line, the worst: Warning");
    assert_eq!(last.severity, Severity::Warning);
    assert_eq!(last.start, 40);
    // Exactly the cap needs no count.
    let fitting: Vec<&Problem> = pile.iter().take(LINE_MESSAGES).collect();
    assert_eq!(message_rows(&fitting).len(), LINE_MESSAGES);
}

/// A block is its gap plus its rows;
///  held space counts where it is taller than the rows;
///  rows are stacked
/// upwards from the code row,
///  messages below hints.
#[test]
fn block_height_and_row_positions() {
    let mut block = Block {
        line: 4,
        hint_rows: 2,
        hints: vec![hint(0), hint(1)],
        messages: vec![row("a"), row("b"), row("c")],
        held: (0.0, 0.0),
    };
    assert_eq!(block.height(), BLOCK_GAP + 5.0 * ROW_HEIGHT);
    assert_eq!(
        block.message_rise(2),
        ROW_HEIGHT,
        "the last message sits on the code row"
    );
    assert_eq!(block.message_rise(0), 3.0 * ROW_HEIGHT);
    assert_eq!(
        block.hint_rise(1),
        4.0 * ROW_HEIGHT,
        "hints stand above the messages"
    );
    assert_eq!(block.hint_rise(0), 5.0 * ROW_HEIGHT);
    // Held space taller than the rows keeps the block's height and leaves the rows where they are.
    block.held = (3.0 * ROW_HEIGHT, 4.0 * ROW_HEIGHT);
    assert_eq!(block.height(), BLOCK_GAP + 7.0 * ROW_HEIGHT);
    assert_eq!(block.message_rise(2), ROW_HEIGHT);
    assert_eq!(block.hint_rise(1), 5.0 * ROW_HEIGHT);
    // Held space lower than the rows changes nothing.
    block.held = (ROW_HEIGHT, ROW_HEIGHT);
    assert_eq!(block.height(), BLOCK_GAP + 5.0 * ROW_HEIGHT);
    // Nothing shown and nothing held: no block, and no gap either.
    let empty = Block {
        line: 0,
        hint_rows: 0,
        hints: Vec::new(),
        messages: Vec::new(),
        held: (0.0, 0.0),
    };
    assert_eq!(empty.height(), 0.0);
    let held_only = Block {
        held: (0.0, 2.0 * ROW_HEIGHT),
        ..empty
    };
    assert_eq!(held_only.height(), BLOCK_GAP + 2.0 * ROW_HEIGHT);
}

/// A message row of error severity that starts at the first character and holds `text`.
fn row(text: &str) -> MessageRow {
    return MessageRow {
        start: 0,
        continued: false,
        severity: Severity::Error,
        gap: None,
        text: text.to_string(),
    };
}

/// A hint on virtual row `index`.
fn hint(index: usize) -> HintPlace {
    return HintPlace {
        row: index,
        position: 0,
        x: 0.0,
        width: 10.0,
        text: "x".to_string(),
    };
}
