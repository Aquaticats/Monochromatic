//! What:
//!  The leave-out step of a snapshot of the main checkout:
//! set aside the nested worktrees that are ordinary directories,
//!  the other checkouts,
//! the caches and the run records that the main-checkout rule leaves out,
//! into a trash directory inside
//!  the new subvolume,
//! and write down what was made and moved;
//! and,
//! once the worktree is in place,
//! hand the trash to the
//!  commit service as a work item that empties it (`worktree_trash_dispose.rs`).
//! Why:
//!  A snapshot carries everything,
//! so the leave-out is a removal.
//! Unlinking about 495 thousand entries takes 14 s,
//! which the
//!  person should not wait for;
//! renaming 480 directories into one trash directory takes 1.6 s,
//! 1.2 s of which is the walk.
//!  The rename keeps the new worktree final at once.
//! The `.git` pointer of a nested worktree that was copied names another
//!  worktree's administrative directory,
//! so it is unlinked before anything else and before the add returns.
//! Nothing the
//!  main checkout tracks is ever moved,
//! and every move and removal names one plain entry below a held directory handle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { count, trash } = await pruneSnapshot({ stage, source }); // before the swap
//! await disposePendingTrash({ context, destination, gitDir, trash, notices }); // after it
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{ObjectId, identity_at, identity_of, make_directory_at, open_directory_at, open_directory_beneath, open_root, plain_components, rename_at};
use crate::fs_kind::{is_stand_in, is_subvolume_root};
use crate::random_id::random_uuid;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::worktree_clean::stored_stats;
use crate::worktree_path_failure_message::{path_failure, subject_failure};
use crate::worktree_copy_prune::Prune;
use crate::worktree_snapshot_source::SnapshotSource;
pub use crate::worktree_trash_dispose::{PendingTrash, TRASH_REFERENCE, delete_trash, dispose_pending_trash, empty_trash, name_unaccounted_trash};
pub use crate::worktree_trash_manifest::{TrashManifest, entry_name, is_trash_name};
use crate::worktree_trash_remove::remove_tree_beneath;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[path = "worktree_trash_set_aside_entries.rs"]
mod set_aside_entries;

use set_aside_entries::{inherited_trash_entries, names_the_container, remove_containers_beneath};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrash";

/// The directory,
/// relative to a worktree root,
/// inside which each snapshot's trash directory is made.
/// It sits below `.claude/`,
/// which the repository ignores,
/// so a trash that has not been emptied yet is no untracked file.
pub const TRASH_PARENT: &str = ".claude/cli-git-trash";

/// The line a snapshot of the main checkout adds to what Git prints,
/// unless `--quiet` was given.
/// Why:
/// a snapshot carries the whole ignored state itself,
/// so no copy summary appears;
/// this says what happened instead.
pub const MAIN_SNAPSHOT_NOTE: &str = "monochromatic-git-policy-cli: made the worktree as a snapshot of the main checkout; nested worktrees, caches, run records and the stage containers of other commands were left out.\n";

/// The mode bits that say an index entry is a directory (a tree) or a gitlink.
const DIRECTORY_MODES: [u32; 2] = [0o040_000, 0o160_000];

/// The mode bits that carry the type of an index entry.
const TYPE_MASK: u32 = 0o170_000;

/// What:
///  Whether Git ignores the place the trash would be made at in this worktree.
/// Why:
///   A trash directory Git does not ignore would list as untracked until emptied,
/// so the leave-out removes in the foreground
///  instead when the answer is no or unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit(['check-ignore', '--no-index', '--', `${TRASH_PARENT}/probe/x`]).exitCode === 0
/// ```
pub fn trash_location_is_ignored(context: &GitContext, root: &Path) -> bool {
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["check-ignore", "--no-index", "--"].iter().map(std::ffi::OsString::from).chain(std::iter::once(std::ffi::OsString::from(format!("{TRASH_PARENT}/probe/x")))).collect(),
        without_prefix: true,
        ..GitRequest::default()
    };
    return match run_git(context, &request) {
        Ok(output) => output.code == Some(0),
        Err(_not_started) => false,
    };
}

