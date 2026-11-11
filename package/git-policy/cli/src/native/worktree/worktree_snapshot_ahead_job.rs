//! What:
//!  The commit service's own snapshot of a source root,
//! made in the folder an add makes its stage in,
//! for the add after this one to claim.
//! Why:
//!  An add's own `BTRFS_IOC_SNAP_CREATE_V2` forces a btrfs transaction commit on the person's thread and waits for the
//!  source's writeback first,
//! which took 7,443.8 ms at the median and 39,828.5 ms at its worst on this volume
//! (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
//! "A worktree add").
//! A client cannot bound that call,
//! because a thread asleep in the ioctl
//!  keeps the process from exiting,
//! so the call lives here,
//! in the service,
//! and the add waits for the answer
//!  instead (`worktree_snapshot_ahead.rs`,
//! `SNAPSHOT_AHEAD_CEILING`).
//! The service makes one entry at a time and stops when the folder already
//!  holds what an add keeps (`AHEAD_KEPT`),
//! so a burst of requests cannot fill the volume with copies of one tree.
//! Nothing here proves the copy:
//! the add that claims one walks it against its own source's index before it installs
//!  that index over it (`worktree_snapshot_index_step.rs`,
//! `prove_the_copy`),
//! so a stale copy is refused there and
//!  handed back here for deletion.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = snapshotAhead(item); // names the folder, makes one snapshot of item.repository, or says why not
//! ```

use crate::diagnostic_log::debug;
use crate::fs_btrfs_ioctl::snapshot_create_at;
use crate::fs_kind::FilesystemKind::Btrfs;
use crate::fs_kind::{filesystem_kind, is_subvolume_root};
use crate::fs_primitive_error::PrimitiveError;
use crate::service_work::Verdict;
use crate::work_item::WorkItem;
use crate::private_storage::create_private_directory_all;
use crate::worktree_snapshot_ahead::{AHEAD_KEPT, ahead_name, kept_in, pool_folder};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

/// Tag used for diagnostics from this module.
const TAG: &str = "snapshotAheadJob";

/// What:
///  The Git directory the worktree at `root` uses:
/// `<root>/.git` itself when that is a directory,
/// or the directory
///  its pointer file names.
/// Why:
///   The record names the worktree and the Git directory it copies,
/// and the two must be one worktree before either is
///  used,
/// so the question is asked from the source the record names.
/// A main checkout's Git directory is its
///  own `.git`;
/// a linked worktree's,
/// and a main checkout made with `git init --separate-git-dir`,
/// have a `.git`
///  pointer file that names it,
/// and a pointer may be relative to the folder it stands in.
/// Nothing here searches the
///  filesystem for a root:
/// a record can name a tree that is no worktree only because it says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDirectoryOf = (root) => statSync(join(root, '.git')).isDirectory() ? join(root, '.git') : resolvePointer(readFileSync(join(root, '.git'), 'utf8'), root);
/// ```
fn git_directory_of_worktree(root: &Path) -> Option<PathBuf> {
    let pointer: PathBuf = root.join(".git");
    if std::fs::metadata(pointer.as_path()).is_ok_and(|found| { return found.is_dir(); }) {
        return Some(pointer);
    }
    let text: String = std::fs::read_to_string(pointer.as_path()).ok()?;
    let named: &str = text.strip_prefix("gitdir:")?.trim();
    if named.is_empty() {
        return None;
    }
    let target: &Path = Path::new(named);
    return Some(if target.is_absolute() { target.to_path_buf() } else { root.join(target) });
}

/// What:
///  The source root and the folder this record names,
/// or why it names none this service may use.
/// Why:
///   Every field of a record is written by whoever can write the work directory,
/// so the two paths are checked against the
///  repository before either is used:
/// the source is the worktree whose Git directory the record carries,
/// and the
///  folder is one on this filesystem that already exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planned = (item) => { const gitDir = gitDirectoryOf(item.repository); if (gitDir !== item.gitDir) throw; if (filesystemKind(item.parent) !== 'btrfs') throw; return { root: item.repository, pool: item.parent }; };
/// ```
fn planned(item: &WorkItem) -> Result<(PathBuf, PathBuf), String> {
    let root: PathBuf = item.repository.clone();
    let Some(named) = git_directory_of_worktree(root.as_path()) else {
        return Err(format!("{} is no worktree this service can name: it has no Git directory", root.display()));
    };
    if named != item.git_dir {
        return Err(format!(
            "the record names {} as the source of a snapshot, and the Git directory of that worktree is {}",
            root.display(),
            named.display()
        ));
    }
    // The folder is the repository's own, inside its Git directory, so no record can make this service copy a tree into a place of
    //  its choosing; the record names it for a reader, and a record that names another folder is refused.
    let pool: PathBuf = pool_folder(item.git_dir.as_path());
    if Path::new(item.parent.as_str()) != pool {
        return Err(format!("the record names {} as the folder of a snapshot, and this repository's is {}", item.parent, pool.display()));
    }
    if filesystem_kind(root.as_path()) != Ok(Btrfs) {
        return Err(format!("{} is not on a btrfs filesystem, so nothing can share its data", root.display()));
    }
    if !is_subvolume_root(root.as_path()).unwrap_or(false) {
        return Err(format!("{} is not a btrfs subvolume, so nothing can copy it in one step", root.display()));
    }
    return Ok((root, pool));
}

