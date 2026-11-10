//! What:
//!  A guard over every Git process the built wrapper starts for a command that has a pathspec:
//! no option of the wrapper's
//!  follows the `--` the person typed,
//! and no `--quiet` or `-q` reaches Git unless the person typed one.
//! Why:
//!  After `--` Git reads every token as a path,
//! so an option the wrapper adds there is silently taken for a file name:
//! it
//!  commits a file named `--quiet`,
//! or fails with `pathspec '--quiet' did not match`.
//! The wrapper also adds no `--quiet`
//!  of its own to a command (owner decision,
//! "No the wrapper should never append --quiet.
//! Letting native git emit whatever
//!  it wants to is expected."),
//! so what Git prints reaches the person as Git prints it.
//!  A stand-in Git logs the arguments of each start and runs real Git,
//! so the guard sees the commands the wrapper builds
//!  and not only the commands the person typed.
//! The checks are plain functions,
//! tried on arguments with the fault planted.

use super::commit::{COMMIT_BOUND_SECONDS, remove_settled};
use super::service_commit::{Running, runtime_for, start};
use super::service_sequencer::State;
use super::support::{Fixture, bounded_for, executable, fixture};
use git_policy_cli::global_arguments::global_layout;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Output;

/// The variable that names the file the stand-in Git appends to.
const LOG_VARIABLE: &str = "CLI_GIT_TEST_ARGV_LOG";

/// The separator between the arguments of one logged start (unit separator),
/// so no argument can be split.
const ARGUMENT_SEPARATOR: char = '\u{1f}';

/// A stand-in Git that logs the arguments of every start,
/// one line per start,
/// then runs real Git.
fn install_logging_git(fixture: &Fixture) {
    std::fs::create_dir(fixture.root.join("shim")).expect("shim directory");
    executable(
        fixture.root.join("shim/git").as_path(),
        b"#!/bin/sh\nprintf '%s\\037' \"$@\" >> \"$CLI_GIT_TEST_ARGV_LOG\"\nprintf '\\n' >> \"$CLI_GIT_TEST_ARGV_LOG\"\nexec /usr/bin/git \"$@\"\n",
    );
}

/// Runs a wrapped command whose real Git is the logging stand-in and returns what it showed with every start it made.
fn run_logged(fixture: &Fixture, repo: &Path, service: &str, arguments: &[&str]) -> (Output, Vec<Vec<String>>) {
    let log: PathBuf = fixture.root.join("argv.log");
    std::fs::write(&log, b"").expect("empty log");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("shim"));
    let output: Output = bounded_for(fixture, fixture.root.join("bin/git").as_path(), path.as_os_str(), COMMIT_BOUND_SECONDS)
        .current_dir(repo)
        .args(arguments)
        .env("XDG_RUNTIME_DIR", runtime_for(service))
        .env(LOG_VARIABLE, &log)
        .output()
        .expect("the wrapped command runs");
    let starts: Vec<Vec<String>> = std::fs::read_to_string(&log)
        .expect("the log")
        .lines()
        .map(|line| { return line.split(ARGUMENT_SEPARATOR).filter(|word| { return !word.is_empty(); }).map(String::from).collect(); })
        .collect();
    return (output, starts);
}

/// The index of the subcommand of a logged start:
/// the first word that is not a global option or its value.
fn subcommand_index(start: &[String]) -> usize {
    let words: Vec<OsString> = start.iter().map(OsString::from).collect();
    return global_layout(words.as_slice()).prefix_len;
}

/// Whether `word` contains one of the words the person typed after their `--`.
fn names_a_typed_path(word: &str, typed: &[&str]) -> bool {
    return typed.iter().any(|path| { return word.contains(path); });
}

/// What:
///  The words of one logged start that follow a `--` and are neither paths the person typed nor pathspecs the wrapper wrote.
/// Why:
///   A start is judged only when a `--` is followed by a word carrying a path the person typed,
/// which is the person's
///  separator carried into the wrapper's command;
/// a start whose `--` ends the wrapper's own pathspecs is not about it.
///  Whatever else follows is a wrapper option read by Git as a path.
///  A `:(top,literal)<path>` or `:/` word is a pathspec and never an option,
/// and the wrapper lists the
///  commit's changed index paths beside the person's,
/// so those words are the wrapper's own paths and not the fault this guard exists for.
fn wrapper_words_after_separator(start: &[String], typed: &[&str]) -> Vec<String> {
    let from: usize = subcommand_index(start) + 1;
    let Some(separator) = start.iter().skip(from).position(|word| { return word == "--"; }).map(|at| { return at + from; }) else {
        return Vec::new();
    };
    let after: &[String] = &start[separator + 1..];
    if !after.iter().any(|word| { return names_a_typed_path(word, typed); }) {
        return Vec::new();
    }
    return after
        .iter()
        .filter(|word| { return !names_a_typed_path(word, typed) && !is_a_wrapper_pathspec(word); })
        .cloned()
        .collect();
}

