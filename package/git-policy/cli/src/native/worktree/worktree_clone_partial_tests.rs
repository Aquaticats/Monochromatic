//! Controls for the clone rung of a source with changes:
//! the paths their entries prove are cloned,
//! the paths a change reaches are written by Git,
//! the new index
//!  describes the worktree Git's own checkout would make,
//! and the result is proven before it is published.

use super::*;
use crate::fs_beneath::open_root;
use crate::fs_clone_exchange::clone_file;
use crate::test_support::{git, git_context, git_text, repository};
use crate::worktree_clean::head_commit;
use crate::worktree_ladder_target::Target;
use crate::worktree_snapshot_source::{SnapshotSource, source_or_dirty};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// What:
///  The fixture root when a file on it can be reflinked,
/// or nothing after saying that this control did not run.
/// Why:
///   A clone of a file with bytes needs `FICLONE`,
/// which the gate image's fixture root (a tmpfs) does not answer,
/// and a control that passed there would claim coverage it has not got.
///A root that cannot clone says so once,
///  with the control's name,
///  as a control that needs btrfs does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneCapableFixture(name: string): string | undefined;
/// ```
fn clone_capable_fixture(name: &str) -> Option<PathBuf> {
    let root: PathBuf = crate::test_support::fixture(name);
    let from: PathBuf = root.join("probe-from");
    let to: PathBuf = root.join("probe-to");
    std::fs::write(from.as_path(), b"probe\n").expect("the probe's file");
    std::fs::write(to.as_path(), b"").expect("the probe's clone");
    let opened = (std::fs::File::open(from.as_path()), std::fs::OpenOptions::new().write(true).open(to.as_path()));
    let cloned: bool = match opened {
        (Ok(source), Ok(destination)) => clone_file(&source, &destination, to.as_path()).is_ok(),
        _unreadable => false,
    };
    let _removed = (std::fs::remove_file(from.as_path()), std::fs::remove_file(to.as_path()));
    if cloned {
        return Some(root);
    }
    eprintln!("CLONE CONTROL NOT RUN: {name} (the fixture root cannot reflink a file)");
    return None;
}

/// What:
///  A main checkout with a tracked plain file,
/// an executable,
/// a link,
/// a file whose bytes a control changes,
/// a staged path,
///  a deleted path and an untracked path,
/// and a `--no-checkout` worktree beside it.
/// Why:
///   These are the shapes the design answers one by one,
/// and every one of them is a state the machine's own main
///  checkout is in on most days.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Dirty = { repo: string; destination: string; admin: string; source: SnapshotSource };
/// ```
struct Dirty {
    /// The source checkout.
    repo: PathBuf,
    /// The new worktree's directory,
    /// which Git made with no checkout.
    destination: PathBuf,
    /// The new worktree's administrative directory.
    admin: PathBuf,
    /// What the source check proved about the source.
    source: SnapshotSource,
}

/// The fixture's file contents,
/// which every control asserts.
const KEPT: &[u8] = b"the commit's bytes for a path nobody changed\n";
/// The bytes the edited file's commit holds.
const COMMITTED_EDIT: &[u8] = b"the commit's bytes\n";
/// The bytes the person wrote into the edited file.
const PERSONAL_EDIT: &[u8] = b"a change nobody committed\n";
/// The bytes the staged path's commit holds.
const COMMITTED_STAGED: &[u8] = b"what the commit holds\n";
/// The bytes the person staged for the staged path.
const STAGED_BYTES: &[u8] = b"what the person staged\n";
/// The bytes the deleted path's commit holds.
const COMMITTED_DELETED: &[u8] = b"a path the person deleted\n";

