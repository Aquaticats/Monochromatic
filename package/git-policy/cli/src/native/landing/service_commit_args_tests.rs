//! Controls for which command lines the service covers,
//! and a differential check of the message clean-up against the
//!  message real `git commit -m` stores.

use super::*;
use crate::test_support::{fixture, git, git_text, remove, repository};

fn args(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The plain scoped commit is covered,
/// in each spelling of the message and the quiet flag.
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

/// The options that only shape the message or the identity are covered,
/// in each spelling.
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

/// `-a` and `--all` only widen what the selection takes,
/// which the preparation decides;
/// the command line is covered.
#[test]
fn all_is_covered() {
    for option in ["-a", "--all", "-qa", "-am"] {
        let words: Vec<&str> = if option == "-am" { vec![option, "s"] } else { vec!["-m", "s", option] };
        assert!(parse_service_commit(&args(words.as_slice())).is_some(), "{option}");
    }
}

/// Sign-off,
/// empty commits,
/// reused messages,
/// fixups and the strip clean-up are covered,
/// with their values.
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

/// A message from two places,
/// or a reused commit with an author of its own,
/// is Git's to refuse or answer.
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

/// Everything else is left to the transaction:
/// editor,
/// standard input,
/// a message from two places,
/// signing,
/// sign-off,
///  an author Git would have to look up,
/// an unknown clean-up mode,
/// `-a`.
#[test]
fn other_options_are_not_covered() {
    for option in ["-c", "-S", "-e", "--edit", "-t", "-v", "-u", "--author=somebody", "--cleanup=scissors", "--reset-author", "-F-",
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

/// What:
///  A line loses the white space Git removes from its end and nothing else.
/// Why:
///  The clean-up trimmed every character Rust calls white space,
/// and the audit found a no-break space at the end of a message removed where real Git
///  records it.
///  Measured against Git 2.56.0,
/// a trailing no-break space,
/// a trailing figure space,
/// a trailing vertical tab and a trailing form feed are all recorded,
///  while a trailing space,
/// tab,
/// line break or carriage return is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cleanupMessage(['subject '])).toEqual('subject \n');
/// ```
#[test]
fn a_line_keeps_the_white_space_git_does_not_remove() {
    for kept in ["\u{a0}", "\u{2007}", "\u{202f}", "\u{b}", "\u{c}"] {
        let text: String = format!("subject{kept}");
        assert_eq!(cleanup_message(&[text]), Some(format!("subject{kept}\n")), "{kept:?}");
    }
    for removed in [" ", "\t", "\n", "\r", "\r\n", "  \t "] {
        assert_eq!(cleanup_message(&[format!("subject{removed}")]), Some(String::from("subject\n")), "{removed:?}");
    }
    // A line of a character Git does not remove is a line of the message, so the paragraph keeps it.
    assert_eq!(cleanup_message(&[String::from("\u{a0}")]), Some(String::from("\u{a0}\n")));
    // Git's own joining of the lines of one text and of two texts.
    assert_eq!(cleanup_message(&[String::from("one\n  two\n")]), Some(String::from("one\n  two\n")));
    assert_eq!(cleanup_message(&[String::from("one"), String::from("two")]), Some(String::from("one\n\ntwo\n")));
}

/// A message of only blanks is no message.
#[test]
fn a_blank_message_is_no_message() {
    assert_eq!(cleanup_message(&[String::from("  \n\n \t")]), None);
    assert_eq!(cleanup_message(&[]), None);
}

/// An amend is covered with a message option,
/// and without one only when `--no-edit` says no editor opens.
#[test]
fn an_amend_is_covered_with_a_message_or_no_edit() {
    let with_message = parse_service_commit(&args(&["--amend", "-m", "again", "--", "a.txt"])).expect("covered");
    assert!(with_message.amend && !with_message.reset_author);
    let kept = parse_service_commit(&args(&["--amend", "--no-edit", "--", "a.txt"])).expect("covered");
    assert!(kept.amend && kept.messages.is_empty());
    assert!(parse_service_commit(&args(&["--amend", "--no-edit"])).is_some_and(|parsed| { return parsed.amend; }), "no path at all");
    assert_eq!(parse_service_commit(&args(&["--amend", "--", "a.txt"])), None, "no message and no `--no-edit` opens the editor");
    assert_eq!(parse_service_commit(&args(&["--no-edit", "--", "a.txt"])), None, "`--no-edit` keeps nothing without an amend");
    assert!(parse_service_commit(&args(&["--amend", "-C", "HEAD~1", "--", "a.txt"])).is_some_and(|parsed| { return parsed.amend && parsed.reuse.is_some(); }));
}

/// A line with a message and nothing that asks for an editor opens none;
/// every way of asking for one,
/// and every option this
///  reading does not know,
/// counts as an editor.
#[test]
fn the_editor_is_predicted_from_the_message_source_and_errs_towards_opening() {
    for words in [
        vec!["-m", "s", "--", "a.txt"],
        vec!["-qm", "s"],
        vec!["-ms"],
        vec!["--message=s", "-S"],
        vec!["-F", "m.txt", "--gpg-sign=KEY"],
        vec!["--amend", "--no-edit"],
        vec!["-C", "HEAD"],
        vec!["--fixup=HEAD"],
        vec!["-e", "-m", "s", "--no-edit"],
        vec!["-m", "s", "--author", "A <a@b>", "--date", "2020-01-01T00:00:00Z", "--trailer", "Fixes: 1"],
    ] {
        assert!(!opens_editor(&args(&words)), "{words:?}");
    }
    for words in [
        vec!["--", "a.txt"],
        vec!["--amend"],
        vec!["-e", "-m", "s"],
        vec!["--edit", "-m", "s"],
        vec!["-m", "s", "-e"],
        vec!["-c", "HEAD"],
        vec!["--squash=HEAD"],
        vec!["--fixup=amend:HEAD"],
        vec!["--fixup=reword:HEAD", "-m", "s"],
        vec!["-m", "s", "--ed"],
        vec!["-m", "s", "--no-e"],
        vec!["-m", "s", "--surprise"],
        vec!["-m", "s", "-Z"],
        vec!["--no-edit", "-e"],
    ] {
        assert!(opens_editor(&args(&words)), "{words:?}");
    }
}

/// The options that choose what to commit (a selection,
/// a pathspec file) do not change the message,
/// so a line that also has
///  a message is covered,
/// and none of them makes the editor open.
#[test]
fn selection_options_are_covered_and_open_no_editor() {
    let message = ServiceCommitArgs { messages: vec![String::from("s")], ..ServiceCommitArgs::default() };
    for words in [
        vec!["-i", "-m", "s", "--", "a.txt"],
        vec!["--include", "-m", "s", "--", "a.txt"],
        vec!["-p", "-m", "s"],
        vec!["--patch", "-m", "s"],
        vec!["--interactive", "-m", "s"],
        vec!["-m", "s", "--pathspec-from-file=list.txt"],
        vec!["-m", "s", "--pathspec-from-file", "list.txt", "--pathspec-file-nul"],
    ] {
        assert_eq!(parse_service_commit(&args(&words)), Some(message.clone()), "{words:?}");
    }
    for words in [vec!["-i", "-m", "s", "--", "a.txt"], vec!["-m", "s", "--pathspec-from-file", "list.txt", "--pathspec-file-nul"], vec!["-m", "s", "--include"]] {
        assert!(!opens_editor(&args(&words)), "{words:?}");
    }
}

/// A commit whose message is fixed is rebuilt from the saved message:
/// every message source and the editor options go,
/// the
///  rest of the line stays,
/// and the pathspecs after the separator are untouched.
#[test]
fn a_fixed_message_replaces_every_message_option_and_keeps_the_rest() {
    let file = std::path::Path::new("/saved/message");
    let rebuilt = |words: &[&str], at: usize| -> Option<Vec<String>> {
        return with_fixed_message(&args(words), at, file).map(|found| { return found.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect(); });
    };
    let tail: [&str; 4] = ["-F", "/saved/message", "--cleanup=verbatim", "--no-edit"];
    let expected = |before: &[&str], after: &[&str]| -> Vec<String> {
        return before.iter().chain(tail.iter()).chain(after.iter()).map(|word| { return (*word).to_string(); }).collect();
    };
    assert_eq!(rebuilt(&["commit", "-m", "s", "--", "a.txt"], 0), Some(expected(&["commit"], &["--", "a.txt"])));
    // A message that is `--` is dropped with its option, and the separator the person typed after it still ends the options.
    assert_eq!(rebuilt(&["commit", "-m", "--", "--", "a.txt"], 0), Some(expected(&["commit"], &["--", "a.txt"])));
    assert_eq!(rebuilt(&["commit", "--message", "--", "--", "--quiet"], 0), Some(expected(&["commit"], &["--", "--quiet"])));
    assert_eq!(
        rebuilt(&["commit", "-e", "-m", "seed", "-S", "--author=A <a@b>", "--trailer", "T: v", "--cleanup=strip", "--", "a.txt"], 0),
        Some(expected(&["commit", "-S", "--author=A <a@b>"], &["--", "a.txt"]))
    );
    assert_eq!(rebuilt(&["-c", "core.x=1", "commit", "-qsm", "x"], 2), Some(expected(&["-c", "core.x=1", "commit", "-q"], &[])));
    assert_eq!(rebuilt(&["commit", "--message=a", "--file", "f", "--template=t", "--signoff", "--edit"], 0), Some(expected(&["commit"], &[])));
    for words in [
        vec!["commit", "-c", "HEAD"],
        vec!["commit", "-C", "HEAD"],
        vec!["commit", "--amend", "--no-edit"],
        vec!["commit", "--fixup=HEAD"],
        vec!["commit", "--squash=HEAD"],
        vec!["commit", "--reedit-message=HEAD"],
        vec!["commit", "--ame"],
        vec!["commit", "--mess=x"],
    ] {
        assert_eq!(rebuilt(&words, 0), None, "{words:?}");
    }
}

/// `--reset-author` is covered only for an amend that names no author,
/// no date and no reused commit.
#[test]
fn reset_author_is_covered_only_for_a_plain_amend() {
    assert!(parse_service_commit(&args(&["--amend", "--reset-author", "--no-edit", "--", "a.txt"])).is_some_and(|parsed| { return parsed.reset_author; }));
    for words in [
        vec!["--reset-author", "-m", "s"],
        vec!["--amend", "--reset-author", "-m", "s", "--author=A <a@b>"],
        vec!["--amend", "--reset-author", "-m", "s", "--date=2020-01-01T00:00:00Z"],
        vec!["--amend", "--reset-author", "-C", "HEAD~1"],
    ] {
        assert_eq!(parse_service_commit(&args(&words)), None, "{words:?}");
    }
}
