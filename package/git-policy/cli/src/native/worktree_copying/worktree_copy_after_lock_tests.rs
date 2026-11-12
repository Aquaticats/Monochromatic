//! What happens once the ignored state of a new worktree is in place:
//! the trash a snapshot left is handed to the commit service and left standing,
//! its record outlives the
//!  command,
//! and no command waits for the emptying.

use super::*;
use crate::fs_beneath::{ObjectId, identity_at, identity_of, open_directory_at, open_directory_beneath, open_root, plain_components};
use crate::test_support::{fixture, git_context, remove};
use crate::work_item::{PENDING, WorkItem, WorkKind, state_directory};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_trash::{PendingTrash, TRASH_PARENT, TRASH_REFERENCE, TrashManifest, entry_name};
use std::path::Path;

/// A worktree whose trash holds `count` entries,
/// the administrative directory its `.git` file names,
/// and the manifest a leave-out would have written for it.
fn worktree_with_planted_trash(name: &str, trash_name: &str, count: usize) -> (PathBuf, PathBuf, PathBuf, TrashManifest) {
    let root: PathBuf = fixture(name);
    let worktree: PathBuf = root.join("topic");
    let common: PathBuf = root.join("common");
    let git_dir: PathBuf = common.join("worktrees").join("topic");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    for index in 0..count {
        let entry: PathBuf = worktree.join(TRASH_PARENT).join(trash_name).join(entry_name(index));
        std::fs::create_dir_all(&entry).expect("trash entry");
        std::fs::write(entry.join("file"), b"left out\n").expect("entry file");
    }
    let held: std::fs::File = open_root(worktree.as_path()).expect("the worktree");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("plain parent");
    let parent: std::fs::File = open_directory_beneath(&held, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = open_directory_at(&parent, std::ffi::OsStr::new(trash_name)).expect("the trash");
    let entries: Vec<ObjectId> = (0..count).map(|index: usize| { return identity_at(&trash, std::ffi::OsStr::new(entry_name(index).as_str())).expect("entry identity"); }).collect();
    let manifest: TrashManifest = TrashManifest { name: String::from(trash_name), trash: identity_of(&trash).expect("trash identity"), entries };
    return (root, worktree, common, manifest);
}

/// The trash of a snapshot waiting to be handed over,
/// in the worktree that holds it.
fn pending_of(worktree: &Path, common: &Path, manifest: TrashManifest) -> PendingTrash {
    return PendingTrash { destination: worktree.to_path_buf(), common_dir: common.to_path_buf(), manifest, taken_over: 0, quiet: false };
}

/// The invocation of a creating command,
/// which decides only what the repair may print.
fn creating_invocation() -> Invocation {
    return Invocation {
        identity: WorktreeIdentity::OutsideWorktree,
        effective_cwd: PathBuf::from("/"),
        creates: true,
        destination: None,
        opted_out: false,
        short_circuit: false,
        context: git_context(),
        add_plan: None,
        repair: crate::package_repair::RepairSwitch::default(),
        quiet: false,
    };
}

/// What:
///  A trash that no commit service can be told about is named by a record under the repository's common
///  directory,
/// and it is left in the worktree,
/// so the service empties it when it next runs.
/// Why:
///   The add no longer empties anything,
/// so the record is the only thing that says the directories exist;
/// it goes
///  into the common work directory because that is the one the repository's service drains,
/// and a trash left
///  without one would be disk nobody knows about.
/// A fixture's administrative directory reaches no service from a session with no
///  remembered executable,
/// so this is the path every add takes on a machine whose units are not installed.
#[cfg(target_os = "linux")]
#[test]
fn a_trash_no_service_can_take_is_named_by_a_record_and_left_standing() {
    let name: String = "6".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("after-lock-record", name.as_str(), 2);
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let after: AfterLock = AfterLock { source_root: None, created: Vec::new(), trash: Some(pending) };
    let invocation: Invocation = creating_invocation();
    let mut stderr: Vec<u8> = Vec::new();
    finish_trash_and_repair(&invocation, &after, &mut stderr);
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d1/file").is_file(), "the command returns without emptying anything");
    let record: PathBuf = state_directory(crate::service_work::work_directory(common.as_path()).as_path(), PENDING).join(format!("trash-{name}"));
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record the hand-over wrote");
    let item: WorkItem = WorkItem::decode(text.as_str()).expect("a work item");
    assert_eq!(item.kind, WorkKind::Trash);
    assert_eq!(item.git_dir, common, "the record is the repository's, so one service empties its trashes one at a time");
    assert_eq!(item.repository, worktree);
    assert_eq!(item.oid, name);
    assert_eq!(item.parent, manifest.encode());
    assert_eq!(item.branch_ref, TRASH_REFERENCE);
    assert_eq!(stderr, Vec::<u8>::new(), "nothing is repaired here, and the reason the wake failed is a warning of its own");
    remove(root.as_path());
}

/// A hand-over with nothing to hand over writes nothing and says nothing.
#[cfg(target_os = "linux")]
#[test]
fn an_add_that_set_nothing_aside_hands_nothing_over() {
    let root: PathBuf = fixture("after-lock-nothing");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(&common).expect("the common directory");
    let after: AfterLock = AfterLock { source_root: None, created: Vec::new(), trash: None };
    let invocation: Invocation = creating_invocation();
    let mut stderr: Vec<u8> = Vec::new();
    finish_trash_and_repair(&invocation, &after, &mut stderr);
    assert_eq!(stderr, Vec::<u8>::new());
    assert!(!crate::service_work::work_directory(common.as_path()).exists(), "no work directory is made for nothing");
    remove(root.as_path());
}
