//! What:
//!  Model of M1's lock handling with comparison and removal as two steps, because Linux has no call that compares a pathname's file
//!  identity and unlinks it in one operation: a recovery that observed the journaled identity at `index.lock` and unlinks later, a live
//!  creator's own normal abort (`fstat` against `stat`, then `unlink`), and a foreign program that replaces the lock in between.
//! Why:
//!  Superseded for a lock of the lane's own dead incarnation: the owner decided on 2026-11-04 ("Reclaim it (Recommended)") that the lane
//!  removes a proven-stale lock of its own dead incarnation, and `model_m1_reclaim_r19.rs` models that. This model stays as the control of
//!  the split compare and unlink and of a live creator's own abort; its revision 13 rule (never remove a lock found without a live creator)
//!  is the earlier choice and is a labelled historical rule, not the current one.
//!  Revision 12's text allowed a recovery to remove the lock when its journaled identity matched; the match and the removal are two
//!  system calls, and the file at the name can be replaced between them. Revision 13 never removes a lock found without a live creator
//!  (it reports and defers), and keeps unlink only for a live creator's own lock, where the premise of every Git lock (no program
//!  removes a lock it did not create while its creator lives) narrows the window and cannot close it. The model reaches that residue
//!  only through a step labelled as breaking the premise, so the claim is exactly the stated one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Lock = { rules; lock: 'absent' | 'mine' | 'foreign'; creatorAlive; observed; premiseBroken; removedForeign };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12: a recovery unlinks the lock after matching the journaled identity.
    Twelve,
    /// Revision 13: a lock without a live creator is never removed; a live creator removes only its own, after comparing.
    Thirteen,
}

/// Who made the file at `index.lock`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Holder {
    /// No lock file.
    Absent,
    /// The lane's own creator made it (the journaled identity).
    Mine,
    /// Another program made it.
    Foreign,
}

/// The lock handling state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Lock {
    /// The rule under test.
    pub rules: Rules,
    /// Who made the file now at the name.
    pub holder: Holder,
    /// The creator lives.
    pub creator_alive: bool,
    /// The recovery or the creator compared the identity and found the lane's own file (the comparison step happened).
    pub matched: bool,
    /// A foreign program replaced the lock while the creator lived (the premise of every Git lock is broken).
    pub premise_broken: bool,
    /// A foreign lock was removed by the lane.
    pub removed_foreign: bool,
}

/// The start state: the lane's creator holds its lock.
pub fn initial(rules: Rules) -> Lock {
    return Lock { rules, holder: Holder::Mine, creator_alive: true, matched: false, premise_broken: false, removed_foreign: false };
}

/// Every step enabled.
pub fn steps(state: &Lock) -> Vec<(String, Lock)> {
    let mut found: Vec<(String, Lock)> = Vec::new();
    if state.creator_alive {
        let mut after: Lock = state.clone();
        after.creator_alive = false;
        push_step(&mut found, "environment: the creator dies with its lock file in place", after);
    }
    if state.holder == Holder::Mine && !state.matched && state.creator_alive {
        let mut after: Lock = state.clone();
        after.holder = Holder::Absent;
        push_step(&mut found, "environment: the creator's own release removes the lock", after);
    }
    if state.holder != Holder::Foreign {
        let mut after: Lock = state.clone();
        after.holder = Holder::Foreign;
        // Only replacing a live creator's own lock breaks the premise; acquiring after a release or after the creator died does not.
        after.premise_broken = state.premise_broken || (state.holder == Holder::Mine && state.creator_alive);
        push_step(&mut found, "environment: a foreign program replaces the file at the name", after);
    }
    if !state.matched && state.holder == Holder::Mine && (state.creator_alive || state.rules == Rules::Twelve) {
        let mut after: Lock = state.clone();
        after.matched = true;
        push_step(&mut found, "owner: compares the file identity with the journaled one and they agree", after);
    }
    // The creator's own unlink is a step of the creator, so it needs the creator alive; revision 12's recovery was a step of the lane.
    if state.matched && (state.creator_alive || state.rules == Rules::Twelve) {
        let mut after: Lock = state.clone();
        after.removed_foreign = state.holder == Holder::Foreign;
        after.holder = Holder::Absent;
        after.matched = false;
        push_step(&mut found, "owner: unlinks the pathname", after);
    }
    return found;
}

/// The lane never removes a foreign lock unless the premise of the lock was broken first.
pub fn invariant(state: &Lock) -> Option<String> {
    if state.removed_foreign && !state.premise_broken {
        return Some("the lane removed a foreign lock after a comparison, with no broken premise to blame".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 20 };
}

/// Explores M1's lock handling.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, |_| { return None; }, limits());
}
