//! What:
//!  The sequencer-state query,
//!  its exact output reading and its decision,
//!  checked
//!       against real Git 2.56.0 in a main worktree,
//!  a linked worktree and a directory
//!       whose name contains a newline.
//! Why:
//!  A pathless commit is let through only while a merge,
//!  cherry-pick or revert awaits
//!      its commit;
//!  a misread path would look in the wrong place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await sequencerInProgress({ preSubcommandArgs: ['-C', repo] })).toBe('in-progress');
//! ```

/// The functions under test and the real-Git fixture helpers.
use super::{
    SequencerFacts, SequencerOutputError, SequencerState, sequencer_head_paths,
    sequencer_query_arguments, sequencer_state, sequencer_state_when_query_fails,
};
use crate::command_test_support::{
    fixture, git, git_output, os_arguments, remove, repository, repository_with_tracked_file,
    start_conflicted_merge,
};
use std::path::{Path, PathBuf};
use std::process::Output;

/// Any one head file means an operation is in progress.
#[test]
fn decides_from_the_three_presence_facts() {
    for (merge_head, cherry_pick_head, revert_head, expected) in [
        (false, false, false, SequencerState::NotInProgress),
        (true, false, false, SequencerState::InProgress),
        (false, true, false, SequencerState::InProgress),
        (false, false, true, SequencerState::InProgress),
        (true, true, true, SequencerState::InProgress),
    ] {
        assert_eq!(
            sequencer_state(SequencerFacts {
                merge_head,
                cherry_pick_head,
                revert_head
            }),
            expected
        );
    }
    assert_eq!(
        sequencer_state_when_query_fails(),
        SequencerState::NotInProgress
    );
}

/// The global prefix is forwarded unchanged in front of the fixed query.
#[test]
fn builds_the_query_after_the_global_prefix() {
    assert_eq!(
        sequencer_query_arguments(os_arguments(&["-C", "/r"]).as_slice()),
        os_arguments(&[
            "-C",
            "/r",
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "MERGE_HEAD",
            "--git-path",
            "CHERRY_PICK_HEAD",
            "--git-path",
            "REVERT_HEAD"
        ])
    );
}

/// Output is read as three paths of one directory,
///  even when the directory has a newline.
#[test]
fn reads_exact_query_output() {
    assert_eq!(
        sequencer_head_paths(
            b"/r/.git/MERGE_HEAD\n/r/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD\n"
        ),
        Ok(vec![
            b"/r/.git/MERGE_HEAD".to_vec(),
            b"/r/.git/CHERRY_PICK_HEAD".to_vec(),
            b"/r/.git/REVERT_HEAD".to_vec()
        ])
    );
    assert_eq!(
        sequencer_head_paths(b"/a\nb/MERGE_HEAD\n/a\nb/CHERRY_PICK_HEAD\n/a\nb/REVERT_HEAD\n"),
        Ok(vec![
            b"/a\nb/MERGE_HEAD".to_vec(),
            b"/a\nb/CHERRY_PICK_HEAD".to_vec(),
            b"/a\nb/REVERT_HEAD".to_vec()
        ])
    );
    // The shortest complete answer: three names of an empty directory part, as Git prints
    // them from inside the Git directory without an absolute format. One byte less is refused.
    assert_eq!(
        sequencer_head_paths(b"MERGE_HEAD\nCHERRY_PICK_HEAD\nREVERT_HEAD\n"),
        Ok(vec![
            b"MERGE_HEAD".to_vec(),
            b"CHERRY_PICK_HEAD".to_vec(),
            b"REVERT_HEAD".to_vec()
        ])
    );
    assert_eq!(
        sequencer_head_paths(b"MERGE_HEAD\nCHERRY_PICK_HEAD\nREVERT_HEAD"),
        Err(SequencerOutputError)
    );
    for malformed in [
        &b""[..],
        b"/r/.git/MERGE_HEAD\n",
        b"/r/.git/MERGE_HEAD\n/r/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD",
        b"/r/.git/MERGE_HEAD\n/x/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD\n",
        b"/r/.git/MERGE_HEAD\n/r/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD\n\n\n\n",
        b"/r/.git/REVERT_HEAD\n/r/.git/CHERRY_PICK_HEAD\n/r/.git/MERGE_HEAD\n",
        b"/r/.git/MERGE_HEAD /r/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD\n",
        // One or two trailing bytes beyond three complete lines.
        b"/r/.git/MERGE_HEAD\n/r/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD\nx",
        b"/r/.git/MERGE_HEAD\n/r/.git/CHERRY_PICK_HEAD\n/r/.git/REVERT_HEAD\n\n\n",
    ] {
        assert_eq!(
            sequencer_head_paths(malformed),
            Err(SequencerOutputError),
            "{:?}",
            String::from_utf8_lossy(malformed)
        );
    }
    assert!(SequencerOutputError.to_string().contains("--git-path"));
}

