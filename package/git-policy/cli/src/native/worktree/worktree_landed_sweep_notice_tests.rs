//! What:
//!  Controls for the words one landed sweep prints.
//! Why:
//!   A person whose worktree went,
//! or stayed,
//! must be able to read why and what to do,
//!  and a line that leaves out the path,
//!  the branch,
//!  or the reason is no such line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(landedSweepLines(sweep).join('\n')).toContain('because its branch refs/heads/feature is now on main');
//! ```

use super::landed_sweep_lines;
use crate::work_item::{WorkItem, WorkKind};
use crate::worktree_landed_removal::{HandedOverWorktree, KeptWorktree, LandedSweep, RemovedWorktree, UnprovenWorktree};
use std::path::PathBuf;

/// A sweep that removed one worktree.
fn a_sweep_that_removed() -> LandedSweep {
    return LandedSweep {
        landed_branch: String::from("main"),
        removed: vec![RemovedWorktree { worktree: PathBuf::from("/a worktree/with a space"), branch: String::from("refs/heads/feature") }],
        ..LandedSweep::default()
    };
}

#[test]
fn a_removed_worktree_is_named_with_its_branch_and_what_it_kept() {
    let lines: Vec<String> = landed_sweep_lines(&a_sweep_that_removed());
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &str = lines[0].as_str();
    assert!(line.contains("/a worktree/with a space"), "{line}");
    assert!(line.contains("refs/heads/feature"), "{line}");
    assert!(line.contains("is now on main"), "{line}");
    assert!(line.contains("the branch stays"), "{line}");
    assert!(line.contains("what frees the space its snapshot held"), "{line}");
}

/// What:
///  The line about a removal says a directory Git ignores went with it unread.
/// Why:
///   `worktree_landed_ignored_files.rs` never reads an ignored directory,
///  so whatever a person kept inside one is not in the worktree the merge ran in,
///  and a line that speaks only about the ignored files reads as if all of that state is safe elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(landedSweepLines(aSweepThatRemoved())[0]).toContain('A directory Git ignores went with it unread');
/// ```
#[test]
fn a_removed_worktree_says_an_ignored_directory_went_unread() {
    let lines: Vec<String> = landed_sweep_lines(&a_sweep_that_removed());
    let line: &str = lines[0].as_str();
    assert!(line.contains("ignored files there went with it"), "{line}");
    assert!(line.contains("A directory Git ignores went with it unread"), "{line}");
    assert!(line.contains("anything you kept inside one"), "{line}");
}

/// What:
///  A worktree whose removal the merge handed to the commit service is named with where its outcome is read.
/// Why:
///  The removal is Git's own reading of every tracked file there and finishes after the merge,
///  so a person whose worktree is still there at the prompt must read where the answer will stand
///  and that the worktree may stay because Git kept it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(landedSweepLines(aSweepThatHandedOneOver())[0]).toContain('handed the removal of the worktree');
/// ```
#[test]
fn a_handed_over_removal_is_named_with_the_record_its_outcome_stands_in() {
    let sweep: LandedSweep = LandedSweep {
        landed_branch: String::from("main"),
        handed_over: vec![HandedOverWorktree {
            worktree: PathBuf::from("/a worktree/with a space"),
            branch: String::from("refs/heads/feature"),
            record: WorkItem {
                kind: WorkKind::SubvolumeDelete,
                repository: PathBuf::from("/a worktree"),
                git_dir: PathBuf::from("/the repository/.git"),
                branch_ref: String::from(crate::worktree_remove_hand_over::WORKTREE_REMOVAL_REFERENCE),
                oid: String::from("0".repeat(40)),
                parent: String::from("/a worktree/with a space"),
                created: String::from("2026-01-01T00:00:00+00:00"),
                tries: 0,
            },
        }],
        ..LandedSweep::default()
    };
    let lines: Vec<String> = landed_sweep_lines(&sweep);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &str = lines[0].as_str();
    assert!(line.contains("/a worktree/with a space"), "{line}");
    assert!(line.contains("refs/heads/feature"), "{line}");
    assert!(line.contains("is now on main"), "{line}");
    assert!(line.contains("the commit service"), "{line}");
    assert!(line.contains("The branch stays"), "{line}");
    let record: String = crate::service_work::work_directory(std::path::Path::new("/the repository/.git"))
        .join(format!("subvolume-delete-{}", "0".repeat(40)))
        .display()
        .to_string();
    assert!(line.contains(record.as_str()), "the record the outcome stands in is named: {line}");
    assert!(line.contains("`done` there means the worktree is gone"), "{line}");
    assert!(line.contains("means Git kept it"), "{line}");
    assert!(line.contains("git worktree remove '/a worktree/with a space'"), "{line}");
}

