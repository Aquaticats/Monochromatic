//! What:
//!  Where the shutdown pass reads and writes:
//!  this boot's readiness record and the directory it lives in,
//!  the root-owned configuration the boot unit itself read,
//!  the per-slot claims the handler already takes,
//!  the folder the route recorded the identities of,
//!  and the kernel files that name this boot and this process.
//! Why:
//!  A pass that unmounts at shutdown runs where a mode runs,
//!  as root and in the host mount namespace,
//!  and every place it names is the installed one the boot unit's own
//!  `ExecStart` read
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  They travel as one value so a control hands in folders of its own and
//!  never reads or writes `/run`,
//!  `/var/lib`,
//!  `/etc` or this machine's `/proc`
//! (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "Nothing a test changes may be process-wide").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const places = ShutdownUnmountPlaces.live();
//! places.readiness.directory; // '/run/git-worktree-clone-mount-readiness'
//! ```

use crate::worktree_clone_mount::slot_identity_store::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_protocol::ReadinessFile;
use crate::worktree_clone_unmount::staging_calls::BOOT_ID_PATH;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The `/proc` root a live holder scan reads.
/// Why:
///  The holder scan takes the tree it reads as an argument,
///  and the release path reads this machine's own
///  (`crate::clone_holder_scan::holder_scan::HolderScanRequest`).
pub(crate) const PROC_ROOT: &str = "/proc";

/// What:
///  The task whose namespace and root are this process's own.
/// Why:
///  A scan tells its own namespace's mounts from another namespace's,
///  and this process is the one that would not unmount.
pub(crate) const SELF_TASK: &str = "/proc/self";

/// What:
///  Every place the shutdown pass touches,
///  as folders and files rather than as calls.
/// Why:
///   The places are what a control replaces,
///  and the calls are what a control scripts,
///  so the two are separate values and a reader sees which of the two a
///  particular test changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShutdownUnmountPlaces = { readiness; configuration; claims; store; bootIdentity; procRoot; selfTask };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShutdownUnmountPlaces {
    /// This boot's readiness record and the directory it lives in.
    pub(crate) readiness: ReadinessFile,
    /// The root-owned configuration the boot pass read.
    pub(crate) configuration: PathBuf,
    /// The per-slot claims a root handler takes before it acts on a slot.
    pub(crate) claims: SlotClaimDirectory,
    /// The folder the route recorded the `.claude/worktrees` identity in.
    pub(crate) store: PathBuf,
    /// The kernel file that names this boot.
    pub(crate) boot_identity: PathBuf,
    /// The `/proc` root a holder scan reads.
    pub(crate) proc_root: PathBuf,
    /// The task whose namespace and root are this process's own.
    pub(crate) self_task: PathBuf,
}

impl ShutdownUnmountPlaces {
    /// What:
    ///  The installed places:
    ///  what a unit's `ExecStopPost` reaches.
    /// Why:
    ///   The shutdown unit runs the same words the boot pass runs,
    ///  so the record,
    ///  the configuration and the claims are the ones that pass used
    ///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
    ///  "Boot and shutdown unit").
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const places = ShutdownUnmountPlaces.live();
    /// ```
    pub fn live() -> Self {
        return ShutdownUnmountPlaces {
            readiness: ReadinessFile::live(),
            configuration: PathBuf::from(CONFIGURATION_PATH),
            claims: SlotClaimDirectory::live(),
            store: PathBuf::from(SLOT_IDENTITY_STORE_PATH),
            boot_identity: PathBuf::from(BOOT_ID_PATH),
            proc_root: PathBuf::from(PROC_ROOT),
            self_task: PathBuf::from(SELF_TASK),
        };
    }

    /// What:
    ///  This boot's identifier as the kernel writes it,
    ///  without the line's newline.
    /// Why:
    ///   The readiness record names the boot it belongs to,
    /// so the pass has to know which boot it is before it can tell this boot's
    ///  record from an earlier one's
    /// (`crate::worktree_clone_mount_protocol::clone_mount_readiness_admission`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const boot = places.thisBoot(); // '0f3a5c7e-…'
    /// ```
    ///
    /// # Errors
    /// The file cannot be read,
    /// or it holds nothing,
    /// because a boot identifier of no characters matches no record.
    pub(crate) fn this_boot(&self) -> Result<String, String> {
        let text: String = std::fs::read_to_string(self.boot_identity.as_path()).map_err(
            |error| {
                return format!(
                    "this boot's identifier could not be read from {} ({error}), so no readiness record could be told from another boot's and nothing was unmounted. Ways forward: run the mode where the kernel's boot identifier is readable, as the boot unit's `ExecStopPost` does",
                    self.boot_identity.display()
                );
            },
        )?;
        let boot: &str = text.trim();
        if boot.is_empty() {
            return Err(format!(
                "this boot's identifier at {} holds no characters, so no readiness record could be told from another boot's and nothing was unmounted. Ways forward: read the file by hand and run the mode again where the kernel writes an identifier",
                self.boot_identity.display()
            ));
        }
        return Ok(String::from(boot));
    }

    /// What:
    ///  The record's directory,
    ///  which the withdrawal proves before it removes a name.
    /// Why:
    ///   The directory's owner and mode are what make a name under it the
    ///  route's,
    ///  so the pass names it rather than the record's own file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = places.readinessDirectory();
    /// ```
    pub(crate) fn readiness_directory(&self) -> &Path {
        return self.readiness.directory();
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "shutdown_unmount_places_tests.rs"]
mod tests;