/// A name for one trash directory:
/// forty hexadecimal digits,
/// which is also the object name the work item carries.
/// One random identifier for the name of a trash directory;
/// the reason is the error.
fn fresh_uuid() -> Result<String, String> {
    return match random_uuid() {
        Ok(made) => Ok(made),
        Err(error) => Err(format!("a name for the trash could not be made: {error:?}")),
    };
}

fn new_trash_name() -> Result<String, String> {
    let first: String = fresh_uuid()?;
    let second: String = fresh_uuid()?;
    let digits: String = format!("{first}{second}").chars().filter(char::is_ascii_hexdigit).collect();
    return Ok(digits.chars().take(40).collect::<String>().to_ascii_lowercase());
}

/// What the leave-out made and moved,
/// and how many directories it set aside.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pruned {
    /// How many directories were set aside,
    /// moved into the trash or removed in place.
    pub count: usize,
    /// The trash directory with the identity of every directory moved into it;
    /// absent when nothing was moved into a trash.
    pub trash: Option<TrashManifest>,
}

/// The trash directory the leave-out made and holds open.
struct Trash {
    /// The trash directory,
    /// held open so every move names an entry of this object.
    dir: File,
    /// Its name below the trash parent.
    name: String,
    /// Its identity,
    /// taken when it was made.
    id: ObjectId,
    /// The directory that holds it,
    /// for removing it again when nothing is moved into it.
    parent: File,
}

/// Where left-out directories go:
/// into one trash directory when there is one,
/// else they are removed in place.
struct Aside {
    /// The snapshot,
    /// held open;
    /// every directory moved or removed is reached from it by plain names.
    root: File,
    /// The path of the snapshot,
    /// for messages.
    stage: PathBuf,
    /// The trash directory,
    /// absent when the leave-out removes in the foreground.
    trash: Option<Trash>,
    /// The identity of each directory moved into the trash,
    /// in the order moved.
    entries: Vec<ObjectId>,
    /// How many directories were set aside.
    count: usize,
}

impl Aside {
    /// The held directory that contains `relative`,
    /// and the entry's name in it.
    fn holder(&self, relative: &Path) -> Result<(File, OsString), String> {
        let names: Vec<OsString> = path_failure(plain_components(relative), relative, "is not a plain path inside the snapshot")?;
        let Some((name, parents)) = names.split_last() else { return Err(format!("{} names the snapshot itself", relative.display())) };
        let parent: File = path_failure(open_directory_beneath(&self.root, parents), self.stage.join(relative).as_path(), "is reached through a link or another mount in the snapshot")?;
        return Ok((parent, name.clone()));
    }

