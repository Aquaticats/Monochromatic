//! Controls for the route's gate through its entry point:
//!  a main checkout on this test machine's own filesystem keeps the ordinary add,
//!  and a command that creates no worktree with a known plan is never the route's.

use super::run_a_zfs_add;
use crate::test_support::{fixture, remove};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_copy_forward::CopyFacts;
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::path::PathBuf;

/// A main checkout fixture with an add plan at `.claude/worktrees/topic`.
fn a_fixture(name: &str) -> (PathBuf, CopyFacts) {
    let root: PathBuf = fixture(name);
    std::fs::create_dir_all(root.join(".git")).expect("the common directory");
    std::fs::create_dir_all(root.join(".claude").join("worktrees")).expect("the worktrees folder");
    let destination: PathBuf = root.join(".claude").join("worktrees").join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree { common_dir: root.join(".git"), git_dir: root.join(".git"), worktree_root: root.clone() };
    let plan: AddPlan = AddPlan { destination: destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    let facts: CopyFacts = CopyFacts { identity, creates: true, destination: Some(destination), add_plan: Some(plan), remove_plan: None, quiet: false };
    return (root, facts);
}

#[test]
fn this_machine_s_own_filesystem_keeps_the_ordinary_add() {
    let (root, facts) = a_fixture("route-entry-filesystem");
    let arguments: Vec<OsString> = ["worktree", "add", "--no-checkout", "/tmp/topic", "main"].iter().map(OsString::from).collect();
    assert_eq!(run_a_zfs_add(std::path::Path::new("/usr/bin/git"), arguments.as_slice(), &[], &facts), None, "a checkout that is not the design's dataset takes the ladder");
    remove(root.as_path());
}

#[test]
fn a_command_with_no_plan_is_never_the_route_s() {
    let (root, mut facts) = a_fixture("route-entry-no-plan");
    facts.add_plan = None;
    let arguments: Vec<OsString> = ["worktree", "add", "/tmp/topic"].iter().map(OsString::from).collect();
    assert_eq!(run_a_zfs_add(std::path::Path::new("/usr/bin/git"), arguments.as_slice(), &[], &facts), None);
    remove(root.as_path());
}
