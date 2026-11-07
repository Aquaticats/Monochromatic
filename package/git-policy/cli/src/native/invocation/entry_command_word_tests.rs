#![cfg(unix)]

//! What:
//!  Controls on the two spellings of the management command word.
//! Why:
//!  The owner renamed `git monochromatic-git-policy-cli <command>` to `git monochromatic-git-policy-cli <command>` and decided the earlier
//!  spelling "still works,
//! with a one-line notice":
//! the same behaviour for every command,
//! and exactly one line more on
//!  standard error that names the new spelling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(plan(['cli-git', 'doctor'])).toEqual(withNotice(plan(['monochromatic-git-policy-cli', 'doctor'])));
//! ```

/// The planner and the notice under test.
use super::{EARLIER_COMMAND_WORD_NOTICE, MANAGEMENT_COMMAND, plan_invocation};
use super::tests::{inputs, text};
use crate::action::{Action, with_leading_notice};
use crate::management_arguments::MANAGEMENT_HELP;
use crate::real_git::ResolutionInputs;
use crate::test_support::{fixture, remove, repository};
use crate::wrapper_names_old_and_new::COMMAND_WORD;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Every management command the help lists,
///  with the options that make it harmless in an empty repository.
const MANAGEMENT_COMMANDS: [&[&str]; 14] = [
    &["--help"],
    &["-h"],
    &["doctor"],
    &["check", "--all"],
    &["fix", "--all"],
    &["check"],
    &["trust"],
    &["untrust"],
    &["status"],
    &["service", "status"],
    &["migrate-to-subvolume", "--help"],
    &["convert-worktree-to-subvolume", "--help"],
    &["migrate-to-subvolume", "--dry-run", "--no-such-option"],
    &["no-such-command"],
];

/// The arguments of one invocation:
///  the repository directory,
///  the command word,
///  then the command.
fn invocation_with(directory: &Path, word: &str, command: &[&str]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = vec![OsString::from("-C"), directory.as_os_str().to_os_string(), OsString::from(word)];
    arguments.extend(text(command));
    return arguments;
}

/// The program's own name for the command is the crate's name,
///  and the earlier word is the one the notice retires.
#[test]
fn the_management_word_is_the_crate_name_and_the_earlier_word_is_named() {
    assert_eq!(MANAGEMENT_COMMAND, "monochromatic-git-policy-cli");
    assert_eq!(COMMAND_WORD.earlier, "cli-git");
}

/// The new word answers every command exactly as the earlier word does,
///  and the earlier word adds one notice line before the same output.
#[test]
fn the_earlier_word_does_the_same_as_the_new_word_plus_one_notice_for_every_command() {
    let root: PathBuf = fixture("command-word-every-command");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for command in MANAGEMENT_COMMANDS {
        let current: Action = plan_invocation(invocation_with(repo.as_path(), COMMAND_WORD.current, command).as_slice(), &[], &resolution);
        let earlier: Action = plan_invocation(invocation_with(repo.as_path(), COMMAND_WORD.earlier, command).as_slice(), &[], &resolution);
        match &current {
            Action::Exit { stderr, .. } => {
                assert!(!stderr.contains(EARLIER_COMMAND_WORD_NOTICE), "the new word gives no notice: {command:?}: {stderr}");
            }
            other => panic!("the management namespace is answered by the wrapper: {command:?}: {other:?}"),
        }
        assert_eq!(earlier, with_leading_notice(current, Some(EARLIER_COMMAND_WORD_NOTICE)), "{command:?}");
    }
    remove(root.as_path());
}

/// The notice is one line,
///  names the new spelling and the earlier one,
///  and ends with a line feed.
#[test]
fn the_notice_is_one_line_that_names_both_spellings() {
    assert_eq!(EARLIER_COMMAND_WORD_NOTICE.matches('\n').count(), 1, "{EARLIER_COMMAND_WORD_NOTICE:?}");
    assert!(EARLIER_COMMAND_WORD_NOTICE.ends_with('\n'));
    assert!(EARLIER_COMMAND_WORD_NOTICE.contains(&format!("git {}", COMMAND_WORD.current)), "{EARLIER_COMMAND_WORD_NOTICE}");
    assert!(EARLIER_COMMAND_WORD_NOTICE.contains(&format!("git {}", COMMAND_WORD.earlier)), "{EARLIER_COMMAND_WORD_NOTICE}");
}

/// Help through either word shows the same text,
///  and that text spells the command with the new word only.
#[test]
fn help_names_the_new_word_only() {
    let root: PathBuf = fixture("command-word-help");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for word in [COMMAND_WORD.current, COMMAND_WORD.earlier] {
        match plan_invocation(text(&[word, "--help"]).as_slice(), &[], &resolution) {
            Action::Exit { code, stdout, .. } => {
                assert_eq!(code, 0);
                assert_eq!(stdout, MANAGEMENT_HELP);
            }
            other => panic!("help is answered by the wrapper: {other:?}"),
        }
    }
    assert!(MANAGEMENT_HELP.contains(&format!("git {} <command>", COMMAND_WORD.current)));
    assert!(!MANAGEMENT_HELP.contains(&format!("git {} ", COMMAND_WORD.earlier)), "{MANAGEMENT_HELP}");
    remove(root.as_path());
}

/// Global options before the word and wrapper controls before it do not hide either spelling.
#[test]
fn global_options_do_not_hide_either_word() {
    let root: PathBuf = fixture("command-word-globals");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for word in [COMMAND_WORD.current, COMMAND_WORD.earlier] {
        for before in [vec!["--no-pager"], vec!["-c", "a.b=c"], vec!["--cli-git-keep-going"]] {
            let mut arguments: Vec<&str> = before;
            arguments.push(word);
            arguments.push("--help");
            match plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution) {
                Action::Exit { code, stdout, .. } => {
                    assert_eq!((code, stdout.as_str()), (0, MANAGEMENT_HELP), "{arguments:?}");
                }
                other => panic!("{arguments:?} must be answered by the wrapper: {other:?}"),
            }
        }
    }
    remove(root.as_path());
}

/// A word that merely starts with either spelling is a Git command and is forwarded,
///  not taken for the management word.
#[test]
fn a_longer_word_is_not_the_management_word() {
    let root: PathBuf = fixture("command-word-prefix");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for word in ["monochromatic-git-policy-cli-x", "cli-git-x", "cli-gi", "monochromatic-git-policy-cl"] {
        let planned: Action = plan_invocation(text(&["-C", ".", word, "--help"]).as_slice(), &[], &resolution);
        assert!(!matches!(&planned, Action::Exit { stdout, .. } if stdout == MANAGEMENT_HELP), "{word}: {planned:?}");
    }
    remove(root.as_path());
}
