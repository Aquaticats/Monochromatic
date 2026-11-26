//! What:
//!  The nine arms of the root start probe and every step inside them,
//!  in the order the worker runs them.
//! Why:
//!  The design numbers the arms and hangs one outcome on each
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms"),
//!  so the record says which arm passed,
//!  failed or did not run,
//!  and a timed-out record names the step that was in flight.
//!  The order is data rather than the order of some lines in a function,
//!  because the mode works out the step in flight from the last status line it
//!  read and the sequence here
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Q-b" and "Who runs it,
//! and where").
//!  A removal is refused after a failed or timed-out move,
//!  unmount or scan,
//!  so the steps of those three say so here rather than in a second list.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Arm = 'namespace' | 'walk' | ... ; type Step = { arm: Arm; name: string };
//! const workerOrder = [unshare, propagatePrivate, scratchTmpfs, ...];
//! ```

/// What:
///  One arm of the probe,
///  with the words the record and the refusal lines use.
/// Why:
///   The arm is what a verdict names,
///  and the record carries one line per arm,
///  so the names are fixed here and no caller words them again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Arm = 'namespace' | 'walk' | 'source' | 'attach' | 'verify' | 'holder' | 'scan' | 'move-unmount' | 'teardown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeArm {
    /// Arm 1:
    /// the private mount namespace and the scratch tmpfs.
    Namespace,
    /// Arm 2:
    /// the no-crossing walk and its refusal controls.
    Walk,
    /// Arm 3:
    /// the source name from the root-owned configuration.
    Source,
    /// Arm 4:
    /// the exclusive attach of the snapshot.
    Attach,
    /// Arm 5:
    /// the flags and the mount read back.
    Verify,
    /// Arm 6:
    /// the holder's mount in a second namespace.
    Holder,
    /// Arm 7:
    /// the two-pass namespace scan.
    Scan,
    /// Arm 8:
    /// the move and the unmount from a second process,
    /// and the closing scan.
    MoveAndUnmount,
    /// Arm 9:
    /// the teardown the worker ends in.
    Teardown,
}

impl ProbeArm {
    /// Every arm,
    /// in the design's own order.
    pub const ALL: &[ProbeArm] = &[
        ProbeArm::Namespace,
        ProbeArm::Walk,
        ProbeArm::Source,
        ProbeArm::Attach,
        ProbeArm::Verify,
        ProbeArm::Holder,
        ProbeArm::Scan,
        ProbeArm::MoveAndUnmount,
        ProbeArm::Teardown,
    ];

    /// The arm in the words a record line carries.
    pub fn name(self) -> &'static str {
        return match self {
            ProbeArm::Namespace => "namespace",
            ProbeArm::Walk => "walk",
            ProbeArm::Source => "source",
            ProbeArm::Attach => "attach",
            ProbeArm::Verify => "verify",
            ProbeArm::Holder => "holder",
            ProbeArm::Scan => "scan",
            ProbeArm::MoveAndUnmount => "move-unmount",
            ProbeArm::Teardown => "teardown",
        };
    }
}

