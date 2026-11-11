//! What:
//!  A merge that carries the person's edits,
//! killed at each step of the carry,
//! and what the next command's recovery leaves.
//! Why:
//!  The owner's rule is "never discard user data",
//! and the carry exists because the person's bytes must survive a
//!  `SIGKILL` at any instant (`SPEC.md`,
//! "Merges and commits never fail").
//! The steps are:
//! the capture and its record,
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
use super::merge_carries_edits::{FEATURE_THIRD, PERSON_LINE, assert_merged, diverged, on_main_a, records, said, side_refs, status, text_at};
use super::support::{Fixture, Observed, fixture, git, observe};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The line the person staged,
/// which the index holds and the worktree no longer shows as the only layer.
const STAGED_LINE: &str = "the staged line";

/// The line the person left unsaved over the staged one.
const UNSAVED_LINE: &str = "the unsaved line";

/// The line a program saves while the merge runs.
const SAVED_DURING_THE_MERGE: &str = "a line saved while the merge ran";

/// How long a control waits for a paused merge to reach its marker,
/// which is far longer than a start takes and short of the command's own bound.
const MARKER_PATIENCE: Duration = Duration::from_secs(100);

/// How often a control looks for a marker.
const MARKER_POLL: Duration = Duration::from_millis(20);

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
    killed_at_occurrence(fixture, repo, phase, None);
}

/// Kill a merge at the nth arrival of one repeatable phase:
/// the variable names the phase,
/// and the marker file carries the number,
/// as `transaction_phase.rs` spells both.
fn killed_at_occurrence(fixture: &Fixture, repo: &Path, phase: &str, occurrence: Option<u64>) {
    let stem: String = match occurrence {
        Some(number) => format!("{phase}-{number}"),
        None => String::from(phase),
    };
    let markers: PathBuf = fixture.root.join(format!("markers-{stem}"));
    std::fs::create_dir(&markers).expect("marker directory");
    let signal: String = format!("{phase}:kill:{}", markers.display());
    let output: Output = wrapped_commit(fixture, repo, &["merge", "--no-edit", "feature"])
        .env(PHASE_SIGNAL_VARIABLE, signal)
        .output()
        .expect("the merge starts");
    assert!(
        output.status.signal() == Some(9) || output.status.code() == Some(137),
        "{stem}: the merge died of the marker: {:?} {}",
        output.status,
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    assert!(markers.join(format!("{stem}.reached")).exists(), "{stem} was reached");
}

/// Run a command that reads the repository,
/// which is what finishes or puts back a killed merge,
/// and give what it said.
fn after_the_kill(fixture: &Fixture, repo: &Path) -> Observed {
    let observed: Observed = observe(&mut wrapped_commit(fixture, repo, &["status", "--porcelain=v1"]), b"");
    assert_eq!(observed.code, Some(0), "the next command runs: {}", String::from_utf8_lossy(observed.stderr.as_slice()));
    return observed;
}

/// A repository whose `a.txt` holds a staged line in the index and an unsaved line over it in the worktree.
/// Why:
///   The two layers are what a kill between the index step of a carry and its file step can separate:
/// one is written before the other.
fn staged_and_unsaved(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = diverged(fixture, name);
    std::fs::write(repo.join("a.txt"), format!("{}{STAGED_LINE}\n", on_main_a())).expect("the person's staged edit");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), unsaved_text()).expect("the person's unsaved edit");
    return repo;
}

/// The person's staged line over `main`'s file,
/// which is what the index holds and the worktree of a later save starts from.
fn staged_text() -> String {
    return format!("{}{STAGED_LINE}\n", on_main_a());
}

/// The person's unsaved line over their staged one.
fn unsaved_text() -> String {
    return format!("{}{UNSAVED_LINE}\n", staged_text());
}

/// A merge paused at one phase,
/// with the directory its markers are written to.
struct Paused {
    /// The merge waiting for its release.
    child: Child,
    /// Where the marker and the release file go.
    markers: PathBuf,
}

