//! What:
//!  The leave-out step of a snapshot of the main checkout: set aside the nested worktrees that are ordinary directories,
//!  the other checkouts, the caches and the run records that the main-checkout rule leaves out, into a trash directory inside
//!  the new subvolume; and, once the worktree is in place, hand the trash to the commit service as a work item that empties it.
//! Why:
//!  A snapshot carries everything, so the leave-out is a removal. Unlinking about 495 thousand entries takes 14 s, which the
//!  person should not wait for; renaming 480 directories into one trash directory takes 1.6 s, 1.2 s of which is the walk.
//!  The rename keeps the new worktree final at once. The `.git` pointer of a nested worktree that was copied names another
//!  worktree's administrative directory, so it is unlinked before anything else and before the add returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const count = await pruneSnapshot({ stage, source }); // before the swap
//! await disposePendingTrash({ context, destination, gitDir, notices }); // after it
//! ```

use super::diagnostic_log::debug;
use super::fs_kind::is_subvolume_root;
use super::iso_time::now_iso;
use super::random_id::random_uuid;
use super::service_client::{Activation, COMMIT_WAIT, ClientConfig, START_WAIT, wake};
use super::service_route::{Route, route_from_environment};
use super::service_work::{Verdict, work_directory};
use super::subvolume_remove::remove_tree;
use super::transaction_git::{GitContext, GitRequest, run_git};
use super::work_item::{WorkItem, WorkKind, enqueue};
use super::worktree_clean::stored_stats;
use super::worktree_copy_prune::Prune;
use super::worktree_snapshot_source::SnapshotSource;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrash";

/// The directory, relative to a worktree root, inside which each snapshot's trash directory is made.
/// It sits below `.claude/`, which the repository ignores, so a trash that has not been emptied yet is no untracked file.
pub const TRASH_PARENT: &str = ".claude/cli-git-trash";

/// The reference every trash work item carries; the service accepts only a reference under `refs/`.
const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// What:
///  Whether Git ignores the place the trash would be made at in this worktree.
/// Why:
///   A trash directory Git does not ignore would list as untracked until emptied, so the leave-out removes in the foreground
///  instead when the answer is no or unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit(['check-ignore', '--quiet', '--no-index', '--', `${TRASH_PARENT}/probe/x`]).exitCode === 0
/// ```
pub fn trash_location_is_ignored(context: &GitContext, root: &Path) -> bool {
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["check-ignore", "--quiet", "--no-index", "--"].iter().map(std::ffi::OsString::from).chain(std::iter::once(std::ffi::OsString::from(format!("{TRASH_PARENT}/probe/x")))).collect(),
        without_prefix: true,
        ..GitRequest::default()
    };
    return run_git(context, &request).is_ok_and(|output| { return output.code == Some(0); });
}

/// A name for one trash directory: forty hexadecimal digits, which is also the object name the work item carries.
fn new_trash_name() -> Result<String, String> {
    let first: String = random_uuid().map_err(|error| { return format!("a name for the trash could not be made: {error:?}"); })?;
    let second: String = random_uuid().map_err(|error| { return format!("a name for the trash could not be made: {error:?}"); })?;
    let digits: String = format!("{first}{second}").chars().filter(char::is_ascii_hexdigit).collect();
    return Ok(digits.chars().take(40).collect());
}

/// Whether a name is one this module made: forty lowercase hexadecimal digits.
fn is_trash_name(name: &str) -> bool {
    return name.len() == 40 && name.bytes().all(|byte: u8| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte); });
}

/// Where left-out directories go: into one trash directory when there is one, else they are removed in place.
struct Aside {
    /// The trash directory, absent when the leave-out removes in the foreground.
    trash: Option<PathBuf>,
    /// How many directories were set aside.
    count: usize,
}

impl Aside {
    /// Set one directory aside by renaming it into the trash, or remove it when there is no trash.
    fn put(&mut self, from: &Path) -> Result<(), String> {
        match self.trash.as_ref() {
            Some(trash) => {
                let to: PathBuf = trash.join(format!("d{}", self.count));
                std::fs::rename(from, &to).map_err(|error: std::io::Error| { return format!("{} cannot be set aside ({error})", from.display()); })?;
            }
            None => remove_tree(from).map_err(|error: std::io::Error| { return format!("{} cannot be removed ({error})", from.display()); })?,
        }
        self.count += 1;
        return Ok(());
    }
}

