// Line-splitting edge-case and fail-closed unit tests for the frx scan path.

use super::{line_starts, scan_file, scan_one_set};
use crate::ScanFinding;
use std::panic::AssertUnwindSafe;

// Creates a fresh temp dir under the OS temp root, keyed by label and pid.
fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("frx-scan-{}-{}", label, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    return dir
}

// Builds LoadedRules from an in-repo temp rules file with no builtin baseline.
fn load_runtime(rules: &str) -> crate::frx_load::LoadedRules {
    let dir = temp_dir("rules");
    let path = dir.join("rules.txt");
    std::fs::write(&path, rules).expect("write rules");
    let loaded = crate::frx_load::load(path.to_str().expect("utf8 path"), false, true, b"", "")
        .expect("load runtime rules");
    let _ = std::fs::remove_dir_all(&dir);
    return loaded
}

#[test]
fn line_starts_plain_lines_index_each_start() {
    // "abc\ndef" -> line 1 at 0, line 2 at 4; both valid indices.
    assert_eq!(line_starts(b"abc\ndef"), vec![0, 4]);
}

#[test]
fn line_starts_unterminated_final_line_keeps_its_start() {
    // No trailing newline: the final line is still a start.
    assert_eq!(line_starts(b"a\nbc"), vec![0, 2]);
}

#[test]
fn line_starts_trailing_newline_drops_empty_final_line() {
    // The empty line after a trailing newline would sit at buf.len(); dropped so
    // every offset stays a valid in-bounds index.
    assert_eq!(line_starts(b"abc\n"), vec![0]);
    assert_eq!(line_starts(b"a\nb\n"), vec![0, 2]);
}

#[test]
fn line_starts_crlf_offsets_point_past_the_newline() {
    // The carriage return stays inside the line span; the engine strips it. Starts
    // point past '\n' regardless of the preceding '\r'.
    assert_eq!(line_starts(b"abc\r\ndef"), vec![0, 5]);
    assert_eq!(line_starts(b"abc\r\n"), vec![0]);
}

#[test]
fn line_starts_interior_empty_lines_keep_their_offset() {
    // "a\n\nb": the blank middle line keeps its start so numbering stays aligned; the
    // matcher skips the empty span.
    assert_eq!(line_starts(b"a\n\nb"), vec![0, 2, 3]);
}

#[test]
fn line_starts_empty_buffer_is_just_the_zero_start() {
    assert_eq!(line_starts(b""), vec![0]);
}

#[test]
fn scan_one_set_normal_return_formats_and_offsets() {
    // Unnamed (line index, rule id) pairs render 1-based with the base offset added.
    let matcher = AssertUnwindSafe(|| vec![(0usize, 0usize), (2usize, 1usize)]);
    let hits = scan_one_set(5, &[], matcher);
    assert_eq!(hits, vec![
        ScanFinding::Content { line: 1, rule: String::from("5") },
        ScanFinding::Content { line: 3, rule: String::from("6") },
    ]);
    assert_eq!(hits[0].render("a.txt"), "a.txt:1 rule=5");
    assert_eq!(hits[1].render("a.txt"), "a.txt:3 rule=6");
}

#[test]
fn scan_one_set_named_rules_render_names_and_skip_offset() {
    // A named rule renders its section name; an unnamed slot in the same set still
    // falls back to the offset numeric id.
    let names = vec![Some("qqq-named".to_string()), None];
    let matcher = AssertUnwindSafe(|| vec![(0usize, 0usize), (1usize, 1usize)]);
    let hits = scan_one_set(5, &names, matcher);
    assert_eq!(hits, vec![
        ScanFinding::Content { line: 1, rule: String::from("qqq-named") },
        ScanFinding::Content { line: 2, rule: String::from("6") },
    ]);
    assert_eq!(hits[0].render("a.txt"), "a.txt:1 rule=qqq-named");
    assert_eq!(hits[1].render("a.txt"), "a.txt:2 rule=6");
}

#[test]
fn scan_one_set_engine_panic_fails_closed() {
    // A synthetic engine panic must be caught and reported as a redacted finding, and
    // the panic must not escape. Silence the default hook so the caught panic prints
    // nothing (nextest runs each test in its own process, so the swap is isolated).
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let matcher = AssertUnwindSafe(|| -> Vec<(usize, usize)> { panic!("synthetic engine fault") });
    let hits = scan_one_set(0, &[], matcher);
    std::panic::set_hook(previous);
    assert_eq!(hits, vec![ScanFinding::EngineError]);
    assert_eq!(hits[0].render("a.txt"), "a.txt: engine error");
}

#[test]
fn scan_file_reports_matched_line_numbers() {
    // End-to-end through a real compiled set: a literal on line 3 reports line 3.
    let loaded = load_runtime("NEEDLE_LITERAL_LONG_ENOUGH\n");
    let hits = scan_file("t.txt", b"clean\nalso clean\nNEEDLE_LITERAL_LONG_ENOUGH\n", &loaded);
    assert_eq!(hits, vec!["t.txt:3 rule=0".to_string()]);
}

#[test]
fn scan_file_crlf_line_still_matches() {
    // A CRLF-terminated line matches with the carriage return excluded.
    let loaded = load_runtime("NEEDLE_LITERAL_LONG_ENOUGH\n");
    let hits = scan_file("t.txt", b"NEEDLE_LITERAL_LONG_ENOUGH\r\ntail\r\n", &loaded);
    assert_eq!(hits, vec!["t.txt:1 rule=0".to_string()]);
}

#[test]
fn scan_file_interior_empty_line_preserves_numbering() {
    // The blank line 2 contributes no finding but the match on line 3 keeps its
    // number.
    let loaded = load_runtime("NEEDLE_LITERAL_LONG_ENOUGH\n");
    let hits = scan_file("t.txt", b"first\n\nNEEDLE_LITERAL_LONG_ENOUGH\n", &loaded);
    assert_eq!(hits, vec!["t.txt:3 rule=0".to_string()]);
}

#[test]
fn scan_file_empty_buffer_is_clean() {
    let loaded = load_runtime("NEEDLE_LITERAL_LONG_ENOUGH\n");
    assert!(scan_file("t.txt", b"", &loaded).is_empty());
}
