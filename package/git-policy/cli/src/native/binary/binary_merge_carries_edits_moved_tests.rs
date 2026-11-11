//! What:
//!  An isolated merge that has landed and is still carrying the person's work,
//!  while another command lands a commit on
//!  the same branch.
//! Why:
//!  Dozens of agents run Git in one worktree at once (`SPEC.md`,
//! "Merges and commits never fail"),
//!  and the window between the merge's own update and its carry is long enough for one of them to land a commit
//!  in it.
//!  The carry then finds the merge's paths holding that commit's content,
//!  and the record and the ref that keep every
//!  byte of the person's work are the only copy left of the older layer.
//!  `never discard user data` says the bytes stay, and the
//!  answer a recovery already gives for a killed merge is the one these controls demand of a running command:
//!  the person's layers are merged with the branch as it is now,
//!  the later commit stands,
//!  and the command
//!  exits 0.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paused = pausedAt('merge-applied'); commit('another session'); const ran = paused.released();
//! expect(ran.code).toBe(0); expect(read('a.txt')).toContain('the person line'); expect(head()).toBe(landed);
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::merge_carries_edits::{FEATURE_THIRD, PERSON_LINE, diverged, on_main_a, records, said, side_refs, status, text_at};
use super::merge_carries_edits_kill::{Paused, edited, paused_at};
use super::support::{Fixture, Observed, fixture, git, observe};
use std::path::{Path, PathBuf};
use std::process::Output;

/// The line another session commits to the very path this merge carries.
const LANDED_LINE: &str = "another session's committed line";

/// The line the agent branch appends to the same path,
/// which the second merge lands while the first one carries.
const AGENT_LINE: &str = "another agent's committed line";

/// What:
///  The promises a merge that met a commit landing under it keeps,
/// read from the checkout alone:
///  the branch holds the commit that landed,
///  and nothing of the merge's own bookkeeping outlives the person's bytes.
/// Why:
///   The merge is the parent of the branch here,
/// so the two-parent check of a finished merge is the wrong one:
///  what must hold is that the other command's commit was not rewritten,
/// and that no record or ref is left behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(head()).toBe(landed); expect(records()).toBe(''); expect(sideRefs()).toBe('');
/// ```
fn assert_the_later_commit_stands(fixture: &Fixture, repo: &Path, landed: &str, label: &str) {
    assert_eq!(git_text(fixture, repo, &["rev-parse", "HEAD"]), landed, "{label}: the branch holds the commit that landed");
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "{label}: no MERGE_HEAD is left");
    assert_eq!(git_text(fixture, repo, &["--no-optional-locks", "ls-files", "--unmerged"]), "", "{label}: no unmerged entry");
    assert_eq!(records(repo), "", "{label}: no merge record is left");
    assert_eq!(side_refs(fixture, repo), "", "{label}: no side ref is left");
}

/// A commit another session lands on this very branch while the merge is between its own update and its carry:
/// the later commit stands,
/// the person's unsaved line comes back onto the branch as it is now,
/// and the merge exits 0
/// with no record and no side ref left.
#[test]
fn an_edit_survives_a_commit_that_lands_while_the_carry_runs() {
    let fixture: Fixture = fixture("carry-live-commit");
    let repo: PathBuf = edited(&fixture, "repo");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-applied");
    // Another session commits a change to the very path this merge carries, on this very branch.
    let merged: String = text_at(repo.as_path(), "a.txt").expect("a.txt holds the merge");
    assert!(merged.contains(FEATURE_THIRD), "the merge landed before the other commit: {merged:?}");
    assert!(!merged.contains(PERSON_LINE), "the person's line is in the capture, not in the file: {merged:?}");
    std::fs::write(repo.join("a.txt"), format!("{merged}{LANDED_LINE}\n")).expect("the other session's edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=another session's commit"]);
    let landed: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let output: Output = paused.released("merge-applied");
    let words: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "the merge exits 0 with the other session's commit standing: {words}");
    assert_the_later_commit_stands(&fixture, repo.as_path(), landed.as_str(), "a commit landing during the carry");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(LANDED_LINE), "the other session's line stands in the file: {after:?}");
    assert!(after.contains(FEATURE_THIRD), "the merge's own line is in the file: {after:?}");
    assert!(after.contains(PERSON_LINE), "the person's earlier line is merged onto the branch as it is now: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the person's line is still unsaved: {words}");
    assert!(words.contains("moved to"), "the command says the branch moved under the carry: {words}");
    remove_settled(&fixture);
}

/// The interleaving the audit reported:
/// a second wrapped merge lands on the branch while the first merge carries,
/// both commands exit 0,
/// the
///  second agent's commit and line survive,
/// and the person's earlier unsaved line is merged onto that new head
///  rather than dropped.
#[test]
fn an_edit_survives_a_second_merge_that_lands_while_the_carry_runs() {
    let fixture: Fixture = fixture("carry-live-second-merge");
    let repo: PathBuf = diverged(&fixture, "repo");
    // The branch a second agent merged while the first merge carries: it appends a line to the very same file.
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "agent"]);
    std::fs::write(repo.join("a.txt"), format!("{}{AGENT_LINE}\n", on_main_a())).expect("the agent branch's edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=agent branch work"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", on_main_a())).expect("the person's unsaved line");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-applied");
    let second: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "agent"]), b"");
    assert_eq!(second.code, Some(0), "the second merge lands while the first carries: {}", said(&second));
    let landed: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let output: Output = paused.released("merge-applied");
    let words: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "the first merge exits 0: {words}");
    assert_the_later_commit_stands(&fixture, repo.as_path(), landed.as_str(), "a second merge landing during the carry");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(AGENT_LINE), "the second agent's line stands in the file: {after:?}");
    assert!(after.contains(PERSON_LINE), "the person's earlier line is merged onto the new head: {after:?}");
    assert!(after.contains(FEATURE_THIRD), "the first merge's line is in the file too: {after:?}");
    let head: String = git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]);
    assert!(head.contains(AGENT_LINE), "the new head holds the second agent's line: {head:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the person's line is still unsaved: {words}");
    remove_settled(&fixture);
}
