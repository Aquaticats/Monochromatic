//! What:
//!  Links every non-private entry of the real common Git directory into a shadow repository,
//!       and copies the per-worktree entries (`config.worktree`,
//! a linked worktree's sparse checkout).
//! Why:
//!  Shared entries such as `hooks`,
//! `info`,
//! `rr-cache`,
//! `lfs` and `modules` must be the real ones,
//!  so hooks and native Git see them;
//!  transaction-private entries (refs,
//! index,
//! objects,
//! conclusion state,
//! lock files) and the repository scratch root stay the shadow's own.
//!  On Windows the TypeScript wrapper (deleted on 2026-10-13 in `642b6d72cb`) used junctions and
//!  copies;
//!  this port uses directory symbolic links and copies,
//! because the standard library has no junction API.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await linkShadowEntries({ shadowPath, commonDir, gitDir });
//! ```

/// The conclusion state names a shadow keeps privately.
use crate::conclusion_cleanup::{CONCLUSION_STATE_FILES, SEQUENCER_DIRECTORY};
/// The name prefix of every private copy of an index the wrapper makes beside the real one.
use crate::candidate_private_index::PRIVATE_INDEX_PREFIX;
/// The names of the repository scratch root,
/// which holds the shadow itself.
use crate::wrapper_names_old_and_new::REPOSITORY_SCRATCH;
/// The infix of the file one of the wrapper's commands stages a lock in before it becomes the lock,
/// so the lock's name is worked out from the name Git's lock would have.
use crate::service_index_staging::STAGED_INFIX;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Real common-directory entries the shadow never links:
///  its ref store,
/// index,
/// config,
/// object store,
/// message file,
/// copied conclusion state,
///  and the program's own state under its current name and under the name an earlier release gave it,
/// which contains the shadow itself.
pub const PRIVATE_SHADOW_ENTRIES: [&str; 14] = [
    "HEAD",
    "refs",
    "packed-refs",
    "reftable",
    "logs",
    "index",
    "config",
    "config.worktree",
    "objects",
    "COMMIT_EDITMSG",
    "monochromatic-git-policy-cli",
    "cli-git",
    "cli-git-transactions",
    "cli-git-transaction",
];

/// What:
///  Reports whether an entry must stay private to the shadow.
/// Why:
///   A shadow Git that locks the same name as the real owner would collide with it,
///  so lock files are never linked.
///  The wrapper's own private copy of an index is private for a second reason:
/// its directory sits beside the real index,
///  a common directory collects one for every command that died before it removed its own,
///  so the number of names to link grows with no limit the person chose,
/// and no hook resolves one.
///  The repository scratch root is private because it holds the shadow itself:
/// a link to it points the shadow at its own directory,
///  and every name inside it that a shadow links costs one directory-entry creation
///  on a volume where those calls wait.
///  A file a command stages a lock in is private for the same reason a lock is:
/// it is named after the lock (`index.lock.cli-git-marker-<pid>.<namespace>-<batch>-<token>`),
/// it belongs to one command,
/// and every command of a repository leaves a new one while it runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isPrivateShadowEntry(name)
/// ```
pub fn is_private_shadow_entry(name: &str) -> bool {
    return PRIVATE_SHADOW_ENTRIES.contains(&name)
        || name == REPOSITORY_SCRATCH.current
        || name == REPOSITORY_SCRATCH.earlier
        || CONCLUSION_STATE_FILES.contains(&name)
        || name == SEQUENCER_DIRECTORY
        || name.starts_with(PRIVATE_INDEX_PREFIX)
        || name.contains(STAGED_INFIX)
        || name.ends_with(".lock");
}

/// What:
///  Removes whatever sits at a shadow entry name,
/// without following a link.
/// Why:
///   A link in the shadow is removed itself,
/// never its target in the real repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(destination, { recursive: true, force: true });
/// ```
fn remove_entry(path: &Path) -> std::io::Result<()> {
    return match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    };
}

/// What:
///  Shares one real entry with the shadow.
/// Why:
///   Unix links every entry;
///  Windows links directories and copies files,
/// since a file link needs a privilege.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(source, destination);
/// ```
fn share_entry(source: &Path, destination: &Path, directory: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let _ = directory;
        return std::os::unix::fs::symlink(source, destination);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(windows)]
    {
        if directory {
            return std::os::windows::fs::symlink_dir(source, destination);
        }
        return std::fs::copy(source, destination).map(|_| ());
    }
}

