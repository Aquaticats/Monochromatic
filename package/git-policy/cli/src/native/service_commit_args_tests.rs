//! Controls for which command lines the service covers, and a differential check of the message clean-up against the
//!  message real `git commit -m` stores.

use super::*;
use crate::test_support::{fixture, git, git_text, remove, repository};

fn args(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The plain scoped commit is covered, in each spelling of the message and the quiet flag.
#[test]
fn plain_scoped_commits_are_covered() {
    let one = ServiceCommitArgs { messages: vec![String::from("subject")], ..ServiceCommitArgs::default() };
    assert_eq!(parse_service_commit(&args(&["-m", "subject", "--", "a.txt"])), Some(one.clone()));
    assert_eq!(parse_service_commit(&args(&["-msubject", "a.txt"])), Some(one.clone()));
    assert_eq!(parse_service_commit(&args(&["--message=subject", "--", "a.txt"])), Some(one.clone()));
    assert_eq!(parse_service_commit(&args(&["--message", "subject", "a.txt"])), Some(one));
    let quiet = ServiceCommitArgs { messages: vec![String::from("s"), String::from("body")], quiet: true, ..ServiceCommitArgs::default() };
    assert_eq!(parse_service_commit(&args(&["-qm", "s", "-m", "body", "--only", "--", "a.txt"])), Some(quiet.clone()));
    assert_eq!(parse_service_commit(&args(&["--quiet", "--no-verify", "-m", "s", "-m", "body", "-o", "--", "a.txt"])), Some(quiet));
}

/// The options that only shape the message or the identity are covered, in each spelling.
#[test]
fn message_and_identity_options_are_covered() {
    let file = parse_service_commit(&args(&["-F", "m.txt", "--", "a.txt"])).expect("covered");
    assert_eq!(file.file, Some(OsString::from("m.txt")));
    assert_eq!(parse_service_commit(&args(&["-Fm.txt", "a.txt"])), Some(file.clone()));
    assert_eq!(parse_service_commit(&args(&["--file=m.txt", "a.txt"])), Some(file.clone()));
    assert_eq!(parse_service_commit(&args(&["--file", "m.txt", "a.txt"])), Some(file));
    let full = parse_service_commit(&args(&[
        "-m", "s", "--author=A U <a@b.c>", "--date", "2020-01-02T03:04:05Z", "--trailer=Fixes: 1", "--trailer", "Cc: x", "--cleanup=verbatim",
        "--allow-empty-message", "--", "a.txt",
    ]))
    .expect("covered");
    assert_eq!(full.author.as_deref(), Some("A U <a@b.c>"));
    assert_eq!(full.date.as_deref(), Some("2020-01-02T03:04:05Z"));
    assert_eq!(full.trailers, vec![String::from("Fixes: 1"), String::from("Cc: x")]);
    assert!(full.verbatim && full.allow_empty_message);
    let whitespace = parse_service_commit(&args(&["-m", "s", "--cleanup", "whitespace", "--no-edit", "--no-gpg-sign", "--", "a.txt"])).expect("covered");
    assert!(!whitespace.verbatim);
}

/// `-a` and `--all` only widen what the selection takes, which the preparation decides; the command line is covered.
#[test]
fn all_is_covered() {
    for option in ["-a", "--all", "-qa", "-am"] {
        let words: Vec<&str> = if option == "-am" { vec![option, "s"] } else { vec!["-m", "s", option] };
        assert!(parse_service_commit(&args(words.as_slice())).is_some(), "{option}");
    }
}

/// Sign-off, empty commits, reused messages, fixups and the strip clean-up are covered, with their values.
#[test]
fn text_arithmetic_options_are_covered() {
    assert!(parse_service_commit(&args(&["-m", "s", "-s", "--", "a"])).expect("covered").signoff);
    assert!(parse_service_commit(&args(&["-m", "s", "--signoff", "--allow-empty", "--", "a"])).expect("covered").signoff);
    let reuse = parse_service_commit(&args(&["-C", "HEAD~1", "--", "a"])).expect("covered");
    assert_eq!(reuse.reuse.as_deref(), Some("HEAD~1"));
    assert_eq!(parse_service_commit(&args(&["--reuse-message=HEAD~1", "a"])), Some(reuse.clone()));
    assert_eq!(parse_service_commit(&args(&["-CHEAD~1", "a"])), Some(reuse));
    let fixup = parse_service_commit(&args(&["--fixup=HEAD", "--", "a"])).expect("covered");
    assert_eq!(fixup.fixup, Some((FixupKind::Fixup, String::from("HEAD"))));
    assert!(parse_service_commit(&args(&["-m", "s", "--cleanup=strip", "--", "a"])).expect("covered").strip);
    assert!(parse_service_commit(&args(&["-m", "s", "--cleanup=strip", "--cleanup=whitespace", "--", "a"])).is_some_and(|parsed| { return !parsed.strip; }));
}

/// A message from two places, or a reused commit with an author of its own, is Git's to refuse or answer.
#[test]
fn conflicting_sources_are_not_covered() {
    for words in [
        vec!["-C", "HEAD", "-m", "s"],
        vec!["-C", "HEAD", "--author=A <a@b>"],
        vec!["-C", "HEAD", "--date=2020-01-01T00:00:00Z"],
        vec!["--fixup=HEAD", "-m", "s"],
        vec!["--fixup=HEAD", "-C", "HEAD"],
    ] {
        assert_eq!(parse_service_commit(&args(&words)), None, "{words:?}");
    }
}

/// Everything else is left to the transaction: amend, editor, standard input, a message from two places, signing, sign-off,
///  an author Git would have to look up, an unknown clean-up mode, `-a`.
#[test]
fn other_options_are_not_covered() {
    for option in ["--amend", "-c", "-S", "-e", "--edit", "-t", "-v", "-u", "--author=somebody", "--cleanup=scissors", "--reset-author", "-F-",
        "--gpg-sign", "--fixup=amend:HEAD", "--fixup=reword:HEAD", "--squash=HEAD", "--squash"] {
        assert_eq!(parse_service_commit(&args(&["-m", "s", option, "--", "a.txt"])), None, "{option}");
    }
    assert_eq!(parse_service_commit(&args(&["-F", "m.txt", "-m", "s", "--", "a.txt"])), None, "two places for the message");
    assert_eq!(parse_service_commit(&args(&["-F", "-", "--", "a.txt"])), None, "standard input is Git's to read");
    assert_eq!(parse_service_commit(&args(&["--", "a.txt"])), None, "no message means the editor");
    assert_eq!(parse_service_commit(&args(&["-m"])), None, "a message option with no text");
}

/// The clean-up gives the same stored message as Git for a table of awkward texts.
#[test]
fn the_message_cleanup_matches_what_git_stores() {
    let root = fixture("service-commit-cleanup");
    let repo = repository(root.as_path(), "repo");
    let cases: [&[&str]; 9] = [
        &["subject"],
        &["subject", "body"],
        &["subject  \n\n\n\nbody with trailing   \nnext line  "],
        &["\n\n  leading blanks kept inside the line\n\n"],
        &["a", "", "b"],
        &["line one\nline two\n"],
        &["tab\tinside\t\nafter"],
        &["# not a comment in this mode"],
        &["ünï code ✓", "second"],
    ];
    for (n, texts) in cases.iter().enumerate() {
        std::fs::write(repo.join("f.txt"), format!("{n}\n")).expect("write");
        git(repo.as_path(), &["add", "f.txt"]);
        let mut arguments: Vec<String> = vec![String::from("commit"), String::from("--quiet"), String::from("--allow-empty-message")];
        for text in *texts {
            arguments.push(String::from("-m"));
            arguments.push((*text).to_string());
        }
        let stored_args: Vec<&str> = arguments.iter().map(String::as_str).collect();
        git(repo.as_path(), &stored_args);
        let stored: String = git_text(repo.as_path(), &["log", "-1", "--format=%B"]);
        let owned: Vec<String> = texts.iter().map(|text| { return (*text).to_string(); }).collect();
        match cleanup_message(&owned) {
            Some(made) => assert_eq!(format!("{}\n", stored.trim_end_matches('\n')), made, "case {n}: {texts:?}"),
            None => assert!(stored.trim().is_empty(), "case {n}"),
        }
    }
    remove(root.as_path());
}

/// A message of only blanks is no message.
#[test]
fn a_blank_message_is_no_message() {
    assert_eq!(cleanup_message(&[String::from("  \n\n \t")]), None);
    assert_eq!(cleanup_message(&[]), None);
}
