// Unit tests for the two-form structured generator. Pulled in by
// `#[cfg(test)] #[path = "generators_tests.rs"] mod tests;` at the bottom of
// `generators.rs`, so they compile only under `cargo nextest run --lib` and reach the
// generator types via `use super::*` (super == `generators`). They pin the properties
// the format-driven fuzz targets rely on: a generated file renders to source the strict
// loader classifies as intended, a bad flag fails closed, and the redacted fingerprint
// never echoes content.

use super::*;

use forbidden_strings::fuzz_api::{load_from_text, scan_file};

// Builds a one-literal file and confirms it matches at boundaries, not inside a word.
#[test]
fn literal_line_loads_and_matches() {
    let file = RuleFile { lines: vec![RuleLine::Literal(SafeBytes(b"abc".to_vec()))] };
    let source = file.render();
    assert_eq!(source, "abc");
    let loaded = load_from_text(&source).expect("bare literal must load");
    let hits = scan_file("fuzz.txt", b"zz\nabc\nxabcx\n", &loaded);
    assert_eq!(hits, vec!["fuzz.txt:2 rule=0".to_string()]);
}

// Builds a regex line with an accepted no-op flag run and confirms it loads.
#[test]
fn mx_flags_are_noops() {
    let file = RuleFile {
        lines: vec![RuleLine::Regex { body: AlnumBytes(b"abc".to_vec()), flags: FlagRun::Both }],
    };
    assert!(!file.has_bad_flag());
    let loaded = load_from_text(&file.render()).expect("m/x flags are no-ops");
    let hits = scan_file("fuzz.txt", b"abc\n", &loaded);
    assert_eq!(hits, vec!["fuzz.txt:1 rule=0".to_string()]);
}

// Builds a regex line with a rejected flag and confirms the loader fails closed.
#[test]
fn bad_flag_line_fails_closed() {
    let file = RuleFile {
        lines: vec![RuleLine::Regex {
            body: AlnumBytes(b"abc".to_vec()),
            flags: FlagRun::Bad(BadFlag::I),
        }],
    };
    assert!(file.has_bad_flag());
    assert_eq!(file.render(), "/abc/i");
    assert!(load_from_text(&file.render()).is_err(), "a rejected flag must not load");
}

// A comment-only file has no loadable rule, so the strict loader returns NoRules.
#[test]
fn comment_only_file_has_no_rules() {
    let file = RuleFile { lines: vec![RuleLine::Comment(SafeBytes(b"note".to_vec()))] };
    assert_eq!(file.render(), "#note");
    assert!(load_from_text(&file.render()).is_err(), "a comment-only file loads no rules");
}

// Reversing the render reverses line order, the rule-order-invariance oracle.
#[test]
fn reversed_render_reverses_lines() {
    let file = RuleFile {
        lines: vec![
            RuleLine::Literal(SafeBytes(b"aaa".to_vec())),
            RuleLine::Literal(SafeBytes(b"bbb".to_vec())),
        ],
    };
    assert_eq!(file.render(), "aaa\nbbb");
    assert_eq!(file.render_reversed(), "bbb\naaa");
}

// The redacted fingerprint carries a length and digest, never the raw content bytes.
#[test]
fn fingerprint_is_redacted() {
    let fingerprint = redacted_fingerprint(b"secret-token-value");
    assert!(!fingerprint.contains("secret"), "fingerprint must not echo content: {fingerprint}");
    assert!(fingerprint.contains("len=18"));
    assert!(fingerprint.contains("sha256="));
}