/// What:
///  A repository in the state the design calls dirty:
/// two paths the person changed,
/// one change staged,
/// one deletion,
/// and one untracked file.
/// Why:
///   A control of the partial clone needs every case the design answers in one source,
/// so the numbers it
///  asserts are the rung's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = await dirtyRepository(root); // throws the reason Git refused a step
/// ```
fn dirty_repository(root: &Path) -> Dirty {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "kept.txt", KEPT);
    tracked(repo.as_path(), "bin/tool.sh", KEPT);
    std::fs::set_permissions(repo.join("bin/tool.sh"), std::fs::Permissions::from_mode(0o755)).expect("executable");
    tracked(repo.as_path(), "edited.txt", COMMITTED_EDIT);
    tracked(repo.as_path(), "staged.txt", COMMITTED_STAGED);
    tracked(repo.as_path(), "deleted.txt", COMMITTED_DELETED);
    std::os::unix::fs::symlink("kept.txt", repo.join("link")).expect("link");
    crate::test_support::age(repo.join("link").as_path());
    commit_all(repo.as_path(), "the commit the person edits");
    std::fs::write(repo.join("edited.txt"), PERSONAL_EDIT).expect("the person's change");
    std::fs::write(repo.join("staged.txt"), STAGED_BYTES).expect("the staged change");
    git(repo.as_path(), &["add", "--", "staged.txt"]);
    std::fs::remove_file(repo.join("deleted.txt")).expect("the person's deletion");
    std::fs::write(repo.join("untracked.txt"), b"nobody tracks this\n").expect("the untracked file");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source with changes");
    assert!(source.dirty.is_some(), "the fixture's source has uncommitted changes");
    return Dirty { repo, destination, admin, source };
}

/// The target of the add:
/// the administrative directory Git made,
/// Git's pointer,
/// and the commit the new `HEAD` names.
fn target_of(dirty: &Dirty) -> Target {
    let hash_length: usize = dirty.source.clean.hash_length;
    let format = if hash_length == 32 { crate::repo_read_layout::ObjectFormat::Sha256 } else { crate::repo_read_layout::ObjectFormat::Sha1 };
    let commit: String = head_commit(dirty.admin.as_path(), dirty.source.common_dir.as_path(), format).expect("the new HEAD");
    let pointer: Vec<u8> = std::fs::read(dirty.destination.join(".git")).expect("Git's pointer");
    return Target { admin: dirty.admin.clone(), pointer, commit, differs: false, line: String::new() };
}

/// What:
///  Run the partial clone step,
/// prove the worktree it left,
/// and answer what it did.
/// Why:
///   Every control asks the same three things of the same fixture;
/// the step,
/// its proof,
/// and one line that
///  says what happened if either fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const part = await cloneAndProve(dirty); // throws the reason the worktree is not Git's
/// ```
fn clone_and_prove(dirty: &Dirty) -> PartCloned {
    let target: Target = target_of(dirty);
    let held: std::fs::File = open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File = open_root(dirty.admin.as_path()).expect("the administrative directory");
    let part: PartCloned = clone_the_unchanged(&git_context(), &dirty.source, &target, dirty.destination.as_path(), &held, &admin).expect("the partial clone");
    prove_the_worktree(&git_context(), &target, dirty.destination.as_path()).expect("the worktree Git's checkout makes");
    return part;
}

/// The partial clone clones the paths their entries prove,
/// writes the paths a change reaches,
/// leaves the untracked file behind,
/// and the worktree it leaves is clean at once.
#[test]
fn the_unchanged_paths_are_cloned_and_the_changed_ones_written() {
    let Some(root) = clone_capable_fixture("clone-partial") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let part: PartCloned = clone_and_prove(&dirty);
    // The two paths nothing changed, the executable and the link are clones; the three a change touches are Git's.
    assert_eq!(part.cloned, 3, "kept.txt, bin/tool.sh and link are cloned; edited.txt, staged.txt and deleted.txt are not");
    assert_eq!(part.written, 3);
    assert_eq!(git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]), "", "the first status of the worktree is clean");
    assert_eq!(std::fs::read(dirty.destination.join("edited.txt")).expect("the commit's bytes"), COMMITTED_EDIT);
    assert_eq!(std::fs::read(dirty.destination.join("staged.txt")).expect("the commit's bytes"), COMMITTED_STAGED);
    assert_eq!(std::fs::read(dirty.destination.join("deleted.txt")).expect("the commit's bytes"), COMMITTED_DELETED);
    assert!(!dirty.destination.join("untracked.txt").exists(), "an untracked file of the source reaches no worktree");
    assert_eq!(std::fs::read_link(dirty.destination.join("link")).expect("the link"), PathBuf::from("kept.txt"));
    let tool: std::fs::Metadata = std::fs::metadata(dirty.destination.join("bin/tool.sh")).expect("the executable");
    assert_eq!(tool.mode() & 0o7777, 0o755, "the cloned file keeps the mode Git's checkout writes");
    crate::test_support::remove(root.as_path());
}

