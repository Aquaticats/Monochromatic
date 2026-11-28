//! What:
//!  The finish pass as a command runs it:
//!  the plan folder,
//!  the start-time gate,
//!  the pool reader and the Git settle this machine answers with.
//! Why:
//!  The pass is written against four seams,
//!  and every caller that is not a control hands the live ones,
//!  so the one place they are chosen sits beside the pass rather than in the
//!  command that runs it
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration").
//!  A repository with no plan folder is the healthy state and costs one `stat`:
//!  the pool is asked and the device opened only when a plan stands,
//!  so a command in a repository the route never served opens nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = finishTheRemovalPlansLive(commonDir);
//! ```

use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use crate::worktree_clone_removal_finish_pass::pass_over_the_plans::finish_the_removal_plans;
use crate::worktree_clone_removal_finish_pass::the_pass_seams::FinishPass;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::{
    FolderLook,
    TheLivePlansFolder,
    ThePlansFolder,
};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::TheLiveSettle;
use crate::zfs_compatibility_gate::gate_verdict::LiveGate;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Every line the pass prints for one repository,
///  over this machine's own seams.
/// Why:
///  A command calls it before its own work,
///  and a repository with no plan folder costs one `stat` and prints nothing.
///  The device is opened only when a plan stands,
///  and a device that cannot be opened answers one line rather than finishing
///  nothing silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = finishTheRemovalPlansLive(commonDir);
/// ```
pub fn finish_the_removal_plans_live(common_dir: &Path) -> Vec<String> {
    return finish_the_removal_plans_over(common_dir, &TheLiveDatasetReader);
}

/// What:
///  How the pass gets the reader that proves a clone's absence.
/// Why:
///  Opening the reader opens this machine's ZFS device,
///  which no control may do,
///  so the opener is a seam:
///  the release hands the live reader,
///  and a control hands one that records it was asked and opens nothing.
///  The guard in the pass is then shown failing without any run touching the
///  device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheDatasetReaderOpener { open(): Reader }
/// ```
trait TheDatasetReaderOpener {
    /// The reader the absence proof reads the pool through.
    fn open(&self) -> Result<ZfsDatasetReader, String>;
}

/// The release answer:
/// the typed reader over this machine's ZFS device.
struct TheLiveDatasetReader;
impl TheDatasetReaderOpener for TheLiveDatasetReader {
    fn open(&self) -> Result<ZfsDatasetReader, String> {
        return ZfsDatasetReader::open().map_err(|error| return error.to_string());
    }
}

/// What:
///  The pass over the reader opener a caller hands it,
///  which is the release's own shape of the guard that keeps a forwarded
///  command off the device.
/// Why:
///  A repository with no plan folder must cost one `stat` and no open,
///  and only a control that replaces the opener can show that guard missing
///  without opening the machine's device itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = finishTheRemovalPlansOver(commonDir, opener);
/// ```
fn finish_the_removal_plans_over(
    common_dir: &Path,
    reader: &dyn TheDatasetReaderOpener,
) -> Vec<String> {
    let folder: TheLivePlansFolder = TheLivePlansFolder;
    let at: PathBuf = plans_folder(common_dir);
    if let FolderLook::NoFolder = folder.look(at.as_path()) {
        return Vec::new();
    }
    let gate: LiveGate = LiveGate;
    let probe: ZfsDatasetReader = match reader.open() {
        Ok(probe) => probe,
        Err(error) => {
            return vec![format!(
                "the removal plans of this repository could not be finished, because the pool could not be read ({error}); the command that reads such a plan next tries again."
            )];
        }
    };
    return finish_the_removal_plans(&FinishPass {
        common_dir,
        folder: &folder,
        gate: &gate,
        probe: &probe,
        settle: &TheLiveSettle,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_live_pass_tests.rs"]
mod tests;
