//! What:
//!  Value-consumption cases for every arity,
//!  with a real-Git control for the arity
//!       `git rev-parse --parseopt` cannot express.
//! Why:
//!  Whether the next token is a value decides whether it is a pathspec.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseOptions(['--contains', '-v']).occurrences[0].value).toBeDefined();
//! ```

/// Tokenizer types,
///  the shared synthetic table and the real-Git fixture helpers.
use crate::command_options::{
    Occurrence, OptionError, OptionErrorKind, OptionValue, ParsedOptions,
};
use crate::command_test_support::{
    CONTAINS, MESSAGE, fixture, git, git_status, parse_synthetic, remove, repository,
};
use std::path::PathBuf;
use std::process::Output;

/// Values on flags and on negated options are refused.
#[test]
fn refuses_unexpected_values() {
    for values in [vec!["--all=1"], vec!["--no-message=x"], vec!["--amend="]] {
        assert_eq!(
            parse_synthetic(values.as_slice()),
            Err(OptionError {
                kind: OptionErrorKind::UnexpectedValue,
                token: 0
            }),
            "{values:?}"
        );
    }
}

/// A negated value option reads no value,
///  so the next token stays positional.
#[test]
fn a_negated_value_option_reads_no_value() {
    let unset: ParsedOptions = parse_synthetic(&["--no-message", "x"]).expect("valid");
    assert_eq!(
        unset.occurrences,
        vec![Occurrence {
            id: MESSAGE,
            negated: true,
            value: None,
            token: 0
        }]
    );
    assert_eq!(unset.leading, vec![1]);
}

/// An optional value is taken only when attached with `=`.
#[test]
fn an_optional_long_value_never_takes_the_next_token() {
    let bare: ParsedOptions = parse_synthetic(&["--untracked-files", "no"]).expect("valid");
    assert_eq!(bare.occurrences[0].value, None);
    assert_eq!(bare.leading, vec![1]);
    let joined: ParsedOptions = parse_synthetic(&["--untracked-files=no", "x"]).expect("valid");
    assert!(joined.occurrences[0].value.is_some());
    assert_eq!(joined.leading, vec![1]);
}

/// A required value takes the next token whatever it looks like,
///  and is refused without one.
#[test]
fn a_required_value_takes_any_next_token() {
    for value in ["--", "-a", "--all", "", "-"] {
        let parsed: ParsedOptions = parse_synthetic(&["--message", value, "x"]).expect("valid");
        assert_eq!(
            parsed.occurrences[0].value,
            Some(OptionValue::Detached { token: 1 }),
            "{value}"
        );
        assert_eq!(parsed.occurrences.len(), 1, "{value}");
        assert_eq!(parsed.leading, vec![2], "{value}");
    }
    assert_eq!(
        parse_synthetic(&["x", "--message"]),
        Err(OptionError {
            kind: OptionErrorKind::MissingValue,
            token: 1
        })
    );
}

/// `PARSE_OPT_LASTARG_DEFAULT`:
///  a value unless the option ends the region.
#[test]
fn reads_last_argument_default_options() {
    let last: ParsedOptions = parse_synthetic(&["--contains"]).expect("valid");
    assert_eq!(
        last.occurrences,
        vec![Occurrence {
            id: CONTAINS,
            negated: false,
            value: None,
            token: 0
        }]
    );
    let followed: ParsedOptions = parse_synthetic(&["--contains", "-a"]).expect("valid");
    assert_eq!(
        followed.occurrences,
        vec![Occurrence {
            id: CONTAINS,
            negated: false,
            value: Some(OptionValue::Detached { token: 1 }),
            token: 0
        }]
    );
    let joined: ParsedOptions = parse_synthetic(&["--contains=x", "y"]).expect("valid");
    assert_eq!(joined.leading, vec![1]);
}

/// `PARSE_OPT_LASTARG_DEFAULT` has no `--parseopt` spelling,
///  so `git branch --contains`
/// (parse-options.h:615-625) is observed directly.
#[test]
fn last_argument_default_matches_git_branch_contains() {
    let directory: PathBuf = fixture("lastarg-default");
    let root: PathBuf = repository(directory.as_path(), "repository");
    // Last argument: Git uses the default `HEAD` and lists the branch.
    let last: Output = git(root.as_path(), &["branch", "--contains"]);
    assert!(String::from_utf8_lossy(&last.stdout).contains("main"));
    // Followed by a dash-led token: Git reads the token as the commit, not as `-v`.
    let followed: Output = git_status(root.as_path(), &["branch", "--contains", "-v"]);
    assert!(!followed.status.success());
    assert!(
        String::from_utf8_lossy(&followed.stderr).contains("malformed object name -v"),
        "{}",
        String::from_utf8_lossy(&followed.stderr)
    );
    remove(directory.as_path());
}