#[test]
fn a_kept_worktree_is_named_with_its_reason_and_the_command_that_removes_it() {
    let sweep: LandedSweep = LandedSweep {
        landed_branch: String::from("main"),
        kept: vec![KeptWorktree {
            worktree: PathBuf::from("/a worktree/busy"),
            branch: String::from("refs/heads/feature"),
            why: String::from("process 4242 is working in it"),
        }],
        ..LandedSweep::default()
    };
    let lines: Vec<String> = landed_sweep_lines(&sweep);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &str = lines[0].as_str();
    assert!(line.contains("left the worktree /a worktree/busy in place"), "{line}");
    assert!(line.contains("process 4242 is working in it"), "{line}");
    assert!(line.contains("git worktree remove '/a worktree/busy'"), "the command is quoted for a shell: {line}");
}

#[test]
fn an_unproven_worktree_says_the_ignored_files_may_be_the_persons() {
    let sweep: LandedSweep = LandedSweep {
        landed_branch: String::from("main"),
        unproven: vec![UnprovenWorktree { worktree: PathBuf::from("/a worktree/theirs"), branch: String::from("refs/heads/feature") }],
        ..LandedSweep::default()
    };
    let lines: Vec<String> = landed_sweep_lines(&sweep);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &str = lines[0].as_str();
    assert!(line.contains("/a worktree/theirs"), "{line}");
    assert!(line.contains("may be yours"), "{line}");
    assert!(line.contains("git worktree remove '/a worktree/theirs'"), "{line}");
}

#[test]
fn a_sweep_that_looked_at_nothing_says_why() {
    let sweep: LandedSweep = LandedSweep { landed_branch: String::from("main"), failed: Some(String::from("Git cannot list the branches")), ..LandedSweep::default() };
    let lines: Vec<String> = landed_sweep_lines(&sweep);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("no landed worktree was looked at"), "{}", lines[0]);
    assert!(lines[0].contains("Git cannot list the branches"), "{}", lines[0]);
}

#[test]
fn a_commit_a_removal_would_have_deleted_is_said_before_the_removal() {
    let sweep: LandedSweep = LandedSweep {
        landed_branch: String::from("main"),
        kept_commits: vec![String::from("the commit abcdef is held behind a lock and is kept at `refs/cli-git/kept/7`")],
        removed: vec![RemovedWorktree { worktree: PathBuf::from("/a worktree/held"), branch: String::from("refs/heads/feature") }],
        ..LandedSweep::default()
    };
    let lines: Vec<String> = landed_sweep_lines(&sweep);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("monochromatic-git-policy-cli: the commit abcdef"), "{}", lines[0]);
    assert!(lines[0].contains("refs/cli-git/kept/7"), "{}", lines[0]);
    assert!(lines[1].contains("removed the worktree /a worktree/held"), "{}", lines[1]);
}

#[test]
fn a_sweep_with_nothing_to_say_prints_nothing() {
    let sweep: LandedSweep = LandedSweep { landed_branch: String::from("main"), ..LandedSweep::default() };
    assert!(landed_sweep_lines(&sweep).is_empty(), "a merge that landed no branch of this program says nothing");
    assert!(!landed_sweep_lines(&a_sweep_that_removed()).is_empty());
}
