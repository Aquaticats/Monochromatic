//! Controls for what the slot's pointer says against the recorded bytes,
//! and for which copy of it the slot shows.

use super::*;
use crate::test_support::{
    fixture,
    remove,
};
use crate::worktree_clone_managed_lock::reading_a_bounded_file::ARefusedReading;
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use std::path::PathBuf;

/// What:
///  The recorded bytes and the bytes at the slot are compared byte for byte.
/// Why:
///  The record holds the pointer Git was given,
///  so a different pointer names a different registration.
#[test]
fn the_bytes_decide_the_state() {
    let recorded: &[u8] = b"gitdir: /repo/.git/worktrees/one\n";
    assert_eq!(
        pointer_state_of(Some(recorded), &Reading::Bytes(recorded.to_vec())),
        PointerState::Intact
    );
    assert_eq!(
        pointer_state_of(
            Some(recorded),
            &Reading::Bytes(b"gitdir: /repo/.git/worktrees/two\n".to_vec())
        ),
        PointerState::Differs {
            found: 33
        }
    );
    assert_eq!(
        pointer_state_of(Some(recorded), &Reading::Absent),
        PointerState::Missing
    );
    assert_eq!(
        pointer_state_of(Some(recorded), &Reading::Refused(ARefusedReading::ALink)),
        PointerState::Unprovable {
            reason: String::from(
                "a symbolic link stands at the name and the file this route reads is a regular file"
            )
        }
    );
}

/// What:
///  A record with no bytes leaves a pointer that stands its own answer.
/// Why:
///  An absent pointer is a state whether or not the record can be compared,
///  and Git's prune reads only the file.
#[test]
fn a_record_without_bytes_still_answers_absence() {
    assert_eq!(
        pointer_state_of(None, &Reading::Absent),
        PointerState::Missing
    );
    assert_eq!(
        pointer_state_of(None, &Reading::Bytes(b"gitdir: /elsewhere\n".to_vec())),
        PointerState::Present {
            found: 19
        }
    );
    assert_eq!(
        pointer_state_of(None, &Reading::Refused(ARefusedReading::ADirectory)),
        PointerState::Unprovable {
            reason: String::from(
                "a directory stands at the name and the file this route reads is a regular file"
            )
        }
    );
}

/// What:
///  The mount table decides which copy of the pointer the slot shows.
/// Why:
///  A mounted clone's root and the checkout's underlay both hold a `.git`,
///  and a person reading the report must know which one was read.
#[test]
fn the_mount_decides_which_copy_is_read() {
    assert_eq!(
        pointer_site_of(&CloneMount::AtItsSlot),
        PointerSite::AtTheCloneRoot
    );
    assert_eq!(pointer_site_of(&CloneMount::Nowhere), PointerSite::Underlay);
    assert_eq!(
        pointer_site_of(&CloneMount::Elsewhere {
            described: String::from("`tank/clone` at `/mnt`")
        }),
        PointerSite::Underlay
    );
    assert_eq!(
        pointer_site_of(&CloneMount::SomethingElseAtTheSlot {
            described: String::from("`tank/other` stands at the slot")
        }),
        PointerSite::AnotherMount {
            described: String::from("`tank/other` stands at the slot")
        }
    );
    assert_eq!(
        pointer_site_of(&CloneMount::Unreadable {
            reason: String::from("denied")
        }),
        PointerSite::Unprovable {
            reason: String::from("denied")
        }
    );
}

/// What:
///  A pointer read from a real slot folder is compared with the recorded bytes,
///  and a link planted at `.git` is unprovable rather than followed.
/// Why:
///  The release read is the one the prune guard and the report share,
///  and a link is what an attacker or a stray copy leaves.
#[test]
fn the_slot_read_refuses_a_link() {
    let root: PathBuf = fixture("pointer-state");
    let slot: PathBuf = root.join("one");
    std::fs::create_dir(slot.as_path()).expect("slot");
    let recorded: &[u8] = b"gitdir: /repo/.git/worktrees/one\n";
    assert_eq!(
        read_pointer_state_at(slot.as_path(), Some(recorded)),
        PointerState::Missing
    );
    std::fs::write(slot.join(".git"), recorded).expect("pointer");
    assert_eq!(
        read_pointer_state_at(slot.as_path(), Some(recorded)),
        PointerState::Intact
    );
    std::fs::remove_file(slot.join(".git")).expect("pointer removed");
    std::os::unix::fs::symlink(root.join("elsewhere"), slot.join(".git")).expect("planted link");
    assert!(matches!(
        read_pointer_state_at(slot.as_path(), Some(recorded)),
        PointerState::Unprovable { .. }
    ));
    remove(root.as_path());
}