    /// Set one directory aside by renaming it into the trash,
    /// or remove it when there is no trash or when it is another command's stage
    ///  container;
    /// `relative` is its path below the snapshot.
    /// Why:
    ///   A stage container holds another worktree's ignored state,
    /// credentials included,
    /// and it is removed here,
    ///  before the snapshot is installed,
    /// because the trash stands inside the new worktree until the emptying runs and a
    ///  command that runs `git add --all` in that window stages what it holds;
    /// emptying the trash afterwards does not
    ///  take those blobs out of the index.
    /// Everything else still waits for the emptying,
    /// so the add pays only the
    ///  container's own removal.
    fn put(&mut self, relative: &Path) -> Result<(), String> {
        // A container is removed here and never renamed into the trash: the pattern is None for it either way.
        let direct_container: bool = names_the_container(relative);
        if !direct_container {
            // A directory set aside whole can hold one, because a container stands beside the destination of one add and a
            //  nested worktree is another such place; the sweep removes those before the rename, so a kill before the
            //  emptying cannot leave another command's credentials in the trash.
            let (holder, name): (File, OsString) = self.holder(relative)?;
            remove_containers_beneath(&holder, name.as_os_str(), &self.stage.join(relative))?;
        }
        let (parent, name): (File, OsString) = self.holder(relative)?;
        let shown: PathBuf = self.stage.join(relative);
        let into_trash: Option<&Trash> = match direct_container {
            true => None,
            false => self.trash.as_ref(),
        };
        match into_trash {
            Some(trash) => {
                let looked: ObjectId = path_failure(identity_at(&parent, name.as_os_str()), shown.as_path(), "cannot be examined")?;
                let to: String = entry_name(self.entries.len());
                path_failure(rename_at(&parent, name.as_os_str(), &trash.dir, OsStr::new(to.as_str()), false), shown.as_path(), "cannot be set aside")?;
                let moved: ObjectId = path_failure(identity_at(&trash.dir, OsStr::new(to.as_str())), shown.as_path(), "cannot be examined in the trash")?;
                if moved != looked {
                    return Err(format!("{} changed while it was set aside ({} instead of {})", shown.display(), moved.encode(), looked.encode()));
                }
                self.entries.push(moved);
            }
            None => {
                path_failure(remove_tree_beneath(shown.parent().unwrap_or(self.stage.as_path()), &parent, name.as_os_str(), None), shown.as_path(), "cannot be removed")?;
            }
        }
        self.count += 1;
        return Ok(());
    }
}

/// Make the trash directory below the trash parent in the snapshot,
/// one plain name at a time;
/// the reason is the error.
fn make_trash(root: &File) -> Result<Trash, String> {
    let names: Vec<OsString> = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(format!("the trash parent is not a plain path ({error})")),
    };
    let mut parent: File = subject_failure(root.try_clone(), &"the snapshot", "cannot be held")?;
    for name in &names {
        subject_failure(make_directory_at(&parent, name.as_os_str(), 0o755), &name.to_string_lossy(), "cannot be made in the snapshot")?;
        parent = subject_failure(open_directory_at(&parent, name.as_os_str()), &name.to_string_lossy(), "is not a plain directory in the snapshot")?;
    }
    let name: String = new_trash_name()?;
    let trash_subject: String = format!("the trash {name}");
    if !subject_failure(make_directory_at(&parent, OsStr::new(name.as_str()), 0o700), &trash_subject, "cannot be made")? {
        return Err(format!("the trash {name} already exists"));
    }
    let dir: File = subject_failure(open_directory_at(&parent, OsStr::new(name.as_str())), &trash_subject, "cannot be opened")?;
    let id: ObjectId = subject_failure(identity_of(&dir), &trash_subject, "cannot be examined")?;
    return Ok(Trash { dir, name, id, parent });
}

/// What:
///  The paths of the snapshot that are never left out,
/// as repository paths:
/// every directory above a tracked file,
/// and every
///  tracked entry that is itself a directory (a gitlink).
/// Why:
///   Whatever the name of such a path,
/// it holds or is tracked content.
/// A gitlink is a directory in the snapshot with no
///  tracked file below it,
/// so only its own path protects it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keep = trackedPaths(source); // has 'src', 'src/.cache' and the gitlink 'vendor/lib'
/// ```
fn tracked_paths(source: &SnapshotSource) -> Result<HashSet<Vec<u8>>, String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the index entries have no plain layout"));
    };
    let mut directories: HashSet<Vec<u8>> = HashSet::new();
    for entry in entries {
        if DIRECTORY_MODES.contains(&(entry.mode & TYPE_MASK)) {
            directories.insert(entry.path.clone());
        }
        let mut end: usize = entry.path.len();
        while let Some(at) = entry.path[..end].iter().rposition(is_slash_byte) {
            if !directories.insert(entry.path[..at].to_vec()) {
                break;
            }
            end = at;
        }
    }
    return Ok(directories);
}

/// A tracked path at or below `directory`,
/// to name in a refusal.
/// Whether a byte is a slash.
fn is_slash_byte(byte: &u8) -> bool {
    return *byte == b'/';
}

