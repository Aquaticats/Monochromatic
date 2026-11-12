//! What:
//!  The abort forms (`git merge --abort`,
//!  `git rebase --abort`,
//!  `git cherry-pick --abort`,
//!  `git revert --abort`,
//!  `git am --abort`)
//!  through the built executable,
//!  while the operation they undo is in progress.
//! Why:
//!  The owner's rule is that nothing the person wrote is discarded,
//!  and that a command the person needs must run.
//!  An operation that stops with conflicts makes `git status` unclean by itself,
//!  so a guard that reads only that status refused every abort form for as long as the
//!  operation waited,
//! which is exactly when a person reaches for one.
//!  Here the abort runs when the only changes are the operation's own,
//!  the person's own changes are saved and named before it runs,
//!  and an abort with nothing in progress is left to Git,
//!  which refuses it without discarding anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merge = await wrapped(['merge', 'topic']);           // CONFLICT
//! expect(await wrapped(['merge', '--abort'])).toExit(0);      // the merge is gone
//! ```

use super::support::{
    Fixture, Observed, TIME_BOUND_SECONDS, bounded_for, direct, executable, fixture, git, remove, repository, run_wrapped, stderr_of,
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

/// The wrapper's saved copies in `common_dir`,
///  by their path below the copy directory,
///  with the bytes each holds.
fn saved_copies(common_dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found: Vec<(String, Vec<u8>)> = Vec::new();
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
            let stamp: String = path
                .parent()
                .and_then(Path::file_name)
                .map(|name| { return name.to_string_lossy().into_owned(); })
                .unwrap_or_default();
            let name: String = path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default();
            let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
            found.push((format!("{stamp}/{name}"), bytes));
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
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
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

/// The merge of `side` is the only thing `git status` reports,
///  and the abort runs,
///  saves nothing,
///  and leaves the state before the merge.
#[test]
fn a_merge_abort_runs_when_the_merge_s_own_changes_are_all_there_is() {
    let fixture: Fixture = fixture("abort-merge-own");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "a conflicted merge is in progress: {during}");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    let after: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    let state: String = plain(&fixture, linked.as_path(), &["rev-parse", "--git-path", "MERGE_HEAD"]);
    let merge_head: PathBuf = PathBuf::from(state.trim());
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    let contents: String = std::fs::read_to_string(linked.join("a.txt")).unwrap_or_default();
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert_eq!(after.trim(), "", "the worktree is back to the state before the merge: {said}");
    assert!(!merge_head.exists(), "the merge state is gone: {said}");
    assert_eq!(contents, "main\n", "the file holds the branch's own content again: {said}");
    assert!(copies.is_empty(), "the merge's own state needs no copy: {copies:?}");
    let common: PathBuf = common_dir(&fixture, linked.as_path());
    let audit: Vec<String> = audit_lines(common.as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"allowed\""), "{last}");
    assert!(last.contains("\"form\":\"AbortMerge\""), "{last}");
    assert!(last.contains("\"dirty\":null"), "no status was read: {last}");
    remove(&fixture);
}

/// A conflict resolved by hand is the person's work,
///  so the abort runs only after the file is saved,
///  and the notice names where.
#[test]
fn a_merge_abort_saves_a_hand_resolution_before_it_runs() {
    let fixture: Fixture = fixture("abort-merge-hand");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert_eq!(copies.len(), 1, "one copy of the one edited file: {copies:?} ({said})");
    assert_eq!(copies[0].0.rsplit('/').next(), Some("a.txt"), "{copies:?}");
    assert_eq!(copies[0].1, b"resolved by hand\n", "the copy holds the resolution");
    assert!(said.contains("a.txt"), "the notice names the file: {said}");
    let common: PathBuf = common_dir(&fixture, linked.as_path());
    let audit: Vec<String> = audit_lines(common.as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"saved-abort\""), "{last}");
    assert!(last.contains("\"form\":\"AbortMerge\""), "{last}");
    assert!(last.contains("\"dirty\":null"), "no status was read: {last}");
    assert!(
        last.contains("a.txt => ") && last.contains("discarded/"),
        "the record names where the copy went: {last}"
    );
    remove(&fixture);
}

