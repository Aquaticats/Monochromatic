//! Controls for the Git version check:
//!  the `git version` parser,
//!  the message,
//!  and the answer
//! of a real `git version` run through a fixture executable that prints a chosen release.

use super::*;
use crate::test_support::{executable, fixture, git_context, remove};
use crate::transaction_git::GitContext;
use std::path::PathBuf;
use std::sync::OnceLock;

/// The release lines `git version` prints,
///  with the release each one carries.
#[test]
fn release_numbers_are_read_from_plain_and_suffixed_output() {
    assert_eq!(parse_git_version(b"git version 2.56.0\n"), Some((2, 56, 0)));
    assert_eq!(parse_git_version(b"git version 2.56.0.windows.1\n"), Some((2, 56, 0)));
    assert_eq!(parse_git_version(b"git version 2.56.0-rc1\n"), Some((2, 56, 0)));
    assert_eq!(parse_git_version(b"git version 2.56.0a\n"), Some((2, 56, 0)));
    assert_eq!(parse_git_version(b"git version 2.55.9\n"), Some((2, 55, 9)));
    assert_eq!(parse_git_version(b"git version 10.0.12 extra\n"), Some((10, 0, 12)));
}

/// Output that is not a `git version` line,
///  or whose numbers are missing or not numeric,
///  is refused.
#[test]
fn unreadable_version_output_is_refused() {
    assert_eq!(parse_git_version(b""), None);
    assert_eq!(parse_git_version(b"version 2.56.0\n"), None);
    assert_eq!(parse_git_version(b"git version 2.56\n"), None);
    assert_eq!(parse_git_version(b"git version x.56.0\n"), None);
    assert_eq!(parse_git_version(b"git version 2.56.x\n"), None);
    assert_eq!(parse_git_version(b"git version \n"), None);
    assert_eq!(parse_git_version(&[0xff, 0xfe, 0x00]), None);
}

/// The minimum is 2.55.0 itself,
///  which is supported,
///  and every release before it is not.
#[test]
fn the_minimum_is_inclusive() {
    assert_eq!(MINIMUM_GIT_VERSION, (2, 55, 0));
    assert!((2, 55, 0) >= MINIMUM_GIT_VERSION);
    assert!((2, 56, 0) >= MINIMUM_GIT_VERSION);
    assert!((3, 0, 0) >= MINIMUM_GIT_VERSION);
    assert!((2, 54, 9) < MINIMUM_GIT_VERSION);
    assert!((2, 50, 0) < MINIMUM_GIT_VERSION);
}

/// The unsupported message names the Git that was asked,
///  what it reported,
///  and the remedy.
#[test]
fn the_unsupported_message_names_the_git_and_the_remedy() {
    let context: GitContext = git_context();
    let message: String = unsupported_message(&context, "version 2.55.0");
    assert!(message.contains("/usr/bin/git"));
    assert!(message.contains("version 2.55.0"));
    assert!(message.contains("2.55.0 or newer"));
    assert!(message.contains("Upgrade Git"));
}

/// Build a fixture Git that answers `git version` with `line` and exits with success.
#[cfg(unix)]
fn fake_git(name: &str, line: &str) -> (PathBuf, GitContext) {
    let root: PathBuf = fixture(name);
    let path: PathBuf = root.join("git");
    executable(
        path.as_path(),
        format!("#!/bin/sh\necho '{line}'\n").as_bytes(),
    );
    let context: GitContext = GitContext {
        real_git: path.clone(),
        overlay: crate::test_support::isolated_overlay(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    return (root, context);
}

/// A real `git version` run through the production runner reports the release it prints.
#[test]
#[cfg(unix)]
fn a_supported_release_is_accepted() {
    let (root, context): (PathBuf, GitContext) = fake_git("git-version-supported", "git version 2.55.0");
    assert_eq!(ask_git_version(&context), Ok((2, 55, 0)));
    remove(root.as_path());
}

/// A release before 2.55.0 is refused with the reported release and the remedy.
#[test]
#[cfg(unix)]
fn an_older_release_is_refused() {
    let (root, context): (PathBuf, GitContext) = fake_git("git-version-older", "git version 2.54.0");
    let refusal: String = ask_git_version(&context).expect_err("2.54.0 must be refused");
    assert!(refusal.contains("version 2.54.0"));
    assert!(refusal.contains("2.55.0 or newer"));
    remove(root.as_path());
}

/// A `git version` answer that cannot be read is refused and shows what was printed.
#[test]
#[cfg(unix)]
fn an_unreadable_answer_is_refused() {
    let (root, context): (PathBuf, GitContext) = fake_git("git-version-unreadable", "hello");
    let refusal: String = ask_git_version(&context).expect_err("unreadable output must be refused");
    assert!(refusal.contains("an unreadable version"));
    assert!(refusal.contains("hello"));
    remove(root.as_path());
}

/// A Git that cannot start is refused,
///  naming the failure,
///  and nothing is guessed.
#[test]
fn a_missing_git_is_refused() {
    let root: PathBuf = fixture("git-version-missing");
    let context: GitContext = GitContext {
        real_git: root.join("no-such-git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    let refusal: String = ask_git_version(&context).expect_err("a missing Git must be refused");
    assert!(refusal.contains("nothing ("));
    remove(root.as_path());
}

/// Asks made by the counting fixture,
///  so the controls can prove how many Git processes were run.
static ASKS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// A version answer that counts its calls and reports a supported release.
fn counting_supported(_context: &GitContext) -> Result<(u64, u64, u64), String> {
    ASKS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    return Ok((2, 55, 0));
}

/// The first answer is kept,
///  so later dependent steps in the same process never ask Git again.
#[test]
fn the_answer_is_asked_once_and_then_reused() {
    let cell: OnceLock<Result<(u64, u64, u64), String>> = OnceLock::new();
    let context: GitContext = git_context();
    let before: usize = ASKS.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(check_once(&cell, &context, counting_supported), Ok((2, 55, 0)));
    assert_eq!(check_once(&cell, &context, counting_supported), Ok((2, 55, 0)));
    assert_eq!(ASKS.load(std::sync::atomic::Ordering::SeqCst) - before, 1);
}

/// A refusal is kept as well,
///  so an unsupported Git is reported the same way every time.
#[test]
fn a_refusal_is_reused_without_asking_again() {
    fn refused(context: &GitContext) -> Result<(u64, u64, u64), String> {
        return Err(unsupported_message(context, "version 2.55.0"));
    }
    let cell: OnceLock<Result<(u64, u64, u64), String>> = OnceLock::new();
    let context: GitContext = git_context();
    let first: Result<(u64, u64, u64), String> = check_once(&cell, &context, refused);
    let second: Result<(u64, u64, u64), String> = check_once(&cell, &context, counting_supported);
    assert_eq!(first, second);
    assert!(first.expect_err("refused").contains("version 2.55.0"));
}

/// The process-wide check answers what Git reports, not a constant: it equals a direct query of the same Git.
#[test]
fn the_process_check_answers_what_git_reports() {
    let context: GitContext = git_context();
    let direct: Result<GitRelease, String> = ask_git_version(&context);
    let checked: Result<GitRelease, String> = check_git_version(&context);
    assert_eq!(checked, direct);
    assert!(checked.expect("the host Git is supported") >= MINIMUM_GIT_VERSION);
}
