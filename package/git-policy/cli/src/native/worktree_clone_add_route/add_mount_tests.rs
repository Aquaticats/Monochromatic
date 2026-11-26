//! Controls for the mount's read:
//!  a plain fixture holds no clone's mount,
//!  and the wait's name for its bound is the protocol's own.

use super::{
    MOUNT_WAIT,
    the_mount_at,
};
use crate::test_support::{
    fixture,
    remove,
};
use std::path::PathBuf;

#[test]
fn a_plain_folder_shows_no_clone_mount() {
    let root: PathBuf = fixture("mount-none");
    assert_eq!(
        the_mount_at(root.as_path(), "tank/clones/topic")
            .expect("a plain folder reads as no mount"),
        None
    );
    remove(root.as_path());
}

#[test]
fn the_bound_is_the_protocol_s_provisional_deadline() {
    assert_eq!(MOUNT_WAIT, crate::worktree_clone_mount_protocol::mount_request_wait::PROVISIONAL_MOUNT_REQUEST_DEADLINE);
    assert!(
        MOUNT_WAIT.as_millis() > 0,
        "a wait of no time would never see the handler's mount"
    );
}
