//! What:
//!  Tests for config text splitting:
//! accepted comments and blanks,
//! refused commas,
//! CR and sections.
//! Why:
//!  The decision record accepts exactly `#` comments and blank lines and refuses every other `parseConfig` feature.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseConfig', () => { /* comments, blanks, refusals */ });
//! ```

use crate::config::{ConfigLine, parse_config};

/// Build the expected line.
fn line(number: usize, text: &str) -> ConfigLine {
    return ConfigLine { number, text: text.to_string() };
}

/// Blank lines and comments vanish while line numbers keep counting.
#[test]
fn drops_comments_and_blanks() {
    let text = "# top\nlast 1 Chrome versions # why\n\n  \t\n\tFirefox ESR\t\n#only\nnot dead";
    assert_eq!(
        parse_config(text).expect("parses"),
        vec![line(2, "last 1 Chrome versions"), line(5, "Firefox ESR"), line(7, "not dead")]
    );
}

/// Empty text and comment-only text give no lines.
#[test]
fn empty_inputs_give_no_lines() {
    assert_eq!(parse_config("").expect("empty"), Vec::<ConfigLine>::new());
    assert_eq!(parse_config("# nothing\n").expect("comment"), Vec::<ConfigLine>::new());
    assert_eq!(parse_config("\n\n").expect("blank"), Vec::<ConfigLine>::new());
}

/// A comma inside a comment is part of the comment,
/// not a separator.
#[test]
fn comma_in_comment_is_harmless() {
    assert_eq!(parse_config("not dead # a, b\n").expect("parses"), vec![line(1, "not dead")]);
}

/// Commas as separators are refused and the message names the line.
#[test]
fn refuses_commas() {
    let message = parse_config("Firefox ESR\nlast 1 Chrome versions, not dead\n").expect_err("comma").to_string();
    assert!(message.starts_with("not implemented: commas"), "{message}");
    assert!(message.contains("line 2"), "{message}");
}

/// CR anywhere is refused,
/// including a lone CR and CRLF,
/// and the message names the line.
#[test]
fn refuses_carriage_returns() {
    for text in ["not dead\r\n", "a\nb\rc", "\r"] {
        let message = parse_config(text).expect_err("cr").to_string();
        assert!(message.starts_with("not implemented: CR line endings"), "{text:?}: {message}");
    }
    assert!(parse_config("a\nb\rc").expect_err("cr").to_string().contains("line 2"));
}

/// Section headers are refused,
/// whatever name or spacing they carry.
#[test]
fn refuses_sections() {
    for text in ["[production]", "  [dev test]  ", "[defaults]", "[x]"] {
        let message = parse_config(&format!("not dead\n{text}\n")).expect_err("section").to_string();
        assert!(message.starts_with("not implemented: config sections"), "{text:?}: {message}");
        assert!(message.contains("line 2"), "{message}");
    }
}

/// Bracket text that is not a header (too short,
/// or not bracket-enclosed) is passed on as a query line.
#[test]
fn bracket_lookalikes_are_ordinary_lines() {
    assert_eq!(parse_config("[]\nchrome [1]\n[1] x").expect("lines"), vec![line(1, "[]"), line(2, "chrome [1]"), line(3, "[1] x")]);
}

/// Only spaces and tabs are trimmed:
/// other whitespace stays in the line for the query matcher to refuse.
#[test]
fn trims_only_spaces_and_tabs() {
    let parsed = parse_config("\u{a0}not dead\u{feff}\n\u{b}\n").expect("parses");
    assert_eq!(parsed, vec![line(1, "\u{a0}not dead\u{feff}"), line(2, "\u{b}")]);
}
