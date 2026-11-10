//! What:
//!  A merge that carries the person's edits,
//! killed at each step of the carry,
//! and what the next command's recovery leaves.
//! Why:
//!  The owner's rule is "never discard user data",
//! and the carry exists because the person's bytes must survive a
//!  `SIGKILL` at any instant (`SPEC.md`,
//! "Merges and commits never fail").
//! The steps are: the capture and its record,
//! the
//!  clear that hands the paths to Git,
//! the branch move,
//! and the carry back.
//! After a kill the next command must
//!  leave the checkout holding either what it held before the merge or the whole result,
//! never a half-applied
//!  one and never a record that stops every later command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const killed = await wrapped(['merge', '--no-edit', 'feature'], { env: { CLI_GIT_TEST_ONLY_PHASE_SIGNAL: 'merge-cleared:kill:dir' } });
//! expect(killed.signal).toBe(9); await wrapped(['status']); expect(read('a.txt')).toContain('the person line');
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::merge_carries_edits::{FEATURE_THIRD, PERSON_LINE, assert_merged, diverged, on_main_a, records, side_refs, status, text_at};
use super::support::{Fixture, Observed, fixture, git, observe};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Output;

/// What the checkout held before the merge,
/// read as the things a carry must not change.
#[derive(Debug, Eq, PartialEq)]
struct Held {
    head: String,
    index: String,
    status: String,
    person: Option<String>,
}

/// The checkout as the person left it,
/// which a merge that did not land must not change.
fn held(fixture: &Fixture, repo: &Path) -> Held {
    return Held {
        head: git_text(fixture, repo, &["rev-parse", "HEAD"]),
        index: git_text(fixture, repo, &["--no-optional-locks", "ls-files", "--stage"]),
        status: status(fixture, repo),
        person: text_at(repo, "a.txt"),
    };
}

/// Kill a merge at one phase,
/// and say that the phase was reached.
fn killed_at(fixture: &Fixture, repo: &Path, phase: &str) {
    let markers: PathBuf = fixture.root.join(format!("markers-{phase}"));
    std::fs::create_dir(&markers).expect("marker directory");
    let signal: String = format!("{phase}:kill:{}", markers.display());
    let output: Output = wrapped_commit(fixture, repo, &["merge", "--no-edit", "feature"])
        .env(PHASE_SIGNAL_VARIABLE, signal)
        .output()
        .expect("the merge starts");
    assert!(
        output.status.signal() == Some(9) || output.status.code() == Some(137),
        "{phase}: the merge died of the marker: {:?} {}",
        output.status,
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    assert!(markers.join(format!("{phase}.reached")).exists(), "{phase} was reached");
}

/// Run a command that reads the repository,
/// which is what finishes or puts back a killed merge.
fn after_the_kill(fixture: &Fixture, repo: &Path) {
    let observed: Observed = observe(&mut wrapped_commit(fixture, repo, &["status", "--porcelain=v1"]), b"");
    assert_eq!(observed.code, Some(0), "the next command runs: {}", String::from_utf8_lossy(observed.stderr.as_slice()));
}

/// A repository whose `a.txt` holds the person's unsaved line after `main`'s own text.
fn edited(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = diverged(fixture, name);
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", on_main_a())).expect("the person's edit");
    return repo;
}

/// A kill after the capture and before the clear:
/// the checkout is exactly as the person left it,
/// and no record or side ref outlives the command.
#[test]
fn a_kill_after_the_capture_leaves_the_checkout_as_it_was() {
    let fixture: Fixture = fixture("carry-kill-captured");
    let repo: PathBuf = edited(&fixture, "repo");
    let before: Held = held(&fixture, repo.as_path());
    killed_at(&fixture, repo.as_path(), "merge-carried");
    after_the_kill(&fixture, repo.as_path());
    assert_ne!(before.person, Some(on_main_a()), "the person's edit is in the fixture");
    assert_eq!(held(&fixture, repo.as_path()), before, "the checkout is what the person left");
    assert_eq!(records(repo.as_path()), "", "the record went with the merge that did not start");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A kill after the clear and before Git updates the checkout:
/// the next command puts the person's edits back,
/// and the branch does not move.
#[test]
fn a_kill_after_the_clear_puts_the_edits_back() {
    let fixture: Fixture = fixture("carry-kill-cleared");
    let repo: PathBuf = edited(&fixture, "repo");
    let before: Held = held(&fixture, repo.as_path());
    killed_at(&fixture, repo.as_path(), "merge-cleared");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(on_main_a().as_str()), "the path was cleared for the merge");
    after_the_kill(&fixture, repo.as_path());
    assert_eq!(held(&fixture, repo.as_path()), before, "the person's edits are back, and the branch did not move");
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A kill after the branch moved and before the carry:
/// the next command carries the edits and leaves the merge in place.
#[test]
fn a_kill_after_the_branch_moved_carries_the_edits() {
    let fixture: Fixture = fixture("carry-kill-applied");
    let repo: PathBuf = edited(&fixture, "repo");
    killed_at(&fixture, repo.as_path(), "merge-applied");
    after_the_kill(&fixture, repo.as_path());
    assert_merged(&fixture, repo.as_path(), "kill after the branch moved");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(FEATURE_THIRD), "the merge's line is there: {after:?}");
    assert!(after.contains(PERSON_LINE), "the person's line is there: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    remove_settled(&fixture);
}

/// A kill after the carry and before the record went:
/// the next command finds the carried bytes and only finishes the record.
#[test]
fn a_kill_after_the_carry_keeps_the_carried_bytes() {
    let fixture: Fixture = fixture("carry-kill-carried");
    let repo: PathBuf = edited(&fixture, "repo");
    killed_at(&fixture, repo.as_path(), "merge-carried-back");
    after_the_kill(&fixture, repo.as_path());
    assert_merged(&fixture, repo.as_path(), "kill after the carry");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(PERSON_LINE) && after.contains(FEATURE_THIRD), "the carried bytes are still there: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    remove_settled(&fixture);
}

/// A merge that landed and was then built on by another session before the carry ran:
/// the person's edits come back onto the branch as it now is,
/// and nothing of theirs is lost.
#[test]
fn an_edit_survives_a_branch_that_moved_after_the_merge_landed() {
    let fixture: Fixture = fixture("carry-moved");
    let repo: PathBuf = edited(&fixture, "repo");
    killed_at(&fixture, repo.as_path(), "merge-applied");
    std::fs::write(repo.join("other.txt"), b"another session's file\n").expect("other session");
    git(&fixture, repo.as_path(), &["add", "--", "other.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=another session's commit"]);
    after_the_kill(&fixture, repo.as_path());
    assert_eq!(
        text_at(repo.as_path(), "a.txt").as_deref(),
        Some(format!("{}{PERSON_LINE}\n", on_main_a().replace("three", FEATURE_THIRD)).as_str()),
        "the merge's line and the person's line"
    );
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}
