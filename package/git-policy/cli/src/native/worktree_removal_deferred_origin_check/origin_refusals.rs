//! What:
//!  Why a removal of a ZFS worktree clone stopped at one of the check's refusal points:
//!  each cause as its own typed value,
//!  the word a receipt files it under,
//!  the form of message it takes,
//!  and what it leaves of the removal's own record.
//! Why:
//!  Each cause sends a person to a different object or field,
//!  and a receipt,
//!  a control and `doctor` must be able to say which of the three points refused and why
//!  without parsing a sentence
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals").
//!  The record rule is the project's answer of 2026-11-22:
//!  a refusal before the unmount leaves nothing pending,
//!  and a refusal at either check keeps the removal pending with its intent and its plan
//!  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`).
//!  The check writes nothing of its own,
//!  so no refusal can leave half a record
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  veto V5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = checkPlannedOrigin(reader, planned);
//! if (verdict.kind === 'refused') console.error(verdict.cause.message(history));
//! ```


//region The objects and fields a cause names

/// What:
///  One field of the planned tuple a plan may fail to carry.
/// Why:
///  The refusal names the field in the words a person reads on the plan's own lines,
///  never in the plan grammar's hyphens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanField = 'clone-name' | 'clone-guid' | 'origin-dataset-name' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanField {
    /// The clone's full dataset name.
    CloneName,
    /// The clone's dataset GUID.
    CloneGuid,
    /// The origin dataset's full name.
    OriginDatasetName,
    /// The origin dataset's GUID.
    OriginDatasetGuid,
    /// The origin snapshot's full name.
    OriginSnapshotName,
    /// The origin snapshot's GUID.
    OriginSnapshotGuid,
    /// The transaction group the origin snapshot was created in.
    OriginSnapshotCreatetxg,
}

impl PlanField {
    /// The field in the words a person reads.
    pub fn words(self) -> &'static str {
        return match self {
            PlanField::CloneName => "the clone's dataset name",
            PlanField::CloneGuid => "the clone's dataset GUID",
            PlanField::OriginDatasetName => "the origin dataset's name",
            PlanField::OriginDatasetGuid => "the origin dataset's GUID",
            PlanField::OriginSnapshotName => "the origin snapshot's name",
            PlanField::OriginSnapshotGuid => "the origin snapshot's GUID",
            PlanField::OriginSnapshotCreatetxg => "the transaction group the origin snapshot was created in",
        };
    }
}

/// What:
///  Which of the three reads a failed one was.
/// Why:
///  A read that failed for a reason other than "not there" is not a gone object,
///  and the message says which of the three objects it could not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JudgedObject = 'clone' | 'origin-dataset' | 'origin-snapshot';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JudgedObject {
    /// The clone dataset.
    Clone,
    /// The origin dataset.
    OriginDataset,
    /// The origin snapshot.
    OriginSnapshot,
}

impl JudgedObject {
    /// The object in the words a person reads.
    pub fn words(self) -> &'static str {
        return match self {
            JudgedObject::Clone => "the planned clone",
            JudgedObject::OriginDataset => "the origin dataset",
            JudgedObject::OriginSnapshot => "the origin snapshot",
        };
    }
}

/// What:
///  Which refusal point judged the mark.
/// Why:
///  One cause names an unreadable mark at any point,
///  and the state sentence that is true differs:
///  before the unmount nothing has been taken down,
///  while at a check the clone is unmounted and the removal stays pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefusalPoint = 'before-the-unmount' | 'at-a-check';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalPoint {
    /// The look inside the lineage proof, before anything is taken down.
    BeforeTheUnmount,
    /// One of the two checks, after the verified unmount.
    AtACheck,
}

//endregion

//region The forms a refusal takes and what it leaves

/// What:
///  What a refusal's message may claim,
///  decided by the point that refused.
/// Why:
///  A form claims only what its refusal point read,
///  so a message never says "unmounted" where the worktree may still be mounted,
///  and never says that nothing is pending where the removal stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefusalForm = 'nothing-taken-down' | 'pending-after-the-unmount' | 'nothing-compared';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalForm {
    /// Nothing was taken down, nothing is pending and there is nothing to cancel.
    NothingTakenDown,
    /// The clone is unmounted, the removal stays pending, and the cancel command is the way back.
    PendingAfterTheUnmount,
    /// The check sent no request and could not compare a planned object.
    NothingCompared,
}

impl RefusalForm {
    /// The form's own name,
    /// for a control to compare against.
    pub fn name(self) -> &'static str {
        return match self {
            RefusalForm::NothingTakenDown => "nothing-taken-down",
            RefusalForm::PendingAfterTheUnmount => "pending-after-the-unmount",
            RefusalForm::NothingCompared => "nothing-compared",
        };
    }
}

/// What:
///  Whether a refusal keeps the removal's record,
///  which the project decided for the two checks and for the look.
/// Why:
///  The two records are the clone's `unmount` intent and the removal's plan,
///  and no refusal rewrites the first,
///  reattaches the clone,
///  unlinks the second or removes a registration
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  "What a refusal leaves, and what the record keeps").
///  The answer is a value rather than a sentence so the cancel command,
///  `doctor` and a receipt read one rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatARefusalLeaves = { kind: 'nothing-pending' } | { kind: 'pending-with-its-record' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatARefusalLeaves {
    /// The removal stopped before it took anything down:
    /// nothing is pending and there is nothing to cancel.
    NothingPending,
    /// The removal stays pending,
    /// the clone stays unmounted,
    /// and its intent and plan stand.
    PendingWithItsRecord,
}

impl WhatARefusalLeaves {
    /// What:
    ///  What this state means for the person,
    ///  naming the slot's cancel command where one is the way back.
    /// Why:
    ///  The same sentence serves a refusal's message,
    ///  a receipt and the words a later command prints.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// state.sentence('fix-flaky-test');
    /// ```
    pub fn sentence(&self, slot: &str) -> String {
        return match self {
            WhatARefusalLeaves::NothingPending => String::from("nothing is pending and there is nothing to cancel"),
            WhatARefusalLeaves::PendingWithItsRecord => format!(
                "the removal stays pending with its own record and the clone stays unmounted; `cancel-worktree-removal {slot}` is the way back for a removal that is not forced, and it reports any snapshot already destroyed"
            ),
        };
    }
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_refusals_tests.rs"]
mod tests;
