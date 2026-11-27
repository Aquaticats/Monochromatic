//! What:
//!  The states one slot's Git lock can be in,
//!  and the three answers a caller acts on.
//! Why:
//!  The design names nine states,
//!  S1 to S9,
//!  and says one function answers with them so no caller invents its own order
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The states,
//! the checks and the words",
//!  and "Where the code goes").
//!  The slot's missing `.git` is a state beside the lock's rather than instead of it,
//!  so it travels with every answer and is printed as its own line.
//!  What a caller may do with an answer is not decided here:
//!  the removal's refusals and the settle are steps 14 and 16,
//!  and `doctor` prints these words and changes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type State = { kind: 'managed' } | { kind: 'cut-short' } | { kind: 'not-the-adds-words' } | ... ;
//! type Classification = { state: State; pointer: Reading };
//! ```

use super::reading_a_bounded_file::{
    ARefusedReading,
    Reading,
};
use crate::worktree_clone_binding_record::EntryRow;
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::PathBuf;

/// What:
///  Why no sentence can be recomposed for one lock,
///  or why the sentence the wrapper recomposed is not the one on disk.
/// Why:
///  A lock the read refused,
///  a lock holding a person's words and a record that is unreadable
///  are three different things for a person to do something about,
///  and all three are the state S3.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cause = { kind: 'no-record', reason } | { kind: 'anothers-words', firstLine } | { kind: 'the-read-refused', cause };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheWordsCause {
    /// The slot's record gave no nonce,
    /// so nothing could be recomposed to compare against.
    NoRecordToRecompose {
        /// What the record read answered.
        reason: String,
    },
    /// The lock stands and holds words this add did not write.
    AnothersWords {
        /// The lock's first line,
        /// as the same bounded read found it.
        first_line: String,
    },
    /// The lock's own read refused the name.
    TheReadRefused {
        /// Why the read refused it.
        cause: ARefusedReading,
    },
}

/// What:
///  One slot's lock state,
///  in the design's words S1 to S9.
/// Why:
///  Every caller needs the same states in the same order,
///  and each carries what its sentence prints.
///  S5 is not here:
///  the slot's missing pointer stands beside the lock's state and travels in
///  [`Classification`].
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { kind: 'managed', nonce, registration } | { kind: 'cut-short', nonce, registration, at, of } | { kind: 'not-the-adds-words', cause } | { kind: 'no-lock', nonce, registration } | { kind: 'no-registration', nonce, recorded } | { kind: 'no-binding-explains', found } | { kind: 'half-made', nonce, name, row } | { kind: 'somebody-elses-worktree', registration, found };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagedLockState {
    /// S1:
    /// the wrapper's verified lock,
    /// whose bytes are the recomposed sentence and Git's newline.
    Managed {
        /// The add's nonce.
        nonce: RequestNonce,
        /// The registration the lock stands in.
        registration: PathBuf,
    },
    /// S2:
    /// the wrapper's own lock,
    /// cut short by a killed write,
    /// still covering the whole nonce.
    CutShort {
        /// The add's nonce.
        nonce: RequestNonce,
        /// The registration the lock stands in.
        registration: PathBuf,
        /// How many bytes the killed write left.
        at: u64,
        /// How many bytes the sentence and Git's newline hold.
        of: u64,
    },
    /// S3:
    /// a lock the wrapper cannot recompose.
    NotTheAddsWords {
        /// Why it cannot.
        cause: TheWordsCause,
    },
    /// S4:
    /// no lock stands,
    /// while the record and the registration do.
    NoLock {
        /// The add's nonce.
        nonce: RequestNonce,
        /// The registration that holds no lock.
        registration: PathBuf,
    },
    /// S6:
    /// no registration stands to hold the lock at all.
    NoRegistration {
        /// The add's nonce.
        nonce: RequestNonce,
        /// The registration the record names,
        /// when it names one.
        recorded: Option<PathBuf>,
    },
    /// S7:
    /// a registration stands that this slot's record does not explain.
    NoBindingExplains {
        /// What was found in its place.
        found: String,
    },
    /// S8:
    /// a half-made registration the finder names,
    /// by the binding design's row the entry answers.
    HalfMade {
        /// The add's nonce.
        nonce: RequestNonce,
        /// The entry's name below `worktrees`.
        name: String,
        /// The row it answers.
        row: EntryRow,
    },
    /// S9:
    /// a registration that is not this slot's at all,
    /// with its lock whatever it is.
    SomebodyElsesWorktree {
        /// The registration that stands there.
        registration: PathBuf,
        /// What said so.
        found: String,
    },
}

/// What:
///  The answer a caller acts on,
///  which is what the prompt for this piece calls the three classes.
/// Why:
///  `doctor` and the removal must not disagree about whether a lock is the wrapper's own,
///  a person's or nothing either of them explains,
///  and the removal's refusals are stated in these terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ownership = 'the-wrappers-managed' | 'an-operators-lock' | 'unexplained' | 'no-lock-stands' | 'no-registration-stands';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockOwnership {
    /// S1 or S2:
    /// the lock the add wrote,
    /// whole or cut short.
    TheWrappersManaged,
    /// S3:
    /// words this add did not write,
    /// which the wrapper never releases.
    AnOperatorsLock,
    /// S6 to S9:
    /// nothing this slot's record explains.
    Unexplained,
    /// S4:
    /// the registration stands and holds no lock.
    NoLockStands,
    /// S6:
    /// no registration stands,
    /// so there is no lock to find.
    NoRegistrationStands,
}

/// What:
///  One slot's answer:
///  the lock's state and the slot's pointer.
/// Why:
///  A missing pointer changes what `git worktree prune` does at once,
///  and the design prints it beside the lock's state rather than instead of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Classification = { state: State; pointer: Reading };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    /// The lock's state.
    pub state: ManagedLockState,
    /// The slot's `.git`,
    /// which S5 prints when it is absent.
    pub pointer: Reading,
}

impl Classification {
    /// The class a caller acts on.
    pub fn ownership(&self) -> LockOwnership {
        return match self.state {
            ManagedLockState::Managed {
                ..
            }
            | ManagedLockState::CutShort {
                ..
            } => LockOwnership::TheWrappersManaged,
            ManagedLockState::NotTheAddsWords {
                ..
            } => LockOwnership::AnOperatorsLock,
            ManagedLockState::NoLock {
                ..
            } => LockOwnership::NoLockStands,
            ManagedLockState::NoRegistration {
                ..
            } => LockOwnership::NoRegistrationStands,
            ManagedLockState::NoBindingExplains {
                ..
            }
            | ManagedLockState::HalfMade {
                ..
            }
            | ManagedLockState::SomebodyElsesWorktree {
                ..
            } => LockOwnership::Unexplained,
        };
    }

    /// Whether the slot shows no `.git`,
    /// the state the design prints beside the lock's own.
    pub fn the_slot_shows_no_pointer(&self) -> bool {
        return self.pointer == Reading::Absent;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_state_tests.rs"]
mod tests;
