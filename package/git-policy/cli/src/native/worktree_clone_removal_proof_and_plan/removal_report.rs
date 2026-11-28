//! What:
//!  The report one finished removal prints:
//!  what it destroyed,
//!  what the checkout kept,
//!  what the origin cleanup did,
//!  what the settle removed,
//!  and what could not be finished.
//! Why:
//!  The design requires the removal to print what disappeared and what remains and to keep
//!  per-name results,
//!  because the printed report is the only trace of a destruction that outlives the removal
//!  beyond its records
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Which resume path a removal takes";
//!  `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "What trace of a destruction outlives the removal").
//!  The report is built from the plan's own lines and the cleanup's own record,
//!  so it cannot claim a snapshot the plan does not record,
//!  and it names the losses a person cannot undo.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(done.lines().join('\n'));
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_records_end::RecordsGone;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::SettleAnswer;
use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::ClaimEnd;
use crate::worktree_clone_snapshot_batch::snapshot_batch_outcome::BatchOutcome;
use std::path::PathBuf;

/// What:
///  What the removal's own origin attempt did:
///  finished the cleanup,
///  left it blocked,
///  or found the record in a state it must report rather than act on.
/// Why:
///  The settle never waits for the origin,
///  so a person has to read what was left and where the record stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OriginCleanupDone = { kind: 'finished'; snapshot?; line? } | { kind: 'blocked'; ... } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OriginCleanupDone {
    /// The record was gone before this run reached it.
    NoRecord {
        /// Why no record was found,
        /// in the run's own words.
        said: String,
    },
    /// The record could not be read,
    /// so nothing was destroyed on its word.
    Unreadable {
        /// The record's file.
        path: PathBuf,
        /// What the read answered.
        said: String,
    },
    /// The recorded snapshot is gone and the record was removed.
    Finished {
        /// The snapshot this run destroyed,
        /// or nothing when it was already gone.
        snapshot: Option<String>,
        /// The line the project decided prints when a snapshot is destroyed.
        line: Option<String>,
    },
    /// The record stands blocked with its reason.
    Blocked {
        /// The snapshot the record names.
        snapshot: String,
        /// Why it is blocked.
        reason: BlockReason,
        /// The detail a person acts on.
        detail: String,
        /// The line the record printed,
        /// or nothing while its reason did not change.
        line: Option<String>,
    },
}

/// What:
///  One finished removal,
///  as the value its report is built from.
/// Why:
///  The names destroyed and the names remaining travel together with the plan's own path,
///  so a report cannot be printed without saying where the witness stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Done = { slot; dataset; destroyed; bookmarks; remaining; head; holderReadsUnfinished; origin; settle; records; planPath; requests };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheRemovalDone {
    /// The slot the removal removed.
    pub slot: String,
    /// The clone dataset that is gone.
    pub dataset: String,
    /// Every snapshot the plan records as destroyed,
    /// in the plan's order.
    pub destroyed: Vec<String>,
    /// Every bookmark that went with the head.
    pub bookmarks: Vec<String>,
    /// Every name the plan does not record as destroyed,
    /// which is empty on a finished removal.
    pub remaining: Vec<String>,
    /// The head's final word.
    pub head: HeadWord,
    /// How many namespace reads the fresh scans could not complete.
    pub holder_reads_unfinished: usize,
    /// What the origin attempt did.
    pub origin: OriginCleanupDone,
    /// What the settle did.
    pub settle: SettleAnswer,
    /// What the removal's own records were.
    pub records: RecordsGone,
    /// What the slot's per-slot claim did at the order's last step.
    pub claim: ClaimEnd,
    /// The plan's file,
    /// which a finished removal removed.
    pub plan_path: PathBuf,
    /// What this run did only because of a force,
    /// with the line a person reads about each act.
    pub forced: crate::worktree_clone_removal_proof_and_plan::removal_forced_acts::ForcedActs,
    /// Every snapshot request this run sent,
    /// with the kernel's per-name answers.
    pub requests: Vec<BatchOutcome>,
}

