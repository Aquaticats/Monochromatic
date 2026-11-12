//! What:
//!  The emptying of the trash a snapshot's leave-out made:
//! what a trash is,
//! how the commit service's handler deletes it,
//! and how every later command
//!  names a trash nothing will empty.
//! The deletion takes only the directory the leave-out recorded and only the
//!  entries it recorded,
//! found by walking down from the worktree's handle without following a link,
//! and at the lowest I/O
//!  priority Linux offers.
//! Why:
//!  A name of forty hexadecimal digits proves nothing about who made a directory,
//! and a path string is resolved again by every
//!  call that takes it.
//! The manifest says which objects the leave-out made and moved;
//! the deletion opens each directory on the
//!  way by one plain name below the handle of the one above,
//! compares identities,
//! and leaves anything else alone with a
//!  reason.
//! A record stays until the trash is gone or the service has taken it,
//! so a failure is never a silent success.
//! The add never waits for any of this (`worktree_trash_hand_over.rs` hands the trash over):
//! the
//!  service empties one trash at a time for a repository,
//! at the idle I/O class,
//! so concurrent adds do not delete at
//!  once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! emptyTrash(item); // the service's handler, and the reason a trash stays
//! ```

use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::diagnostic_log::debug;
use crate::fs_beneath::{device_of, handle_path, identity_of, open_directory_at, open_file_at, open_root, plain_components, remove_at};
use crate::idle_io_priority::with_idle_io_priority;
use crate::service_work::Verdict;
use crate::work_item::WorkItem;
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::TrashManifest;
use crate::parallel_budget::HostResources;
use crate::worktree_trash_entries::{EntryRemoval, remove_entries, remove_entries_awaiting};
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_remove::remove_tree_beneath;
use std::fs::File;
use std::sync::Arc;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashDispose";

/// The most bytes of a work item record that is read:
/// the manifest of one trash,
/// which is short;
/// the scan that names what nothing will empty reads them with this bound.
pub const RECORD_LIMIT: u64 = 64 * 1024;

/// The reference every trash work item carries;
/// the service accepts only a reference under `refs/`.
pub const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// The most bytes of a worktree's `.git` file that are read.
const POINTER_LIMIT: u64 = 64 * 1024;

/// What:
///  A trash the emptying left where it is,
/// with the reason it gives for every step it would not take.
/// Why:
///   A directory under the recorded name that the snapshot did not make,
/// an entry that is no longer the object
///  the leave-out moved there,
/// and a file somebody put in the trash all stay,
/// so the reason names what is there and the
///  service reports what it did not delete.
/// A step that could not finish leaves what it did not examine where it is,
///  and the reason says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrashLeft = { reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrashLeft {
    /// Why the trash stays,
    /// naming what is in it.
    pub reason: String,
}

impl TrashLeft {
    /// What:
    ///  A refusal with the reason it gives.
    /// Why:
    ///   Every reason the emptying answers with is the same shape,
    /// so the service's line and the records cannot disagree
    ///  about one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = (reason) => ({ reason });
    /// ```
    fn kept(reason: String) -> TrashLeft {
        return TrashLeft { reason };
    }
}

/// What:
///  A snapshot's trash waiting for the commit service:
/// the worktree that holds it,
/// the repository's common
///  directory that its record is written under,
/// what the leave-out moved,
/// and whether the person asked for silence.
/// Why:
///   The add hands it over and returns;
/// the record is the common directory's,
/// because one repository's
///  service drains one work directory after another and so empties one trash at a time,
/// whichever
///  worktree holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingTrash = { destination: string; commonDir: string; manifest: TrashManifest; quiet: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTrash {
    /// The new worktree that holds the trash.
    pub destination: PathBuf,
    /// The repository's common Git directory,
    /// whose work directory holds the record and whose service empties it.
    pub common_dir: PathBuf,
    /// What the leave-out made and moved.
    pub manifest: TrashManifest,
    /// How many of the trash's directories came from an earlier add's trash,
    /// which nothing of this add would ever
    ///  have emptied;
    /// the hand-over says so when it is more than none,
    /// because every add hands over at least the stand-ins and the
    ///  `.git` copy it made itself.
    pub taken_over: usize,
    /// Whether the person's `--quiet` silences the notices of the hand-over.
    pub quiet: bool,
}

