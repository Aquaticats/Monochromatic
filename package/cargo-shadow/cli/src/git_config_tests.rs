//! The unit tests of the Git configuration reader.

use crate::git_config::{ConfigEntry, parse_bool, parse_config};

/// Builds the entry the reader should produce.
fn entry(name: &str, value: Option<&str>) -> ConfigEntry {
    return ConfigEntry { name: name.to_string(), value: value.map(str::to_string) };
}

/// Sections, subsections and keys are named the way Git names them: case folded outside the quoted subsection.
#[test]
fn sections_subsections_keys_and_values_are_read_with_git_case_rules() {
    let text: &str = "[Core]\n\tBare = false\n[branch \"Feature/One\"]\n\tRemote = origin\n\tmerge = refs/heads/main\n";

    let parsed = parse_config(text).expect("parses");

    assert_eq!(
        parsed.entries,
        vec![entry("core.bare", Some("false")), entry("branch.Feature/One.remote", Some("origin")), entry("branch.Feature/One.merge", Some("refs/heads/main"))]
    );
    assert_eq!(parsed.last("BRANCH", Some("Feature/One"), "MERGE"), Some("refs/heads/main"));
    assert_eq!(parsed.last("branch", Some("feature/one"), "merge"), None, "subsection names keep their case");
}

/// The old dotted header lowercases its subsection.
#[test]
fn the_older_dotted_header_lowercases_its_subsection() {
    let parsed = parse_config("[branch.Main]\n\tremote = origin\n").expect("parses");

    assert_eq!(parsed.last("branch", Some("main"), "remote"), Some("origin"));
}

/// A repeated key keeps every value in order, and the last one wins for a single-valued key.
#[test]
fn a_key_may_repeat_and_the_last_value_wins_for_a_single_valued_key() {
    let parsed = parse_config("[remote \"origin\"]\n\tfetch = a\n\tfetch = b\n[core]\n\tx = 1\n\tx = 2\n").expect("parses");

    assert_eq!(parsed.values("remote", Some("origin"), "fetch"), vec![Some("a"), Some("b")]);
    assert_eq!(parsed.last("core", None, "x"), Some("2"));
}

/// Quotes, escapes, comments and continuations follow Git's value lexer.
#[test]
fn quotes_escapes_comments_and_continuations_are_applied() {
    let text: &str = "[a]\n\tq = \"x ; y\"  ; trailing comment\n\te = a\\tb\n\tc = one \\\ntwo\n\tbare\n# whole line\n";

    let parsed = parse_config(text).expect("parses");

    assert_eq!(parsed.last("a", None, "q"), Some("x ; y"));
    assert_eq!(parsed.last("a", None, "e"), Some("a\tb"));
    assert_eq!(parsed.last("a", None, "c"), Some("one two"));
    assert_eq!(parsed.values("a", None, "bare"), vec![None], "a key without an equals sign has no value");
}

/// Review finding (2a 5): quoted text keeps its trailing spaces and decoded escapes; unquoted edges and runs are handled as Git does.
#[test]
fn quoted_whitespace_is_kept_and_unquoted_edges_are_dropped() {
    let parsed = parse_config("[t]\n\tvalue = \"a \"\n\tother = \"a\\t\"\n\tspaced =   x  y   \n\tempty =\n").expect("parses");

    assert_eq!(parsed.last("t", None, "value"), Some("a "));
    assert_eq!(parsed.last("t", None, "other"), Some("a\t"));
    assert_eq!(parsed.last("t", None, "spaced"), Some("x  y"));
    assert_eq!(parsed.values("t", None, "empty"), vec![Some("")]);
}

/// Review finding (2a 1): a backslash at the end of a comment does not continue it.
#[test]
fn a_comment_is_not_continued_by_a_trailing_backslash() {
    let parsed = parse_config("[core]\n\t# comment \\\n[include]\n\tpath = x\n").expect("parses");

    assert!(parsed.has_include);
}

/// Review finding (2a 4): a dotted section with a quoted subsection is one flat name, and a quoted subsection may hold a bracket.
#[test]
fn a_dotted_section_with_a_quoted_subsection_and_a_bracket_in_a_subsection_are_read_as_git_reads_them() {
    let parsed = parse_config("[branch.topic \"sub\"]\n\tremote = .\n[branch \"a\\\"]#b\"]\n\tmerge = m\n").expect("parses");

    assert_eq!(parsed.last("branch", Some("topic.sub"), "remote"), Some("."));
    assert_eq!(parsed.last("branch", Some("a\"]#b"), "merge"), Some("m"));
}

/// A key on the header line is read.
#[test]
fn a_key_on_the_header_line_is_read() {
    let parsed = parse_config("[core] bare = true\n").expect("parses");

    assert_eq!(parsed.last("core", None, "bare"), Some("true"));
}

/// A byte order mark and carriage returns are what Git skips.
#[test]
fn a_byte_order_mark_and_windows_line_ends_are_read_like_plain_text() {
    let parsed = parse_config("\u{feff}[core]\r\n\tbare = true\r\n").expect("parses");

    assert_eq!(parsed.last("core", None, "bare"), Some("true"));
}

/// A value at the end of the text without a final newline is read.
#[test]
fn a_last_line_without_a_newline_is_read() {
    let parsed = parse_config("[core]\n\tbare = true").expect("parses");

    assert_eq!(parsed.last("core", None, "bare"), Some("true"));
}

/// `include` and `includeIf` are flagged and never followed.
#[test]
fn include_and_include_if_sections_are_flagged_and_not_followed() {
    assert!(parse_config("[include]\n\tpath = x\n").expect("parses").has_include);
    assert!(parse_config("[includeIf \"gitdir:/x/\"]\n\tpath = y\n").expect("parses").has_include);
    assert!(!parse_config("[core]\n\tbare = false\n").expect("parses").has_include);
}

/// Text Git rejects, or that is not read exactly, is an error.
#[test]
fn text_that_cannot_be_read_exactly_is_an_error() {
    assert!(parse_config("key = value\n").is_err(), "a line before any header");
    assert!(parse_config("[core\n").is_err(), "an unclosed header");
    assert!(parse_config("[core]\n\tx = \"open\n").is_err(), "an unclosed quote");
    assert!(parse_config("[core]\n\tx = a\\qb\n").is_err(), "an unknown escape");
    assert!(parse_config("[core]\n\t1x = y\n").is_err(), "a key starting with a digit");
    assert!(parse_config("[branch main]\n").is_err(), "a subsection without quotes");
    assert!(parse_config("[branch \"a\" ]\n").is_err(), "a blank before the closing bracket");
    assert!(parse_config("[core]\n\tx y = z\n").is_err(), "a key followed by a word");
}

/// Review finding (2a 2): every Git spelling of a boolean is understood, and an unknown one is an error.
#[test]
fn boolean_spellings_follow_git() {
    for spelling in ["true", "TRUE", "yes", "On", "1", "2", "-1", "1k"] {
        assert_eq!(parse_bool(Some(spelling)), Ok(true), "{spelling}");
    }
    for spelling in ["false", "No", "OFF", "", "0", "-0"] {
        assert_eq!(parse_bool(Some(spelling)), Ok(false), "{spelling}");
    }
    assert_eq!(parse_bool(None), Ok(true), "a key without a value is true");
    assert!(parse_bool(Some("maybe")).is_err());
    assert!(parse_bool(Some("0x1")).is_err());
}
