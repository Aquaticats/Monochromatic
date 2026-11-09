//! What:
//!  The status query of each scope and the reading of its answers.
//! Why:
//!  A scope read as the other one would reject a harmless command or allow a deleting one;
//! a failed status read
//!  as clean would let every discarding command through.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(dirtyFromOutput({ success: false, ... })).toThrow();
//! ```

/// The query builder,
/// the reader and the scopes.
use super::{DirtyScope, dirty_from_output, dirty_query_arguments, index_file_is_absent};
use crate::command_test_support::{fixture, git, os_arguments, remove, repository};
use crate::git_metadata::MetadataOutput;
use std::ffi::{OsStr, OsString};

/// Build the output of one status run.
fn output(success: bool, stdout: &[u8], stderr: &[u8]) -> MetadataOutput {
    return MetadataOutput {
        success,
        code: Some(if success { 0 } else { 128 }),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    };
}

/// The global options come first,
/// then the read-only status with the scope's untracked setting.
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
    // Git refuses `--ignored=matching` together with `--untracked-files=no`, so a reading that reaches ignored
    // files asks for untracked ones too and filters the answer itself.
    for scope in [DirtyScope::Everything, DirtyScope::IgnoredOnly] {
        assert_eq!(
            dirty_query_arguments(&[], scope),
            os_arguments(&[
                "--no-optional-locks",
                "status",
                "--porcelain=v1",
                "--untracked-files=normal",
                "--ignored=matching"
            ]),
            "{scope:?}"
        );
    }
}

/// Output means dirty,
/// silence means clean,
/// and a failure is an error carrying Git's first line.
#[test]
fn output_means_dirty_and_failure_is_an_error() {
    assert_eq!(dirty_from_output(&output(true, b" M a.txt\n", b""), DirtyScope::Tracked), Ok(true));
    assert_eq!(dirty_from_output(&output(true, b"", b""), DirtyScope::Tracked), Ok(false));
    assert_eq!(
        dirty_from_output(&output(false, b"", b"fatal: not a git repository\nmore\n"), DirtyScope::Tracked),
        Err(String::from(
            "Git could not read the worktree status: fatal: not a git repository"
        ))
    );
    assert_eq!(
        dirty_from_output(&output(false, b" M a.txt\n", b""), DirtyScope::Tracked),
        Err(String::from("Git could not read the worktree status: "))
    );
}

/// Real Git:
/// a tracked edit counts in every scope but the ignored one,
/// an untracked file only in the wider ones,
/// and an ignored file only where the reading is about ignored files.
#[test]
fn each_scope_counts_the_state_it_can_destroy() {
    let root = fixture("dirty-scope");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b".env\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    for scope in [DirtyScope::Tracked, DirtyScope::TrackedAndUntracked, DirtyScope::Everything, DirtyScope::IgnoredOnly] {
        assert!(!ask(repo.as_path(), scope), "{scope:?} clean");
    }
    std::fs::write(repo.join("new.txt"), "x\n").expect("untracked file");
    assert!(!ask(repo.as_path(), DirtyScope::Tracked), "an untracked file is not tracked work");
    assert!(!ask(repo.as_path(), DirtyScope::IgnoredOnly), "an untracked file is not ignored");
    assert!(ask(repo.as_path(), DirtyScope::TrackedAndUntracked));
    assert!(ask(repo.as_path(), DirtyScope::Everything));
    git(repo.as_path(), &["add", "--", "new.txt"]);
    assert!(ask(repo.as_path(), DirtyScope::Tracked));
    git(repo.as_path(), &["commit", "--quiet", "--message=untracked"]);
    std::fs::write(repo.join(".env"), b"SECRET=1\n").expect("ignored file");
    assert!(!ask(repo.as_path(), DirtyScope::Tracked), "an ignored file is not tracked work");
    assert!(!ask(repo.as_path(), DirtyScope::TrackedAndUntracked), "an ignored file is not untracked");
    assert!(ask(repo.as_path(), DirtyScope::Everything), "a clean -x deletes the ignored file");
    assert!(ask(repo.as_path(), DirtyScope::IgnoredOnly), "a clean -X deletes the ignored file");
    remove(root.as_path());
}

/// Run the query with real Git inside a repository and read it.
fn ask(repo: &std::path::Path, scope: DirtyScope) -> bool {
    let arguments: Vec<OsString> = dirty_query_arguments(&[], scope);
    let raw = crate::command_test_support::git_output(repo, arguments.as_slice());
    let output = output(raw.status.success(), &raw.stdout, &raw.stderr);
    return dirty_from_output(&output, scope).expect("status answers");
}

/// A Git directory without an `index` file is absent;
/// one with an `index` file,
/// empty included,
/// is not.
#[test]
fn the_index_file_is_absent_only_where_the_git_directory_has_none() {
    let root = fixture("index-file-absent");
    let git_directory = root.join("git-directory");
    std::fs::create_dir(&git_directory).expect("git directory");
    assert!(index_file_is_absent(Some(git_directory.as_path()), None));
    std::fs::write(git_directory.join("index"), b"").expect("empty index file");
    assert!(!index_file_is_absent(Some(git_directory.as_path()), None));
    remove(root.as_path());
}

/// A Git directory that does not exist,
/// no Git directory,
/// and a `GIT_INDEX_FILE` that names another index leave the question to the status.
#[test]
fn the_index_file_is_not_absent_when_the_question_is_not_the_default_index() {
    let root = fixture("index-file-unknown");
    let missing = root.join("no-such-git-directory");
    assert!(!index_file_is_absent(Some(missing.as_path()), None));
    assert!(!index_file_is_absent(None, None));
    let git_directory = root.join("git-directory");
    std::fs::create_dir(&git_directory).expect("git directory");
    assert!(!index_file_is_absent(Some(git_directory.as_path()), Some(OsStr::new("other-index"))));
    remove(root.as_path());
}
