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

use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::diagnostic_log::debug;
use crate::fs_beneath::{device_of, handle_path, identity_of, open_directory_at, open_file_at, open_root, plain_components, remove_at};
use crate::service_work::{Verdict, work_directory};
use crate::transaction_git::GitContext;
use crate::work_item::{CLAIMED, FAILED, PENDING, RECOVERING, WorkItem, WorkKind, state_directory};
use crate::plain_file::read_plain_file;
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_manifest::TrashManifest;
use crate::parallel_budget::HostResources;
use crate::worktree_trash_entries::{EntryRemoval, remove_entries, remove_entries_awaiting};
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_remove::remove_tree_beneath;
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
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
///  `dispose_pending_trash` on the process's runtime:
/// the emptying is the awaited removal,
/// so a caller that is itself a task of the runtime holds no storage wait.
/// Why:
///   The add empties a snapshot's trash beside the package repair,
/// and both are waits of one task of the runtime
/// (`worktree_copy_after_lock.rs`);
/// a lookup or a removal on that task would hold the repair's own waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await disposePendingTrash({ destination, gitDir, trash, notices }); // throws the reason the trash stays
/// ```
pub async fn dispose_pending_trash_awaiting(_context: &GitContext, destination: &Path, git_dir: &Path, trash: Option<&TrashManifest>, notices: &mut dyn Write) -> Result<(), String> {
    let Some(manifest) = trash else { return Ok(()) };
    return empty_before_returning_awaiting(destination, git_dir, manifest, notices).await;
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
    let _told = writeln!(notices, "{}", emptying_words(destination));
    return match delete_trash(destination, git_dir, manifest) {
        Ok(()) => Ok(()),
        // Both sides, one line: a later command in this worktree names the trash and deletes nothing from it, and a
        //  removal command is given only when that command removes exactly what the snapshot made.
        Err(left) => Err(trash_stays_words(destination, manifest, &left)),
    };
}

/// What:
///  `empty_before_returning` on the process's runtime:
/// the line the person reads is said here and the removal is awaited.
/// Why:
///   The two faces say the same line and report the same reason,
/// so a caller on the runtime cannot tell which of them ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function emptyBeforeReturning({ destination, gitDir, manifest, notices }): Promise<void>;
/// ```
async fn empty_before_returning_awaiting(destination: &Path, git_dir: &Path, manifest: &TrashManifest, notices: &mut dyn Write) -> Result<(), String> {
    let _told = writeln!(notices, "{}", emptying_words(destination));
    return match delete_trash_awaiting(destination, git_dir, manifest).await {
        Ok(()) => Ok(()),
        Err(left) => Err(trash_stays_words(destination, manifest, &left)),
    };
}

/// What:
///  The line that tells the person why the command takes longer.
/// Why:
///   Both faces of the emptying say the same words,
/// so the two cannot report the same work differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const emptyingWords = (destination: string) => `monochromatic-git-policy-cli: emptying the directories left out of the new worktree ${destination} now, ...`;
/// ```
fn emptying_words(destination: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: emptying the directories left out of the new worktree {} now, so nothing another command was filling stays in it; this can take a while.",
        destination.display()
    );
}

/// What:
///  What stays when the emptying could not finish,
///  named with the reason and the directory.
/// Why:
///   The trash outlives a command that could not empty it,
/// so the person is told which directory to look at,
/// and the same words are the failure the two faces report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trashStaysWords = (destination, manifest, reason) => `the directories left out of the new worktree ${destination} were not emptied (${reason}); the trash ${manifest.name} stays there, ...`;
/// ```
fn trash_stays_words(destination: &Path, manifest: &TrashManifest, left: &TrashLeft) -> String {
    let trash: PathBuf = destination.join(TRASH_PARENT).join(&manifest.name);
    let shown: String = crate::shell_quote::shell_quote(trash.as_os_str().as_bytes());
    let removing: String = match left.only_the_snapshots {
        true => format!(": `rm --recursive --force -- {shown}`"),
        // The command could delete what the emptying kept, so the line names what is there and gives none.
        false => String::from(". No removal command is given here: nothing showed that everything below that path is what this snapshot made, and a recursive removal would delete whatever the emptying left alone; remove by name only what you want gone"),
    };
    return format!(
        "the directories left out of the new worktree {} were not emptied ({}); the trash {shown} stays inside the worktree, no work item names it, a later command in this worktree names it and deletes nothing, and only you will remove it{removing}",
        destination.display(),
        left.reason
    );
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTrashRemoval({ repository, gitDir, manifest }): HeldTrash | undefined;
/// ```
fn prepare_trash_removal(ATrashToOpen { repository, git_dir, manifest }: ATrashToOpen) -> Result<Option<HeldTrash>, String> {
    let root: File = path_failure(open_root(repository.as_path()), repository.as_path(), "cannot be opened as the worktree")?;
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
    let mut strangers: Vec<String> = Vec::new();
    for entry in path_failure(std::fs::read_dir(handle_path(&trash)), inside.as_path(), "cannot be listed").map_err(TrashLeft::kept)?.flatten() {
        strangers.push(entry.file_name().to_string_lossy().into_owned());
    }
    if !strangers.is_empty() {
        return Err(TrashLeft::kept(format!("{} also holds {}, which the snapshot did not put there, so the trash directory is left alone", inside.display(), strangers.join(", "))));
    }
    drop(trash);
    let Some(parent) = holders.last() else { return Ok(()) };
    // Everything the trash held is gone, so only its parent's permissions kept it: a removal that names it deletes
    //  exactly what the snapshot made, and the line the person reads may say so.
    path_failure(remove_at(parent.as_ref(), std::ffi::OsStr::new(inside.file_name().unwrap_or_default()), true), inside.as_path(), "cannot be removed").map_err(TrashLeft::kept_empty_trash)?;
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
/// The callers of this face run on a thread with no runtime of its own (a command that hands the trash
/// over,
/// and the synchronous handler face),
///  so it does the lookups here and the entries through the one removal face that enters the runtime.
/// A caller that is itself a task of the runtime awaits `delete_trash_awaiting` instead:
/// the drain of a
///  round asks `empty_trash_awaiting`
/// (`worktree_trash_dispose_awaiting.rs`),
/// because the synchronous face is refused from inside the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deleteTrash({ repository, gitDir, manifest }); // rejects with the reason something stays, and whether its removal is the snapshot's own
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
/// async function deleteTrashAwaiting({ repository, gitDir, manifest }): Promise<void>; // rejects with the reason and whether its removal is the snapshot's own
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
        Err(refused) => Err(TrashLeft::kept_empty_trash(format!("the trash directory {} stays ({refused})", manifest.name))),
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
///  `.git` file points at the record's Git directory,
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
