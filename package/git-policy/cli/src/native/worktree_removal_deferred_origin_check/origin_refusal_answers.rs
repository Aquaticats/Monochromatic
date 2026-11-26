//! What:
//!  The answers each cause gives without printing anything:
//!  the form of message it takes,
//!  the record rule that follows from its point,
//!  and the message itself.
//! Why:
//!  These are pure views of one cause,
//!  and keeping them apart from the cause's fields lets both files stay short enough to read whole
//!  while the enum stays one type a caller matches on.
//!  One cause's answers are split by what they produce:
//!  the words a record files it under and the object it names each sit in
//!  the file named for them,
//!  and this file holds the form,
//!  the record rule and the sentence a person reads,
//!  because those three answer one question: what a refusal leaves and says.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cause.form(); cause.whatItLeaves(); cause.message(history);
//! ```

use super::origin_refusal::OriginRefusal;
use super::origin_refusal_words::message;
use super::origin_refusals::{
    RefusalForm,
    RefusalPoint,
    WhatARefusalLeaves,
};
use super::removal_history::RemovalHistory;

impl OriginRefusal {
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
            OriginRefusal::OriginMarkedBeforeTheUnmount {
                ..
            } => RefusalForm::NothingTakenDown,
            OriginRefusal::MarkUnreadable {
                point,
                ..
            } => match point {
                RefusalPoint::BeforeTheUnmount => RefusalForm::NothingTakenDown,
                RefusalPoint::AtACheck => RefusalForm::PendingAfterTheUnmount,
            },
            OriginRefusal::OriginMarkedForDeferredDestroy {
                ..
            } => RefusalForm::PendingAfterTheUnmount,
            OriginRefusal::PlanIncomplete {
                ..
            }
            | OriginRefusal::CloneGone {
                ..
            }
            | OriginRefusal::CloneIsNotAFilesystem {
                ..
            }
            | OriginRefusal::CloneIsNotThePlannedOne {
                ..
            }
            | OriginRefusal::CloneHasNoOrigin {
                ..
            }
            | OriginRefusal::CloneOriginDiffers {
                ..
            }
            | OriginRefusal::OriginDatasetGone {
                ..
            }
            | OriginRefusal::OriginDatasetIsNotAFilesystem {
                ..
            }
            | OriginRefusal::OriginDatasetIsNotThePlannedOne {
                ..
            }
            | OriginRefusal::OriginSnapshotGone {
                ..
            }
            | OriginRefusal::OriginIsNotASnapshot {
                ..
            }
            | OriginRefusal::OriginSnapshotIsNotThePlannedOne {
                ..
            }
            | OriginRefusal::OriginSnapshotCreatetxgDiffers {
                ..
            }
            | OriginRefusal::ReadFailed {
                ..
            } => RefusalForm::NothingCompared,
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
            RefusalForm::PendingAfterTheUnmount | RefusalForm::NothingCompared => {
                WhatARefusalLeaves::PendingWithItsRecord
            }
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
    pub fn message(
        &self,
        history: &RemovalHistory,
    ) -> String {
        return message(self, history);
    }
}
