//! What:
//!  Which `git add` invocations the add-explicit policy rejects,
//!  and the exact text.
//! Why:
//!  The text names the caller's own tokens and how many paths they swept up;
//!  a bulk form read as explicit stages the whole worktree,
//!  and an explicit form read as bulk blocks ordinary staging.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(decideAddExplicit(question(['add', 'file.ts']))).toBeUndefined();
//! ```

/// The decision,
///  its question,
///  its codes and the argument builders.
use super::{
    BULK_ADD_CODE, AddExplicitQuestion, BulkScope, StagedSelection, add_explicit_question,
    decide_add_explicit, pathspec_spellings, scope_of,
};
use crate::command_test_support::{byte_argument, os_arguments};
use std::ffi::OsString;

/// A measurement that found every path of a repository holding `count` paths.
fn whole_repository(count: usize) -> Option<StagedSelection> {
    return Some(StagedSelection {
        selected: count,
        repository: count,
        directory: count,
    });
}

/// A measurement that found every path of the directory the caller runs in,
///  `below` of the repository's `count`.
fn whole_directory(below: usize, count: usize) -> Option<StagedSelection> {
    return Some(StagedSelection {
        selected: below,
        repository: count,
        directory: below,
    });
}

/// A measurement that left `missing` paths of the repository out.
fn some_selected(selected: usize, count: usize) -> Option<StagedSelection> {
    return Some(StagedSelection {
        selected,
        repository: count,
        directory: count,
    });
}

/// The rejection text naming the given tokens and what they sweep up.
fn rejection(tokens: &str, swept: &str) -> Option<String> {
    return Some(format!(
        "monochromatic-git-policy-cli: git add rejects bulk-staging patterns ({tokens}) because they sweep up paths \
         the caller did not intend to stage, leaving the index in a state that does not match a \
         single logical change. {swept} Name the paths explicitly, or pass --no-enforce-bulk-add to \
         bypass for this invocation."
    ));
}

/// The sentence a selection of every path of the repository is reported with.
fn repository_paths(count: usize) -> String {
    return format!("It selects all {count} paths of this repository.");
}

/// The sentence a selection of every path of the caller's directory is reported with.
fn directory_paths(count: usize) -> String {
    return format!("It selects all {count} paths of this directory.");
}

/// The question one invocation asks,
///  or nothing when it stages nothing in bulk.
fn question(values: &[&str]) -> Option<AddExplicitQuestion> {
    return add_explicit_question(os_arguments(values).as_slice());
}

/// Decide one invocation written as text against a measured selection.
fn decide(values: &[&str], selection: Option<StagedSelection>) -> Option<String> {
    return decide_add_explicit(&question(values)?, selection.as_ref());
}

/// A flag stages every changed path whatever the tree holds,
///  so it is rejected without a measurement,
///  and the text says which set it sweeps up.
#[test]
fn bulk_flags_are_rejected_and_named() {
    assert_eq!(BULK_ADD_CODE, "bulk-add-rejected");
    for (values, tokens, swept) in [
        (vec!["add", "-A"], "-A", "It selects every changed path in this repository."),
        (vec!["add", "--all"], "--all", "It selects every changed path in this repository."),
        (vec!["add", "-vA"], "-vA", "It selects every changed path in this repository."),
        (vec!["add", "-Au"], "-Au", "It selects every changed path in this repository."),
        (vec!["add", "-u"], "-u", "It selects every changed tracked path in this repository."),
        (vec!["add", "--update"], "--update", "It selects every changed tracked path in this repository."),
        (vec!["-C", "dir", "--no-pager", "add", "-A"], "-A", "It selects every changed path in this repository."),
    ] {
        assert_eq!(
            decide(values.as_slice(), None),
            rejection(tokens, swept),
            "{values:?}"
        );
    }
}

/// Every pathspec spelling that selects every path of the repository is refused,
///  with the count Git gave,
///  including the spellings a fixed list of tokens never held.
#[test]
fn every_spelling_that_selects_the_whole_repository_is_refused() {
    for spelling in [
        ".",
        "./",
        ".//",
        "././",
        "./**",
        "*",
        "**",
        ":/*",
        ":/**",
        ":/",
        ":(top)",
        ":(glob)**",
        ":(top,glob)**",
        ":(top)**",
        ":(icase)*",
        ":(attr:!nope)",
        ":!nothing-here",
        ":^nothing-here",
        "sub/..",
        "sub/../",
        "sub/../.",
        "..",
    ] {
        assert_eq!(
            decide(&["add", spelling], whole_repository(5)),
            rejection(spelling, repository_paths(5).as_str()),
            "{spelling:?}"
        );
    }
    // The count is the one the measurement gave.
    assert_eq!(
        decide(&["add", "."], whole_repository(1234)),
        rejection(".", repository_paths(1234).as_str())
    );
}

