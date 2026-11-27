//! What:
//!  What one unmount turn answered:
//!  the word and the mount numbers a receipt carries,
//!  the step and the errno a mount that stayed names,
//!  and the refusals the turn and the record's checks build on the way.
//! Why:
//!  The wrapper,
//!  the receipt and the busy answer read one type,
//!  so a refused step is written once in one shape,
//!  and the record's own checks refuse with the same shape before the bind
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging").
//!  The answers sit apart from the turn that fills them,
//!  because three files build them and only one runs the calls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type UnmountOutcome = { kind: 'unmounted' } | { kind: 'reattached'; cause: ReattachCause } | { kind: 'retained' | 'refused'; step; errno; said };
//! const refused = (step: string, said: string): UnmountOutcome => ...;
//! ```

use super::staging_calls::StagingCallRefusal;
use crate::fs_mount_options::MountFacts;
use std::path::PathBuf;

/// What:
///  What one turn did.
/// Why:
///  The wrapper and the receipt need one word and the mount numbers,
///  and a person needs the step and errno when nothing was unmounted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnmountOutcome = { kind: 'unmounted'; mountId; bindGone } | { kind: 'rolled-back'; ... } | { kind: 'retained' | 'refused'; step; errno; said; mount? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnmountOutcome {
    /// The clone and the bind are gone.
    Unmounted {
        /// The clone's mount number.
        mount_id: u64,
        /// The slot's path.
        slot: PathBuf,
        /// Whether the bind was unbound too.
        bind_gone: bool,
    },
    /// The clone is back at its slot under the checked reattach,
    /// and the bind may be gone.
    Reattached {
        /// The clone's mount number.
        mount_id: u64,
        /// The slot's path.
        slot: PathBuf,
        /// Whether the bind was unbound too.
        bind_gone: bool,
        /// The clone root's device.
        device: u64,
        /// The clone root's inode.
        inode: u64,
        /// Why the mount was moved back.
        cause: ReattachCause,
    },
    /// A mount stays where a step left it.
    Retained {
        /// The step that refused.
        step: String,
        /// The kernel's errno,
        /// or zero.
        errno: i32,
        /// What refused.
        said: String,
        /// The topmost row when one was read.
        mount: Option<MountFacts>,
    },
    /// Nothing was moved,
    /// and the step says why.
    Refused {
        /// The step that refused.
        step: String,
        /// The kernel's errno,
        /// or zero.
        errno: i32,
        /// What refused.
        said: String,
        /// The row when one was read.
        mount: Option<MountFacts>,
    },
}

/// What:
///  Why a mount was moved back to its slot.
/// Why:
///  The accepted decision gives one of the two a refusal receipt and no other,
///  so the caller must be able to tell them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReattachCause = 'misplaced-rolled-back' | 'busy';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReattachCause {
    /// The move landed on a mount the account stacked,
    /// so the rollback moved it back.
    MisplacedRolledBack,
    /// The unmount answered `EBUSY`,
    /// so the checked reattach moved it back.
    Busy,
}

impl ReattachCause {
    /// The word a report carries.
    pub fn word(self) -> &'static str {
        return match self {
            ReattachCause::MisplacedRolledBack => "misplaced-rolled-back",
            ReattachCause::Busy => "busy-reattached",
        };
    }
}

impl UnmountOutcome {
    /// The word a receipt carries.
    pub fn word(&self) -> &'static str {
        return match self {
            UnmountOutcome::Unmounted {
                ..
            } => "done",
            UnmountOutcome::Reattached {
                ..
            } => "reattached",
            UnmountOutcome::Retained {
                ..
            }
            | UnmountOutcome::Refused {
                ..
            } => "refused",
        };
    }

    /// Whether the clone's mount is absent.
    pub fn unmounted(&self) -> bool {
        return matches!(self, UnmountOutcome::Unmounted { .. });
    }

    /// Whether a mount stays where it was.
    pub fn retained(&self) -> bool {
        return matches!(self, UnmountOutcome::Retained { .. });
    }
}

/// One refused read as a refused outcome.
pub(super) fn refused(
    step: &str,
    said: String,
) -> UnmountOutcome {
    return UnmountOutcome::Refused {
        step: String::from(step),
        errno: 0,
        said,
        mount: None,
    };
}

/// One call's refusal as a refused outcome.
pub(super) fn call_refused(
    step: &str,
    refusal: StagingCallRefusal,
    mount: Option<MountFacts>,
) -> UnmountOutcome {
    return UnmountOutcome::Refused {
        step: String::from(step),
        errno: refusal.errno,
        said: refusal.message(step),
        mount,
    };
}
