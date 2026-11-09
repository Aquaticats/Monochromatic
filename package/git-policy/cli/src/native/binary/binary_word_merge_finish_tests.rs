//! What:
//!  The finish after the word-level second pass,
//! through the built executable:
//! when every conflicted file of a merge,
//! rebase,
//!  cherry-pick,
//! revert or pull resolves,
//! the operation ends the way Git ends it after a clean run,
//! and the options that stop a
//!  clean run (`--no-commit`,
//! `--squash`,
//! `--edit`) stop it here too.
//! Why:
//!  The owner answered on 2026-11-03:
//! "Finish like a clean merge (Recommended)".
//! The continuing step goes through the wrapper's
//!  own command path,
//! so a later pick of a rebase gets the second pass and the commit gets the policies and the hooks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = await wrapped(['merge', '--no-edit', 'topic']); // status 0, a merge commit, nothing left unmerged
//! ```

use super::commit::wrapped_commit;
use super::support::{Fixture, Observed, executable, fixture, git, observe, remove, repository, run_direct, run_wrapped, stderr_of, wrapped};
use super::word_merge::{ADJACENT, Stop, STOPS, arguments_of, corpus, expected_after, merge_lines, prepared, tracked, unmerged};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The corpus input whose two sides edit a list item and add an item after it:
/// Git conflicts,
/// the word pass resolves.
const SECOND_ADJACENT: &str = "md1c";

/// The state files and directories an unfinished operation leaves in the Git directory.
const STATE: [&str; 6] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply", "sequencer"];

/// The state of an unfinished operation that is present.
fn state_left(repo: &Path) -> Vec<&'static str> {
    return STATE.iter().copied().filter(|name| { return repo.join(".git").join(name).exists(); }).collect();
}

/// A query against the repository,
/// as trimmed text.
fn text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, arguments).stdout).trim().to_owned();
}

/// The porcelain status,
/// which is empty when nothing is staged,
/// modified or untracked.
fn status(fixture: &Fixture, repo: &Path) -> String {
    return text(fixture, repo, &["status", "--porcelain=v1", "--untracked-files=all"]);
}

/// An editor that appends a line,
/// so the message it was handed shows in the commit.
fn appending_editor(fixture: &Fixture) -> PathBuf {
    let path: PathBuf = fixture.root.join("appending-editor");
    executable(path.as_path(), b"#!/bin/sh\nprintf 'edited by the editor\\n' >> \"$1\"\n");
    return path;
}

/// Run the arguments with an editor named.
fn run_with_editor(fixture: &Fixture, repo: &Path, editor: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).env("GIT_EDITOR", editor).current_dir(repo).args(arguments), b"");
}

/// Each of the four operations ends like a clean run:
/// status 0,
/// the commit,
/// nothing left unmerged or unfinished,
/// and the part line.
#[test]
fn each_operation_is_finished_like_a_clean_run() {
    for stop in STOPS {
        let fixture: Fixture = fixture(format!("finish-{stop:?}").to_lowercase().as_str());
        let repo: PathBuf = prepared(&fixture, stop, &[ADJACENT]);
        let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(stop).as_slice());
        let said: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(0), "{stop:?}: {said}");
        assert_eq!(unmerged(&fixture, repo.as_path()), Vec::<String>::new(), "{stop:?}: {said}");
        assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "{stop:?}: the operation is over: {said}");
        assert_eq!(status(&fixture, repo.as_path()), "", "{stop:?}: nothing is left staged or modified: {said}");
        assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == expected_after(stop, ADJACENT), "{stop:?}: {said}");
        assert_ne!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before, "{stop:?}: a commit was made: {said}");
        assert!(merge_lines(&observed).iter().any(|line| { return line.contains(tracked(ADJACENT).as_str()); }), "{stop:?}: {said}");
        let subject: String = text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]);
        match stop {
            Stop::Merge => {
                assert_eq!(text(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", "HEAD"]).split_whitespace().count(), 3, "a merge commit");
                assert_eq!(subject, "Merge branch 'topic'");
            }
            Stop::Rebase => assert_eq!(subject, "theirs"),
            Stop::CherryPick => assert_eq!(subject, "theirs"),
            Stop::Revert => assert!(subject.starts_with("Revert "), "{subject}"),
        }
        let message: String = text(&fixture, repo.as_path(), &["log", "-1", "--format=%B"]);
        assert!(!message.contains("Conflicts") && !message.contains('#'), "{stop:?}: the conflict hint is not in the message: {message}");
        remove(&fixture);
    }
}

