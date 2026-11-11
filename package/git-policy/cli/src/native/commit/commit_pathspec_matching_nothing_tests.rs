//! Controls for the pathspec-that-matches-nothing input refusal:
//! the words of Git's it reads,
//! the tokens it attributes to the caller,
//! and what the message offers.

use super::*;
use std::ffi::OsString;

/// The elements one command line names,
/// and the borrowed slice the classifier is asked about.
fn written(elements: &[&str]) -> Vec<OsString> {
    return elements.iter().map(|element| { return OsString::from(*element); }).collect();
}

/// The tokens the classifier refuses,
/// as text.
fn refused(stderr: &str, elements: &[&str]) -> Option<Vec<String>> {
    let specs: Vec<OsString> = written(elements);
    let borrowed: Vec<&OsString> = specs.iter().collect();
    return pathspec_matching_nothing(stderr.as_bytes(), WrittenPathspecs::Elements(borrowed.as_slice()))
        .map(|found| { return found.iter().map(|token| { return String::from_utf8_lossy(token).into_owned(); }).collect(); });
}

/// Git's own sentence for `git add` is read,
/// and the token is the one the caller wrote.
#[test]
fn the_refusal_git_add_prints_names_the_token_the_caller_wrote() {
    let stderr: &str = "fatal: pathspec 'nope.txt' did not match any files\n";
    assert_eq!(refused(stderr, &["nope.txt"]), Some(vec![String::from("nope.txt")]));
}

/// `git commit` words the same refusal differently,
/// and that wording is recognized too,
/// so a pathspec that reaches the commit is not missed.
#[test]
fn the_refusal_native_git_commit_prints_is_understood() {
    let stderr: &str = "error: pathspec 'a.txt' did not match any file(s) known to git\n";
    assert_eq!(refused(stderr, &["a.txt"]), Some(vec![String::from("a.txt")]));
}

/// Only a failure whose sentence is that refusal is classified:
/// every other refusal of the step stays an engine failure.
#[test]
fn another_refusal_of_the_step_names_no_pathspec() {
    let stderr: &str = "fatal: Invalid pathspec magic 'bogus' in ':(bogus)a.txt'\n";
    assert_eq!(refused(stderr, &[":(bogus)a.txt"]), None);
}

/// A sentence that merely resembles the refusal names nothing,
/// because the wording that closes it is missing.
#[test]
fn a_sentence_that_only_looks_like_the_refusal_names_nothing() {
    let stderr: &str = "fatal: pathspec 'nope.txt' did not match anything this reader knows\n";
    assert_eq!(refused(stderr, &["nope.txt"]), None);
}

/// A pathspec the wrapper wrote itself is not the caller's input,
/// so a miss of one stays an engine failure.
#[test]
fn a_pathspec_the_caller_never_wrote_is_not_reported_as_the_callers_error() {
    let stderr: &str = "fatal: pathspec ':/' did not match any files\n";
    assert_eq!(refused(stderr, &["a.txt"]), None);
}

/// A pathspec Git read from the caller's own file is the caller's,
/// even though the command line never spelled it.
#[test]
fn a_pathspec_read_from_the_callers_file_is_the_callers() {
    let stderr: &[u8] = b"fatal: pathspec 'nope.txt' did not match any files\n";
    let found: Option<Vec<Vec<u8>>> = pathspec_matching_nothing(stderr, WrittenPathspecs::FromTheCallersFile);
    assert_eq!(found, Some(vec![b"nope.txt".to_vec()]));
}

/// One token is reported once,
/// and several are reported in Git's order.
#[test]
fn several_refused_tokens_are_reported_in_the_order_git_names_them() {
    let stderr: &str = "fatal: pathspec 'first.txt' did not match any files\nfatal: pathspec 'second.txt' did not match any files\n";
    assert_eq!(
        refused(stderr, &["first.txt", "second.txt"]),
        Some(vec![String::from("first.txt"), String::from("second.txt")])
    );
}

/// A token whose bytes are not valid UTF-8 is compared and kept as written,
/// so a path that is not text is still named exactly.
#[test]
fn a_token_of_bytes_that_are_not_valid_utf_8_is_kept_as_written() {
    let mut stderr: Vec<u8> = Vec::from(&b"fatal: pathspec '"[..]);
    stderr.extend_from_slice(&[0xff, 0xfe]);
    stderr.extend_from_slice(b"' did not match any files\n");
    let element: OsString = {
        // `OsString::from` of a byte slice that is not UTF-8 needs the platform's own bytes type.
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![0xff, 0xfe])
    };
    let elements: Vec<&OsString> = vec![&element];
    assert_eq!(
        pathspec_matching_nothing(stderr.as_slice(), WrittenPathspecs::Elements(elements.as_slice())),
        Some(vec![vec![0xff, 0xfe]])
    );
}

/// The message names the token Git refused and what happened to the commit,
/// and offers the spelling remedy when no token starts with a hyphen.
#[test]
fn the_message_names_the_token_and_what_became_of_the_commit() {
    let message: String = pathspec_matching_nothing_message(&[b"nope.txt".to_vec()]);
    assert!(message.contains("pathspec 'nope.txt' did not match any files"), "{message}");
    assert!(message.contains("this commit was not made and nothing was staged by it"), "{message}");
    assert!(message.contains("git add -- <path>"), "the remedy for a path that is new: {message}");
    assert!(!message.contains("options belong before"), "nothing here starts with a hyphen: {message}");
    assert!(!message.contains("--message <text>"), "{message}");
}

/// A token that begins with a hyphen also says where options belong,
/// which is the shape issue 6283 reported,
/// and it never tells the person to stage an option as a path.
#[test]
fn the_message_says_where_an_option_belongs_when_the_token_starts_with_a_hyphen() {
    let message: String = pathspec_matching_nothing_message(&[b"--message=second".to_vec()]);
    assert!(message.contains("pathspec '--message=second' did not match any files"), "{message}");
    assert!(message.contains("options belong before --"), "{message}");
    assert!(message.contains("git commit --message <text> -- <paths>"), "{message}");
    assert!(!message.contains("git add -- <path>"), "an option is not a path to stage: {message}");
}

/// Several tokens are listed together,
/// so the person sees every pathspec Git refused.
#[test]
fn the_message_lists_every_refused_token() {
    let message: String = pathspec_matching_nothing_message(&[b"first.txt".to_vec(), b"second.txt".to_vec()]);
    assert!(message.contains("pathspec 'first.txt', 'second.txt' did not match any files"), "{message}");
}
