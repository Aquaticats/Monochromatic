//! What:
//!  The disposal of the trash a snapshot's leave-out made:
//! hand it to the commit service as a work item,
//! or empty it here when
//!  the service cannot be reached,
//! and the service's handler.
//! Both delete only the directory the leave-out recorded and only
//!  the entries it recorded,
//! found by walking down from the worktree's handle without following a link.
//! Why:
//!  A name of forty hexadecimal digits proves nothing about who made a directory,
//! and a path string is resolved again by every
//!  call that takes it.
//! The manifest says which objects the leave-out made and moved;
//! the disposal opens each directory on the
//!  way by one plain name below the handle of the one above,
//! compares identities,
//! and leaves anything else alone with a
//!  reason.
//! A record stays until the trash is gone or the service has taken it,
//! so a failure is never a silent success.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await disposePendingTrash({ context, destination, gitDir, trash, notices }); // throws the reason the trash stays
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{device_of, handle_path, identity_of, open_directory_at, open_file_at, open_root, plain_components, remove_at};
use crate::service_work::{Verdict, work_directory};
use crate::transaction_git::GitContext;
use crate::work_item::{CLAIMED, FAILED, PENDING, RECOVERING, WorkItem, WorkKind, state_directory};
use crate::plain_file::read_plain_file;
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::TrashManifest;
use crate::parallel_budget::HostResources;
use crate::worktree_trash_entries::{EntryRemoval, remove_entries};
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_remove::remove_tree_beneath;
use std::fs::File;
use std::sync::Arc;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashDispose";

/// The most bytes of a work item record that is read:
/// the manifest of one trash,
/// which is short.
const RECORD_LIMIT: u64 = 64 * 1024;

/// The reference every trash work item carries;
/// the service accepts only a reference under `refs/`.
pub const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// The most bytes of a worktree's `.git` file that are read.
const POINTER_LIMIT: u64 = 64 * 1024;

/// What:
///  A snapshot's trash waiting for the disposal:
/// the worktree that holds it,
/// its administrative directory,
/// what the
///  leave-out moved,
/// and whether the person asked for silence.
/// Why:
///   The add hands it to the lifecycle,
/// which empties it beside the package repair instead of before it,
/// so the
///  person waits for the slower of the two and not for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingTrash = { destination: string; gitDir: string; manifest: TrashManifest; quiet: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTrash {
    /// The new worktree that holds the trash.
    pub destination: PathBuf,
    /// That worktree's administrative directory.
    pub git_dir: PathBuf,
    /// What the leave-out made and moved.
    pub manifest: TrashManifest,
    /// Whether the person's `--quiet` silences the notices of the hand-over.
    pub quiet: bool,
}

/// What:
///  Empty the trash a snapshot made in the new worktree `destination`,
/// when it made one,
///  before the command that made it returns.
/// Why:
///   The trash holds the directories the leave-out set aside,
/// another command's stage container with its
///  credentials among them,
/// and it sits inside the new worktree.
/// While it stands there,
/// a target commit that no longer
///  ignores its path makes it untracked,
/// so `git add --all` in the new worktree stages another command's state.
///  Handing it to the commit service kept it alive until that service ran,
/// which is exactly that window,
/// so the
///  emptying happens here:
/// the add waits for it,
/// beside the package repair (`worktree_copy_after_lock.rs`),
/// and a trash
///  that cannot be emptied is reported with its path and the reason and never outlives the command
/// silently.
/// A work item of an earlier release that still names a trash is emptied by the same handler
/// (`empty_trash`),
/// so a repository that ran one keeps its promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await disposePendingTrash({ destination, gitDir, trash, notices }); // throws the reason the trash stays
/// ```
pub fn dispose_pending_trash(_context: &GitContext, destination: &Path, git_dir: &Path, trash: Option<&TrashManifest>, notices: &mut dyn Write) -> Result<(), String> {
    let Some(manifest) = trash else { return Ok(()) };
    return empty_before_returning(destination, git_dir, manifest, notices);
}

