//! The pinned bin-paths answer,
//!  messages,
//!  and the noise removal on captured and hostile text.

use super::{MESSAGE_LIMIT, bin_paths, message};
use std::fs;

/// The pinned answer of `doc/planning/mpm-toolchain-migration.md`,
///  with the directories of a disposable machine.
fn answer(dirs: &[&str], bins: &[(&str, &str)]) -> String {
    let listed: Vec<String> = dirs.iter().map(|dir| format!("{dir:?}")).collect();
    let links: Vec<String> = bins
        .iter()
        .map(|(name, path)| format!("{name:?}: {path:?}"))
        .collect();
    return format!(
        r#"{{"exposureDir": ".cache/toolchain-bin", "pathDirs": [{}], "bins": {{{}}}}}"#,
        listed.join(", "),
        links.join(", ")
    );
}

#[test]
fn the_directories_of_the_pinned_answer_are_kept_whole() {
    let root = crate::test_scratch::scratch().expect("directory");
    let node = root.path().join("node/bin");
    let taplo = root.path().join("taplo");
    fs::create_dir_all(&node).expect("directory");
    fs::create_dir_all(&taplo).expect("directory");
    let missing = root.path().join("not-there");
    let listed = answer(
        &[
            node.to_str().expect("Unicode"),
            taplo.to_str().expect("Unicode"),
            missing.to_str().expect("Unicode"),
        ],
        &[("taplo", taplo.to_str().expect("Unicode"))],
    );
    let answered = bin_paths(&listed).expect("answer");
    assert_eq!(
        answered,
        [node, taplo],
        "a missing directory must be dropped"
    );
}

#[test]
fn a_directory_is_kept_once_and_spaces_quotes_and_dashes_stay_whole() {
    let root = crate::test_scratch::scratch().expect("directory");
    let odd = root.path().join("it's \"quoted\"/-dash/with space");
    fs::create_dir_all(&odd).expect("directory");
    let text = answer(
        &[
            odd.to_str().expect("Unicode"),
            odd.to_str().expect("Unicode"),
        ],
        &[],
    );
    assert_eq!(bin_paths(&text).expect("answer"), [odd]);
}

#[test]
fn a_relative_directory_is_dropped_because_it_depends_on_the_working_directory() {
    let root = crate::test_scratch::scratch().expect("directory");
    let relative = root.path().join("relative");
    fs::create_dir_all(&relative).expect("directory");
    let text = answer(&["relative/bin", "./also-relative"], &[]);
    assert!(bin_paths(&text).expect("answer").is_empty());
}

#[test]
fn an_answer_of_another_shape_fails_instead_of_guessing() {
    for (text, names) in [
        ("<html>", "not JSON"),
        ("[]", "not a JSON object"),
        (r#"{"pathDirs": [], "bins": {}}"#, "exposureDir"),
        (
            r#"{"exposureDir": 3, "pathDirs": [], "bins": {}}"#,
            "exposureDir",
        ),
        (
            r#"{"exposureDir": "/absolute", "pathDirs": [], "bins": {}}"#,
            "not relative to the worktree",
        ),
        (
            r#"{"exposureDir": "", "pathDirs": [], "bins": {}}"#,
            "not relative",
        ),
        (r#"{"exposureDir": "x", "bins": {}}"#, "pathDirs"),
        (
            r#"{"exposureDir": "x", "pathDirs": {}, "bins": {}}"#,
            "pathDirs",
        ),
        (
            r#"{"exposureDir": "x", "pathDirs": [3], "bins": {}}"#,
            "pathDirs",
        ),
        (r#"{"exposureDir": "x", "pathDirs": []}"#, "bins"),
        (
            r#"{"exposureDir": "x", "pathDirs": [], "bins": []}"#,
            "bins",
        ),
        (
            r#"{"exposureDir": "x", "pathDirs": [], "bins": {"taplo": 3}}"#,
            "taplo",
        ),
    ] {
        let reason = bin_paths(text).expect_err(text);
        assert!(reason.contains(names), "{text} gave {reason}");
    }
}

/// The standard error of node for an entry point that throws,
///  captured from a disposable run.
const UNREADABLE: &str = "file:///work/project/package/file-enforcer/core/src/toolchain/bin-paths.ts:12
  throw new Error('toolchain.lock.json is unreadable');
        ^
Error: toolchain.lock.json is unreadable
    at readLock (file:///work/project/package/file-enforcer/core/src/toolchain/bin-paths.ts:12:9)
Node.js v26.12.0
";

#[test]
fn a_message_keeps_the_lookup_words_and_drops_the_node_version_footer() {
    let text = message(UNREADABLE);
    assert!(text.contains("toolchain.lock.json is unreadable"), "{text}");
    assert!(text.contains("readLock"), "{text}");
    assert!(!text.contains("Node.js v"), "{text}");
}

#[test]
fn a_message_drops_escape_sequences_and_control_characters() {
    assert_eq!(
        message("\u{1b}[31mred\u{1b}[0m\u{7}\tnull\u{0} and \u{1b}]0;title\u{7}text"),
        "red\tnull and text"
    );
}

#[test]
fn a_message_keeps_project_names_verbatim_and_cuts_at_the_limit() {
    assert_eq!(
        message("mise WARN  missing: taplo@0.9.0\n"),
        "mise WARN  missing: taplo@0.9.0"
    );
    let long = "x".repeat(MESSAGE_LIMIT + 10);
    let cut = message(&long);
    assert_eq!(cut.chars().count(), MESSAGE_LIMIT + 3, "{cut}");
    assert!(cut.ends_with("..."), "{cut}");
    let exact = "y".repeat(MESSAGE_LIMIT);
    assert_eq!(message(&exact), exact);
}

#[test]
fn empty_lines_are_dropped_and_the_kept_ones_join_with_one_newline() {
    assert_eq!(
        message("first line\n\n  \nsecond line\n"),
        "first line\nsecond line"
    );
    assert_eq!(message("   "), "");
    assert_eq!(message(""), "");
}
