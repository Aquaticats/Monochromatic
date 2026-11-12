//! What:
//!  A `git checkout` or `git restore` that would replace a conflicted file during a real merge,
//! rebase (both backends),
//! cherry-pick and revert,
//! through the
//!  built executable:
//! the command is refused,
//! the refusal names every file it would replace and the opt-out,
//! the opt-out runs the command as real Git runs it,
//! and nothing is saved.
//! Why:
//!  The resolution of a conflict is where the person's own work stands during an operation,
//! and no copy of it is kept.
//! These controls make a real conflict with real Git,
//! edit the conflicted file by hand
//!  as a person resolving it would,
//! run the wrapper,
//! compare what it said and what is left on disk,
//! and run the same command through real Git alone for the answer the opt-out must reach.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await makeConflict('merge'); edit('a.rs'); const out = run(['checkout', '--theirs', '--', 'a.rs']);
//! expect(out.stderr).toContain('--no-enforce-worktree'); expect(readFileSync('a.rs')).toEqual(handEdited);
//! ```

use super::support::{Fixture, Observed, fixture, git, remove, repository, run_direct, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What a person had in the conflicted file when they ran the command:
/// not any side's content,
/// not valid UTF-8.
const HAND_EDITED: &[u8] = b"<<<<<<< ours\nmine \xff\n=======\ntheirs\n>>>>>>> topic\nand my own line\n";

/// Which operation a control stops in.
#[derive(Clone, Copy, Debug)]
enum Stop {
    /// `git merge topic`.
    Merge,
    /// `git rebase main` on `topic`,
    /// with the merge backend.
    RebaseMerge,
    /// `git rebase --apply main` on `topic`,
    /// with the apply backend.
    RebaseApply,
    /// `git cherry-pick topic`.
    CherryPick,
    /// `git revert` of the commit that changed `a.rs` on `main`,
    /// after `a.rs` changed again.
    Revert,
}

/// The word the refusal uses for the operation `stop` leaves waiting.
fn operation_word(stop: Stop) -> &'static str {
    return match stop {
        Stop::Merge => "merge",
        Stop::RebaseMerge | Stop::RebaseApply => "rebase",
        Stop::CherryPick => "cherry-pick",
        Stop::Revert => "revert",
    };
}

/// A repository stopped by `stop` with `a.rs` conflicted and `b.rs` and `c.rs` clean,
/// and `a.rs` hand-edited.
fn conflicted(fixture: &Fixture, stop: Stop) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    for (name, text) in [("a.rs", "base\n"), ("b.rs", "base\n"), ("c.rs", "keep\n")] {
        std::fs::write(repo.join(name), text).expect("file");
    }
    git(fixture, repo.as_path(), &["add", "--", "a.rs", "b.rs", "c.rs"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join("a.rs"), "topic\n").expect("topic side");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic side"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("a.rs"), "main\n").expect("main side");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main side"]);
    let started: Observed = match stop {
        Stop::Merge => run_direct(fixture, repo.as_path(), &["merge", "--no-edit", "topic"]),
        Stop::CherryPick => run_direct(fixture, repo.as_path(), &["cherry-pick", "topic"]),
        Stop::RebaseMerge | Stop::RebaseApply => {
            git(fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
            let arguments: Vec<&str> = if matches!(stop, Stop::RebaseApply) { vec!["rebase", "--apply", "main"] } else { vec!["rebase", "main"] };
            run_direct(fixture, repo.as_path(), arguments.as_slice())
        }
        Stop::Revert => {
            std::fs::write(repo.join("a.rs"), "later\n").expect("later change");
            git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=later change"]);
            run_direct(fixture, repo.as_path(), &["revert", "--no-edit", "HEAD~1"])
        }
    };
    assert_eq!(started.code, Some(1), "the operation stops with a conflict: {}{}", String::from_utf8_lossy(&started.stdout), stderr_of(&started));
    assert!(
        git(fixture, repo.as_path(), &["ls-files", "--unmerged"]).stdout.windows(4).any(|window| { return window == b"a.rs"; }),
        "a.rs is unmerged"
    );
    std::fs::write(repo.join("a.rs"), HAND_EDITED).expect("hand edit");
    return repo;
}