/// Start a merge that stops at one phase,
/// with the marker written and the merge waiting.
fn paused_at(fixture: &Fixture, repo: &Path, phase: &str) -> Paused {
    let markers: PathBuf = fixture.root.join(format!("markers-{phase}-{}", std::process::id()));
    std::fs::create_dir(&markers).expect("marker directory");
    let signal: String = format!("{phase}:pause:{}", markers.display());
    let mut command: Command = wrapped_commit(fixture, repo, &["merge", "--no-edit", "feature"]);
    let child: Child = command
        .env(PHASE_SIGNAL_VARIABLE, signal)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the merge starts");
    let paused: Paused = Paused { child, markers };
    paused.wait_for(phase);
    return paused;
}

impl Paused {
    /// Wait until the merge reached its marker,
    /// which is what says the test may act before the merge goes on.
    fn wait_for(&self, phase: &str) {
        let marker: PathBuf = self.markers.join(format!("{phase}.reached"));
        let began: Instant = Instant::now();
        while !marker.exists() {
            assert!(began.elapsed() < MARKER_PATIENCE, "{phase} was not reached within {} seconds", MARKER_PATIENCE.as_secs());
            std::thread::sleep(MARKER_POLL);
        }
    }

    /// Let the merge go on and read everything it said.
    fn released(self, phase: &str) -> Output {
        std::fs::write(self.markers.join(format!("{phase}.release")), b"").expect("the release file");
        return self.child.wait_with_output().expect("the merge ends");
    }
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

/// What:
///  Both of the person's layers are back in one path,
///  each where it belongs.
/// Why:
///   A staged line and an unsaved line are two layers of one path,
///  written in two steps of one carry;
///  a kill between those steps loses the later one unless recovery writes it,
///  and a test that looks only at the file cannot tell.
fn assert_both_layers(fixture: &Fixture, repo: &Path, label: &str) {
    let worktree: String = text_at(repo, "a.txt").expect("a.txt is there");
    assert!(worktree.contains(FEATURE_THIRD), "{label}: the merge's line is in the file: {worktree:?}");
    assert!(worktree.contains(STAGED_LINE), "{label}: the staged line is back in the file: {worktree:?}");
    assert!(worktree.contains(UNSAVED_LINE), "{label}: the unsaved line is back in the file, not lost: {worktree:?}");
    let index: String = git_text(fixture, repo, &["show", ":a.txt"]);
    assert!(index.contains(FEATURE_THIRD), "{label}: the index holds the staged layer over the merge: {index:?}");
    assert!(index.contains(STAGED_LINE), "{label}: the staged line is still staged: {index:?}");
    assert!(!index.contains(UNSAVED_LINE), "{label}: the unsaved line is not staged: {index:?}");
    assert_eq!(status(fixture, repo), "MM a.txt\n", "{label}: one layer is staged and the other is unsaved");
}

/// A kill between the index step of the carry back and its first worktree file:
/// the next command writes the layers it had not written,
/// and the unsaved line is not lost.
#[test]
fn a_kill_between_the_index_step_and_the_file_keeps_every_layer() {
    let fixture: Fixture = fixture("carry-kill-between-layers");
    let repo: PathBuf = staged_and_unsaved(&fixture, "repo");
    killed_at(&fixture, repo.as_path(), "merge-carry-index");
    let recovered: Observed = after_the_kill(&fixture, repo.as_path());
    assert_merged(&fixture, repo.as_path(), "kill between the layers");
    // The bytes come first: a line that says the right thing about a file whose layer never came back is the worse failure.
    assert_both_layers(&fixture, repo.as_path(), "kill between the layers");
    assert!(!said(&recovered).contains("was written again while the merge ran"), "nothing changed under the next command, so it must not claim one did: {}", said(&recovered));
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A kill before the clear wrote the index:
/// the checkout is exactly what the person left,
/// both layers of it,
/// and the branch does not move.
#[test]
fn a_kill_before_the_clear_writes_anything_keeps_every_layer() {
    let fixture: Fixture = fixture("carry-kill-before-clear");
    let repo: PathBuf = staged_and_unsaved(&fixture, "repo");
    let before: Held = held(&fixture, repo.as_path());
    assert_eq!(before.person.as_deref(), Some(unsaved_text().as_str()), "the fixture holds both layers");
    killed_at(&fixture, repo.as_path(), "merge-intent");
    after_the_kill(&fixture, repo.as_path());
    assert_eq!(held(&fixture, repo.as_path()), before, "both layers are what the person left, and the branch did not move");
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A kill between two carried files:
/// the next command writes the file it had not written,
/// and both edits are there.
#[test]
fn a_kill_between_two_carried_files_keeps_every_layer() {
    let fixture: Fixture = fixture("carry-kill-between-files");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", on_main_a())).expect("the person's edit");
    std::fs::write(repo.join("gone.txt"), format!("gone base\n{} edited\n", PERSON_LINE)).expect("the person's edit of a file the merge deletes");
    killed_at_occurrence(&fixture, repo.as_path(), "merge-carry-file", Some(1));
    after_the_kill(&fixture, repo.as_path());
    assert_merged(&fixture, repo.as_path(), "kill between two carried files");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(FEATURE_THIRD) && after.contains(PERSON_LINE), "a.txt holds the merge and the person's edit: {after:?}");
    assert_eq!(
        text_at(repo.as_path(), "gone.txt").unwrap_or_default(),
        format!("gone base\n{} edited\n", PERSON_LINE),
        "the file the merge deletes keeps the person's bytes"
    );
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A kill after both carried files are written and before the record goes:
/// the next command finds the layers where they belong and only finishes the record.
#[test]
fn a_kill_after_the_carried_files_keeps_every_layer() {
    let fixture: Fixture = fixture("carry-kill-after-files");
    let repo: PathBuf = staged_and_unsaved(&fixture, "repo");
    killed_at(&fixture, repo.as_path(), "merge-carried-back");
    let recovered: Observed = after_the_kill(&fixture, repo.as_path());
    assert_merged(&fixture, repo.as_path(), "kill after the carried files");
    assert!(!said(&recovered).contains("was written again while the merge ran"), "nothing changed under the next command, so it must not claim one did: {}", said(&recovered));
    assert_both_layers(&fixture, repo.as_path(), "kill after the carried files");
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A save that lands after the clear wrote the index and before it wrote the file:
/// the person's bytes are kept,
/// the merge lands,
/// and the command says what became of the merge's version.
#[test]
fn a_save_between_the_index_step_and_the_file_is_kept() {
    let fixture: Fixture = fixture("carry-save-during-clear");
    let repo: PathBuf = staged_and_unsaved(&fixture, "repo");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-clear-index");
    let saved: String = format!("{}{SAVED_DURING_THE_MERGE}\n", staged_text());
    std::fs::write(repo.join("a.txt"), &saved).expect("the save that lands while the merge runs");
    let output: Output = paused.released("merge-clear-index");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "the merge exits 0 with the person's save kept: {said}");
    assert_merged(&fixture, repo.as_path(), "save during the clear");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(SAVED_DURING_THE_MERGE), "the save is kept rather than replaced: {after:?}");
    assert!(after.contains(STAGED_LINE), "the staged line is still in the file: {after:?}");
    assert!(after.contains(FEATURE_THIRD), "the merge's line reaches the file: {after:?}");
    assert!(said.contains("a.txt"), "the command says which file it is about: {said}");
    assert_eq!(status(&fixture, repo.as_path()), "MM a.txt\n", "one layer is staged and the other is unsaved: {said}");
    assert_eq!(records(repo.as_path()), "", "the record went");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "the captured bytes are not left on a ref nothing names");
    remove_settled(&fixture);
}

/// A save that keeps the file's length and its modification time is kept too:
/// the identity cannot see it,
/// and the bytes can.
#[test]
fn a_save_that_keeps_the_length_and_the_time_is_kept() {
    let fixture: Fixture = fixture("carry-save-same-identity");
    let repo: PathBuf = staged_and_unsaved(&fixture, "repo");
    let held: std::time::SystemTime = std::fs::metadata(repo.join("a.txt")).expect("a.txt").modified().expect("the modification time");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-clear-index");
    // The same length as the line it replaces, written over the same inode, with the time put back.
    let saved: String = unsaved_text().replace(UNSAVED_LINE, "the unsaved LINE");
    assert_eq!(saved.len(), unsaved_text().len(), "the control writes the same length");
    std::fs::write(repo.join("a.txt"), &saved).expect("the save that lands while the merge runs");
    std::fs::File::options()
        .write(true)
        .open(repo.join("a.txt"))
        .expect("a.txt")
        .set_modified(held)
        .expect("the modification time put back");
    let output: Output = paused.released("merge-clear-index");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "the merge exits 0 with the save kept: {said}");
    assert_merged(&fixture, repo.as_path(), "save with the same length and time");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains("the unsaved LINE"), "the bytes are what the control wrote, not the old line: {after:?}");
    assert!(after.contains(FEATURE_THIRD), "the merge's line reaches the file: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), "MM a.txt\n", "one layer is staged and the other is unsaved");
    remove_settled(&fixture);
}