/// The trash's own name,
/// the last component of its path,
/// for the manifest inside it.
fn trash_name_of(inside: &Path) -> String {
    let Some(name) = inside.file_name() else { return String::new() };
    return name.to_string_lossy().into_owned();
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

/// What:
///  Whether the administrative directory a worktree's `.git` file names belongs to the directory the record was read
///  from:
/// that directory itself,
/// or one of the per-worktree directories below it.
/// Why:
///   A trash's record is written under the repository's common directory,
/// because one repository's service
///  empties its trashes one at a time and a record is drained where it was read;
/// the worktree that holds the
///  trash is a linked one,
/// so its pointer names `<common>/worktrees/<name>` and never the common directory
///  itself.
/// The proof is still a proof:
/// the pointer names a directory the record's Git directory contains,
/// and the
///  emptying then opens every step below the worktree without following a link and compares the trash's identity.
/// A path
///  that resolves to the same place is accepted as well,
/// so a repository whose paths are reached through a link is not
///  refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pointsInto = (gitDir, pointed) => pointed === gitDir || pointed.parent() === join(gitDir, 'worktrees') || sameAfterResolving(pointed, gitDir) || sameAfterResolving(pointed.parent(), join(gitDir, 'worktrees'));
/// ```
fn points_into(git_dir: &Path, pointed: &Path) -> bool {
    if pointed == git_dir || same_after_resolving(pointed, git_dir) {
        return true;
    }
    // Every per-worktree directory of a repository sits below `worktrees` in its common directory; a record and a pointer
    //  that reach it through different names are the same place when their parents resolve to one.
    let below: PathBuf = git_dir.join("worktrees");
    let Some(parent) = pointed.parent() else { return false };
    return parent == below || same_after_resolving(parent, below.as_path());
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
    if !points_into(git_dir, pointed.as_path()) {
        return Err(format!("{}/.git points at {} and not into {}, so its trash is left alone", repository.display(), pointed.display(), git_dir.display()));
    }
    return Ok(());
}

/// What:
///  The trash of one worktree as the removal holds it:
/// the directories opened on the way down,
/// the trash itself,
/// the path every message names,
/// and the names walked to reach it.
/// Why:
///   Every directory stays open by its own name,
/// so nothing below the handles can be replaced by a link between two steps;
/// and the entries go before the directory that holds them,
/// so the two are separate steps of one removal,
///  each of which may be a job of the blocking pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldTrash = { holders: File[]; trash: Arc<File>; inside: string; shown: string; names: OsString[] };
/// ```
#[derive(Clone)]
struct HeldTrash {
    /// The worktree's root and every directory below it on the way to the trash parent,
    /// shared so a job the system refused a thread for can be done in turn with the same handles.
    holders: Vec<Arc<File>>,
    /// The trash directory,
    /// held open.
    trash: Arc<File>,
    /// The trash's path,
    /// which every message names.
    inside: PathBuf,
    /// The path the walk down has reached.
    shown: PathBuf,
    /// The plain names walked from the root to the trash parent.
    names: Vec<std::ffi::OsString>,
}

/// What:
///  The removal of one trash as the state of one job of the blocking pool.
/// Why:
///   The budget runner takes a function and the state it reads,
/// never a closure,
/// and this state is what an awaiting caller hands the pool the moment its handles are open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ATrashToOpen = { repository: string; gitDir: string; manifest: TrashManifest };
/// ```
#[derive(Clone)]
struct ATrashToOpen {
    /// The worktree that holds the trash.
    repository: PathBuf,
    /// That worktree's administrative directory.
    git_dir: PathBuf,
    /// What the leave-out made and moved.
    manifest: TrashManifest,
}

/// What:
///  Open every directory from the worktree down to the trash and prove the trash is the one the snapshot made,
/// or answer nothing when it is already gone.
/// Why:
///   These are lookups and identities,
/// and they come before the first removal:
/// a trash that is gone is done,
///  and a directory that is not the recorded object is left alone.
/// A worktree that is gone is done too:
/// a removal of that worktree took
///  the trash with it,
/// so a record that outlived it names nothing on this disk and needs no report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTrashRemoval({ repository, gitDir, manifest }): HeldTrash | undefined;
/// ```
fn prepare_trash_removal(ATrashToOpen { repository, git_dir, manifest }: ATrashToOpen) -> Result<Option<HeldTrash>, String> {
    let root: File = match open_root(repository.as_path()) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{} cannot be opened as the worktree ({error})", repository.display())),
    };
    verify_association(&root, repository.as_path(), git_dir.as_path())?;
    let device: u64 = path_failure(device_of(&root), repository.as_path(), "cannot be examined")?;
    let names = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(format!("the trash parent is not a plain path ({error})")),
    };
    let mut shown: PathBuf = repository.clone();
    let mut holders: Vec<Arc<File>> = vec![Arc::new(root)];
    for name in &names {
        shown.push(name);
        let Some(held) = holders.last() else { return Err(format!("{} cannot be held", shown.display())) };
        let next: File = match open_directory_at(held.as_ref(), name.as_os_str()) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{} is not a plain directory ({error}), so the trash below it is left alone", shown.display())),
        };
        if device_of(&next).ok() != Some(device) {
            return Err(format!("{} is on another device, so the trash below it is left alone", shown.display()));
        }
        holders.push(Arc::new(next));
    }
    let Some(parent) = holders.last() else { return Ok(None) };
    let trash: Arc<File> = match open_directory_at(parent.as_ref(), std::ffi::OsStr::new(manifest.name.as_str())).map(Arc::new) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{} is not a plain directory ({error}), so it is left alone", shown.join(&manifest.name).display())),
    };
    let found = path_failure(identity_of(&trash), shown.join(&manifest.name).as_path(), "cannot be examined")?;
    if found != manifest.trash {
        return Err(format!("{} is not the directory the snapshot made ({} instead of {}), so it is left alone", shown.join(&manifest.name).display(), found.encode(), manifest.trash.encode()));
    }
    let inside: PathBuf = shown.join(&manifest.name);
    return Ok(Some(HeldTrash { holders, trash, inside, shown, names }));
}