/// What:
///  Shares one real entry with the shadow,
/// replacing whatever the shadow already holds at that name.
/// Why:
///   A fresh shadow holds nothing at a shared entry's name,
/// and the template entries only exist on the reftable path,
/// where Git's own `init` makes them;
/// so the common case is one call,
///  and a name that is taken is replaced only when the link is refused.
///  Looking at the name first cost one call per entry on every commit,
///  for an answer that is almost always "nothing there".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(source, destination).catch((error) => { if (error.code !== 'EEXIST') throw error; await rm(destination, { recursive: true, force: true }); await symlink(source, destination); });
/// ```
#[cfg(unix)]
fn share_replacing_template(source: &Path, destination: &Path, directory: bool) -> std::io::Result<()> {
    return match share_entry(source, destination, directory) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            remove_entry(destination)?;
            share_entry(source, destination, directory)
        }
        other => other,
    };
}

/// What:
///  The same on Windows,
/// where a shared file is a copy rather than a link.
/// Why:
///   A copy writes over a file and refuses a directory,
/// so what sits at the name is removed first,
///  as it always was on that side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(destination, { recursive: true, force: true }); await copyOrLink(source, destination);
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn share_replacing_template(source: &Path, destination: &Path, directory: bool) -> std::io::Result<()> {
    remove_entry(destination)?;
    return share_entry(source, destination, directory);
}

/// What:
///  Which entries of a real directory are not linked into the shadow.
///  An `enum` is a closed set of named alternatives;
/// here each alternative may carry a value.
/// Why:
///   The two callers skip different entries,
/// and a rule value stands where a closure that decided by name stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SkipRule = { kind: 'PrivateEntries'; alsoInfo: boolean } | { kind: 'Named'; name: string };
/// ```
#[derive(Clone, Copy)]
enum SkipRule {
    /// The entries a shadow keeps its own (`is_private_shadow_entry`),
    /// and `info` too when `also_info` is set.
    PrivateEntries {
        /// Whether the `info` directory is kept private too,
        /// because the shadow links a single file of it.
        also_info: bool,
    },
    /// The one entry with this exact name.
    Named(&'static str),
}

impl SkipRule {
    /// Whether the entry called `name` is skipped.
    fn skips(self, name: &str) -> bool {
        return match self {
            SkipRule::PrivateEntries { also_info } => is_private_shadow_entry(name) || (also_info && name == "info"),
            SkipRule::Named(skipped) => name == skipped,
        };
    }
}

/// One entry the link step shares with a shadow: its name, the real path it links to, and whether that path is a
/// directory.
/// Why:
///  The link step and the pool's check of a finished member read the same list,
/// so a member holds exactly the entries a creation would make and nothing else.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedEntry {
    /// The name under the common Git directory, and under the shadow.
    pub name: std::ffi::OsString,
    /// The real entry the shadow links to.
    pub source: PathBuf,
    /// Whether the real entry is a directory, which Windows links differently.
    pub directory: bool,
}

/// What:
///  The entries of one real directory that a shadow shares,
/// with the path each becomes.
/// Why:
///   Entries are typed without following links,
///  so a link in the real directory is shared as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await sharedEntries(sourceDirectory, skip);
/// ```
fn shared_entries(
    source_directory: &Path,
    skip: SkipRule,
) -> std::io::Result<Vec<SharedEntry>> {
    let mut shared: Vec<SharedEntry> = Vec::new();
    for entry_result in std::fs::read_dir(source_directory)? {
        let entry: std::fs::DirEntry = entry_result?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if skip.skips(name.as_str()) {
            continue;
        }
        shared.push(SharedEntry {
            source: entry.path(),
            name: entry.file_name(),
            directory: entry.file_type()?.is_dir(),
        });
    }
    return Ok(shared);
}

