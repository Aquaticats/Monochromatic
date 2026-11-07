//! What:
//!  The index-against-`HEAD` exit-status mapping,
//!  checked against real Git 2.56.0.
//! Why:
//!  The commit-only transform rejects only on `Differs`;
//!  a wrong mapping either blocks
//!      valid amends or lets a staged change be ignored silently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await indexDiffersFromHead({ preSubcommandArgs: ['-C', repo] })).toBe('differs');
//! ```

/// The functions under test and the real-Git fixture helpers.
use super::{INDEX_QUERY, IndexVsHead, index_query_arguments, index_state_from_exit};
use crate::command_test_support::{
    git, git_output, os_arguments, remove, repository_with_tracked_file,
};
use std::ffi::OsString;
use std::path::PathBuf;

/// Only exit codes 0 and 1 are answers;
///  every other outcome is "Git cannot say".
#[test]
fn maps_exit_statuses() {
    assert_eq!(index_state_from_exit(Some(0)), IndexVsHead::Matches);
    assert_eq!(index_state_from_exit(Some(1)), IndexVsHead::Differs);
    for other in [Some(2), Some(128), Some(129), Some(-1), None] {
        assert_eq!(
            index_state_from_exit(other),
            IndexVsHead::Unknown,
            "{other:?}"
        );
    }
}

/// The global prefix is forwarded unchanged in front of the fixed query.
#[test]
fn builds_the_query_after_the_global_prefix() {
    assert_eq!(index_query_arguments(&[]), os_arguments(INDEX_QUERY));
    assert_eq!(
        index_query_arguments(os_arguments(&["-C", "", "--git-dir=x"]).as_slice()),
        os_arguments(&[
            "-C",
            "",
            "--git-dir=x",
            "diff-index",
            "--quiet",
            "--cached",
            "HEAD",
            "--"
        ])
    );
}

/// Real Git answers 0 for a clean index,
///  1 for a staged change,
///  and another code when it
/// cannot compare (unborn `HEAD`,
///  no repository).
#[test]
fn real_git_exit_statuses_map_to_the_three_states() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("index-states");
    let prefix: Vec<OsString> = vec![OsString::from("-C"), root.clone().into_os_string()];
    let query: Vec<OsString> = index_query_arguments(prefix.as_slice());
    let clean: Option<i32> = git_output(directory.as_path(), query.as_slice())
        .status
        .code();
    assert_eq!(
        index_state_from_exit(clean),
        IndexVsHead::Matches,
        "{clean:?}"
    );
    std::fs::write(root.join("tracked.txt"), b"two\n").expect("modify tracked file");
    // An unstaged change does not count: the question is index against `HEAD`.
    let unstaged: Option<i32> = git_output(directory.as_path(), query.as_slice())
        .status
        .code();
    assert_eq!(index_state_from_exit(unstaged), IndexVsHead::Matches);
    git(root.as_path(), &["add", "--", "tracked.txt"]);
    let staged: Option<i32> = git_output(directory.as_path(), query.as_slice())
        .status
        .code();
    assert_eq!(
        index_state_from_exit(staged),
        IndexVsHead::Differs,
        "{staged:?}"
    );
    let unborn: PathBuf = directory.join("unborn");
    std::fs::create_dir(&unborn).expect("unborn repository directory");
    git(unborn.as_path(), &["init", "--quiet"]);
    for place in [unborn.as_path(), directory.as_path()] {
        let code: Option<i32> = git_output(place, index_query_arguments(&[]).as_slice())
            .status
            .code();
        assert_ne!(code, Some(0), "{place:?}");
        assert_ne!(code, Some(1), "{place:?}");
        assert_eq!(
            index_state_from_exit(code),
            IndexVsHead::Unknown,
            "{place:?}"
        );
    }
    remove(directory.as_path());
}

/// The index check the wrapper makes before a merge,
/// a commit and a restore starts no Git `diff-index`,
/// and still says 0 for an
/// index equal to `HEAD` and 1 for a staged change.
#[test]
fn the_index_check_starts_no_git() {
    use crate::recording_git::{recorded_starts, recording_context};
    use crate::repo_read_hook::SWITCH_VARIABLE;
    use crate::test_support::{fixture, git_text, repository};
    use crate::transaction_git::{GitRequest, run_git};
    let root: PathBuf = fixture("rule-commit-index-no-start");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (mut context, log) = recording_context(root.as_path());
    // The recorder switches the in-process readers off; the check under test must run with them on.
    context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    let commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let ask = |words: &[&str]| -> Option<i32> { return run_git(&context, &GitRequest::new(repo.as_path(), words)).expect("ran").code; };
    assert_eq!(ask(INDEX_QUERY), Some(0));
    assert_eq!(ask(&["diff-index", "--cached", "--quiet", commit.as_str()]), Some(0));
    std::fs::write(repo.join("staged.txt"), "staged\n").expect("write");
    git(repo.as_path(), &["add", "staged.txt"]);
    assert_eq!(ask(INDEX_QUERY), Some(1));
    assert_eq!(ask(&["diff-index", "--cached", "--quiet", commit.as_str()]), Some(1));
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert!(!starts.iter().any(|start| { return start.starts_with("diff-index"); }), "Git was started for the index check: {starts:?}");
    remove(root.as_path());
}
