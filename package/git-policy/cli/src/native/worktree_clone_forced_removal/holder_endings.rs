//! What:
//!  What this build may do to each kind of holder,
//!  the round that showed the ending works,
//!  and the refusal it answers instead where no round shows it.
//! Why:
//!  Plan step 17 builds the ending "once the owed measurement shows it works",
//!  so a kind whose ending the probe rounds and this round's guest arm do not
//!  show is identified and refused rather than ended
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 17", and
//!  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Who ends each kind of holder").
//!  Every answer here names its evidence in one constant beside it,
//!  so a reader can see which sentence of which log the ending rests on,
//!  and the two answers that are not endings say which arm is still owed.
//!  The table is data rather than prose so that the wrap-level code has one
//!  place to ask, and a round that measures a new kind changes one row.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = endingOf('path-holder'); ending.kind; ending.evidence;
//! ```

use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;

/// What:
///  One kind of holder this machine produces.
/// Why:
///  The design names five kinds and a sixth,
///  and each is told by what holds the clone rather than by which account owns
///  it,
///  because the ending is the same for two kinds that hold it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderKind = 'session-task' | 'other-account-task' | 'unit-task' | 'taskless-namespace' | 'path-holder' | 'locked-holder';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolderKind {
    /// The same account, in another session: a leftover agent shell or a build job.
    SessionTask,
    /// Another account's process.
    OtherAccountTask,
    /// A task of a unit the caller cannot signal itself.
    UnitTask,
    /// A mount namespace whose tasks are gone,
    /// which only a namespace handle or `listns` reaches.
    TasklessNamespace,
    /// A task with a working directory,
    /// open descriptor,
    /// executable,
    /// mapping or root inside the clone,
    /// which holds the clone without mounting it.
    PathHolder,
    /// A holder whose mount namespace is owned by a user namespace of its own,
    /// so the kernel locked its copy.
    LockedHolder,
}

impl HolderKind {
    /// The word a report names the kind by.
    pub fn word(self) -> &'static str {
        return match self {
            HolderKind::SessionTask => "session-task",
            HolderKind::OtherAccountTask => "other-account-task",
            HolderKind::UnitTask => "unit-task",
            HolderKind::TasklessNamespace => "taskless-namespace",
            HolderKind::PathHolder => "path-holder",
            HolderKind::LockedHolder => "locked-holder",
        };
    }
}

/// What:
///  What the act does with one kind of holder.
/// Why:
///  The three endings are the design's,
///  and the two answers that are not endings are the ones this build must give
///  where nothing measured the ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ending = { kind: 'end-blockers' | 'unmount-in-place' | 'move-then-unmount' } | { kind: 'refuse'; word };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ending {
    /// `SIGTERM`, a wait, then `SIGKILL` to every blocker,
    /// and the host's own unmount afterwards.
    EndBlockersThenUnmount,
    /// Way 8:
    /// unmount the copy in place by the recorded handle,
    /// changing no sharing.
    UnmountInPlaceByHandle,
    /// Way 7:
    /// make the copy's parent a slave,
    /// move the copy onto a short-lived directory and unmount it there.
    MakeTheParentASlaveThenMoveAndUnmount,
    /// Refuse,
    /// naming the reason,
    /// because no round shows an ending for this kind.
    Refuse(RefusalWord),
}

/// What:
///  One row of the table:
///  the kind,
///  what the act does,
///  the round that showed it,
///  and the arm still owed for it.
/// Why:
///  A reader of a report asks "measured where", and a later round that runs the
///  owed arm replaces one row rather than rewriting the prose around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KindEvidence = { kind; ending; shownBy: string; owedArm: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KindEvidence {
    /// The kind.
    pub kind: HolderKind,
    /// What the act does with it.
    pub ending: Ending,
    /// The round and the lines that showed it,
    /// or the reason no round shows it.
    pub shown_by: &'static str,
    /// The arm that would settle what is still open,
    /// or an empty text where nothing is.
    pub owed_arm: &'static str,
}

/// The evidence this round's guest arm recorded for a path holder:
/// the control refused `dataset is busy` with the holder alive,
/// the end sent `SIGTERM` to the holder's group and it exited,
/// the host's unmount was accepted,
/// and the head destroy answered `gone`,
/// with the caller's session's other job alive throughout.
pub const PATH_HOLDER_SHOWN_BY: &str = "this round's guest arm, phases path-holder-terminated and path-holder-killed: the control refused, the end emptied the slot and the head destroy answered gone, with the caller's session's other job alive";

/// The evidence for a same-account task of another session that holds the clone,
/// and the two shapes it does not cover.
pub const OTHER_SESSION_SHOWN_BY: &str = "this round's guest arm, phases path-holder-terminated and path-holder-killed: a working directory inside the mounted clone, in a session of its own, was asked out and killed, and the host's unmount and the head destroy then answered gone";

/// The evidence a task of another account or of a unit is owed:
/// the end is the same pinned signal,
/// and no round sent one to such a task.
pub const OTHER_ACCOUNT_OWED: &str = "a task of another account, and a task of a unit the caller cannot signal, end through the same pinned signal and no round shows either: the schedule shows one account's own child, and F8's refusal is the account's, not the handler's";

/// The evidence for way 8:
/// the unmount by the recorded handle inside the holder's namespace left the
/// host's own copy standing and the destroy answered `gone`,
/// across the probe rounds' handle arms and this round's guest arm.
pub const IN_PLACE_SHOWN_BY: &str = "this round's guest arm, phase namespace-way8-repaired: the holder's copy went, the host's copy stayed and the head destroy answered gone; the probe rounds' T0-5a, T0-5c and Z1-1 show the same unmount by the handle on 6.8 and on 7.3 over ZFS";