/// Whether the wrapper itself rejected the command.
fn rejected_by_policy(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// The directory of the copies below the common Git directory of `repo`.
fn saved_root(repo: &Path) -> PathBuf {
    return repo.join(".git/monochromatic-git-policy-cli/discarded");
}

/// Every audit line of the repository.
fn audit_lines(repo: &Path) -> Vec<String> {
    return std::fs::read_to_string(repo.join(".git/monochromatic-git-policy-cli-audit.jsonl")).unwrap_or_default().lines().map(String::from).collect();
}

/// The wrapper-only opt-out every refusal of this policy names.
const OPT_OUT: &str = "--no-enforce-worktree";

/// Whether the refusal says what the command would replace:
/// the file,
/// that the index marks it as unmerged,
/// the operation that waits,
/// and the opt-out.
fn names_the_conflicted_file(said: &str, stop: Stop) -> bool {
    return said.contains("`a.rs`")
        && said.contains("unmerged")
        && said.contains(operation_word(stop))
        && said.contains(OPT_OUT)
        && said.contains("keeps no copy");
}

/// No copy of the person's work is anywhere below the Git directory.
fn nothing_was_saved(repo: &Path, said: &str) {
    assert!(!saved_root(repo).exists(), "no copy was made: {said}");
    assert!(audit_lines(repo).iter().all(|line| { return !line.contains("saved-conflict"); }), "no saved-conflict line: {said}");
}

/// Every form the reading covers is refused during the operation it settles:
/// the refusal names the file it would replace and the opt-out,
/// the file is as the person wrote it,
/// and nothing is saved.
#[test]
fn a_conflict_side_is_refused_naming_the_file_and_the_opt_out() {
    let cases: [(Stop, &[&[&str]]); 5] = [
        (
            Stop::Merge,
            &[
                &["checkout", "--theirs", "--", "a.rs"],
                &["checkout", "--ours", "--", "a.rs"],
                &["checkout", "topic", "--", "a.rs"],
                &["checkout", "--merge", "--", "a.rs"],
                &["restore", "--theirs", "--", "a.rs"],
                &["restore", "--ours", "--", "a.rs"],
                &["restore", "--source=topic", "--", "a.rs"],
            ],
        ),
        (Stop::RebaseMerge, &[&["checkout", "--theirs", "--", "a.rs"], &["restore", "--ours", "--", "a.rs"]]),
        (Stop::RebaseApply, &[&["checkout", "--theirs", "--", "a.rs"]]),
        (Stop::CherryPick, &[&["checkout", "--theirs", "--", "a.rs"], &["restore", "--theirs", "--", "a.rs"]]),
        (Stop::Revert, &[&["checkout", "--ours", "--", "a.rs"]]),
    ];
    for (stop, forms) in cases {
        for (round, arguments) in forms.iter().enumerate() {
            let fixture: Fixture = fixture(format!("side-refused-{stop:?}-{round}").to_lowercase().as_str());
            let repo: PathBuf = conflicted(&fixture, stop);
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments);
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(names_the_conflicted_file(said.as_str(), stop), "{stop:?} {arguments:?}: the refusal names the file and the opt-out: {said}");
            assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "{stop:?} {arguments:?}: the file is as the person had it");
            nothing_was_saved(repo.as_path(), said.as_str());
            remove(&fixture);
        }
    }
}

