//! Controls for the retry of a commit the service sent back: the merge of the captured change onto the branch with real
//!  Git (clean, conflicting, and the captured file that already holds the other commit's edit), and the words a conflict
//!  ends the command with.

use super::*;
use crate::policy_events::PolicyEvent;
use crate::service_messages::DENYLIST;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

const FILE: &str = "f.txt";

/// The file with the lines named replaced.
fn lines_with(edits: &[(usize, &str)]) -> String {
    return (1..=9)
        .map(|number| {
            let line: String = edits.iter().find(|(at, _)| { return *at == number; }).map_or_else(|| { return format!("line {number}"); }, |(_, edit)| { return (*edit).to_string(); });
            return line + "\n";
        })
        .collect();
}

struct World {
    root: PathBuf,
    repo: PathBuf,
    context: GitContext,
    base: String,
}

/// A repository whose first commit holds the file, with the branch left on it.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(FILE), lines_with(&[])).expect("file");
    git(repo.as_path(), &["add", FILE]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::create_dir(root.join("scratch")).expect("scratch");
    return World { root, repo, context: git_context(), base };
}

/// Commits `edits` of the file on the current branch with `message`.
fn land(world: &World, edits: &[(usize, &str)], message: &str) {
    std::fs::write(world.repo.join(FILE), lines_with(edits)).expect("edit");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", format!("--message={message}").as_str()]);
}

/// The tree of a commit that applies `edits` to the base, made on a side branch the way a captured change is.
fn captured(world: &World, edits: &[(usize, &str)]) -> Captured {
    git(world.repo.as_path(), &["checkout", "--quiet", "-b", "mine", world.base.as_str()]);
    land(world, edits, "mine");
    let tree: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    return Captured { base: world.base.clone(), tree };
}

fn merge(world: &World, captured: &Captured) -> Merged {
    let tip: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let common: PathBuf = world.repo.join(".git");
    let scratch: PathBuf = world.root.join("scratch");
    let site: MergeSite<'_> =
        MergeSite { context: &world.context, cwd: world.repo.as_path(), common_dir: common.as_path(), object_directory: &common.join("objects"), scratch: scratch.as_path() };
    return merge_captured(&site, captured, tip.as_str(), 1).expect("the merge runs");
}

fn blob_of(world: &World, tree: &str) -> String {
    return git_text(world.repo.as_path(), &["show", format!("{tree}:{FILE}").as_str()]) + "\n";
}

/// Edits of different lines merge: the tree holds the landed edit and the captured one.
#[test]
fn edits_of_different_lines_merge_into_one_tree() {
    let world: World = world("replay-clean");
    let mine: Captured = captured(&world, &[(7, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    let Merged::Tree(tree) = merge(&world, &mine) else { panic!("expected a clean merge") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(1, "theirs"), (7, "mine")]));
    remove(world.root.as_path());
}

/// The same line changed differently cannot merge: the path is named, and so is the commit that landed first.
#[test]
fn edits_of_the_same_line_conflict_and_the_first_commit_is_named() {
    let world: World = world("replay-conflict");
    let mine: Captured = captured(&world, &[(1, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    land(&world, &[(1, "theirs"), (9, "later")], "later");
    let Merged::Conflict(paths) = merge(&world, &mine) else { panic!("expected a conflict") };
    assert_eq!(paths, vec![FILE.as_bytes().to_vec()]);
    let tip: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let first = first_landed(&world.context, world.repo.as_path(), world.base.as_str(), tip.as_str(), paths.as_slice()).expect("the log runs").expect("a commit");
    assert_eq!(first.subject, "theirs", "the commit that landed first, not the last");
    assert_eq!(first.short, git_text(world.repo.as_path(), &["rev-parse", "--short", "HEAD~1"]));
    remove(world.root.as_path());
}

/// A captured file that already holds the landed edit and one more on the next line lands as captured: Git alone calls the
///  adjacent hunks a conflict, and the worktree both commands share is why the captured bytes contain the landed edit.
#[test]
fn a_captured_file_that_holds_the_landed_edit_and_an_adjacent_one_lands_as_captured() {
    let world: World = world("replay-adjacent");
    let mine: Captured = captured(&world, &[(4, "theirs"), (5, "mine")]);
    land(&world, &[(4, "theirs")], "theirs");
    let Merged::Tree(tree) = merge(&world, &mine) else { panic!("expected the captured bytes to land") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(4, "theirs"), (5, "mine")]));
    remove(world.root.as_path());
}

/// A change that overlaps what landed, with different bytes, stays a conflict even when the file holds the landed edit
///  partly.
#[test]
fn a_captured_file_that_rewrote_the_landed_lines_still_conflicts() {
    let world: World = world("replay-rewrote");
    let mine: Captured = captured(&world, &[(3, "theirs"), (4, "mine")]);
    land(&world, &[(3, "theirs"), (4, "theirs again")], "theirs");
    assert!(matches!(merge(&world, &mine), Merged::Conflict(_)));
    remove(world.root.as_path());
}

/// The conflict finding names every path, the commit, the saved message and the command, in words with no internal term.
#[test]
fn the_conflict_text_names_what_the_person_needs_and_leaks_no_internal_term() {
    let conflict: ReplayConflict = ReplayConflict {
        paths: vec![b"a/one.txt".to_vec(), b"two.txt".to_vec()],
        landed: Some(LandedFirst { short: String::from("3f2a1c9"), subject: String::from("Add the thing") }),
        git_dir: PathBuf::from("/work/repo/.git"),
    };
    let saved: PathBuf = PathBuf::from("/work/repo/.git/cli-git-saved-message-ab12");
    let text: String = conflict_text(&conflict, Some(saved.as_path()));
    for fact in ["a/one.txt", "two.txt", "3f2a1c9", "Add the thing", "/work/repo/.git/cli-git-saved-message-ab12", NOTHING_CHANGED] {
        assert!(text.contains(fact), "{fact} missing: {text}");
    }
    assert!(text.contains("`git commit -F /work/repo/.git/cli-git-saved-message-ab12`") && text.contains("`git show 3f2a1c9`"), "{text}");
    let unknown: String = conflict_text(&ReplayConflict { landed: None, ..conflict }, None);
    for shown in [text, unknown] {
        let plain: String = shown.split('`').enumerate().filter(|(at, _)| { return at % 2 == 0; }).map(|(_, part)| { return part; }).collect::<Vec<&str>>().join(" ").to_lowercase();
        for term in DENYLIST {
            assert!(!plain.split(|character: char| { return !character.is_alphanumeric(); }).any(|word| { return word == term; }), "{term} leaked: {shown}");
        }
    }
}

/// Both findings end the command with status 1, land nothing, and are one core finding of the concurrent-commit transform.
#[test]
fn both_endings_are_one_core_finding_with_status_one() {
    let conflict: ReplayConflict = ReplayConflict { paths: vec![b"a.txt".to_vec()], landed: None, git_dir: PathBuf::from("/r/.git") };
    for result in [conflict_result(&conflict, None), already_committed_result("main")] {
        assert_eq!(result.exit_code, 1);
        assert!(result.landed_oid.is_none());
        assert!(matches!(result.events.as_slice(), [PolicyEvent::CoreFinding { core_id: "concurrent-commit", .. }]));
    }
}
