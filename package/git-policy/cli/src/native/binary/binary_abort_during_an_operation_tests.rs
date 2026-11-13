//! What:
//!  The abort and skip forms (`git merge --abort`,
//!  `git rebase --abort`,
//!  `git cherry-pick --abort`,
//!  `git revert --abort`,
//!  `git am --abort` and the `--skip` forms of the same operations)
//!  through the built executable,
//!  while the operation they undo is in progress.
//! Why:
//!  The owner's rule is that a command which would discard work standing nowhere else is refused,
//!  and that the person who means it runs it with the guard's opt-out
//!  (`doc/handover/cli-git-rust-implementation.md`,
//! the answers of 2026-11-12T10:35:07).
//!  An operation that stops with conflicts makes `git status` unclean by itself,
//!  so a guard that read only that status refused every abort form for as long as the
//!  operation waited,
//! which is exactly when a person reaches for one.
//!  Here the abort runs when the only changes are the operation's own,
//!  it is refused before anything is changed when the person's own work would go,
//!  the refusal names that work,
//! the quit form that keeps it and the opt-out,
//!  the opt-out leaves the same state real Git leaves,
//!  and nothing is ever copied.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merge = await wrapped(['merge', 'topic']);           // CONFLICT
//! expect(await wrapped(['merge', '--abort'])).toExit(1);      // refused, merged state intact
//! expect(await wrapped(['merge', '--no-enforce-worktree', '--abort'])).toExit(0);
//! ```

use super::support::{
    Fixture, Observed, bounded_for, direct, executable, fixture, git, remove, repository, run_direct, run_wrapped, stderr_of,
};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs real Git and accepts any status,
///  because an operation that stops with a conflict exits nonzero.
fn attempt(fixture: &Fixture, directory: &Path, arguments: &[&str]) {
    let mut command: Command = direct(fixture);
    command.current_dir(directory).args(arguments).env("GIT_EDITOR", "true").env("GIT_PAGER", "cat");
    let _output = command.output().expect("native Git starts");
}

/// The fixture's `bin/git` link,
///  for the few queries the controls make themselves.
fn plain(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> String {
    let mut command: Command = direct(fixture);
    command.current_dir(directory).args(arguments);
    let output = command.output().expect("native Git starts");
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned()
        + &String::from_utf8_lossy(output.stderr.as_slice());
}

/// The common Git directory of the repository a directory belongs to.
fn common_dir(fixture: &Fixture, directory: &Path) -> PathBuf {
    let text: String = plain(fixture, directory, &["rev-parse", "--path-format=absolute", "--git-common-dir"]);
    return PathBuf::from(text.trim());
}

/// Every file the wrapper's copy folder holds,
/// by its path below that folder.
/// Why:
///   Nothing is copied any more,
///  so every control that could have earned a copy asserts this list is empty.
fn saved_copy_names(common_dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    let root: PathBuf = common_dir.join("monochromatic-git-policy-cli/discarded");
    let mut pending: Vec<PathBuf> = vec![root];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            found.push(path.display().to_string());
        }
    }
    found.sort();
    return found;
}

/// The audit lines of a common Git directory,
///  newest last.
fn audit_lines(common_dir: &Path) -> Vec<String> {
    let text: String = std::fs::read_to_string(common_dir.join("monochromatic-git-policy-cli-audit.jsonl")).unwrap_or_default();
    return text.lines().map(str::to_owned).collect();
}

/// A repository whose `side` branch and `main` branch both changed `a.txt`,
///  with a linked worktree on a new branch `work` checked out at `../work`,
///  which holds both edits and no merge yet.
fn linked_worktree_with_a_diverged_side(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    std::fs::write(repo.join("b.txt"), b"base b\n").expect("tracked file");
    std::fs::write(repo.join("victim.txt"), b"HEAD holds this\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt", "victim.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    std::fs::write(repo.join("a.txt"), b"side\n").expect("side edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), b"main\n").expect("main edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let linked: PathBuf = fixture.root.join("work");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "work", "../work", "main"]);
    return linked;
}

/// Whether the policy rejected the command.
fn rejected(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// What a directory holds,
/// as a person reads it:
///  the porcelain status,
///  the tree `HEAD` names,
///  and the bytes of the files the caller names.
///  `&[&str]` borrows those file names.
/// Why:
///   A refusal's opt-out has to leave the state real Git leaves,
///  and comparing a status against a status misses the bytes Git replaced.
fn state_after(fixture: &Fixture, directory: &Path, paths: &[&str]) -> String {
    let mut text: String = plain(fixture, directory, &["status", "--porcelain=v1"]);
    // The tree is content-addressed, so two fixtures built the same way have the same tree and not the same commit
    //  (a commit also holds its time), which is what a comparison of two states can use.
    text.push_str(plain(fixture, directory, &["rev-parse", "HEAD^{tree}"]).as_str());
    for path in paths {
        let bytes: Vec<u8> = std::fs::read(directory.join(path)).unwrap_or_default();
        text.push_str(format!("\n{path} holds {}", String::from_utf8_lossy(bytes.as_slice())).as_str());
    }
    return text;
}

/// A merge of `side` that stopped at a conflict,
///  whose hand resolution is the person's own work.
///  `Option<&str>` names the merge strategy, and nothing uses the one Git picks.
fn linked_worktree_at_a_conflicted_merge_with(fixture: &Fixture, strategy: Option<&str>) -> PathBuf {
    let linked: PathBuf = linked_worktree_with_a_diverged_side(fixture);
    let arguments: Vec<&str> = match strategy {
        Some(named) => vec!["merge", "--no-ff", "-s", named, "side"],
        None => vec!["merge", "side"],
    };
    attempt(fixture, linked.as_path(), arguments.as_slice());
    let during: String = plain(fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "a conflicted merge is in progress: {during}");
    return linked;
}

/// The same, with the strategy Git picks.
fn linked_worktree_at_a_conflicted_merge(fixture: &Fixture) -> PathBuf {
    return linked_worktree_at_a_conflicted_merge_with(fixture, None);
}

/// What:
///  A repository whose `a.txt` `side` and `main` both changed,
///  with the merge of `side` stopped at a conflict in the main worktree itself.
///  `Option<&str>` names the merge strategy.
/// Why:
///  The owner decided on 2026-11-13 that an abort is judged by the work it would discard
///  in the main worktree as in a linked one,
///  which ends the older rule that every abort there is refused;
///  both outcomes need a control in the main worktree itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = mainWorktreeAtAConflictedMerge(fixture, undefined);
/// ```
fn main_worktree_at_a_conflicted_merge(fixture: &Fixture, strategy: Option<&str>) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    std::fs::write(repo.join("a.txt"), b"side\n").expect("side edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), b"main\n").expect("main edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let arguments: Vec<&str> = match strategy {
        Some(named) => vec!["merge", "--no-ff", "-s", named, "side"],
        None => vec!["merge", "side"],
    };
    attempt(fixture, repo.as_path(), arguments.as_slice());
    let during: String = plain(fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "a conflicted merge is in progress in the main worktree: {during}");
    return repo;
}

/// The merge's own state is the only thing there is,
///  and the abort runs exactly as real Git runs it,
///  saves nothing and adds no line.
#[test]
fn a_merge_abort_runs_when_the_merge_s_own_changes_are_all_there_is() {
    let fixture: Fixture = fixture("abort-merge-own");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    let after: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    let copies: Vec<String> = saved_copy_names(common_dir(&fixture, linked.as_path()).as_path());
    let contents: String = std::fs::read_to_string(linked.join("a.txt")).unwrap_or_default();
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert_eq!(after.trim(), "", "the worktree is back to the state before the merge: {said}");
    assert_eq!(contents, "main\n", "the file holds the branch's own content again: {said}");
    assert!(copies.is_empty(), "the merge's own state needs no copy: {copies:?}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge state is gone: {said}");
    let common: PathBuf = common_dir(&fixture, linked.as_path());
    let audit: Vec<String> = audit_lines(common.as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"allowed\""), "{last}");
    assert!(last.contains("\"form\":\"AbortMerge\""), "{last}");
    assert!(last.contains("\"dirty\":null"), "no status was read: {last}");
    remove(&fixture);
}

