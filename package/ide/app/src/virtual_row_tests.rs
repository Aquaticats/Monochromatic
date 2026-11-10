//! One cap cuts a message to its row limit and counts the rows it left out.

/// The cap under test and the row type it keeps.
use super::{MESSAGE_ROWS, MessageRow, cap_message_rows};
/// Every row of a message carries the severity that selects its ink.
use crate::language::diagnostics::Severity;

/// One row of a single message:
///  its first row starts a line of the message,
///  every later row continues one.
fn row(index: usize) -> MessageRow {
    return MessageRow {
        start: 7,
        continued: index > 0,
        severity: Severity::Warning,
        gap: if index == 0 { None } else { Some(" ") },
        text: format!("line {index}"),
    };
}

/// What:
///  A message with more rows than the cap keeps the cap's first rows and counts the rest on its last row.
/// Why:
///  The virtual rows of a diagnostic and the rows an annotation block draws after a width refit cut the same
///  message,
///  so the kept count,
///  the hidden count,
///  and the wording are one rule.
#[test]
fn a_message_past_the_cap_keeps_its_first_rows_and_counts_the_rest() {
    let total = MESSAGE_ROWS + 8;
    let rows: Vec<MessageRow> = (0..total).map(row).collect();
    let capped = cap_message_rows(&rows);
    assert_eq!(capped.len(), MESSAGE_ROWS);
    for (index, kept) in capped[..MESSAGE_ROWS - 1].iter().enumerate() {
        assert_eq!(kept.text, format!("line {index}"));
    }
    let counted = &capped[MESSAGE_ROWS - 1];
    assert_eq!(
        counted.text,
        format!("… {} more lines", total - (MESSAGE_ROWS - 1))
    );
    assert!(counted.continued, "the counted row continues its message");
    assert_eq!(counted.start, 7, "the counted row keeps its anchor");
    assert_eq!(counted.severity, Severity::Warning);
}

/// A message at or under the cap is shown whole, with no row counting anything.
#[test]
fn a_message_within_the_cap_is_shown_whole() {
    for total in [1, MESSAGE_ROWS - 1, MESSAGE_ROWS] {
        let rows: Vec<MessageRow> = (0..total).map(row).collect();
        let capped = cap_message_rows(&rows);
        assert_eq!(capped.len(), total);
        assert!(
            capped
                .iter()
                .all(|shown| return !shown.text.contains("more lines")),
            "a message within the cap counts nothing"
        );
    }
}