/// A staged edit of a path the merge never touched is discarded by Git's own abort,
///  so it is saved too.
#[test]
fn a_merge_abort_saves_a_staged_edit_of_another_path_before_it_runs() {
    let fixture: Fixture = fixture("abort-merge-staged");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    std::fs::write(linked.join("b.txt"), b"another session's work\n").expect("the staged edit");
    git(&fixture, linked.as_path(), &["add", "--", "b.txt"]);
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    let saved: Option<&Vec<u8>> = copies.iter().find(|(name, _)| { return name.ends_with("b.txt"); }).map(|(_, bytes)| { return bytes; });
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(saved, Some(&b"another session's work\n".to_vec()), "the staged edit is saved: {copies:?} ({said})");
    assert!(said.contains("b.txt"), "the notice names the file: {said}");
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
    assert!(saved_copies(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "nothing to save");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let left: Vec<&String> = audit.iter().rev().take(5).collect();
    let forms: Vec<String> = [
        "AbortAm",
        "AbortPick",
        "AbortPick",
        "AbortRebase",
        "AbortMerge",
    ]
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
///  and the hand edit it would discard is saved first.
#[test]
fn a_rebase_abort_runs_and_saves_the_person_s_edit() {
    let fixture: Fixture = fixture("abort-rebase");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(!during.trim().is_empty(), "the rebase stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand edit is saved: {copies:?} ({said})"
    );
    assert_eq!(
        plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(),
        "",
        "the rebase is undone: {said}"
    );
    remove(&fixture);
}

/// A cherry-pick that stopped at a conflict is undone by its own abort.
#[test]
fn a_cherry_pick_abort_runs_during_a_conflicted_pick() {
    let fixture: Fixture = fixture("abort-pick");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    let side: String = plain(&fixture, linked.as_path(), &["rev-parse", "side"]).trim().to_string();
    attempt(&fixture, linked.as_path(), &["cherry-pick", side.as_str()]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the pick stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand edit is saved: {copies:?} ({said})"
    );
    assert_eq!(plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(), "", "the pick is undone: {said}");
    remove(&fixture);
}

/// An `am` that stopped at a conflicting patch is undone by its own abort,
///  and the resolution it would discard is saved first.
/// A patch that cannot be applied at all is not this control's case:
///  `git am` then leaves the index and the worktree alone,
///  and its abort keeps the edits it finds (measured with Git 2.56.0),
///  so there is nothing for the wrapper to save.
#[test]
fn an_am_abort_runs_during_a_conflicted_patch() {
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
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand edit is saved: {copies:?} ({said})"
    );
    assert_eq!(plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(), "", "the am is undone: {said}");
    assert_eq!(std::fs::read(linked.join("a.txt")).expect("the file after the abort"), before, "the file holds what stood before the am: {said}");
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

/// A rebase that replays a merge keeps the hand resolution when `git rebase --abort` ends it:
///  the rebase's own state is what makes the abort live,
///  even beside the `MERGE_HEAD` of the merge step it stopped at.
#[test]
fn a_rebase_abort_at_a_rebase_merges_merge_step_saves_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-rebase-merges-step");
    let linked: PathBuf = linked_worktree_whose_rebase_recreates_a_merge(&fixture, false);
    attempt(&fixture, linked.as_path(), &["rebase", "--rebase-merges", "main"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge step waits");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase waits");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand resolution is saved before the rebase goes: {copies:?} ({said})"
    );
    assert!(said.contains("a.txt"), "the notice names the file: {said}");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"form\":\"AbortRebase\""), "{last}");
    assert!(last.contains("\"outcome\":\"saved-abort\""), "{last}");
    assert_eq!(plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(), "", "the rebase is undone: {said}");
    remove(&fixture);
}

