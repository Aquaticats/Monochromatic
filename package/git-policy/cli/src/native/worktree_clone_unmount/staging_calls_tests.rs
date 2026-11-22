//! What:
//!  Controls for the release calls' reads:
//!  this boot,
//!  this process's mount namespace,
//!  and the mount table a folder is on.
//! Why:
//!  A mount call is proved in the guest,
//!  and the reads that decide whether it may run are proved here,
//!  so the guest does not have to show a boot identifier or a namespace pair.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(live.boot()).not.toBe('');
//! ```

use super::{LiveStagingCalls, StagingCalls, MOUNT_NAMESPACE_PATH};
use std::os::unix::fs::MetadataExt;

/// The boot identifier is the kernel's own and is not empty.
#[test]
fn the_boot_is_the_kernels() {
    let boot: String = LiveStagingCalls.boot().expect("this boot is read");
    assert!(!boot.trim().is_empty());
    assert!(boot.len() <= 36);
}

/// The namespace pair is this process's own mount namespace.
#[test]
fn the_namespace_pair_is_this_processs() {
    let (device, inode): (u64, u64) = LiveStagingCalls.namespace_pair().expect("this namespace is read");
    let facts: std::fs::Metadata = std::fs::metadata(MOUNT_NAMESPACE_PATH).expect("the namespace file is read");
    assert_eq!((device, inode), (facts.dev(), facts.ino()));
}

/// A fixture folder is on a mount this process's table lists.
#[test]
fn a_folder_is_on_a_listed_mount() {
    let root: std::path::PathBuf = crate::testing::test_support::fixture_root().join("staging-calls-mount");
    std::fs::create_dir_all(root.as_path()).expect("the fixture folder is made");
    let folder: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture folder opens");
    let mount_id: u64 = LiveStagingCalls.mount_id(&folder).expect("the folder's mount is read");
    let facts = LiveStagingCalls.mount_facts(mount_id).expect("the mount's facts are read");
    assert_eq!(facts.mount_id, mount_id);
}
