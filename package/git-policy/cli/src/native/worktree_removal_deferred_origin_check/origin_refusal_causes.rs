//! What:
//!  The part of a refusal's message that names the cause,
//!  its object and its field,
//!  and the two mark numbers with the sentence that says what they mean for the next destroy.
//! Why:
//!  A person reads this part to learn which object is not what the removal recorded,
//!  so it names the field and both values rather than saying that something differed
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals").
//!  The mark's two numbers explain the kernel's reap condition;
//!  they do not change the decision,
//!  and the words say what the kernel's code says rather than what a run has shown
//!  (the same design, limits L2 and L5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cause(refusal); // 'its origin snapshot `pool/checkout@s` is marked for deferred destroy ...'
//! ```

use super::origin_mark::{DEFER_DESTROY_PROPERTY, MARK_ON, UserHolds};
use super::origin_refusal::OriginRefusal;

/// What:
///  The cause as the sentence that follows the clone's name.
/// Why:
///  The message's first line is the clone and this cause,
///  so every cause words only itself and the check's own state comes from the form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cause(refusal);
/// ```
pub(super) fn cause(refusal: &OriginRefusal) -> String {
    return match refusal {
        OriginRefusal::PlanIncomplete { field, plan_home, .. } => {
            format!("the removal's plan at {} holds no line for {}", plan_home.display(), field.words())
        }
        OriginRefusal::OriginMarkedBeforeTheUnmount { origin_snapshot, clones, holds, .. } => marked_cause(origin_snapshot, *clones, holds),
        OriginRefusal::OriginMarkedForDeferredDestroy { planned, clones, holds } => marked_cause(&planned.origin_snapshot_name, *clones, holds),
        OriginRefusal::MarkUnreadable { origin_snapshot, said, .. } => format!(
            "the mark on its origin snapshot `{origin_snapshot}` could not be read: {said}, and a mark this check cannot read is never taken as off"
        ),
        OriginRefusal::CloneGone { planned } => {
            format!("the planned clone `{}` is not there, and the kernel answered that no such dataset exists", planned.clone_name)
        }
        OriginRefusal::CloneIsNotAFilesystem { planned, found, snapshot } => {
            not_a_filesystem("the object at the planned clone's name", &planned.clone_name, *found, *snapshot)
        }
        OriginRefusal::CloneIsNotThePlannedOne { planned, live } => {
            guid_differs("the dataset at the planned clone's name", &planned.clone_name, &planned.clone_guid.file_name(), *live)
        }
        OriginRefusal::CloneHasNoOrigin { planned } => {
            format!("the dataset at the planned clone's name `{}` reports no origin, so it is not a clone of anything", planned.clone_name)
        }
        OriginRefusal::CloneOriginDiffers { planned, live } => format!(
            "the clone `{}` names `{live}` as its origin, and this removal recorded `{}`",
            planned.clone_name, planned.origin_snapshot_name
        ),
        OriginRefusal::OriginDatasetGone { planned } => {
            format!("the origin dataset `{}` this removal recorded is not there, and the kernel answered that no such dataset exists", planned.origin_dataset_name)
        }
        OriginRefusal::OriginDatasetIsNotAFilesystem { planned, found, snapshot } => {
            not_a_filesystem("the object at the origin dataset's name", &planned.origin_dataset_name, *found, *snapshot)
        }
        OriginRefusal::OriginDatasetIsNotThePlannedOne { planned, live } => {
            guid_differs("the dataset at the origin's name", &planned.origin_dataset_name, &planned.origin_dataset_guid.file_name(), *live)
        }
        OriginRefusal::OriginSnapshotGone { planned } => {
            format!("the origin snapshot `{}` this removal recorded is not there, and the kernel answered that no such dataset exists", planned.origin_snapshot_name)
        }
        OriginRefusal::OriginIsNotASnapshot { planned } => {
            format!("`{}` is not a snapshot: the kernel answers a filesystem at that name", planned.origin_snapshot_name)
        }
        OriginRefusal::OriginSnapshotIsNotThePlannedOne { planned, live } => {
            guid_differs("the snapshot at the origin's name", &planned.origin_snapshot_name, &planned.origin_snapshot_guid.file_name(), *live)
        }
        OriginRefusal::OriginSnapshotCreatetxgDiffers { planned, live } => format!(
            "the origin snapshot `{}` carries the recorded GUID and was created in transaction group {live}, and this removal recorded {}, so it is a different snapshot under the same name",
            planned.origin_snapshot_name, planned.origin_snapshot_createtxg
        ),
        OriginRefusal::ReadFailed { object, name, said, .. } => format!("{} `{name}` could not be read: {said}", object.words()),
    };
}

