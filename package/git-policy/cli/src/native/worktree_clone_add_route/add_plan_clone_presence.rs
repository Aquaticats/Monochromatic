//! What:
//!  The pool's answer about the clone one standing removal plan records:
//!  whether a child of the clone container still carries its GUID,
//!  whether its recorded name is free,
//!  and what the pool said instead when it could not prove either.
//! Why:
//!  The command a person reads beside a standing plan follows from the plan's
//!  head word and this answer together
//!  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
//!  choice 2C),
//!  and the proof of one clone's absence must be the removal's own,
//!  or the add and the removal could disagree about what "gone" means
//!  (`worktree_clone_record_settling::settling_absence`).
//!  So this file asks the removal's proof and translates its answer,
//!  rather than walking the container a second way.
//!  The walk is bounded here:
//!  the add waits on it, and the step 8 record treats growth on a waiting path
//!  as a defect, so a container with more children than one sweep takes answers
//!  "the absence cannot be proved" instead of walking for ever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const presence = thePresenceOfThePlansClone(probe, plan);
//! ```

use super::add_zfs::OpenedDevice;
use crate::worktree_clone_binding_record::add_state_commands::ClonePresence;
use crate::worktree_clone_binding_record::plan_folder_read::APlanThatNamesTheClone;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, MAXIMUM_CHILDREN, ReadFailure};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{CloneAbsence, the_clone_absence};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, ListedObject};

/// What:
///  The pool's answer about the clone a standing plan records,
///  in the words the command list reads.
/// Why:
///  The removal's proof answers five ways about a clone,
///  and the command list needs four:
///  a clone that stands under its name or under another name is the same clone,
///  a proved absence is its own arm,
///  a name that answers another clone is the state no command settles,
///  and everything the pool refused is one unproved absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const presence = thePresenceOfThePlansClone(probe, plan);
/// ```
pub fn the_presence_of_the_plans_clone(probe: &dyn DatasetReader, plan: &APlanThatNamesTheClone) -> ClonePresence {
    // What:     `TheBoundedListing { .. }` builds the walker that counts the
    //           children it hands on, borrowing the caller's pool reads rather
    //           than owning them.
    // Why:      The removal's proof walks a container without an end, and this
    //           add must not wait on one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bounded = new BoundedListing(probe);
    // ```
    let bounded: TheBoundedListing<'_> = TheBoundedListing { probe, handed: std::cell::Cell::new(0) };
    let answer: Result<CloneAbsence, crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop> =
        the_clone_absence(&bounded, plan.clone_dataset.as_str(), plan.clone_guid);
    return match answer {
        // The recorded name answered with the recorded GUID,
        // and the plan's clone stands where the plan says it does.
        Ok(CloneAbsence::There { .. }) => ClonePresence::Stands,
        // The recorded name answers another clone,
        // which is the one state no command of this release settles.
        Ok(CloneAbsence::Reused { guid }) => ClonePresence::TheNameAnswersAnotherClone { live_guid: guid.file_name() },
        Ok(CloneAbsence::Gone) => ClonePresence::GoneProved,
        Err(stop) => ClonePresence::GoneUnprovable { evidence: stop.message() },
    };
}

/// What:
///  The pool reads one walk of the container,
///  handing on no more children than the sweep takes.
/// Why:
///  The bound is the sweep's own number and lives here so the walker owns it,
///  and a container past it answers the same refusal a sweep's listing does:
///  nothing proves the clone gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BoundedListing { constructor(private probe: DatasetReader) {} nextChild(parent, cursor) { ... } }
/// ```
struct TheBoundedListing<'a> {
    /// The pool reads this walk asks.
    probe: &'a dyn DatasetReader,
    /// How many children this walk has already handed on.
    handed: std::cell::Cell<usize>,
}

impl DatasetReader for TheBoundedListing<'_> {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        return self.probe.identity(dataset);
    }

    fn next_child(&self, parent: &str, cursor: u64) -> Result<Option<ListedChild>, String> {
        let step: Option<ListedChild> = self.probe.next_child(parent, cursor)?;
        let Some(child) = step else { return Ok(None) };
        if self.handed.get() == MAXIMUM_CHILDREN {
            return Err(format!(
                "the clone container `{parent}` holds more children than the {MAXIMUM_CHILDREN} one walk of it takes, so the clones past that bound were not read and nothing proves this plan's clone gone"
            ));
        }
        self.handed.set(self.handed.get() + 1);
        // What:     `Ok(Some(child))` wraps the next child in the two variants
        //           that mean "a child, and no error". It is the tail
        //           expression, so it is the return.
        // Why:      The proof asks again until this answer is `Ok(None)`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return child;
        // ```
        Ok(Some(child))
    }
}

// The route's open device answers the sweep's two reads,
// so the add's walk asks the pool through the handle it already opened
// behind the start-time gate.
impl DatasetReader for OpenedDevice {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        return self.device.objset_stats(dataset).map_err(|error| return ReadFailure { said: error.to_string(), errno: error.errno() });
    }

    fn next_child(&self, parent: &str, cursor: u64) -> Result<Option<ListedChild>, String> {
        let listed: Option<ListedObject> = self.device.dataset_list_next(parent, cursor).map_err(|error| return error.to_string())?;
        // What:     `Ok(listed.map(..))` turns the binding's listed object into
        //           this walk's `ListedChild`, and wraps the option in the
        //           success variant of the read's `Result`.
        // Why:      The proof reads one step at a time and needs both the name
        //           and the cursor the kernel answered with.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return listed === undefined ? undefined : { name: listed.name, cursor: listed.cursor };
        // ```
        return Ok(listed.map(|object| return ListedChild { name: object.name, cursor: object.cursor }));
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_plan_clone_presence_tests.rs"]
mod tests;
