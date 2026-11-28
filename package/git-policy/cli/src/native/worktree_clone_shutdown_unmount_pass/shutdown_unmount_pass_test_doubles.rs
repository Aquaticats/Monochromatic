//! What:
//!  The doubles the shutdown pass's value tests share:
//!  a mount table a control states,
//!  a walk that hands back a fixture folder and a stated mount number,
//!  a staging route whose unmount answers what the control states,
//!  a fresh lookup,
//!  the route's recorded folder identity,
//!  one clone's GUID,
//!  and a holder scan the control built.
//! Why:
//!  The pass decides everything it decides on these answers,
//!  so a control proves each branch without mounting,
//!  unmounting or reading another account's processes
//!  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "Nothing a test changes may be process-wide":
//!  a control cannot mount,
//!  so the mount calls are a seam).
//!  One copy is shared by the folder's tests rather than one per file,
//!  so a change to a seam breaks one double and not four,
//!  and the values they answer with sit in the file beside this one.
//!  Nothing here is part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const calls = new StatedCalls({ mountId: 41, row: mountRow() });
//! const walk = new StatedWalk({ slot: folder, lookAgain: 41 });
//! ```

use super::shutdown_unmount_pass_test_fixture_values::refusal;
use super::shutdown_unmount_sources::{
    HolderScanning,
    SlotLookups,
    WorktreesFolder,
};
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::FreshSlotLookup;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_unmount::slot_walk_calls::{
    DatasetGuids,
    SlotWalk,
};
use crate::worktree_clone_unmount::staging_call_words::{
    StagingCall,
    StagingCallRefusal,
};
use crate::worktree_clone_unmount::staging_calls::StagingCalls;
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountedDatasetInstance,
    MountedDatasets,
};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// A mount table a control states.
///
/// Why:
///  The pass asks one question of a table,
///  and the scripted table answers it from the rows it holds.
pub(crate) struct StatedMounts {
    /// The rows the table lists.
    pub(crate) rows: Vec<MountedDatasetInstance>,
}

impl MountedDatasets for StatedMounts {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return Ok(self.rows.clone());
    }

    fn mount_holding(
        &self,
        path: &Path,
    ) -> Result<
        Option<crate::zfs_mounted_dataset::mounted_dataset_instance::MountHoldingAPath>,
        String,
    > {
        return Err(format!(
            "the scripted table has no answer for {}",
            path.display()
        ));
    }
}

/// The walk a control states:
/// the slot's folder it hands back,
/// and the mount number a fresh walk of it reaches.
pub(crate) struct StatedWalk {
    /// The slot's folder,
    /// which is a real folder of the test's fixture,
    /// so the descriptor's facts are the kernel's.
    pub(crate) slot: std::fs::File,
    /// The mount number a fresh walk reaches.
    pub(crate) look_again: u64,
}

impl SlotWalk for StatedWalk {
    fn open_the_slot(
        &self,
        _checkout: &Path,
        _slot: &str,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        return self
            .slot
            .try_clone()
            .map_err(|error| return refusal(StagingCall::OpenSlot, 0, error.to_string().as_str()));
    }

    fn open_the_slot_uncrossed(
        &self,
        _checkout: &Path,
        _slot: &str,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        return self.open_the_slot(_checkout, _slot);
    }

    fn look_again(
        &self,
        _checkout: &Path,
        _slot: &str,
    ) -> Result<u64, StagingCallRefusal> {
        return Ok(self.look_again);
    }
}

/// The staging route's calls a control states:
/// the readings it makes and the answer its unmount gives.
pub(crate) struct StatedCalls {
    /// The mount number the slot's descriptor is on.
    pub(crate) mount_id: u64,
    /// The path the descriptor names.
    pub(crate) observed_path: PathBuf,
    /// The rows one mount number at a time.
    pub(crate) rows: BTreeMap<u64, MountFacts>,
    /// What the unmount answers.
    pub(crate) unmount: Result<(), StagingCallRefusal>,
}

impl StagingCalls for StatedCalls {
    fn bind_the_folder(
        &self,
        _folder: &std::fs::File,
        _cache: &std::fs::File,
        _name: &OsStr,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        return Err(unused(StagingCall::Bind));
    }

    fn make_private(
        &self,
        _bind: &std::fs::File,
    ) -> Result<(), StagingCallRefusal> {
        return Err(unused(StagingCall::MakePrivate));
    }

