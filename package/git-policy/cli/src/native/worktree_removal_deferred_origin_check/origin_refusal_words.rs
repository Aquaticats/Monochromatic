//! What:
//!  The three forms of a refusal's message,
//!  the state sentence each is allowed to claim,
//!  and the ways forward a person is given.
//! Why:
//!  A refusal is the answer to the attempt that met it,
//!  and its words are the trace the project chose
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "What trace of a destruction outlives the removal").
//!  Each form claims only what its point read,
//!  the look makes no claim about the mount it never reads,
//!  and the ways forward name reads and decisions rather than a destructive command to paste
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals" and veto V7).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(refusal.message(history)); // three paragraphs' worth of one witness, per form
//! ```

use super::origin_refusal::OriginRefusal;
use super::origin_refusal_causes::cause;
use super::origin_refusals::{
    JudgedObject,
    RefusalForm,
};
use super::removal_history::RemovalHistory;

/// The clone the message is about,
/// or the slot for a plan that names no clone.
fn addressed(refusal: &OriginRefusal) -> String {
    return match refusal.clone_name() {
        Some(name) => format!("`{name}`"),
        None => format!("for the slot `{}`", refusal.slot()),
    };
}

/// What a form C refusal could not compare,
/// which is the cause's own subject.
fn compared_nothing(refusal: &OriginRefusal) -> String {
    return match refusal {
        OriginRefusal::PlanIncomplete {
            ..
        } => String::from(
            "Nothing this removal planned was read, because the plan does not say which object to read.",
        ),
        OriginRefusal::CloneGone {
            ..
        } => {
            String::from("Nothing this removal recorded about that clone can be compared any more.")
        }
        OriginRefusal::CloneIsNotAFilesystem {
            ..
        } => String::from(
            "What answers at that name is not the kind of object this removal recorded, so no field of it can be compared.",
        ),
        OriginRefusal::CloneIsNotThePlannedOne {
            ..
        } => String::from(
            "The dataset at that name is not the one this removal recorded, so nothing else about it is compared.",
        ),
        OriginRefusal::CloneHasNoOrigin {
            ..
        } => String::from(
            "The dataset at that name reports no origin, so the origin this removal recorded cannot be compared.",
        ),
        OriginRefusal::CloneOriginDiffers {
            ..
        } => String::from(
            "The clone's origin is not the snapshot this removal recorded, so that snapshot was not read and its mark was not judged.",
        ),
        OriginRefusal::OriginDatasetGone {
            ..
        } => String::from(
            "Nothing this removal recorded about that dataset can be compared any more.",
        ),
        OriginRefusal::OriginDatasetIsNotAFilesystem {
            ..
        } => String::from(
            "What answers at that name is not the kind of object this removal recorded, so its identity cannot be compared.",
        ),
        OriginRefusal::OriginDatasetIsNotThePlannedOne {
            ..
        } => String::from(
            "The dataset at that name is not the one this removal recorded, so the snapshot under it was not read and the mark was not judged.",
        ),
        OriginRefusal::OriginSnapshotGone {
            ..
        } => String::from(
            "Nothing this removal recorded about that snapshot can be compared any more, so its mark was not judged.",
        ),
        OriginRefusal::OriginIsNotASnapshot {
            ..
        } => String::from(
            "What answers at that name is not a snapshot, so its identity and its mark were not judged.",
        ),
        OriginRefusal::OriginSnapshotIsNotThePlannedOne {
            ..
        } => String::from(
            "The snapshot at that name is not the one this removal recorded, so its mark was not judged.",
        ),
        OriginRefusal::OriginSnapshotCreatetxgDiffers {
            ..
        } => String::from(
            "The snapshot at that name was made in another transaction group than the one this removal recorded, so its mark was not judged.",
        ),
        OriginRefusal::ReadFailed {
            object,
            ..
        } => match object {
            JudgedObject::Clone => String::from(
                "Nothing this removal recorded about the clone could be compared, and no origin was read.",
            ),
            JudgedObject::OriginDataset => String::from(
                "Nothing this removal recorded about the origin dataset could be compared, and the snapshot under it was not read.",
            ),
            JudgedObject::OriginSnapshot => String::from(
                "Nothing this removal recorded about the origin snapshot could be compared, and its mark was not judged.",
            ),
        },
        OriginRefusal::OriginMarkedBeforeTheUnmount {
            ..
        }
        | OriginRefusal::OriginMarkedForDeferredDestroy {
            ..
        }
        | OriginRefusal::MarkUnreadable {
            ..
        } => String::from(
            "Only the object whose reply carried the mark was compared, and no other object of this removal was.",
        ),
    };
}

/// What the point that refused may claim,
/// in the form its state takes.
fn state(
    refusal: &OriginRefusal,
    history: &RemovalHistory,
) -> String {
    let slot: &str = refusal.slot();
    return match refusal.form() {
        RefusalForm::NothingTakenDown => format!(
            "{} This check changed nothing and the removal stopped before it took anything down: {}.",
            RemovalHistory::NothingDestroyed.sentence(),
            refusal.what_it_leaves().sentence(slot)
        ),
        RefusalForm::PendingAfterTheUnmount => {
            format!(
                "{} This check changed nothing. {}.",
                history.sentence(),
                refusal.what_it_leaves().sentence(slot)
            )
        }
        RefusalForm::NothingCompared => format!(
            "{} This check changed nothing, and it sent no request. {} The removal stays pending; putting the worktree back needs a person to reconcile that first, and `cancel-worktree-removal {slot}` reports the same mismatch rather than attaching anything.",
            history.sentence(),
            compared_nothing(refusal)
        ),
    };
}