/// Spellings the fixed list never held are refused however the caller writes them,
///  which is what the failing test that started this change asserted.
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
        assert_eq!(
            decide(&["add", spelling], whole_repository(5)),
            rejection(spelling, repository_paths(5).as_str()),
            "{spelling}"
        );
    }
}

/// A command run below the worktree's top sweeps up the paths of its own directory,
///  and the text names that set and its count.
#[test]
fn a_selection_of_the_callers_own_directory_is_refused() {
    assert_eq!(
        decide(&["add", "."], whole_directory(2, 5)),
        rejection(".", directory_paths(2).as_str())
    );
    assert_eq!(
        decide(&["add", "*"], whole_directory(7, 9)),
        rejection("*", directory_paths(7).as_str())
    );
}

/// A selection that leaves a path out is no bulk staging,
///  whatever spelling reached it.
#[test]
fn a_selection_that_leaves_a_path_out_is_let_through() {
    for spelling in [".", "*", "**", ":/", "sub/..", "*.txt", "./**"] {
        assert_eq!(
            decide(&["add", spelling], some_selected(4, 5)),
            None,
            "{spelling:?}"
        );
    }
    // Nothing selected at all is nothing swept up.
    assert_eq!(decide(&["add", "."], some_selected(0, 0)), None);
    assert_eq!(decide(&["add", "."], some_selected(0, 5)), None);
}

/// A spelling that names a character of a name stages everything only because the tree fits it,
///  so a repository whose every path fits is still staged by name.
#[test]
fn a_spelling_that_names_something_is_let_through_even_when_it_selects_everything() {
    for spelling in ["*.txt", "sub/*", "a.txt", ":(glob)**.txt", ":(literal)**"] {
        assert_eq!(
            decide(&["add", spelling], whole_repository(3)),
            None,
            "{spelling:?}"
        );
    }
    // A reach-only spelling beside a name is the one that is refused.
    assert_eq!(
        decide(&["add", "*.txt", "a.txt"], whole_repository(3)),
        None,
        "both spellings name something"
    );
    assert_eq!(
        decide(&["add", "*", "a.txt"], whole_repository(3)),
        rejection("*", repository_paths(3).as_str())
    );
}

/// Several pathspecs on one command are measured together,
///  and only the reach-only ones are named.
#[test]
fn several_pathspecs_are_judged_together() {
    let asked: AddExplicitQuestion = question(&["add", "-A", "file.ts", ".", "./x"]).expect("a question");
    let spellings: Vec<String> = pathspec_spellings(&asked)
        .iter()
        .map(|spelling| { return spelling.to_string_lossy().into_owned(); })
        .collect();
    assert_eq!(spellings, vec![String::from("."), String::from("./x")]);
    // The flag is named first because the caller wrote it first.
    let mixed: AddExplicitQuestion = question(&["add", ".", "-u", "file"]).expect("a question");
    assert_eq!(
        decide_add_explicit(&mixed, whole_repository(5).as_ref()),
        rejection("., -u", repository_paths(5).as_str())
    );
    // One cluster matching twice is named once.
    assert_eq!(
        decide_add_explicit(&question(&["add", "-Au", "."]).expect("a question"), whole_repository(2).as_ref()),
        rejection("-Au, .", repository_paths(2).as_str())
    );
}

/// A lookup that failed or timed out lets the pathspec through with the measurement missing,
///  while a flag is still refused.
#[test]
fn a_missing_measurement_lets_a_pathspec_through() {
    for spelling in [".", "*", ":(top)", "sub/.."] {
        assert_eq!(decide(&["add", spelling], None), None, "{spelling:?}");
    }
    assert_eq!(
        decide(&["add", "-A", "."], None),
        rejection("-A", "It selects every changed path in this repository.")
    );
    assert_eq!(
        decide(&["add", "-u"], None),
        rejection("-u", "It selects every changed tracked path in this repository.")
    );
}

