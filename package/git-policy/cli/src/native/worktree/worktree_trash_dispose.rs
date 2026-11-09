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
use crate::service_work::Verdict;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::TrashManifest;
use crate::parallel_budget::HostResources;
use crate::worktree_trash_entries::{EntryRemoval, remove_entries};
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_remove::remove_tree_beneath;
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
use std::sync::Arc;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashDispose";

/// The reference every trash work item carries;
/// the service accepts only a reference under `refs/`.
pub const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// The most bytes of a worktree's `.git` file that are read.
const POINTER_LIMIT: u64 = 64 * 1024;

/// What:
///  A trash the emptying left where it is,
/// and whether a removal that names it would delete only what the snapshot made.
/// Why:
///   The line the person reads must never hand them a command that deletes what the emptying
///  deliberately kept:
/// a directory under the recorded name that the snapshot did not make,
/// an entry that
///  is no longer the object the leave-out moved there,
/// and a file somebody put in the trash all stay,
///  so the line may give no removal command for them.
/// A step that could not finish leaves what it did
///  not examine where it is,
/// so a trash whose contents were not shown to be the snapshot's own is
///  never called the snapshot's either.
/// The one case that keeps the command is the trash directory the
///  snapshot made,
/// empty,
/// which only its parent's permissions kept from being removed;
/// `rm --recursive
///  --force` on it deletes exactly what the snapshot made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrashLeft = { reason: string; onlyTheSnapshots: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrashLeft {
    /// Why the trash stays,
    /// naming what is in it.
    pub reason: String,
    /// Whether everything the trash holds is what the snapshot made,
    /// so a recursive removal of its path deletes nothing else.
    pub only_the_snapshots: bool,
}

impl TrashLeft {
    /// What:
    ///  A refusal with the reason it gives,
    /// for a trash nothing showed to hold only the snapshot's own directories.
    /// Why:
    ///   A step that could not finish leaves what it did not examine where it is,
    /// and an unexamined
    ///  directory is never called the snapshot's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = (reason) => ({ reason, onlyTheSnapshots: false });
    /// ```
    fn kept(reason: String) -> TrashLeft {
        return TrashLeft { reason, only_the_snapshots: false };
    }

    /// What:
    ///  A refusal of the empty trash directory the snapshot made,
    /// which a refusal of its parent's permissions kept.
    /// Why:
    ///   Everything the trash held is gone,
    /// so nothing below it is anybody else's and the
    ///  removal command names only what the snapshot made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const keptEmptyTrash = (reason) => ({ reason, onlyTheSnapshots: true });
    /// ```
    fn kept_empty_trash(reason: String) -> TrashLeft {
        return TrashLeft { reason, only_the_snapshots: true };
    }
}

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
        // No work item and no durable record of this manifest is written, so nothing else will ever empty it: the line says
        //  that, names the directory, and gives the command that removes it the way the person's shell reads it.
        Err(left) => {
            let trash: PathBuf = destination.join(TRASH_PARENT).join(&manifest.name);
            let shown: String = crate::shell_quote::shell_quote(trash.as_os_str().as_bytes());
            let removing: String = match left.only_the_snapshots {
                true => format!(": `rm --recursive --force -- {shown}`"),
                // The command could delete what the emptying kept, so the line names what is there and gives none.
                false => String::from(". No removal command is given here: nothing showed that everything below that path is what this snapshot made, and a recursive removal would delete whatever the emptying left alone; remove by name only what you want gone"),
            };
            Err(format!("the directories left out of the new worktree {} were not emptied ({}); the trash {shown} stays inside the worktree, no work item names it, and only you will remove it{removing}", destination.display(), left.reason))
        }
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
/// await deleteTrash({ repository, gitDir, manifest }); // rejects with the reason something stays, and whether its removal is the snapshot's own
/// ```
pub fn delete_trash(repository: &Path, git_dir: &Path, manifest: &TrashManifest) -> Result<(), TrashLeft> {
    let root: File = path_failure(open_root(repository), repository, "cannot be opened as the worktree").map_err(TrashLeft::kept)?;
    verify_association(&root, repository, git_dir).map_err(TrashLeft::kept)?;
    let device: u64 = path_failure(device_of(&root), repository, "cannot be examined").map_err(TrashLeft::kept)?;
    let names = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(TrashLeft::kept(format!("the trash parent is not a plain path ({error})"))),
    };
    let mut shown: PathBuf = repository.to_path_buf();
    let mut holders: Vec<File> = vec![root];
    for name in &names {
        shown.push(name);
        let Some(held) = holders.last() else { return Err(TrashLeft::kept(format!("{} cannot be held", shown.display()))) };
        let next: File = match open_directory_at(held, name.as_os_str()) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(TrashLeft::kept(format!("{} is not a plain directory ({error}), so the trash below it is left alone", shown.display()))),
        };
        if device_of(&next).ok() != Some(device) {
            return Err(TrashLeft::kept(format!("{} is on another device, so the trash below it is left alone", shown.display())));
        }
        holders.push(next);
    }
    let Some(parent) = holders.last() else { return Ok(()) };
    let trash: Arc<File> = match open_directory_at(parent, std::ffi::OsStr::new(manifest.name.as_str())).map(Arc::new) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(TrashLeft::kept(format!("{} is not a plain directory ({error}), so it is left alone", shown.join(&manifest.name).display()))),
    };
    let found = path_failure(identity_of(&trash), shown.join(&manifest.name).as_path(), "cannot be examined").map_err(TrashLeft::kept)?;
    if found != manifest.trash {
        return Err(TrashLeft::kept(format!("{} is not the directory the snapshot made ({} instead of {}), so it is left alone", shown.join(&manifest.name).display(), found.encode(), manifest.trash.encode())));
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
        // An entry that stayed is either not the recorded object or one whose removal failed, and the line cannot tell
        //  them apart, so what the trash holds is not shown to be the snapshot's own.
        return Err(TrashLeft::kept(left.join("; ")));
    }
    let mut strangers: Vec<String> = Vec::new();
    for entry in path_failure(std::fs::read_dir(handle_path(&trash)), inside.as_path(), "cannot be listed").map_err(TrashLeft::kept)?.flatten() {
        strangers.push(entry.file_name().to_string_lossy().into_owned());
    }
    if !strangers.is_empty() {
        return Err(TrashLeft::kept(format!("{} also holds {}, which the snapshot did not put there, so the trash directory is left alone", inside.display(), strangers.join(", "))));
    }
    drop(trash);
    path_failure(remove_at(parent, std::ffi::OsStr::new(manifest.name.as_str()), true), inside.as_path(), "cannot be removed").map_err(TrashLeft::kept_empty_trash)?;
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
        Err(left) => Verdict::Failed(left.reason),
    };
}
