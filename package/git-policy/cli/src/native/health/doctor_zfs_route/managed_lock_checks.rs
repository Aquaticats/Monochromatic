//! What:
//!  The line one managed clone slot's Git lock prints:
//!  the wrapper's verified lock,
//!  its cut-short twin,
//!  words its add did not write,
//!  no lock,
//!  the slot's missing pointer beside the lock's own line,
//!  no registration,
//!  a registration no binding of the slot explains,
//!  a half-made registration,
//!  and another worktree's registration.
//! Why:
//!  `package/git-policy/cli/SPEC.md`,
//!  "### Doctor",
//!  hands these lines to plan step 18
//!  and names the module that already writes their words
//!  (`src/native/worktree_clone_managed_lock/lock_state_words.rs`).
//!  The state is step 13's classification and the sentences are step 13's,
//!  so this check reads both and decides neither:
//!  one lock never has two readings and no sentence has two spellings
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How doctor and removal tell the states apart",
//!  and `doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  which is why nothing calls this check yet).
//!  `doctor` writes no lock and removes no registration,
//!  and these lines are setup findings on standard error,
//!  so no answer here changes its exit status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkManagedLock({ slot, classification }); // pass | finding('lock' | 'registration')
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
};
use crate::worktree_clone_managed_lock::lock_state::{
    Classification,
    ManagedLockState,
};
use crate::worktree_clone_managed_lock::lock_state_words::the_lines;

/// What:
///  One managed clone slot's name and the lock state step 13's classification
///  answered for it.
/// Why:
///  The sentence names the slot,
///  and the state decides which sentence it is,
///  so both travel in one value a control writes and no control reads a
///  repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagedLockFacts = { slot: string; classification: Classification };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedLockFacts {
    /// The slot's name,
    /// which is the store's key for it and the name every sentence prints.
    pub slot: String,
    /// What step 13's classification answered for that slot,
    /// with the slot's pointer,
    /// which its own sentence prints when absent.
    pub classification: Classification,
}

/// What:
///  The sentence,
///  or sentences,
///  one slot's managed lock prints,
///  as a pass where the lock is the one the add wrote
///  and as a finding naming what a person must look at everywhere else.
/// Why:
///  The design answers "the exact words" for each state,
///  so the words are `lock_state_words`' own and this check only joins them
///  in the order a person reads them:
///  the lock's line,
///  then the missing pointer's.
///  A lock the add wrote whole is the state the route intends and passes;
///  a lock cut short by a killed write,
///  a person's words,
///  a lost lock,
///  a registration that is gone,
///  unexplained,
///  half made or another worktree's
///  are each a state a person reads and acts on,
///  so each is a finding whose field says which half of the pair to look at.
///  Nothing is written or removed here,
///  and none of these answers changes `doctor`'s exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkManagedLock(facts); // { kind: 'pass', detail } | { kind: 'finding', field: 'lock' | 'registration', detail }
/// ```
pub fn check_managed_lock(facts: &ManagedLockFacts) -> CheckOutcome {
    // What:     `the_lines` answers the design's sentences for this state, one or two of them.
    // Why:      Calling it keeps every sentence in the module that writes words, so this file holds none of its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const words = theLines(facts.classification, facts.slot).join('; ');
    // ```
    let words: String = the_lines(&facts.classification, facts.slot.as_str()).join("; ");
    // What:     The state decides which half of the pair a person looks at, and the verified lock is the one state the route intends.
    // Why:      A finding's field is the words a report prints before the sentence, so it must name the fact that differs rather than the check.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (facts.classification.state.kind) {
    //   case 'managed': return { kind: 'pass', detail: words };
    //   case 'cut-short': case 'not-the-adds-words': case 'no-lock': return { kind: 'finding', field: 'lock', detail: words };
    //   default: return { kind: 'finding', field: 'registration', detail: words };
    // }
    // ```
    return match &facts.classification.state {
        ManagedLockState::Managed {
            ..
        } => pass(words),
        ManagedLockState::CutShort {
            ..
        }
        | ManagedLockState::NotTheAddsWords {
            ..
        }
        | ManagedLockState::NoLock {
            ..
        } => finding("lock", words),
        ManagedLockState::NoRegistration {
            ..
        }
        | ManagedLockState::NoBindingExplains {
            ..
        }
        | ManagedLockState::HalfMade {
            ..
        }
        | ManagedLockState::SomebodyElsesWorktree {
            ..
        } => finding("registration", words),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "managed_lock_checks_tests.rs"]
mod tests;
