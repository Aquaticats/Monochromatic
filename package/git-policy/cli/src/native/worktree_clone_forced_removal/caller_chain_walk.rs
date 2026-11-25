//! What:
//!  The walk from the command that asked for a forced removal to the first
//!  process,
//!  the two checks every step makes,
//!  and the protected set the walk proves.
//! Why:
//!  The project's answer spares "the caller and the processes of its session"
//!  and, in its fourth answer, "any ancestor of the command up to the first
//!  process",
//!  refusing when one of them holds the clone
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "Which processes a forced removal never ends").
//!  A number can be reused between two reads,
//!  so each step re-reads the child's recorded parent after reading the
//!  parent's own record and compares the parent's start time with the child's,
//!  which is what the design's walk asks
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The caller and its protected set").
//!  The walk stops at the first process because nothing above it is a process
//!  of this machine's lifetime,
//!  and it is bounded like the crate's own `chain_ancestors`, whose loop bound
//!  guards a tree that cannot be walked rather than a depth a real chain
//!  reaches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const chain = walkTheCallerChain(readRecord, identity);   // throws a refusal
//! chain.spares(holderRecord);
//! ```

use crate::worktree_clone_forced_removal::caller_property::CallerIdentity;
use crate::worktree_clone_forced_removal::process_session_record::{ProcessRecord, ReadOutcome};

/// How many ancestors one walk asks for.
/// Why:
///  The crate's own chain walk uses the same bound for the same reason
///  (`invocation/started_as_git.rs`,
///  `MOST_ANCESTORS_ASKED_FOR_THE_RECORD`):
///  it guards a tree that cannot be walked,
///  and no chain of finishable depth reaches it.
pub const MOST_ANCESTORS_ASKED_FOR: usize = 4096;

/// What:
///  Why the walk could not prove the protected set.
/// Why:
///  The handler never signals or unmounts anything before the walk succeeds,
///  so each cause is a refusal of the whole forced removal,
///  and the report has to name which proof failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChainRefusal = { kind: 'caller-gone' | 'parent-gone' | 'parent-changed' | 'start-order' | 'blind' | 'too-deep'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChainRefusal {
    /// The wrapper's number does not name the process the property recorded.
    CallerGone,
    /// The recorded parent has gone,
    /// or its number now names another process.
    ParentGone,
    /// The wrapper's record names another parent than the property's,
    /// read again after the parent's own record was read.
    ParentChanged {
        /// The parent the property recorded.
        recorded: i32,
        /// The parent the wrapper's record names now.
        found: i32,
    },
    /// A parent's start time is later than its child's,
    /// which the kernel cannot record for a real parent.
    StartOrder {
        /// The child whose record was read.
        child: i32,
        /// The child's start time.
        child_started: u64,
        /// The parent's start time.
        parent_started: u64,
    },
    /// A read the walk needed was refused,
    /// so the set it would prove is unknown.
    Blind {
        /// The process whose record was refused.
        pid: i32,
    },
    /// The chain is longer than one walk asks for,
    /// or a record names itself as its own parent.
    TooDeep,
}

/// What:
///  The processes a forced removal never ends,
///  proved from the command that asked.
/// Why:
///  A holder in this set refuses the removal in both arms,
///  and the report names which half of the set held it,
///  because "leave the folder" answers a working directory and does not
///  release a mount namespace's copy
///  (the design's two refusal texts).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CallerChain = { caller; session; ancestors: { pid; started }[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerChain {
    /// The wrapper's own process.
    pub caller: ProcessRecord,
    /// Every ancestor from the wrapper's parent to the first process,
    /// nearest first.
    pub ancestors: Vec<ProcessRecord>,
}

impl CallerChain {
    /// What:
    ///  Whether a holder is one this removal never ends.
    /// Why:
    ///  The three halves are compared differently on purpose:
    ///  the caller and the ancestors by number and start time,
    ///  because a number can be reused,
    ///  and the session by number alone,
    ///  because the kernel gives a session one leader for its life and the
    ///  design spares every task in it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// holder.pid === caller.pid || holder.session === caller.session || ancestors.some(a => a.pid === holder.pid && a.started === holder.started);
    /// ```
    pub fn spares(&self, holder: &ProcessRecord) -> bool {
        if holder.pid == self.caller.pid && holder.started == self.caller.started {
            return true;
        }
        if holder.session == self.caller.session {
            return true;
        }
        return self.ancestors.iter().any(|ancestor| {
            return ancestor.pid == holder.pid && ancestor.started == holder.started;
        });
    }

