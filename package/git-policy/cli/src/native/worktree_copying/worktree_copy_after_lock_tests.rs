//! What happens once the ignored state of a new worktree is in place:
//! the trash a snapshot left is emptied in the foreground when no commit service can take it,
//! its notice reaches the stream the command was given,
//! and the emptying is a wait of the process's runtime only when something runs beside it.

use super::*;
use crate::fs_beneath::{ObjectId, identity_at, identity_of, open_directory_at, open_directory_beneath, open_root, plain_components};
use crate::test_support::{fixture, git_context, remove};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_trash::{PendingTrash, TRASH_PARENT, TrashManifest, entry_name};
use std::path::Path;

/// A worktree whose trash holds `count` entries,
/// the administrative directory its `.git` file names,
/// and the manifest a leave-out would have written for it.
fn worktree_with_planted_trash(name: &str, trash_name: &str, count: usize) -> (PathBuf, PathBuf, PathBuf, TrashManifest) {
    let root: PathBuf = fixture(name);
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
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
    return (root, worktree, git_dir, manifest);
}

/// The trash of a snapshot waiting for its disposal,
/// in the worktree that holds it.
fn pending_of(worktree: &Path, git_dir: &Path, manifest: TrashManifest) -> PendingTrash {
    return PendingTrash {
        destination: worktree.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        manifest,
        quiet: false,
    };
}

/// The invocation of a creating command,
/// which decides only what the repair may print and which Git the disposal names.
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
    };
}

/// Where the last disposal ran,
/// or nothing when no disposal ran since the control started.
fn disposed_on() -> Option<crate::worktree_copy_after_lock_disposal::WhereTheDisposalRan> {
    return DISPOSED_ON.lock().expect("the control's own record").take();
}

/// The one line the foreground disposal writes,
/// as the strand of it every reader can look for.
const FOREGROUND_DISPOSAL_LINE: &str = "emptying the directories left out of the new worktree";

/// A trash no commit service can take over is emptied by the command itself,
/// and the line that says so reaches the stream the command was given,
/// as the rest of its report does.
///
/// No route to the service can be chosen for a fixture's administrative directory,
/// so the disposal always takes the foreground path here;
/// the emptied trash and the withdrawn work item are the proof that it did.
#[cfg(target_os = "linux")]
#[test]
fn a_foreground_disposal_notice_reaches_the_stream_the_command_was_given() {
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("after-lock-notice", "6".repeat(40).as_str(), 2);
    let pending: PendingTrash = pending_of(worktree.as_path(), git_dir.as_path(), manifest);
    let after: AfterLock = AfterLock { source_root: None, created: Vec::new(), trash: Some(pending) };
    let invocation: Invocation = creating_invocation();
    let mut stderr: Vec<u8> = Vec::new();
    finish_trash_and_repair(&invocation, &after, &mut stderr);
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert!(disposed_on().is_some(), "the disposal ran");
    assert!(!worktree.join(TRASH_PARENT).exists(), "the trash is emptied in the foreground, so the line must precede a wait the person had");
    assert!(said.contains(FOREGROUND_DISPOSAL_LINE), "the notice the person must read is in the stream the command was given, not beside it: {said:?}");
    assert!(said.contains(worktree.display().to_string().as_str()), "the line names the worktree: {said:?}");
    remove(root.as_path());
}

/// A disposal with nothing running beside it needs no thread:
/// the trash is emptied on the thread that owns the command's stream.
///
/// Nothing to repair is the case the command already had an answer for:
/// a bare repository copies nothing,
/// a copy that failed leaves no whole worktree to repair,
/// and `--no-checkout` has nothing to check out.
#[cfg(target_os = "linux")]
#[test]
fn a_disposal_with_nothing_beside_it_runs_on_the_calling_thread() {
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("after-lock-alone", "7".repeat(40).as_str(), 1);
    let pending: PendingTrash = pending_of(worktree.as_path(), git_dir.as_path(), manifest);
    let after: AfterLock = AfterLock { source_root: None, created: Vec::new(), trash: Some(pending) };
    let invocation: Invocation = creating_invocation();
    let mut stderr: Vec<u8> = Vec::new();
    finish_trash_and_repair(&invocation, &after, &mut stderr);
    let ran: crate::worktree_copy_after_lock_disposal::WhereTheDisposalRan = disposed_on().expect("the disposal ran");
    assert_eq!(ran.thread, std::thread::current().id(), "a wait with nothing to run beside it is a thread of its own for nothing");
    assert!(!ran.inside_the_runtime, "a disposal with nothing beside it enters no runtime either");
    assert!(!worktree.join(TRASH_PARENT).exists(), "and the trash is emptied all the same");
    remove(root.as_path());
}

/// The control for the test above:
/// a disposal beside a real repair is a wait of the process's runtime,
/// so where the disposal ran tells the two cases apart.
#[cfg(target_os = "linux")]
#[test]
fn a_disposal_beside_a_repair_runs_inside_the_processs_runtime() {
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("after-lock-beside", "8".repeat(40).as_str(), 1);
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(&source).expect("source");
    let pending: PendingTrash = pending_of(worktree.as_path(), git_dir.as_path(), manifest);
    let after: AfterLock = AfterLock { source_root: Some(source), created: vec![worktree.clone()], trash: Some(pending) };
    let invocation: Invocation = creating_invocation();
    let mut stderr: Vec<u8> = Vec::new();
    finish_trash_and_repair(&invocation, &after, &mut stderr);
    let ran: crate::worktree_copy_after_lock_disposal::WhereTheDisposalRan = disposed_on().expect("the disposal ran");
    assert!(ran.inside_the_runtime, "the repair of the created worktree and the emptying are two waits of one task of the process's runtime");
    assert!(!worktree.join(TRASH_PARENT).exists(), "the trash is emptied");
    assert!(String::from_utf8(stderr).expect("utf8").contains(FOREGROUND_DISPOSAL_LINE), "the notice reaches the command's stream from the runtime too");
    remove(root.as_path());
}
