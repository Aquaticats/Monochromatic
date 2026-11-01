//! The store of servers' last standard-error lines.
//!  Every test uses its own server name,
//!  because
//! the store is shared by the whole test process.

use super::{
    CUT_MARK, KEPT_LINES, LINE_BYTES, cut, ended, forget, is_closed, remember, take, unescape,
};

#[test]
fn unescape_reverses_debug_quoting() {
    let line = "tab\there \"quoted\" back\\slash bell\u{7} escape\u{1b}[0m nul\0 é\r\n";
    // `{:?}` is the quoting helix-lsp applies; the outer quotes are not part of the payload.
    let quoted = format!("{line:?}");
    let payload = &quoted[1..quoted.len() - 1];
    assert_eq!(unescape(payload), line);
}

#[test]
fn unescape_keeps_what_names_no_character() {
    assert_eq!(unescape(r"bad \u{zz} end"), r"bad \u{zz} end");
    assert_eq!(unescape(r"surrogate \u{d800}"), r"surrogate \u{d800}");
    assert_eq!(unescape(r"trailing \"), r"trailing \");
    assert_eq!(unescape(r"plain \q"), "plain q");
}

#[test]
fn cut_keeps_short_lines_and_marks_long_ones_at_a_character_boundary() {
    let exact = "a".repeat(LINE_BYTES);
    assert_eq!(cut(&exact), exact);
    // Two-byte characters after one ASCII byte: byte `LINE_BYTES` falls inside a character.
    let long = format!("a{}", "é".repeat(LINE_BYTES));
    let shortened = cut(&long);
    assert!(shortened.ends_with(CUT_MARK), "{shortened}");
    let kept = shortened.strip_suffix(CUT_MARK).expect("mark");
    assert_eq!(kept.len(), LINE_BYTES - 1);
    assert!(long.starts_with(kept));
}

#[test]
fn a_tail_keeps_the_newest_lines_without_their_newline() {
    let server = "tail-newest";
    for number in 0..KEPT_LINES + 3 {
        remember(server, &format!("line {number}\\n"));
    }
    let lines = take(server);
    assert_eq!(lines.len(), KEPT_LINES);
    assert_eq!(lines.first().map(String::as_str), Some("line 3"));
    assert_eq!(
        lines.last(),
        Some(&format!("line {}", KEPT_LINES + 2)),
        "{lines:?}"
    );
}

#[test]
fn take_empties_a_tail() {
    let server = "tail-take";
    remember(server, "only\\n");
    ended(server);
    assert_eq!(take(server), vec!["only".to_string()]);
    assert!(take(server).is_empty());
}

#[test]
fn the_end_of_a_stream_is_seen_and_a_new_line_starts_a_new_tail() {
    let server = "tail-closed";
    assert!(!is_closed(server));
    remember(server, "old process\\n");
    assert!(!is_closed(server));
    ended(server);
    assert!(is_closed(server));
    remember(server, "new process\\n");
    assert!(!is_closed(server));
    assert_eq!(take(server), vec!["new process".to_string()]);
}

#[test]
fn a_server_that_wrote_nothing_still_has_a_stream_end() {
    let server = "tail-silent";
    ended(server);
    assert!(is_closed(server));
    assert!(take(server).is_empty());
}

#[test]
fn lines_after_a_report_are_dropped_until_that_stream_ends() {
    let server = "tail-retired";
    remember(server, "before the report\\n");
    assert_eq!(take(server), vec!["before the report".to_string()]);
    remember(server, "same process, after the report\\n");
    assert!(!is_closed(server), "a retired tail is not a closed one");
    ended(server);
    remember(server, "next process\\n");
    assert_eq!(take(server), vec!["next process".to_string()]);
}

#[test]
fn forget_drops_lines_and_the_rest_of_the_stream() {
    let server = "tail-forget";
    remember(server, "stopped by the worker\\n");
    forget(server);
    remember(server, "still the stopped process\\n");
    ended(server);
    ended(server);
    assert!(take(server).is_empty());
}
