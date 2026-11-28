//! What:
//!  The deferred-destroy mark of one dataset or snapshot:
//!  the one property the kernel computes for it,
//!  the two numbers a refusal prints beside it,
//!  and the one read of it a caller outside this folder makes.
//! Why:
//!  A snapshot somebody marked with `zfs destroy -d` is destroyed by ZFS itself once its
//!  last clone is gone,
//!  so destroying that clone inside a removal can destroy history another program asked
//!  to keep (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Reading the mark",
//!  F1 to F10).
//!  The mark is a property of the object-stats reply the route already reads for the
//!  object's identity,
//!  so the check adds no request kind,
//!  and a reply without the property is unreadable rather than off:
//!  a check that cannot read its one field has not checked
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  veto V3).
//!  This read is exported because the origin-cleanup record's design assumes it may read
//!  the mark without a second implementation
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "Dependencies on sibling designs").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mark = originMark(reader, 'pool/checkout@git-worktree-clone-origin-7f3a9c2e');
//! if (mark.kind === 'marked') refuse(mark.clones, mark.holds);
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    PropertyValue,
    value_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ReadFailure,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

//region The property and its two values

/// What:
///  The read-only property that says whether a snapshot is marked for deferred destroy.
/// Why:
///  The kernel computes it from the dataset's own flag rather than storing it as a value,
///  so it is the flag itself that decides the mark and never a written property
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  F1 and F2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFER_DESTROY = 'defer_destroy';
/// ```
pub const DEFER_DESTROY_PROPERTY: &str = "defer_destroy";

/// The property counting the user holds on a snapshot,
/// which decides whether the mark's reap happens with the last clone's destroy.
pub const USER_HOLDS_PROPERTY: &str = "userrefs";

/// What:
///  The property value that says the object is not marked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MARK_OFF = 0;
/// ```
pub const MARK_OFF: u64 = 0;

/// The property value that says the object is marked.
pub const MARK_ON: u64 = 1;

//endregion

//region The mark's answer

/// What:
///  How many user holds the reply counted,
///  told apart from a count the reply does not carry.
/// Why:
///  A refusal says whether this removal would destroy the marked snapshot,
///  and a hold count it could not read must not be printed as zero:
///  zero holds is part of the condition under which the destroy reaps the snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UserHolds = { kind: 'counted'; count: number } | { kind: 'unreadable'; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserHolds {
    /// The reply counted this many holds on the snapshot.
    Counted(u64),
    /// The reply carries no hold count this folder could read,
    /// and what it carried instead.
    Unreadable {
        /// What the reply holds in the property's place.
        said: String,
    },
}

/// What:
///  One dataset's or snapshot's state of the deferred-destroy mark.
/// Why:
///  Three states reach a caller and only two of them are values of the property:
///  a marked object,
///  an unmarked one,
///  and a reply whose property could not be read,
///  which never falls through to "not marked".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OriginMark = { kind: 'unmarked' } | { kind: 'marked'; clones; holds } | { kind: 'unreadable'; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OriginMark {
    /// The property reads off.
    Unmarked,
    /// The property reads on,
    /// with the two counts a refusal prints to say whether the next destroy reaps the object.
    Marked {
        /// How many clones of this snapshot the reply reported.
        clones: u64,
        /// How many user holds it counted.
        holds: UserHolds,
    },
    /// The property,
    /// its value or the whole reply could not be read.
    Unreadable {
        /// What the reply held instead of an off or on value.
        said: String,
    },
}

impl OriginMark {
    /// Whether the object is marked.
    pub fn is_marked(&self) -> bool {
        return matches!(self, OriginMark::Marked { .. });
    }

    /// How a message words the value the reply carried.
    pub fn described(&self) -> String {
        return match self {
            OriginMark::Unmarked => format!("`{DEFER_DESTROY_PROPERTY}` reads {MARK_OFF}"),
            OriginMark::Marked {
                ..
            } => format!("`{DEFER_DESTROY_PROPERTY}` reads {MARK_ON}"),
            OriginMark::Unreadable {
                said,
            } => format!("`{DEFER_DESTROY_PROPERTY}` could not be read ({said})"),
        };
    }
}

//endregion

//region Reading the mark

/// The holds one reply counted,
/// or why its hold count is not a number this folder reads.
fn holds_of(identity: &DatasetIdentity) -> UserHolds {
    let value: PropertyValue = value_of(&identity.properties, USER_HOLDS_PROPERTY);
    return match value.number() {
        Some(count) => UserHolds::Counted(count),
        None => UserHolds::Unreadable {
            said: value.describe(USER_HOLDS_PROPERTY),
        },
    };
}

/// What:
///  The mark one object-stats reply carries.
/// Why:
///  The check judges the mark of the snapshot whose own reply proved its identity,
///  so the mark is read from that reply rather than from a second request that a
///  same-account process could answer differently
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  "What the check reads,
///  in order").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mark = markOfReply(identity);
/// ```
pub fn mark_of_reply(identity: &DatasetIdentity) -> OriginMark {
    let value: PropertyValue = value_of(&identity.properties, DEFER_DESTROY_PROPERTY);
    return match value {
        PropertyValue::Number(MARK_OFF) => OriginMark::Unmarked,
        PropertyValue::Number(MARK_ON) => OriginMark::Marked {
            clones: identity.stats.dds_num_clones,
            holds: holds_of(identity),
        },
        PropertyValue::Number(other) => OriginMark::Unreadable {
            said: format!(
                "`{DEFER_DESTROY_PROPERTY}` reads {other}, and a mark is {MARK_OFF} or {MARK_ON}"
            ),
        },
        other => OriginMark::Unreadable {
            said: other.describe(DEFER_DESTROY_PROPERTY),
        },
    };
}

/// What:
///  One read of one dataset's or snapshot's mark,
///  for a caller that judges the mark and not the object's identity.
/// Why:
///  The origin-cleanup design reads a leftover origin snapshot's mark in five `git worktree`
///  commands,
///  and a second reader for the same property would be a second answer to one question
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  "Where the code lives,
///  and what it exports").
///  A read that failed is one more unreadable state,
///  because the caller's next act is a destroy either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mark = originMark(reader, name);
/// ```
pub fn origin_mark(
    reader: &dyn DatasetReader,
    name: &str,
) -> OriginMark {
    let read: Result<DatasetIdentity, ReadFailure> = reader.identity(name);
    return match read {
        Ok(identity) => mark_of_reply(&identity),
        Err(failure) => OriginMark::Unreadable {
            said: failure.message(),
        },
    };
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_mark_tests.rs"]
mod tests;