/// The evidence for way 7:
/// the parent made a slave,
/// the copy moved onto a directory in the checkout,
/// unmounted at the destination and the directory removed,
/// with the host's copy standing and the destroy answering `gone`.
pub const MOVE_SHOWN_BY: &str = "this round's guest arm, phases namespace-way7-repaired and namespace-way7-shared: the move and the unmount at the destination were accepted, the host's copy stayed and the head destroy answered gone; the probe rounds' T0-3 and T0-7 show the slave change and the move";

/// Why the locked holder's ending is not built:
/// the kernel answers `EINVAL` to both the unmount and the move inside such a
/// namespace,
/// which this round's arm and the probe rounds' T0-14 show,
/// and the design's V10 answer,
/// stopping a unit of the account's own user manager and finishing the removal,
/// has no measured arm at all.
pub const LOCKED_NOT_BUILT: &str = "the locked holder is refused: T0-14 answered EINVAL to both the unmount and the move inside such a namespace, this round's arm answered EINVAL to the unmount and read the namespace's owning user namespace, and the design's V10 answer, stopping the account's own user-manager unit and finishing the removal, is measured by no round";

/// Why the taskless namespace's ending is not built:
/// nothing in this build lists a namespace whose tasks are gone,
/// the `6.8` guest answers `ENOSYS` to `listns`,
/// and no arm unmounted a copy inside one.
pub const TASKLESS_NOT_BUILT: &str = "a taskless namespace is refused: this build lists no namespace whose tasks are gone, the guest's 6.8 kernel answers ENOSYS to listns, and no arm unmounted a copy inside one";

/// What:
///  The one table, in the design's order of kinds.
/// Why:
///  One list is what a report prints and what the act reads,
///  so no kind can be handled by code without a row that says what it may do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TABLE: readonly KindEvidence[] = [ ... ];
/// ```
pub const THE_KINDS: [KindEvidence; 6] = [
    KindEvidence {
        kind: HolderKind::SessionTask,
        ending: Ending::EndBlockersThenUnmount,
        shown_by: OTHER_SESSION_SHOWN_BY,
        owed_arm: "a holder of another session that keeps the clone mounted in the host's namespace, ended in the full act (M1)",
    },
    KindEvidence {
        kind: HolderKind::OtherAccountTask,
        ending: Ending::EndBlockersThenUnmount,
        shown_by: "no round shows this kind's end; the schedule that ran is one account's own child",
        owed_arm: OTHER_ACCOUNT_OWED,
    },
    KindEvidence {
        kind: HolderKind::UnitTask,
        ending: Ending::EndBlockersThenUnmount,
        shown_by: "no round shows this kind's end; no arm sent a pinned signal to a unit's task",
        owed_arm: OTHER_ACCOUNT_OWED,
    },
    KindEvidence {
        kind: HolderKind::TasklessNamespace,
        ending: Ending::Refuse(RefusalWord::HolderUnendable),
        shown_by: TASKLESS_NOT_BUILT,
        owed_arm: "T0-10's namespace on a 6.19 or newer guest, with a copy inside it unmounted",
    },
    KindEvidence {
        kind: HolderKind::PathHolder,
        ending: Ending::EndBlockersThenUnmount,
        shown_by: PATH_HOLDER_SHOWN_BY,
        owed_arm: "the same holder inside the full act, with the write that lands in the copy during the wait (M7)",
    },
    KindEvidence {
        kind: HolderKind::LockedHolder,
        ending: Ending::Refuse(RefusalWord::HolderLocked),
        shown_by: LOCKED_NOT_BUILT,
        owed_arm: "the design's V10 stop: a unit of the account's own user manager stopped through its manager, its cgroup waited for, and the removal finished (M-test 22 on a v262 guest)",
    },
];

/// What:
///  What the act does with one kind of holder.
/// Why:
///  A caller that asks about a kind gets the row's own answer,
///  and a kind with no row is a programming error rather than a default,
///  because a default ending would be an ending no evidence covers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = endingOf(kind);   // throws when the table lacks the kind
/// ```
pub fn ending_of(kind: HolderKind) -> Result<KindEvidence, MissingKind> {
    for row in THE_KINDS {
        if row.kind == kind {
            return Ok(row);
        }
    }
    return Err(MissingKind { kind });
}

/// What:
///  A kind the table does not answer for.
/// Why:
///  The refusal names the kind,
///  because the act must not guess an ending for a holder nothing measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MissingKind extends Error { constructor(readonly kind: HolderKind) {} }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MissingKind {
    /// The kind no row answers for.
    pub kind: HolderKind,
}

impl std::fmt::Display for MissingKind {
    /// One sentence naming the kind and the rule it breaks.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            formatter,
            "the forced removal has no measured ending for a holder of kind {}: no round shows one, so this build would have to refuse it, and the table's own rule is that every kind has a row",
            self.kind.word()
        );
    }
}

/// What:
///  Whether a kind's ending needs the block of `SIGTERM`, wait and `SIGKILL`.
/// Why:
///  A refusal ends nothing,
///  and an act that asked a blocker for its end would be ending a job in a
///  namespace this build refuses:
///  the locked holder's copy is refused before any signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ending.kind === 'end-blockers' || ending.kind === 'move-then-unmount'
/// ```
pub fn ends_blockers(ending: Ending) -> bool {
    return match ending {
        Ending::EndBlockersThenUnmount => true,
        // Way 7 and way 8 end the blockers that make the unmount refuse, which the matcher names.
        Ending::UnmountInPlaceByHandle | Ending::MakeTheParentASlaveThenMoveAndUnmount => true,
        Ending::Refuse(_word) => false,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_endings_tests.rs"]
mod tests;