/// Every directory above a tracked file, as repository paths: these are never left out whatever their name.
fn tracked_directories(source: &SnapshotSource) -> Result<HashSet<Vec<u8>>, String> {
    let entries = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length).ok_or_else(|| { return String::from("the index entries have no plain layout"); })?;
    let mut directories: HashSet<Vec<u8>> = HashSet::new();
    for entry in entries {
        let mut end: usize = entry.path.len();
        while let Some(at) = entry.path[..end].iter().rposition(|byte: &u8| { return *byte == b'/'; }) {
            if !directories.insert(entry.path[..at].to_vec()) {
                break;
            }
            end = at;
        }
    }
    return Ok(directories);
}

/// What:
///  Set aside, inside a fresh snapshot of the main checkout, everything the main-checkout rule leaves out; return how many
///  directories were set aside.
/// Why:
///   The order matters: the `.git` pointer of every copied nested worktree and of every other checkout goes first, because a Git
///  command run inside such a copy would read and write another worktree's administrative directory; then the directories move
///  into the trash. A directory above a tracked file is never moved, whatever its name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = await pruneSnapshot({ stage, source }); // throws the reason; the caller destroys the stage
/// ```
pub fn prune_snapshot(stage: &Path, source: &SnapshotSource) -> Result<usize, String> {
    if source.git_dir != source.common_dir {
        return Ok(0);
    }
    let prune: Prune = Prune::main_checkout();
    let trash: Option<PathBuf> = if source.trash_ignored {
        let directory: PathBuf = stage.join(TRASH_PARENT).join(new_trash_name()?);
        std::fs::create_dir_all(&directory).map_err(|error: std::io::Error| { return format!("{} cannot be made ({error})", directory.display()); })?;
        Some(directory)
    } else {
        None
    };
    let mut aside: Aside = Aside { trash: trash.clone(), count: 0 };
    for nested in &source.nested {
        let Ok(relative) = nested.strip_prefix(source.root.as_path()) else { continue };
        if is_subvolume_root(nested.as_path()).unwrap_or(false) {
            continue;
        }
        let copy: PathBuf = stage.join(relative);
        if !std::fs::symlink_metadata(&copy).is_ok_and(|metadata: std::fs::Metadata| { return metadata.is_dir(); }) {
            continue;
        }
        unlink_pointer(copy.as_path())?;
        aside.put(copy.as_path())?;
    }
    let tracked: HashSet<Vec<u8>> = tracked_directories(source)?;
    let mut pending: Vec<(PathBuf, String)> = vec![(stage.to_path_buf(), String::new())];
    while let Some((directory, relative)) = pending.pop() {
        let mut children: Vec<(String, bool)> = Vec::new();
        for listed in std::fs::read_dir(&directory).map_err(|error: std::io::Error| { return format!("{} cannot be listed ({error})", directory.display()); })? {
            let entry: std::fs::DirEntry = listed.map_err(|error: std::io::Error| { return format!("{} cannot be listed ({error})", directory.display()); })?;
            let is_directory: bool = entry.file_type().is_ok_and(|kind: std::fs::FileType| { return kind.is_dir(); });
            children.push((entry.file_name().to_string_lossy().into_owned(), is_directory));
        }
        let names: Vec<String> = children.iter().map(|(name, _is_directory): &(String, bool)| { return name.clone(); }).collect();
        // A directory that is another checkout is set aside whole, its pointer first.
        if !relative.is_empty() && prune.skips_checkout(names.as_slice()) && !tracked.contains(relative.as_bytes()) {
            unlink_pointer(directory.as_path())?;
            aside.put(directory.as_path())?;
            continue;
        }
        for (name, is_directory) in children {
            if !is_directory {
                continue;
            }
            let child_relative: String = if relative.is_empty() { name.clone() } else { format!("{relative}/{name}") };
            if child_relative == ".git" || child_relative == TRASH_PARENT {
                continue;
            }
            if !tracked.contains(child_relative.as_bytes()) && prune.skips_path(child_relative.as_str()) {
                aside.put(directory.join(&name).as_path())?;
            } else {
                pending.push((directory.join(&name), child_relative));
            }
        }
    }
    if aside.count == 0
        && let Some(empty) = trash.as_ref()
    {
        let _ignored: std::io::Result<()> = std::fs::remove_dir(empty);
    }
    debug(TAG, format!("{} directories set aside in {} ({})", aside.count, stage.display(), if trash.is_some() { "into the trash" } else { "removed in place" }).as_str());
    return Ok(aside.count);
}

/// Unlink the `.git` file of a copied directory, when it is a file; a `.git` directory is a repository of its own and stays.
fn unlink_pointer(directory: &Path) -> Result<(), String> {
    let pointer: PathBuf = directory.join(".git");
    match std::fs::symlink_metadata(&pointer) {
        Ok(metadata) if metadata.is_file() => return std::fs::remove_file(&pointer).map_err(|error: std::io::Error| { return format!("{} cannot be unlinked ({error})", pointer.display()); }),
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} cannot be examined ({error})", pointer.display())),
    }
}