/// A caller whose own command reads pathspecs literally stages by its own spelling:
/// `--literal-pathspecs add -- '**'` stages one file named `**`,
/// so the spelling is no reach-only patterns,
/// while `.` and `./` still name the working directory in either mode.
#[test]
fn a_literal_pathspec_mode_caller_stages_by_its_own_spelling() {
    for spelling in ["*", "**", ":(top)", ":(glob)**", ":/", ":/*", ":/**"] {
        assert_eq!(
            decide(&["--literal-pathspecs", "add", "--", spelling], whole_repository(1)),
            None,
            "{spelling:?} is a file name that happens to be the whole repository"
        );
    }
    for spelling in [".", "./"] {
        assert_eq!(
            decide(&["--literal-pathspecs", "add", "--", spelling], whole_repository(2)),
            rejection(spelling, repository_paths(2).as_str()),
            "{spelling:?} is a directory in either mode"
        );
    }
    assert_eq!(
        decide(&["--literal-pathspecs", "--no-literal-pathspecs", "add", "**"], whole_repository(2)),
        rejection("**", repository_paths(2).as_str()),
        "the later global option wins"
    );
    assert_eq!(
        decide(&["--no-literal-pathspecs", "--literal-pathspecs", "add", "**"], whole_repository(1)),
        None
    );
    assert_eq!(
        decide(&["--literal-pathspecs", "add", "-A"], None),
        rejection("-A", "It selects every changed path in this repository."),
        "a flag is no pathspec and stays bulk"
    );
}

/// A value a global option consumes is no mode of the caller's:
/// `git -C --literal-pathspecs add -- '**'` changes into the directory of that name,
/// and Git then expands the pattern over the whole worktree.
#[test]
fn a_global_option_value_is_no_pathspec_mode() {
    assert_eq!(
        decide(&["-C", "--literal-pathspecs", "add", "--", "**"], whole_repository(2)),
        rejection("**", repository_paths(2).as_str())
    );
    assert_eq!(
        decide(&["-c", "--literal-pathspecs", "add", "--", "**"], whole_repository(2)),
        rejection("**", repository_paths(2).as_str())
    );
    assert_eq!(
        decide(&["--git-dir", "--literal-pathspecs", "add", "--", "."], whole_repository(2)),
        rejection(".", repository_paths(2).as_str())
    );
    assert_eq!(
        decide(
            &["-C", "--literal-pathspecs", "--literal-pathspecs", "add", "--", "**"],
            whole_repository(1)
        ),
        None,
        "the option after the value is the caller's own mode again"
    );
}

/// Explicit paths,
///  other commands,
///  option values spelling a bulk token and regions Git refuses ask nothing.
#[test]
fn explicit_forms_ask_nothing() {
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
        vec!["add", ".hidden"],
    ] {
        assert_eq!(question(values.as_slice()), None, "{values:?}");
    }
    // A spelling that is no ordinary name is still let through when Git finds it selects a part of the tree.
    for spelling in ["./file.ts", ":(literal)**", "*.txt", "sub/*"] {
        assert_eq!(
            decide(&["add", spelling], some_selected(1, 4)),
            None,
            "{spelling:?}"
        );
    }
    // An ordinary name beside a bulk flag still leaves the flag to reject.
    let named: AddExplicitQuestion = question(&["add", "-A", "file.ts"]).expect("a question");
    assert_eq!(named.tokens.len(), 1);
    assert_eq!(
        decide(&["add", "-A", "file.ts"], None),
        rejection("-A", "It selects every changed path in this repository.")
    );
}

/// Which reachable set a measurement covers,
///  and nothing for a selection that leaves a path out.
#[test]
fn the_scope_of_a_measurement_names_the_set_it_covers() {
    assert_eq!(scope_of(&whole_repository(4).expect("a selection")), Some(BulkScope::Repository));
    assert_eq!(scope_of(&whole_directory(2, 9).expect("a selection")), Some(BulkScope::Directory));
    assert_eq!(scope_of(&some_selected(4, 9).expect("a selection")), None);
    assert_eq!(scope_of(&some_selected(0, 0).expect("a selection")), None);
    // The whole repository is named before the directory it also happens to be.
    assert_eq!(scope_of(&whole_repository(1).expect("a selection")), Some(BulkScope::Repository));
}

/// A token that is not UTF-8 beside a bulk token does not disturb the question.
#[cfg(unix)]
#[test]
fn undecodable_paths_do_not_disturb_the_decision() {
    let raw: OsString = byte_argument(b"f\xff");
    assert_eq!(
        add_explicit_question(&[OsString::from("add"), raw.clone()]),
        None
    );
    let mixed: AddExplicitQuestion =
        add_explicit_question(&[OsString::from("add"), raw, OsString::from(".")]).expect("a question");
    assert_eq!(
        decide_add_explicit(&mixed, whole_repository(2).as_ref()),
        rejection(".", repository_paths(2).as_str())
    );
}
