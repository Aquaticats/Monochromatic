//! What:
//!  The values the shutdown pass's value tests state:
//!  the datasets and mount numbers they name,
//!  the refusals a scripted call answers with,
//!  one slot's fresh capture,
//!  the identity of a folder,
//!  one mount's row,
//!  and the holder scans a busy mount is answered with.
//! Why:
//!  A control that built these facts twice would let one copy drift from the
//!  grammar the pass reads,
//!  so the folder's tests share one set beside the doubles that answer them.
//!  Nothing here is part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const row = mountRow(CLONE, slotPath, '/', 'zfs', PARENT_MOUNT_ID);
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::FreshSlotLookup;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_unmount::staging_call_words::{
    StagingCall,
    StagingCallRefusal,
};
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// The clone dataset every control reads,
/// which is one immediate child of the container.
pub(crate) const CLONE: &str = "zpcachyos/nosnap/git-worktree-clones/one";

/// The container the sweep enumerates.
pub(crate) const CONTAINER: &str = "zpcachyos/nosnap/git-worktree-clones";

/// The clone GUID every control knows.
pub(crate) const CLONE_GUID: &str = "4821193048571903102";

/// The mount number the controls state.
pub(crate) const MOUNT_ID: u64 = 41;

/// The mount the slot's folder is on.
pub(crate) const PARENT_MOUNT_ID: u64 = 40;

/// The one refusal a control states as the kernel's busy answer.
pub(crate) fn busy() -> StagingCallRefusal {
    return StagingCallRefusal {
        call: StagingCall::Unmount,
        errno: libc::EBUSY,
        detail: String::from("a process holds the mount"),
    };
}

/// One refusal of one call,
/// as the kernel's own words would carry it.
pub(crate) fn refusal(
    call: StagingCall,
    errno: i32,
    detail: &str,
) -> StagingCallRefusal {
    return StagingCallRefusal {
        call,
        errno,
        detail: String::from(detail),
    };
}

/// The GUID every control knows as the clone's.
pub(crate) fn clone_guid() -> DatasetGuid {
    return DatasetGuid::parse(CLONE_GUID).expect("the fixture GUID reads");
}

/// A fresh lookup of one slot,
/// whose folder is the one the walk hands back.
///
/// Why:
///  The pass compares the descriptor's facts with this capture,
///  so a capture built from the same folder is the agreeing arm.
pub(crate) fn fresh_lookup_of(
    slot: &std::fs::File,
    parent_identity: &CloneFolderIdentity,
) -> FreshSlotLookup {
    let fields: std::fs::Metadata = slot.metadata().expect("the fixture folder is examined");
    let identity: CloneFolderIdentity = CloneFolderIdentity {
        device: fields.dev(),
        inode: fields.ino(),
        owner: fields.uid(),
        mode: fields.mode() & 0o7777,
        access_acl: None,
        default_acl: None,
    };
    return FreshSlotLookup {
        parent_identity: parent_identity.clone(),
        slot_identity: identity,
        mount_id: MOUNT_ID,
        parent_mount_id: PARENT_MOUNT_ID,
        unique_mount_number: Some(MOUNT_ID),
        observed_path: PathBuf::from("/tmp/the-slot-the-walk-reached"),
    };
}

/// The identity of one folder,
/// captured from the folder itself.
///
/// Why:
///  A control compares an identity it captured with one the pass captured,
///  so the agreeing arm is the same folder read twice.
pub(crate) fn identity_of(folder: &Path) -> CloneFolderIdentity {
    let opened: std::fs::File = std::fs::File::open(folder).expect("the fixture folder opens");
    return CloneFolderIdentity::capture(&opened).expect("the fixture folder has an identity");
}

/// One mount's row as the kernel's table would carry it.
///
/// Why:
///  Every field the pass compares lives here,
///  so a control that changes one field states exactly that difference.
pub(crate) fn mount_row(
    source: &str,
    mount_point: &Path,
    root: &Path,
    filesystem_type: &str,
    parent_id: u64,
) -> MountFacts {
    return MountFacts {
        mount_id: MOUNT_ID,
        parent_id,
        root: root.to_path_buf(),
        optional_fields: Vec::new(),
        mount_point: mount_point.to_path_buf(),
        filesystem_type: String::from(filesystem_type),
        source: String::from(source),
        mount_options: vec![String::from("rw")],
        super_options: Vec::new(),
    };
}

/// One holder scan that found one namespace holding the clone.
///
/// Why:
///  The busy arm's line names what the scan found,
///  so a control needs a scan that found something.
pub(crate) fn scan_finding_one_holder() -> HolderScan {
    return HolderScan {
        dataset: clone_guid(),
        source: String::from(CLONE),
        nonce: crate::worktree_clone_mount_protocol::request_nonce::RequestNonce::parse(
            "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f",
        )
        .expect("the fixture nonce reads"),
        own_namespace: None,
        namespaces_scanned: 2,
        own_mounts: vec![HeldMount {
            mount_id: MOUNT_ID,
            root: PathBuf::from("/"),
            mount_point: PathBuf::from("/a/slot"),
        }],
        holders: vec![NamespaceHolder {
            namespace: String::from("mnt:[4026532001]"),
            task: PathBuf::from("/proc/1234"),
            mounts: vec![HeldMount {
                mount_id: MOUNT_ID + 7,
                root: PathBuf::from("/"),
                mount_point: PathBuf::from("/a/holder/slot"),
            }],
        }],
        incomplete: Vec::new(),
        time: String::from("2026-11-27T21:00:00-05:00"),
    };
}

/// A holder scan that read nothing it meant to.
///
/// Why:
///  An incomplete read is a fact a person acts on,
///  and a control proves the count reaches the line.
pub(crate) fn scan_with_one_incomplete_read() -> HolderScan {
    let mut scan: HolderScan = scan_finding_one_holder();
    scan.incomplete.push(
        crate::clone_holder_scan::incomplete_reads::IncompleteRead::Listing {
            proc_root: PathBuf::from("/proc"),
            reason: String::from("the tree could not be listed"),
        },
    );
    return scan;
}