/// The marked origin's cause,
/// with the two counts and what they mean for the destroy that would follow.
fn marked_cause(origin_snapshot: &str, clones: u64, holds: &UserHolds) -> String {
    return format!(
        "its origin snapshot `{origin_snapshot}` is marked for deferred destroy (`{DEFER_DESTROY_PROPERTY}` reads {MARK_ON}), and {}, {}",
        mark_numbers(clones, holds),
        what_the_destroy_does(clones, holds)
    );
}

/// The two counts the refusal prints,
/// in the words the reply supports.
fn mark_numbers(clones: u64, holds: &UserHolds) -> String {
    let cloned: String = noun(clones, "clone", "clones");
    return match holds {
        UserHolds::Counted(count) => format!("the kernel reports {cloned} of it and {}", noun(*count, "user hold", "user holds")),
        UserHolds::Unreadable { said } => format!("the kernel reports {cloned} of it, and its hold count could not be read ({said})"),
    };
}

/// What the next destroy of this clone does to the marked snapshot,
/// in the kernel's own condition.
fn what_the_destroy_does(clones: u64, holds: &UserHolds) -> String {
    return match (clones, holds) {
        (1, UserHolds::Counted(0)) => String::from("so destroying this clone destroys that snapshot in the same transaction"),
        (1, UserHolds::Counted(count)) => format!("so that snapshot stands after this removal, because {}", holds_phrase(*count)),
        (0, _) => String::from("so that snapshot stands after this removal, because no clone of it is here to destroy"),
        (_, UserHolds::Counted(0)) => String::from("so that snapshot stands after this removal, because another clone of it is in place"),
        (_, UserHolds::Counted(count)) => format!("so that snapshot stands after this removal, because another clone of it is in place and {}", holds_phrase(*count)),
        (_, UserHolds::Unreadable { .. }) => String::from("so whether destroying this clone would destroy that snapshot cannot be told from this reply"),
    };
}

/// One or more user holds,
/// worded for the sentence that follows.
fn holds_phrase(count: u64) -> String {
    if count == 1 {
        return String::from("one user hold pins it");
    }
    return format!("{count} user holds pin it");
}

/// A count with the noun that agrees with it.
fn noun(count: u64, one: &str, many: &str) -> String {
    if count == 1 {
        return format!("{count} {one}");
    }
    return format!("{count} {many}");
}

/// The cause of an object at a name that is not a filesystem,
/// or that is the snapshot the kernel named.
fn not_a_filesystem(role: &str, name: &str, found: u32, snapshot: bool) -> String {
    if snapshot {
        return format!("{role} `{name}` is a snapshot, not a filesystem");
    }
    return format!("{role} `{name}` is not a filesystem (the kernel reports object-set type {found})");
}

/// The cause of a GUID that is not the recorded one,
/// naming both values.
fn guid_differs(role: &str, name: &str, planned: &str, live: u64) -> String {
    return format!("{role} `{name}` is not the one this removal recorded: it recorded GUID `{planned}`, and the kernel reports `{live}`");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_refusal_causes_tests.rs"]
mod tests;
