//! What:
//!  The two stable words a refusal is filed under without printing a sentence:
//!  the cause's own name,
//!  and the step whose read refused.
//! Why:
//!  A receipt,
//!  a control and `doctor` must be able to say why a removal stopped without
//!  printing a sentence,
//!  and one stable word per cause and per step is what lets a person and a
//!  control agree on which read refused
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Reading the mark").
//!  These are pure views of one cause,
//!  kept apart from the cause's fields and from its message,
//!  so a reader meets every word a record carries in one file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cause.causeName(); // 'clone-gone'
//! cause.step(); // 'origin-mark'
//! ```

use super::origin_refusal::OriginRefusal;

impl OriginRefusal {
    /// What:
    ///  The cause's own name,
    ///  one stable kebab-case word per cause.
    /// Why:
    ///  A receipt,
    ///  a control and `doctor` must be able to say why a removal stopped without printing a
    ///  sentence,
    ///  and the step word names the read rather than the reason.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal.causeName(); // 'clone-gone'
    /// ```
    pub fn cause_name(&self) -> &'static str {
        return match self {
            OriginRefusal::PlanIncomplete {
                ..
            } => "plan-incomplete",
            OriginRefusal::OriginMarkedBeforeTheUnmount {
                ..
            } => "origin-marked-before-the-unmount",
            OriginRefusal::OriginMarkedForDeferredDestroy {
                ..
            } => "origin-marked-for-deferred-destroy",
            OriginRefusal::MarkUnreadable {
                ..
            } => "mark-unreadable",
            OriginRefusal::CloneGone {
                ..
            } => "clone-gone",
            OriginRefusal::CloneIsNotAFilesystem {
                ..
            } => "clone-is-not-a-filesystem",
            OriginRefusal::CloneIsNotThePlannedOne {
                ..
            } => "clone-is-not-the-planned-one",
            OriginRefusal::CloneHasNoOrigin {
                ..
            } => "clone-has-no-origin",
            OriginRefusal::CloneOriginDiffers {
                ..
            } => "clone-origin-differs",
            OriginRefusal::OriginDatasetGone {
                ..
            } => "origin-dataset-gone",
            OriginRefusal::OriginDatasetIsNotAFilesystem {
                ..
            } => "origin-dataset-is-not-a-filesystem",
            OriginRefusal::OriginDatasetIsNotThePlannedOne {
                ..
            } => "origin-dataset-is-not-the-planned-one",
            OriginRefusal::OriginSnapshotGone {
                ..
            } => "origin-snapshot-gone",
            OriginRefusal::OriginIsNotASnapshot {
                ..
            } => "origin-is-not-a-snapshot",
            OriginRefusal::OriginSnapshotIsNotThePlannedOne {
                ..
            } => "origin-snapshot-is-not-the-planned-one",
            OriginRefusal::OriginSnapshotCreatetxgDiffers {
                ..
            } => "origin-snapshot-createtxg-differs",
            OriginRefusal::ReadFailed {
                ..
            } => "read-failed",
        };
    }

    /// What:
    ///  The word a receipt,
    ///  `doctor` and a control name this refusal's failed step with.
    /// Why:
    ///  One stable word per step is what lets a person and a control agree on which read
    ///  refused without comparing sentences.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal.step(); // 'origin-mark'
    /// ```
    pub fn step(&self) -> &'static str {
        return match self {
            OriginRefusal::PlanIncomplete {
                ..
            } => "plan",
            OriginRefusal::OriginMarkedBeforeTheUnmount {
                ..
            }
            | OriginRefusal::OriginMarkedForDeferredDestroy {
                ..
            }
            | OriginRefusal::MarkUnreadable {
                ..
            } => "origin-mark",
            OriginRefusal::CloneGone {
                ..
            } => "clone",
            OriginRefusal::CloneIsNotAFilesystem {
                ..
            } => "clone-type",
            OriginRefusal::CloneIsNotThePlannedOne {
                ..
            } => "clone-guid",
            OriginRefusal::CloneHasNoOrigin {
                ..
            }
            | OriginRefusal::CloneOriginDiffers {
                ..
            } => "clone-origin",
            OriginRefusal::OriginDatasetGone {
                ..
            } => "origin-dataset",
            OriginRefusal::OriginDatasetIsNotAFilesystem {
                ..
            } => "origin-dataset-type",
            OriginRefusal::OriginDatasetIsNotThePlannedOne {
                ..
            } => "origin-dataset-guid",
            OriginRefusal::OriginSnapshotGone {
                ..
            } => "origin-snapshot",
            OriginRefusal::OriginIsNotASnapshot {
                ..
            } => "origin-snapshot-type",
            OriginRefusal::OriginSnapshotIsNotThePlannedOne {
                ..
            } => "origin-snapshot-guid",
            OriginRefusal::OriginSnapshotCreatetxgDiffers {
                ..
            } => "origin-snapshot-createtxg",
            OriginRefusal::ReadFailed {
                ..
            } => "read",
        };
    }
}
