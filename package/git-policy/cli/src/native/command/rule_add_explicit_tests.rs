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
        "monochromatic-git-policy-cli: git add rejects bulk-staging patterns ({tokens}) because they sweep up paths \
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
        (vec!["add", ":/*"], ":/*"),
        (vec!["add", ":/**"], ":/**"),
        (vec!["add", ":(top)"], ":(top)"),
        (vec!["add", "**"], "**"),
        (vec!["add", ":(glob)**"], ":(glob)**"),
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

/// Divergence:
///  spellings that select the whole repository and that the fixed list never held.
/// Why:
///  `git add` reads a pathspec by Git's grammar,
/// so `.//`, `././`, `sub/..`, case-insensitive magic and an
///  exclusion each sweep the tree,
/// and a list of exact tokens cannot name them all.
///  Real Git stages every file for each of them (`command_add_tests.rs` holds that control).
#[test]
fn spellings_the_fixed_list_never_held_are_refused() {
    for spelling in [
        ".//",
        "././",
        "sub/..",
        "sub/../",
        "sub/../.",
        ":(icase)*",
        ":!nothing-here",
        ":^nothing-here",
    ] {
        assert_eq!(decide(&["add", spelling]), rejection(spelling), "{spelling}");
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
        vec!["add", ":(top)sub/file"],
        vec!["add", ":(literal)**"],
    ] {
        assert_eq!(decide(values.as_slice()), None, "{values:?}");
    }
}

/// A caller whose own command reads pathspecs literally stages by its own spelling:
/// `--literal-pathspecs add -- '**'` stages one file named `**`,
/// so the spelling is no bulk pathspec.
/// `.` and `./` still name the working directory and take everything below it in either mode.
#[test]
fn a_literal_pathspec_mode_caller_stages_by_its_own_spelling() {
    for spelling in ["*", "**", ":(top)", ":(glob)**", ":/", ":/*", ":/**"] {
        assert_eq!(decide(&["--literal-pathspecs", "add", "--", spelling]), None, "{spelling}");
        assert_eq!(decide(&["add", "--", spelling]), rejection(spelling), "without the mode {spelling} still sweeps the tree");
    }
    for spelling in [".", "./"] {
        assert_eq!(decide(&["--literal-pathspecs", "add", "--", spelling]), rejection(spelling), "a directory spelling still stages everything below it");
    }
    assert_eq!(decide(&["--literal-pathspecs", "--no-literal-pathspecs", "add", "**"]), rejection("**"), "the later global option wins");
    assert_eq!(decide(&["--no-literal-pathspecs", "--literal-pathspecs", "add", "**"]), None);
    assert_eq!(decide(&["--literal-pathspecs", "add", "-A"]), rejection("-A"), "a flag is no pathspec and stays bulk");
}

/// A value a global option consumes is no mode of the caller's:
/// `git -C --literal-pathspecs add -- '**'` changes into the directory of that name,
/// and Git then expands the pattern over the whole worktree.
#[test]
fn a_global_option_value_is_no_pathspec_mode() {
    assert_eq!(decide(&["-C", "--literal-pathspecs", "add", "--", "**"]), rejection("**"));
    assert_eq!(decide(&["-c", "--literal-pathspecs", "add", "--", "**"]), rejection("**"));
    assert_eq!(decide(&["--git-dir", "--literal-pathspecs", "add", "--", "."]), rejection("."));
    assert_eq!(
        decide(&["-C", "--literal-pathspecs", "--literal-pathspecs", "add", "--", "**"]),
        None,
        "the option after the value is the caller's own mode again"
    );
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
