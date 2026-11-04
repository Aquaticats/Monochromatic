//! Controls for the replay patch parser:
//!  line splitting, hunk ranges, markers, incomplete lines and format failures.

use super::*;

/// Lines keep their newline, and an unterminated last line stays whole.
#[test]
fn splitting_keeps_newlines_and_the_unterminated_tail() {
    assert!(split_keeping_newlines(b"").is_empty());
    assert_eq!(
        split_keeping_newlines(b"a\nb"),
        vec![b"a\n".to_vec(), b"b".to_vec()]
    );
    assert_eq!(split_keeping_newlines(b"\n\n"), vec![b"\n".to_vec(), b"\n".to_vec()]);
}

/// Digits only: an empty range part, a sign, or a space is a format error.
#[test]
fn numbers_must_be_plain_digits() {
    assert_eq!(parse_number(b"0"), Some(0));
    assert_eq!(parse_number(b"123"), Some(123));
    assert_eq!(parse_number(b""), None);
    assert_eq!(parse_number(b"-1"), None);
    assert_eq!(parse_number(b"1 "), None);
    assert_eq!(parse_number(b"99999999999999999999"), None);
}

/// A range without a count names one line.
#[test]
fn ranges_default_the_count_to_one() {
    assert_eq!(parse_range(b"-3,2").expect("range"), (3, 2));
    assert_eq!(parse_range(b"+4").expect("range"), (4, 1));
    assert_eq!(parse_range(b"-0,0").expect("range"), (0, 0));
    assert!(parse_range(b"-x,1").is_err());
    assert!(parse_range(b"-1,").is_err());
}

/// A header with its ranges parses into the hunk and points after its body.
#[test]
fn one_hunk_reads_its_body_and_stops() {
    let lines: Vec<Vec<u8>> = split_keeping_newlines(b"@@ -1,2 +1,2 @@\n-a\n+b\n c\nnext\n");
    let (hunk, next): (PatchHunk, usize) = parse_hunk(&lines, 0).expect("hunk");
    assert_eq!(hunk.old_start, 1);
    assert_eq!(hunk.new_start, 1);
    assert_eq!(next, 4);
    assert_eq!(
        hunk.lines,
        vec![
            PatchLine { op: b'-', text: b"a\n".to_vec() },
            PatchLine { op: b'+', text: b"b\n".to_vec() },
            PatchLine { op: b' ', text: b"c\n".to_vec() },
        ]
    );
}

/// A bare newline is an empty context line, kept as a newline.
#[test]
fn a_bare_newline_is_an_empty_context_line() {
    let lines: Vec<Vec<u8>> = split_keeping_newlines(b"@@ -1,1 +1,1 @@\n\n");
    let (hunk, _next): (PatchHunk, usize) = parse_hunk(&lines, 0).expect("hunk");
    assert_eq!(hunk.lines, vec![PatchLine { op: b' ', text: b"\n".to_vec() }]);
}

/// The marker after an incomplete last line drops that line's newline,
///  wherever the marker sits: mid-hunk or at the end.
#[test]
fn incomplete_lines_lose_their_newline() {
    let mid: Vec<Vec<u8>> = split_keeping_newlines(b"@@ -1,2 +1,2 @@\n-a\n\\ No newline at end of file\n+b\n c\n");
    let (hunk, next): (PatchHunk, usize) = parse_hunk(&mid, 0).expect("hunk");
    assert_eq!(hunk.lines[0], PatchLine { op: b'-', text: b"a".to_vec() });
    assert_eq!(hunk.lines[1], PatchLine { op: b'+', text: b"b\n".to_vec() });
    assert_eq!(next, 5);

    let at_end: Vec<Vec<u8>> = split_keeping_newlines(b"@@ -1 +1 @@\n-a\n+b\n\\ No newline at end of file\n");
    let (tail, after): (PatchHunk, usize) = parse_hunk(&at_end, 0).expect("hunk");
    assert_eq!(tail.lines[1], PatchLine { op: b'+', text: b"b".to_vec() });
    // The cursor steps over the marker line, so the next parse starts after it.
    assert_eq!(after, 4);
}

/// A marker with no line before it is a format error.
#[test]
fn a_marker_before_any_line_fails() {
    let lines: Vec<Vec<u8>> = split_keeping_newlines(b"@@ -0,0 +1 @@\n\\ No newline\n+a\n");
    let error: ReplayPatchFormatError = parse_hunk(&lines, 0).expect_err("format error");
    assert!(error.0.contains("precedes every hunk line"), "{}", error.0);
}