/// `git am --abort` ends a rebase that applies patches,
///  so the hand resolution it would discard is saved first,
///  and the notice names the rebase it ends.
#[test]
fn an_am_abort_during_an_apply_backend_rebase_saves_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-am-ends-rebase");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "--apply", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-apply/rebasing"), "the rebase waits");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["am", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand resolution is saved before the rebase goes: {copies:?} ({said})"
    );
    assert!(said.contains("git am --abort"), "the notice names the command as typed: {said}");
    assert!(said.contains("rebase in progress"), "the notice names the operation it ends: {said}");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"form\":\"AbortAm\""), "{last}");
    assert!(last.contains("\"outcome\":\"saved-abort\""), "{last}");
    remove(&fixture);
}

/// The tree a pick sequence restores is the one its own sequencer recorded,
///  not whatever `ORIG_HEAD` last held:
///  an ignored file the restored tree tracks keeps the refusal and its content.
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
    assert!(saved_copies(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "nothing was saved because nothing ran");
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

/// Content that stands only in the index is not left to the abort:
///  the worktree file was deleted after it was staged,
///  so no copy can be made from the worktree,
///  and the abort would drop the only reference to it.
///  The command is refused naming the path, and the index still holds it.
#[test]
fn a_merge_abort_refuses_content_that_stands_only_in_the_index() {
    let fixture: Fixture = fixture("abort-merge-index-only");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    std::fs::write(linked.join("new.txt"), b"content that exists only in the index\n").expect("the staged file");
    git(&fixture, linked.as_path(), &["add", "--", "new.txt"]);
    let staged: String = plain(&fixture, linked.as_path(), &["rev-parse", ":new.txt"]).trim().to_string();
    assert_eq!(staged.len(), 40, "the index holds the new file's bytes: {staged}");
    std::fs::remove_file(linked.join("new.txt")).expect("the worktree file goes");
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]);
    assert!(during.contains("AD new.txt"), "the index added it and the worktree no longer holds it: {during}");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "content only the index holds keeps the refusal: {said}");
    assert!(said.contains("new.txt"), "the refusal names the path: {said}");
    assert!(said.contains("the index does"), "the refusal says where the content stands: {said}");
    let after: String = plain(&fixture, linked.as_path(), &["rev-parse", ":new.txt"]).trim().to_string();
    assert_eq!(after, staged, "the index still holds the content: {said}");
    assert!(saved_copies(common_dir(&fixture, linked.as_path()).as_path()).is_empty(), "the abort did not run");
    remove(&fixture);
}

/// An untracked file at a path the restored tree tracks stops Git's own reset,
///  so the wrapper refuses the abort naming it,
///  saves nothing and changes nothing,
///  and says that Git will not restore the tree while the file is there.
///  Measured with Git 2.56.0:
///  every abort form refuses such a reset and loses nothing,
///  and no copy of the untracked file is owed.
#[test]
fn a_rebase_abort_names_an_untracked_file_that_stops_git_s_own_reset() {
    let fixture: Fixture = fixture("abort-rebase-untracked");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"base\n").expect("tracked file");
    std::fs::write(repo.join("victim.txt"), b"original tracked victim\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt", "victim.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "onto"]);
    std::fs::write(repo.join("a.txt"), b"onto\n").expect("onto edit");
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--", "victim.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=onto"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), b"main\n").expect("main edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let linked: PathBuf = fixture.root.join("work");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "work", "../work", "main"]);
    attempt(&fixture, linked.as_path(), &["rebase", "onto"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase waits");
    assert!(!linked.join("victim.txt").exists(), "the rebase took the file away, so a new one is untracked");
    std::fs::write(linked.join("victim.txt"), b"untracked work created during the rebase\n").expect("the untracked file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    assert!(rejected(&observed), "the untracked file keeps the refusal: {said}");
    assert!(said.contains("victim.txt"), "the refusal names the file: {said}");
    assert!(said.contains("untracked"), "the refusal says the file is untracked: {said}");
    assert!(
        said.contains("Git will not restore that tree while it is there"),
        "the refusal says Git will not abort while the file is there: {said}"
    );
    assert!(said.contains("mv <path>"), "the refusal says what moves the person forward: {said}");
    assert_eq!(
        std::fs::read(linked.join("victim.txt")).expect("the untracked file"),
        b"untracked work created during the rebase\n",
        "nothing was written over the untracked file: {said}"
    );
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase still waits: {said}");
    assert!(
        saved_copies(common_dir(&fixture, linked.as_path()).as_path()).is_empty(),
        "no copy is announced that Git never needed: {said}"
    );
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"rejected\""), "{last}");
    assert!(last.contains("\"form\":\"AbortRebase\""), "{last}");
    remove(&fixture);
}

