//! What:
//!  The reading of each command that can discard uncommitted state,
//! in every spelling Git accepts.
//! Why:
//!  A destructive spelling read as harmless lets `git checkout -- .` or `git rm -rf` run in the primary checkout;
//!  a harmless one read as destructive blocks routine work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(checkoutDiscard(['--', 'a.txt'])).toBe('overwrites');
//! ```

/// The readings and the argument builder.
use super::{
    Discard, abort_discard, checkout_discard, force_discard, previous_branch_expression,
    read_tree_discard, restore_discard, restores_index_only, switch_discard,
};
use crate::command_test_support::{fixture, git, git_status, os_arguments, remove, repository, repository_with_tracked_file};
use std::path::{Path, PathBuf};
use std::process::Output;

/// Every read of a checkout region that Git accepts.
#[test]
fn checkout_regions_are_read_by_what_they_overwrite() {
    for (values, expected) in [
        (vec![], Discard::Harmless),
        (vec!["-q"], Discard::Harmless),
        (vec!["-b", "topic"], Discard::Harmless),
        (vec!["topic"], Discard::MovesHead),
        (vec!["-"], Discard::MovesHead),
        (vec!["--detach", "HEAD~1"], Discard::MovesHead),
        (vec!["-b", "topic", "origin/main"], Discard::MovesHead),
        (vec!["--", "a.txt"], Discard::Overwrites),
        (vec!["--"], Discard::Overwrites),
        (vec!["."], Discard::MovesHead),
        (vec!["HEAD", "a.txt"], Discard::Overwrites),
        (vec!["HEAD", "--", "a.txt"], Discard::Overwrites),
        (vec!["-f"], Discard::Overwrites),
        (vec!["--force", "topic"], Discard::Overwrites),
        (vec!["--forc", "topic"], Discard::Overwrites),
        (vec!["-qf", "topic"], Discard::Overwrites),
        (vec!["-f", "--no-force", "topic"], Discard::MovesHead),
        (vec!["-p"], Discard::Overwrites),
        (vec!["--patch", "HEAD"], Discard::Overwrites),
        (vec!["--ours", "--", "a.txt"], Discard::Overwrites),
        (vec!["--theirs", "a.txt", "b.txt"], Discard::Overwrites),
        (vec!["--pathspec-from-file=list"], Discard::Overwrites),
        (vec!["--pathspec-from-file", "list"], Discard::Overwrites),
    ] {
        assert_eq!(
            checkout_discard(os_arguments(values.as_slice()).as_slice()),
            Ok(expected),
            "{values:?}"
        );
    }
}

/// A checkout region Git refuses is the refusal,
/// never a reading.
#[test]
fn a_refused_checkout_region_is_an_error() {
    assert!(checkout_discard(os_arguments(&["--no-such-option"]).as_slice()).is_err());
    assert!(checkout_discard(os_arguments(&["-b"]).as_slice()).is_err());
}

/// Every read of a switch region that Git accepts.
#[test]
fn switch_regions_are_read_by_what_they_overwrite() {
    for (values, expected) in [
        (vec![], Discard::Harmless),
        (vec!["-c", "topic"], Discard::Harmless),
        (vec!["topic"], Discard::MovesHead),
        (vec!["-"], Discard::MovesHead),
        (vec!["--detach", "HEAD~1"], Discard::MovesHead),
        (vec!["-m", "topic"], Discard::MovesHead),
        (vec!["-f", "topic"], Discard::Overwrites),
        (vec!["--force", "topic"], Discard::Overwrites),
        (vec!["--discard-changes", "topic"], Discard::Overwrites),
        (vec!["--discard", "topic"], Discard::Overwrites),
        (vec!["--discard-changes", "--no-discard-changes", "topic"], Discard::MovesHead),
    ] {
        assert_eq!(
            switch_discard(os_arguments(values.as_slice()).as_slice()),
            Ok(expected),
            "{values:?}"
        );
    }
    assert!(switch_discard(os_arguments(&["--no-such-option"]).as_slice()).is_err());
}

