//! What:
//!  The add's wait for the clone's mount, verified at the slot in the caller's
//!  own namespace.
//! Why:
//!  A clone that exists is not a mounted clone, and only the caller's view says
//!  whether the slot holds the clone's root
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Add and Git registration", step 4).
//!  The handler attaches through the durable intent and files a receipt, but the
//!  add reads the mount table through the slot itself: the wait must end on what
//!  the person's own namespace shows, and the receipt is the handler's account
//!  of the same fact.
//!  The deadline is the protocol's provisional request deadline, which step 22
//!  sizes; a deadline is not cancellation, so the report says the request stays.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = waitForTheMount(slot, clone, PROVISIONAL_MOUNT_REQUEST_DEADLINE);
//! ```

use crate::worktree_clone_mount::attached_mount_identity::{MountIdentity, mount_identity_of};
use std::path::Path;
use std::time::{Duration, Instant};

/// The wait's poll interval,
/// the protocol's own between two receipt reads.
pub const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The bound on one add's wait,
/// the protocol's provisional mount-request deadline until step 22 sizes it.
pub const MOUNT_WAIT: Duration = crate::worktree_clone_mount_protocol::mount_request_wait::PROVISIONAL_MOUNT_REQUEST_DEADLINE;

/// What:
///  The slot's own mount, once it shows the clone's root.
/// Why:
///  The mount's source names the dataset,
///  its mount point names the slot,
///  and its identity is what the verification compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountedHere = { mount: MountIdentity; waited: Duration };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountedHere {
    /// The mount the slot holds.
    pub mount: MountIdentity,
    /// How long the wait took.
    pub waited: Duration,
}

/// What:
///  Wait until the slot shows the clone's own mount, or report what it showed.
/// Why:
///  The one fact that ends the wait is the caller's own mount view: a source
///  that is the clone's dataset and a mount point that is the slot.
///
/// # Errors
/// The deadline passed,
/// or the slot cannot be opened.
pub fn wait_for_the_mount(slot: &Path, clone: &str) -> Result<MountedHere, String> {
    let started: Instant = Instant::now();
    let mut last: String = String::from("the slot could not be examined");
    while started.elapsed() < MOUNT_WAIT {
        match one_look(slot, clone) {
            Ok(Some(mount)) => return Ok(MountedHere { mount, waited: started.elapsed() }),
            Ok(None) => last = format!("the slot shows no mount of {clone}"),
            Err(reason) => last = reason,
        }
        std::thread::sleep(POLL_INTERVAL.min(MOUNT_WAIT.saturating_sub(started.elapsed())));
    }
    return Err(format!(
        "the clone {clone} was not mounted at {} within {} milliseconds, so the fill cannot see the clone in this command's namespace; {last}. The durable mount request stays on the clone and a later sweep may still act on it. Ways forward: run `monochromatic-git-policy-cli doctor` and read the handler's journal; do not retry the add until the clone's mount state has been inspected",
        slot.display(),
        MOUNT_WAIT.as_millis()
    ));
}

/// One look at the slot's mount.
fn one_look(slot: &Path, clone: &str) -> Result<Option<MountIdentity>, String> {
    let folder: std::fs::File = match std::fs::File::open(slot) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("the slot {} cannot be opened ({error})", slot.display())),
    };
    let mount: MountIdentity = mount_identity_of(&folder).map_err(|refusal| return format!("the slot's mount cannot be read ({})", refusal.message()))?;
    if mount.source != clone || mount.mount_point != slot || mount.file_system_type != "zfs" {
        return Ok(None);
    }
    return Ok(Some(mount));
}

/// What:
///  The slot's mount as a resume finds it, without waiting.
/// Why:
///  A resumed add reads whether its clone is mounted before it decides where it
///  is, and a slot that shows the clone is the state the phase records.
///
/// # Errors
/// The slot or its mount cannot be read.
pub fn the_mount_at(slot: &Path, clone: &str) -> Result<Option<MountIdentity>, String> {
    return one_look(slot, clone);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_mount_tests.rs"]
mod tests;