/// What:
///  Name every trash in this worktree that nothing will empty.
/// Why:
///   A command empties the trash of its own snapshot before it returns (`empty_before_returning`),
/// so a trash that could not be emptied stays in the worktree,
/// and nothing else would ever mention it again.
/// The trash is a full copy of the directories the snapshot left out,
/// so one that stays is disk a person cannot find without being told.
/// A record an earlier release wrote accounts for its trash while the service still works on it,
/// because its own handler empties that one (`empty_trash`);
/// the service settles a trash it cannot empty as failed and never tries it again,
/// so a failed record accounts for nothing here.
/// This scan deletes nothing:
/// a directory nothing recorded cannot be proven to be one the leave-out moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nameUnaccountedTrash(gitDir, worktree, notices);
/// ```
pub fn name_unaccounted_trash(git_dir: &Path, worktree: &Path, notices: &mut dyn Write) {
    // A worktree whose trash parent is gone has nothing to name, and that is the ordinary case.
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    if std::fs::symlink_metadata(&parent).is_err() {
        return;
    }
    let work: PathBuf = work_directory(git_dir);
    let mut accounted: Vec<String> = Vec::new();
    for state in [PENDING, CLAIMED, RECOVERING, FAILED] {
        let Ok(listing) = std::fs::read_dir(state_directory(work.as_path(), state)) else { continue };
        for entry in listing.flatten() {
            let record: PathBuf = entry.path();
            // A record is a plain file: a named pipe or a device in a state directory is somebody's own entry, and
            // opening it would wait for a writer that never comes, on every command in this worktree.
            let Ok(bytes) = read_plain_file(record.as_path(), RECORD_LIMIT) else { continue };
            let Ok(text) = String::from_utf8(bytes) else { continue };
            let Ok(item) = WorkItem::decode(text.as_str()) else { continue };
            if item.kind != WorkKind::Trash {
                continue;
            }
            // A record the service still works on accounts for its trash: its own handler empties that one, and naming
            // it here would send the person to look at work that is not theirs. A failed record is settled for good,
            // so the trash it names is left to the scan below, which names it and deletes nothing.
            if state == FAILED {
                continue;
            }
            accounted.push(item.oid.clone());
        }
    }
    name_trash_nothing_will_empty(worktree, &accounted, notices);
}

/// Say what sits in the trash parent of `worktree` that nothing will empty.
/// A directory nothing recorded cannot be proven to be one the leave-out moved,
/// so it is only named.
fn name_trash_nothing_will_empty(worktree: &Path, accounted: &[String], notices: &mut dyn Write) {
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    let Ok(listing) = std::fs::read_dir(&parent) else { return };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if accounted.contains(&name) {
            continue;
        }
        let _told = writeln!(
            notices,
            "monochromatic-git-policy-cli: {} holds directories a snapshot left out that nothing will empty now (the commit service gave up on the record, or there is none), so they are left as they are: look at them, and remove them yourself if nothing in them is wanted.",
            entry.path().display()
        );
    }
}

/// What:
///  Empty one trash here and now,
/// telling the person what is being done once.
/// Why:
///   The person waits for this,
/// so they are told why the command takes longer,
/// and a failure says what
///  stays without failing the add,
/// because the worktree is whole either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeln(notices, `emptying the directories left out of ${destination} now`); return deleteTrash({ destination, gitDir, manifest });
/// ```
fn empty_before_returning(destination: &Path, git_dir: &Path, manifest: &TrashManifest, notices: &mut dyn Write) -> Result<(), String> {
    let _told = writeln!(notices, "monochromatic-git-policy-cli: emptying the directories left out of the new worktree {} now, so nothing another command was filling stays in it; this can take a while.", destination.display());
    return match delete_trash(destination, git_dir, manifest) {
        Ok(()) => Ok(()),
        Err(reason) => Err(format!("the directories left out of the new worktree {} were not emptied ({reason}); the trash {} stays there, and a later command in this worktree names it", destination.display(), manifest.name)),
    };
}

/// Whether an error says a directory is not empty.
fn is_not_empty(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::ENOTEMPTY | libc::EEXIST));
}

/// Whether two paths that both resolve name the same place;
/// a path that cannot be resolved names none.
fn same_after_resolving(left: &Path, right: &Path) -> bool {
    return match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(resolved_left), Ok(resolved_right)) => resolved_left == resolved_right,
        _ => false,
    };
}

/// Check that the worktree at `repository` is the one whose administrative directory is `git_dir`:
/// its `.git` file,
/// read
///  below the held root,
/// points at that directory.
fn verify_association(root: &File, repository: &Path, git_dir: &Path) -> Result<(), String> {
    let pointer: File = match open_file_at(root, std::ffi::OsStr::new(".git")) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("{} has no readable .git file ({error}), so it is no linked worktree and its trash is left alone", repository.display())),
    };
    let is_file: bool = match pointer.metadata() {
        Ok(found) => found.is_file(),
        Err(_unreadable) => false,
    };
    if !is_file {
        return Err(format!("{}/.git is not a file, so its trash is left alone", repository.display()));
    }
    let mut text: String = String::new();
    if let Err(error) = std::io::Read::read_to_string(&mut std::io::Read::take(&pointer, POINTER_LIMIT), &mut text) {
        return Err(format!("{}/.git cannot be read ({error})", repository.display()));
    }
    let Some(after_prefix) = text.strip_prefix("gitdir:") else {
        return Err(format!("{}/.git does not point at an administrative directory, so its trash is left alone", repository.display()));
    };
    let recorded: &str = after_prefix.trim();
    let pointed: PathBuf = if Path::new(recorded).is_absolute() { PathBuf::from(recorded) } else { repository.join(recorded) };
    let same: bool = pointed == git_dir || same_after_resolving(pointed.as_path(), git_dir);
    if !same {
        return Err(format!("{}/.git points at {} and not at {}, so its trash is left alone", repository.display(), pointed.display(), git_dir.display()));
    }
    return Ok(());
}

