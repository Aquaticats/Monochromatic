//! The standard-error lines a failed start's reason carries:
//!  how many,
//!  how long,
//!  and what stays untouched.

use super::{CAUSE_LINE_BYTES, CAUSE_LINES, with_lines};

#[test]
fn a_reason_without_lines_is_unchanged() {
    assert_eq!(with_lines("it ended".to_string(), &[]), "it ended");
}

#[test]
fn the_last_lines_follow_the_reason_oldest_first() {
    let mut lines: Vec<String> = Vec::new();
    for number in 1..=6 {
        lines.push(format!("line {number}"));
    }
    let reason = with_lines("it ended".to_string(), &lines);
    let kept: Vec<&str> = reason.lines().skip(1).collect();
    assert_eq!(kept.len(), CAUSE_LINES);
    assert_eq!(kept, ["line 3", "line 4", "line 5", "line 6"]);
    assert_eq!(
        reason.lines().next(),
        Some("it ended; the server wrote this to its standard error:")
    );
}

#[test]
fn a_long_line_is_cut_at_a_character_boundary_and_marked() {
    // Two-byte characters make the byte limit fall inside one when it is odd.
    let long = "é".repeat(CAUSE_LINE_BYTES);
    let reason = with_lines("it ended".to_string(), &[long]);
    let line = reason.lines().nth(1).expect("the carried line");
    assert!(line.ends_with(" [cut]"), "{line}");
    let body = line.trim_end_matches(" [cut]");
    assert!(body.len() <= CAUSE_LINE_BYTES, "{} bytes", body.len());
    let mut accents_only = true;
    for character in body.chars() {
        if character != 'é' {
            accents_only = false;
        }
    }
    assert!(accents_only, "{body}");
}

#[test]
fn a_short_line_is_not_marked() {
    let reason = with_lines("it ended".to_string(), &["short".to_string()]);
    assert!(reason.ends_with("\nshort"), "{reason}");
}