/// What:
///  A finish whose commit cannot reach the commit service says what the second pass did:
/// the conflict is gone,
/// the files are staged as resolved,
/// and only the step that ends the operation was not made.
/// Why:
///   The commit's own failure text is true of the commit alone
/// (it ends with `Nothing in the repository changed.`),
/// while the second pass changed the index
///  and the worktree before that commit started,
/// so the last word must name the state the person will find.
#[test]
fn a_finish_that_cannot_reach_the_commit_service_names_what_changed() {
    let fixture: Fixture = fixture("finish-no-service");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let head: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    // No runtime directory and no session bus: this account has no way to start the commit service.
    let observed: Observed = observe(
        wrapped(&fixture).current_dir(repo.as_path()).env_remove("XDG_RUNTIME_DIR").env_remove("DBUS_SESSION_BUS_ADDRESS").args(arguments_of(Stop::Merge).as_slice()),
        b"",
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(1), "{said}");
    assert!(said.contains("this commit was not made"), "{said}");
    assert!(said.contains("staged as resolved by the second merge pass"), "the pass says what it staged: {said}");
    assert_eq!(status(&fixture, repo.as_path()), format!("M  {}", tracked(ADJACENT)), "the resolved file is staged: {said}");
    assert_eq!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file"), corpus(ADJACENT, "word.out"), "the merged text is in the worktree: {said}");
    assert_eq!(unmerged(&fixture, repo.as_path()), Vec::<String>::new(), "the conflict is gone: {said}");
    assert_eq!(state_left(repo.as_path()), vec!["MERGE_HEAD"], "the merge is still in progress: {said}");
    assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head, "no commit was made: {said}");
    let last: &str = said.lines().last().expect("a last line");
    assert!(last.contains("did not finish"), "the last word names the operation: {last}");
    assert!(last.contains("staged as resolved") && last.contains("git commit"), "the last word names the state and the way on: {last}");
    remove(&fixture);
}

/// A file that does not resolve keeps today's stop:
/// nothing is committed and Git's status stays.
/// (A revert has no such input in the
///  corpus:
/// its later state is the resolved text of the input.)
#[test]
fn a_part_that_does_not_resolve_keeps_the_stop_for_every_operation() {
    for stop in STOPS.into_iter().filter(|stop| { return !matches!(stop, Stop::Revert); }) {
        let fixture: Fixture = fixture(format!("finish-stays-{stop:?}").to_lowercase().as_str());
        let repo: PathBuf = prepared(&fixture, stop, &[ADJACENT, "md2"]);
        let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(stop).as_slice());
        let said: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(1), "{stop:?}: {said}");
        assert_eq!(unmerged(&fixture, repo.as_path()), vec![tracked("md2")], "{stop:?}: {said}");
        // A stopped rebase has HEAD detached at the new base, so the commit count is what shows that nothing was made.
        if !matches!(stop, Stop::Rebase) {
            assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before, "{stop:?}: nothing is committed: {said}");
        }
        assert!(!state_left(repo.as_path()).is_empty(), "{stop:?}: the operation is still in progress");
        remove(&fixture);
    }
}

/// `git merge --no-commit` stops as a clean one does:
/// status 0,
/// Git's line,
/// the merge state kept,
/// nothing committed.
#[test]
fn merge_no_commit_stops_after_the_resolution() {
    let fixture: Fixture = fixture("finish-merge-no-commit");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-commit", "--no-edit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(said.contains("Automatic merge went well; stopped before committing as requested"), "{said}");
    assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before);
    assert_eq!(state_left(repo.as_path()), vec!["MERGE_HEAD"]);
    assert_eq!(unmerged(&fixture, repo.as_path()), Vec::<String>::new());
    let message: String = std::fs::read_to_string(repo.join(".git/MERGE_MSG")).expect("the prepared message");
    assert!(!message.contains("Conflicts"), "the message waits for the person's commit without the hint: {message}");
    assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == corpus(ADJACENT, "word.out"));
    remove(&fixture);
}

/// `git merge --squash` stops as a clean one does:
/// status 0,
/// Git's line,
/// no merge state,
/// no commit,
/// the result staged.
#[test]
fn merge_squash_stops_after_the_resolution() {
    let fixture: Fixture = fixture("finish-merge-squash");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--squash", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(said.contains("Automatic merge went well; stopped before committing as requested"), "{said}");
    assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before);
    assert_eq!(state_left(repo.as_path()), Vec::<&str>::new());
    assert!(repo.join(".git/SQUASH_MSG").exists());
    assert_eq!(status(&fixture, repo.as_path()), format!("M  {}", tracked(ADJACENT)));
    remove(&fixture);
}

