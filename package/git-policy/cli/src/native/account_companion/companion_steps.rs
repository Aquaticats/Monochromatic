//! What:
//!  The account companion's steps,
//! in the order its child runs them,
//!  the word each step is named by,
//!  and the boundary between a refusal that means this host will not give the
//!  companion a user namespace and one that means the companion itself failed.
//! Why:
//!  The design draws that boundary by the call and the errno,
//! not by the step,
//!  and closes it when the scratch tmpfs is in place
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Its skip").
//!  Everything at or after the walk is an arm failure whichever errno it
//!  answers,
//! so a defect in the companion's own code refuses the managed attach
//!  rather than turning it off in silence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const steps = ['user-namespace', 'setgroups', 'uid-map', 'gid-map', 'mount-namespace', 'private-propagation', 'scratch-tmpfs', 'scratch-tree', 'walk', 'attach', 'mount-facts', 'refusal-controls', 'move-unmount'] as const;
//! ```

use super::companion_calls::CompanionCall;

/// What:
///  One step of the companion's child,
///  in the order the child runs it.
/// Why:
///  The parent reads one record per step,
///  and a record it cannot place is an arm failure,
///  so the order and the numbering are one fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = 'user-namespace' | 'setgroups' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CompanionStep {
    /// `unshare(CLONE_NEWUSER)`,
    /// the first setup call.
    UserNamespace = 0,
    /// The `deny` write to `/proc/self/setgroups`.
    Setgroups = 1,
    /// The first mapping write,
    /// `/proc/self/uid_map`.
    UidMap = 2,
    /// The second mapping write,
    /// `/proc/self/gid_map`.
    GidMap = 3,
    /// `unshare(CLONE_NEWNS)`.
    MountNamespace = 4,
    /// The private propagation of every mount in the new namespace.
    PrivatePropagation = 5,
    /// The scratch tmpfs,
    /// the last setup call.
    ScratchTmpfs = 6,
    /// The slot,
    /// the move target and the refusal tree inside that tmpfs.
    ScratchTree = 7,
    /// The no-crossing walk to the slot and its identity.
    Walk = 8,
    /// The descriptor attach onto the held slot.
    Attach = 9,
    /// The mount facts read back from `/proc/self/mountinfo`.
    MountFacts = 10,
    /// The four refusal controls.
    RefusalControls = 11,
    /// The move and the unmount from a second child.
    MoveUnmount = 12,
}

/// How many steps one run reports.
pub const STEP_COUNT: usize = 13;

/// The steps in the order the child runs them,
/// which is also their numbering.
pub const STEPS_IN_ORDER: [CompanionStep; STEP_COUNT] = [
    CompanionStep::UserNamespace,
    CompanionStep::Setgroups,
    CompanionStep::UidMap,
    CompanionStep::GidMap,
    CompanionStep::MountNamespace,
    CompanionStep::PrivatePropagation,
    CompanionStep::ScratchTmpfs,
    CompanionStep::ScratchTree,
    CompanionStep::Walk,
    CompanionStep::Attach,
    CompanionStep::MountFacts,
    CompanionStep::RefusalControls,
    CompanionStep::MoveUnmount,
];

impl CompanionStep {
    /// The step one record's number names,
    /// or nothing when the number names no step of this build.
    pub fn from_number(number: u8) -> Option<CompanionStep> {
        return STEPS_IN_ORDER.iter().copied().find(|step| return *step as u8 == number);
    }

    /// The word the step is named by,
    /// in messages and in the test switch's values.
    pub fn name(self) -> &'static str {
        return match self {
            CompanionStep::UserNamespace => "user-namespace",
            CompanionStep::Setgroups => "setgroups",
            CompanionStep::UidMap => "uid-map",
            CompanionStep::GidMap => "gid-map",
            CompanionStep::MountNamespace => "mount-namespace",
            CompanionStep::PrivatePropagation => "private-propagation",
            CompanionStep::ScratchTmpfs => "scratch-tmpfs",
            CompanionStep::ScratchTree => "scratch-tree",
            CompanionStep::Walk => "walk",
            CompanionStep::Attach => "attach",
            CompanionStep::MountFacts => "mount-facts",
            CompanionStep::RefusalControls => "refusal-controls",
            CompanionStep::MoveUnmount => "move-unmount",
        };
    }

    /// The step one word names,
    /// or nothing when the word names no step of this build.
    pub fn from_name(word: &str) -> Option<CompanionStep> {
        return STEPS_IN_ORDER.iter().copied().find(|step| return step.name() == word);
    }

    /// Whether the step is one of the setup's calls,
    /// which are the only calls whose refusal the design lets a host's answer
    /// turn into a skip.
    pub fn is_setup(self) -> bool {
        return (self as u8) <= (CompanionStep::ScratchTmpfs as u8);
    }
}

/// What:
///  Whether one failed call means this host refuses the companion its user
///  namespace,
///  or refuses a capability or a mount inside it,
///  rather than the companion's own defect.
/// Why:
///  The design names the exact call and errno of each skip,
///  so a call it does not name,
///  an errno it does not name for that call,
///  and every answer after the scratch tmpfs is in place are arm failures.
///  `EINVAL` from `unshare(CLONE_NEWUSER)` is a skip only when this host has no
///  user-namespace entry at all:
/// with the entry there,
///  `EINVAL` is the companion's own malformed request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// skipsAt({ step: 'user-namespace', errno: 'EPERM', userNamespaceEntry: true }) // true
/// ```
pub fn is_skip(step: CompanionStep, call: CompanionCall, errno: i32, user_namespace_entry: bool) -> bool {
    if !step.is_setup() {
        return false;
    }
    return match (step, call) {
        (CompanionStep::UserNamespace, CompanionCall::UnshareUserNamespace) => {
            matches!(errno, libc::ENOSPC | libc::EPERM | libc::EACCES) || (errno == libc::EINVAL && !user_namespace_entry)
        }
        (CompanionStep::UidMap, _) => errno == libc::EPERM,
        (CompanionStep::MountNamespace, CompanionCall::UnshareMountNamespace) => errno == libc::EPERM,
        (CompanionStep::PrivatePropagation, _) => errno == libc::EPERM || errno == libc::EACCES,
        (CompanionStep::ScratchTmpfs, CompanionCall::MountTmpfs) => matches!(errno, libc::EPERM | libc::EACCES),
        _other => false,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_steps_tests.rs"]
mod tests;
