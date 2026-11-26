//! What:
//!  What every one of the add's steps is given and what each answers with:
//!  the fixed facts one add's steps share,
//!  the refusal a step stops with,
//!  and the one write that persists a phase of the record.
//! Why:
//!  `add_route_steps.rs` runs the steps and would go over the file-size limit
//!  carrying these too,
//!  and every step file takes the same facts and answers with the same refusal,
//!  so these stand below the steps rather than inside one of them.
//!  Nothing here reads the pool or a folder:
//!  the facts are the ones the runner proved,
//!  and the write is the record's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Steps = { route; names; source; plan; context; home; device; arguments; realGit; overlay; quiet; stops; out };
//! ```

use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_add_route::add_route_plan::{
    AddNames,
    Stops,
};
use crate::worktree_clone_add_route::add_zfs::OpenedDevice;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::record_publication::write_binding;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The fixed facts one add's steps share.
/// Why:
///  Every step names the same slot, records and device, so one value carries
///  them and no step recomputes one.
pub struct Steps<'a> {
    /// The gate's facts.
    pub route: &'a crate::worktree_clone_add_route::add_route_gate::RouteFacts,
    /// The names chosen before the first write.
    pub names: &'a AddNames,
    /// The proven clean source.
    pub source: &'a SnapshotSource,
    /// The add's plan.
    pub plan: &'a AddPlan,
    /// The repository's Git context.
    pub context: &'a GitContext,
    /// The wrapper's record home, open.
    pub home: &'a std::fs::File,
    /// The record home's path.
    pub home_path: &'a Path,
    /// The pool's device, opened behind the gate.
    pub device: &'a OpenedDevice,
    /// The checkout dataset's GUID.
    pub checkout_guid: DatasetGuid,
    /// The pool's GUID.
    pub pool_guid: DatasetGuid,
    /// The person's arguments.
    pub arguments: &'a [OsString],
    /// The real Git.
    pub real_git: &'a Path,
    /// The variables every Git child receives.
    pub overlay: &'a [(OsString, OsString)],
    /// Whether the person asked for no lines.
    pub quiet: bool,
    /// The test-only switch that ends a control run at one step.
    pub stops: &'a Stops,
    /// The lines the add prints for the person.
    pub out: std::cell::RefCell<Vec<String>>,
}

/// What:
///  Why one step stopped without finishing.
/// Why:
///  A step's refusal and Git's own status are different answers:
///  a refusal changes nothing and keeps the records,
///  and a Git failure keeps its status so a script reads the same one.
#[derive(Debug)]
pub struct StepRefusal {
    /// The words the person reads.
    pub message: String,
    /// The exit status this add ends with.
    pub code: i32,
}

impl StepRefusal {
    /// A refusal of the add's own.
    pub fn refused(message: String) -> StepRefusal {
        return StepRefusal {
            message,
            code: REFUSED_EXIT_CODE,
        };
    }

    /// A Git run that ended unsuccessfully,
    /// keeping Git's own status.
    pub fn git(code: i32) -> StepRefusal {
        return StepRefusal {
            message: String::new(),
            code,
        };
    }
}

/// What:
///  Persist one phase of the record and flush it.
/// Why:
///  Every write of this record is flushed, because a later command must not have
///  to rediscover a fact the pool already committed.
pub fn write_the_record(
    steps: &Steps<'_>,
    record: &BindingRecord,
) -> Result<(), StepRefusal> {
    write_binding(steps.home, &steps.route.slot, record).map_err(|error| return StepRefusal::refused(format!("the add's record cannot be written ({error}); the claim, the slot and the registration stand, and running the add again resumes them")))?;
    return Ok(());
}