/// What:
///  A path deleted between the branch move and the carry back:
/// this merge wrote nothing to it,
///  and the line says the path changed instead of claiming a file holds bytes that are not there.
/// Why:
///  A person may delete a file while a slow merge runs,
/// and the checkout then holds nothing at that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = pausedAt('merge-applied'); rm('a.txt'); const ran = paused.released('merge-applied');
/// expect(status()).toBe(' D a.txt\n'); expect(ran.stderr).toSay('changed while the merge ran'); expect(ran.stderr).not.toSay('the file holds');
/// ```
#[test]
fn a_path_deleted_while_the_merge_runs_is_named_as_changed() {
    let fixture: Fixture = fixture("carry-deleted-during");
    let repo: PathBuf = edited(&fixture, "repo");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-applied");
    std::fs::remove_file(repo.join("a.txt")).expect("the deletion that lands while the merge runs");
    let output: Output = paused.released("merge-applied");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "the merge exits 0: {said}");
    assert_merged(&fixture, repo.as_path(), "deletion during the merge");
    assert_eq!(text_at(repo.as_path(), "a.txt"), None, "the deletion stands");
    assert_eq!(status(&fixture, repo.as_path()), " D a.txt\n", "the checkout holds the deletion the person made");
    assert!(said.contains("changed while the merge ran"), "the line says the path changed: {said}");
    assert!(!said.contains("the file holds"), "the line claims no file the person's bytes are in: {said}");
    remove_settled(&fixture);
}