/// The index the partial clone installs records the files the clones made,
/// so Git's first status of the new worktree costs no hashing of them.
#[test]
fn the_index_records_the_files_the_clones_made() {
    let Some(root) = clone_capable_fixture("clone-partial-index") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let _part: PartCloned = clone_and_prove(&dirty);
    let index: Vec<u8> = std::fs::read(dirty.admin.join("index")).expect("the new index");
    let entries = crate::worktree_clean::stored_stats(index.as_slice(), dirty.source.clean.hash_length).expect("the index's stat data");
    for name in ["kept.txt", "bin/tool.sh"] {
        let entry = entries.iter().find(|entry| { return entry.path.as_slice() == name.as_bytes(); }).expect("the entry of a cloned path");
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(dirty.destination.join(name)).expect("the cloned file");
        assert_eq!(entry.stored.size, metadata.size() as u32, "{name}");
        assert_eq!(entry.stored.mtime_seconds, metadata.mtime() as u32, "{name}");
        assert_eq!(entry.stored.ino, metadata.ino() as u32, "{name}");
    }
    // A link's entry keeps the fields the source recorded, so no name another process may have replaced is certified:
    //  Git compares the target, which is what the source's own index install leaves in a whole clone.
    let link_entry = entries.iter().find(|entry| { return entry.path.as_slice() == b"link"; }).expect("the link's entry");
    let source_entries = crate::worktree_clean::stored_stats(dirty.source.clean.index_bytes.as_slice(), dirty.source.clean.hash_length).expect("the source's stat data");
    let source_link = source_entries.iter().find(|entry| { return entry.path.as_slice() == b"link"; }).expect("the source's link entry");
    assert_eq!(link_entry.stored.ino, source_link.stored.ino, "a link's entry keeps the source's inode");
    crate::test_support::remove(root.as_path());
}

/// A path written after its entry was recorded is written by Git:
/// the entry's stat data names bytes the file no longer holds.
#[test]
fn a_path_written_after_its_entry_was_recorded_is_written() {
    let Some(root) = clone_capable_fixture("clone-partial-changed") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    std::fs::write(dirty.repo.join("kept.txt"), b"bytes the entry never recorded\n").expect("the write");
    let part: PartCloned = clone_and_prove(&dirty);
    assert_eq!(part.written, 4, "the written path joins the three the fixture's changes reach");
    assert_eq!(std::fs::read(dirty.destination.join("kept.txt")).expect("the commit's bytes"), KEPT);
    crate::test_support::remove(root.as_path());
}

/// The flow runs where no file can be reflinked:
/// a clone of a file with no bytes needs none,
/// and the paths Git writes are the same.
#[test]
fn the_flow_runs_where_no_file_can_be_cloned() {
    let root: PathBuf = crate::test_support::fixture("clone-partial-empty");
    let repository: PathBuf = std::fs::canonicalize(crate::test_support::repository(root.as_path(), "repo")).expect("canonical");
    tracked(repository.as_path(), "empty.txt", b"");
    std::os::unix::fs::symlink("empty.txt", repository.join("link")).expect("link");
    crate::test_support::age(repository.join("link").as_path());
    tracked(repository.as_path(), "edited.txt", COMMITTED_EDIT);
    commit_all(repository.as_path(), "the commit the person edits");
    std::fs::write(repository.join("edited.txt"), PERSONAL_EDIT).expect("the person's change");
    git(repository.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repository.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repository.as_path())).expect("a source with changes");
    let dirty: Dirty = Dirty { repo: repository, destination, admin, source };
    let part: PartCloned = clone_and_prove(&dirty);
    assert_eq!((part.cloned, part.written), (2, 1), "the empty file and the link are clones and the changed file is Git's");
    assert_eq!(git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]), "");
    crate::test_support::remove(root.as_path());
}