/// Malformed headers, truncated bodies and lines without markers all fail.
#[test]
fn malformed_hunks_fail() {
    for text in [
        &b"@@ -1 +1\n"[..],
        &b"@@ -a +1 @@\n"[..],
        &b"@@ -1,2 +1,1 @@\n-a\n"[..],
        &b"@@ -1 +1 @@\nx\n"[..],
    ] {
        let lines: Vec<Vec<u8>> = split_keeping_newlines(text);
        assert!(parse_hunk(&lines, 0).is_err(), "{}", String::from_utf8_lossy(text));
    }
}

/// Every marker line type is classified, and anything else fails.
#[test]
fn markers_are_classified() {
    assert_eq!(line_marker(b"\n").expect("empty context"), b' ');
    assert_eq!(line_marker(b" x\n").expect("context"), b' ');
    assert_eq!(line_marker(b"-x\n").expect("removal"), b'-');
    assert_eq!(line_marker(b"+x\n").expect("addition"), b'+');
    assert_eq!(line_marker(b"\\ No newline\n").expect("marker"), b'\\');
    assert!(line_marker(b"").is_err());
    assert!(line_marker(b"xx\n").is_err());
}

/// Hunks found before any section header are dropped, a section without hunks maps to none,
///  and a section header without a destination fails.
#[test]
fn sections_group_hunks_per_entry_name() {
    let text: &[u8] = b"@@ -1 +1 @@\n-x\n+y\n\
diff --git a/0 b/0\nindex 1..2\n--- a/0\n+++ b/0\n@@ -1,2 +1 @@\n-a\n-b\n+c\n\
diff --git a/1 b/1\nindex 3..4\n";
    let files: BTreeMap<Vec<u8>, Vec<PatchHunk>> = parse_indexed_patch(text).expect("patch");
    assert_eq!(files.len(), 2);
    assert_eq!(files[&b"0".to_vec()].len(), 1);
    assert_eq!(files[&b"0".to_vec()][0].lines.len(), 3);
    assert!(files[&b"1".to_vec()].is_empty());
    assert!(parse_indexed_patch(b"diff --git a/0\n").is_err());
}

/// A section names its entry only by the bytes before the first space;
///  a truncated hunk anywhere in the patch fails the whole parse.
#[test]
fn a_truncated_hunk_fails_the_whole_patch() {
    assert!(parse_indexed_patch(b"diff --git a/0 b/0\n@@ -1,3 +1 @@\n-a\n").is_err());
    let text: &[u8] = b"diff --git a/10 b/10\n@@ -1 +1 @@\n-a\n+b\n";
    let files: BTreeMap<Vec<u8>, Vec<PatchHunk>> = parse_indexed_patch(text).expect("patch");
    assert!(files.contains_key(b"10".as_slice()));
}

/// A removal after an addition still counts against the old side, so a hunk whose old side ends with removals
///  reads those removals as its body and stops where the counts say.
#[test]
fn a_removal_after_an_addition_still_counts_the_old_side() {
    let lines: Vec<Vec<u8>> = split_keeping_newlines(b"@@ -1,2 +1,1 @@\n+x\n-a\n-b\nnext\n");
    let (hunk, next): (PatchHunk, usize) = parse_hunk(&lines, 0).expect("hunk");
    assert_eq!(hunk.lines.len(), 3);
    assert_eq!(next, 4);
    assert_eq!(lines[next], b"next\n".to_vec());
}

/// The format error displays exactly its message, so a caller that prints it names the malformed part.
#[test]
fn the_format_error_displays_its_message() {
    let error: ReplayPatchFormatError = ReplayPatchFormatError(String::from("malformed hunk range -x"));
    assert_eq!(error.to_string(), "malformed hunk range -x");
}

/// A needle that is exactly the haystack is found at zero, so the length guard admits equal lengths.
#[test]
fn a_needle_equal_to_the_haystack_is_found() {
    assert_eq!(find_bytes(b" @@", b" @@", 0), Some(0));
    assert_eq!(find_bytes(b"ab", b"ab", 0), Some(0));
    assert_eq!(find_bytes(b"ab", b"abc", 0), None);
}
