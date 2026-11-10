//! What:
//!  The binding of one staged copy to the registration it was made for:
//! recorded inside the stage container before the journal exists,
//!  and read back by every later installation,
//! whether this command's or a recovery's.
//! Why:
//!  A worktree's administrative directory is found by following the pointer in its `.git` file,
//! and Git reuses a path and an
//!  administrative name after a worktree is removed.
//! The destination of a copy can therefore be a different registration
//!  by the time the copy installs,
//! and a worktree another command is still checking out is not this copy's to fill.
//! The journal
//!  names a destination path,
//! which proves nothing about the registration behind it,
//! so the binding the creating command
//!  verified is carried beside the staged payload and checked again before every write.
//! A stage
//!  without one is a journal an earlier release wrote;
//! its destination check stands alone as it did.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordStageBinding(container, binding);
//! const binding = await stageBinding(container); // undefined for an older stage
//! await verifyStageBinding(commonDir, binding); // throws a reason naming both registrations
//! ```

use crate::json_record::quote;
use crate::private_storage::sync_directory;
use crate::worktree_add_binding::{Binding, Recorded, binding_still_holds, decode_record};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure_of};
use crate::worktree_copy_journal::write_atomic;
use std::path::{Path, PathBuf};

/// The name of the binding record inside a stage container.
/// Why it sits beside the payload:
///  installation reads `payload/` alone,
/// so a file of the container is never installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAGE_BINDING_NAME = 'registration';
/// ```
pub const STAGE_BINDING_NAME: &str = "registration";

/// What:
///  Records the registration a staged copy is made for,
/// inside the stage container.
/// Why:
///   The record is written before the journal names the stage,
/// so a journal this release writes always has one,
/// and a copy killed
///  between staging and its journal is still bound to the worktree it was made for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordStageBinding(container, binding);
/// ```
pub fn record_stage_binding(container: &Path, binding: &Binding) -> Result<(), WorktreeCopyError> {
    let path: PathBuf = container.join(STAGE_BINDING_NAME);
    write_atomic(path.as_path(), binding.encode().as_bytes(), "stage binding")?;
    return sync_directory(container).map_err(raw_failure_of);
}

/// What:
///  The registration a stage container was made for,
/// or nothing when it records none.
/// Why:
///   A stage an earlier release staged has no record,
/// and its journal is settled as before.
/// A record that does not read as a registration is reported instead of guessed at.
///  The record is read as a plain file,
///  opened without waiting and refused when the path is not one:
///  a named pipe at the name would otherwise hold a recovery for a writer that may never come
///  (`SPEC.md`,
///  "Every wait has a bound";
///  a cross-vendor review's finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binding = await stageBinding(container); // undefined when the file is absent
/// ```
pub fn stage_binding(container: &Path) -> Result<Option<Binding>, WorktreeCopyError> {
    let path: PathBuf = container.join(STAGE_BINDING_NAME);
    // `match` treats an absent record as a stage of an earlier release, and every other failure as a fault.
    let text: String = match crate::plain_file::read_plain_text(path.as_path()) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read the registration recorded for the private stage {}.",
                quote(container.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    return match decode_record(text.as_str()) {
        Some(Recorded::Registered(binding)) => Ok(Some(binding)),
        _ => Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: the registration recorded for the private stage {} is unreadable; nothing was changed and the stage stays.",
                quote(container.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        )),
    };
}

/// What:
///  Checks that the registration a stage was made for is still the one at its destination.
/// Why:
///   Installation writes into the destination directory,
/// so a registration that was removed,
/// moved or replaced must stop it
///  before the first entry,
/// and the reason names both registrations so the person can tell what happened.
/// The stage is already validated as a private directory by the caller,
/// and this is only ever asked about a binding read from one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await verifyStageBinding({ commonDir, binding }); // throws a reason
/// ```
pub fn verify_stage_binding(common_dir: &Path, binding: &Binding) -> Result<(), String> {
    return binding_still_holds(common_dir, binding);
}

/// Stage binding controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_binding_tests.rs"]
mod tests;