/// A conflict resolved by hand is the person's work,
///  so the abort is refused before anything is changed,
///  and the refusal names the file,
/// the kind of work,
/// the quit form and the opt-out.
#[test]
fn a_merge_abort_refuses_a_hand_resolution_and_names_it() {
    let fixture: Fixture = fixture("abort-merge-hand");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let before: String = state_after(&fixture, linked.as_path(), &["a.txt"]);
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the abort would discard the resolution: {said}");
    assert!(said.contains("git merge --abort is refused"), "{said}");
    assert!(said.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{said}");
    assert!(said.contains("Nothing was changed, and nothing was copied."), "{said}");
    assert!(said.contains("git merge --quit"), "the way out that keeps the work: {said}");
    assert!(said.contains("git merge --no-enforce-worktree --abort"), "the opt-out: {said}");
    assert_eq!(state_after(&fixture, linked.as_path(), &["a.txt"]), before, "nothing was changed: {said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge still waits: {said}");
    let copies: Vec<String> = saved_copy_names(common_dir(&fixture, linked.as_path()).as_path());
    assert!(copies.is_empty(), "nothing was copied: {copies:?}");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"rejected\""), "{last}");
    assert!(last.contains("\"form\":\"AbortMerge\""), "{last}");
    assert!(!last.contains("saved-"), "no record claims a copy: {last}");
    remove(&fixture);
}

/// The opt-out runs the abort,
/// and the state it leaves is the state real Git leaves
///  in a repository built the same way.
#[test]
fn the_opt_out_leaves_the_state_real_git_leaves() {
    let hatched: Fixture = fixture("abort-merge-hatch");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&hatched);
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&hatched, linked.as_path(), &["merge", "--no-enforce-worktree", "--abort"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!git_path_is_there(&hatched, linked.as_path(), "MERGE_HEAD"), "the merge is gone: {said}");
    // The same state, built the same way, where real Git runs the abort itself.
    let oracle: Fixture = fixture("abort-merge-hatch-oracle");
    let oracle_linked: PathBuf = linked_worktree_at_a_conflicted_merge(&oracle);
    std::fs::write(oracle_linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let direct: Observed = run_direct(&oracle, oracle_linked.as_path(), &["merge", "--abort"]);
    assert_eq!(direct.code, Some(0), "real Git runs the abort in the same state: {}", stderr_of(&direct));
    assert_eq!(
        state_after(&hatched, linked.as_path(), &["a.txt", "victim.txt"]),
        state_after(&oracle, oracle_linked.as_path(), &["a.txt", "victim.txt"]),
        "the opt-out leaves what real Git leaves"
    );
    let audit: Vec<String> = audit_lines(common_dir(&hatched, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"hatch\""), "the record says the guard stepped aside: {last}");
    remove(&hatched);
    remove(&oracle);
}

/// The advice to stop the operation and keep the work gets the person out:
///  the quit form is allowed,
/// and it keeps the resolution and the files.
#[test]
fn the_advice_to_quit_keeps_the_work_and_ends_the_operation() {
    let fixture: Fixture = fixture("abort-merge-quit");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let refused: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    assert!(rejected(&refused), "the abort is refused first: {}", stderr_of(&refused));
    assert!(stderr_of(&refused).contains("git merge --quit"), "the refusal names the quit form");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--quit"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge is over: {said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "the resolution is kept: {said}"
    );
    let staged: String = plain(&fixture, linked.as_path(), &["diff", "--cached", "--name-only"]);
    assert!(staged.contains("a.txt") || staged.contains("something"), "the index keeps what the quit left: {staged}");
    remove(&fixture);
}

/// Content that stands only in the index is refused as such,
///  and the opt-out runs the abort as real Git does.
#[test]
fn a_merge_abort_refuses_content_only_the_index_holds() {
    let fixture: Fixture = fixture("abort-merge-index-only");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    std::fs::write(linked.join("new.txt"), b"content that stands only in the index\n").expect("the staged file");
    git(&fixture, linked.as_path(), &["add", "--", "new.txt"]);
    std::fs::remove_file(linked.join("new.txt")).expect("the worktree file goes");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the abort would discard the staged content: {said}");
    assert!(said.contains("new.txt (content only the index holds)"), "{said}");
    assert!(said.contains("git merge --quit"), "{said}");
    let copies: Vec<String> = saved_copy_names(common_dir(&fixture, linked.as_path()).as_path());
    assert!(copies.is_empty(), "nothing was copied: {copies:?}");
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--no-enforce-worktree", "--abort"]);
    assert_eq!(hatched.code, Some(0), "the opt-out runs the abort: {}", stderr_of(&hatched));
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge is gone");
    remove(&fixture);
}

/// Every abort form is left to Git where no operation of its kind waits,
///  and Git's own refusal is what the caller sees,
///  because such a command discards nothing.
#[test]
fn every_abort_form_is_left_to_git_where_nothing_is_in_progress() {
    let fixture: Fixture = fixture("abort-nothing");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    std::fs::write(linked.join("a.txt"), b"the person's own work\n").expect("edit");
    let before: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    for arguments in [
        vec!["merge", "--abort"],
        vec!["rebase", "--abort"],
        vec!["cherry-pick", "--abort"],
        vec!["revert", "--abort"],
        vec!["am", "--abort"],
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        let lowered: String = said.to_lowercase();
        assert!(!rejected(&observed), "{arguments:?} is left to Git: {said}");
        assert_ne!(observed.code, Some(0), "{arguments:?} with nothing in progress exits nonzero: {said}");
        assert!(
            lowered.contains("no merge to abort")
                || lowered.contains("no rebase in progress")
                || lowered.contains("no cherry-pick or revert in progress")
                || lowered.contains("not in progress"),
            "{arguments:?} carries Git's own words: {said}"
        );
    }
    assert_eq!(
        plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]),
        before,
        "nothing changed"
    );
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("tracked file"),
        b"the person's own work\n"
    );
    assert!(saved_copy_names(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "nothing to save");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let left: Vec<&String> = audit.iter().rev().take(5).collect();
    let forms: Vec<String> = ["AbortAm", "AbortPick", "AbortPick", "AbortRebase", "AbortMerge"]
        .iter()
        .map(|form| { return format!("\"form\":\"{form}\""); })
        .collect();
    for (line, form) in left.iter().zip(forms.iter()) {
        assert!(line.contains("\"outcome\":\"allowed\""), "{line}");
        assert!(line.contains(form.as_str()), "the record names {form}: {line}");
        assert!(line.contains("\"dirty\":null"), "no status was read: {line}");
    }
    remove(&fixture);
}

