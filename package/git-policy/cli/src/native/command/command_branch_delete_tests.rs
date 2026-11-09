//! What:
//!  Every `git branch` spelling that deletes a branch,
//!  and the forms that delete nothing.
//! Why:
//!  A reading form or a branch creation must write no audit line,
//!  and a deletion must be named with the
//!       branch Git will remove.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // branchDeletions(['-D', 'gone', 'other']) === ['gone', 'other']
//! ```

use super::{branch_deletion, branch_deletions};
use crate::command_test_support::{git, git_status, os_arguments, remove, repository};
use crate::test_support::fixture;
use std::ffi::OsString;
use std::path::PathBuf;

/// Read one region written as text.
fn deletions(values: &[&str]) -> Option<Vec<OsString>> {
    return branch_deletions(os_arguments(values).as_slice());
}

/// Both short forms,
///  the long form in its shortest accepted abbreviation,
///  and the clusters Git accepts.
#[test]
fn every_deleting_spelling_names_the_branches() {
    for values in [
        vec!["-d", "gone"],
        vec!["-D", "gone"],
        vec!["--delete", "gone"],
        vec!["--delete", "gone"],
        vec!["--delete", "--force", "gone"],
        vec!["-D", "gone", "other"],
        vec!["-rd", "gone"],
        vec!["-d", "--", "gone"],
        vec!["--quiet", "-D", "gone"],
    ] {
        let expected: Vec<&str> = values
            .iter()
            .copied()
            .filter(|token| return !token.starts_with('-'))
            .collect();
        assert_eq!(
            deletions(values.as_slice()),
            Some(os_arguments(expected.as_slice())),
            "{values:?}"
        );
    }
}

/// A form that creates,
///  moves,
///  lists or renames deletes nothing.
#[test]
fn a_form_that_deletes_no_branch_is_not_read() {
    for values in [
        vec![],
        vec!["-a"],
        vec!["--list"],
        vec!["-l"],
        vec!["topic"],
        vec!["-m", "old", "new"],
        vec!["-c", "old", "copy"],
        vec!["-f", "topic", "start"],
        vec!["-u", "upstream", "topic"],
        vec!["--contains", "HEAD"],
        // A name that begins with `-` after the separator is a name, not an option.
        vec!["--", "-weird"],
        // Every abbreviation of `--delete` is ambiguous with `--delete-merged`, and that one deletes only branches
        //  a merged branch already contains, so it cannot lose a commit.
        vec!["--delet", "gone"],
        vec!["--delete-merged", "origin/main"],
    ] {
        assert_eq!(deletions(values.as_slice()), None, "{values:?}");
    }
}

/// Real Git:
///  every spelling the reader calls a deletion really deletes a branch,
/// and a creation the reader ignores really creates one.
#[test]
fn real_git_deletes_what_the_reader_names() {
    let root: PathBuf = fixture("branch-delete-spellings");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    for values in [
        vec!["-d", "short"],
        vec!["-D", "forced"],
        vec!["--delete", "long"],
        vec!["--delete", "abbreviated"],
        vec!["-d", "--", "separated"],
    ] {
        let name: &str = values[values.len() - 1];
        let named: Option<Vec<OsString>> = deletions(values.as_slice());
        assert_eq!(named, Some(os_arguments(&[name])), "{values:?}");
        git(repo.as_path(), &["branch", name]);
        let mut spelled: Vec<&str> = vec!["branch"];
        spelled.extend(values.iter().copied());
        let output: std::process::Output = git_status(repo.as_path(), spelled.as_slice());
        assert!(output.status.success(), "{values:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()));
        assert!(branch_list(repo.as_path(), name).is_empty(), "{values:?} kept {name}");
    }
    // The reader calls this one no deletion, and Git agrees: it creates the branch.
    assert_eq!(deletions(&["topic"]), None);
    let created: std::process::Output = git_status(repo.as_path(), &["branch", "topic"]);
    assert!(created.status.success(), "{}", String::from_utf8_lossy(created.stderr.as_slice()));
    assert!(branch_list(repo.as_path(), "topic").contains("topic"));
    remove(root.as_path());
}

/// The `git branch --list` line of one name,
/// or empty text when there is no such branch.
fn branch_list(repo: &std::path::Path, name: &str) -> String {
    let output: std::process::Output = git(repo, &["branch", "--list", name]);
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// A deletion names the reference namespace it deletes from,
/// in every spelling of `--remotes`,
/// and the value of an upstream option is never read as one.
#[test]
fn a_deletion_names_the_namespace_it_deletes_from() {
    for spelled in [
        vec!["--remotes", "-D", "origin/gone"],
        vec!["-r", "-D", "origin/gone"],
        vec!["-rD", "origin/gone"],
        vec!["-Dr", "origin/gone"],
        vec!["-d", "--remotes", "origin/gone"],
    ] {
        let deletion = branch_deletion(os_arguments(spelled.as_slice()).as_slice()).expect("a deletion");
        assert!(deletion.remotes, "{spelled:?}");
        assert_eq!(deletion.names, vec![OsString::from("origin/gone")], "{spelled:?}");
    }
    for spelled in [
        vec!["-D", "gone"],
        vec!["-d", "gone"],
        // `-u r` sets the upstream of no branch: `r` is that option's value, not `--remotes`.
        vec!["-ur", "-D", "gone"],
        vec!["-u", "r", "-D", "gone"],
    ] {
        let deletion = branch_deletion(os_arguments(spelled.as_slice()).as_slice()).expect("a deletion");
        assert!(!deletion.remotes, "{spelled:?}");
    }
    assert_eq!(branch_deletion(os_arguments(&["--list"]).as_slice()), None);
    assert_eq!(branch_deletion(os_arguments(&["-r"]).as_slice()), None);
}

/// Remote-tracking references are read in every spelling Git's parser accepts,
/// because the namespace decides which
///  reference the record names.
#[test]
fn the_namespace_is_read_the_way_gits_parser_reads_it() {
    for spelled in [
        vec!["--remotes", "-D", "origin/gone"],
        vec!["--remote", "-D", "origin/gone"],
        vec!["--remot", "-D", "origin/gone"],
        vec!["-r", "--delete", "origin/gone"],
        vec!["-rD", "origin/gone"],
        vec!["--all", "-D", "origin/gone"],
    ] {
        let deletion = branch_deletion(os_arguments(spelled.as_slice()).as_slice()).expect("a deletion");
        // `--all` lists every branch, and a deletion under it still deletes the local one.
        let expected: bool = spelled[0] != "--all";
        assert_eq!(deletion.remotes, expected, "{spelled:?}");
        assert_eq!(deletion.names, vec![OsString::from("origin/gone")], "{spelled:?}");
    }
    // A name that follows a value-taking option is that option's value, never a branch to record.
    let upstream = branch_deletion(os_arguments(&["-u", "origin/main", "-D", "gone"]).as_slice()).expect("a deletion");
    assert_eq!(upstream.names, vec![OsString::from("gone")]);
}
