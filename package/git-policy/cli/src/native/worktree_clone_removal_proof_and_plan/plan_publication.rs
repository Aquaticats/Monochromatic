//! What:
//!  The plan folder's creation,
//!  and the one write a removal makes into it:
//!  the record staged beside its name and published whole,
//!  after this build's own reader has read it back.
//! Why:
//!  The plan lives in `<common Git directory>/monochromatic-git-policy-cli/clone-destruction-plans/`,
//!  is named by the clone's GUID,
//!  and is written by the removal and by nothing else:
//!  `doctor` and the finish pass never create that folder
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The folder,
//! its owner and its modes").
//!  The project's flush rule leaves a plan write unflushed because every one of them precedes an
//!  irreversible ZFS step,
//!  whose loss costs one repeated proof and never an object nothing can prove
//!  (its "Choice 5"),
//!  so the write is the repository's unsynced replacing write and the staging file it makes is the
//!  one its own writer removes.
//!  A writer that published bytes its reader would refuse would leave a removal with no record,
//!  so the bytes are read back here before the rename.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = PlanFolder.openOrCreate(commonDir);
//! const path = folder.publish(plan);
//! ```

use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced};
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::plan_name::is_canonical_plan_name;
use crate::worktree_clone_removal_proof_and_plan::plan_parse::parse_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::{check_the_folder, split_the_header};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{PlanRefusal, PlanRefusalCause};
use std::path::{Path, PathBuf};

/// What:
///  The folder one repository's destruction plans live in,
///  open for the one write a removal makes.
/// Why:
///  The folder outlives the clone,
///  its registration and its dataset,
///  and a plan inside the clone would be unreachable exactly when it is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlanFolder { readonly path: string; static openOrCreate(commonDir): PlanFolder; publish(plan): string }
/// ```
pub struct PlanFolder {
    path: PathBuf,
}

impl std::fmt::Debug for PlanFolder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.debug_struct("PlanFolder").field("path", &self.path).finish();
    }
}

impl PlanFolder {
    /// What:
    ///  The plan folder,
    ///  made when it is not there yet and checked when it is.
    /// Why:
    ///  A repository that has no plan folder is the healthy state,
    ///  and the removal that writes the first plan makes it once.
    ///
    /// # Throws
    /// The folder cannot be made,
    /// or it is not the common directory's owner's `0700` folder.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const folder = PlanFolder.openOrCreate(commonDir);
    /// ```
    pub fn open_or_create(common_dir: &Path) -> Result<PlanFolder, PlanRefusal> {
        let path: PathBuf = plans_folder(common_dir);
        create_private_directory_all(path.as_path()).map_err(|error| return PlanRefusal::at(PlanRefusalCause::Unreadable, "folder", error.to_string()))?;
        let owner: u32 = common_directory_owner(common_dir)?;
        check_the_folder(path.as_path(), owner)?;
        return Ok(PlanFolder { path });
    }

    /// The folder's path.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Publish one plan under the clone's GUID,
    ///  after this build's reader has read the bytes back.
    /// Why:
    ///  A plan's name is its key,
    ///  so a record whose own `cloneGuid` disagreed with its name would be one no reader reads,
    ///  and a removal would have deleted snapshots with no plan to say so.
    ///
    /// # Throws
    /// A field past its bound,
    /// bytes this build's own reader refuses,
    /// or a write the filesystem refused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const path = folder.publish(plan);
    /// ```
    pub fn publish(&self, plan: &DestructionPlan) -> Result<PathBuf, PlanRefusal> {
        let name: String = plan.facts.clone_guid.file_name();
        if !is_canonical_plan_name(name.as_str()) {
            return Err(PlanRefusal::at(PlanRefusalCause::ForeignName, "cloneGuid", name));
        }
        let bytes: Vec<u8> = plan.encode()?;
        let (header, attempts) = split_the_header(bytes.as_slice())?;
        match parse_plan(header, attempts, Some(name.as_str())) {
            Ok(read_back) if read_back == *plan => {}
            Ok(_) => return Err(PlanRefusal::of(PlanRefusalCause::WriterWouldNotReadItBack, name)),
            Err(refusal) => return Err(refusal),
        }
        let path: PathBuf = self.path.join(name.as_str());
        replace_private_file_unsynced(path.as_path(), bytes.as_slice()).map_err(|error| return PlanRefusal::at(PlanRefusalCause::Unreadable, "file", error.to_string()))?;
        return Ok(path);
    }
}

/// The account that owns the common Git directory.
fn common_directory_owner(common_dir: &Path) -> Result<u32, PlanRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(common_dir).map_err(|error| return PlanRefusal::at(PlanRefusalCause::Unreadable, "common directory", error.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return Ok(metadata.uid());
    }
    #[cfg(not(unix))]
    {
        drop(metadata);
        return Ok(0);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_publication_tests.rs"]
mod tests;
