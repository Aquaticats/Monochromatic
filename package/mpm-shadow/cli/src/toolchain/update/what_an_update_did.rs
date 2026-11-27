//! What:
//! What an `update` run did:
//! one outcome per plan tool,
//! and the report that says whether a lock was written and which tools an auto-sync installed.
//! Why:
//! `update` and `outdated` print one line per tool from this report,
//! and both decide their exit status from what it holds,
//! so the type and the two questions asked of it live together.
//! A caller imports from `update` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type OutcomeKind = 'unchanged' | 'updated' | 'added' | 'pinned' | 'kept' | 'failed';
//! export type Outcome = { id: string; kind: OutcomeKind };
//! export type UpdateReport = { outcomes: Outcome[]; wrote: boolean; synced: string[] };
//! ```
/// What happened to one tool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    /// The lookup found the locked version and artifacts again.
    Unchanged,
    /// The version or artifacts moved.
    Updated {
        /// The locked version before.
        old_version: String,
        /// The version found.
        new_version: String,
    },
    /// The tool was not in the lock;
    /// this is its first resolution.
    Added {
        /// The version found.
        version: String,
    },
    /// The tool is pinned and the lock already holds that pin.
    Pinned,
    /// A named `update` did not select this tool:
    /// no lookup was made and its row keeps its version and artifacts.
    Kept,
    /// The lookup failed;
    /// nothing is written.
    Failed {
        /// Why,
        /// naming the URL or the field.
        reason: String,
    },
}

/// One line of the report:
/// a tool and what happened.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    /// Tool id.
    pub id: String,
    /// What happened.
    pub kind: OutcomeKind,
}

/// What `update` did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateReport {
    /// One outcome per plan tool,
    /// in plan order.
    pub outcomes: Vec<Outcome>,
    /// Whether `toolchain.lock.json` was written.
    pub wrote: bool,
    /// Tool ids the auto-sync installed before a retry.
    pub synced: Vec<String>,
}

/// The methods of `UpdateReport`.
impl UpdateReport {
    /// Whether any lookup failed.
    pub fn has_failures(&self) -> bool {
        for outcome in &self.outcomes {
            if matches!(outcome.kind, OutcomeKind::Failed { .. }) {
                return true;
            }
        }
        return false;
    }

    /// Whether any tool moved or was added (the condition `outdated` exits 1 for).
    pub fn has_movement(&self) -> bool {
        for outcome in &self.outcomes {
            if matches!(
                outcome.kind,
                OutcomeKind::Updated { .. } | OutcomeKind::Added { .. }
            ) {
                return true;
            }
        }
        return false;
    }
}