/// Every restore form overwrites;
/// only a request for usage text does not.
#[test]
fn every_restore_form_overwrites_except_usage() {
    for (values, expected) in [
        (vec![], Discard::Overwrites),
        (vec!["a.txt"], Discard::Overwrites),
        (vec!["--staged", "a.txt"], Discard::Overwrites),
        (vec!["--source=HEAD", "--staged", "--worktree", "--", "."], Discard::Overwrites),
        (vec!["-p"], Discard::Overwrites),
        (vec!["--", "-h"], Discard::Overwrites),
        (vec!["-h"], Discard::Harmless),
        (vec!["--help"], Discard::Harmless),
        (vec!["--he"], Discard::Harmless),
    ] {
        assert_eq!(
            restore_discard(os_arguments(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// Force spells discard for `rm`,
/// `mv` and `checkout-index`,
/// in every cluster and abbreviation.
#[test]
fn force_in_any_spelling_overwrites() {
    for (values, expected) in [
        (vec![], Discard::Harmless),
        (vec!["a.txt"], Discard::Harmless),
        (vec!["-r", "--cached", "a.txt"], Discard::Harmless),
        (vec!["-n", "-k", "a", "b"], Discard::Harmless),
        (vec!["--prefix=force/", "-a"], Discard::Harmless),
        (vec!["-f", "a.txt"], Discard::Overwrites),
        (vec!["-rf", "dir"], Discard::Overwrites),
        (vec!["-fr", "dir"], Discard::Overwrites),
        (vec!["--force", "a.txt"], Discard::Overwrites),
        (vec!["--forc", "a.txt"], Discard::Overwrites),
        (vec!["--fo", "a.txt"], Discard::Overwrites),
        (vec!["--force=x", "a.txt"], Discard::Overwrites),
        (vec!["--", "-f"], Discard::Harmless),
        (vec!["--fe", "a.txt"], Discard::Harmless),
    ] {
        assert_eq!(
            force_discard(os_arguments(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// `read-tree --reset` in every abbreviation discards;
/// the other modes do not.
#[test]
fn read_tree_reset_overwrites() {
    for (values, expected) in [
        (vec!["HEAD"], Discard::Harmless),
        (vec!["-m", "-u", "HEAD"], Discard::Harmless),
        (vec!["--recurse-submodules", "HEAD"], Discard::Harmless),
        (vec!["--reset", "HEAD"], Discard::Overwrites),
        (vec!["--res", "HEAD"], Discard::Overwrites),
        (vec!["-u", "--reset", "HEAD"], Discard::Overwrites),
        (vec!["--re", "HEAD"], Discard::Harmless),
        (vec!["--", "--reset"], Discard::Harmless),
    ] {
        assert_eq!(
            read_tree_discard(os_arguments(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// `--abort` in every abbreviation discards;
/// continuing does not.
#[test]
fn abort_in_any_abbreviation_overwrites() {
    for (values, expected) in [
        (vec![], Discard::Harmless),
        (vec!["topic"], Discard::Harmless),
        (vec!["--continue"], Discard::Harmless),
        (vec!["--autostash", "topic"], Discard::Harmless),
        (vec!["--abort"], Discard::Overwrites),
        (vec!["--abo"], Discard::Overwrites),
        (vec!["--ab"], Discard::Overwrites),
        (vec!["--a"], Discard::Harmless),
        (vec!["--", "--abort"], Discard::Harmless),
    ] {
        assert_eq!(
            abort_discard(os_arguments(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// Run real Git in a repository and return its standard error.
fn stderr_of(repo: &Path, values: &[&str]) -> String {
    let output = git_status(repo, values);
    return String::from_utf8_lossy(&output.stderr).into_owned();
}

/// Real Git accepts the shortest abbreviations the readings rely on;
/// `cherry-pick` and `revert` take `--abort`
///  only in full,
/// and reading shorter spellings as aborts only rejects a command Git refuses.
#[test]
fn real_git_accepts_the_shortest_abbreviations() {
    let root = fixture("discard-abbreviations");
    let repo = repository(root.as_path(), "repo");
    for values in [
        vec!["rm", "--fo", "--ignore-unmatch", "missing.txt"],
        vec!["mv", "--fo", "missing.txt", "other.txt"],
        vec!["checkout-index", "--fo", "--all"],
        vec!["read-tree", "--res", "HEAD"],
        vec!["switch", "--discard", "main"],
        vec!["checkout", "--forc", "main"],
    ] {
        let stderr = stderr_of(repo.as_path(), values.as_slice());
        assert!(
            !stderr.contains("unknown option")
                && !stderr.contains("ambiguous option")
                && !stderr.contains("unknown switch"),
            "{values:?}: {stderr}"
        );
    }
    for (values, mark) in [
        (vec!["merge", "--ab"], "no merge to abort"),
        (vec!["cherry-pick", "--abort"], "no cherry-pick or revert in progress"),
        (vec!["revert", "--abort"], "no cherry-pick or revert in progress"),
        (vec!["rebase", "--ab"], "no rebase in progress"),
    ] {
        let stderr = stderr_of(repo.as_path(), values.as_slice());
        assert!(stderr.contains(mark), "{values:?}: {stderr}");
    }
    remove(root.as_path());
}

/// Only a restore that names the index and not the worktree leaves the files alone.
#[test]
fn staged_alone_restores_the_index_only() {
    for (values, expected) in [
        (vec!["a.txt"], false),
        (vec!["--staged", "a.txt"], true),
        (vec!["-S", "a.txt"], true),
        (vec!["--st", "a.txt"], true),
        (vec!["--source=HEAD", "--staged", "a.txt"], true),
        (vec!["--staged", "--worktree", "a.txt"], false),
        (vec!["-SW", "a.txt"], false),
        (vec!["--staged", "--w", "a.txt"], false),
        (vec!["--worktree", "a.txt"], false),
        (vec!["--", "--staged"], false),
    ] {
        assert_eq!(
            restores_index_only(os_arguments(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// A token that Git takes as the value of `--source`,
/// `-s` or `--conflict` is not a flag:
/// real Git runs `restore --source -W
///  --staged -- f` and `restore -sW --staged -- f` as index-only restores (`-W` and `W` are tag names there),
/// so they are
///  index-only here,
/// and a real `-W` beside such a value still makes a worktree restore.
#[test]
fn an_option_value_is_not_a_restore_flag() {
    for (values, expected) in [
        (vec!["--source", "-W", "--staged", "--", "f"], true),
        (vec!["--staged", "--source", "-W", "--", "f"], true),
        (vec!["-S", "-s", "-W", "--", "f"], true),
        (vec!["--staged", "--conflict", "-W", "--", "f"], true),
        (vec!["-sW", "--staged", "--", "f"], true),
        (vec!["-Ss", "-W", "--", "f"], true),
        (vec!["--staged", "--so", "-W", "--", "f"], true),
        (vec!["--source=-W", "--staged", "--", "f"], true),
        (vec!["--source", "--staged", "--", "f"], false),
        (vec!["--source", "-S", "-W", "--", "f"], false),
        (vec!["--staged", "-W", "--source", "HEAD", "--", "f"], false),
        (vec!["-SW", "-s", "HEAD", "--", "f"], false),
        (vec!["--staged", "--worktree", "--source=-S", "--", "f"], false),
    ] {
        assert_eq!(restores_index_only(os_arguments(values.as_slice()).as_slice()), expected, "{values:?}");
    }
}

/// The one shorthand that only `switch` and `checkout` read becomes the expression Git resolves wherever an
/// object name is read,
/// and every other revision is left as it was written.
#[test]
fn the_previous_branch_shorthand_becomes_the_expression_git_resolves() {
    assert_eq!(previous_branch_expression(b"-"), Some(&b"@{-1}"[..]));
    for written in [
        &b"@{-1}"[..],
        &b"@{-2}"[..],
        &b"tracks"[..],
        &b"--"[..],
        &b"-f"[..],
        &b""[..],
    ] {
        assert_eq!(previous_branch_expression(written), None, "{written:?}");
    }
}

/// Real Git resolves the expression the shorthand becomes to the previous branch of a worktree.
#[test]
fn real_git_resolves_the_previous_branch_expression() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("previous-branch-expression");
    git(root.as_path(), &["branch", "tracks"]);
    git(root.as_path(), &["switch", "--quiet", "tracks"]);
    let main: String = String::from_utf8_lossy(git(root.as_path(), &["rev-parse", "--abbrev-ref", "HEAD"]).stdout.as_slice())
        .trim()
        .to_owned();
    assert_eq!(main, "tracks");
    // The worktree is on `tracks`, so the previous branch is the one before it.
    git(root.as_path(), &["switch", "--quiet", "tracks"]);
    let expression: &[u8] = previous_branch_expression(b"-").expect("the expression");
    let shown: String = String::from_utf8_lossy(expression).into_owned();
    let resolved: Output = git(root.as_path(), &["rev-parse", "--symbolic-full-name", shown.as_str()]);
    assert_eq!(
        String::from_utf8_lossy(resolved.stdout.as_slice()).trim(),
        "refs/heads/tracks"
    );
    remove(directory.as_path());
}