/// Whether a word is a pathspec the wrapper wrote:
/// `:/`,
/// or `:(top,literal)<path>`.
/// Why:
///   Git reads a pathspec after `--` as a path,
/// so it can never be the option this guard is about,
///  and the wrapper lists the changed paths of the commit with that magic.
fn is_a_wrapper_pathspec(word: &str) -> bool {
    return word == ":/" || word.starts_with(":(top,literal)");
}

/// Whether a word asks for quiet:
/// `--quiet`,
/// or a short cluster holding `q` (`-q`,
/// `-qn`).
fn asks_quiet(word: &str) -> bool {
    return word == "--quiet" || (word.starts_with('-') && !word.starts_with("--") && word.len() > 1 && word[1..].chars().all(|letter| { return letter.is_ascii_lowercase(); }) && word.contains('q'));
}

/// What:
///  The quiet words of a start of the person's own subcommand when the person typed none.
/// Why:
///   The wrapper's other Git starts use `--quiet` for what it asks them (`rev-parse --verify --quiet`),
/// which are not the
///  person's command;
/// a start of the same subcommand as the person's is,
/// and carries only what they typed,
/// before the
///  separator and after it.
fn quiet_the_person_did_not_type(start: &[String], subcommand: &str, typed: &[&str]) -> Vec<String> {
    let at: usize = subcommand_index(start);
    if start.get(at).map(String::as_str) != Some(subcommand) || typed.iter().any(|word| { return asks_quiet(word); }) {
        return Vec::new();
    }
    return start[at + 1..].iter().filter(|word| { return asks_quiet(word); }).cloned().collect();
}

/// The check applied to every logged start of one command;
/// the messages name the start.
fn violations(starts: &[Vec<String>], subcommand: &str, typed_arguments: &[&str], typed_paths: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for start in starts {
        let after: Vec<String> = wrapper_words_after_separator(start, typed_paths);
        if !after.is_empty() {
            found.push(format!("wrapper words {after:?} follow the person's -- in {start:?}"));
        }
        let quiet: Vec<String> = quiet_the_person_did_not_type(start, subcommand, typed_arguments);
        if !quiet.is_empty() {
            found.push(format!("the wrapper asked {subcommand} for quiet ({quiet:?}) in {start:?}"));
        }
    }
    return found;
}

fn words(text: &[&str]) -> Vec<String> {
    return text.iter().map(|word| { return (*word).to_string(); }).collect();
}

/// The checks catch the faults they exist for,
/// and pass what they must:
/// the fault planted in a start,
/// the same words typed
///  by the person,
/// a `--` that ends the wrapper's own pathspecs,
/// a query's own `--quiet`,
/// and the person's own `-q`.
#[test]
fn the_checks_catch_a_wrapper_option_after_the_separator_and_a_quiet_nobody_typed() {
    let planted: Vec<Vec<String>> = vec![words(&["-c", "k=v", "commit", "-o", "--message=undo", "--", "a.txt", "--quiet"])];
    let found: Vec<String> = violations(&planted, "commit", &["commit", "--message=undo", "--", "a.txt"], &["a.txt"]);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].contains("follow the person's --") && found[0].contains("\"--quiet\""), "{found:?}");
    assert!(found[1].contains("asked commit for quiet"), "{found:?}");
    let clean: Vec<Vec<String>> = vec![
        words(&["commit", "-o", "--message=undo", "--", "a.txt"]),
        words(&["diff-index", "--quiet", "--cached", "HEAD", "--"]),
        words(&["rev-parse", "--verify", "--quiet", "MERGE_HEAD"]),
        words(&["add", "--update", "--", ":/"]),
        words(&["add", "--all", "--", "a.txt"]),
        words(&["-c", "commit", "diff", "--", ":(top,literal)a.txt"]),
        // The wrapper lists the commit's changed index paths beside the person's, each with the magic it writes, and none of
        //  them is an option.
        words(&["commit", "-o", "--message=undo", "--", ":(top,literal)--quiet", ":(top,literal)-dash.txt", ":(top,literal)-o", ":(top,literal)a.txt"]),
        words(&["diff-index", "--cached", "--raw", "HEAD", "--", ":(top,literal)a.txt", ":(top,literal)b.txt"]),
    ];
    assert_eq!(violations(&clean, "commit", &["commit", "--message=undo", "--", "a.txt"], &["a.txt"]), Vec::<String>::new());
    let typed: Vec<Vec<String>> = vec![words(&["commit", "-qn", "--message=undo", "--", "a.txt"])];
    assert_eq!(violations(&typed, "commit", &["commit", "-qn", "--message=undo", "--", "a.txt"], &["a.txt"]), Vec::<String>::new());
    let message_value: Vec<Vec<String>> = vec![words(&["commit", "-m", "quick", "--", "a.txt"])];
    assert_eq!(violations(&message_value, "commit", &["commit", "-m", "quick", "--", "a.txt"], &["a.txt"]), Vec::<String>::new());
}