/// What:
///  One step of the probe:
///  a call or a check whose outcome the record keeps,
///  and the arm it belongs to.
/// Why:
///   An arm passes only when every step of it passed,
///  and a missed deadline names the step in flight,
///  so the steps are the record's smallest facts.
///  Arm 8's move,
///  its unmount and the closing scan are steps of their own,
///  which is what lets a timed-out record name which one was running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = 'unshare' | 'propagate-private' | ... | 'teardown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeStep {
    /// `unshare(CLONE_NEWNS)`.
    Unshare,
    /// Every mount made private.
    PrivatePropagation,
    /// The scratch tmpfs mounted.
    ScratchTmpfs,
    /// The slot,
    /// the move target and the refusal tree created.
    ScratchTree,
    /// The walk to the slot and its identity recorded.
    WalkSlot,
    /// The four refusal controls.
    WalkRefusalControls,
    /// The checkout root opened read-only.
    CheckoutRoot,
    /// The source name read from the configuration.
    ReadSourceName,
    /// `fsopen` on the `zfs` type.
    AttachContext,
    /// The source written into the context.
    AttachSource,
    /// The read-only options written into the context.
    AttachOptions,
    /// The exclusive create.
    AttachExclusiveCreate,
    /// `fsmount` with the read-only flags.
    AttachMountDescriptor,
    /// `move_mount` onto the held slot descriptor.
    AttachPlace,
    /// The mount ID,
    /// type,
    /// source and flags read back.
    VerifyMount,
    /// The holder's own mount in its own namespace.
    HolderAttach,
    /// The holder waiting on its control pipe.
    HolderWait,
    /// The scan while both mounts exist.
    ScanWhileHeld,
    /// The scan after the holder is gone.
    ScanAfterHolder,
    /// The move from the slot to the move target.
    Move,
    /// The unmount by path.
    Unmount,
    /// The scan after the unmount.
    ClosingScan,
    /// The worker's last line.
    Teardown,
}

impl ProbeStep {
    /// The steps the worker runs,
    /// in order,
    /// with the scan in its two passes around the holder.
    pub const WORKER_ORDER: &[ProbeStep] = &[
        ProbeStep::Unshare,
        ProbeStep::PrivatePropagation,
        ProbeStep::ScratchTmpfs,
        ProbeStep::ScratchTree,
        ProbeStep::WalkSlot,
        ProbeStep::WalkRefusalControls,
        ProbeStep::CheckoutRoot,
        ProbeStep::ReadSourceName,
        ProbeStep::AttachContext,
        ProbeStep::AttachSource,
        ProbeStep::AttachOptions,
        ProbeStep::AttachExclusiveCreate,
        ProbeStep::AttachMountDescriptor,
        ProbeStep::AttachPlace,
        ProbeStep::VerifyMount,
        ProbeStep::ScanWhileHeld,
        ProbeStep::ScanAfterHolder,
        ProbeStep::Move,
        ProbeStep::Unmount,
        ProbeStep::ClosingScan,
        ProbeStep::Teardown,
    ];

    /// The steps the holder runs,
    /// in order.
    pub const HOLDER_ORDER: &[ProbeStep] = &[ProbeStep::HolderAttach, ProbeStep::HolderWait];

