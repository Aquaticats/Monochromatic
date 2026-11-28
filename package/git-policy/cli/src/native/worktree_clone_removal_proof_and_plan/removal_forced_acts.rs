//! What:
//!  The acts a removal does only because of a force,
//!  and the lines a person reads about them afterwards.
//! Why:
//!  Two forces pass every refusal this removal has and one force passes the
//!  states the removal can answer for itself,
//!  and three of those acts change a record a person should know about:
//!  a lock this add did not write is deleted,
//!  a destruction plan that cannot be read is replaced and then removed,
//!  and a clone destroyed without a proof is named by both GUIDs.
//!  The project decided the first and the second:
//!  "it proves the clone by the record's GUID against the pool,
//!  destroys the clone and its snapshots from the pool's own list,
//!  deletes the unreadable plan file",
//!  and,
//!  for the wrapper's own lock file,
//!  "the wrapper deletes the person's lock file and then runs the removal"
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
//!  Nothing here decides anything:
//!  each call is made by a step that already read the force count and the state,
//!  so the line and the act cannot disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const acts = ForcedActs.nothing();
//! acts.lines.push(deleteThePersonsLock(registration));
//! ```

use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::{
    Path,
    PathBuf,
};

/// The registration file Git writes a lock into.
/// Why:
///  Git's own words live in `locked` under the registration's directory
///  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
///  "What proves ownership"),
///  and a lock this add did not write is that file and nothing else.
pub const THE_LOCKS_FILE: &str = "locked";

/// What:
///  What this run did only because of a force,
///  in the order a report prints it.
/// Why:
///  A person reads the report and not the code,
///  so every act force made has one line,
///  and the plan file an unreadable plan leaves behind is carried here until
///  the removal's own records go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForcedActs = { lines: string[]; theUnreadablePlan?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ForcedActs {
    /// The lines the report prints about what force did.
    pub lines: Vec<String>,
    /// A plan file this run could not read and removes with its own records.
    pub the_unreadable_plan: Option<PathBuf>,
    /// The clone GUID the add's record held,
    /// when this run destroyed a clone the pool answered with another one.
    pub recorded_guid: Option<String>,
    /// The clone GUID the pool answered,
    /// when it differs from the record's.
    pub answered_guid: Option<String>,
}

impl ForcedActs {
    /// Nothing was forced,
    /// which is every removal whose count is zero or whose states all proved.
    pub fn nothing() -> ForcedActs {
        return ForcedActs::default();
    }

    /// Whether no act of force was made,
    /// so the report prints no line of it.
    pub fn is_nothing(&self) -> bool {
        return self.lines.is_empty()
            && self.the_unreadable_plan.is_none()
            && self.recorded_guid.is_none()
            && self.answered_guid.is_none();
    }
}

/// What:
///  Delete the lock a person left on this slot's registration,
///  and name it for the report.
/// Why:
///  The owner answered on 2026-11-28 that two forces pass a person's lock,
///  and the act stated with that answer is:
///  "the wrapper deletes the person's lock file and then runs the removal"
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
///  "Two forces at a person's lock").
///  A lock that went between the read and this unlink is the same state,
///  so a missing file is no refusal.
///
/// # Throws
/// The unlink that failed,
/// which leaves the lock standing and the removal stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = deleteThePersonsLock(registration); // throws when the lock cannot go
/// ```
pub fn delete_the_persons_lock(registration: &Path) -> Result<String, RemovalStop> {
    let path: PathBuf = registration.join(THE_LOCKS_FILE);
    return match std::fs::remove_file(path.as_path()) {
        Ok(()) => Ok(format!(
            "two forces were given, so this removal deleted the lock at {} that this add did not write, and then removed the worktree",
            path.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(format!(
            "two forces were given, so this removal went on past the lock at {}, which was gone by the time it deleted it",
            path.display()
        )),
        Err(error) => Err(RemovalStop::TheLockCouldNotGo {
            path,
            said: error.to_string(),
        }),
    };
}

/// What:
///  The line for a destruction plan a forced removal went past,
///  and the file it has to remove with its own records.
/// Why:
///  The owner answered on 2026-11-28 that a forced removal
///  "proves the clone by the record's GUID against the pool,
///  destroys the clone and its snapshots from the pool's own list,
///  deletes the unreadable plan file",
///  so the file is named here and removed where every record of this run goes,
///  after the destruction that makes removing it safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const acts = forcedPastAPlan(planPath);
/// ```
pub fn forced_past_a_plan(plan_path: &Path) -> ForcedActs {
    return ForcedActs {
        lines: vec![format!(
            "a force was given, so this removal went past the destruction plan at {}, which could not be read: it proves the clone by the record's GUID against the pool, destroys the clone and its snapshots from the pool's own list, and removes that plan file with its own records",
            plan_path.display()
        )],
        the_unreadable_plan: Some(plan_path.to_path_buf()),
        recorded_guid: None,
        answered_guid: None,
    };
}

/// What:
///  The line for a clone the pool answered with another GUID than the record
///  held,
///  which only two forces pass.
/// Why:
///  The risk is named in the answer this follows:
///  "a dataset that is not this slot's can be destroyed,
///  and nothing can prove afterwards which it was"
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
///  "How far force reaches").
///  The report is the only trace,
///  so it names the dataset destroyed and both GUIDs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const acts = forcedPastAProof({ dataset, recordedGuid, answeredGuid });
/// ```
pub fn forced_past_a_proof(
    dataset: &str,
    recorded_guid: &str,
    answered_guid: &str,
) -> ForcedActs {
    return ForcedActs {
        lines: vec![format!(
            "two forces were given, so this removal destroyed the dataset `{dataset}` even though the pool did not answer the GUID the add's record held: the record held `{recorded_guid}` and the pool answered `{answered_guid}`. Nothing proves which clone that dataset was, and this line is the only record of it."
        )],
        the_unreadable_plan: None,
        recorded_guid: Some(String::from(recorded_guid)),
        answered_guid: Some(String::from(answered_guid)),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_forced_acts_tests.rs"]
mod tests;
