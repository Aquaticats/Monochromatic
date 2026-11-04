//! What:
//!  Which `git add` invocations the add-explicit policy rejects,
//!  and the exact text.
//! Why:
//!  The text names the caller's own tokens;
//!  a bulk form read as explicit stages the
//!      whole worktree,
//!  and an explicit form read as bulk blocks ordinary staging.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(decideAddExplicit(['add', 'file.ts'])).toBeUndefined();
//! ```

/// The decision,
///  its code and the argument builders.
use super::{BULK_ADD_CODE, decide_add_explicit};
use crate::command_test_support::{byte_argument, os_arguments};
use std::ffi::OsString;

/// The rejection text naming the given tokens.
fn rejection(tokens: &str) -> Option<String> {
    return Some(format!(
        "cli-git: git add rejects bulk-staging patterns ({tokens}) because they sweep up paths \
         the caller did not intend to stage, leaving the index in a state that does not match a \
         single logical change. Name the paths explicitly, or pass --no-enforce-bulk-add to \
         bypass for this invocation."
    ));
}

/// Decide one invocation written as text.
fn decide(values: &[&str]) -> Option<String> {
    return decide_add_explicit(os_arguments(values).as_slice());
}

/// Bulk tokens are rejected and named as the caller wrote them,
///  in argument order,
///  each once.
#[test]
fn bulk_forms_are_rejected_and_named() {
    assert_eq!(BULK_ADD_CODE, "bulk-add-rejected");
    for (values, tokens) in [
        (vec!["add", "."], "."),
        (vec!["add", "-A"], "-A"),
        (vec!["add", "--all"], "--all"),
        (vec!["add", "-u"], "-u"),
        (vec!["add", "--update"], "--update"),
        (vec!["add", "*"], "*"),
        (vec!["add", "-vA"], "-vA"),
        (vec!["add", "-Au"], "-Au"),
        (vec!["add", "-A", "."], "-A, ."),
        (vec!["add", ".", "-u", "file"], "., -u"),
        (vec!["add", "--", "."], "."),
        (vec!["-C", "dir", "--no-pager", "add", "-A"], "-A"),
    ] {
        assert_eq!(decide(values.as_slice()), rejection(tokens), "{values:?}");
    }
}

/// Explicit paths,
///  other commands,
///  option values spelling a bulk token and regions Git refuses pass.
#[test]
fn explicit_forms_pass() {
    for values in [
        vec![],
        vec!["--version"],
        vec!["status", "."],
        vec!["commit", "-A"],
        vec!["add"],
        vec!["add", "file.ts"],
        vec!["add", "--", "dir/file"],
        vec!["add", "dir/"],
        vec!["add", "-p", "file"],
        vec!["add", "-A", "--no-all", "file"],
        vec!["add", "--pathspec-from-file", "-A"],
        vec!["add", "--pathspec-from-file=."],
        vec!["add", "--no-such-option", "."],
        vec!["--no-such-global", "add", "."],
        vec!["add", "./file"],
        vec!["add", ".hidden"],
    ] {
        assert_eq!(decide(values.as_slice()), None, "{values:?}");
    }
}

/// A token that is not UTF-8 beside a bulk token does not disturb the decision or the text.
#[test]
fn undecodable_paths_do_not_disturb_the_decision() {
    let raw: OsString = byte_argument(b"f\xff");
    assert_eq!(
        decide_add_explicit(&[OsString::from("add"), raw.clone()]),
        None
    );
    assert_eq!(
        decide_add_explicit(&[OsString::from("add"), raw, OsString::from(".")]),
        rejection(".")
    );
}