/// `--edit` hands the prepared message to the editor and commits what it leaves;
/// `-m` is the message of a merge with no editor.
#[test]
fn merge_edit_runs_the_editor_and_merge_message_is_kept() {
    let first: Fixture = fixture("finish-merge-edit");
    let repo: PathBuf = prepared(&first, Stop::Merge, &[ADJACENT]);
    let editor: PathBuf = appending_editor(&first);
    let observed: Observed = run_with_editor(&first, repo.as_path(), editor.as_path(), &["merge", "--edit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let message: String = text(&first, repo.as_path(), &["log", "-1", "--format=%B"]);
    assert!(message.starts_with("Merge branch 'topic'") && message.contains("edited by the editor"), "{message}");
    assert!(!message.contains("Conflicts"), "{message}");
    remove(&first);
    let second: Fixture = fixture("finish-merge-message");
    let again: PathBuf = prepared(&second, Stop::Merge, &[ADJACENT]);
    let named: Observed = run_wrapped(&second, again.as_path(), &["merge", "-m", "a message of my own", "topic"]);
    assert_eq!(named.code, Some(0), "{}", stderr_of(&named));
    assert_eq!(text(&second, again.as_path(), &["log", "-1", "--format=%B"]), "a message of my own");
    remove(&second);
}

/// A rebase whose second pick also conflicts:
/// the continuing step runs through the wrapper,
/// so the later pick gets the pass too.
#[test]
fn a_rebase_with_two_conflicting_picks_is_resolved_and_finished() {
    let fixture: Fixture = fixture("finish-rebase-two");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let names: [&str; 2] = [ADJACENT, SECOND_ADJACENT];
    let write = |extension: &str, name: &str| { std::fs::write(repo.join(tracked(name)), corpus(name, extension)).expect("an input"); };
    for name in names {
        write("base", name);
    }
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    for name in names {
        write("theirs", name);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", format!("--message=theirs {name}").as_str()]);
    }
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    for name in names {
        write("ours", name);
    }
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=ours"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["rebase", "main"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "{said}");
    assert_eq!(text(&fixture, repo.as_path(), &["log", "--format=%s", "main..HEAD"]), format!("theirs {SECOND_ADJACENT}\ntheirs {ADJACENT}"), "{said}");
    for name in names {
        assert!(std::fs::read(repo.join(tracked(name))).expect("the file") == corpus(name, "word.out"), "{name}");
    }
    assert!(merge_lines(&observed).iter().any(|line| { return line.contains(tracked(ADJACENT).as_str()); }), "{said}");
    assert!(merge_lines(&observed).iter().any(|line| { return line.contains(tracked(SECOND_ADJACENT).as_str()); }), "the later pick got the pass: {said}");
    remove(&fixture);
}

/// `git cherry-pick --no-commit` stops as a clean one does:
/// status 0,
/// the result staged,
/// no commit,
/// no pick state.
#[test]
fn cherry_pick_no_commit_stops_after_the_resolution() {
    let fixture: Fixture = fixture("finish-pick-no-commit");
    let repo: PathBuf = prepared(&fixture, Stop::CherryPick, &[ADJACENT]);
    let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["cherry-pick", "--no-commit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before);
    assert_eq!(status(&fixture, repo.as_path()), format!("M  {}", tracked(ADJACENT)));
    assert_eq!(state_left(repo.as_path()), Vec::<&str>::new());
    remove(&fixture);
}

/// `--edit` of a cherry-pick and of a revert hands the original message to the editor.
#[test]
fn cherry_pick_and_revert_edit_run_the_editor() {
    for (stop, arguments) in [(Stop::CherryPick, vec!["cherry-pick", "--edit", "topic"]), (Stop::Revert, vec!["revert", "--edit", "HEAD~1"])] {
        let fixture: Fixture = fixture(format!("finish-edit-{stop:?}").to_lowercase().as_str());
        let repo: PathBuf = prepared(&fixture, stop, &[ADJACENT]);
        let editor: PathBuf = appending_editor(&fixture);
        let observed: Observed = run_with_editor(&fixture, repo.as_path(), editor.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(0), "{stop:?}: {said}");
        let message: String = text(&fixture, repo.as_path(), &["log", "-1", "--format=%B"]);
        assert!(message.contains("edited by the editor") && !message.contains("Conflicts"), "{stop:?}: {message}");
        remove(&fixture);
    }
}

/// A range whose first pick conflicts:
/// the first is committed and the rest of the sequence is picked,
/// so both land.
#[test]
fn a_cherry_pick_range_is_resolved_and_continued() {
    let fixture: Fixture = fixture("finish-pick-range");
    let repo: PathBuf = prepared(&fixture, Stop::CherryPick, &[ADJACENT]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
    std::fs::write(repo.join("second.txt"), b"a second commit\n").expect("the file");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["cherry-pick", "main..topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "{said}");
    assert_eq!(text(&fixture, repo.as_path(), &["log", "--format=%s", "-2"]), "second\ntheirs", "{said}");
    assert_eq!(status(&fixture, repo.as_path()), "");
    remove(&fixture);
}

/// A pull's merge,
/// its merge with `--no-commit`,
/// and its rebase end like the plain commands do.
#[test]
fn a_pull_is_finished_like_a_clean_one() {
    for (label, option, expect_commit) in [("merge", "--no-rebase", true), ("no-commit", "--no-commit", false), ("rebase", "--rebase", true)] {
        let fixture: Fixture = fixture(format!("finish-pull-{label}").as_str());
        let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
        let remote: PathBuf = fixture.root.join("remote.git");
        git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
        git(&fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
        git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "topic:main"]);
        git(&fixture, repo.as_path(), &["fetch", "--quiet", "origin"]);
        git(&fixture, repo.as_path(), &["branch", "--set-upstream-to=origin/main", "main"]);
        let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let arguments: Vec<&str> = if label == "no-commit" { vec!["pull", "--no-rebase", option, "--no-edit"] } else if label == "merge" { vec!["pull", option, "--no-edit"] } else { vec!["pull", option] };
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(0), "{label}: {said}");
        assert_eq!(unmerged(&fixture, repo.as_path()), Vec::<String>::new(), "{label}: {said}");
        let moved: bool = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]) != before;
        assert_eq!(moved, expect_commit, "{label}: {said}");
        assert_eq!(state_left(repo.as_path()).is_empty(), expect_commit, "{label}: {said}");
        assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == corpus(ADJACENT, "word.out"), "{label}: {said}");
        assert!(merge_lines(&observed).iter().any(|line| { return line.contains(tracked(ADJACENT).as_str()); }), "{label}: {said}");
        remove(&fixture);
    }
}