/// What:
///  How many copies the folder holds that no record is already taking away,
/// which is the number that decides whether an add has one to claim.
/// Why:
///   A copy an add proved stale,
/// or one that aged out,
/// has a delete record of its own,
/// and that delete runs
///  in the same round of the service as this request,
/// after the copies are made;
/// counting it as kept would answer the
///  request with a full pool that is about to be emptied,
/// and no record would ask again for the add that is
///  waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptOrLeaving = (pool, root, queued) => keptIn(pool, root).filter((name) => !queued.has(deletionOf(pool, name).id));
/// ```
fn copies_that_stay(git_dir: &Path, pool: &Path, root: &Path) -> usize {
    let queued: std::collections::HashSet<String> = crate::work_item::queued_ids(crate::service_work::work_directory(git_dir).as_path());
    let mut staying: usize = 0;
    for name in kept_in(pool, root) {
        let leaving: crate::work_item::WorkItem = crate::worktree_snapshot_ahead::deletion_item(git_dir, pool, name.as_os_str());
        if !queued.contains(leaving.id().as_str()) {
            staying += 1;
        }
    }
    return staying;
}

/// What:
///  Make one snapshot of this repository's source root in the folder the wrapper keeps the snapshots ahead in,
/// unless that
///  folder already holds what an add keeps.
/// Why:
///  The add after this one claims it with one rename instead of forcing a commit of the whole filesystem on the
///  person's thread.
/// A folder that already holds `AHEAD_KEPT` copies no record is taking away is the answer to the requests of the
///  adds that came before this one,
/// so nothing more is made and the record is done;
/// a copy whose delete is queued is
///  room the round is about to free,
/// so the request is answered now.
/// The ioctl runs where the person is not waiting:
/// this is
///  the service's own thread,
/// taken from its blocking pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshotAhead = (item) => { const {root, pool} = planned(item); if (keptIn(pool, root).length >= AHEAD_KEPT) return Skipped(); const name = aheadName(root, Date.now()); snapshotCreateAt(root, join(pool, name)); return Done; };
/// ```
pub fn snapshot_ahead(item: &WorkItem) -> Verdict {
    let (root, pool): (PathBuf, PathBuf) = match planned(item) {
        Ok(found) => found,
        // None of these can pass by being tried again: the record names what the service may not copy, or the source can
        //  never be copied in one step.
        Err(why) => return Verdict::Failed(why),
    };
    if let Err(error) = create_private_directory_all(pool.as_path()) {
        return Verdict::Retry(format!("the folder of the snapshots ahead, {}, cannot be made ({error})", pool.display()));
    }
    let held: usize = copies_that_stay(item.git_dir.as_path(), pool.as_path(), root.as_path());
    if held >= AHEAD_KEPT {
        debug(TAG, format!("{}: {} snapshot(s) ahead are already there, so none more is made", root.display(), held).as_str());
        return Verdict::Skipped(format!("{held} snapshots ahead are already waiting in {}, and an add keeps {AHEAD_KEPT}", pool.display()));
    }
    let made_in: OsString = match ahead_name(root.as_path(), SystemTime::now()) {
        Ok(name) => name,
        // A random source the operating system refuses is not a snapshot this service can name repeatedly.
        Err(why) => return Verdict::Failed(why),
    };
    let destination: PathBuf = pool.join(&made_in);
    let began: Instant = Instant::now();
    let outcome: Result<(), PrimitiveError> = snapshot_create_at(root.as_path(), destination.as_path());
    return match outcome {
        Ok(()) => {
            crate::service_log!("made a snapshot ahead of {} in {} snapshot_ms={}", root.display(), destination.display(), began.elapsed().as_millis());
            Verdict::Done
        }
        // The kernel's own refusal of a source it cannot snapshot is worth no second try, and its words name the reason.
        Err(error) if matches!(error.errno, libc::EINVAL | libc::EXDEV | libc::EPERM) => Verdict::Failed(format!(
            "{} cannot be snapshotted into {} ({error}), so every add of it takes its own snapshot",
            root.display(),
            pool.display()
        )),
        Err(error) => Verdict::Retry(format!("the snapshot of {} could not be made ({error})", root.display())),
    };
}

/// Controls of the service's own snapshot stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_ahead_job_tests.rs"]
mod tests;