/// A rebase that stopped at a conflict is undone by its own abort,
///  and the hand edit it would discard is refused.
#[test]
fn a_rebase_abort_refuses_the_person_s_edit() {
    let fixture: Fixture = fixture("abort-rebase");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(!during.trim().is_empty(), "the rebase stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the rebase abort would discard the edit: {said}");
    assert!(said.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{said}");
    assert!(said.contains("git rebase --quit"), "{said}");
    assert!(said.contains("git rebase --no-enforce-worktree --abort"), "{said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "nothing was changed: {said}"
    );
    remove(&fixture);
}

/// A cherry-pick that stopped at a conflict refuses the hand resolution,
///  and its own quit form keeps the work.
#[test]
fn a_cherry_pick_abort_refuses_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-pick");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    let side: String = plain(&fixture, linked.as_path(), &["rev-parse", "side"]).trim().to_string();
    attempt(&fixture, linked.as_path(), &["cherry-pick", side.as_str()]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the pick stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the pick abort would discard the resolution: {said}");
    assert!(said.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{said}");
    assert!(said.contains("git cherry-pick --quit or git revert --quit"), "{said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "CHERRY_PICK_HEAD"), "the pick still waits: {said}");
    let quit: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--quit"]);
    assert_eq!(quit.code, Some(0), "the quit form is allowed: {}", stderr_of(&quit));
    assert!(!git_path_is_there(&fixture, linked.as_path(), "CHERRY_PICK_HEAD"), "the pick is over");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "the quit keeps the resolution"
    );
    remove(&fixture);
}

/// An `am` that stopped at a conflicting patch refuses the resolution,
///  and the opt-out aborts it exactly as real Git does.
#[test]
fn an_am_abort_refuses_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-am");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    let patches: PathBuf = fixture.root.join("patches");
    std::fs::create_dir_all(patches.as_path()).expect("patch directory");
    git(
        &fixture,
        linked.as_path(),
        &["format-patch", "--full-index", "--output-directory", patches.to_str().expect("UTF-8 path"), "side", "--max-count=1"],
    );
    let patch: PathBuf = std::fs::read_dir(patches.as_path())
        .expect("patch directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .next()
        .expect("one patch");
    let before: Vec<u8> = std::fs::read(linked.join("a.txt")).expect("the file before the am");
    // `--3way` is what makes the patch conflict in the index;
    //  without it Git refuses the patch and changes neither the index nor the file.
    attempt(&fixture, linked.as_path(), &["am", "--3way", patch.to_str().expect("UTF-8 path")]);
    let applying: String = plain(&fixture, linked.as_path(), &["rev-parse", "--path-format=absolute", "--git-path", "rebase-apply/applying"]);
    assert!(Path::new(applying.trim()).exists(), "the am stopped and waits: {applying}");
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the patch conflicted: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the am abort would discard the resolution: {said}");
    assert!(said.contains("git am --abort is refused"), "{said}");
    assert!(said.contains("git am --quit"), "{said}");
    assert!(said.contains("git am --no-enforce-worktree --abort"), "{said}");
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--no-enforce-worktree", "--abort"]);
    assert_eq!(hatched.code, Some(0), "the opt-out aborts the am: {}", stderr_of(&hatched));
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file after the abort"),
        before,
        "the file holds what stood before the am"
    );
    assert_eq!(plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(), "", "the am is undone");
    remove(&fixture);
}

/// A linked worktree whose branch holds a merge of `feature`,
///  with `main` moved on,
///  so `git rebase --rebase-merges main` replays the merge and stops at it.
///  `local_env` adds a commit after the merge that tracks `local.env`,
///  which the replayed merge step is reached before.
fn linked_worktree_whose_rebase_recreates_a_merge(fixture: &Fixture, local_env: bool) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=c0"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "feature"]);
    std::fs::write(repo.join("a.txt"), b"feature\n").expect("feature edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=c1"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), b"mainline\n").expect("mainline edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=c2"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "integration"]);
    attempt(fixture, repo.as_path(), &["merge", "feature"]);
    std::fs::write(repo.join("a.txt"), b"merged\n").expect("the merge's resolution");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=c3-merge"]);
    if local_env {
        std::fs::write(repo.join("local.env"), b"tracked\n").expect("the tracked local.env");
        git(fixture, repo.as_path(), &["add", "--", "local.env"]);
        git(fixture, repo.as_path(), &["commit", "--quiet", "--message=c5-adds-local.env"]);
    }
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), b"newmain\n").expect("newmain edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=c4"]);
    let linked: PathBuf = fixture.root.join("work");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "work", "../work", "integration"]);
    return linked;
}

/// Whether a path inside a worktree's Git directory is there.
fn git_path_is_there(fixture: &Fixture, directory: &Path, name: &str) -> bool {
    return Path::new(plain(fixture, directory, &["rev-parse", "--path-format=absolute", "--git-path", name]).trim()).exists();
}

/// A rebase that replays a merge is still the operation `git rebase --abort` ends,
///  even beside the `MERGE_HEAD` of the merge step it stopped at,
///  so the hand resolution is refused and the rebase is what is named.
#[test]
fn a_rebase_abort_at_a_rebase_merges_merge_step_refuses_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-rebase-merges-step");
    let linked: PathBuf = linked_worktree_whose_rebase_recreates_a_merge(&fixture, false);
    attempt(&fixture, linked.as_path(), &["rebase", "--rebase-merges", "main"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge step waits");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase waits");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the hand resolution is refused before the rebase goes: {said}");
    assert!(said.contains("a.txt"), "the refusal names the file: {said}");
    assert!(said.contains("the rebase in progress still waits"), "{said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    assert!(saved_copy_names(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "nothing was copied");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"form\":\"AbortRebase\""), "{last}");
    assert!(last.contains("\"outcome\":\"rejected\""), "{last}");
    remove(&fixture);
}

/// `git am --abort` ends a rebase that applies patches,
///  so the refusal names the rebase it ends and the command as typed.
#[test]
fn an_am_abort_during_an_apply_backend_rebase_refuses_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-am-ends-rebase");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "--apply", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-apply/rebasing"), "the rebase waits");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the hand resolution is refused: {said}");
    assert!(said.contains("git am --abort is refused"), "the command is named as typed: {said}");
    assert!(said.contains("rebase in progress still waits"), "the operation it ends is named: {said}");
    assert!(!said.contains("the am in progress"), "no sentence calls the rebase an am: {said}");
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--no-enforce-worktree", "--abort"]);
    assert_eq!(hatched.code, Some(0), "the opt-out aborts the rebase: {}", stderr_of(&hatched));
    assert!(!git_path_is_there(&fixture, linked.as_path(), "rebase-apply/rebasing"), "the rebase is gone");
    remove(&fixture);
}