/// The commit that ends the merge goes through the wrapper's command path:
/// a policy of the repository judges it,
/// and a rejected
///  commit leaves the merge in progress with the files staged.
#[test]
fn the_ending_commit_is_judged_by_the_repositorys_policies() {
    let fixture: Fixture = fixture("finish-policy");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    // Text only the other side's insertion holds, so the merged file carries something new against the first parent.
    std::fs::create_dir(repo.join("rules")).expect("rules directory");
    std::fs::write(repo.join("rules/private.txt"), b"Exit code is zero on success.\n").expect("rules");
    std::fs::write(
        repo.join("monochromatic-git-policy-cli.config.jsonc"),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false, \"rulesFile\": \"rules/private.txt\" }] } }\n",
    )
    .expect("configuration");
    let before: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_ne!(observed.code, Some(0), "the policy rejected the ending commit: {said}");
    assert!(said.contains("forbidden-strings"), "{said}");
    assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before, "{said}");
    assert_eq!(state_left(repo.as_path()), vec!["MERGE_HEAD"], "{said}");
    remove(&fixture);
}

/// A tracked file with an unstaged change,
/// committed on the branch that is checked out:
/// the work an autostash carries.
fn with_unstaged_work(fixture: &Fixture, repo: &Path) -> &'static str {
    std::fs::write(repo.join("dirty.txt"), b"committed\n").expect("the file");
    git(fixture, repo, &["add", "--", "dirty.txt"]);
    git(fixture, repo, &["commit", "--quiet", "--message=a file that gets unstaged work"]);
    std::fs::write(repo.join("dirty.txt"), b"unstaged work\n").expect("the change");
    return "unstaged work\n";
}

/// Whether the stash list and the merge autostash reference are empty:
/// the stash was applied and dropped.
fn autostash_is_gone(fixture: &Fixture, repo: &Path) -> bool {
    let listed: String = text(fixture, repo, &["stash", "list"]);
    let reference: Option<i32> = run_direct(fixture, repo, &["rev-parse", "--verify", "--quiet", "MERGE_AUTOSTASH"]).code;
    let entries: String = std::fs::read_dir(repo.join(".git")).expect("the Git directory").filter_map(|entry| { return entry.ok()?.file_name().into_string().ok(); }).filter(|name| { return name.contains("AUTOSTASH") || name.contains("MERGE"); }).collect::<Vec<String>>().join(" ");
    assert!(listed.is_empty() && reference == Some(1), "stash list {listed:?}, MERGE_AUTOSTASH lookup {reference:?}, state files {entries}");
    return true;
}

/// A merge told `--autostash`,
/// or set to it by `merge.autoStash`,
/// is finished by `git commit`,
/// which applies the stash after it
///  concludes the merge (`builtin/commit.c` line 1982);
/// the unstaged work comes back and nothing is left in the stash.
#[test]
fn a_merge_autostash_is_applied_by_the_ending_commit() {
    for (label, arguments, setting) in [("flag", vec!["merge", "--autostash", "--no-edit", "topic"], false), ("config", vec!["merge", "--no-edit", "topic"], true)] {
        let fixture: Fixture = fixture(format!("finish-merge-autostash-{label}").as_str());
        let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
        if setting {
            git(&fixture, repo.as_path(), &["config", "merge.autoStash", "true"]);
        }
        let work: &str = with_unstaged_work(&fixture, repo.as_path());
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(0), "{label}: {said}");
        assert_eq!(std::fs::read_to_string(repo.join("dirty.txt")).expect("the file"), work, "{label}: {said}");
        assert!(autostash_is_gone(&fixture, repo.as_path()), "{label}: the stash was applied and dropped: {said}");
        assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "{label}: {said}");
        assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == corpus(ADJACENT, "word.out"), "{label}");
        remove(&fixture);
    }
}

/// Unstaged work on the very line the word pass keeps,
/// so that applying the autostash after the merge conflicts.
fn with_overlapping_work(fixture: &Fixture, repo: &Path) -> &'static str {
    let edited: &str = "Flags are optional, edited by hand.\n";
    let original: String = String::from_utf8(corpus(ADJACENT, "ours")).expect("UTF-8");
    let changed: String = original.replace("Flags are optional, see below.\n", edited);
    assert_ne!(changed, original, "the edit lands on the line the merge keeps");
    std::fs::write(repo.join(tracked(ADJACENT)), changed.as_bytes()).expect("the change");
    git(fixture, repo, &["status", "--porcelain=v1"]);
    return edited;
}