    /// Which half of the set holds this holder,
    /// for the report's own sentence.
    pub fn why_spared(&self, holder: &ProcessRecord) -> Option<SparedBy> {
        if holder.pid == self.caller.pid && holder.started == self.caller.started {
            return Some(SparedBy::Caller);
        }
        if holder.session == self.caller.session {
            return Some(SparedBy::Session);
        }
        if self.ancestors.iter().any(|ancestor| {
            return ancestor.pid == holder.pid && ancestor.started == holder.started;
        }) {
            return Some(SparedBy::Ancestor);
        }
        return None;
    }
}

/// What:
///  Which half of the protected set spares a holder.
/// Why:
///  The two refusal sentences differ:
///  a session's holder is told to leave the folder,
///  and a namespace's copy is not released by leaving it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SparedBy = 'caller' | 'session' | 'ancestor';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparedBy {
    /// The command's own process.
    Caller,
    /// A task of the command's session.
    Session,
    /// A process the command descends from.
    Ancestor,
}

/// What:
///  Walk from the recorded caller to the first process,
///  proving each step,
///  and answer the protected set.
/// Why:
///  The handler proves the wrapper's own identity and its recorded parent before
///  it walks,
///  and each step re-reads the child after reading the parent,
///  so a parent that ended and whose number passed to another process is caught
///  rather than spared as an ancestor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let child = readTheCaller(identity); const ancestors = [];
/// while (child.parent > 1) { const parent = read(child.parent); reRead(child); ancestors.push(parent); child = parent; }
/// ```
pub fn walk_the_caller_chain<Read>(read: Read, identity: &CallerIdentity) -> Result<CallerChain, ChainRefusal>
where
    Read: Fn(i32) -> ReadOutcome,
{
    let caller: ProcessRecord = match read(identity.pid) {
        ReadOutcome::Found(record) if record.started == identity.started => record,
        ReadOutcome::Found(_record) => return Err(ChainRefusal::CallerGone),
        ReadOutcome::Gone => return Err(ChainRefusal::CallerGone),
        ReadOutcome::Blind { .. } => return Err(ChainRefusal::Blind { pid: identity.pid }),
    };
    if caller.parent != identity.parent {
        return Err(ChainRefusal::ParentChanged { recorded: identity.parent, found: caller.parent });
    }
    let mut ancestors: Vec<ProcessRecord> = Vec::new();
    let mut child: ProcessRecord = caller.clone();
    let mut reached_the_first_process: bool = false;
    for _step in 0..MOST_ANCESTORS_ASKED_FOR {
        if child.parent <= 1 {
            reached_the_first_process = true;
            break;
        }
        let parent: ProcessRecord = match read(child.parent) {
            ReadOutcome::Found(record) => record,
            ReadOutcome::Gone => return Err(ChainRefusal::ParentGone),
            ReadOutcome::Blind { .. } => return Err(ChainRefusal::Blind { pid: child.parent }),
        };
        if parent.started > child.started {
            return Err(ChainRefusal::StartOrder {
                child: child.pid,
                child_started: child.started,
                parent_started: parent.started,
            });
        }
        if parent.pid == identity.parent {
            // The one hop the property records is proved against its own start time, so a number a later process took
            // is not read as the wrapper's parent.
            if parent.started != identity.parent_started {
                return Err(ChainRefusal::ParentGone);
            }
        }
        let again: ProcessRecord = match read(child.pid) {
            ReadOutcome::Found(record) => record,
            ReadOutcome::Gone => return Err(ChainRefusal::ParentGone),
            ReadOutcome::Blind { .. } => return Err(ChainRefusal::Blind { pid: child.pid }),
        };
        if again.parent != parent.pid || again.started != child.started {
            return Err(ChainRefusal::ParentChanged { recorded: child.parent, found: again.parent });
        }
        if ancestors.iter().any(|seen| return seen.pid == parent.pid) {
            return Err(ChainRefusal::TooDeep);
        }
        ancestors.push(parent.clone());
        child = parent;
    }
    if !reached_the_first_process {
        return Err(ChainRefusal::TooDeep);
    }
    return Ok(CallerChain { caller, ancestors });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "caller_chain_walk_tests.rs"]
mod tests;