/// What:
///  Empty the trash directory the manifest describes,
/// in the worktree `repository` whose administrative directory is
///  `git_dir`;
/// a trash that is already gone is done.
/// Why:
///   Every directory from the worktree down is opened by one plain name below the handle of the one above,
/// so a link at any
///  level is an error and never followed.
/// The trash is the directory with the recorded identity,
/// each entry in it is removed
///  only when it is the object that was moved there,
/// and anything else in the trash stays with a reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deleteTrash({ repository, gitDir, manifest }); // throws the reason something stays
/// ```
pub fn delete_trash(repository: &Path, git_dir: &Path, manifest: &TrashManifest) -> Result<(), String> {
    let root: File = path_failure(open_root(repository), repository, "cannot be opened as the worktree")?;
    verify_association(&root, repository, git_dir)?;
    let device: u64 = path_failure(device_of(&root), repository, "cannot be examined")?;
    let names = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(format!("the trash parent is not a plain path ({error})")),
    };
    let mut shown: PathBuf = repository.to_path_buf();
    let mut holders: Vec<File> = vec![root];
    for name in &names {
        shown.push(name);
        let Some(held) = holders.last() else { return Err(format!("{} cannot be held", shown.display())) };
        let next: File = match open_directory_at(held, name.as_os_str()) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("{} is not a plain directory ({error}), so the trash below it is left alone", shown.display())),
        };
        if device_of(&next).ok() != Some(device) {
            return Err(format!("{} is on another device, so the trash below it is left alone", shown.display()));
        }
        holders.push(next);
    }
    let Some(parent) = holders.last() else { return Ok(()) };
    let trash: Arc<File> = match open_directory_at(parent, std::ffi::OsStr::new(manifest.name.as_str())).map(Arc::new) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} is not a plain directory ({error}), so it is left alone", shown.join(&manifest.name).display())),
    };
    let found = path_failure(identity_of(&trash), shown.join(&manifest.name).as_path(), "cannot be examined")?;
    if found != manifest.trash {
        return Err(format!("{} is not the directory the snapshot made ({} instead of {}), so it is left alone", shown.join(&manifest.name).display(), found.encode(), manifest.trash.encode()));
    }
    let inside: PathBuf = shown.join(&manifest.name);
    let left: Vec<String> = remove_entries(&EntryRemoval {
        inside: inside.clone(),
        trash: Arc::clone(&trash),
        manifest: Arc::new(manifest.clone()),
        host: HostResources::read(),
        remove: Arc::new(remove_tree_beneath),
    });
    if !left.is_empty() {
        return Err(left.join("; "));
    }
    let mut strangers: Vec<String> = Vec::new();
    for entry in path_failure(std::fs::read_dir(handle_path(&trash)), inside.as_path(), "cannot be listed")?.flatten() {
        strangers.push(entry.file_name().to_string_lossy().into_owned());
    }
    if !strangers.is_empty() {
        return Err(format!("{} also holds {}, which the snapshot did not put there, so the trash directory is left alone", inside.display(), strangers.join(", ")));
    }
    drop(trash);
    path_failure(remove_at(parent, std::ffi::OsStr::new(manifest.name.as_str()), true), inside.as_path(), "cannot be removed")?;
    // The parent goes when nothing else is in it; a refusal only means another trash directory is still there.
    if let [.., above, _] = holders.as_slice()
        && let Some(last) = names.last()
        && let Err(error) = remove_at(above, last.as_os_str(), true)
        && !is_not_empty(&error)
    {
        debug(TAG, format!("{} cannot be removed ({error}); it stays", shown.display()).as_str());
    }
    return Ok(());
}

/// What:
///  The service's handler:
/// empty the trash directory a work item names,
/// after checking that it is the one the snapshot made.
/// Why:
///   The record is a file in the Git directory;
/// the handler trusts nothing in it beyond what it can verify:
/// the worktree's
///  `.git` file points at the record's Git directory,
/// every directory down to the trash is a real one,
/// the trash has the
///  recorded identity,
/// and each entry removed is the recorded object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emptyTrash(item) // 'done' | { failed: reason }
/// ```
pub fn empty_trash(item: &WorkItem) -> Verdict {
    let manifest: TrashManifest = match TrashManifest::decode(item.oid.as_str(), item.parent.as_str()) {
        Ok(found) => found,
        Err(reason) => return Verdict::Failed(reason),
    };
    return match delete_trash(item.repository.as_path(), item.git_dir.as_path(), &manifest) {
        Ok(()) => Verdict::Done,
        Err(reason) => Verdict::Failed(reason),
    };
}