impl TheRemovalDone {
    /// What:
    ///  The lines a person reads.
    /// Why:
    ///  The first line says what is gone for good,
    ///  the middle lines say what the clone's history was,
    ///  and the last lines name what a person still has to act on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lines = done.lines();
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "the ZFS worktree clone at `{}` (dataset `{}`) is destroyed, with {} of the snapshot(s) this removal recorded.",
            self.slot,
            self.dataset,
            self.destroyed.len()
        ));
        for name in &self.destroyed {
            lines.push(format!("  destroyed snapshot {name}"));
        }
        for name in &self.remaining {
            lines.push(format!("  NOT destroyed snapshot {name}"));
        }
        if !self.bookmarks.is_empty() {
            lines.push(format!(
                "  {} bookmark(s) went with the clone's head and cannot be restored: {}",
                self.bookmarks.len(),
                self.bookmarks.join(", ")
            ));
        }
        lines.push(match self.head {
            HeadWord::Gone => {
                String::from("  the clone's head destroy answered and its absence is proved.")
            }
            HeadWord::Refused => {
                String::from("  the clone's head destroy was refused, so the clone still stands.")
            }
            HeadWord::Unknown => {
                String::from("  the clone's head destroy's outcome cannot be attributed.")
            }
            HeadWord::Requested | HeadWord::Present => {
                String::from("  no head destroy was proved for this clone.")
            }
        });
        if self.holder_reads_unfinished > 0 {
            lines.push(format!(
                "  {} namespace read(s) of the fresh scans could not be completed; the destroy-anyway decision accepts that residual, and a scan that saw no holder never proves there is none.",
                self.holder_reads_unfinished
            ));
        }
        for line in &self.settle.lines {
            lines.push(format!("  {line}"));
        }
        lines.push(format!(
            "  the Git registration and the slot's pointer {} removed, the leftover underlay folder {} removed, and the managed lock {} released.",
            if self.settle.removed_the_registration { "were" } else { "were not" },
            if self.settle.removed_the_underlay { "was" } else { "was not" },
            if self.settle.released_the_lock { "was" } else { "was not" }
        ));
        lines.push(match &self.origin {
            OriginCleanupDone::Finished { line: Some(line), .. } => format!("  {line}"),
            OriginCleanupDone::Finished { line: None, .. } => String::from("  the origin cleanup's record is gone, and the snapshot it named was already absent."),
            OriginCleanupDone::Blocked { snapshot, reason, detail, line } => match line {
                Some(text) => format!("  {text}"),
                None => format!("  the origin cleanup of `{snapshot}` is {} ({detail}) and its reason did not change this run.", reason.clause()),
            },
            OriginCleanupDone::NoRecord { said } => format!("  no origin cleanup record was attempted ({said})."),
            OriginCleanupDone::Unreadable { path, said } => format!("  the origin cleanup record at {} could not be read ({said}), so nothing was destroyed on its word.", path.display()),
        });
        for line in &self.forced.lines {
            lines.push(format!("  {line}"));
        }
        if self.records.plan_removed {
            lines.push(String::from("  the removal's plan, its binding and both inventories are removed, which frees the slot for the next add."));
        } else {
            lines.push(format!(
                "  the removal's plan still stands at {}; nothing was destroyed twice.",
                self.plan_path.display()
            ));
        }
        lines.push(match &self.claim {
            ClaimEnd::Released => format!("  the slot's per-slot claim is released, so nothing of this removal keeps another command off `{}`.", self.slot),
            ClaimEnd::RefusedToRelease { said } => format!(
                "  {said}, so another command on `{}` is refused until this process ends, when the claim stops counting as alive and the next command takes the next generation.",
                self.slot
            ),
            ClaimEnd::Held => format!("  the removal still holds the slot's per-slot claim: `{}` is not free until its last step runs.", self.slot),
        });
        let refused: usize = self
            .requests
            .iter()
            .flat_map(|outcome| return outcome.names())
            .filter(|submitted| return submitted.outcome != crate::worktree_clone_snapshot_batch::snapshot_name_outcome::NameOutcome::Destroyed)
            .count();
        if refused > 0 {
            lines.push(format!("  {refused} snapshot name(s) of this run's requests were not destroyed; the plan records each answer."));
        }
        return lines;
    }

    /// The lines as one block a caller prints as it stands.
    pub fn text(&self) -> String {
        return self.lines().join("\n");
    }

    /// Whether every name the plan recorded is proved destroyed.
    pub fn is_finished(&self) -> bool {
        return self.head == HeadWord::Gone
            && self.remaining.is_empty()
            && self.destroyed.iter().all(|name| return !name.is_empty());
    }
}

/// The word one snapshot outcome prints,
/// which the report reuses rather than inventing a second vocabulary.
pub fn the_outcome_word(outcome: SnapshotOutcome) -> &'static str {
    return outcome.word();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_report_tests.rs"]
mod tests;
