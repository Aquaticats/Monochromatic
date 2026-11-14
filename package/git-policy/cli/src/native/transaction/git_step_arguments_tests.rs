//! Controls for the arguments a slow step's record names:
//! the question a wrapper query of its own asked is kept,
//! every word the person typed is replaced,
//! and the list is bounded.

use super::*;

/// The arguments of one step,
/// as the record names them.
fn named(words: &[&str]) -> String {
    let arguments: Vec<OsString> = words.iter().map(OsString::from).collect();
    return named_arguments(arguments.as_slice());
}

/// The ref snapshot of a commit is named whole,
/// which is the one line the record of a 26 second child needed.
///
/// The four commits of 2026-11-12 that took 6 to 99 seconds each showed a child named
/// `git for-each-ref` and nothing more,
/// so a reader could not tell the whole listing
///  of every ref from a question about one.
#[test]
fn the_ref_snapshot_of_a_commit_is_named_whole() {
    let format: String = format!("--format=%(refname)%00%(objectname)%00%(symref)");
    assert_eq!(
        named(&["for-each-ref", format.as_str()]),
        format!("for-each-ref {format}"),
        "the question a slow child asked is what the record must name"
    );
}

/// An object name and a ref name are the repository's own and are kept,
/// so a slow step names which commit it walked.
#[test]
fn object_names_and_ref_names_are_kept() {
    let first: &str = "e6a1b7c9d2f30415263748596a7b8c9d0e1f2a3b";
    let second: &str = "0123456789abcdef0123456789abcdef01234567";
    assert_eq!(
        named(&["rev-list", "--objects", first, "--not", second, "refs/heads/main"]),
        format!("rev-list --objects {first} --not {second} refs/heads/main")
    );
}

/// A message never reaches the record,
/// whether it came on its own word or on the option's,
/// and a path becomes the same placeholder as any other word the person typed.
#[test]
fn a_message_and_a_path_are_never_written() {
    assert_eq!(
        named(&["commit",
        "--message=the person's words",
        "a.txt"]),
        "commit --message an argument an argument",
        "the message and the path are each a placeholder,
        and neither is written"
    );
    assert_eq!(named(&["commit", "-m", "the person's words", "--no-only"]), "commit -m an argument --no-only");
    assert_eq!(named(&["commit", "--file", "/somewhere/MESSAGE"]), "commit --file an argument");
    assert!(!named(&["commit", "--message=secret"]).contains("secret"));
}

/// The options before the subcommand take their value with them,
/// because a global option's value is a directory or a setting of the wrapper's own.
#[test]
fn a_global_option_s_value_is_left_out() {
    assert_eq!(named(&["-C", "/var/home/person/work", "status", "--porcelain"]), "status --porcelain");
    assert_eq!(named(&["-c", "core.hooksPath=/dev/null", "commit"]), "commit");
}

/// A step whose arguments name nothing has no words at all,
/// so the record of one such step reads as it did before this field existed.
#[test]
fn a_step_with_nothing_to_name_has_no_words() {
    assert_eq!(named_arguments(&[]), "");
    assert_eq!(named_arguments(&[OsString::from("-C"), OsString::from("/var/home/person/work")]), "");
}

/// The words are bounded and the rest are counted,
/// so a step asked about ten thousand paths writes a few bytes and never a path.
#[test]
fn the_words_are_bounded_and_the_rest_counted() {
    let paths: usize = 10_000;
    let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--")];
    for index in 0..paths {
        arguments.push(OsString::from(format!("package/path-{index:05}.rs")));
    }
    let text: String = named_arguments(arguments.as_slice());
    // Two words are the subcommand and the separator; every word after the bound is counted.
    let words_before_the_bound: usize = 2;
    let left_out: usize = paths.saturating_sub(NAMED_WORDS_LIMIT.saturating_sub(words_before_the_bound));
    assert!(text.ends_with(format!("and {left_out} more").as_str()), "{text}");
    assert!(!text.contains("package/path-"), "{text}");
    assert!(text.len() < 200, "the record carries a bounded number of words: {text}");
}

/// A word holding a quote or a backslash never breaks the record,
/// because the figure writes the words as a JSON string.
#[test]
fn a_word_with_a_quote_cannot_break_the_record() {
    let arguments: Vec<OsString> = vec![OsString::from("log"), OsString::from("--format=%H\"%n\\")];
    let words:
    String = named_arguments(arguments.as_slice());
    let figure:
    crate::transaction_step_durations::StepFigure =
        crate::transaction_step_durations::StepFigure { name:
        String::from("git log"),
        arguments:
        Some(words),
        offset_ms:
        1,
        duration_ms:
        1500 };
    assert_eq!(
        crate::transaction_step_durations::steps_json(&[figure]),
        "[{\"name\":\"git log\",\"arguments\":\"log --format=%H\\\"%n\\\\\",\"offsetMs\":1,\"durationMs\":1500}]"
    );
}