    fn move_clone(
        &self,
        _clone: &std::fs::File,
        _bind: &std::fs::File,
    ) -> Result<(), StagingCallRefusal> {
        return Err(unused(StagingCall::Move));
    }

    fn unmount_top(
        &self,
        _held: &std::fs::File,
    ) -> Result<(), StagingCallRefusal> {
        return match &self.unmount {
            Ok(()) => Ok(()),
            Err(refusal) => Err(refusal.clone()),
        };
    }

    fn name_handle(
        &self,
        _folder: &std::fs::File,
    ) -> Result<(i32, Vec<u8>), StagingCallRefusal> {
        return Err(unused(StagingCall::NameHandle));
    }

    fn open_from_handle(
        &self,
        _cache: &std::fs::File,
        _handle_type: i32,
        _handle: &[u8],
    ) -> Result<std::fs::File, StagingCallRefusal> {
        return Err(unused(StagingCall::OpenHandle));
    }

    fn open_by_name(
        &self,
        _cache: &std::fs::File,
        _name: &OsStr,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        return Err(unused(StagingCall::OpenBind));
    }

    fn mount_id(
        &self,
        _file: &std::fs::File,
    ) -> Result<u64, StagingCallRefusal> {
        return Ok(self.mount_id);
    }

    fn observed_path(
        &self,
        _file: &std::fs::File,
    ) -> Result<PathBuf, StagingCallRefusal> {
        return Ok(self.observed_path.clone());
    }

    fn mount_facts(
        &self,
        mount_id: u64,
    ) -> Result<MountFacts, StagingCallRefusal> {
        return self
            .rows
            .get(&mount_id)
            .cloned()
            .ok_or_else(|| return refusal(StagingCall::MountTable, 0, "no such mount"));
    }

    fn mount_table(&self) -> Result<Vec<MountFacts>, StagingCallRefusal> {
        return Ok(self.rows.values().cloned().collect());
    }

    fn namespace_pair(&self) -> Result<(u64, u64), StagingCallRefusal> {
        return Err(unused(StagingCall::Namespace));
    }

    fn boot(&self) -> Result<String, StagingCallRefusal> {
        return Err(unused(StagingCall::Boot));
    }
}

/// The refusal a call this pass never makes answers,
/// so a control that reached one would read which call it was.
fn unused(call: StagingCall) -> StagingCallRefusal {
    return refusal(call, 0, "the shutdown pass makes no call of this kind");
}

/// The fresh lookup a control states.
pub(crate) struct StatedLookups {
    /// What the walk of the slot answers,
    /// or why it refused.
    pub(crate) answer: Result<Box<FreshSlotLookup>, String>,
}

impl SlotLookups for StatedLookups {
    fn fresh_lookup(
        &self,
        _checkout: &Path,
        _slot: &str,
    ) -> Result<FreshSlotLookup, String> {
        return match &self.answer {
            Ok(fresh) => Ok((**fresh).clone()),
            Err(said) => Err(said.clone()),
        };
    }
}

/// The `.claude/worktrees` identity a control records.
pub(crate) struct StatedWorktrees {
    /// The identity the route's store holds,
    /// or why it could not be read.
    pub(crate) answer: Result<CloneFolderIdentity, String>,
}

impl WorktreesFolder for StatedWorktrees {
    fn recorded(&self) -> Result<CloneFolderIdentity, String> {
        return match &self.answer {
            Ok(identity) => Ok(identity.clone()),
            Err(said) => Err(said.clone()),
        };
    }
}

/// One clone's GUID as a control states it.
pub(crate) struct StatedGuids {
    /// The pool's answer.
    pub(crate) answer: Result<DatasetGuid, String>,
}

impl DatasetGuids for StatedGuids {
    fn live_guid(
        &self,
        _dataset: &str,
    ) -> Result<DatasetGuid, String> {
        return match &self.answer {
            Ok(guid) => Ok(guid.clone()),
            Err(said) => Err(said.clone()),
        };
    }
}

/// A holder scan a control states.
pub(crate) struct StatedHolders {
    /// What the scan answers,
    /// or why no scan could be made.
    pub(crate) answer: Result<Box<HolderScan>, String>,
}

impl HolderScanning for StatedHolders {
    fn holders_of(
        &self,
        _clone: &str,
        _dataset: DatasetGuid,
    ) -> Result<HolderScan, String> {
        return match &self.answer {
            Ok(scan) => Ok((**scan).clone()),
            Err(said) => Err(said.clone()),
        };
    }
}