/// What:
///  The removal of one trash's entries as the state of one job:
/// what was opened,
/// and what the leave-out recorded.
/// Why:
///   The entries are removed by the one worker pool of the process,
/// and each entry is removed only when it is the object that was moved there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRemovalOfTrashEntries = { inside: string; trash: Arc<File>; manifest: TrashManifest };
/// ```
struct TheRemovalOfTrashEntries {
    /// The trash's path,
    /// which every reason names.
    inside: PathBuf,
    /// The trash directory,
    /// held open.
    trash: Arc<File>,
    /// What the leave-out moved there.
    manifest: TrashManifest,
}

/// What:
///  Ask for one entry removal request of the trash's entries.
/// Why:
///   The synchronous and the awaiting removal take the same request,
/// so the two faces differ in the wait only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { inside, trash, manifest, host: hostResources(), remove: removeTreeBeneath };
/// ```
fn entry_removal_of(TheRemovalOfTrashEntries { inside, trash, manifest }: TheRemovalOfTrashEntries) -> EntryRemoval {
    return EntryRemoval {
        inside,
        trash,
        manifest: Arc::new(manifest),
        host: HostResources::read(),
        remove: Arc::new(remove_tree_beneath),
    };
}

/// What:
///  Remove the trash's own directory,
/// which only its permissions kept.
/// Why:
///   The name inside its parent is the trash's name,
/// so the removal deletes exactly what the snapshot made and nothing beside it.
fn remove_the_trash_parent(state: (&File, &Path)) -> std::io::Result<()> {
    let (parent, inside) = state;
    return remove_at(parent, std::ffi::OsStr::new(inside.file_name().unwrap_or_default()), true);
}

/// What:
///  The name the manifest is taken to while the emptying decides whether it is the file that was read.
/// Why:
///   The take is one rename inside the trash the emptying is about to remove,
/// so it reaches nothing outside it,
/// and the name says what happened to a file the emptying found there and would not remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MANIFEST_TAKEN_NAME = '.manifest.removing';
/// ```
const MANIFEST_TAKEN_NAME: &str = ".manifest.removing";

