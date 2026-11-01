//! What:
//!  The hooks directory of one commit transaction: it holds `plan.json` and nothing else.
//! Why:
//!  `plan.json` is the durable record of the transaction's preparation lease, which a nested wrapper invocation
//!  validates (`transaction_lease.rs`) and which recovery sweeps with the rest of the directory.
//!  The directory holds no hook entry: Git never starts the wrapper for a hook, because it reads no hooks path
//!  that names this directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeHookPlanRecord({ hooksDirectory, plan });
//! ```

/// Plan spelling shared with the lease validator.
use super::hook_dispatch_plan::{HookDispatchPlan, PLAN_FILENAME, encode_hook_dispatch_plan};
/// Private directory creation and exclusive private file writes.
use super::private_storage::{create_private_directory, write_private_file_unsynced};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Why writing the plan record failed, naming the path.
/// Why:
///   The caller stops the transaction; a partial hooks directory is never reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`cannot write ${path}`);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookPlanRecordError(pub String);

/// What:
///  One sentence naming the path that failed.
/// Why:
///   The caller prints it as the diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for HookPlanRecordError {
    /// Writes the stored sentence.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{}", self.0);
    }
}

/// What:
///  Create a new hooks directory holding the plan record.
/// Why:
///   The directory must not exist, so nothing half-written is reused.
///  The write is unsynced: an empty, halved or lost plan completes or discards in the tearing table at every phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(hooksDirectory); await writeFile(join(hooksDirectory, 'plan.json'), `${JSON.stringify(plan)}\n`);
/// ```
pub fn write_hook_plan_record(hooks_directory: &Path, plan: &HookDispatchPlan) -> Result<(), HookPlanRecordError> {
    create_private_directory(hooks_directory).map_err(|error| {
        return HookPlanRecordError(format!(
            "cannot create the hook directory {}: {error}",
            hooks_directory.display()
        ))
    })?;
    let plan_path: PathBuf = hooks_directory.join(PLAN_FILENAME);
    let plan_bytes: String = format!("{}\n", encode_hook_dispatch_plan(plan));
    write_private_file_unsynced(plan_path.as_path(), plan_bytes.as_bytes()).map_err(|error| {
        return HookPlanRecordError(format!(
            "cannot write the hook plan {}: {error}",
            plan_path.display()
        ))
    })?;
    return Ok(());
}

/// The record's directory shape is controlled in the test module.
#[cfg(test)]
#[path = "hook_plan_record_tests.rs"]
mod tests;
