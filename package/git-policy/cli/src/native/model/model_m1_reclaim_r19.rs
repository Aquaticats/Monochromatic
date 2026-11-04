//! What:
//!  Model of the lane reclaiming a proven-stale `index.lock` of its own dead incarnation, as the owner decided on 2026-11-04 ("Reclaim it
//!  (Recommended)"), replacing the earlier choice that the lane never removes it. The steps are the observable steps of the shipped
//!  mechanism: a reclaimer proves the holder gone and remembers which file it judged; exactly one reclaimer may proceed (an exclusive
//!  `flock` on the stale file's own inode where `flock` works, a claim chain of files beside the lock where it does not); the holder
//!  checks that the name still refers to the judged file; and only then unlinks the name. Check and unlink are two steps, because Linux
//!  has no call that removes a name only if it still names a given file; a landing, a foreign program and a person can act between them.
//! Why:
//!  The checks are the two properties the owner kept: no foreign lock is removed and a live lock is never removed, each unless the premise
//!  of every Git lock was broken first (a person removed the stale lock by hand). The other check is the reason for the change: at
//!  quiescence a stale lock of the lane's own dead incarnation is not left behind unless a reclaimer died holding its claim. The
//!  mechanism's exact steps are the branch `fix/git-policy-cli-index-lock-reclaim-without-flock` at `b071e48f74`, not yet reviewed; this
//!  model fixes the steps named above and the part that waits for that review is the claim chain's slot protocol (the slot scan and the
//!  re-read of lower slots are modeled as the document describes them, with claimers that can die).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // reclaim: judge(stale) -> hold (flock | claim chain) -> check(name is the judged file) -> unlink(name) -> release
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The number of reclaimers (two lanes' services or commands that found the same stale lock).
pub const RECLAIMERS: usize = 2;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Superseded: the lane never removes the stale lock and reports it (the earlier choice R-never).
    NeverRemove,
    /// Current: every reclaimer holds an exclusive `flock` on the stale file's inode.
    Flock,
    /// Current: no reclaimer has a working `flock`; every reclaimer uses the claim chain.
    ClaimChain,
    /// The stated residue: one reclaimer's `flock` works and the other's does not (two mounts of one file system), so one holds the
    /// `flock` and the other a claim, and neither sees the other.
    Mixed,
    /// Control: no exclusion between reclaimers.
    NoHold,
    /// Control: the holder unlinks the name without checking that it still refers to the judged file.
    NoNameCheck,
    /// Control: a reclaimer judges any lock at the name, without proving its holder gone.
    Unproven,
}

/// Who made the file now at `index.lock`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    /// No lock.
    Absent,
    /// The lane's own dead incarnation's lock: its holder is proven gone.
    Stale,
    /// A live landing's lock.
    Live,
    /// A foreign program's lock.
    Foreign,
}

/// A reclaimer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rec {
    /// Has not looked.
    Idle,
    /// Judged the file at the name (the identity it will compare against).
    Judged(Name),
    /// Placed its claim at this slot and has not re-read the lower slots.
    Claimed(u8, Name),
    /// Holds the exclusion; the check is next.
    Holding(Name, Option<u8>),
    /// Checked that the name refers to the judged file; the unlink is next.
    Checked(Option<u8>),
    /// Finished.
    Done,
    /// Backed off.
    BackedOff,
    /// Died (its `flock` is gone with it; its claim stays as a gone claim).
    Dead,
}

/// A claim file beside the lock.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Claim {
    /// No file at the slot.
    Free,
    /// A running claimer's claim.
    Live(u8),
    /// A claim whose claimer is gone.
    Gone,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The file at the name.
    pub name: Name,
    /// The reclaimers.
    pub recs: [Rec; RECLAIMERS],
    /// The reclaimer that holds the `flock`.
    pub flock_holder: Option<u8>,
    /// The claim slots (one per reclaimer).
    pub claims: [Claim; RECLAIMERS],
    /// A landing placed its live lock once.
    pub landing_placed: bool,
    /// A foreign program placed its lock once.
    pub foreign_placed: bool,
    /// A person removed the stale lock by hand once: the premise of every Git lock is broken from then on.
    pub premise_broken: bool,
    /// A reclaimer died once.
    pub died: bool,
    /// A live lock was unlinked by a reclaimer.
    pub removed_live: bool,
    /// A foreign lock was unlinked by a reclaimer.
    pub removed_foreign: bool,
}