/// The tree a pick sequence restores is the one its own sequencer recorded,
///  not whatever `ORIG_HEAD` last held:
///  an ignored file the restored tree tracks is refused and its content kept.
#[test]
fn a_pick_abort_checks_the_ignored_files_of_the_tree_the_sequence_restores() {
    let fixture: Fixture = fixture("abort-pick-sequence-ignored");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=c0"]);
    let linked: PathBuf = fixture.root.join("work");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "work", "../work", "main"]);
    // The worktree's own commits track `local.env`, while `ORIG_HEAD` still names the worktree's creation commit.
    std::fs::write(linked.join("local.env"), b"tracked\n").expect("the tracked local.env");
    git(&fixture, linked.as_path(), &["add", "--", "local.env"]);
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--message=w1-adds-local.env"]);
    std::fs::write(linked.join("a.txt"), b"work\n").expect("the worktree's edit");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=w2"]);
    let tracks_local_env: String = plain(&fixture, linked.as_path(), &["ls-tree", "HEAD", "local.env"]);
    assert!(tracks_local_env.contains("local.env"), "the worktree's HEAD tracks local.env: {tracks_local_env}");
    // A branch from the commit that added `local.env`: one pick deletes it, the next conflicts on `a.txt`.
    git(&fixture, linked.as_path(), &["checkout", "--quiet", "-b", "removal", "work~1"]);
    git(&fixture, linked.as_path(), &["rm", "--quiet", "local.env"]);
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--message=r1-deletes-local.env"]);
    std::fs::write(linked.join("a.txt"), b"removal\n").expect("the removal's edit");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=r2"]);
    git(&fixture, linked.as_path(), &["checkout", "--quiet", "work"]);
    std::fs::write(repo.join(".git/info/exclude"), b"local.env\n").expect("the exclude pattern");
    attempt(&fixture, linked.as_path(), &["cherry-pick", "removal~1", "removal"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the second pick stopped: {during}");
    let orig_head: String = plain(&fixture, linked.as_path(), &["ls-tree", "ORIG_HEAD", "local.env"]);
    assert!(!orig_head.contains("local.env"), "ORIG_HEAD does not track local.env, so reading it is what loses the file: {orig_head}");
    // The person's own content, in a file the exclude pattern makes ignored.
    std::fs::write(linked.join("local.env"), b"my secret\n").expect("the person's file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the ignored file at a path the sequence's tree tracks keeps the refusal: {said}");
    assert!(said.contains("local.env"), "the refusal names the file: {said}");
    assert_eq!(
        std::fs::read(linked.join("local.env")).expect("the person's file"),
        b"my secret\n",
        "nothing was written over the ignored file: {said}"
    );
    assert!(saved_copy_names(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "nothing was copied because nothing ran");
    remove(&fixture);
}

/// The tree a rebase restores is the one its own backend recorded,
///  not whatever `ORIG_HEAD` last held:
///  a wrapped `git merge --abort` rewrites `ORIG_HEAD` between the rebase steps,
///  and the ignored file the rebase's own record tracks keeps the refusal.
#[test]
fn a_rebase_abort_checks_the_ignored_files_of_the_original_head_it_restores() {
    let fixture: Fixture = fixture("abort-rebase-orig-head-ignored");
    let linked: PathBuf = linked_worktree_whose_rebase_recreates_a_merge(&fixture, true);
    std::fs::write(common_dir(&fixture, linked.as_path()).join("info/exclude"), b"local.env\n").expect("the exclude pattern");
    attempt(&fixture, linked.as_path(), &["rebase", "--rebase-merges", "main"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase waits");
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "at its merge step");
    let merge_abort: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let merge_said: String = stderr_of(&merge_abort);
    assert_eq!(merge_abort.code, Some(0), "{merge_said}");
    let rewritten: String = plain(&fixture, linked.as_path(), &["ls-tree", "ORIG_HEAD", "local.env"]);
    assert!(
        !rewritten.contains("local.env"),
        "the merge abort rewrote ORIG_HEAD to a tree that does not track local.env: {rewritten}"
    );
    std::fs::write(linked.join("local.env"), b"my secret\n").expect("the person's file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the ignored file at a path the rebase's record tracks keeps the refusal: {said}");
    assert!(said.contains("local.env"), "the refusal names the file: {said}");
    assert_eq!(
        std::fs::read(linked.join("local.env")).expect("the person's file"),
        b"my secret\n",
        "nothing was written over the ignored file: {said}"
    );
    remove(&fixture);
}

/// A skip drops the hand resolution of the conflict it stops at,
///  so it is refused,
/// and the opt-out runs the skip as real Git runs it.
#[test]
fn a_rebase_skip_refuses_the_hand_resolution() {
    let fixture: Fixture = fixture("skip-rebase");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the rebase stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--skip"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the skip would discard the resolution: {said}");
    assert!(said.contains("git rebase --skip is refused"), "{said}");
    assert!(said.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{said}");
    assert!(said.contains("git rebase --quit"), "{said}");
    assert!(said.contains("git rebase --no-enforce-worktree --skip"), "{said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "nothing was changed: {said}"
    );
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--no-enforce-worktree", "--skip"]);
    let hatched_said: String = stderr_of(&hatched);
    assert_ne!(hatched.code, Some(1), "the opt-out does not refuse: {hatched_said}");
    assert!(!hatched_said.contains("is refused:"), "{hatched_said}");
    remove(&fixture);
}

/// A skip that discards nothing a person made runs,
/// in the main worktree as in a linked one:
///  the operation's own state is all there is.
#[test]
fn a_skip_that_discards_nothing_runs_in_the_main_worktree() {
    let fixture: Fixture = fixture("skip-main-worktree");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    std::fs::write(repo.join("a.txt"), b"side\n").expect("side edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), b"main\n").expect("main edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    attempt(&fixture, repo.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the rebase stopped: {during}");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["rebase", "--skip"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "the skip discards nothing the person made: {said}");
    assert!(!rejected(&observed), "{said}");
    assert!(saved_copy_names(common_dir(&fixture, repo.as_path()).as_path()).is_empty(), "nothing was copied");
    remove(&fixture);
}

/// What:
///  A stand-in for real Git that pauses on the first start made after a marker file appears,
///  so a control can write to a file inside the window between the wrapper's measurement and Git.
/// Why:
///   The window is what the second measurement exists for,
///  and nothing else makes it deterministically reachable:
///  the marker is the audit line the policy pass writes once it has decided the command may run,
///  so the paused start is the first one of the second measurement itself.
///  Every other invocation is real Git,
/// unchanged.
fn pausing_real_git(fixture: &Fixture, marker: &Path, paused: &Path, release: &Path) -> PathBuf {
    let directory: PathBuf = fixture.root.join("shim");
    std::fs::create_dir(directory.as_path()).expect("shim directory");
    let shim: PathBuf = directory.join("git");
    executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\n\
             if [ ! -f '{done}' ] && [ -f '{marker}' ]; then\n\
             \t: > '{done}'\n\
             \t: > '{paused}'\n\
             \twaited=0\n\
             \twhile [ ! -f '{release}' ] && [ \"$waited\" -lt 400 ]; do\n\
             \t\tsleep 0.01\n\
             \t\twaited=$((waited + 1))\n\
             \tdone\n\
             fi\n\
             exec /usr/bin/git \"$@\"\n",
            done = fixture.root.join("shim-paused.once").display(),
            marker = marker.display(),
            paused = paused.display(),
            release = release.display()
        )
        .as_bytes(),
    );
    return shim;
}

/// The bound the control that pauses a forwarded command gives it:
///  the command waits for the control's own release,
/// so the shared five-second bound is not a limit on it.
const PAUSED_COMMAND_SECONDS: &str = "60";

/// The wrapper of `fixture`,
/// with the pausing stand-in for real Git and no `/usr/bin` before it.
fn wrapper_paused_at_real_git(fixture: &Fixture, shim: &Path) -> Command {
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(shim.parent().expect("the shim's directory").as_os_str());
    path.push(":/bin");
    return bounded_for(fixture, fixture.root.join("bin/git").as_path(), path.as_os_str(), PAUSED_COMMAND_SECONDS);
}

/// Starts the paused wrapper for `git merge --abort` in `linked`
///  and hands back the child once its stand-in for real Git has paused.
///  `&Path` borrows the marker,
/// the pause file and the release file.
/// Why:
///   Both controls of the window need the same steps,
///  and a control that waited differently could reach the pause after it had released it.
fn start_at_the_pause(fixture: &Fixture, linked: &Path, marker: &Path, paused: &Path, release: &Path) -> std::process::Child {
    let shim: PathBuf = pausing_real_git(fixture, marker, paused, release);
    let mut command: Command = wrapper_paused_at_real_git(fixture, shim.as_path());
    let mut child: std::process::Child = command
        .current_dir(linked)
        .args(["merge", "--abort"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the wrapper starts");
    // A bounded wait for the second measurement's own Git start, so nothing here waits forever.
    let mut waited: std::time::Duration = std::time::Duration::ZERO;
    while !paused.exists() && waited < std::time::Duration::from_secs(45) {
        std::thread::sleep(std::time::Duration::from_millis(10));
        waited += std::time::Duration::from_millis(10);
    }
    if !paused.exists() {
        let _killed = child.kill();
        let told: std::process::Output = child.wait_with_output().expect("the wrapper finishes");
        panic!(
            "the wrapper reached no second measurement (exit {:?}); said: {}",
            told.status.code(),
            String::from_utf8_lossy(told.stderr.as_slice())
        );
    }
    return child;
}

/// An edit written between the wrapper's measurement and Git's start is seen by the second measurement,
///  and the command is refused with that work named.
#[test]
fn an_edit_written_between_the_measurement_and_git_s_start_is_refused() {
    let fixture: Fixture = fixture("abort-edit-after-measurement");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    let common: PathBuf = common_dir(&fixture, linked.as_path());
    let marker: PathBuf = common.join("monochromatic-git-policy-cli-audit.jsonl");
    let paused: PathBuf = fixture.root.join("shim-paused");
    let release: PathBuf = fixture.root.join("shim-release");
    let child: std::process::Child = start_at_the_pause(&fixture, linked.as_path(), marker.as_path(), paused.as_path(), release.as_path());
    // The edit the first measurement could not see, written while the second one waits.
    std::fs::write(linked.join("a.txt"), b"resolution written after the measurement\n").expect("the later resolution");
    std::fs::write(release.as_path(), b"go").expect("the release");
    let output: std::process::Output = child.wait_with_output().expect("the wrapper finishes");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(1), "the second measurement refuses the command: {said}");
    assert!(said.contains("git merge --abort is refused"), "{said}");
    assert!(said.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{said}");
    assert!(said.contains("This work appeared between the wrapper's measurement and Git's start."), "{said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "nothing was changed: {said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolution written after the measurement\n",
        "the edit is still there: {said}"
    );
    assert!(saved_copy_names(common.as_path()).is_empty(), "nothing was copied");
    let audit: Vec<String> = audit_lines(common.as_path());
    assert!(audit.iter().any(|line| { return line.contains("\"outcome\":\"rejected\""); }), "{audit:?}");
    remove(&fixture);
}

/// The same pause with nothing written during it lets the command run:
///  the control above can only prove what it claims if this one runs.
#[test]
fn the_same_pause_with_no_edit_lets_the_abort_run() {
    let fixture: Fixture = fixture("abort-edit-after-measurement-control");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    let common: PathBuf = common_dir(&fixture, linked.as_path());
    let marker: PathBuf = common.join("monochromatic-git-policy-cli-audit.jsonl");
    let paused: PathBuf = fixture.root.join("shim-paused");
    let release: PathBuf = fixture.root.join("shim-release");
    let child: std::process::Child = start_at_the_pause(&fixture, linked.as_path(), marker.as_path(), paused.as_path(), release.as_path());
    std::fs::write(release.as_path(), b"go").expect("the release");
    let output: std::process::Output = child.wait_with_output().expect("the wrapper finishes");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "the abort ran: {said}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge is gone: {said}");
    remove(&fixture);
}

/// What:
///  A repository whose linked worktree stops an interactive rebase at an `edit` step,
///  with a commit the next steps would apply still ahead of it.
///  `local_env` adds an earlier commit that tracks `local.env`,
///  which the stop's own `reset` can move out of the way.
/// Why:
///   An edit stop is where a person splits or amends a commit,
///  and a second operation started there writes beside the rebase's own state:
///  the combination the wrapper's per-command reading has to get right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const work = stopAtAnEditStep(fixture, false);
/// ```
fn worktree_stopped_at_an_edit_step(fixture: &Fixture, local_env: bool) -> PathBuf {
    let editor: PathBuf = fixture.root.join("todo-editor.sh");
    executable(editor.as_path(), b"#!/bin/sh\nsed -i -e 's/^pick /edit /' -- \"$1\"\n");
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    std::fs::write(repo.join("other.txt"), b"other\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "other.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked: PathBuf = fixture.root.join("work");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "work", "../work", "main"]);
    let mut first: Command = direct(fixture);
    first.current_dir(linked.as_path()).args(["commit", "--quiet", "--allow-empty", "--message=T0"]);
    let _made = first.output().expect("native Git starts");
    // A commit the stop never applies: its tree is the one `rebase-merge/orig-head` records.
    if local_env {
        std::fs::write(linked.join("local.env"), b"tracked\n").expect("the tracked local.env");
        git(fixture, linked.as_path(), &["add", "--", "local.env"]);
        git(fixture, linked.as_path(), &["commit", "--quiet", "--message=adds-local.env"]);
    }
    git(fixture, linked.as_path(), &["branch", "side", "main"]);
    let mut side: Command = direct(fixture);
    side.current_dir(repo.as_path()).args(["switch", "--quiet", "side"]);
    let _switched = side.output().expect("native Git starts");
    std::fs::write(repo.join("a.txt"), b"side change\n").expect("side edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=side changes a"]);
    let mut back: Command = direct(fixture);
    back.current_dir(repo.as_path()).args(["switch", "--quiet", "main"]);
    let _back = back.output().expect("native Git starts");
    let mut stopped: Command = direct(fixture);
    stopped
        .current_dir(linked.as_path())
        .args(["rebase", "--interactive", "main"])
        .env("GIT_SEQUENCE_EDITOR", editor.as_os_str())
        .env("GIT_EDITOR", "true");
    let _ran = stopped.output().expect("native Git starts");
    return linked;
}

/// A merge that stops inside an edit step of an interactive rebase leaves two operations' state at once,
///  and `git rebase --abort` is a word Git acts on:
/// the hand resolution and the unstaged edit are both refused.
#[test]
fn a_rebase_abort_inside_an_edit_step_refuses_the_hand_resolution_and_the_unstaged_edit() {
    let fixture: Fixture = fixture("abort-rebase-edit-step-merge");
    let linked: PathBuf = worktree_stopped_at_an_edit_step(&fixture, false);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the edit step waits");
    // A commit made at the stop, which the merge then conflicts with.
    std::fs::write(linked.join("a.txt"), b"edited where the stop landed\n").expect("the stop's own change");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=the stop changes a"]);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge waits beside the rebase");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "and the rebase still waits");
    std::fs::write(linked.join("a.txt"), b"hand resolution of the merge\n").expect("the resolution");
    std::fs::write(linked.join("other.txt"), b"an edit outside the merge\n").expect("the unstaged edit");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the abort would discard both: {said}");
    assert!(said.contains("a.txt"), "the hand resolution is named: {said}");
    assert!(said.contains("other.txt"), "the unstaged edit is named: {said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    assert_eq!(
        std::fs::read(linked.join("other.txt")).expect("the file"),
        b"an edit outside the merge\n",
        "nothing was changed: {said}"
    );
    remove(&fixture);
}

/// `git cherry-pick --abort` is live while a cherry-pick waits inside a rebase's edit step,
///  and the work it would discard is refused.
#[test]
fn a_cherry_pick_abort_inside_an_edit_step_refuses_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-pick-inside-edit-step");
    let linked: PathBuf = worktree_stopped_at_an_edit_step(&fixture, false);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the edit step waits");
    std::fs::write(linked.join("a.txt"), b"edited where the stop landed\n").expect("the stop's own change");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=the stop changes a"]);
    attempt(&fixture, linked.as_path(), &["cherry-pick", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "CHERRY_PICK_HEAD"), "the pick waits beside the rebase");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "and the rebase still waits");
    std::fs::write(linked.join("a.txt"), b"hand resolution of the pick\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the pick abort would discard the resolution: {said}");
    assert!(said.contains("a.txt"), "the refusal names the file: {said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "CHERRY_PICK_HEAD"), "the pick still waits: {said}");
    remove(&fixture);
}

/// The ordinary split-commit step at an edit stop (`git reset HEAD^`) moves `ORIG_HEAD`,
///  and the rebase's own record is what its abort restores:
///  an ignored file the original tip tracks keeps the refusal and its content.
#[test]
fn a_rebase_abort_after_a_reset_at_an_edit_step_reads_the_rebases_own_record() {
    let fixture: Fixture = fixture("abort-rebase-orig-head-moved");
    let linked: PathBuf = worktree_stopped_at_an_edit_step(&fixture, true);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the edit step waits");
    let record: String = std::fs::read_to_string(Path::new(
        plain(&fixture, linked.as_path(), &["rev-parse", "--path-format=absolute", "--git-path", "rebase-merge/orig-head"]).trim(),
    ))
    .expect("the rebase's own record");
    let recorded: &str = record.trim();
    let tracks: String = plain(&fixture, linked.as_path(), &["ls-tree", recorded, "local.env"]);
    assert!(tracks.contains("local.env"), "the rebase's record tracks local.env: {tracks}");
    // The split-commit step a person takes at an edit stop.
    attempt(&fixture, linked.as_path(), &["reset", "--quiet", "HEAD^"]);
    let moved: String = plain(&fixture, linked.as_path(), &["ls-tree", "ORIG_HEAD", "local.env"]);
    assert!(!moved.contains("local.env"), "the reset moved ORIG_HEAD to a tree that does not track it: {moved}");
    std::fs::write(common_dir(&fixture, linked.as_path()).join("info/exclude"), b"local.env\n").expect("the exclude pattern");
    std::fs::write(linked.join("local.env"), b"my secret\n").expect("the person's ignored file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the ignored file the record tracks keeps the refusal: {said}");
    assert!(said.contains("local.env"), "the refusal names the file: {said}");
    assert_eq!(
        std::fs::read(linked.join("local.env")).expect("the person's ignored file"),
        b"my secret\n",
        "nothing was written over the ignored file: {said}"
    );
    remove(&fixture);
}

/// An untracked file at a path the restored tree tracks is where Git itself refuses the abort,
///  so the wrapper's words say that Git refuses and name the file,
///  and the opt-out gets Git's own refusal in Git's own words.
#[test]
fn an_untracked_file_in_the_way_is_named_as_gits_own_refusal() {
    let fixture: Fixture = fixture("abort-untracked-in-the-way");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    // The path stays on disk and leaves the index, so the worktree holds it as an untracked file
    //  while the tree the abort restores still tracks it.
    git(&fixture, linked.as_path(), &["rm", "--cached", "--quiet", "--", "victim.txt"]);
    std::fs::write(linked.join("victim.txt"), b"the person's own file\n").expect("the untracked file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "Git would refuse this abort: {said}");
    assert!(said.contains("Git itself refuses it here"), "the words say it is Git that refuses: {said}");
    assert!(said.contains("victim.txt"), "the refusal names the file: {said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge still waits: {said}");
    assert_eq!(
        std::fs::read(linked.join("victim.txt")).expect("the person's file"),
        b"the person's own file\n",
        "nothing was written over it: {said}"
    );
    // With the guard's opt-out the command reaches Git, which refuses it in its own words.
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--no-enforce-worktree", "--abort"]);
    let hatched_said: String = stderr_of(&hatched);
    assert_ne!(hatched.code, Some(0), "Git refuses the abort itself: {hatched_said}");
    assert!(
        hatched_said.to_lowercase().contains("would be overwritten"),
        "Git's own words name the file it will not overwrite: {hatched_said}"
    );
    assert!(hatched_said.contains("victim.txt"), "{hatched_said}");
    remove(&fixture);
}

/// A skip's reset refuses an untracked file at a path the tree it writes tracks,
///  as the abort's does (measured with Git 2.56.0 for every skip form),
///  so the words say that Git refuses and the file is kept.
#[test]
fn a_skip_is_refused_for_an_untracked_file_gits_own_reset_will_not_write_over() {
    let fixture: Fixture = fixture("skip-untracked-in-the-way");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the rebase stopped: {during}");
    git(&fixture, linked.as_path(), &["rm", "--cached", "--quiet", "--", "victim.txt"]);
    std::fs::write(linked.join("victim.txt"), b"the person's own file\n").expect("the untracked file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--skip"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "Git refuses this skip itself: {said}");
    assert!(said.contains("git rebase --skip is refused because Git itself refuses it here"), "{said}");
    assert!(said.contains("victim.txt"), "the refusal names the file: {said}");
    assert_eq!(
        std::fs::read(linked.join("victim.txt")).expect("the person's file"),
        b"the person's own file\n",
        "nothing was written over it: {said}"
    );
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    // The same command with the guard's opt-out reaches Git, which refuses it in its own words.
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--no-enforce-worktree", "--skip"]);
    let hatched_said: String = stderr_of(&hatched);
    assert_ne!(hatched.code, Some(0), "Git refuses the skip itself: {hatched_said}");
    assert!(
        hatched_said.to_lowercase().contains("would be overwritten"),
        "Git's own words name the file it will not overwrite: {hatched_said}"
    );
    assert!(hatched_said.contains("victim.txt"), "{hatched_said}");
    assert_eq!(
        std::fs::read(linked.join("victim.txt")).expect("the person's file"),
        b"the person's own file\n",
        "Git's own refusal lost nothing either: {hatched_said}"
    );
    remove(&fixture);
}

/// The `--quit` a rebase's refusal names ends that rebase and keeps the hand resolution.
/// Why:
///  A refusal's advice is a promise,
/// and this is the one the refusal gives first.
#[test]
fn the_advice_to_quit_ends_a_rebase_and_keeps_the_work() {
    let fixture: Fixture = fixture("abort-rebase-quit");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the rebase stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let refused: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&refused);
    assert!(rejected(&refused), "{said}");
    assert!(said.contains("git rebase --quit"), "the refusal names the quit form: {said}");
    let quit: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--quit"]);
    let quit_said: String = stderr_of(&quit);
    assert_eq!(quit.code, Some(0), "the quit form the refusal named is allowed: {quit_said}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase is over: {quit_said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "the quit keeps the resolution: {quit_said}"
    );
    remove(&fixture);
}

/// The `--quit` an `am` abort's refusal names ends that `am` and keeps the hand resolution.
#[test]
fn the_advice_to_quit_ends_an_am_and_keeps_the_work() {
    let fixture: Fixture = fixture("abort-am-quit");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    let patches: PathBuf = fixture.root.join("patches");
    std::fs::create_dir_all(patches.as_path()).expect("patch directory");
    git(
        &fixture,
        linked.as_path(),
        &["format-patch", "--full-index", "--output-directory", patches.to_str().expect("UTF-8 path"), "side", "--max-count=1"],
    );
    let patch: PathBuf = std::fs::read_dir(patches.as_path())
        .expect("patch directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .next()
        .expect("one patch");
    attempt(&fixture, linked.as_path(), &["am", "--3way", patch.to_str().expect("UTF-8 path")]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-apply/applying"), "the am waits");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let refused: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--abort"]);
    let said: String = stderr_of(&refused);
    assert!(rejected(&refused), "{said}");
    assert!(said.contains("git am --quit"), "the refusal names the quit form: {said}");
    let quit: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--quit"]);
    let quit_said: String = stderr_of(&quit);
    assert_eq!(quit.code, Some(0), "the quit form the refusal named is allowed: {quit_said}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "rebase-apply/applying"), "the am is over: {quit_said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "the quit keeps the resolution: {quit_said}"
    );
    remove(&fixture);
}

/// The advice to move the untracked file out of the way gets the person out:
///  the command the refusal named then goes through.
#[test]
fn the_advice_to_move_the_untracked_file_away_gets_the_person_out() {
    let fixture: Fixture = fixture("abort-untracked-moved-away");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge(&fixture);
    git(&fixture, linked.as_path(), &["rm", "--cached", "--quiet", "--", "victim.txt"]);
    std::fs::write(linked.join("victim.txt"), b"the person's own file\n").expect("the untracked file");
    let refused: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&refused);
    assert!(rejected(&refused), "{said}");
    assert!(said.contains("victim.txt"), "the refusal names the file: {said}");
    assert!(said.contains("mv <path> <somewhere else>"), "the refusal gives the move as the way out: {said}");
    // The advice, run as printed: the file leaves the worktree and keeps its content.
    let elsewhere: PathBuf = fixture.root.join("victim.moved-away");
    std::fs::rename(linked.join("victim.txt"), elsewhere.as_path()).expect("the file moves out of the worktree");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let after: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "the abort goes through once the file is out of the way: {after}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge is gone: {after}");
    assert_eq!(
        std::fs::read(elsewhere.as_path()).expect("the moved file"),
        b"the person's own file\n",
        "the content the person kept: {after}"
    );
    remove(&fixture);
}