/// What a person keeps after a merge with `--autostash` ends:
/// the stash list,
/// the porcelain status,
/// the merged file and the file
///  the unsaved work touches.
type End = (String, String, String, String);

/// The end state of a repository after a merge with `--autostash`.
fn after_autostash(fixture: &Fixture, repo: &Path) -> End {
    return (
        text(fixture, repo, &["stash", "list"]),
        status(fixture, repo),
        std::fs::read_to_string(repo.join(tracked(ADJACENT))).expect("the file"),
        std::fs::read_to_string(repo.join("dirty.txt")).unwrap_or_default(),
    );
}

/// How a control asks for the merge:
/// the command,
/// and whether it is a pull from a remote that holds the topic as its `main`.
#[derive(Clone, Copy)]
enum Ask {
    /// `git merge --autostash --no-edit topic`.
    Merge,
    /// `git pull --no-rebase --autostash --no-edit`.
    Pull,
}

impl Ask {
    /// The command line.
    fn arguments(self) -> Vec<&'static str> {
        return match self {
            Ask::Merge => vec!["merge", "--autostash", "--no-edit", "topic"],
            Ask::Pull => vec!["pull", "--no-rebase", "--autostash", "--no-edit"],
        };
    }
}

/// A repository whose merge conflicts for Git and resolves by word,
/// carrying unstaged work the stash will conflict with when it
///  is applied after the merge:
/// a change on the line the merge keeps and,
/// when `with_dirty_file` is set,
/// a change of a second file.
fn conflicted_autostash_repo(fixture: &Fixture, ask: Ask, with_dirty_file: bool) -> PathBuf {
    let repo: PathBuf = prepared(fixture, Stop::Merge, &[ADJACENT]);
    if matches!(ask, Ask::Pull) {
        git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
        git(fixture, repo.as_path(), &["remote", "add", "origin", fixture.root.join("remote.git").to_str().expect("UTF-8")]);
        git(fixture, repo.as_path(), &["push", "--quiet", "origin", "topic:main"]);
        git(fixture, repo.as_path(), &["fetch", "--quiet", "origin"]);
        git(fixture, repo.as_path(), &["branch", "--set-upstream-to=origin/main", "main"]);
    }
    if with_dirty_file {
        with_unstaged_work(fixture, repo.as_path());
    }
    with_overlapping_work(fixture, repo.as_path());
    return repo;
}

/// What real Git ends with when the merge stops,
/// the person resolves it as the word pass does,
/// and `git commit` concludes it:
/// the
///  status of that commit,
/// the end state,
/// and whether `MERGE_AUTOSTASH` is gone.
/// `unsaved_after_stop` writes the second file again
///  before the commit,
/// so the apply meets unsaved changes to a file the stash touches.
fn git_twin(label: &str, ask: Ask, with_dirty_file: bool, unsaved_after_stop: bool) -> (Option<i32>, End) {
    let twin: Fixture = fixture(format!("autostash-twin-{label}").as_str());
    let repo: PathBuf = conflicted_autostash_repo(&twin, ask, with_dirty_file);
    let stopped: Observed = run_direct(&twin, repo.as_path(), ask.arguments().as_slice());
    assert_eq!(stopped.code, Some(1), "Git stops the merge: {}", stderr_of(&stopped));
    std::fs::write(repo.join(tracked(ADJACENT)), corpus(ADJACENT, "word.out")).expect("the resolution");
    git(&twin, repo.as_path(), &["add", "--", tracked(ADJACENT).as_str()]);
    if unsaved_after_stop {
        std::fs::write(repo.join("dirty.txt"), b"unsaved again\n").expect("the unsaved work");
    }
    let concluded: Observed = run_direct(&twin, repo.as_path(), &["commit", "--no-edit"]);
    let end: End = after_autostash(&twin, repo.as_path());
    assert!(end.0.contains("autostash"), "Git keeps the stash it could not apply: {end:?} {}", stderr_of(&concluded));
    remove(&twin);
    return (concluded.code, end);
}

/// The wrapper's merge with `--autostash`,
/// whose stash conflicts when it is applied after the merge,
/// ends as Git ends:
/// the merge
///  commit is made,
/// the apply conflicts,
/// `stash@{0}: autostash` is kept,
/// the file is unmerged,
/// and the status is the commit's.
#[test]
fn a_merge_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it() {
    let (git_code, git_end): (Option<i32>, End) = git_twin("merge", Ask::Merge, false, false);
    let fixture: Fixture = fixture("finish-autostash-conflict-merge");
    let repo: PathBuf = conflicted_autostash_repo(&fixture, Ask::Merge, false);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), Ask::Merge.arguments().as_slice());
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, git_code, "the status is the concluding commit's, as in Git: {said}");
    assert_eq!(after_autostash(&fixture, repo.as_path()), git_end, "the stash list, the status and the files are Git's: {said}");
    assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "the merge is concluded: {said}");
    assert!(!repo.join(".git/MERGE_AUTOSTASH").exists(), "the stash was consumed: {said}");
    assert_eq!(text(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", "HEAD"]).split_whitespace().count(), 3, "a merge commit: {said}");
    remove(&fixture);
}

/// The same through a pull,
/// which hands `--autostash` to its merge.
#[test]
fn a_pull_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it() {
    let (git_code, git_end): (Option<i32>, End) = git_twin("pull", Ask::Pull, false, false);
    let fixture: Fixture = fixture("finish-autostash-conflict-pull");
    let repo: PathBuf = conflicted_autostash_repo(&fixture, Ask::Pull, false);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), Ask::Pull.arguments().as_slice());
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, git_code, "the status is the concluding commit's, as in Git: {said}");
    assert_eq!(after_autostash(&fixture, repo.as_path()), git_end, "the stash list, the status and the files are Git's: {said}");
    assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "the merge is concluded: {said}");
    remove(&fixture);
}

