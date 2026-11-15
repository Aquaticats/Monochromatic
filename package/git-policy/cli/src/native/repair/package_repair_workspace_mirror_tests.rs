//! Controls of the mirror a workspace is resolved and cleaned in:
//! its shape, its own lock, and one folder per job.

use super::{a_name_of_this_job, mirror_the_workspace, remove_the_mirror, the_mirror_root};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  A mirror keeps the workspace's place below the worktree root,
///  its directories on the way are real,
///  every sibling is a link to the real one,
///  and the workspace's manifest and lock are copies.
/// Why:
///   A path dependency is written relative to the manifest that names it,
///  so the mirror has to stand where the workspace stands;
///  and the copies of the two files are what make Cargo resolve and rewrite the mirror's own lock
///  instead of opening the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mirrored = mirrorTheWorkspace(worktree, join(worktree, 'package/own'), join(root, 'job')); expect(statSync(join(mirrored, 'Cargo.lock')).ino).not.toBe(statSync(join(workspace, 'Cargo.lock')).ino)
/// ```
#[test]
fn a_mirror_stands_where_the_workspace_stands_with_copies_of_its_two_files() {
    let root: PathBuf = fixture("workspace-mirror-shape");
    let worktree: PathBuf = root.join("worktree");
    let workspace: PathBuf = worktree.join("package/own");
    std::fs::create_dir_all(workspace.join("src")).expect("the workspace");
    std::fs::write(workspace.join("Cargo.toml"), b"[package]\nname = \"own\"\n").expect("the manifest");
    std::fs::write(workspace.join("Cargo.lock"), b"# the person's lock\n").expect("the lock");
    std::fs::write(workspace.join("src/lib.rs"), b"pub fn own() {}\n").expect("the source");
    std::fs::write(worktree.join("sibling.txt"), b"a sibling of the workspace\n").expect("a sibling");
    let job: String = a_name_of_this_job();
    let mirror_root: PathBuf = the_mirror_root(root.as_path(), workspace.as_path(), job.as_str());
    let mirrored: PathBuf = mirror_the_workspace(worktree.as_path(), workspace.as_path(), mirror_root.as_path()).expect("the mirror is made");
    assert!(mirrored.ends_with(Path::new("package/own")), "the mirror keeps the workspace's place: {}", mirrored.display());
    assert!(mirrored.join("Cargo.toml").is_file() && !std::fs::symlink_metadata(mirrored.join("Cargo.toml")).expect("the manifest").file_type().is_symlink(), "the manifest is a copy");
    assert!(mirrored.join("Cargo.lock").is_file() && !std::fs::symlink_metadata(mirrored.join("Cargo.lock")).expect("the lock").file_type().is_symlink(), "the lock is a copy");
    assert!(std::fs::symlink_metadata(mirrored.join("src")).expect("the source directory").file_type().is_symlink(), "everything else is a link to the real one");
    assert!(std::fs::symlink_metadata(mirrored.join("..").join("sibling.txt").canonicalize().unwrap_or_default()).is_ok() || mirrored.join("../../sibling.txt").exists(), "a sibling of the workspace is reachable from the mirror");
    // The copy is the mirror's own: a write to it is not a write to the person's file.
    std::fs::write(mirrored.join("Cargo.lock"), b"# what the resolution wrote\n").expect("the mirror's lock is written");
    assert_eq!(std::fs::read(workspace.join("Cargo.lock")).expect("the person's lock"), b"# the person's lock\n".to_vec(), "the person's lock is untouched");
    remove_the_mirror(mirror_root.as_path());
    assert!(!mirror_root.exists(), "and the mirror goes when it is removed");
    remove(root.as_path());
}

/// What:
///  Two jobs of one workspace never share a mirror,
///  and two workspaces of one worktree never share one either.
/// Why:
///   A job that removed a mirror another job's Cargo was running in failed that job for nothing
///  (an earlier round's finding), so the name carries the job;
///  and the workspace's own path decides the part above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theMirrorRoot(git, w, 'a')).not.toBe(theMirrorRoot(git, w, 'b')); expect(theMirrorRoot(git, one, 'a')).not.toBe(theMirrorRoot(git, two, 'a'))
/// ```
#[test]
fn two_jobs_and_two_workspaces_have_two_mirror_roots() {
    let git: PathBuf = PathBuf::from("/git");
    let one: PathBuf = PathBuf::from("/worktree/package/one");
    let two: PathBuf = PathBuf::from("/worktree/package/two");
    assert_ne!(the_mirror_root(git.as_path(), one.as_path(), "a"), the_mirror_root(git.as_path(), one.as_path(), "b"), "two jobs of one workspace");
    assert_ne!(the_mirror_root(git.as_path(), one.as_path(), "a"), the_mirror_root(git.as_path(), two.as_path(), "a"), "two workspaces of one job");
    assert!(the_mirror_root(git.as_path(), one.as_path(), "a").starts_with(git.as_path()), "every mirror is under the Git directory, outside the worktree");
}
