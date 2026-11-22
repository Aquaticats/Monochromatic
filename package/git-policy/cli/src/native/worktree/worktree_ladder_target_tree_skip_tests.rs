//! What:
//!  Controls for the up-front skip of a target tree:
//! the commit `git worktree add` really checks out,
//! what a rung pays for a tree no rule of its own can serve,
//!  and a link no clone's index entry can vouch for.
//! Why:
//!  The skip reads one name and copies nothing,
//! so the name it reads has to be the one Git checks out
//!  (`Documentation/git-worktree.adoc`
//!  and `builtin/worktree.c`),
//! and the read has to be worth its two milliseconds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const ladder = beginLadder(plan, context, identity);
//! // expect(ladder.rung).toBe('git'); expect(ladder.slow.word()).toBe('declines');
//! ```

use super::*;
use crate::add_timing::ADD_TIMING_FILE;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_copy::GitStep;
use crate::worktree_copy_stage_claim::stage_claims_exist;
use crate::worktree_ladder_run::{Ladder, Rung, begin_ladder};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_naming_a_commit, plan_naming_no_commit, plan_to_at, tracked};
use crate::worktree_slow_route::SlowRoute;
use std::path::{Path, PathBuf};

/// What:
///  A clean repository whose tracked `.gitattributes` turns every `*.txt` path into the CRLF form,
///  with a branch `other` at a commit that lacks `extra.txt`,
///  and this `HEAD` holding it.
/// Why:
///   One fixture body serves every control here:
/// the fence hands Git every `.txt` path,
///  so a tree difference in one of them is the class the skip exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertingFixture(root) { tracked('.gitattributes', '*.txt text eol=crlf\n'); ...; updateRef('refs/heads/other'); tracked('extra.txt'); }
/// ```
fn converting_fixture(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "a.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "conversion");
    git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "second");
    return repo;
}

/// What:
///  The ladder's route and sentence for one plan,
///  and the record the add wrote while it chose.
/// Why:
///   The route is what says whether Git's checkout or a rung wrote the worktree,
/// and the record names every read the choice paid for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chosen(plan, repo) { beginTiming(); const ladder = beginLadder(plan, context, identity); endTiming(); return { ladder, record }; }
/// ```
fn chosen(plan: &AddPlan, repo: &Path) -> (Ladder, serde_json::Value) {
    let common: PathBuf = repo.join(".git");
    crate::add_timing::begin(common.clone(), true);
    let ladder: Ladder = begin_ladder(Some(plan), &git_context(), &main_identity(repo));
    crate::add_timing::finish(0);
    let text: String = std::fs::read_to_string(common.join(ADD_TIMING_FILE)).expect("the add's own record");
    let record: serde_json::Value = serde_json::from_str(text.trim_end()).expect("one JSON object");
    return (ladder, record);
}

/// What:
///  An add that names no commit-ish,
///  whose destination names an existing local branch at another tree,
///  is served by Git's checkout before anything is claimed or copied.
/// Why:
///   `git worktree add <path>` checks out the branch its destination is named after when that branch exists
/// (`Documentation/git-worktree.adoc`,
/// "COMMANDS",
/// and `dwim_branch` in `builtin/worktree.c`),
/// so such an add does not start at the source's `HEAD`;
///  read as a commit it names,
///  this branch's tree difference was never looked at,
///  and the add paid the copies and then Git's whole checkout over them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan_naming_no_commit(topic), context, identity); // the branch `topic` is older
/// expect(ladder.firstStep()).toBe('person'); expect(ladder.rung).toBe('git');
/// ```
#[test]
fn a_destination_that_names_an_existing_branch_is_skipped_before_any_copy() {
    let root: PathBuf = fixture("target-tree-destination-branch");
    let repo: PathBuf = converting_fixture(root.as_path());
    git(repo.as_path(), &["update-ref", "refs/heads/topic", "refs/heads/other"]);
    let plan: AddPlan = plan_naming_no_commit(root.join("topic"));
    let (ladder, record) = chosen(&plan, repo.as_path());
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the worktree: {record}");
    assert!(matches!(ladder.rung, Rung::Git(_)), "the rung steps aside before the first copy: {record}");
    let route: &SlowRoute = ladder.slow.as_ref().expect("the sentence that says why");
    assert_eq!(route.word(), "declines", "{route:?}");
    assert!(route.line().contains("differs from the source's index at extra.txt"), "{}", route.line());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "no stage name was claimed, so nothing was copied");
    remove(root.as_path());
}

