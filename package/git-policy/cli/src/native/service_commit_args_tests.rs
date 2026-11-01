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
    let one = ServiceCommitArgs { messages: vec![String::from("subject")], quiet: false };
    assert_eq!(parse_service_commit(&args(&["-m", "subject", "--", "a.txt"])), Some(one.clone()));
    assert_eq!(parse_service_commit(&args(&["-msubject", "a.txt"])), Some(one.clone()));
    assert_eq!(parse_service_commit(&args(&["--message=subject", "--", "a.txt"])), Some(one.clone()));
    assert_eq!(parse_service_commit(&args(&["--message", "subject", "a.txt"])), Some(one));
    let quiet = ServiceCommitArgs { messages: vec![String::from("s"), String::from("body")], quiet: true };
    assert_eq!(parse_service_commit(&args(&["-qm", "s", "-m", "body", "--only", "--", "a.txt"])), Some(quiet.clone()));
    assert_eq!(parse_service_commit(&args(&["--quiet", "--no-verify", "-m", "s", "-m", "body", "-o", "--", "a.txt"])), Some(quiet));
}

/// Everything else is left to native `git commit`: amend, editor, files, authorship, signing, trailers, `-a`.
#[test]
fn other_options_are_not_covered() {
    for option in ["--amend", "-a", "--all", "-F", "--file=x", "-C", "--author=A <a@b>", "--date=now", "-S", "-s", "--signoff",
        "--allow-empty", "--fixup=HEAD", "-e", "--edit", "--no-edit", "--trailer=a:b", "--cleanup=verbatim", "-t", "-v", "-u"] {
        assert_eq!(parse_service_commit(&args(&["-m", "s", option, "--", "a.txt"])), None, "{option}");
    }
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