/// What:
///  Links every shared entry of one real directory into one shadow directory.
/// Why:
///   The entries are the ones `shared_entries` lists, so the two never disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkDirectoryEntries({ sourceDirectory, shadowDirectory, skip });
/// ```
fn link_directory_entries(
    source_directory: &Path,
    shadow_directory: &Path,
    skip: SkipRule,
) -> std::io::Result<()> {
    for entry in shared_entries(source_directory, skip)? {
        let destination: PathBuf = shadow_directory.join(&entry.name);
        // `git init` may have created a template entry of the same name; the real one replaces it.
        share_replacing_template(entry.source.as_path(), destination.as_path(), entry.directory)?;
    }
    return Ok(());
}

/// What:
///  The entries a shadow of this worktree shares,
/// and whether this worktree also keeps its own `info`.
/// Why:
///   The pool fills a member once and checks it on every claim,
/// and both need the same list the creation links.
///  A worktree that keeps its own `info` builds its own `info` directory and copies a sparse checkout into it,
/// so its shadows are made where the commit is and never taken from a pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await sharedEntriesOf({ commonDir, gitDir }) // { entries, ownInfo }
/// ```
pub fn shared_entries_of(
    common_dir: &Path,
    git_dir: &Path,
) -> std::io::Result<(Vec<SharedEntry>, bool)> {
    let per_worktree_sparse: PathBuf = git_dir.join("info").join("sparse-checkout");
    let mixed_info: bool =
        git_dir != common_dir && regular_file_exists(per_worktree_sparse.as_path())?;
    let entries: Vec<SharedEntry> =
        shared_entries(common_dir, SkipRule::PrivateEntries { also_info: mixed_info })?;
    return Ok((entries, mixed_info));
}

/// What:
///  Whether the creation copies a `config.worktree` into the shadow.
/// Why:
///   A linked worktree's own configuration reaches a shadow as a copy, and a pool member must hold it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await hasWorktreeConfig(gitDir);
/// ```
pub fn has_worktree_config(git_dir: &Path) -> std::io::Result<bool> {
    return regular_file_exists(git_dir.join("config.worktree").as_path());
}

/// What:
///  Reports whether a regular per-worktree file exists.
/// Why:
///   A missing file is an answer,
/// every other failure is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await regularFileExists(path);
/// ```
fn regular_file_exists(path: &Path) -> std::io::Result<bool> {
    return match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    };
}

/// What:
///  Links the real common directory into the shadow and copies the per-worktree entries.
/// Why:
///   A linked worktree's `info/sparse-checkout` lives outside the common directory,
///  so its `info` directory is private and only its shared entries are linked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkShadowEntries({ shadowPath, commonDir, gitDir });
/// ```
pub fn link_shadow_entries(shadow: &Path, common_dir: &Path, git_dir: &Path) -> std::io::Result<()> {
    let per_worktree_sparse: PathBuf = git_dir.join("info").join("sparse-checkout");
    let mixed_info: bool =
        git_dir != common_dir && regular_file_exists(per_worktree_sparse.as_path())?;
    link_directory_entries(common_dir, shadow, SkipRule::PrivateEntries { also_info: mixed_info })?;
    let worktree_config: PathBuf = git_dir.join("config.worktree");
    if regular_file_exists(worktree_config.as_path())? {
        std::fs::copy(&worktree_config, shadow.join("config.worktree"))?;
    }
    if !mixed_info {
        return Ok(());
    }
    let shadow_info: PathBuf = shadow.join("info");
    remove_entry(shadow_info.as_path())?;
    std::fs::create_dir(&shadow_info)?;
    set_private_directory(shadow_info.as_path())?;
    let shared_info: PathBuf = common_dir.join("info");
    match link_directory_entries(shared_info.as_path(), shadow_info.as_path(), SkipRule::Named("sparse-checkout")) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    std::fs::copy(&per_worktree_sparse, shadow_info.join("sparse-checkout"))?;
    return Ok(());
}

/// What:
///  Gives a directory the private mode of the transaction's directories.
/// Why:
///   The shadow's private `info` holds a copy of per-worktree state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(shadowInfo, { mode: 0o700 });
/// ```
#[cfg(unix)]
fn set_private_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

/// Windows has no mode bits for this directory;
/// the incumbent's ACL work is not ported.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_directory(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// Shadow link controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_links_tests.rs"]
mod tests;