/// What:
///  An add that names a local branch which a tag of the same name shadows
///  reads that branch's tree, as Git's own checkout does.
/// Why:
///   `add_worktree` sets the new worktree's `HEAD` to `refs/heads/<name>` when that ref exists and the add detaches
/// nothing,
///  while `rev-parse` reaches `refs/tags/<name>` first;
/// the name-only read used to take the tag's tree,
///  so a tag and a branch of one name made the rung say the tree differed where Git's own checkout wrote the same
///  tree,
///  and pay Git's checkout where `main` served the clone rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan_naming_a_commit(topic, 'same-name'), context, identity); // tag at the older commit, branch at HEAD
/// expect(ladder.rung).toBe('clone'); expect(ladder.slow).toBeUndefined();
/// ```
#[test]
fn a_branch_of_a_tags_name_is_the_tree_the_add_starts_at() {
    let root: PathBuf = fixture("target-tree-tag-and-branch");
    let repo: PathBuf = repository(root.as_path(), "repo");
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "a.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "conversion");
    // The tag holds the older tree, and the branch of the same name holds this `HEAD`: Git checks the branch out.
    git(repo.as_path(), &["tag", "same-name"]);
    tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    commit_all(repo.as_path(), "second");
    git(repo.as_path(), &["update-ref", "refs/heads/same-name", "HEAD"]);
    let plan: AddPlan = plan_naming_a_commit(root.join("topic"), "same-name");
    let (ladder, record) = chosen(&plan, repo.as_path());
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "the source is eligible: {record}");
    assert!(matches!(ladder.rung, Rung::Clone(..)), "the clone rung serves a tree Git's checkout writes no differently: {record}");
    assert!(ladder.slow.is_none(), "no sentence says the tree differs: {record}");
    remove(root.as_path());
}

/// What:
///  An add at a tree that changes a path,
///  in a repository where no rule can convert one,
///  pays no read of the target tree.
/// Why:
///   The skip's read is one Git run,
/// and when the fence hands Git no path at all no changed path can make it decline;
///  the read buys nothing there,
///  and the record of this add names it unless it is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { record } = chosen(plan_to_at(topic, 'other'), repoWithoutRules);
/// expect(record.steps['git diff-tree']).toBeUndefined(); expect(record.steps.targetTreeDifference).toBeDefined();
/// ```
#[test]
fn a_target_tree_no_rule_of_the_fence_can_serve_costs_no_read() {
    let root: PathBuf = fixture("target-tree-no-rule");
    let repo: PathBuf = repository(root.as_path(), "repo");
    tracked(repo.as_path(), "a.txt", b"line one\n");
    tracked(repo.as_path(), "keep.dat", b"line one\n");
    commit_all(repo.as_path(), "first");
    git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    tracked(repo.as_path(), "extra.txt", b"line one\n");
    commit_all(repo.as_path(), "second");
    let plan: AddPlan = plan_to_at(root.join("topic"), Some("other"));
    let (ladder, record) = chosen(&plan, repo.as_path());
    assert!(matches!(ladder.rung, Rung::Clone(..)), "the clone rung serves the add: {record}");
    assert!(ladder.slow.is_none(), "no sentence says the tree differs: {record}");
    let steps: &serde_json::Map<String, serde_json::Value> = record["steps"].as_object().expect("the record's steps");
    assert!(steps.contains_key("targetTreeDifference"), "the question is still asked, and answered before any read: {record}");
    assert!(!steps.contains_key("git diff-tree"), "no read of the target tree is paid when the fence hands Git no path: {record}");
    remove(root.as_path());
}

/// What:
///  An add at a tree that changes a tracked link,
///  and nothing else,
///  is served by Git's checkout before any copy.
/// Why:
///   A clone makes a link again instead of hard-linking it,
/// so the fields its index records cannot describe the destination's link,
///  and `git read-tree -m -u` refuses the entry the target tree changes
///  (`Entry 'link' not uptodate. Cannot merge.`);
/// the rung then pays its copies and Git's whole checkout over them,
///  which the one name-only read can spare,
///  because a link is an entry of the source's own index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan_to_at(topic, 'other'), context, identity); // the link points elsewhere at `other`
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.line()).toContain('link');
/// ```
#[test]
fn a_target_tree_that_changes_a_link_is_skipped_before_any_copy() {
    let root: PathBuf = fixture("target-tree-link");
    let repo: PathBuf = repository(root.as_path(), "repo");
    tracked(repo.as_path(), "a.txt", b"line one\n");
    tracked(repo.as_path(), "b.txt", b"line two\n");
    std::os::unix::fs::symlink("a.txt", repo.join("link")).expect("the link");
    commit_all(repo.as_path(), "the link points at a.txt");
    git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    std::fs::remove_file(repo.join("link")).expect("the link is replaced");
    std::os::unix::fs::symlink("b.txt", repo.join("link")).expect("the new link");
    commit_all(repo.as_path(), "the link points at b.txt");
    let plan: AddPlan = plan_to_at(root.join("topic"), Some("other"));
    let (ladder, record) = chosen(&plan, repo.as_path());
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the link: {record}");
    assert!(matches!(ladder.rung, Rung::Git(_)), "the rung steps aside before the copies: {record}");
    let route: &SlowRoute = ladder.slow.as_ref().expect("the sentence that says why");
    assert!(route.line().contains("differs from the source's index at link"), "{}", route.line());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "no stage name was claimed, so nothing was copied");
    remove(root.as_path());
}
