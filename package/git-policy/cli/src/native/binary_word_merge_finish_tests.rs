//! What:
//!  The finish after the word-level second pass, through the built executable: when every conflicted file of a merge, rebase,
//!  cherry-pick, revert or pull resolves, the operation ends the way Git ends it after a clean run, and the options that stop a
//!  clean run (`--no-commit`, `--squash`, `--edit`) stop it here too.
//! Why:
//!  The owner answered on 2026-11-03: "Finish like a clean merge (Recommended)". The continuing step goes through the wrapper's
//!  own command path, so a later pick of a rebase gets the second pass and the commit gets the policies and the hooks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = await wrapped(['merge', '--no-edit', 'topic']); // status 0, a merge commit, nothing left unmerged
//! ```

use super::support::{Fixture, Observed, executable, fixture, git, observe, remove, repository, run_wrapped, stderr_of, wrapped};
use super::word_merge::{ADJACENT, Stop, STOPS, arguments_of, corpus, expected_after, merge_lines, prepared, tracked, unmerged};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The corpus input whose two sides edit a list item and add an item after it: Git conflicts, the word pass resolves.
const SECOND_ADJACENT: &str = "md1c";

/// The state files and directories an unfinished operation leaves in the Git directory.
const STATE: [&str; 6] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply", "sequencer"];

/// The state of an unfinished operation that is present.
fn state_left(repo: &Path) -> Vec<&'static str> {
    return STATE.iter().copied().filter(|name| { return repo.join(".git").join(name).exists(); }).collect();
}

/// A query against the repository, as trimmed text.
fn text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, arguments).stdout).trim().to_owned();
}

/// The porcelain status, which is empty when nothing is staged, modified or untracked.
fn status(fixture: &Fixture, repo: &Path) -> String {
    return text(fixture, repo, &["status", "--porcelain=v1", "--untracked-files=all"]);
}

/// An editor that appends a line, so the message it was handed shows in the commit.
fn appending_editor(fixture: &Fixture) -> PathBuf {
    let path: PathBuf = fixture.root.join("appending-editor");
    executable(path.as_path(), b"#!/bin/sh\nprintf 'edited by the editor\\n' >> \"$1\"\n");
    return path;
}

/// Run the arguments with an editor named.
fn run_with_editor(fixture: &Fixture, repo: &Path, editor: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).env("GIT_EDITOR", editor).current_dir(repo).args(arguments), b"");
}

/// Each of the four operations ends like a clean run: status 0, the commit, nothing left unmerged or unfinished, and the part line.
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

/// A file that does not resolve keeps today's stop: nothing is committed and Git's status stays. (A revert has no such input in the
///  corpus: its later state is the resolved text of the input.)
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
        assert_eq!(text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), before, "{stop:?}: nothing is committed: {said}");
        assert!(!state_left(repo.as_path()).is_empty(), "{stop:?}: the operation is still in progress");
        remove(&fixture);
    }
}

/// `git merge --no-commit` stops as a clean one does: status 0, Git's line, the merge state kept, nothing committed.
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

/// `git merge --squash` stops as a clean one does: status 0, Git's line, no merge state, no commit, the result staged.
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

/// `--edit` hands the prepared message to the editor and commits what it leaves; `-m` is the message of a merge with no editor.
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

/// A rebase whose second pick also conflicts: the continuing step runs through the wrapper, so the later pick gets the pass too.
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

/// `git cherry-pick --no-commit` stops as a clean one does: status 0, the result staged, no commit, no pick state.
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

/// A range whose first pick conflicts: the first is committed and the rest of the sequence is picked, so both land.
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

/// A pull's merge, its merge with `--no-commit`, and its rebase end like the plain commands do.
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

/// The commit that ends the merge goes through the wrapper's command path: a configured commit hook runs for it.
#[test]
fn the_ending_commit_runs_the_configured_hook() {
    let fixture: Fixture = fixture("finish-hook");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let marker: PathBuf = fixture.root.join("hook-ran");
    let hook: PathBuf = fixture.root.join("record-hook");
    executable(hook.as_path(), format!("#!/bin/sh\nprintf ran > '{}'\n", marker.display()).as_bytes());
    git(&fixture, repo.as_path(), &["config", "hook.record.event", "pre-commit"]);
    git(&fixture, repo.as_path(), &["config", "hook.record.command", hook.to_str().expect("UTF-8")]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(marker.exists(), "the hook ran for the ending commit: {}", stderr_of(&observed));
    remove(&fixture);
}