/// The person's own `git commit` after `git merge --autostash` stopped,
/// with a stash that conflicts on apply,
/// ends as Git's does:
///  `stash@{0}: autostash` kept and the file unmerged.
/// `unsaved` adds a second file the stash touches that the person edited again
///  after the merge stopped,
/// so Git's apply refuses it;
/// the wrapper's commit must end the same way.
fn commit_after_a_stopped_merge_ends_as_git_does(label: &str, unsaved: bool) {
    let (git_code, git_end): (Option<i32>, End) = git_twin(label, Ask::Merge, unsaved, unsaved);
    let fixture: Fixture = fixture(format!("commit-autostash-conflict-{label}").as_str());
    let repo: PathBuf = conflicted_autostash_repo(&fixture, Ask::Merge, unsaved);
    let first: Observed = run_direct(&fixture, repo.as_path(), Ask::Merge.arguments().as_slice());
    assert_eq!(first.code, Some(1), "Git stops the merge: {}", stderr_of(&first));
    std::fs::write(repo.join(tracked(ADJACENT)), corpus(ADJACENT, "word.out")).expect("the resolution");
    git(&fixture, repo.as_path(), &["add", "--", tracked(ADJACENT).as_str()]);
    if unsaved {
        std::fs::write(repo.join("dirty.txt"), b"unsaved again\n").expect("the unsaved work");
    }
    let concluded: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--no-edit"]), b"");
    let said: String = stderr_of(&concluded);
    assert_eq!(concluded.code, git_code, "the status is Git's: {said}");
    let ours: End = after_autostash(&fixture, repo.as_path());
    assert_eq!(ours, git_end, "the stash list, the status and the files are Git's: {said}");
    // The stash entry is whole: its objects are in the repository and hold the person's edit, and the repository is sound.
    assert!(text(&fixture, repo.as_path(), &["stash", "show", "-p", "stash@{0}"]).contains("edited by hand"), "the stash holds the edit: {said}");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove(&fixture);
}

/// A conflicting autostash concluded by the person's own commit.
#[test]
fn a_commit_that_applies_a_conflicting_autostash_keeps_the_stash_as_git_keeps_it() {
    commit_after_a_stopped_merge_ends_as_git_does("conflict", false);
}

/// The case the finish's dry run never modelled:
/// unsaved changes in the worktree to a file the stash touches.
/// Git's apply refuses
///  them and keeps the whole stash;
/// the wrapper's commit ends the same way.
#[test]
fn a_commit_over_unsaved_changes_to_a_stash_file_keeps_the_stash_as_git_keeps_it() {
    commit_after_a_stopped_merge_ends_as_git_does("unsaved", true);
}

/// A rebase told `--autostash`,
/// or set to it by `rebase.autoStash`,
/// applies the stash when `rebase --continue` ends it
///  (`sequencer.c` lines 4980 and 5330).
#[test]
fn a_rebase_autostash_is_applied_when_the_rebase_ends() {
    for (label, arguments, setting) in [("flag", vec!["rebase", "--autostash", "main"], false), ("config", vec!["rebase", "main"], true)] {
        let fixture: Fixture = fixture(format!("finish-rebase-autostash-{label}").as_str());
        let repo: PathBuf = prepared(&fixture, Stop::Rebase, &[ADJACENT]);
        if setting {
            git(&fixture, repo.as_path(), &["config", "rebase.autoStash", "true"]);
        }
        let work: &str = with_unstaged_work(&fixture, repo.as_path());
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert_eq!(observed.code, Some(0), "{label}: {said}");
        assert_eq!(std::fs::read_to_string(repo.join("dirty.txt")).expect("the file"), work, "{label}: {said}");
        assert!(autostash_is_gone(&fixture, repo.as_path()), "{label}: {said}");
        assert_eq!(state_left(repo.as_path()), Vec::<&str>::new(), "{label}: {said}");
        remove(&fixture);
    }
}