    /// The step in the words a record line carries.
    pub fn name(self) -> &'static str {
        return match self {
            ProbeStep::Unshare => "unshare",
            ProbeStep::PrivatePropagation => "propagate-private",
            ProbeStep::ScratchTmpfs => "scratch-tmpfs",
            ProbeStep::ScratchTree => "scratch-tree",
            ProbeStep::WalkSlot => "walk-slot",
            ProbeStep::WalkRefusalControls => "walk-refusal-controls",
            ProbeStep::CheckoutRoot => "checkout-root",
            ProbeStep::ReadSourceName => "read-source-name",
            ProbeStep::AttachContext => "attach-context",
            ProbeStep::AttachSource => "attach-source",
            ProbeStep::AttachOptions => "attach-options",
            ProbeStep::AttachExclusiveCreate => "attach-exclusive-create",
            ProbeStep::AttachMountDescriptor => "attach-mount-descriptor",
            ProbeStep::AttachPlace => "attach-place",
            ProbeStep::VerifyMount => "verify-mount",
            ProbeStep::HolderAttach => "holder-attach",
            ProbeStep::HolderWait => "holder-wait",
            ProbeStep::ScanWhileHeld => "scan-while-held",
            ProbeStep::ScanAfterHolder => "scan-after-holder",
            ProbeStep::Move => "move",
            ProbeStep::Unmount => "unmount",
            ProbeStep::ClosingScan => "closing-scan",
            ProbeStep::Teardown => "teardown",
        };
    }

    /// Every step of both children,
    /// the worker's and the holder's,
    /// in one list for parsing and folding.
    pub const ALL: &[ProbeStep] = &[
        ProbeStep::Unshare,
        ProbeStep::PrivatePropagation,
        ProbeStep::ScratchTmpfs,
        ProbeStep::ScratchTree,
        ProbeStep::WalkSlot,
        ProbeStep::WalkRefusalControls,
        ProbeStep::CheckoutRoot,
        ProbeStep::ReadSourceName,
        ProbeStep::AttachContext,
        ProbeStep::AttachSource,
        ProbeStep::AttachOptions,
        ProbeStep::AttachExclusiveCreate,
        ProbeStep::AttachMountDescriptor,
        ProbeStep::AttachPlace,
        ProbeStep::VerifyMount,
        ProbeStep::HolderAttach,
        ProbeStep::HolderWait,
        ProbeStep::ScanWhileHeld,
        ProbeStep::ScanAfterHolder,
        ProbeStep::Move,
        ProbeStep::Unmount,
        ProbeStep::ClosingScan,
        ProbeStep::Teardown,
    ];

    /// The arm the step belongs to.
    pub fn arm(self) -> ProbeArm {
        return match self {
            ProbeStep::Unshare
            | ProbeStep::PrivatePropagation
            | ProbeStep::ScratchTmpfs
            | ProbeStep::ScratchTree => ProbeArm::Namespace,
            ProbeStep::WalkSlot | ProbeStep::WalkRefusalControls | ProbeStep::CheckoutRoot => {
                ProbeArm::Walk
            }
            ProbeStep::ReadSourceName => ProbeArm::Source,
            ProbeStep::AttachContext
            | ProbeStep::AttachSource
            | ProbeStep::AttachOptions
            | ProbeStep::AttachExclusiveCreate
            | ProbeStep::AttachMountDescriptor
            | ProbeStep::AttachPlace => ProbeArm::Attach,
            ProbeStep::VerifyMount => ProbeArm::Verify,
            ProbeStep::HolderAttach | ProbeStep::HolderWait => ProbeArm::Holder,
            ProbeStep::ScanWhileHeld | ProbeStep::ScanAfterHolder => ProbeArm::Scan,
            ProbeStep::Move | ProbeStep::Unmount | ProbeStep::ClosingScan => {
                ProbeArm::MoveAndUnmount
            }
            ProbeStep::Teardown => ProbeArm::Teardown,
        };
    }

    /// Whether this step's failure or a deadline that caught it refuses removal
    ///  of a mounted clone.
    ///
    /// The design refuses that removal only when the failed arm is the move,
    /// the unmount or the scan,
    /// and a deadline that caught one of those is treated as a failed arm
    /// (`doc/decision/cli-git-zfs-start-probe.md`,
    /// Q2 and Q5).
    pub fn refuses_removal_of_a_mounted_clone(self) -> bool {
        return match self {
            ProbeStep::ScanWhileHeld
            | ProbeStep::ScanAfterHolder
            | ProbeStep::Move
            | ProbeStep::Unmount
            | ProbeStep::ClosingScan => true,
            ProbeStep::Unshare
            | ProbeStep::PrivatePropagation
            | ProbeStep::ScratchTmpfs
            | ProbeStep::ScratchTree
            | ProbeStep::WalkSlot
            | ProbeStep::WalkRefusalControls
            | ProbeStep::CheckoutRoot
            | ProbeStep::ReadSourceName
            | ProbeStep::AttachContext
            | ProbeStep::AttachSource
            | ProbeStep::AttachOptions
            | ProbeStep::AttachExclusiveCreate
            | ProbeStep::AttachMountDescriptor
            | ProbeStep::AttachPlace
            | ProbeStep::VerifyMount
            | ProbeStep::HolderAttach
            | ProbeStep::HolderWait
            | ProbeStep::Teardown => false,
        };
    }

    /// The step's name back from the word a status line carries,
    /// or nothing when the word is none of them.
    pub fn from_name(word: &str) -> Option<ProbeStep> {
        return ProbeStep::ALL
            .iter()
            .copied()
            .find(|step| return step.name() == word);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_arms_tests.rs"]
mod tests;