/// A skip drops the hand resolution of the conflict it stops at,
///  so the wrapper copies it first and lets the skip go on,
///  in a linked worktree and in the main one alike:
///  the copy is the protection, and a refusal is only for the case where no copy can be made.
#[test]
fn a_rebase_skip_saves_the_hand_resolution_before_it_goes_on() {
    let fixture: Fixture = fixture("skip-rebase-saves");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    let during: String = plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]);
    assert!(during.contains("UU a.txt"), "the rebase stopped: {during}");
    std::fs::write(linked.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--skip"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand resolution is saved before the skip goes on: {copies:?} ({said})"
    );
    assert!(said.contains("git rebase --skip"), "the notice names the command as typed: {said}");
    assert!(said.contains("goes on with the next commit"), "the notice says the skip continues: {said}");
    assert!(!said.contains("restores the state before"), "a skip does not restore the state before: {said}");
    let audit: Vec<String> = audit_lines(common_dir(&fixture, linked.as_path()).as_path());
    let last: &String = audit.last().expect("one audit line");
    assert!(last.contains("\"outcome\":\"saved-skip\""), "{last}");
    assert!(last.contains("\"form\":\"SkipRebase\""), "{last}");
    assert_eq!(plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(), "", "the skip went on: {said}");
    remove(&fixture);
}

/// A skip that a copy makes safe runs in the main worktree too,
///  where every overwriting form is otherwise refused.
#[test]
fn a_skip_runs_in_the_main_worktree_after_its_copy() {
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
    std::fs::write(repo.join("a.txt"), b"resolved by hand\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["rebase", "--skip"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, repo.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "a copy makes the skip safe here: {said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolved by hand\n"; }),
        "the hand resolution is saved: {copies:?} ({said})"
    );
    remove(&fixture);
}

/// Content that stands only in the index is copied from the index and named for its stage,
///  so the abort goes on instead of refusing a command a copy makes safe.
#[test]
fn a_merge_abort_saves_content_that_stands_only_in_the_index() {
    let fixture: Fixture = fixture("abort-merge-index-only-saved");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    std::fs::write(linked.join("new.txt"), b"content that exists only in the index\n").expect("the staged file");
    git(&fixture, linked.as_path(), &["add", "--", "new.txt"]);
    std::fs::remove_file(linked.join("new.txt")).expect("the worktree file goes");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["merge", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "the abort runs once the content is copied: {said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.contains("index-stage-0/new.txt") && bytes == b"content that exists only in the index\n"; }),
        "the index's content is copied beside the worktree copies: {copies:?} ({said})"
    );
    assert!(said.contains("new.txt (index-stage-0)"), "the notice names the path and the stage: {said}");
    assert_eq!(plain(&fixture, linked.as_path(), &["status", "--porcelain=v1"]).trim(), "", "the merge is undone: {said}");
    remove(&fixture);
}

/// What:
///  A stand-in for real Git that pauses once the wrapper has copied a file,
///  so a control can write to that file inside the window between the copy and Git.
/// Why:
///   The window is what the last comparison exists for,
///  and nothing else makes it deterministically reachable:
///  the pause is a wrapper-owned Git start that happens after the copy and before the command.
///  Every other invocation is real Git, unchanged.
fn pausing_real_git(fixture: &Fixture, saved_root: &Path, paused: &Path, release: &Path) -> PathBuf {
    let directory: PathBuf = fixture.root.join("shim");
    std::fs::create_dir(directory.as_path()).expect("shim directory");
    let shim: PathBuf = directory.join("git");
    executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\n\
             if [ ! -f '{done}' ] && [ -d '{saved}' ] && [ -n \"$(ls -A '{saved}' 2>/dev/null)\" ]; then\n\
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
            saved = saved_root.display(),
            paused = paused.display(),
            release = release.display()
        )
        .as_bytes(),
    );
    return shim;
}

