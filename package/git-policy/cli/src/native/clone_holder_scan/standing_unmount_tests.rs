//! What:
//!  Controls of the exclusion a standing `unmount` intent makes:
//!  a local `unmount` request and a slot set the managed attach aside,
//!  a `mount` request,
//!  an absent intent or no known slot sets nothing aside,
//!  and a bind of a folder inside the clone is never set aside.
//! Why:
//!  A holder set aside lets a removal go on around it,
//!  so every input that is not quite a standing unmount at the managed slot
//!  keeps the mount a holder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(StandingUnmount.of(unmountIntent, slot)?.setsAside(managedMount)).toBe(true);
//! ```

use super::StandingUnmount;
use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::{Path, PathBuf};

/// The nonce the fixture's requests carry.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The slot the fixture's managed attach stands at.
fn slot() -> PathBuf {
    return PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one");
}

/// One request of the given operation.
fn request(operation: MountOperation) -> MountIntent {
    return MountIntent { operation, nonce: RequestNonce::parse(NONCE).expect("the fixture nonce is one") };
}

/// The managed attach:
/// the dataset's own mount at the slot.
fn managed_attach() -> HeldMount {
    return HeldMount { mount_id: 41, root: PathBuf::from("/"), mount_point: slot() };
}

/// An `unmount` request at a known slot sets the managed attach aside,
/// and a `mount` request does not.
#[test]
fn only_an_unmount_request_sets_the_managed_attach_aside() {
    let unmount: MountIntent = request(MountOperation::Unmount);
    let standing: StandingUnmount = StandingUnmount::of(Some(&unmount), Some(slot().as_path())).expect("an unmount request at a slot is a standing unmount");
    assert_eq!(standing.slot(), slot().as_path());
    assert!(standing.sets_aside(&managed_attach()));
    let mount: MountIntent = request(MountOperation::Mount);
    assert_eq!(StandingUnmount::of(Some(&mount), Some(slot().as_path())), None);
}

/// An absent intent and an unknown slot set nothing aside,
/// so every mount stays a holder.
#[test]
fn nothing_is_set_aside_without_a_request_or_a_slot() {
    let unmount: MountIntent = request(MountOperation::Unmount);
    assert_eq!(StandingUnmount::of(None, Some(slot().as_path())), None);
    assert_eq!(StandingUnmount::of(Some(&unmount), None), None);
    assert_eq!(StandingUnmount::of(Some(&unmount), Some(Path::new("relative/slot"))), None);
}

/// A bind of a folder inside the clone is never the managed attach,
/// and neither is a mount at another path.
#[test]
fn a_bind_inside_the_clone_is_never_set_aside() {
    let unmount: MountIntent = request(MountOperation::Unmount);
    let standing: StandingUnmount = StandingUnmount::of(Some(&unmount), Some(slot().as_path())).expect("the fixture intent stands");
    let bind: HeldMount = HeldMount { mount_id: 42, root: PathBuf::from("/sub"), mount_point: slot() };
    assert!(!standing.sets_aside(&bind));
    let elsewhere: HeldMount = HeldMount { mount_id: 43, root: PathBuf::from("/"), mount_point: PathBuf::from("/srv/holder") };
    assert!(!standing.sets_aside(&elsewhere));
}
