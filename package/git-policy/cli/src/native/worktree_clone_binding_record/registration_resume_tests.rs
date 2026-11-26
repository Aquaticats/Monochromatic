//! Controls for the resume's rows:
//!  each incomplete shape is cleared,
//!  a complete one is adopted whatever its lock,
//!  an incomplete one holding a person's words refuses,
//!  another slot's entry is left,
//!  two attributing entries refuse,
//!  and the two states no entry holds.

use crate::worktree_clone_binding_record::lock_words::lock_file_bytes;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use crate::worktree_clone_binding_record::registration_resume::{
    EntryDecision,
    ListingDecision,
    decide,
    find_registrations,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::{
    Path,
    PathBuf,
};

/// The fixture's nonce.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        .expect("the fixture nonce reads");
}

/// The fixture's slot path and its pointer's target.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/add-cache-layer");
}

/// A registration's facts,
/// empty unless a test fills them.
fn entry(name: &str) -> EntryFacts {
    return EntryFacts {
        name: String::from(name),
        locked: None,
        gitdir: None,
        slot_git_pointer: None,
        commondir: false,
        head: false,
        slot_other_entries: 0,
    };
}

/// The facts of a complete registration at the slot's own name.
fn complete() -> EntryFacts {
    let mut facts: EntryFacts = entry("add-cache-layer");
    facts.locked = Some(lock_file_bytes(&nonce()));
    facts.gitdir = Some(slot_path().join(".git"));
    facts.slot_git_pointer = Some(b"gitdir: /repo/.git/worktrees/add-cache-layer\n".to_vec());
    facts.commondir = true;
    facts.head = true;
    return facts;
}

/// A bare directory,
/// a lock-only entry,
/// a `gitdir`-only entry and its pointer arms are cleared with their rows.
#[test]
fn each_incomplete_shape_is_cleared_with_its_row() {
    let slot: &str = "add-cache-layer";
    let slot_path_buf: PathBuf = slot_path();
    let path: &Path = slot_path_buf.as_path();
    let bare: EntryDecision = decide(&entry(slot), slot, path, &nonce(), &[]);
    assert_eq!(
        bare,
        EntryDecision::Clear {
            row: crate::worktree_clone_binding_record::registration_finder::EntryRow::Bare
        }
    );
    let mut lock_only: EntryFacts = entry(slot);
    lock_only.locked = Some(lock_file_bytes(&nonce()));
    assert_eq!(
        decide(&lock_only, slot, path, &nonce(), &[]),
        EntryDecision::Clear {
            row: crate::worktree_clone_binding_record::registration_finder::EntryRow::LockOnly
        }
    );
    let mut empty_lock: EntryFacts = entry(slot);
    empty_lock.locked = Some(Vec::new());
    assert_eq!(
        decide(&empty_lock, slot, path, &nonce(), &[]),
        EntryDecision::Clear {
            row: crate::worktree_clone_binding_record::registration_finder::EntryRow::LockOnly
        }
    );
    let mut gitdir_only: EntryFacts = entry(slot);
    gitdir_only.gitdir = Some(path.join(".git"));
    assert_eq!(
        decide(&gitdir_only, slot, path, &nonce(), &[]),
        EntryDecision::Clear {
            row: crate::worktree_clone_binding_record::registration_finder::EntryRow::GitdirOnly
        }
    );
    let mut pointer: EntryFacts = gitdir_only.clone();
    pointer.slot_git_pointer = Some(b"gitdir: /repo/.git/worktrees/add-cache-layer\n".to_vec());
    assert_eq!(
        decide(&pointer, slot, path, &nonce(), &[]),
        EntryDecision::Clear {
            row: crate::worktree_clone_binding_record::registration_finder::EntryRow::PointerOnly
        }
    );
    let mut commondir: EntryFacts = pointer.clone();
    commondir.commondir = true;
    assert_eq!(
        decide(&commondir, slot, path, &nonce(), &[]),
        EntryDecision::Clear {
            row: crate::worktree_clone_binding_record::registration_finder::EntryRow::Commondir
        }
    );
}