/// The same holds in a linked worktree,
/// where a copy used to go to the common Git directory.
#[test]
fn a_linked_worktree_refuses_the_conflict_side_and_saves_nothing() {
    let fixture: Fixture = fixture("side-refused-linked");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.rs"), "base\n").expect("a.rs");
    git(&fixture, repo.as_path(), &["add", "--", "a.rs"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["branch", "topic"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("a.rs"), "side\n").expect("side edit");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
    std::fs::write(repo.join("a.rs"), "topic\n").expect("topic edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic"]);
    let merged: Observed = run_direct(&fixture, linked.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(merged.code, Some(1), "{}", stderr_of(&merged));
    std::fs::write(linked.join("a.rs"), HAND_EDITED).expect("hand edit");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert!(rejected_by_policy(&observed), "{said}");
    assert!(names_the_conflicted_file(said.as_str(), Stop::Merge), "{said}");
    assert_eq!(std::fs::read(linked.join("a.rs")).expect("a.rs"), HAND_EDITED);
    nothing_was_saved(repo.as_path(), said.as_str());
    remove(&fixture);
}

/// The opt-out runs the command as real Git runs it:
/// the file holds the side the command names,
/// no copy is kept,
/// and real Git does exactly the same in the same state.
#[test]
fn the_opt_out_replaces_the_conflicted_file_as_real_git_does() {
    let direct: Fixture = fixture("side-refused-hatch-direct");
    let direct_repo: PathBuf = conflicted(&direct, Stop::Merge);
    let git_ran: Observed = run_direct(&direct, direct_repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(git_ran.code, Some(0), "real Git takes the side: {}", stderr_of(&git_ran));
    let fixture: Fixture = fixture("side-refused-hatch");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", OPT_OUT, "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(
        std::fs::read(repo.join("a.rs")).expect("a.rs"),
        std::fs::read(direct_repo.join("a.rs")).expect("real Git's a.rs"),
        "the opt-out leaves what real Git leaves"
    );
    assert!(!saved_root(repo.as_path()).exists(), "the opt-out saves nothing: {said}");
    assert!(audit_lines(repo.as_path()).iter().any(|line| { return line.contains("\"outcome\":\"hatch\""); }), "the audit line names the opt-out");
    remove(&fixture);
    remove(&direct);
}

/// A path that is not conflicted is not named,
/// and the line that names it beside a conflicted one is refused whole:
/// the refusal claims only the file the index marks as unmerged,
/// and nothing is copied or changed.
#[test]
fn a_clean_path_is_not_named_as_a_conflict_and_the_line_is_refused() {
    for stop in [Stop::Merge, Stop::RebaseMerge, Stop::CherryPick] {
        let fixture: Fixture = fixture(format!("side-refused-clean-{stop:?}").to_lowercase().as_str());
        let repo: PathBuf = conflicted(&fixture, stop);
        for arguments in [
            vec!["checkout", "--theirs", "--", "c.rs"],
            vec!["checkout", "topic", "--", "b.rs"],
            vec!["restore", "--theirs", "--", "c.rs"],
            vec!["checkout", "--theirs", "--", "."],
            vec!["checkout", "--theirs", "--", "*.rs"],
        ] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(!said.contains("keeps no copy"), "{stop:?} {arguments:?}: no file of this line is unmerged, so no file is claimed: {said}");
        }
        for arguments in [vec!["checkout", "--theirs", "--", "a.rs", "c.rs"], vec!["restore", "--ours", "--", "c.rs", "a.rs"]] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(said.contains("`a.rs`") && said.contains("keeps no copy"), "{stop:?} {arguments:?} names the unmerged file: {said}");
            assert!(!said.contains("`c.rs`"), "{stop:?} {arguments:?}: the clean path is not called a conflict: {said}");
        }
        assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "the hand edit is untouched");
        nothing_was_saved(repo.as_path(), "");
        remove(&fixture);
    }
}

/// The forms the reading does not cover keep their rejection even for a conflicted path.
#[test]
fn forms_outside_the_reading_keep_the_rejection() {
    let fixture: Fixture = fixture("side-refused-uncovered");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    for arguments in [
        vec!["checkout", "-f", "--", "a.rs"],
        vec!["checkout", "--theirs", "a.rs"],
        vec!["checkout", "--patch", "--", "a.rs"],
        vec!["restore", "--staged", "--theirs", "--", "a.rs"],
        vec!["restore", "--theirs", "a.rs"],
        vec!["checkout", "--theirs", "--", "sub/"],
        vec!["reset", "--hard"],
        vec!["merge", "--abort"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
    }
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED);
    nothing_was_saved(repo.as_path(), "");
    remove(&fixture);
}

/// With no operation in progress a conflict-shaped line is an ordinary discarding form and is rejected,
/// and it names no conflict,
/// even for a path that
///  still has unmerged entries (what a conflicting `git stash pop` leaves).
#[test]
fn without_an_operation_in_progress_no_conflict_is_named() {
    let fixture: Fixture = fixture("side-refused-no-operation");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::remove_file(repo.join(".git/MERGE_HEAD")).expect("end the merge's state, leaving the unmerged entries");
    for arguments in [vec!["checkout", "--theirs", "--", "a.rs"], vec!["restore", "--source=topic", "--", "a.rs"]] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{arguments:?}: {said}");
        assert!(!said.contains("keeps no copy"), "{arguments:?}: no operation waits, so no conflict is claimed: {said}");
    }
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED);
    nothing_was_saved(repo.as_path(), "");
    remove(&fixture);
}

/// Command lines that look like one conflicted file to the guard and are more to Git,
/// found by a cross-vendor review.
#[path = "binary_conflict_side_hardening_tests.rs"]
mod hardening;
