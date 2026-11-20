//! Controls for the mount facts the attach records:
//! the enforced flags are `nosuid` and `nodev`,
//! a folder's mount identity reads its mount and its parent,
//! and a mount the table does not list is refused rather than guessed.

use super::*;
use crate::fs_beneath::mount_id_of;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// What:
///  The flags the attach enforces are exactly `nosuid` and `nodev`,
///  with neither bit missing.
/// Why:
///  The measured setuid control gained effective UID 0 without them,
///  so a flag dropped here would silently reopen that control.
#[test]
fn the_attach_enforces_nosuid_and_nodev() {
    assert_eq!(ATTACH_MOUNT_FLAGS, libc::MS_NOSUID | libc::MS_NODEV);
    assert_ne!(ATTACH_MOUNT_FLAGS & libc::MS_NOSUID, 0, "no set-user-ID execution");
    assert_ne!(ATTACH_MOUNT_FLAGS & libc::MS_NODEV, 0, "no device nodes");
}

/// What:
///  A folder's mount identity names the mount its descriptor is on,
///  that mount's parent,
///  its mount point and type,
///  and the folder's device and inode,
///  and it describes every one of them.
/// Why:
///  A later unmount and a rollback report both compare these facts,
///  so each must come from the kernel rather than from a path.
#[test]
fn a_folder_mount_identity_reads_the_kernel_facts() {
    let root: PathBuf = fixture("mount-identity");
    let folder: PathBuf = root.join("folder");
    std::fs::create_dir(&folder).expect("folder");
    let opened: std::fs::File = std::fs::File::open(&folder).expect("open");
    let identity: MountIdentity = mount_identity_of(&opened).expect("the folder is on a mount");
    let named: std::fs::Metadata = opened.metadata().expect("metadata");
    assert_eq!(identity.mount_id, mount_id_of(&opened).expect("the descriptor's mount"));
    assert_eq!((identity.device, identity.inode), (named.dev(), named.ino()));
    let facts: MountFacts = mount_facts_of(identity.mount_id).expect("the table lists this mount");
    assert_eq!(facts.parent_id, identity.parent_mount_id);
    assert_eq!(facts.mount_point, identity.mount_point);
    assert_eq!(facts.source, identity.source);
    assert_eq!(facts.filesystem_type, identity.file_system_type);
    assert_ne!(identity.parent_mount_id, 0, "the mount has a parent");
    assert_ne!(identity.parent_mount_id, identity.mount_id, "a mount is not its own parent");
    let described: String = identity.describe();
    for fact in [identity.mount_id.to_string(), identity.parent_mount_id.to_string(), identity.device.to_string(), identity.inode.to_string(), identity.source.clone(), identity.file_system_type.clone()] {
        assert!(described.contains(fact.as_str()), "{fact} is in {described}");
    }
    assert!(described.contains(identity.observed_path.display().to_string().as_str()), "{}", described);
    remove(root.as_path());
}

/// What:
///  A mount number the kernel's table does not list is refused,
///  naming the number and the table,
///  while a listed one is read.
/// Why:
///  A mount this process cannot find is not one it acts on,
///  and a guessed identity would make a rollback unmount the wrong mount.
#[test]
fn a_mount_the_table_does_not_list_is_refused() {
    let root: PathBuf = fixture("mount-identity-absent");
    let opened: std::fs::File = std::fs::File::open(&root).expect("open");
    let listed: u64 = mount_id_of(&opened).expect("the fixture's mount");
    assert!(mount_facts_of(listed).is_ok(), "positive control");
    let unknown: u64 = listed + 1_000_000;
    let refused: MountIdentityRefusal = mount_facts_of(unknown).expect_err("no mount has that number");
    assert_eq!(refused, MountIdentityRefusal::NoMountEntry { mount_id: unknown });
    let message: String = refused.message();
    assert!(message.contains(unknown.to_string().as_str()), "{message}");
    assert!(message.contains(MOUNTINFO_PATH), "{message}");
    assert!(message.contains("Ways forward"), "{message}");
    remove(root.as_path());
}