/// The commands the guard runs besides commits,
/// each with a pathspec after the person's `--`.
const OTHER_COMMANDS: [&[&str]; 9] = [
    &["add", "--", "a.txt"],
    &["reset", "--", "a.txt"],
    &["restore", "--staged", "--", "a.txt"],
    &["checkout", "--", "a.txt"],
    &["rm", "--cached", "--", "b.txt"],
    &["diff", "--", "a.txt"],
    &["status", "--short", "--", "a.txt"],
    &["log", "--oneline", "--", "a.txt"],
    &["stash", "push", "--message=guard", "--", "a.txt"],
];

/// No Git process the wrapper starts for a commit with a pathspec,
/// during a revert or a cherry-pick,
/// carries a word of the
///  wrapper's after the person's `--` or a `--quiet` the person did not type;
/// a `--quiet` they typed reaches Git.
#[test]
fn no_commit_the_wrapper_builds_has_its_own_option_after_the_separator() {
    let fixture: Fixture = fixture("option-order-commit");
    install_logging_git(&fixture);
    let cases: Vec<(State, Vec<&str>, Vec<&str>)> = vec![
        (State::Revert, vec!["commit", "--message=undo", "--", "a.txt"], vec!["a.txt"]),
        (State::Revert, vec!["commit", "--message=undo", "--", "-dash.txt"], vec!["-dash.txt"]),
        (State::Revert, vec!["commit", "--quiet", "--message=undo", "--", "a.txt"], vec!["a.txt"]),
        (State::CherryPick, vec!["commit", "--message=undo", "--", "a.txt"], vec!["a.txt"]),
        (State::CherryPick, vec!["commit", "--message=undo", "--", "-dash.txt"], vec!["-dash.txt"]),
        (State::CherryPick, vec!["commit", "-q", "--message=undo", "--", "a.txt"], vec!["a.txt"]),
    ];
    let repos: Vec<PathBuf> = cases.iter().enumerate().map(|(index, (state, _, _))| { return state.prepare(&fixture, format!("guard-{index}").as_str()); }).collect();
    let name: String = format!("og-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let mut found: Vec<String> = Vec::new();
    for ((state, typed, paths), repo) in cases.iter().zip(repos.iter()) {
        let (output, starts) = run_logged(&fixture, repo, name.as_str(), typed.as_slice());
        assert!(output.status.success(), "{state:?} {typed:?}: {}", String::from_utf8_lossy(&output.stderr));
        let built: Vec<&Vec<String>> = starts.iter().filter(|start| { return start.iter().any(|word| { return word.starts_with("--git-dir=") && word.contains("shadow"); }) && start.get(subcommand_index(start)).map(String::as_str) == Some("commit"); }).collect();
        assert_eq!(built.len(), 1, "{state:?} {typed:?}: the commit Git builds in the scratch repository is the one start this guard is about: {starts:?}");
        if typed.iter().any(|word| { return asks_quiet(word); }) {
            assert!(built[0].iter().any(|word| { return asks_quiet(word); }), "{state:?} {typed:?}: the person's quiet reaches Git: {:?}", built[0]);
        }
        found.extend(violations(starts.as_slice(), "commit", typed.as_slice(), paths.as_slice()).into_iter().map(|line| { return format!("{state:?} {typed:?}: {line}"); }));
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// No Git process the wrapper starts for another command with a pathspec carries a word of the wrapper's after the
///  person's `--`,
/// or a `--quiet` the person did not type in the person's own subcommand.
#[test]
fn no_other_command_the_wrapper_builds_has_its_own_option_after_the_separator() {
    let fixture: Fixture = fixture("option-order-other");
    install_logging_git(&fixture);
    let repo: PathBuf = State::CherryPick.prepare(&fixture, "guard-other");
    let name: String = format!("oo-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let mut found: Vec<String> = Vec::new();
    let mut started: usize = 0;
    for command in OTHER_COMMANDS {
        let paths: Vec<&str> = command.iter().copied().skip_while(|word| { return *word != "--"; }).skip(1).collect();
        let (_output, starts) = run_logged(&fixture, &repo, name.as_str(), command);
        started += starts.len();
        found.extend(violations(starts.as_slice(), command[0], command, paths.as_slice()).into_iter().map(|line| { return format!("{command:?}: {line}"); }));
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
    assert!(started > 0, "the log saw Git starts, so an empty finding means something");
    assert!(found.is_empty(), "{}", found.join("\n"));
}