/// What a person does about a marked origin,
/// which the wrapper cannot finish for them.
fn marked_ways_forward(origin_snapshot: &str) -> String {
    return format!(
        "`zfs get -H -o value defer_destroy {origin_snapshot}` and `zfs holds {origin_snapshot}` show the mark and the holds. The mark cannot be cleared in place, so this slot's removal finishes only outside the wrapper: either you accept that snapshot's destruction and destroy the clone yourself, which takes the worktree's files with it and needs the worktree taken down first, where a program holding it may refuse; or you clone that snapshot elsewhere first, which pins it, and then destroy the clone. Either way, the wrapper's own removal refuses a marked origin and prints this same mark until it is gone."
    );
}

/// What a person does about a mark the wrapper could not read.
fn unreadable_mark_ways_forward(origin_snapshot: &str) -> String {
    return format!(
        "`zfs get -H -o value defer_destroy {origin_snapshot}` shows the property the wrapper could not read, and a mark it cannot read is never taken as off, so nothing here goes on until the reply carries 0 or 1. A reply that never does names a pool or a module this wrapper cannot judge, which a person settles before the removal is run again."
    );
}

/// What a person reads and reconciles when a live object is not the planned one.
fn live_objects_ways_forward(
    name: &str,
    other: &str,
) -> String {
    return format!(
        "`zfs list -H -o name,guid,createtxg {name}` and `zfs get -H -o origin {other}` show what the live objects are now, and the only object this removal may destroy is the one its plan recorded, so a person reconciles those names before the removal is run again."
    );
}

/// The reads and the acts a person has for this cause,
/// with no destructive command among them.
fn ways_forward(refusal: &OriginRefusal) -> String {
    return match refusal {
        OriginRefusal::OriginMarkedBeforeTheUnmount {
            origin_snapshot,
            ..
        }
        | OriginRefusal::OriginMarkedForDeferredDestroy {
            planned:
                super::planned_origin::PlannedOrigin {
                    origin_snapshot_name: origin_snapshot,
                    ..
                },
            ..
        } => marked_ways_forward(origin_snapshot),
        OriginRefusal::MarkUnreadable {
            origin_snapshot,
            ..
        } => unreadable_mark_ways_forward(origin_snapshot),
        OriginRefusal::PlanIncomplete {
            plan_home,
            ..
        } => format!(
            "{} shows the fields the plan holds, and the removal reads no expectation from the live pool, so a person reconciles that record before the removal is run again.",
            plan_home.display()
        ),
        OriginRefusal::CloneGone {
            ..
        } => format!(
            "a later run settles only after it has proved that clone is gone, and no command of this release settles a clone it cannot prove gone."
        ),
        OriginRefusal::CloneIsNotAFilesystem {
            planned,
            ..
        }
        | OriginRefusal::CloneIsNotThePlannedOne {
            planned,
            ..
        }
        | OriginRefusal::CloneHasNoOrigin {
            planned,
        }
        | OriginRefusal::CloneOriginDiffers {
            planned,
            ..
        }
        | OriginRefusal::ReadFailed {
            planned,
            ..
        } => live_objects_ways_forward(&planned.clone_name, &planned.origin_snapshot_name),
        OriginRefusal::OriginDatasetGone {
            planned,
        }
        | OriginRefusal::OriginDatasetIsNotAFilesystem {
            planned,
            ..
        }
        | OriginRefusal::OriginDatasetIsNotThePlannedOne {
            planned,
            ..
        } => live_objects_ways_forward(&planned.origin_dataset_name, &planned.origin_snapshot_name),
        OriginRefusal::OriginSnapshotGone {
            planned,
        }
        | OriginRefusal::OriginIsNotASnapshot {
            planned,
        }
        | OriginRefusal::OriginSnapshotIsNotThePlannedOne {
            planned,
            ..
        }
        | OriginRefusal::OriginSnapshotCreatetxgDiffers {
            planned,
            ..
        } => live_objects_ways_forward(&planned.origin_snapshot_name, &planned.clone_name),
    };
}

/// What:
///  The whole message one refusal prints.
/// Why:
///  A person reads one text,
///  so the opening,
///  the state and the ways forward are assembled once and every form goes through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// message(refusal, history);
/// ```
pub(super) fn message(
    refusal: &OriginRefusal,
    history: &RemovalHistory,
) -> String {
    let opening: String = format!(
        "refusing to remove the ZFS worktree clone {}: {}.",
        addressed(refusal),
        cause(refusal)
    );
    return format!(
        "{opening} {} Ways forward: {}",
        state(refusal, history),
        ways_forward(refusal)
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_refusal_words_tests.rs"]
mod tests;
