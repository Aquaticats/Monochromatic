//! What:
//!  Links the entries of the real common Git directory a commit's shadow reads into a shadow repository,
//!       and copies the per-worktree entries (`config.worktree`,
//! a linked worktree's sparse checkout).
//! Why:
//!  A shadow must resolve these through the real common directory:
//! `info`
//! (attributes,
//! grafts,
//! exclude,
//! refs),
//! `hooks`,
//! `lfs`,
//! `modules`,
//! `rr-cache`,
//! `shallow`,
//! `svn`,
//! and the legacy fetch shorthands `branches` and `remotes`
//! (`gitrepository-layout`,
//! Git 2.56.0:
//!  these are the entries Git shares with `$GIT_COMMON_DIR`,
//!  and the ones that decide what a hook,
//! a filter or native Git reading the shadow can see).
//!  The list is named rather than enumerated,
//! because everything else a common directory collects is either the shadow's own
//! (refs,
//! index,
//! objects,
//! conclusion state,
//! lock files)
//!  or state no commit reads:
//! the wrapper's own logs and folders,
//! other tools' files,
//! `FETCH_HEAD`,
//! `ORIG_HEAD`,
//! `description`,
//! `lost-found`,
//! and `worktrees`,
//!  which a shadow must not claim to have
//! (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
//! "The choices this leaves to the owner").
//!  A named list is what lets a finished shadow stay usable in the pool made ahead:
//! an entry that appears between the fill and the claim would otherwise make every member refuse
//! (`shadow_pool_member.rs`).
//!  On Windows the TypeScript wrapper (deleted on 2026-10-13 in `642b6d72cb`) used junctions and
//!  copies;
//!  this port uses directory symbolic links and copies,
//! because the standard library has no junction API.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await linkShadowEntries({ shadowPath, commonDir, gitDir });
//! ```

/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The entries of the repository's common Git directory a commit's shadow shares,
///  by name.
/// Why:
///  `gitrepository-layout` (Git 2.56.0) names the entries Git resolves from `$GIT_COMMON_DIR`:
///  `branches`,
/// `config`,
/// `hooks`,
/// `info`,
/// `logs`,
/// `objects`,
/// `packed-refs`,
/// `refs`,
/// `remotes`,
/// `shallow`,
/// and `worktrees`,
///  and `modules`,
/// `rr-cache` and `svn` sit beside them.
///  The shadow keeps its own `config`,
/// `logs`,
/// `objects`,
/// `packed-refs` and `refs`,
///  and a commit reads nothing of `worktrees`:
/// what is left is this list.
///  `lfs` is the folder `git-lfs` keeps its objects and logs in,
///  which a clean filter of a commit in the shadow writes into.
pub const SHARED_ENTRY_NAMES: [&str; 9] = [
    "branches",
    "hooks",
    "info",
    "lfs",
    "modules",
    "remotes",
    "rr-cache",
    "shallow",
    "svn",
];

/// What:
///  Reports whether a common-directory entry is one a shadow shares.
/// Why:
///   The link step,
/// the pool's fill and the pool's check of a member read this one answer,
///  so no two of them can disagree about what a creation makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isSharedShadowEntry(name)
/// ```
pub fn is_shared_shadow_entry(name: &str) -> bool {
    return SHARED_ENTRY_NAMES.contains(&name);
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
///  Reports whether a linked worktree keeps its own `info` directory,
///  because it holds a sparse-checkout of its own.
/// Why:
///   Such a worktree's shadows are made where the commit is and never taken from the pool,
///  and the fill must know it before it makes a member no claim can take;
/// so the question is asked in one place.
///  The common directory's `info` is then not shared whole:
/// the shadow's own `info` links the common entries and copies this worktree's pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await keepsItsOwnInfo({ commonDir, gitDir })
/// ```
pub fn keeps_its_own_info(common_dir: &Path, git_dir: &Path) -> std::io::Result<bool> {
    let per_worktree_sparse: PathBuf = git_dir.join("info").join("sparse-checkout");
    return Ok(git_dir != common_dir && regular_file_exists(per_worktree_sparse.as_path())?);
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

/// One entry the link step shares with a shadow:
/// its name,
/// the real path it links to,
/// and whether that path is a
/// directory.
/// Why:
///  The link step and the pool's check of a finished member read the same list,
/// so a member holds exactly the entries a creation would make and nothing else.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedEntry {
    /// The name under the common Git directory,
    /// and under the shadow.
    pub name: std::ffi::OsString,
    /// The real entry the shadow links to.
    pub source: PathBuf,
    /// Whether the real entry is a directory,
    /// which Windows links differently.
    pub directory: bool,
}

/// What:
///  The entries a shadow of this worktree shares,
///  and whether this worktree also keeps its own `info`.
/// Why:
///   The pool fills a member once and checks it on every claim,
/// and both need the same list the creation links.
///  Only the named entries are shared:
/// everything else a common directory holds is the shadow's own or state no commit reads,
///  so an entry that appears there between a fill and a claim
///  (`FETCH_HEAD`,
/// a log generation,
/// a worktree,
/// the wrapper's own records)
///  is no reason to refuse a member.
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
    let own_info: bool = keeps_its_own_info(common_dir, git_dir)?;
    // One listing of the common directory answers for every shared name at once, where a question per name cost up to
    //  nine of them on the creation's path and again on a claim's (`shared_entries_at`).
    // The names are then taken in the order the list has them, so the links a creation makes and the answers a claim
    //  compares are in one order whatever order the directory lists them in.
    let listed: Vec<(std::ffi::OsString, PathBuf, bool)> = listing_of_a_common_directory(common_dir)?;
    let mut entries: Vec<SharedEntry> = Vec::with_capacity(SHARED_ENTRY_NAMES.len());
    for name in SHARED_ENTRY_NAMES {
        if own_info && name == "info" {
            continue;
        }
        for (listed_name, source, directory) in &listed {
            if listed_name != std::ffi::OsStr::new(name) {
                continue;
            }
            entries.push(SharedEntry { name: listed_name.clone(), source: source.clone(), directory: *directory });
            break;
        }
    }
    return Ok((entries, own_info));
}