/// Run the query in a directory and observe which head files exist.
#[cfg(unix)]
fn observed_facts(directory: &Path) -> (Vec<PathBuf>, SequencerFacts) {
    use std::os::unix::ffi::OsStringExt;
    let output: Output = git_output(directory, sequencer_query_arguments(&[]).as_slice());
    assert!(output.status.success());
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for bytes in sequencer_head_paths(output.stdout.as_slice()).expect("three paths") {
        paths.push(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
    }
    let facts: SequencerFacts = SequencerFacts {
        merge_head: paths[0].exists(),
        cherry_pick_head: paths[1].exists(),
        revert_head: paths[2].exists(),
    };
    return (paths, facts);
}

/// A conflicted merge is in progress exactly while Git keeps `MERGE_HEAD`.
#[cfg(unix)]
#[test]
fn real_git_merge_conflict_is_in_progress() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("sequencer-merge");
    let (paths, before): (Vec<PathBuf>, SequencerFacts) = observed_facts(root.as_path());
    assert_eq!(paths[0], root.join(".git").join("MERGE_HEAD"));
    assert_eq!(sequencer_state(before), SequencerState::NotInProgress);
    start_conflicted_merge(root.as_path());
    let (_, during): (Vec<PathBuf>, SequencerFacts) = observed_facts(root.as_path());
    assert!(during.merge_head && !during.cherry_pick_head && !during.revert_head);
    assert_eq!(sequencer_state(during), SequencerState::InProgress);
    git(root.as_path(), &["merge", "--abort"]);
    let (_, after): (Vec<PathBuf>, SequencerFacts) = observed_facts(root.as_path());
    assert_eq!(sequencer_state(after), SequencerState::NotInProgress);
    remove(directory.as_path());
}

/// A linked worktree has its own head files,
///  and a newline in the path is read exactly.
#[cfg(unix)]
#[test]
fn real_git_paths_follow_linked_worktrees_and_newlines() {
    let directory: PathBuf = fixture("sequencer-paths");
    let root: PathBuf = repository(directory.as_path(), "line\nbreak");
    let (paths, _): (Vec<PathBuf>, SequencerFacts) = observed_facts(root.as_path());
    assert_eq!(
        paths,
        vec![
            root.join(".git").join("MERGE_HEAD"),
            root.join(".git").join("CHERRY_PICK_HEAD"),
            root.join(".git").join("REVERT_HEAD")
        ]
    );
    let linked: PathBuf = directory.join("linked");
    git(
        root.as_path(),
        &["worktree", "add", "--quiet", "../linked", "--detach"],
    );
    let (linked_paths, _): (Vec<PathBuf>, SequencerFacts) = observed_facts(linked.as_path());
    assert_eq!(
        linked_paths[0],
        root.join(".git/worktrees/linked/MERGE_HEAD")
    );
    // Outside any repository the query fails, and the fallback is normal enforcement.
    let outside: Output = git_output(
        directory.as_path(),
        sequencer_query_arguments(&[]).as_slice(),
    );
    assert!(!outside.status.success());
    remove(directory.as_path());
}