/// What:
///  Remove the manifest file from the trash,
///  only while the object it takes is the one whose bytes were read as that manifest.
/// Why:
///   A name can be taken between the read and the removal,
/// and nothing in the kernel makes the two one step;
/// the removal takes the name with one rename,
/// so the object it removes is the one it examined,
/// and a file another process moved onto the name
///  keeps its bytes at that name
/// and keeps the whole directory, which is reported instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeTheManifest(trash, inside, expected) { const outcome = takeThenRemoveIfIdentical(trash, '.manifest', MANIFEST_TAKEN_NAME, expected); if (outcome !== 'removed' && outcome !== 'absent') throw kept(`the manifest inside ${inside} ...`); }
/// ```
fn remove_the_manifest(trash: &File, inside: &Path, expected: crate::fs_beneath::ObjectId) -> Result<(), TrashLeft> {
    return match crate::fs_beneath::take_then_remove_if_identical(
        trash,
        std::ffi::OsStr::new(crate::worktree_trash_own_manifest::MANIFEST_FILE_NAME),
        std::ffi::OsStr::new(MANIFEST_TAKEN_NAME),
        expected,
    ) {
        Ok(crate::fs_beneath::Owned::Removed | crate::fs_beneath::Owned::Absent) => Ok(()),
        Ok(crate::fs_beneath::Owned::Kept(reason)) => Err(TrashLeft::kept(format!(
            "the manifest inside {} is not the file that was read as it ({reason}), so the trash directory is left alone",
            inside.display()
        ))),
        Err(error) => Err(TrashLeft::kept(format!("the manifest inside {} cannot be removed ({error}), so the trash directory is left alone", inside.display()))),
    };
}