/// What:
///  A path the person deleted after a killed merge cleared it:
/// the next command leaves that deletion alone,
///  and says so without claiming a file holds their bytes.
/// Why:
///  The restore of a merge that did not land names every path it does not write,
/// and the person's deletion
///  is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// killedAt('merge-cleared'); rm('a.txt'); const ran = await wrapped(['status']);
/// expect(ran.stderr).toSay('changed after this merge captured it'); expect(ran.stderr).not.toSay('it holds your bytes');
/// ```
#[test]
fn a_kill_after_the_clear_names_a_path_deleted_since_the_capture() {
    let fixture: Fixture = fixture("carry-kill-cleared-deleted");
    let repo: PathBuf = edited(&fixture, "repo");
    killed_at(&fixture, repo.as_path(), "merge-cleared");
    std::fs::remove_file(repo.join("a.txt")).expect("the person's deletion after the clear");
    let observed: Observed = after_the_kill(&fixture, repo.as_path());
    assert_eq!(text_at(repo.as_path(), "a.txt"), None, "the deletion stands");
    let said: String = String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
    assert!(said.contains("changed after this merge captured it"), "the line says the path changed: {said}");
    assert!(!said.contains("it holds your bytes"), "the line claims no file that holds their bytes: {said}");
    remove_settled(&fixture);
}