/// The start state: the dead incarnation's lock is at the name and nobody has looked.
pub fn initial(rules: Rules) -> State {
    return State {
        rules,
        name: Name::Stale,
        recs: [Rec::Idle; RECLAIMERS],
        flock_holder: None,
        claims: [Claim::Free; RECLAIMERS],
        landing_placed: false,
        foreign_placed: false,
        premise_broken: false,
        died: false,
        removed_live: false,
        removed_foreign: false,
    };
}

/// Whether reclaimer `at` uses the claim chain.
fn uses_claims(rules: Rules, at: usize) -> bool {
    return match rules {
        Rules::ClaimChain => true,
        Rules::Mixed => at == 1,
        _ => false,
    };
}

/// Ends reclaimer `at`'s hold: releases the `flock`, and removes the claims from slot zero to its own (the name no longer refers to the
/// judged file).
fn release(state: &mut State, at: usize, slot: Option<u8>) {
    if state.flock_holder == Some(at as u8) {
        state.flock_holder = None;
    }
    if let Some(own) = slot {
        for claim in state.claims.iter_mut().take(usize::from(own) + 1) {
            *claim = Claim::Free;
        }
    }
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for at in 0..RECLAIMERS {
        rec_steps(state, at, &mut found);
    }
    if state.name == Name::Absent && !state.landing_placed {
        let mut after: State = state.clone();
        after.name = Name::Live;
        after.landing_placed = true;
        push_step(&mut found, "environment: a live landing places its lock", after);
    }
    if state.name == Name::Absent && !state.foreign_placed {
        let mut after: State = state.clone();
        after.name = Name::Foreign;
        after.foreign_placed = true;
        push_step(&mut found, "environment: a foreign program places its lock", after);
    }
    if state.name == Name::Stale && !state.premise_broken {
        let mut after: State = state.clone();
        after.name = Name::Absent;
        after.premise_broken = true;
        push_step(&mut found, "environment: a person removes the stale lock by hand", after);
    }
    return found;
}

