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
use crate::iso_time::now_iso;
use crate::service_client::{Activation, ClientConfig, START_WAIT, manager_watch_for, wake};
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_messages::{ClientFailure, render};
use crate::service_provision::{ProvisionHost, provision};
use crate::service_route::{Route, route_from_environment};
use crate::service_work::{Verdict, work_directory};
use crate::transaction_git::GitContext;
use crate::work_item::{PENDING, WorkItem, WorkKind, enqueue, state_directory};
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

/// The reference every trash work item carries;
/// the service accepts only a reference under `refs/`.
pub const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// The most bytes of a worktree's `.git` file that are read.
const POINTER_LIMIT: u64 = 64 * 1024;

/// How the service's address is found for a Git directory;
/// injected so a control never reaches the real service.
pub type Router<'a> = &'a dyn Fn(&Path, &Path) -> Option<Route>;

/// Sets up what the service needs before it can be reached,
/// as a commit does for its first message;
/// injected so a control
///  never reaches the session bus.
pub type Provisioner<'a> = &'a dyn Fn(&ProvisionHost) -> Result<(), ClientFailure>;

/// How the service is reached:
/// where its address is found,
/// how it is set up,
/// and how long its start is waited for.
pub struct Reach<'a> {
    /// Finds the service's address for a Git directory.
    pub router: Router<'a>,
    /// Sets up the worktree's socket activation,
    /// which a new worktree does not have yet.
    pub provisioner: Provisioner<'a>,
    /// The longest wait for the service to start.
    pub start_wait: std::time::Duration,
}

/// Where the trash went.
enum Handed {
    /// The service was reached with a record that names the trash,
    /// and owns the emptying now.
    Service,
    /// The service was not reached;
    /// the trash is emptied here.
    /// `record` is the work item that was written for it,
    /// if any,
    /// and
    ///  `why` says in words why the service was not reached.
    Here {
        /// The pending work item,
        /// which stays until the trash is gone.
        record: Option<PathBuf>,
        /// Why the service was not reached.
        why: String,
    },
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
///  Dispose of the trash a snapshot made in the new worktree `destination`,
/// when it made one.
/// Why:
///   The wrapper never starts the service itself;
/// writing the record and connecting to its socket does.
/// A trash that no one
///  empties is only disk,
/// but the foreground fallback means the worktree never keeps one past the command that made it,
/// and a
///  trash that cannot be emptied is reported with its path and the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await disposePendingTrash({ context, destination, gitDir, trash, notices }); // throws the reason the trash stays
/// ```
pub fn dispose_pending_trash(context: &GitContext, destination: &Path, git_dir: &Path, trash: Option<&TrashManifest>, notices: &mut dyn Write) -> Result<(), String> {
    let Some(manifest) = trash else { return Ok(()) };
    return dispose_pending_trash_with(context.real_git.as_path(), destination, git_dir, manifest, notices, &Reach { router: &route_from_environment, provisioner: &provision, start_wait: START_WAIT });
}

/// `dispose_pending_trash` for a known trash,
/// with the route finder and the wait for a current service injected,
/// so a control
///  can use a fixture service and a short wait.
pub fn dispose_pending_trash_with(real_git: &Path, destination: &Path, git_dir: &Path, manifest: &TrashManifest, notices: &mut dyn Write, reach: &Reach<'_>) -> Result<(), String> {
    let (record, why): (Option<PathBuf>, String) = match hand_over(real_git, destination, git_dir, manifest, notices, reach) {
        Handed::Service => return Ok(()),
        Handed::Here { record, why } => (record, why),
    };
    debug(TAG, format!("the trash {} could not be handed to the service ({why}); emptying it here", manifest.name).as_str());
    // The person waits for this, so they are told what is being done and why, once.
    let _told = writeln!(notices, "monochromatic-git-policy-cli: emptying the directories left out of the new worktree {} now, because the commit service could not take them over ({why}); this can take a while.", destination.display());
    if let Err(reason) = delete_trash(destination, git_dir, manifest) {
        let stays: String = match record {
            Some(path) => format!("the work item {} stays, so the commit service empties it when it next runs for this worktree", path.display()),
            None => String::from("no work item could be written for it, so it stays until it is removed by hand"),
        };
        // "Not emptied now": a removal the pool queued may still run later (`no_threads_child_test_support`).
        return Err(format!("the directories left out of the new worktree {} were not emptied now ({reason}); {stays}", destination.display()));
    }
    // The trash is gone, so the record must not outlive it as work for a service to find.
    if let Some(path) = record
        && let Err(error) = std::fs::remove_file(&path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        debug(TAG, format!("the work item {} could not be withdrawn ({error}); the service finds the trash gone", path.display()).as_str());
    }
    return Ok(());
}

/// Write the work item,
/// set up the worktree's socket activation,
/// and connect to the service.
///
/// Why:
/// a new worktree has a commit service instance of its own whose socket does not exist until something sets it up;
/// a
/// commit does that before its first message,
/// so the hand-over does it too,
/// instead of waiting out the start for a socket
/// that nothing will ever create.
fn hand_over(real_git: &Path, destination: &Path, git_dir: &Path, manifest: &TrashManifest, notices: &mut dyn Write, reach: &Reach<'_>) -> Handed {
    // A path that is not text is left to the emptying here: the wake message to the service still spells a Git directory
    //  as text, so a record alone cannot reach it.
    let (Some(repository), Some(git_dir_text)) = (destination.to_str(), git_dir.to_str()) else {
        debug(TAG, format!("{} or its administrative directory is not valid UTF-8, so no work item can name it", destination.display()).as_str());
        return Handed::Here { record: None, why: String::from("the path of the worktree is not valid UTF-8, so a work item cannot name it") };
    };
    let item: WorkItem = WorkItem {
        kind: WorkKind::Trash,
        repository: PathBuf::from(repository),
        git_dir: PathBuf::from(git_dir_text),
        branch_ref: String::from(TRASH_REFERENCE),
        oid: manifest.name.clone(),
        parent: manifest.encode(),
        created: now_iso(),
        tries: 0,
    };
    if let Err(reason) = enqueue(work_directory(git_dir).as_path(), &item) {
        debug(TAG, format!("the work item could not be written: {reason}").as_str());
        return Handed::Here { record: None, why: format!("the work item could not be written: {reason}") };
    }
    let record: PathBuf = state_directory(work_directory(git_dir).as_path(), PENDING).join(item.id());
    let Some(Route { address, provision: host }) = (reach.router)(real_git, git_dir) else {
        return Handed::Here { record: Some(record), why: String::from("no commit service can be started from this session") };
    };
    if let Some(setup) = host.as_ref()
        && let Err(failure) = (reach.provisioner)(setup)
    {
        return Handed::Here { record: Some(record), why: render(&failure) };
    }
    let config: ClientConfig = ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: reach.start_wait,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: host.as_ref().map(manager_watch_for),
        keep_fresh: None, answer_ceiling: crate::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING, extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: crate::wait_ceilings::START_CEILING, waiting_notice_after: crate::wait_ceilings::waiting_notice_after(),
        units_record_dir: host.as_ref().map(crate::service_provision_host_and_manager::units_record_dir_of),
    };
    return match wake(&config, git_dir, None, notices) {
        Ok(()) => Handed::Service,
        Err(failure) => Handed::Here { record: Some(record), why: render(&failure) },
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
