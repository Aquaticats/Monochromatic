//! What:
//!  Model of C2's `bound` step (revision 14, R13-1): the order of the client's lock on `lane/client/<id>` and the `link` that exposes the
//!  request in `inbox/`, an owner that adopts what it finds linked, and the command's exit on each side of the link.
//! Why:
//!  Revision 13 left the order open, so the request could be linked before the client held its lock. An owner that adopted it in that
//!  window saw no client (the lock is how a lane tells an attached command from a gone one) and decided it as a request with no client,
//!  writing `commit`; the command then took its lock, reached its bound and withdrew with an error while the commit landed. Revision 14
//!  takes the lock first and defines `bound` as the link: an exit before the link has no request (nothing to withdraw, nothing to
//!  adopt), and an exit after it has a request that an owner sees as attached while the process lives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Bound = { rules; client: 'init' | 'locked' | 'linked' | 'waiting' | 'withdrawn' | 'exited'; lockHeld; linked; ownerCommitted };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 13: the order of the lock and the link is open; the link comes first.
    Thirteen,
    /// Revision 14: the client lock first, then the link, which is `bound`.
    Fourteen,
}

/// Where the command is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Client {
    /// Before anything.
    Init,
    /// Holds the lock, request not linked (revision 14).
    Locked,
    /// Request linked, lock not held (revision 13).
    Linked,
    /// Holds the lock and the request is linked: waiting at its bound.
    Waiting,
    /// Withdrew and returned the withdrawal error.
    Withdrawn,
    /// Exited before the link: no request exists.
    ExitedBeforeLink,
}

/// The state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bound {
    /// The rule under test.
    pub rules: Rules,
    /// The command.
    pub client: Client,
    /// The client holds `lane/client/<id>`.
    pub lock_held: bool,
    /// The request is linked in `inbox/<id>`.
    pub linked: bool,
    /// The owner adopted the request and decided it as one with no client (it wrote `commit`).
    pub owner_committed: bool,
}

/// The start state.
pub fn initial(rules: Rules) -> Bound {
    return Bound { rules, client: Client::Init, lock_held: false, linked: false, owner_committed: false };
}

/// Every step enabled.
pub fn steps(state: &Bound) -> Vec<(String, Bound)> {
    let mut found: Vec<(String, Bound)> = Vec::new();
    let thirteen: bool = state.rules == Rules::Thirteen;
    if state.client == Client::Init {
        if thirteen {
            let mut after: Bound = state.clone();
            after.client = Client::Linked;
            after.linked = true;
            push_step(&mut found, "client: links the request into inbox/", after);
        } else {
            let mut after: Bound = state.clone();
            after.client = Client::Locked;
            after.lock_held = true;
            push_step(&mut found, "client: takes the client lock", after);
        }
        let mut exit: Bound = state.clone();
        exit.client = Client::ExitedBeforeLink;
        push_step(&mut found, "client: exits before anything (an interrupt)", exit);
    }
    if state.client == Client::Linked {
        let mut after: Bound = state.clone();
        after.client = Client::Waiting;
        after.lock_held = true;
        push_step(&mut found, "client: takes the client lock", after);
    }
    if state.client == Client::Locked {
        let mut after: Bound = state.clone();
        after.client = Client::Waiting;
        after.linked = true;
        push_step(&mut found, "client: links the request into inbox/ (bound)", after);
        let mut exit: Bound = state.clone();
        exit.client = Client::ExitedBeforeLink;
        exit.lock_held = false;
        push_step(&mut found, "client: exits holding the lock and before the link", exit);
    }
    if state.client == Client::Waiting {
        let mut after: Bound = state.clone();
        after.client = Client::Withdrawn;
        push_step(&mut found, "W1: the bound is reached; the command withdraws and returns the error", after);
    }
    if state.linked && !state.lock_held && !state.owner_committed {
        let mut after: Bound = state.clone();
        after.owner_committed = true;
        push_step(&mut found, "owner: adopts the request, sees no client lock and decides it as clientless", after);
    }
    return found;
}

/// A command that returned the withdrawal error never has its request decided by the owner as one with no client.
pub fn invariant(state: &Bound) -> Option<String> {
    if state.client == Client::Withdrawn && state.owner_committed {
        return Some("the command returned the withdrawal error and the owner decided the request as clientless".to_string());
    }
    if state.client == Client::ExitedBeforeLink && state.linked {
        return Some("the command exited before the link and a request exists".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 1_000, max_depth: 12 };
}

/// Explores every interleaving.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, |_| { return None; }, limits());
}