fn rec_steps(state: &State, at: usize, found: &mut Vec<(String, State)>) {
    let with = |next: Rec| -> State {
        let mut after: State = state.clone();
        after.recs[at] = next;
        return after;
    };
    match state.recs[at] {
        Rec::Idle => {
            if state.rules == Rules::NeverRemove {
                return;
            }
            let judges: bool = state.name == Name::Stale || (state.rules == Rules::Unproven && state.name == Name::Live);
            if judges {
                push_step(found, &format!("reclaimer {}: proves the holder gone and judges the file at the name", at), with(Rec::Judged(state.name)));
            }
        }
        Rec::Judged(judged) => {
            if state.rules == Rules::NoHold {
                push_step(found, &format!("reclaimer {}: proceeds with no exclusion", at), with(Rec::Holding(judged, None)));
            } else if uses_claims(state.rules, at) {
                let mut slot: Option<usize> = None;
                let mut backs_off: bool = false;
                for (number, claim) in state.claims.iter().enumerate() {
                    match claim {
                        Claim::Gone => continue,
                        Claim::Live(_) => {
                            backs_off = true;
                            break;
                        }
                        Claim::Free => {
                            slot = Some(number);
                            break;
                        }
                    }
                }
                match (slot, backs_off) {
                    (Some(number), false) => {
                        let mut after: State = with(Rec::Claimed(number as u8, judged));
                        after.claims[number] = Claim::Live(at as u8);
                        push_step(found, &format!("reclaimer {}: places its claim at slot {}", at, number), after);
                    }
                    _ => push_step(found, &format!("reclaimer {}: finds a running claimer and backs off", at), with(Rec::BackedOff)),
                }
            } else if state.flock_holder.is_none() {
                let mut after: State = with(Rec::Holding(judged, None));
                after.flock_holder = Some(at as u8);
                push_step(found, &format!("reclaimer {}: takes the exclusive flock on the stale file", at), after);
            } else {
                push_step(found, &format!("reclaimer {}: finds the flock held and backs off", at), with(Rec::BackedOff));
            }
        }
        Rec::Claimed(own, judged) => {
            let lower_gone: bool = state.claims.iter().take(usize::from(own)).all(|claim| { return *claim == Claim::Gone; });
            if lower_gone {
                push_step(found, &format!("reclaimer {}: re-reads the lower slots, all gone, and holds", at), with(Rec::Holding(judged, Some(own))));
            } else {
                let mut after: State = with(Rec::BackedOff);
                after.claims[usize::from(own)] = Claim::Free;
                push_step(found, &format!("reclaimer {}: re-reads the lower slots, finds one not gone, removes its claim and backs off", at), after);
            }
        }
        Rec::Holding(judged, slot) => {
            if state.rules == Rules::NoNameCheck || state.name == judged {
                push_step(found, &format!("reclaimer {}: checks that the name still refers to the judged file", at), with(Rec::Checked(slot)));
            } else {
                let mut after: State = with(Rec::Done);
                release(&mut after, at, slot);
                push_step(found, &format!("reclaimer {}: finds the name no longer refers to the judged file and leaves", at), after);
            }
        }
        Rec::Checked(slot) => {
            let mut after: State = with(Rec::Done);
            after.removed_live = after.removed_live || state.name == Name::Live;
            after.removed_foreign = after.removed_foreign || state.name == Name::Foreign;
            after.name = Name::Absent;
            release(&mut after, at, slot);
            push_step(found, &format!("reclaimer {}: unlinks whatever is at the name and releases", at), after);
        }
        Rec::Done | Rec::BackedOff | Rec::Dead => {}
    }
    if !state.died && matches!(state.recs[at], Rec::Claimed(..) | Rec::Holding(..) | Rec::Checked(_)) {
        let mut after: State = with(Rec::Dead);
        after.died = true;
        if state.flock_holder == Some(at as u8) {
            after.flock_holder = None;
        }
        for claim in after.claims.iter_mut() {
            if *claim == Claim::Live(at as u8) {
                *claim = Claim::Gone;
            }
        }
        push_step(found, &format!("environment: reclaimer {} dies", at), after);
    }
}

/// The safety checks: neither a live lock nor a foreign lock is unlinked unless a person broke the premise by hand first, and two
/// reclaimers never hold at once.
pub fn invariant(state: &State) -> Option<String> {
    if state.removed_live && !state.premise_broken {
        return Some("a live lock was removed by a reclaimer".to_string());
    }
    if state.removed_foreign && !state.premise_broken {
        return Some("a foreign lock was removed by a reclaimer".to_string());
    }
    let holders: usize = state.recs.iter().filter(|rec| { return matches!(rec, Rec::Holding(..) | Rec::Checked(_)); }).count();
    if holders > 1 {
        return Some("two reclaimers hold at once".to_string());
    }
    return None;
}

/// The reason for the change, conditionally: a quiescent state does not leave the dead incarnation's lock behind unless a reclaimer died.
/// This is not eventual reclamation in general: after a death the lock may stay (the model has two one-shot reclaimers and one death, and
/// no retry), and a stale lock left after a death is accepted here only because a later command's reclaimer is outside the bound.
pub fn terminal(state: &State) -> Option<String> {
    if state.name == Name::Stale && !state.died {
        return Some("a proven-stale lock of the lane's own dead incarnation was left although no reclaimer died".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 40 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, terminal, limits());
}
