//! What:
//!  The staging name one publish of the readiness record writes under,
//!  and the write of one whole record through a staging sibling:
//!  the text proved readable by the route's own reader,
//!  the bytes written and synced,
//!  the rename over the record's name,
//!  the directory sync,
//!  and the removal of the sibling on every way out.
//! Why:
//!  A caller reads the whole earlier record or the whole new one,
//!  never half of either
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The write is apart from the directory the record lives in,
//!  so the rule that a record is published only when the route's own reader
//!  reads it back sits beside the bytes it writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeTheRecord(directory, readinessStagingName(), record);
//! ```

use super::clone_mount_readiness_file_refusals::ReadinessFileRefusal;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    READINESS_MODE,
    READINESS_NAME,
    ReadinessRecord,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_parse::parse_readiness;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{
    create_at,
    rename_at,
    sync_directory,
    unlink_at,
};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// The sibling a record is staged under before its rename.
pub const READINESS_STAGING_NAME: &str = "result.staging";

/// What:
///  The sibling one publish of the record is staged under before its rename.
/// Why:
///   The name carries this process's id and a counter of this process's own
///  attempts,
///  so two publishes in one process never share a staging name (a shared name
///  lets the second publish remove the first one's live sibling and rename a
///  half-written record into the record's name),
///  and a writer killed before its rename leaves a name no later attempt
///  reuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staging = `result.staging.${process.pid}.${nextAttempt()}`;
/// ```
pub fn readiness_staging_name() -> String {
    let attempt: u64 = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
    return format!("{READINESS_STAGING_NAME}.{}.{attempt}", std::process::id());
}

/// Distinguishes the staging names of two publishes one process makes,
/// so no publish ever shares another's sibling.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What:
///  Write one whole record under the staging name and rename it over the
///  record's name,
///  through the descriptor of the directory already proved.
/// Why:
///  The text is proved readable by the route's own reader before a byte is
///  written,
///  every way out after the staging sibling was made removes it,
///  so this process's next publish is never blocked by its own failure,
///  and a caller reads the whole earlier answer or the whole new one.
///
/// # Errors
/// The text is not one this route's reader accepts,
/// or the staging write,
/// its sync or the rename answered an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeTheRecord(directory, staging, record); // throws ReadinessFileRefusal
/// ```
pub(super) fn write_the_record(
    directory: &std::fs::File,
    staging: &str,
    record: &ReadinessRecord,
) -> Result<(), ReadinessFileRefusal> {
    let text: String = record.text();
    match parse_readiness(text.as_str()) {
        Ok(parsed) if parsed == *record => {}
        Ok(_) => return Err(ReadinessFileRefusal::NotWhatWasWritten),
        Err(refusal) => {
            return Err(ReadinessFileRefusal::NotAReadiness {
                refusal,
            });
        }
    }
    let staged: Result<(), ReadinessFileRefusal> = (|| {
        let mut file: std::fs::File =
            create_at(directory, staging, READINESS_MODE).map_err(|error| {
                return ReadinessFileRefusal::Staging {
                    reason: error.to_string(),
                };
            })?;
        file.set_permissions(std::fs::Permissions::from_mode(READINESS_MODE))
            .map_err(|error| {
                return ReadinessFileRefusal::Staging {
                    reason: error.to_string(),
                };
            })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            return ReadinessFileRefusal::Staging {
                reason: error.to_string(),
            };
        })?;
        file.sync_all().map_err(|error| {
            return ReadinessFileRefusal::Staging {
                reason: error.to_string(),
            };
        })?;
        rename_at(directory, staging, READINESS_NAME).map_err(|error| {
            return ReadinessFileRefusal::Staging {
                reason: error.to_string(),
            };
        })?;
        return sync_directory(directory).map_err(|error| {
            return ReadinessFileRefusal::Staging {
                reason: error.to_string(),
            };
        });
    })();
    if staged.is_err() {
        // A rename may have completed before a failed directory sync, and then there is no sibling to remove.
        let _removed: std::io::Result<()> = unlink_at(directory, staging);
    }
    return staged;
}