/// What:
///  Hand every trash directory in a worktree that a snapshot just made to the commit service as a work item, or empty it here
///  when the service cannot be reached.
/// Why:
///   The wrapper never starts the service itself; writing the record and connecting to its socket does. A trash that no one
///  empties is only disk, but the foreground fallback means the worktree never keeps one past the command that made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await disposePendingTrash({ context, destination, gitDir, notices });
/// ```
pub fn dispose_pending_trash(context: &GitContext, destination: &Path, git_dir: &Path, notices: &mut dyn Write) {
    dispose_pending_trash_with(context.real_git.as_path(), destination, git_dir, notices, &route_from_environment);
}

/// How the service's address is found for a Git directory; injected so a control never reaches the real service.
pub type Router<'a> = &'a dyn Fn(&Path, &Path) -> Option<Route>;

/// `dispose_pending_trash` with the route finder injected.
pub fn dispose_pending_trash_with(real_git: &Path, destination: &Path, git_dir: &Path, notices: &mut dyn Write, router: Router<'_>) {
    let parent: PathBuf = destination.join(TRASH_PARENT);
    let Ok(listing) = std::fs::read_dir(&parent) else { return };
    let mut names: Vec<String> = listing.flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name: &String| { return is_trash_name(name.as_str()); }).collect();
    names.sort();
    for name in names {
        let handed: bool = hand_over(real_git, destination, git_dir, name.as_str(), notices, router);
        if !handed {
            debug(TAG, format!("the trash {name} could not be handed to the service; emptying it here").as_str());
            if let Err(error) = remove_tree(parent.join(&name).as_path()) {
                debug(TAG, format!("the trash {name} could not be emptied here either ({error}); it stays under an ignored path").as_str());
            }
        }
    }
    // The parent goes when nothing else is in it; a refusal only means another trash is still pending.
    let _ignored: std::io::Result<()> = std::fs::remove_dir(&parent);
}

/// Write the work item and connect to the service; whether both succeeded.
fn hand_over(real_git: &Path, destination: &Path, git_dir: &Path, name: &str, notices: &mut dyn Write, router: Router<'_>) -> bool {
    let item: WorkItem = WorkItem {
        kind: WorkKind::Trash,
        repository: destination.display().to_string(),
        git_dir: git_dir.display().to_string(),
        branch_ref: String::from(TRASH_REFERENCE),
        oid: String::from(name),
        parent: String::from(name),
        created: now_iso(),
        tries: 0,
    };
    if let Err(reason) = enqueue(work_directory(git_dir).as_path(), &item) {
        debug(TAG, format!("the work item could not be written: {reason}").as_str());
        return false;
    }
    let Some(Route { address, .. }) = router(real_git, git_dir) else { return false };
    let config: ClientConfig = ClientConfig { address, activation: Activation::Managed, start_wait: START_WAIT, commit_wait: COMMIT_WAIT };
    return wake(&config, git_dir, None, notices).is_ok();
}

/// What:
///  The service's handler: empty the trash directory a work item names, after checking that it is one this module made.
/// Why:
///   The record is a file in the Git directory; the handler trusts nothing in it beyond what it can verify: a name of forty
///  hexadecimal digits, a real directory (not a link) at the one place trash is made, in a worktree that still has a `.git`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emptyTrash(item) // 'done' | { failed: reason }
/// ```
pub fn empty_trash(item: &WorkItem) -> Verdict {
    if !is_trash_name(item.oid.as_str()) || item.parent != item.oid {
        return Verdict::Failed(format!("{} is not the name of a trash directory", item.oid));
    }
    let repository: &Path = Path::new(item.repository.as_str());
    let parent: PathBuf = repository.join(TRASH_PARENT);
    let trash: PathBuf = parent.join(item.oid.as_str());
    match std::fs::symlink_metadata(&trash) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Verdict::Failed(format!("{} is not a directory, so it is left alone", trash.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Verdict::Done,
        Err(error) => return Verdict::Failed(format!("{} cannot be examined ({error})", trash.display())),
    }
    if !repository.join(".git").is_file() {
        return Verdict::Failed(format!("{} is no longer a linked worktree, so its trash is left alone", repository.display()));
    }
    if let Err(error) = remove_tree(trash.as_path()) {
        return Verdict::Failed(format!("{} cannot be emptied ({error})", trash.display()));
    }
    // The parent goes when nothing else is in it; a refusal only means another trash is still pending.
    let _ignored: std::io::Result<()> = std::fs::remove_dir(&parent);
    return Verdict::Done;
}

/// Trash controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_tests.rs"]
mod tests;