/// A complete registration is adopted whatever its lock holds,
/// because the lock's own state is the classification's answer and not a row here.
/// Why:
///  The claim holder asks `worktree_clone_managed_lock` which of S1 to S4 stands,
///  and this row must not be a second reading that could disagree with it.
#[test]
fn a_complete_registration_is_adopted_whatever_its_lock() {
    let slot: &str = "add-cache-layer";
    let slot_path_buf: PathBuf = slot_path();
    let path: &Path = slot_path_buf.as_path();
    assert_eq!(
        decide(&complete(), slot, path, &nonce(), &[]),
        EntryDecision::Adopt
    );
    let mut cut: EntryFacts = complete();
    let bytes: Vec<u8> = lock_file_bytes(&nonce());
    cut.locked = Some(
        bytes[..crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_PREFIX.len() + 40]
            .to_vec(),
    );
    assert_eq!(
        decide(&cut, slot, path, &nonce(), &[]),
        EntryDecision::Adopt
    );
    let mut unlocked: EntryFacts = complete();
    unlocked.locked = None;
    assert_eq!(
        decide(&unlocked, slot, path, &nonce(), &[]),
        EntryDecision::Adopt
    );
    let mut foreign: EntryFacts = complete();
    foreign.locked = Some(b"held by a person\n".to_vec());
    assert_eq!(
        decide(&foreign, slot, path, &nonce(), &[]),
        EntryDecision::Adopt
    );
}

/// An incomplete registration holding words this add did not write refuses with its first line,
/// while an empty or too-short lock attributes nothing and is cleared with its row.
#[test]
fn an_incomplete_registration_holding_a_persons_words_refuses() {
    let slot: &str = "add-cache-layer";
    let slot_path_buf: PathBuf = slot_path();
    let path: &Path = slot_path_buf.as_path();
    let mut foreign: EntryFacts = entry(slot);
    foreign.locked = Some(b"held by a person\n".to_vec());
    let refusal: EntryDecision = decide(&foreign, slot, path, &nonce(), &[]);
    assert!(
        matches!(
            refusal,
            EntryDecision::Refuse {
                row:
                    crate::worktree_clone_binding_record::registration_finder::EntryRow::ForeignLock,
                ..
            }
        ),
        "{refusal:?}"
    );
}

/// Another slot's entry and another path's registration are left alone.
#[test]
fn another_slots_entry_and_another_path_are_left() {
    let slot: &str = "topic";
    let path: &Path = Path::new("/repo/.claude/worktrees/topic");
    let mut other: EntryFacts = complete();
    other.name = String::from("topic1");
    other.gitdir = Some(PathBuf::from("/repo/.claude/worktrees/topic1/.git"));
    let left: EntryDecision = decide(&other, slot, path, &nonce(), &[String::from("topic1")]);
    assert_eq!(left, EntryDecision::Leave { row: crate::worktree_clone_binding_record::registration_finder::EntryRow::OwnedByAnotherSlot });
    let another_path: EntryDecision = decide(&other, slot, path, &nonce(), &[]);
    assert_eq!(another_path, EntryDecision::Leave { row: crate::worktree_clone_binding_record::registration_finder::EntryRow::AnotherSlotsRegistration });
}

/// Two entries that attribute themselves refuse,
/// and a listing with none decides the two states no entry holds.
#[test]
fn a_whole_listing_decides_the_states_no_entry_holds() {
    let slot: &str = "add-cache-layer";
    let slot_path_buf: PathBuf = slot_path();
    let path: &Path = slot_path_buf.as_path();
    let first: EntryFacts = complete();
    let mut second: EntryFacts = complete();
    second.name = String::from("add-cache-layer1");
    let both: crate::worktree_clone_binding_record::registration_resume::Finding =
        find_registrations(&[first, second], slot, path, &nonce(), &[], true);
    assert!(
        matches!(both.decision, ListingDecision::Refuse { .. }),
        "{:?}",
        both.decision
    );
    let empty: crate::worktree_clone_binding_record::registration_resume::Finding =
        find_registrations(&[], slot, path, &nonce(), &[], false);
    assert_eq!(empty.decision, ListingDecision::RerunTheGitAdd);
    let pointer: crate::worktree_clone_binding_record::registration_resume::Finding =
        find_registrations(&[], slot, path, &nonce(), &[], true);
    assert!(
        matches!(pointer.decision, ListingDecision::Refuse { .. }),
        "{:?}",
        pointer.decision
    );
}
