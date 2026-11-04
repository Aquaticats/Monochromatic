//! What:
//!  The status query of each scope and the reading of its answers.
//! Why:
//!  A scope read as the other one would reject a harmless command or allow a deleting one; a failed status read
//!  as clean would let every discarding command through.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(dirtyFromOutput({ success: false, ... })).toThrow();
//! ```

/// The query builder, the reader and the scopes.
use super::{DirtyScope, dirty_from_output, dirty_query_arguments};
use crate::command_test_support::{fixture, git, os_arguments, remove, repository};
use crate::git_metadata::MetadataOutput;
use std::ffi::OsString;

/// Build the output of one status run.
fn output(success: bool, stdout: &[u8], stderr: &[u8]) -> MetadataOutput {
    return MetadataOutput {
        success,
        code: Some(if success { 0 } else { 128 }),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    };
}

/// The global options come first, then the read-only status with the scope's untracked setting.
#[test]
fn the_query_names_the_scope_after_the_global_options() {
    let prefix: Vec<OsString> = os_arguments(&["-C", "dir"]);
    assert_eq!(
        dirty_query_arguments(prefix.as_slice(), DirtyScope::Tracked),
        os_arguments(&[
            "-C",
            "dir",
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=no"
        ])
    );
    assert_eq!(
        dirty_query_arguments(&[], DirtyScope::TrackedAndUntracked),
        os_arguments(&[
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=normal"
        ])
    );
}

/// Output means dirty, silence means clean, and a failure is an error carrying Git's first line.
#[test]
fn output_means_dirty_and_failure_is_an_error() {
    assert_eq!(dirty_from_output(&output(true, b" M a.txt\n", b"")), Ok(true));
    assert_eq!(dirty_from_output(&output(true, b"", b"")), Ok(false));
    assert_eq!(
        dirty_from_output(&output(false, b"", b"fatal: not a git repository\nmore\n")),
        Err(String::from(
            "Git could not read the worktree status: fatal: not a git repository"
        ))
    );
    assert_eq!(
        dirty_from_output(&output(false, b" M a.txt\n", b"")),
        Err(String::from("Git could not read the worktree status: "))
    );
}

/// Real Git: a tracked edit counts in both scopes, an untracked file only in the wider one.
#[test]
fn real_git_counts_untracked_files_only_in_the_wider_scope() {
    let root = fixture("dirty-scope");
    let repo = repository(root.as_path(), "repo");
    for (scope, expected) in [(DirtyScope::Tracked, false), (DirtyScope::TrackedAndUntracked, false)] {
        assert_eq!(ask(repo.as_path(), scope), expected, "{scope:?} clean");
    }
    std::fs::write(repo.join("new.txt"), "x\n").expect("untracked file");
    assert!(!ask(repo.as_path(), DirtyScope::Tracked));
    assert!(ask(repo.as_path(), DirtyScope::TrackedAndUntracked));
    git(repo.as_path(), &["add", "--", "new.txt"]);
    assert!(ask(repo.as_path(), DirtyScope::Tracked));
    remove(root.as_path());
}

/// Run the query with real Git inside a repository and read it.
fn ask(repo: &std::path::Path, scope: DirtyScope) -> bool {
    let arguments: Vec<OsString> = dirty_query_arguments(&[], scope);
    let raw = crate::command_test_support::git_output(repo, arguments.as_slice());
    let output = output(raw.status.success(), &raw.stdout, &raw.stderr);
    return dirty_from_output(&output).expect("status answers");
}
