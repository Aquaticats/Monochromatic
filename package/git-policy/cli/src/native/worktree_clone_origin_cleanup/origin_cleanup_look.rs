//! What:
//!  The look every worktree command makes at the origin cleanup folder:
//!  one listing,
//!  one read per record,
//!  one attempt per record,
//!  and the lines the project decided it may print.
//! Why:
//!  Only `git worktree add`,
//!  `remove`,
//!  `list`,
//!  `prune` and `repair` destroy a leftover origin snapshot,
//!  printing one line,
//!  and `doctor` stays read-only
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "Which commands may destroy a leftover origin snapshot").
//!  The look takes no claim,
//!  reads no Git,
//!  waits for no person and changes no command's exit status,
//!  so two commands may attempt one record at once and the second removal of a record is the same
//!  completion.
//!  A record this reader refuses is named and left,
//!  a foreign entry is reported and never read,
//!  and a staging name is reported and left for `doctor`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = theOriginCleanupLook({ commonDir, reads, device, now });
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_attempt::{CleanupAttempt, CleanupOutcome, attempt_the_cleanup};
use crate::worktree_clone_origin_cleanup::origin_cleanup_lines::{line_of_a_block, line_of_a_destruction, line_of_a_refusal, the_reason_changed};
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::{CleanupListing, list_the_origin_cleanups, origin_cleanups_folder, read_the_cleanup_record};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusal;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use std::path::{Path, PathBuf};

/// What:
///  One look's inputs:
///  the repository,
///  the pool reads,
///  the device one destroy is sent to,
///  and the moment the attempt records.
/// Why:
///  Every command that runs the look answers these four from what it already has,
///  and a control answers them without opening this machine's device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Look = { commonDir; reads; device; now };
/// ```
pub struct OriginCleanupLook<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The pool reads the identity proofs make.
    pub reads: &'a dyn RemovalPoolReads,
    /// The device one destroy request is sent to.
    pub device: &'a dyn SnapshotDestroyDevice,
    /// When this attempt ran,
    /// as ISO 8601 with offset.
    pub now: &'a str,
}

/// What:
///  One look's answer:
///  the lines to print on standard error,
///  and what the look did,
///  so a control asserts the attempt and not only the words.
/// Why:
///  A look that printed nothing still has work to report:
///  a record that finished prints nothing when its snapshot was already gone,
///  and a control that could only read lines would call that a missing attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { lines: string[]; finished: number; blocked: number; refused: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LookAnswer {
    /// The lines the command prints on standard error,
    /// in the order it met them.
    pub lines: Vec<String>,
    /// How many records the look finished.
    pub finished: usize,
    /// How many attempts recorded a block.
    pub blocked: usize,
    /// How many records or entries this reader refused.
    pub refused: usize,
}

/// What:
///  What:
///   One look over one repository's cleanup folder.
/// Why:
///  The healthy case is one failed open and no line at all,
///  and every other case is one record's own outcome,
///  so one function serves all five commands that run it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theOriginCleanupLook({ commonDir, reads, device, now });
/// ```
pub fn the_origin_cleanup_look(look: &OriginCleanupLook<'_>) -> LookAnswer {
    let mut answer: LookAnswer = LookAnswer { lines: Vec::new(), finished: 0, blocked: 0, refused: 0 };
    let listing: Option<CleanupListing> = match list_the_origin_cleanups(look.common_dir) {
        Ok(found) => found,
        Err(refusal) => {
            answer.refused += 1;
            answer.lines.push(line_of_a_refusal("<the folder>", &refusal, origin_cleanups_folder(look.common_dir).as_path()));
            return answer;
        }
    };
    let Some(listing) = listing else { return answer };
    for name in &listing.foreign {
        answer.refused += 1;
        let refusal: CleanupRefusal = CleanupRefusal::at(
            crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusalCause::NotCanonical,
            "name",
            String::from(name),
        );
        answer.lines.push(line_of_a_refusal(name, &refusal, origin_cleanups_folder(look.common_dir).as_path()));
    }
    for name in &listing.staging {
        answer.refused += 1;
        let path: PathBuf = origin_cleanups_folder(look.common_dir).join(name);
        answer.lines.push(format!(
            "the origin cleanup folder holds the staging name {}, which a killed writer left; nothing was destroyed and `doctor` removes it once it is sure no writer is alive.",
            path.display()
        ));
    }
    let folder: CleanupFolder = match CleanupFolder::open_or_create(look.common_dir) {
        Ok(opened) => opened,
        Err(refusal) => {
            answer.refused += 1;
            answer.lines.push(line_of_a_refusal("<the folder>", &refusal, origin_cleanups_folder(look.common_dir).as_path()));
            return answer;
        }
    };
    for name in &listing.records {
        attempt_one(look, &folder, name, &mut answer);
    }
    return answer;
}

/// One record's attempt and the lines it prints.
fn attempt_one(look: &OriginCleanupLook<'_>, folder: &CleanupFolder, name: &str, answer: &mut LookAnswer) {
    let path: PathBuf = folder.path().join(name);
    let record: OriginCleanupRecord = match read_the_cleanup_record(look.common_dir, name) {
        Ok(read) => read,
        Err(refusal) => {
            answer.refused += 1;
            answer.lines.push(line_of_a_refusal(name, &refusal, path.as_path()));
            return;
        }
    };
    let outcome: CleanupOutcome = attempt_the_cleanup(&CleanupAttempt { record: &record, reads: look.reads, device: look.device, attempted_at: look.now });
    match outcome {
        CleanupOutcome::Finished { destroyed } => {
            if let Err(refusal) = folder.remove(name) {
                answer.refused += 1;
                answer.lines.push(line_of_a_refusal(name, &refusal, path.as_path()));
                return;
            }
            answer.finished += 1;
            if let Some(snapshot) = destroyed {
                answer.lines.push(line_of_a_destruction(snapshot.as_str()));
            }
        }
        CleanupOutcome::Blocked { record: next, detail } => {
            if let Err(refusal) = folder.publish(&next) {
                answer.refused += 1;
                answer.lines.push(line_of_a_refusal(name, &refusal, path.as_path()));
                return;
            }
            answer.blocked += 1;
            if the_reason_changed(Some(&record), &next) {
                answer.lines.push(line_of_a_block(&next, path.as_path(), detail.as_str()));
            }
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_look_tests.rs"]
mod tests;