/// Whether the path is a directory itself,
/// not a link to one;
/// a path that cannot be examined is not.
fn is_directory_itself(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_unreadable) => false,
    };
}

fn tracked_below(source: &SnapshotSource, directory: &[u8]) -> Option<String> {
    let entries = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length)?;
    for entry in entries {
        if entry.path == directory || (entry.path.starts_with(directory) && entry.path.get(directory.len()) == Some(&b'/')) {
            return Some(String::from_utf8_lossy(entry.path.as_slice()).into_owned());
        }
    }
    return None;
}

/// What:
///  Set aside,
/// inside a fresh snapshot of the main checkout,
/// everything the main-checkout rule leaves out;
/// return how many
///  directories were set aside and what was made and moved.
/// Why:
///   The order matters:
/// what the index tracks is worked out first,
/// and a registered nested worktree that holds or is tracked
///  content refuses the whole leave-out before anything is touched;
/// then the `.git` pointer of every copied nested worktree
///  and of every other checkout goes,
/// because a Git command run inside such a copy would read and write another worktree's
///  administrative directory;
/// then the directories move into the trash.
/// A directory above a tracked file,
/// or a tracked
///  gitlink,
/// is never moved,
/// whatever its name.
/// Names are kept as bytes,
/// so a name that is not text is never taken for another.
/// A linked worktree has no cache or run-record rule of its own,
/// and it hands over everything it ignores,
/// so only another
///  command's stage container is left out,
/// because that container holds another worktree's ignored state (credentials
///  included) and no rule of any source makes it state of this one.
/// A btrfs stand-in holds nothing and btrfs refuses to rename it,
/// so the walk leaves it where it is and the
///  snapshot's stand-in sweep removes it as an empty directory (`worktree_snapshot_placeholders.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { count, trash } = await pruneSnapshot({ stage, source }); // throws the reason; the caller destroys the stage
/// ```
pub fn prune_snapshot(stage: &Path, source: &SnapshotSource) -> Result<Pruned, String> {
    // A linked or bare source hands over everything it ignores, so the main checkout's cache and run-record rule is not its own;
    //  the stage-container exclusion is, because such a container is never state of any source.
    let prune: Prune = if source.git_dir == source.common_dir { Prune::main_checkout() } else { Prune::Never };
    let tracked: HashSet<Vec<u8>> = tracked_paths(source)?;
    let mut nested_copies: Vec<PathBuf> = Vec::new();
    for nested in &source.nested {
        let Ok(relative) = nested.strip_prefix(source.root.as_path()) else { continue };
        if is_subvolume_root(nested.as_path()).unwrap_or(false) {
            continue;
        }
        if !is_directory_itself(stage.join(relative).as_path()) {
            continue;
        }
        if tracked.contains(relative.as_os_str().as_bytes()) {
            let inside: String = match tracked_below(source, relative.as_os_str().as_bytes()) {
                Some(found) => found,
                None => relative.display().to_string(),
            };
            return Err(format!("{} is a registered worktree, but the main checkout tracks {inside} in it, so the snapshot cannot leave it out", relative.display()));
        }
        nested_copies.push(relative.to_path_buf());
    }
    let root: File = subject_failure(open_root(stage), &format!("the snapshot {}", stage.display()), "cannot be opened")?;
    let trash: Option<Trash> = if source.trash_ignored {
        match make_trash(&root) {
            Ok(made) => Some(made),
            Err(reason) => {
                debug(TAG, format!("{reason}; the left-out directories are removed in place instead").as_str());
                None
            }
        }
    } else {
        None
    };
    let mut aside: Aside = Aside { root, stage: stage.to_path_buf(), trash, entries: Vec::new(), count: 0 };
    // A trash an earlier add left is not this add's, and nothing would ever empty it once the snapshot became a worktree: the
    //  disposal is handed only this add's manifest. Its directories go into this add's trash, so the emptying that runs before
    //  this command returns covers them and no snapshot carries a trash folder forward.
    for inherited in inherited_trash_entries(stage, &aside)? {
        if tracked.contains(inherited.as_os_str().as_bytes()) {
            continue;
        }
        aside.put(inherited.as_path())?;
    }
    for relative in &nested_copies {
        unlink_pointer(stage.join(relative).as_path())?;
        aside.put(relative.as_path())?;
    }
    let mut pending: Vec<(PathBuf, Vec<u8>)> = vec![(stage.to_path_buf(), Vec::new())];
    while let Some((directory, relative)) = pending.pop() {
        let mut children: Vec<(OsString, bool)> = Vec::new();
        for listed in path_failure(std::fs::read_dir(&directory), directory.as_path(), "cannot be listed")? {
            let entry: std::fs::DirEntry = path_failure(listed, directory.as_path(), "cannot be listed")?;
            let is_directory: bool = match entry.file_type() {
                Ok(kind) => kind.is_dir(),
                Err(_unreadable) => false,
            };
            children.push((entry.file_name(), is_directory));
        }
        // The rules compare names with ASCII names only, so the text of a name that is not UTF-8 (with the replacement
        //  character in it) can never equal one; the bytes alone decide every move and every protection.
        let mut names: Vec<String> = Vec::new();
        for (name, _is_directory) in &children {
            names.push(name.to_string_lossy().into_owned());
        }
        // A directory that is another checkout is set aside whole, its pointer first.
        if !relative.is_empty() && prune.skips_checkout(names.as_slice()) && !tracked.contains(relative.as_slice()) {
            unlink_pointer(directory.as_path())?;
            aside.put(Path::new(OsStr::from_bytes(relative.as_slice())))?;
            continue;
        }
        for (name, is_directory) in children {
            if !is_directory {
                continue;
            }
            let child_relative: Vec<u8> = if relative.is_empty() { name.as_bytes().to_vec() } else { [relative.as_slice(), b"/", name.as_bytes()].concat() };
            if child_relative == b".git" || child_relative == TRASH_PARENT.as_bytes() {
                continue;
            }
            // A stand-in of a nested subvolume the snapshot did not carry holds nothing and btrfs refuses to rename it, so it
            //  stays for the stand-in sweep, which removes it as an empty directory in the two places that hold worktrees.
            if is_stand_in(directory.join(&name).as_path()) {
                continue;
            }
            if !tracked.contains(child_relative.as_slice()) && prune.skips_path(String::from_utf8_lossy(child_relative.as_slice()).as_ref()) {
                aside.put(Path::new(OsStr::from_bytes(child_relative.as_slice())))?;
            } else {
                pending.push((directory.join(&name), child_relative));
            }
        }
    }
    let count: usize = aside.count;
    let manifest: Option<TrashManifest> = match aside.trash {
        Some(made) if count > 0 => Some(TrashManifest { name: made.name, trash: made.id, entries: aside.entries }),
        Some(empty) => {
            // Nothing was moved, so the trash made for it goes again; a refusal only leaves an empty directory.
            if let Err(error) = crate::fs_beneath::remove_at(&empty.parent, OsStr::new(empty.name.as_str()), true) {
                debug(TAG, format!("the empty trash {} could not be removed ({error})", empty.name).as_str());
            }
            None
        }
        None => None,
    };
    debug(TAG, format!("{count} directories set aside in {} ({})", stage.display(), if manifest.is_some() { "into the trash" } else { "removed in place" }).as_str());
    return Ok(Pruned { count, trash: manifest });
}

/// Unlink the `.git` file of a copied directory,
/// when it is a file;
/// a `.git` directory is a repository of its own and stays.
fn unlink_pointer(directory: &Path) -> Result<(), String> {
    let pointer: PathBuf = directory.join(".git");
    match std::fs::symlink_metadata(&pointer) {
        Ok(metadata) if metadata.is_file() => return path_failure(std::fs::remove_file(&pointer), pointer.as_path(), "cannot be unlinked"),
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} cannot be examined ({error})", pointer.display())),
    }
}

/// Trash controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_tests.rs"]
mod tests;
