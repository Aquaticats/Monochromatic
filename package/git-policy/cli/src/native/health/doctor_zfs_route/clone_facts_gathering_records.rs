//! What:
//!  The two records one clone's fact set reads beside its state:
//!  the Git registration its own record names,
//!  and the origin the record names with the mark that origin's snapshot
//!  carries.
//! Why:
//!  The record names both paths and nothing else does,
//!  so a slot whose record names none has neither to report,
//!  and a link or a snapshot this account may not read is that fact's own
//!  unknown answer rather than a state nobody proved
//!  (plan step 18,
//!  "damaged or pruned Git registrations" and
//!  "the deferred-origin check through the `clone-origin` line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registration = registrationFacts(request, reads, record);
//! const origin = originFacts(reads, clone, record);
//! ```

/// Every read one clone's facts come from,
/// and the request they answer.
use super::clone_facts_reads::{
    CloneFactsReads,
    CloneFactsRequest,
};
/// The origin an add recorded and the mark on it.
use super::clone_origin_checks::{
    CloneOriginFacts,
    RecordedOrigin,
};
/// What stands at one path,
/// and what a read of one object answered.
use super::handler_checks::PathState;
/// The Git registration one slot's record names.
use super::registration_checks::RegistrationFacts;
use super::zfs_checks::Reading;
/// The record one add wrote.
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use std::path::PathBuf;

/// What:
///  The Git registration one slot's record names and what stands at it.
/// Why:
///  The record names the registration's path and nothing else does,
///  so a slot whose record names none has no registration to report,
///  and a slot whose registration is recorded but whose `gitdir` file this
///  account may not read proves nothing about the link:
///  its answer is unknown at the registration's own path rather than the
///  half-made registration a missing file means.
pub(crate) fn registration_facts(
    request: &CloneFactsRequest,
    reads: &dyn CloneFactsReads,
    record: Option<&BindingRecord>,
) -> RegistrationFacts {
    let slot_git: PathBuf = request.slot_path.join(".git");
    let Some(registration) = record.and_then(|held| return held.registration_path()) else {
        return RegistrationFacts {
            slot: String::from(request.slot.as_str()),
            recorded: None,
            at_the_path: PathState::Absent,
            gitdir: None,
            slot_git,
        };
    };
    let mut at_the_path: PathState = reads.path_state(registration.as_path());
    let gitdir: Option<PathBuf> = match reads.registration_gitdir(registration.as_path()) {
        Reading::Read(named) => Some(named),
        Reading::Missing {
            ..
        } => None,
        Reading::Unreadable {
            why,
        } => {
            at_the_path = PathState::Unreadable {
                why: format!(
                    "the registration {} stands, and its `gitdir` file could not be read ({why})",
                    registration.display()
                ),
            };
            None
        }
    };
    return RegistrationFacts {
        slot: String::from(request.slot.as_str()),
        recorded: Some(registration),
        at_the_path,
        gitdir,
        slot_git,
    };
}

/// What:
///  The origin the record names and the mark its snapshot carries.
/// Why:
///  A record that names no origin snapshot has nothing to read,
///  and one that names it is always read:
///  a read that failed is the mark's own unreadable answer,
///  never an absent mark.
pub(crate) fn origin_facts(
    reads: &dyn CloneFactsReads,
    clone: &str,
    record: Option<&BindingRecord>,
) -> CloneOriginFacts {
    let recorded: Option<RecordedOrigin> = record.and_then(|held| {
        let dataset: String = held.origin_dataset.clone()?;
        return Some(RecordedOrigin {
            dataset,
            snapshot: held.origin_snapshot.clone(),
        });
    });
    let live = recorded
        .as_ref()
        .map(|origin| return reads.origin_mark(origin.snapshot.as_str()));
    return CloneOriginFacts {
        clone: String::from(clone),
        recorded,
        live,
    };
}