/// What:
///  Remove the trash directory itself once its entries are gone,
/// and the parent above it when nothing else is in it.
/// Why:
///   Anything the snapshot did not put there keeps the whole directory,
/// so nothing of another command is removed by mistake,
///  and a parent that still holds another trash stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleTrashRemoval(held: HeldTrash): void; // throws the reason something stays
/// ```
fn settle_trash_removal(held: HeldTrash) -> Result<(), TrashLeft> {
    let HeldTrash { holders, trash, inside, shown, names } = held;
    let own: Option<crate::worktree_trash_own_manifest::OwnManifest> = crate::worktree_trash_own_manifest::its_own(&trash, trash_name_of(inside.as_path()).as_str());
    let mut strangers: Vec<String> = Vec::new();
    for entry in path_failure(std::fs::read_dir(handle_path(&trash)), inside.as_path(), "cannot be listed").map_err(TrashLeft::kept)?.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        // The manifest the wrapper wrote inside its own trash goes with it, and only while the file at that name is the object the
        //  bytes were read through; a file of that name that is not that manifest keeps the whole directory, as every other thing
        //  the snapshot did not put there does.
        if own.is_some() && name == crate::worktree_trash_own_manifest::MANIFEST_FILE_NAME {
            continue;
        }
        strangers.push(name);
    }
    if !strangers.is_empty() {
        return Err(TrashLeft::kept(format!("{} also holds {}, which the snapshot did not put there, so the trash directory is left alone", inside.display(), strangers.join(", "))));
    }
    if let Some(leaving) = own.as_ref() {
        remove_the_manifest(&trash, inside.as_path(), leaving.file)?;
    }
    drop(trash);
    let Some(parent) = holders.last() else { return Ok(()) };
    // Everything the trash held is gone, so only its parent's permissions kept it: a removal that names it deletes
    //  exactly what the snapshot made.
    path_failure(with_idle_io_priority((parent.as_ref(), inside.as_path()), remove_the_trash_parent), inside.as_path(), "cannot be removed").map_err(TrashLeft::kept)?;
    // The parent goes when nothing else is in it; a refusal only means another trash directory is still there.
    if let [.., above, _] = holders.as_slice()
        && let Some(last) = names.last()
        && let Err(error) = remove_at(above.as_ref(), last.as_os_str(), true)
        && !is_not_empty(&error)
    {
        debug(TAG, format!("{} cannot be removed ({error}); it stays", shown.display()).as_str());
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
/// The callers of this face run on a thread with no runtime of its own,
/// and the synchronous handler face is one of
///  them;
/// it does the lookups here and the entries through the one removal face that enters the runtime.
/// A caller that is
///  itself a task of the runtime awaits `delete_trash_awaiting` instead:
/// the drain of a round asks
///  `empty_trash_awaiting` (`worktree_trash_dispose_awaiting.rs`),
/// because the synchronous face is refused from inside the
///  runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deleteTrash({ repository, gitDir, manifest }); // rejects with the reason something stays
/// ```
pub fn delete_trash(repository: &Path, git_dir: &Path, manifest: &TrashManifest) -> Result<(), TrashLeft> {
    let opened: ATrashToOpen = ATrashToOpen { repository: repository.to_path_buf(), git_dir: git_dir.to_path_buf(), manifest: manifest.clone() };
    let Some(held) = prepare_trash_removal(opened).map_err(TrashLeft::kept)? else { return Ok(()) };
    let left: Vec<String> = remove_entries(&entry_removal_of(TheRemovalOfTrashEntries { inside: held.inside.clone(), trash: Arc::clone(&held.trash), manifest: manifest.clone() }));
    if !left.is_empty() {
        // An entry that stayed is either not the recorded object or one whose removal failed, and the line cannot tell
        //  them apart, so what the trash holds is not shown to be the snapshot's own.
        return Err(TrashLeft::kept(left.join("; ")));
    }
    return settle_trash_removal(held);
}

/// What:
///  `delete_trash` on the process's runtime:
/// the lookups and the last removal run as jobs of the blocking pool,
///  and the entries are removed by the awaited ordered runner's own jobs.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
/// and a synchronous lookup on a task that other work shares would hold that work;
/// so every step of this removal is a job of the pool and the task only waits.
/// The steps are the synchronous face's:
/// the same `prepare_trash_removal`,
/// the same request and the same `settle_trash_removal`,
///  so the two faces cannot remove different things,
/// and both answer with the same `TrashLeft`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function deleteTrashAwaiting({ repository, gitDir, manifest }): Promise<void>; // rejects with the reason
/// ```
pub async fn delete_trash_awaiting(repository: &Path, git_dir: &Path, manifest: &TrashManifest) -> Result<(), TrashLeft> {
    let opened: ATrashToOpen = ATrashToOpen { repository: repository.to_path_buf(), git_dir: git_dir.to_path_buf(), manifest: manifest.clone() };
    let opened_trash: Option<HeldTrash> = match run_a_job_or_in_turn(opened, prepare_trash_removal).await {
        Ok(Ok(held)) => held,
        // A directory that is not the recorded one is left alone with its reason, and nothing was removed.
        Ok(Err(reason)) => return Err(TrashLeft::kept(reason)),
        // Nothing was opened and nothing was removed, and the reason the system gave is why.
        Err(refused) => return Err(TrashLeft::kept(format!("the directories of the trash {} could not be examined ({refused})", manifest.name))),
    };
    let Some(held) = opened_trash else { return Ok(()) };
    let left: Vec<String> = remove_entries_awaiting(&entry_removal_of(TheRemovalOfTrashEntries { inside: held.inside.clone(), trash: Arc::clone(&held.trash), manifest: manifest.clone() })).await;
    if !left.is_empty() {
        // An entry that stayed is either not the recorded object or one whose removal failed, and the line cannot tell
        //  them apart, so what the trash holds is not shown to be the snapshot's own.
        return Err(TrashLeft::kept(left.join("; ")));
    }
    return match run_a_job_or_in_turn(held, settle_trash_removal).await {
        Ok(outcome) => outcome,
        // The entries are gone and the directory above them stays, which the words say.
        Err(refused) => Err(TrashLeft::kept(format!("the trash directory {} stays ({refused})", manifest.name))),
    };
}

/// What:
///  The service's handler:
/// empty the trash directory a work item names,
/// after checking that it is the one the snapshot made.
/// Why:
///   The record is a file in the Git directory;
/// the handler trusts nothing in it beyond what it can verify:
/// the worktree's
///  `.git` file points into the record's Git directory,
/// every directory down to the trash is a real one,
/// the trash has the
///  recorded identity,
/// and each entry removed is the recorded object.
///
/// A caller that is itself a task of the runtime,
/// which the drain of a round is,
/// asks `empty_trash_awaiting`
/// (`worktree_trash_dispose_awaiting.rs`) instead:
/// this face enters the runtime for the entry removals and is refused from
/// inside it.
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

/// The emptying's own controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_dispose_tests.rs"]
mod tests;