/// What:
///  The entries of a common directory,
///  each with its name,
///  its path and whether it is a directory.
/// Why:
///   One `read_dir` answers every name a shadow shares,
///  where a question per name cost up to one call each on a volume whose metadata calls wait.
///  A directory that cannot be read is an error the creation reports and a claim refuses on:
///  a permission the person has since changed must not quietly remove an entry a commit reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listingOfACommonDirectory = async (dir) => (await readdir(dir, { withFileTypes: true })).map((e) => [e.name, join(dir, e.name), e.isDirectory()]);
/// ```
fn listing_of_a_common_directory(common_dir: &Path) -> std::io::Result<Vec<(std::ffi::OsString, PathBuf, bool)>> {
    let entries = std::fs::read_dir(common_dir)?;
    let mut listed: Vec<(std::ffi::OsString, PathBuf, bool)> = Vec::new();
    for entry in entries.flatten() {
        let directory: bool = entry.file_type().is_ok_and(is_a_directory);
        listed.push((entry.file_name(), entry.path(), directory));
    }
    return Ok(listed);
}

/// What:
///  Links every shared entry of one real directory into one shadow directory.
/// Why:
///   The entries are the ones `shared_entries_of` lists,
/// so the two never disagree.
///  An entry that vanishes between the listing and the link is linked by no creation,
/// so it is not an error:
/// a removal of `lfs` or `rr-cache` while a commit runs must not fail the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkDirectoryEntries({ entries, shadowDirectory });
/// ```
fn link_directory_entries(entries: &[SharedEntry], shadow_directory: &Path) -> std::io::Result<()> {
    for entry in entries {
        let destination: PathBuf = shadow_directory.join(&entry.name);
        // `git init` may have created a template entry of the same name; the real one replaces it.
        match share_replacing_template(entry.source.as_path(), destination.as_path(), entry.directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// What:
///  Whether the creation copies a `config.worktree` into the shadow.
/// Why:
///   A linked worktree's own configuration reaches a shadow as a copy,
/// and a pool member must hold it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await hasWorktreeConfig(gitDir);
/// ```
pub fn has_worktree_config(git_dir: &Path) -> std::io::Result<bool> {
    return regular_file_exists(git_dir.join("config.worktree").as_path());
}

/// What:
///  Whether one entry of a real `info` directory is the per-worktree pattern the shadow copies.
/// Why:
///   A linked worktree's `info/sparse-checkout` is its own,
/// and the shadow's `info` gets the copy the creation makes instead of a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotThePerWorktreePattern = (entry) => entry.name !== 'sparse-checkout';
/// ```
fn is_not_the_per_worktree_pattern(entry: &std::fs::DirEntry) -> bool {
    return entry.file_name() != std::ffi::OsStr::new("sparse-checkout");
}

/// What:
///  Whether an entry's type,
/// when it could be read,
/// is a directory.
/// Why:
///   A type that cannot be read counts as not a directory,
///  which only Windows tells apart:
/// it copies a shared file and links a shared directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (kind) => kind.isDirectory();
/// ```
fn is_a_directory(kind: std::fs::FileType) -> bool {
    return kind.is_dir();
}

/// What:
///  One entry of a real directory as the list of shared entries holds it.
/// Why:
///   The listing of a linked worktree's shared `info` needs the same shape the named
///  entries of the common directory have,
/// so both are linked by one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sharedEntryOf = (entry) => ({ name: entry.name, source: entry.path, directory: isADirectory(entry.type) });
/// ```
fn shared_entry_of(entry: std::fs::DirEntry) -> SharedEntry {
    let directory: bool = entry.file_type().is_ok_and(is_a_directory);
    return SharedEntry { name: entry.file_name(), source: entry.path(), directory };
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
    let (entries, mixed_info): (Vec<SharedEntry>, bool) = shared_entries_of(common_dir, git_dir)?;
    link_directory_entries(entries.as_slice(), shadow)?;
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
    let shared: Vec<SharedEntry> = match std::fs::read_dir(shared_info.as_path()) {
        Ok(listed) => listed
            .flatten()
            .filter(is_not_the_per_worktree_pattern)
            .map(shared_entry_of)
            .collect(),
        // Only an absent shared `info` is tolerated: a per-worktree sparse checkout is then the whole of it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };
    link_directory_entries(shared.as_slice(), shadow_info.as_path())?;
    let per_worktree_sparse: PathBuf = git_dir.join("info").join("sparse-checkout");
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