/// A pull told `--autostash` hands it to its merge,
/// and the ending commit applies the stash.
#[test]
fn a_pull_autostash_is_applied_by_the_ending_commit() {
    let fixture: Fixture = fixture("finish-pull-autostash");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(&fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "topic:main"]);
    git(&fixture, repo.as_path(), &["fetch", "--quiet", "origin"]);
    git(&fixture, repo.as_path(), &["branch", "--set-upstream-to=origin/main", "main"]);
    let work: &str = with_unstaged_work(&fixture, repo.as_path());
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["pull", "--no-rebase", "--autostash", "--no-edit"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(std::fs::read_to_string(repo.join("dirty.txt")).expect("the file"), work, "{said}");
    assert!(autostash_is_gone(&fixture, repo.as_path()), "{said}");
    remove(&fixture);
}

/// A `branch.<name>.mergeOptions` setting is applied as Git applies it,
/// before the command line:
/// `--no-commit` there stops the
///  merge as a clean run stops,
/// and a quoted `-m` there is the merge message.
#[test]
fn a_merge_options_setting_is_applied_to_the_finish() {
    let first: Fixture = fixture("finish-merge-options-stop");
    let repo: PathBuf = prepared(&first, Stop::Merge, &[ADJACENT]);
    git(&first, repo.as_path(), &["config", "branch.main.mergeoptions", "--no-commit"]);
    let before: String = text(&first, repo.as_path(), &["rev-parse", "HEAD"]);
    let observed: Observed = run_wrapped(&first, repo.as_path(), &["merge", "--no-edit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(said.contains("Automatic merge went well; stopped before committing as requested"), "{said}");
    assert_eq!(text(&first, repo.as_path(), &["rev-parse", "HEAD"]), before, "{said}");
    assert_eq!(state_left(repo.as_path()), vec!["MERGE_HEAD"], "{said}");
    remove(&first);
    let second: Fixture = fixture("finish-merge-options-message");
    let again: PathBuf = prepared(&second, Stop::Merge, &[ADJACENT]);
    git(&second, again.as_path(), &["config", "branch.main.mergeoptions", "--no-edit -m \"from the setting\""]);
    let named: Observed = run_wrapped(&second, again.as_path(), &["merge", "topic"]);
    assert_eq!(named.code, Some(0), "{}", stderr_of(&named));
    assert_eq!(text(&second, again.as_path(), &["log", "-1", "--format=%B"]), "from the setting");
    remove(&second);
}

/// The events a user's hook files and configured hooks see:
/// a recorder for each,
/// appending its event name to one file.
fn record_hooks(fixture: &Fixture, repo: &Path) -> PathBuf {
    let log: PathBuf = fixture.root.join("hooks-seen");
    let events: [&str; 10] = ["pre-merge-commit", "prepare-commit-msg", "commit-msg", "post-merge", "pre-commit", "post-commit", "post-checkout", "post-rewrite", "pre-rebase", "reference-transaction"];
    std::fs::create_dir_all(repo.join(".git/hooks")).expect("hooks directory");
    for event in events {
        let body: String = format!("#!/bin/sh\nprintf '%s\\n' '{event}' >> '{}'\nexit 0\n", log.display());
        executable(repo.join(".git/hooks").join(event).as_path(), body.as_bytes());
        let script: PathBuf = fixture.root.join(format!("recorder-{event}"));
        executable(script.as_path(), body.as_bytes());
        git(fixture, repo, &["config", format!("hook.record-{event}.event").as_str(), event]);
        git(fixture, repo, &["config", format!("hook.record-{event}.command").as_str(), script.to_str().expect("UTF-8")]);
    }
    return log;
}

/// The sorted,
/// distinct events a log holds;
/// empty when nothing was written.
fn events_in(log: &Path) -> Vec<String> {
    let mut seen: Vec<String> = std::fs::read_to_string(log).unwrap_or_default().lines().map(String::from).collect();
    seen.sort();
    seen.dedup();
    return seen;
}

/// A user's hooks see the same on a finished run as on a clean one.
/// The wrapper supports no external hook program (every Git it
///  starts has every hook event switched off),
/// so both see nothing;
/// the positive control is that real Git,
/// run directly,
/// does
///  fire them,
/// so the recorder can tell.
/// If hooks were ever supported,
/// the clean merge would run `pre-merge-commit`,
///  `prepare-commit-msg`,
/// `commit-msg` and `post-merge` (`builtin/merge.c` lines 931,
/// 966,
/// 975 and 537) and a concluding `git commit`
///  `pre-commit`,
/// `prepare-commit-msg`,
/// `commit-msg` and `post-commit` (`builtin/commit.c` lines 780,
/// 1116,
/// 1133 and 1966).
#[test]
fn user_hooks_see_the_same_on_a_finished_run_as_on_a_clean_run() {
    for stop in STOPS {
        let finished_fixture: Fixture = fixture(format!("finish-hooks-{stop:?}").to_lowercase().as_str());
        let finished_repo: PathBuf = prepared(&finished_fixture, stop, &[ADJACENT]);
        let finished_log: PathBuf = record_hooks(&finished_fixture, finished_repo.as_path());
        let finished: Observed = run_wrapped(&finished_fixture, finished_repo.as_path(), arguments_of(stop).as_slice());
        assert_eq!(finished.code, Some(0), "{stop:?}: {}", stderr_of(&finished));
        // The same operation without a conflict: the files of the two sides touch different lines, so Git merges cleanly.
        let clean_fixture: Fixture = fixture(format!("finish-hooks-clean-{stop:?}").to_lowercase().as_str());
        let clean_repo: PathBuf = prepared(&clean_fixture, stop, &["md4"]);
        let clean_log: PathBuf = record_hooks(&clean_fixture, clean_repo.as_path());
        let clean: Observed = run_wrapped(&clean_fixture, clean_repo.as_path(), arguments_of(stop).as_slice());
        assert_eq!(clean.code, Some(0), "{stop:?}: {}", stderr_of(&clean));
        assert_eq!(events_in(finished_log.as_path()), events_in(clean_log.as_path()), "{stop:?}: the finished run and the clean run show the same hooks");
        remove(&finished_fixture);
        remove(&clean_fixture);
    }
    // Positive control: the recorders fire for real Git run directly.
    let control: Fixture = fixture("finish-hooks-control");
    let repo: PathBuf = prepared(&control, Stop::Merge, &["md4"]);
    let log: PathBuf = record_hooks(&control, repo.as_path());
    let direct: Observed = run_direct(&control, repo.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(direct.code, Some(0), "{}", stderr_of(&direct));
    assert!(!events_in(log.as_path()).is_empty(), "real Git fires the recorders, so an empty log from the wrapper means something");
    remove(&control);
}

/// Two lines of one file,
/// each changed on one side of the same base,
/// so the first replay's line conflict resolves by word and the second's does not.
const TWO_LINES_BASE: &str = "alpha = 0\nbeta = 0\n";

/// A repository on `topic` whose two commits each change one line of `data.txt`,
/// with `main` holding the other side's change to the second line.
fn two_stops_ready(name: &str) -> (Fixture, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let write = |text: &str| { std::fs::write(repo.join("data.txt"), text).expect("the file"); };
    write(TWO_LINES_BASE);
    git(&fixture, repo.as_path(), &["add", "--", "data.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    write("alpha = 0\nbeta = 9\n");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=ours"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic", base.as_str()]);
    write("alpha = 1\nbeta = 0\n");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=first"]);
    write("alpha = 1\nbeta = 2\n");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=second"]);
    return (fixture, repo);
}

/// What:
///  The line after a rebase that stopped again names the file that is conflicted now,
/// not the state before the continuation ran.
/// Why:
///   `git rebase --continue` commits the resolved pick and stops on the next one's conflict,
/// so the file the wrapper staged is
///  unmerged again when it prints
/// (cross-vendor review of this branch,
/// finding 5):
/// "the conflict is gone" and "stays staged as resolved"
///  describe a state the person does not have.
#[test]
fn a_rebase_that_stops_again_is_told_what_is_conflicted_now() {
    let (fixture, repo) = two_stops_ready("finish-rebase-stopped-again");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["rebase", "main"]);
    let said: String = stderr_of(&observed);
    assert_ne!(observed.code, Some(0), "the rebase is not finished: {said}");
    assert_eq!(unmerged(&fixture, repo.as_path()), vec![String::from("data.txt")], "the second change is conflicted: {said}");
    assert!(state_left(repo.as_path()).contains(&"rebase-merge") || state_left(repo.as_path()).contains(&"rebase-apply"), "the rebase is in progress: {said}");
    assert!(said.contains("conflicted again"), "the line names the state the person has: {said}");
    assert!(!said.contains("the conflict is gone"), "the conflict is back, and the line must not say it is gone: {said}");
    assert!(said.contains("git rebase --continue"), "the command that ends it is named: {said}");
    let marked: String = String::from_utf8(std::fs::read(repo.join("data.txt")).expect("the file")).expect("UTF-8");
    assert!(marked.contains("<<<<<<<") && marked.contains(">>>>>>>"), "Git left its conflict markers: {marked}");
    remove(&fixture);
}

/// What:
///  A `git pull --rebase` that stopped again is told to run `git rebase --continue`,
/// not `git commit`.
/// Why:
///   A pull runs either a merge or a rebase as its child (`builtin/pull.c`),
/// so the operation's own word says
///  nothing about which command ends it
/// (cross-vendor review of this branch,
/// finding 6);
/// `git commit` does not resume the rebase or
///  process its remaining commits.
#[test]
fn a_rebasing_pull_that_stops_again_is_told_to_continue_the_rebase() {
    let (fixture, repo) = two_stops_ready("finish-pull-rebase-stopped-again");
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(&fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "main"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["pull", "--rebase", "origin", "main"]);
    let said: String = stderr_of(&observed);
    assert_ne!(observed.code, Some(0), "the pull is not finished: {said}");
    assert!(said.contains("git rebase --continue"), "the command that ends a rebase is named: {said}");
    assert!(!said.contains("`git commit`"), "`git commit` cannot end a rebase: {said}");
    assert!(said.contains("conflicted again"), "the file that is conflicted now is named: {said}");
    remove(&fixture);
}
