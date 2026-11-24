//! What:
//!  The answers each cause gives without printing anything:
//!  the step word a receipt files it under,
//!  the form of message it takes,
//!  the record rule that follows from its point,
//!  the clone and slot it names,
//!  and the message itself.
//! Why:
//!  These are pure views of one cause,
//!  and keeping them apart from the cause's fields lets both files stay short enough to read whole
//!  while the enum stays one type a caller matches on.
//!  One `impl` block holds every method, so no reader has to look in two files for one cause's
//!  answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cause.step(); cause.form(); cause.whatItLeaves(); cause.message(history);
//! ```

use super::origin_refusal::OriginRefusal;
use super::origin_refusal_words::message;
use super::origin_refusals::{RefusalForm, RefusalPoint, WhatARefusalLeaves};
use super::planned_origin::PlannedOrigin;
use super::removal_history::RemovalHistory;

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
            OriginRefusal::PlanIncomplete { .. } => "plan-incomplete",
            OriginRefusal::OriginMarkedBeforeTheUnmount { .. } => "origin-marked-before-the-unmount",
            OriginRefusal::OriginMarkedForDeferredDestroy { .. } => "origin-marked-for-deferred-destroy",
            OriginRefusal::MarkUnreadable { .. } => "mark-unreadable",
            OriginRefusal::CloneGone { .. } => "clone-gone",
            OriginRefusal::CloneIsNotAFilesystem { .. } => "clone-is-not-a-filesystem",
            OriginRefusal::CloneIsNotThePlannedOne { .. } => "clone-is-not-the-planned-one",
            OriginRefusal::CloneHasNoOrigin { .. } => "clone-has-no-origin",
            OriginRefusal::CloneOriginDiffers { .. } => "clone-origin-differs",
            OriginRefusal::OriginDatasetGone { .. } => "origin-dataset-gone",
            OriginRefusal::OriginDatasetIsNotAFilesystem { .. } => "origin-dataset-is-not-a-filesystem",
            OriginRefusal::OriginDatasetIsNotThePlannedOne { .. } => "origin-dataset-is-not-the-planned-one",
            OriginRefusal::OriginSnapshotGone { .. } => "origin-snapshot-gone",
            OriginRefusal::OriginIsNotASnapshot { .. } => "origin-is-not-a-snapshot",
            OriginRefusal::OriginSnapshotIsNotThePlannedOne { .. } => "origin-snapshot-is-not-the-planned-one",
            OriginRefusal::OriginSnapshotCreatetxgDiffers { .. } => "origin-snapshot-createtxg-differs",
            OriginRefusal::ReadFailed { .. } => "read-failed",
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
            OriginRefusal::PlanIncomplete { .. } => "plan",
            OriginRefusal::OriginMarkedBeforeTheUnmount { .. }
            | OriginRefusal::OriginMarkedForDeferredDestroy { .. }
            | OriginRefusal::MarkUnreadable { .. } => "origin-mark",
            OriginRefusal::CloneGone { .. } => "clone",
            OriginRefusal::CloneIsNotAFilesystem { .. } => "clone-type",
            OriginRefusal::CloneIsNotThePlannedOne { .. } => "clone-guid",
            OriginRefusal::CloneHasNoOrigin { .. } | OriginRefusal::CloneOriginDiffers { .. } => "clone-origin",
            OriginRefusal::OriginDatasetGone { .. } => "origin-dataset",
            OriginRefusal::OriginDatasetIsNotAFilesystem { .. } => "origin-dataset-type",
            OriginRefusal::OriginDatasetIsNotThePlannedOne { .. } => "origin-dataset-guid",
            OriginRefusal::OriginSnapshotGone { .. } => "origin-snapshot",
            OriginRefusal::OriginIsNotASnapshot { .. } => "origin-snapshot-type",
            OriginRefusal::OriginSnapshotIsNotThePlannedOne { .. } => "origin-snapshot-guid",
            OriginRefusal::OriginSnapshotCreatetxgDiffers { .. } => "origin-snapshot-createtxg",
            OriginRefusal::ReadFailed { .. } => "read",
        };
    }

    /// What:
    ///  Which form of message is true for this cause at the point that raised it.
    /// Why:
    ///  One form per state keeps a message from claiming a state its point did not read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal.form(); // 'pending-after-the-unmount'
    /// ```
    pub fn form(&self) -> RefusalForm {
        return match self {
            OriginRefusal::OriginMarkedBeforeTheUnmount { .. } => RefusalForm::NothingTakenDown,
            OriginRefusal::MarkUnreadable { point, .. } => match point {
                RefusalPoint::BeforeTheUnmount => RefusalForm::NothingTakenDown,
                RefusalPoint::AtACheck => RefusalForm::PendingAfterTheUnmount,
            },
            OriginRefusal::OriginMarkedForDeferredDestroy { .. } => RefusalForm::PendingAfterTheUnmount,
            OriginRefusal::PlanIncomplete { .. }
            | OriginRefusal::CloneGone { .. }
            | OriginRefusal::CloneIsNotAFilesystem { .. }
            | OriginRefusal::CloneIsNotThePlannedOne { .. }
            | OriginRefusal::CloneHasNoOrigin { .. }
            | OriginRefusal::CloneOriginDiffers { .. }
            | OriginRefusal::OriginDatasetGone { .. }
            | OriginRefusal::OriginDatasetIsNotAFilesystem { .. }
            | OriginRefusal::OriginDatasetIsNotThePlannedOne { .. }
            | OriginRefusal::OriginSnapshotGone { .. }
            | OriginRefusal::OriginIsNotASnapshot { .. }
            | OriginRefusal::OriginSnapshotIsNotThePlannedOne { .. }
            | OriginRefusal::OriginSnapshotCreatetxgDiffers { .. }
            | OriginRefusal::ReadFailed { .. } => RefusalForm::NothingCompared,
        };
    }

    /// What:
    ///  Whether this refusal keeps the removal's intent and plan.
    /// Why:
    ///  The project decided that a refusal before the unmount leaves nothing pending and that
    ///  a refusal at either check keeps the record,
    ///  and one rule answers it for every cause of its point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal.whatItLeaves(); // 'pending-with-its-record'
    /// ```
    pub fn what_it_leaves(&self) -> WhatARefusalLeaves {
        return match self.form() {
            RefusalForm::NothingTakenDown => WhatARefusalLeaves::NothingPending,
            RefusalForm::PendingAfterTheUnmount | RefusalForm::NothingCompared => WhatARefusalLeaves::PendingWithItsRecord,
        };
    }

    /// The clone's full dataset name,
    /// where the cause knows it.
    pub fn clone_name(&self) -> Option<&str> {
        return match self {
            OriginRefusal::PlanIncomplete { .. } => None,
            OriginRefusal::OriginMarkedBeforeTheUnmount { clone_name, .. }
            | OriginRefusal::MarkUnreadable { clone_name, .. }
            | OriginRefusal::CloneGone { planned: PlannedOrigin { clone_name, .. } }
            | OriginRefusal::CloneIsNotAFilesystem { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::CloneIsNotThePlannedOne { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::CloneHasNoOrigin { planned: PlannedOrigin { clone_name, .. } }
            | OriginRefusal::CloneOriginDiffers { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::OriginDatasetGone { planned: PlannedOrigin { clone_name, .. } }
            | OriginRefusal::OriginDatasetIsNotAFilesystem { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::OriginDatasetIsNotThePlannedOne { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::OriginSnapshotGone { planned: PlannedOrigin { clone_name, .. } }
            | OriginRefusal::OriginIsNotASnapshot { planned: PlannedOrigin { clone_name, .. } }
            | OriginRefusal::OriginSnapshotIsNotThePlannedOne { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::OriginSnapshotCreatetxgDiffers { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::OriginMarkedForDeferredDestroy { planned: PlannedOrigin { clone_name, .. }, .. }
            | OriginRefusal::ReadFailed { planned: PlannedOrigin { clone_name, .. }, .. } => Some(clone_name.as_str()),
        };
    }

    /// The slot the removal was asked for.
    pub fn slot(&self) -> &str {
        return match self {
            OriginRefusal::PlanIncomplete { slot, .. } | OriginRefusal::OriginMarkedBeforeTheUnmount { slot, .. } | OriginRefusal::MarkUnreadable { slot, .. } => slot.as_str(),
            OriginRefusal::OriginMarkedForDeferredDestroy { planned, .. }
            | OriginRefusal::CloneGone { planned }
            | OriginRefusal::CloneIsNotAFilesystem { planned, .. }
            | OriginRefusal::CloneIsNotThePlannedOne { planned, .. }
            | OriginRefusal::CloneHasNoOrigin { planned }
            | OriginRefusal::CloneOriginDiffers { planned, .. }
            | OriginRefusal::OriginDatasetGone { planned }
            | OriginRefusal::OriginDatasetIsNotAFilesystem { planned, .. }
            | OriginRefusal::OriginDatasetIsNotThePlannedOne { planned, .. }
            | OriginRefusal::OriginSnapshotGone { planned }
            | OriginRefusal::OriginIsNotASnapshot { planned }
            | OriginRefusal::OriginSnapshotIsNotThePlannedOne { planned, .. }
            | OriginRefusal::OriginSnapshotCreatetxgDiffers { planned, .. }
            | OriginRefusal::ReadFailed { planned, .. } => planned.slot.as_str(),
        };
    }

    /// What:
    ///  The sentence a person reads for this refusal.
    /// Why:
    ///  A refusal is the answer to the attempt that met it,
    ///  so every attempt prints it,
    ///  and the history of what the plan records comes from the caller rather than from the
    ///  call site.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// console.error(refusal.message(RemovalHistory.ofDestroyed(plan.destroyed)));
    /// ```
    pub fn message(&self, history: &RemovalHistory) -> String {
        return message(self, history);
    }
}