/// The advice a skip's refusal gives about the untracked file works as printed:
///  with the file out of the way the skip goes on.
#[test]
fn the_advice_to_move_the_untracked_file_away_lets_a_skip_go_on() {
    let fixture: Fixture = fixture("skip-untracked-moved-away");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    git(&fixture, linked.as_path(), &["rm", "--cached", "--quiet", "--", "victim.txt"]);
    std::fs::write(linked.join("victim.txt"), b"the person's own file\n").expect("the untracked file");
    let refused: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--skip"]);
    let said: String = stderr_of(&refused);
    assert!(rejected(&refused), "{said}");
    assert!(said.contains("mv <path> <somewhere else>"), "the refusal gives the move as the way out: {said}");
    let elsewhere: PathBuf = fixture.root.join("victim.moved-away");
    std::fs::rename(linked.join("victim.txt"), elsewhere.as_path()).expect("the file moves out of the worktree");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--skip"]);
    let after: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "the skip goes on once the file is out of the way: {after}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase is over: {after}");
    assert_eq!(
        std::fs::read(elsewhere.as_path()).expect("the moved file"),
        b"the person's own file\n",
        "the content the person kept: {after}"
    );
    remove(&fixture);
}

/// A record of the commit an abort restores that is a pipe is not read:
///  the command answers instead of waiting for a writer that never comes,
///  and the advice it prints ends the operation.
/// Why:
///   `git rebase --abort` read `rebase-merge/orig-head` until end of file,
///  and a `mkfifo` at that path held the command for good with nothing able to end it.
#[test]
fn a_rebase_abort_whose_record_is_a_pipe_answers_and_the_advice_ends_it() {
    let fixture: Fixture = fixture("abort-record-is-a-pipe");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase waits");
    let record: PathBuf = PathBuf::from(
        plain(&fixture, linked.as_path(), &["rev-parse", "--path-format=absolute", "--git-path", "rebase-merge/orig-head"]).trim(),
    );
    std::fs::remove_file(record.as_path()).expect("the record goes");
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo").arg(record.as_path()).status().expect("mkfifo runs");
    assert!(made.success(), "mkfifo must create {}", record.display());
    // The shared bound of every fixture command ends a command that waits, so an answer is the only way to pass.
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the pipe is named rather than read: {said}");
    assert!(said.contains("not a regular file"), "the refusal says what stands at the path: {said}");
    assert!(said.contains("rebase-merge/orig-head"), "the refusal names the path: {said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    // The advice it printed, run as printed.
    let quit: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--quit"]);
    let quit_said: String = stderr_of(&quit);
    assert_eq!(quit.code, Some(0), "the quit form the refusal named is allowed: {quit_said}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase is over: {quit_said}");
    remove(&fixture);
}

/// What:
///  Whether the merge Git started recorded the result it computed (`AUTO_MERGE`).
/// Why:
///  The reading leans on that record to tell Git's own conflict text from a resolution the person wrote,
///  and a checked-in strategy can leave it absent,
///  which is the state the fallback has to answer for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = plain(fixture, where, ['rev-parse', '--verify', '--quiet', 'AUTO_MERGE']).trim().length > 0;
/// ```
fn merge_result_recorded(fixture: &Fixture, directory: &Path) -> bool {
    return git_path_is_there(fixture, directory, "AUTO_MERGE");
}

/// A merge under the `resolve` strategy records no result,
///  so a resolution written in the worktree and never staged is refused:
///  nothing proves which of the conflicted paths hold Git's own text,
///  and the abort writes that tree back over all of them.
#[test]
fn a_merge_that_records_no_result_refuses_an_unstaged_resolution() {
    let fixture: Fixture = fixture("abort-merge-resolve-resolution");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge_with(&fixture, Some("resolve"));
    assert!(!merge_result_recorded(&fixture, linked.as_path()), "the resolve strategy records no result");
    std::fs::write(linked.join("a.txt"), b"HAND RESOLUTION written by the person\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the abort would discard the resolution: {said}");
    assert!(said.contains("a.txt (a conflicted path)"), "the path is named as one the wrapper cannot judge: {said}");
    assert!(
        said.contains("Git recorded no result for the conflict it stopped at"),
        "the refusal says why it could not tell the resolution from Git's text: {said}"
    );
    assert!(said.contains("git merge --quit"), "the way out that keeps the work: {said}");
    assert!(said.contains("git merge --no-enforce-worktree --abort"), "the opt-out: {said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge still waits: {said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"HAND RESOLUTION written by the person\n",
        "nothing was changed: {said}"
    );
    assert!(saved_copy_names(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "nothing was copied");
    // The opt-out runs the abort as real Git runs it, which is the loss the refusal named.
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--no-enforce-worktree", "--abort"]);
    assert_eq!(hatched.code, Some(0), "the opt-out aborts the merge: {}", stderr_of(&hatched));
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge is gone");
    remove(&fixture);
}

/// A cherry-pick under the `resolve` strategy records no result either,
///  so a resolution written in the worktree and never staged is refused before `--skip` goes on.
#[test]
fn a_cherry_pick_that_records_no_result_refuses_an_unstaged_resolution() {
    let fixture: Fixture = fixture("skip-pick-resolve-resolution");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    let side: String = plain(&fixture, linked.as_path(), &["rev-parse", "side"]).trim().to_string();
    attempt(&fixture, linked.as_path(), &["cherry-pick", "--strategy=resolve", side.as_str()]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the pick stopped: {during}");
    assert!(!merge_result_recorded(&fixture, linked.as_path()), "the resolve strategy records no result");
    std::fs::write(linked.join("a.txt"), b"HAND RESOLUTION written by the person\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--skip"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the skip would discard the resolution: {said}");
    assert!(said.contains("a.txt (a conflicted path)"), "{said}");
    assert!(said.contains("git cherry-pick --quit or git revert --quit"), "{said}");
    assert!(said.contains("git cherry-pick --no-enforce-worktree --skip"), "{said}");
    assert!(git_path_is_there(&fixture, linked.as_path(), "CHERRY_PICK_HEAD"), "the pick still waits: {said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        b"HAND RESOLUTION written by the person\n",
        "nothing was changed: {said}"
    );
    let hatched: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--no-enforce-worktree", "--skip"]);
    assert_ne!(hatched.code, Some(1), "the opt-out lets the skip go on: {}", stderr_of(&hatched));
    remove(&fixture);
}

/// Where no result is recorded, a conflicted path left exactly as Git wrote it is refused too,
///  and the refusal says why, because nothing the wrapper can read proves it is Git's own text.
///  The advice it prints ends the operation and keeps the files.
#[test]
fn a_conflicted_path_left_as_git_wrote_it_is_refused_where_no_result_is_recorded() {
    let fixture: Fixture = fixture("abort-merge-resolve-untouched");
    let linked: PathBuf = linked_worktree_at_a_conflicted_merge_with(&fixture, Some("resolve"));
    let before: Vec<u8> = std::fs::read(linked.join("a.txt")).expect("the conflicted file");
    assert!(String::from_utf8_lossy(before.as_slice()).contains("<<<<<<<"), "Git wrote conflict markers: {}", String::from_utf8_lossy(before.as_slice()));
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the conflicted path cannot be told from a resolution: {said}");
    assert!(said.contains("a.txt (a conflicted path)"), "{said}");
    assert!(said.contains("Git recorded no result for the conflict it stopped at"), "{said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file"),
        before,
        "nothing was changed: {said}"
    );
    // The advice, run as printed: the quit form ends the merge and keeps the file as it is.
    let quit: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--quit"]);
    let quit_said: String = stderr_of(&quit);
    assert_eq!(quit.code, Some(0), "the quit form the refusal named is allowed: {quit_said}");
    assert!(!git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge is over: {quit_said}");
    assert_eq!(
        std::fs::read(linked.join("a.txt")).expect("the file after the quit"),
        before,
        "the quit keeps the file as it was: {quit_said}"
    );
    remove(&fixture);
}

/// An abort in the main worktree whose only changes are the merge's own runs,
///  which ends the older rule that every abort there is refused.
#[test]
fn an_abort_in_the_main_worktree_runs_when_it_discards_nothing() {
    let fixture: Fixture = fixture("abort-main-worktree-own");
    let repo: PathBuf = main_worktree_at_a_conflicted_merge(&fixture, None);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "the abort discards nothing the person made: {said}");
    assert!(!rejected(&observed), "{said}");
    assert!(!git_path_is_there(&fixture, repo.as_path(), "MERGE_HEAD"), "the merge is gone: {said}");
    assert_eq!(
        plain(&fixture, repo.as_path(), &["status", "--porcelain=v1"]).trim(),
        "",
        "the worktree is back to the state before the merge: {said}"
    );
    assert!(saved_copy_names(common_dir(&fixture, repo.as_path()).as_path()).is_empty(), "nothing was copied");
    remove(&fixture);
}

/// An abort in the main worktree that would discard a hand resolution is refused, naming the file,
///  and the opt-out leaves the state real Git leaves there.
#[test]
fn an_abort_in_the_main_worktree_refuses_a_hand_resolution() {
    let hatched_fixture: Fixture = fixture("abort-main-worktree-resolution");
    let repo: PathBuf = main_worktree_at_a_conflicted_merge(&hatched_fixture, None);
    std::fs::write(repo.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&hatched_fixture, repo.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the abort would discard the resolution: {said}");
    assert!(said.contains("git merge --abort is refused"), "{said}");
    assert!(said.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{said}");
    assert!(!said.contains("main git worktree"), "the older rule's words are gone: {said}");
    assert!(said.contains("git merge --quit"), "{said}");
    assert!(said.contains("git merge --no-enforce-worktree --abort"), "{said}");
    assert!(git_path_is_there(&hatched_fixture, repo.as_path(), "MERGE_HEAD"), "the merge still waits: {said}");
    assert_eq!(
        std::fs::read(repo.join("a.txt")).expect("the file"),
        b"resolved by hand\n",
        "nothing was changed: {said}"
    );
    let hatched: Observed = run_wrapped(&hatched_fixture, repo.as_path(), &["merge", "--no-enforce-worktree", "--abort"]);
    assert_eq!(hatched.code, Some(0), "the opt-out aborts the merge: {}", stderr_of(&hatched));
    assert!(!git_path_is_there(&hatched_fixture, repo.as_path(), "MERGE_HEAD"), "the merge is gone");
    // The same state, built the same way, where real Git runs the abort itself.
    let oracle: Fixture = fixture("abort-main-worktree-resolution-oracle");
    let oracle_repo: PathBuf = main_worktree_at_a_conflicted_merge(&oracle, None);
    std::fs::write(oracle_repo.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let direct: Observed = run_direct(&oracle, oracle_repo.as_path(), &["merge", "--abort"]);
    assert_eq!(direct.code, Some(0), "real Git runs the abort in the same state: {}", stderr_of(&direct));
    assert_eq!(
        state_after(&hatched_fixture, repo.as_path(), &["a.txt"]),
        state_after(&oracle, oracle_repo.as_path(), &["a.txt"]),
        "the opt-out leaves what real Git leaves"
    );
    remove(&hatched_fixture);
    remove(&oracle);
}
