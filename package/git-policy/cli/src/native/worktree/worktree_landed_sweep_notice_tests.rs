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
use crate::worktree_landed_removal::{KeptWorktree, LandedSweep, RemovedWorktree, UnprovenWorktree};
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

/// What:
///  A worktree whose removal refused is named with the report that clears the way,
///  and never with the command that would refuse the same way again.
/// Why:
///   A refusal names its own way forward
///  (a nested worktree is removed before the one that holds it,
/// a pool of snapshots is taken first),
///  and the line the sweep prints for a kept worktree appended
/// `git worktree remove <path>`,
/// which for those states is the command that just refused,
/// so a person reading it
///  would run it and read the same refusal.
#[test]
fn a_worktree_whose_removal_refused_names_the_report_and_not_the_command_that_refused() {
    let sweep: LandedSweep = LandedSweep {
        landed_branch: String::from("main"),
        refused: vec![KeptWorktree {
            worktree: PathBuf::from("/a worktree/holder"),
            branch: String::from("refs/heads/feature"),
            why: String::from("the worktree /a worktree/holder holds the linked worktree /a worktree/holder/inner, and nothing was removed: remove it first, with `git worktree remove /a worktree/holder/inner`"),
        }],
        ..LandedSweep::default()
    };
    let lines: Vec<String> = landed_sweep_lines(&sweep);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &str = lines[0].as_str();
    assert!(line.contains("left the worktree /a worktree/holder in place"), "{line}");
    assert!(line.contains("nothing there was removed"), "{line}");
    assert!(line.contains("the linked worktree /a worktree/holder/inner"), "the report that clears the way is there: {line}");
    assert!(!line.contains("git worktree remove '/a worktree/holder'"), "and not the command that refused: {line}");
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
