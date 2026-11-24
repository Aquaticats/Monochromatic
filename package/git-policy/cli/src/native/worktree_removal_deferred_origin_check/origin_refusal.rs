//! What:
//!  Why a removal of a ZFS worktree clone stopped at one of the check's refusal points:
//!  each cause as a typed value,
//!  with the fields its message names.
//! Why:
//!  Each cause sends a person to a different object or field,
//!  and a receipt,
//!  a control and `doctor` must be able to say which of the three points refused and why
//!  without parsing a sentence
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals").
//!  The cause carries the planned tuple rather than a sentence built where the object was
//!  read,
//!  so the words live in one file and the same cause reads the same at any call site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = checkPlannedOrigin(reader, planned);
//! if (verdict.kind === 'refused') console.error(verdict.cause.message(history));
//! ```

use super::origin_mark::UserHolds;
use super::origin_refusals::{JudgedObject, PlanField, RefusalPoint};
use super::planned_origin::PlannedOrigin;
use std::path::PathBuf;

/// What:
///  One reason a removal does not go on,
///  as the check and the look report it.
/// Why:
///  The verdict carries the cause itself rather than a message,
///  so a caller files a receipt under the cause's step,
///  `doctor` names the same state,
///  and only a person-facing print renders words.
///  The three mark causes name one state at the three points,
///  and the identity causes are form C's,
///  because what they could not compare is their own subject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OriginRefusal = { kind: 'clone-gone'; planned } | { kind: 'origin-marked-for-deferred-destroy'; planned; clones; holds } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OriginRefusal {
    /// The removal's plan carries no value for one field the check compares.
    PlanIncomplete {
        /// The slot the removal was asked for.
        slot: String,
        /// The field the plan does not carry.
        field: PlanField,
        /// The plan file the fields were read from.
        plan_home: PathBuf,
    },
    /// The look inside the lineage proof found the origin snapshot marked,
    /// before anything was taken down.
    OriginMarkedBeforeTheUnmount {
        /// The clone's full dataset name.
        clone_name: String,
        /// The slot the removal was asked for.
        slot: String,
        /// The origin snapshot's full name.
        origin_snapshot: String,
        /// How many clones of that snapshot the reply reported.
        clones: u64,
        /// How many user holds it counted.
        holds: UserHolds,
    },
    /// One of the two checks found the origin snapshot marked,
    /// after the verified unmount.
    OriginMarkedForDeferredDestroy {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// How many clones of the planned origin snapshot the reply reported.
        clones: u64,
        /// How many user holds it counted.
        holds: UserHolds,
    },
    /// The mark could not be read,
    /// which is never the same as off,
    /// at the point that met it.
    MarkUnreadable {
        /// The clone's full dataset name.
        clone_name: String,
        /// The slot the removal was asked for.
        slot: String,
        /// The origin snapshot's full name.
        origin_snapshot: String,
        /// The point whose read could not judge the mark.
        point: RefusalPoint,
        /// What the reply held instead of an off or on value.
        said: String,
    },
    /// The kernel answered that the planned clone is not there.
    CloneGone {
        /// The identities this removal planned.
        planned: PlannedOrigin,
    },
    /// The object at the clone's name is not a filesystem,
    /// or is a snapshot.
    CloneIsNotAFilesystem {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The object-set type the kernel reported.
        found: u32,
        /// Whether the kernel called the object a snapshot.
        snapshot: bool,
    },
    /// The dataset at the clone's name reports another GUID than the planned one.
    CloneIsNotThePlannedOne {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The GUID the kernel reported.
        live: u64,
    },
    /// The clone reports no origin at all.
    CloneHasNoOrigin {
        /// The identities this removal planned.
        planned: PlannedOrigin,
    },
    /// The clone's live origin is not the snapshot this removal planned.
    CloneOriginDiffers {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The origin the kernel reported.
        live: String,
    },
    /// The kernel answered that the planned origin dataset is not there.
    OriginDatasetGone {
        /// The identities this removal planned.
        planned: PlannedOrigin,
    },
    /// The object at the origin dataset's name is not a filesystem,
    /// or is a snapshot.
    OriginDatasetIsNotAFilesystem {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The object-set type the kernel reported.
        found: u32,
        /// Whether the kernel called the object a snapshot.
        snapshot: bool,
    },
    /// The dataset at the origin's name reports another GUID than the planned one.
    OriginDatasetIsNotThePlannedOne {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The GUID the kernel reported.
        live: u64,
    },
    /// The kernel answered that the planned origin snapshot is not there.
    OriginSnapshotGone {
        /// The identities this removal planned.
        planned: PlannedOrigin,
    },
    /// The object at the origin snapshot's name is not a snapshot.
    OriginIsNotASnapshot {
        /// The identities this removal planned.
        planned: PlannedOrigin,
    },
    /// The snapshot at the origin's name reports another GUID than the planned one.
    OriginSnapshotIsNotThePlannedOne {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The GUID the kernel reported.
        live: u64,
    },
    /// The snapshot at the origin's name was created in another transaction group.
    OriginSnapshotCreatetxgDiffers {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// The transaction group the kernel reported.
        live: u64,
    },
    /// One of the three reads answered with a failure that is not "not there".
    ReadFailed {
        /// The identities this removal planned.
        planned: PlannedOrigin,
        /// Which of the three objects the read was for.
        object: JudgedObject,
        /// The name the read asked for.
        name: String,
        /// What the read answered.
        said: String,
    },
}


#[cfg(test)]
#[path = "origin_refusal_tests.rs"]
mod tests;
