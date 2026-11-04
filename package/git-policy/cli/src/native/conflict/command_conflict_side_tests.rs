//! What:
//!  Which `checkout` and `restore` command lines take one side of a conflicted file,
//! in each spelling.
//! Why:
//!  A line that qualified by mistake would let a forced,
//! patched or unreadable form through the guard;
//! a line that was
//!  turned away by mistake blocks the command the owner decided to allow.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(conflictSideRequest('checkout', ['--theirs', '--', 'a.rs'])).toEqual({ paths: ['a.rs'] });
//! ```

use super::{SideRequest, conflict_side_request};
use crate::command_test_support::os_arguments;

/// The request a line gives,
/// with the paths as text.
fn paths_of(word: &str, values: &[&str]) -> Option<Vec<String>> {
    let region = os_arguments(values);
    return conflict_side_request(word.as_bytes(), region.as_slice())
        .map(|SideRequest { paths }| { return paths.iter().map(|path| { return path.to_string_lossy().into_owned(); }).collect(); });
}

/// Each checkout form that chooses a side,
/// names a revision,
/// or sets how a file is written qualifies and names its paths.
#[test]
fn checkout_forms_that_take_a_side_qualify() {
    for (values, expected) in [
        (vec!["--ours", "--", "a.rs"], vec!["a.rs"]),
        (vec!["--theirs", "--", "a.rs", "dir/b.rs"], vec!["a.rs", "dir/b.rs"]),
        (vec!["-2", "--", "a.rs"], vec!["a.rs"]),
        (vec!["-3", "--", "a.rs"], vec!["a.rs"]),
        (vec!["topic", "--", "a.rs"], vec!["a.rs"]),
        (vec!["HEAD~1", "--", "a.rs"], vec!["a.rs"]),
        (vec!["-m", "--", "a.rs"], vec!["a.rs"]),
        (vec!["--merge", "--", "a.rs"], vec!["a.rs"]),
        (vec!["--conflict=diff3", "--", "a.rs"], vec!["a.rs"]),
        (vec!["-q", "--theirs", "--", "a.rs"], vec!["a.rs"]),
        (vec!["--the", "--", "a.rs"], vec!["a.rs"]),
        (vec!["--", "a.rs"], vec!["a.rs"]),
    ] {
        assert_eq!(paths_of("checkout", values.as_slice()), Some(expected.iter().map(|each| { return String::from(*each); }).collect()), "{values:?}");
    }
}

/// Anything that forces,
/// patches,
/// reads paths from a file,
/// makes a branch,
/// has no `--`,
/// or names nothing after it does not qualify.
#[test]
fn checkout_forms_that_discard_more_or_say_less_do_not_qualify() {
    for values in [
        vec!["-f", "--", "a.rs"],
        vec!["--force", "--ours", "--", "a.rs"],
        vec!["-p", "--", "a.rs"],
        vec!["--patch", "topic", "--", "a.rs"],
        vec!["--pathspec-from-file=list", "--ours"],
        vec!["--pathspec-from-file=list", "--", "a.rs"],
        vec!["-b", "topic", "--", "a.rs"],
        vec!["-B", "topic", "--", "a.rs"],
        vec!["--orphan", "topic", "--", "a.rs"],
        vec!["-d", "--", "a.rs"],
        vec!["--ours", "a.rs"],
        vec!["topic", "a.rs"],
        vec!["--ours", "--"],
        vec!["topic", "other", "--", "a.rs"],
        vec!["--no-such-option", "--", "a.rs"],
        vec!["topic"],
        vec![],
    ] {
        assert_eq!(paths_of("checkout", values.as_slice()), None, "{values:?}");
    }
}

/// The restore forms that write worktree files from a side or a revision qualify;
/// index-only and abbreviated ones do not.
#[test]
fn restore_forms_are_read_exactly() {
    for (values, expected) in [
        (vec!["--ours", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--theirs", "--", "a.rs", "b.rs"], Some(vec!["a.rs", "b.rs"])),
        (vec!["--source=topic", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--source", "topic", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["-s", "HEAD~1", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["-W", "--source=topic", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--worktree", "--theirs", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["-S", "-W", "--theirs", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--staged", "--worktree", "--theirs", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["-m", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--conflict=zdiff3", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--conflict", "diff3", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["-S", "--theirs", "--", "a.rs"], None),
        (vec!["--staged", "--source=topic", "--", "a.rs"], None),
        (vec!["--theirs", "a.rs"], None),
        (vec!["--the", "--", "a.rs"], None),
        (vec!["-p", "--", "a.rs"], None),
        (vec!["--patch", "--", "a.rs"], None),
        (vec!["--pathspec-from-file=list"], None),
        (vec!["--theirs", "--"], None),
        (vec!["-sHEAD", "--", "a.rs"], None),
        (vec![], None),
    ] {
        let typed: Option<Vec<String>> = expected.map(|paths| { return paths.iter().map(|each| { return String::from(*each); }).collect(); });
        assert_eq!(paths_of("restore", values.as_slice()), typed, "{values:?}");
    }
}

/// Every other command word is not a way to take a side.
#[test]
fn other_commands_never_qualify() {
    for word in ["switch", "reset", "rm", "merge", "checkout-index", "stash", "add"] {
        assert_eq!(paths_of(word, &["--theirs", "--", "a.rs"]), None, "{word}");
    }
}

/// A token that Git takes as the value of `--source`,
/// `-s` or `--conflict` is not an option:
/// `-W` there does not make an
///  index-only restore a worktree restore,
/// and `-S` there does not make a worktree restore an index-only one.
#[test]
fn an_option_value_is_not_read_as_an_option() {
    for (values, expected) in [
        (vec!["--source", "-W", "--staged", "--", "a.rs"], None),
        (vec!["--staged", "--source", "-W", "--", "a.rs"], None),
        (vec!["-S", "-s", "-W", "--", "a.rs"], None),
        (vec!["--staged", "--conflict", "-W", "--", "a.rs"], None),
        (vec!["--staged", "--source", "--worktree", "--", "a.rs"], None),
        (vec!["--source=-W", "--staged", "--", "a.rs"], None),
        (vec!["--source", "-W", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--source", "--staged", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--source", "-S", "-W", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--staged", "-W", "--source", "HEAD", "--", "a.rs"], Some(vec!["a.rs"])),
        (vec!["--source", "HEAD", "--staged", "--worktree", "--", "a.rs"], Some(vec!["a.rs"])),
    ] {
        let typed: Option<Vec<String>> = expected.map(|paths| { return paths.iter().map(|each| { return String::from(*each); }).collect(); });
        assert_eq!(paths_of("restore", values.as_slice()), typed, "{values:?}");
    }
}
