//! What:
//!  Controls for the owner read:
//!  this process's own namespace is owned by this process's own user namespace,
//!  a namespace path that is not there is an open refusal,
//!  and the locked test is the two inodes' difference.
//! Why:
//!  The read decides whether a holder's copy is refused before any signal,
//!  and a control that cannot show it naming our own namespace as unlocked
//!  cannot show it either.
//!  These controls mount nothing and open no device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('our own mount namespace is owned by our own user namespace', () => { expect(readTheOwner('/proc/self/ns/mnt').locked()).toBe(false); });
//! ```

use super::{
    NamespaceOwner, OUR_USER_NAMESPACE, identity_of, open_a_namespace, our_user_namespace, owner_of, read_the_owner,
};
use std::path::Path;

/// This process's own mount namespace.
const OUR_MOUNT_NAMESPACE: &str = "/proc/self/ns/mnt";

#[test]
fn our_own_mount_namespace_is_owned_by_our_own_user_namespace() {
    let read: NamespaceOwner = read_the_owner(Path::new(OUR_MOUNT_NAMESPACE)).expect("this process's own namespace is readable");
    assert!(!read.locked(), "a process's own mount namespace is owned by its own user namespace, and the read answered {} against {}", read.owner, read.ours);
    assert_eq!(read.ours, our_user_namespace().expect("this process's own user namespace is readable"));
    assert!(read.namespace.starts_with("mnt:["), "the identity is the kernel's own `mnt:[…]` spelling, and it answered {}", read.namespace);
}

#[test]
fn the_identity_is_the_namespace_object_s_own_inode() {
    let namespace: std::fs::File = open_a_namespace(Path::new(OUR_MOUNT_NAMESPACE)).expect("this process's own namespace is opened");
    let named: String = identity_of(&namespace).expect("a descriptor of this process's own namespace is described");
    assert_eq!(named, read_the_owner(Path::new(OUR_MOUNT_NAMESPACE)).expect("the same namespace is read again").namespace);
    assert_eq!(OUR_USER_NAMESPACE, "/proc/self/ns/user");
}

#[test]
fn a_namespace_path_that_is_not_there_is_an_open_refusal() {
    let refused = open_a_namespace(Path::new("/proc/self/ns/a-kind-that-does-not-exist")).expect_err("no such namespace link exists");
    assert_eq!(refused.calling, "open");
    assert_eq!(refused.errno, libc::ENOENT);
    let sentence: String = refused.to_string();
    assert!(sentence.contains("errno"), "a refusal names its errno, and it read {sentence}");
}

#[test]
fn the_locked_test_is_the_two_inodes_difference() {
    let ours: u64 = our_user_namespace().expect("this process's own user namespace is readable");
    let unlocked: NamespaceOwner = NamespaceOwner { namespace: String::from("mnt:[1]"), owner: ours, ours };
    assert!(!unlocked.locked());
    let locked: NamespaceOwner = NamespaceOwner { namespace: String::from("mnt:[2]"), owner: ours.wrapping_add(1), ours };
    assert!(locked.locked());
}

#[test]
fn a_descriptor_that_is_not_a_namespace_is_an_ioctl_refusal() {
    let plain: std::fs::File = std::fs::File::open("/proc/self/stat").expect("a plain file of this process's own is readable");
    let refused = owner_of(&plain).expect_err("a plain file is not a namespace, and the ioctl refuses it");
    assert_eq!(refused.calling, "ioctl NS_GET_USERNS");
    assert_ne!(refused.errno, 0);
}