/// The wrapper of `fixture`, with the pausing stand-in for real Git and no `/usr/bin` before it.
fn wrapper_paused_at_real_git(fixture: &Fixture, shim: &Path) -> Command {
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(shim.parent().expect("the shim's directory").as_os_str());
    path.push(":/bin");
    return bounded_for(fixture, fixture.root.join("bin/git").as_path(), path.as_os_str(), TIME_BOUND_SECONDS);
}

/// An edit written after the wrapper copied the file and before Git discards it is copied again,
///  immediately before Git starts,
///  and the notice names the second copy.
#[test]
fn an_edit_written_after_the_copy_is_saved_before_git_starts() {
    let fixture: Fixture = fixture("abort-edit-after-copy");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    std::fs::write(linked.join("a.txt"), b"resolution present when the wrapper copied the file\n").expect("the first resolution");
    let common: PathBuf = common_dir(&fixture, linked.as_path());
    let saved_root: PathBuf = common.join("monochromatic-git-policy-cli/discarded");
    let paused: PathBuf = fixture.root.join("shim-paused");
    let release: PathBuf = fixture.root.join("shim-release");
    let shim: PathBuf = pausing_real_git(&fixture, saved_root.as_path(), paused.as_path(), release.as_path());
    let mut command: Command = wrapper_paused_at_real_git(&fixture, shim.as_path());
    let mut child: std::process::Child = command
        .current_dir(linked.as_path())
        .args(["merge", "--abort"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the wrapper starts");
    // A bounded wait for the wrapper's own Git start inside the window, so nothing here waits forever.
    let mut waited: std::time::Duration = std::time::Duration::ZERO;
    while !paused.exists() && waited < std::time::Duration::from_secs(4) {
        std::thread::sleep(std::time::Duration::from_millis(10));
        waited += std::time::Duration::from_millis(10);
    }
    let reached: bool = paused.exists();
    if !reached {
        let _killed = child.kill();
    }
    assert!(reached, "the wrapper reached a Git start after copying the file");
    let at_pause: Vec<(String, Vec<u8>)> = saved_copies(common.as_path());
    assert!(
        at_pause.iter().any(|(_, bytes)| { return bytes == b"resolution present when the wrapper copied the file\n"; }),
        "the first copy is on disk at the pause: {at_pause:?}"
    );
    // The edit the wrapper cannot see when it copies, written while it is paused.
    std::fs::write(linked.join("a.txt"), b"resolution written after the wrapper copied the file\n").expect("the later resolution");
    std::fs::write(release.as_path(), b"go").expect("the release");
    let output: std::process::Output = child.wait_with_output().expect("the wrapper finishes");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common.as_path());
    assert_eq!(output.status.code(), Some(0), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"resolution present when the wrapper copied the file\n"; }),
        "the first copy is kept: {copies:?} ({said})"
    );
    assert!(
        copies
            .iter()
            .any(|(name, bytes)| { return name.contains("later-") && name.ends_with("a.txt") && bytes == b"resolution written after the wrapper copied the file\n"; }),
        "the edit written after the copy is saved beside it: {copies:?} ({said})"
    );
    assert!(said.contains("changed again after the wrapper copied"), "the notice names the second copy: {said}");
    remove(&fixture);
}

/// A resolution staged and then edited again leaves two things the person made,
///  and the abort drops the staged one whatever the worktree holds,
///  so both are copied: the worktree file as it stands and the index's own content.
#[test]
fn a_rebase_abort_saves_a_staged_resolution_beside_the_later_worktree_edit() {
    let fixture: Fixture = fixture("abort-rebase-staged-and-edited");
    let linked: PathBuf = linked_worktree_with_a_diverged_side(&fixture);
    attempt(&fixture, linked.as_path(), &["rebase", "side"]);
    std::fs::write(linked.join("a.txt"), b"resolution staged first\n").expect("the staged resolution");
    git(&fixture, linked.as_path(), &["add", "--", "a.txt"]);
    std::fs::write(linked.join("a.txt"), b"resolution edited after staging\n").expect("the later edit");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!rejected(&observed), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("/a.txt") && bytes == b"resolution edited after staging\n"; }),
        "the later worktree edit is saved: {copies:?} ({said})"
    );
    assert!(
        copies
            .iter()
            .any(|(name, bytes)| { return name.contains("index-stage-0/a.txt") && bytes == b"resolution staged first\n"; }),
        "the staged resolution is saved from the index: {copies:?} ({said})"
    );
    assert!(said.contains("a.txt (index-stage-0)"), "the notice names the staged copy and its stage: {said}");
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
    if local_env {
        std::fs::write(linked.join("local.env"), b"tracked\n").expect("the tracked local.env");
        git(fixture, linked.as_path(), &["add", "--", "local.env"]);
        git(fixture, linked.as_path(), &["commit", "--quiet", "--message=before-the-steps-adds-local.env"]);
    }
    let mut first: Command = direct(fixture);
    first.current_dir(linked.as_path()).args(["commit", "--quiet", "--allow-empty", "--message=T0"]);
    let _made = first.output().expect("native Git starts");
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
///  and `git rebase --abort` is a word Git acts on: the hand resolution and the unstaged edit are copied first.
#[test]
fn a_rebase_abort_inside_an_edit_step_saves_the_hand_resolution_and_the_unstaged_edit() {
    let fixture: Fixture = fixture("abort-rebase-edit-step-merge");
    let linked: PathBuf = worktree_stopped_at_an_edit_step(&fixture, false);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the edit step waits");
    attempt(&fixture, linked.as_path(), &["merge", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "MERGE_HEAD"), "the merge waits beside the rebase");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "and the rebase still waits");
    std::fs::write(linked.join("a.txt"), b"hand resolution of the merge\n").expect("the resolution");
    std::fs::write(linked.join("other.txt"), b"an edit outside the merge\n").expect("the unstaged edit");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rebase", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"hand resolution of the merge\n"; }),
        "the hand resolution is saved: {copies:?} ({said})"
    );
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("other.txt") && bytes == b"an edit outside the merge\n"; }),
        "the unstaged edit is saved: {copies:?} ({said})"
    );
    assert!(!git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the rebase is gone: {said}");
    remove(&fixture);
}

/// `git cherry-pick --abort` is live while a cherry-pick waits inside a rebase's edit step,
///  and the work it discards is copied first.
#[test]
fn a_cherry_pick_abort_inside_an_edit_step_saves_the_hand_resolution() {
    let fixture: Fixture = fixture("abort-pick-inside-edit-step");
    let linked: PathBuf = worktree_stopped_at_an_edit_step(&fixture, false);
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "the edit step waits");
    attempt(&fixture, linked.as_path(), &["cherry-pick", "side"]);
    assert!(git_path_is_there(&fixture, linked.as_path(), "CHERRY_PICK_HEAD"), "the pick waits beside the rebase");
    assert!(git_path_is_there(&fixture, linked.as_path(), "rebase-merge"), "and the rebase still waits");
    std::fs::write(linked.join("a.txt"), b"hand resolution of the pick\n").expect("the resolution");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["cherry-pick", "--abort"]);
    let said: String = stderr_of(&observed);
    let copies: Vec<(String, Vec<u8>)> = saved_copies(common_dir(&fixture, linked.as_path()).as_path());
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(
        copies.iter().any(|(name, bytes)| { return name.ends_with("a.txt") && bytes == b"hand resolution of the pick\n"; }),
        "the hand resolution is saved: {copies:?} ({said})"
    );
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
    let record: &str = record.trim();
    let tracks: String = plain(&fixture, linked.as_path(), &["ls-tree", record, "local.env"]);
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
