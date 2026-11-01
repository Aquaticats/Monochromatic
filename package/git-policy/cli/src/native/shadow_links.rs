//! What:
//!  Links every non-private entry of the real common Git directory into a shadow repository,
//!       and copies the per-worktree entries (`config.worktree`, a linked worktree's sparse checkout).
//! Why:
//!  Shared entries such as `hooks`, `info`, `rr-cache`, `lfs` and `modules` must be the real ones,
//!  so hooks and native Git see them;
//!  transaction-private entries (refs, index, objects, conclusion state, lock files) stay the shadow's own
//!  (`src/shadow-repository/shadow-links.ts`).
//!  On Windows the incumbent uses junctions and copies;
//!  this port uses directory symbolic links and copies, because the standard library has no junction API.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await linkShadowEntries({ shadowPath, commonDir, gitDir });
//! ```

/// The conclusion state names a shadow keeps privately.
use super::conclusion_cleanup::{CONCLUSION_STATE_FILES, SEQUENCER_DIRECTORY};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Real common-directory entries the shadow never links:
///  its ref store, index, config, object store, message file, copied conclusion state,
///  and cli-git's own state, which contains the shadow itself.
pub const PRIVATE_SHADOW_ENTRIES: [&str; 13] = [
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
    "cli-git",
    "cli-git-transactions",
    "cli-git-transaction",
];

/// What:
///  Reports whether an entry must stay private to the shadow.
/// Why:
///   A shadow Git that locks the same name as the real owner would collide with it,
///  so lock files are never linked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isPrivateShadowEntry(name)
/// ```
pub fn is_private_shadow_entry(name: &str) -> bool {
    return PRIVATE_SHADOW_ENTRIES.contains(&name)
        || CONCLUSION_STATE_FILES.contains(&name)
        || name == SEQUENCER_DIRECTORY
        || name.ends_with(".lock");
}

/// What:
///  Removes whatever sits at a shadow entry name, without following a link.
/// Why:
///   A link in the shadow is removed itself, never its target in the real repository.
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
///  Windows links directories and copies files, since a file link needs a privilege.
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
///  Links every shared entry of one real directory into one shadow directory.
/// Why:
///   Entries are typed without following links,
///  so a link in the real directory is shared as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkDirectoryEntries({ sourceDirectory, shadowDirectory, skip });
/// ```
fn link_directory_entries(
    source_directory: &Path,
    shadow_directory: &Path,
    skip: impl Fn(&str) -> bool,
) -> std::io::Result<()> {
    for entry_result in std::fs::read_dir(source_directory)? {
        let entry: std::fs::DirEntry = entry_result?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if skip(name.as_str()) {
            continue;
        }
        let destination: PathBuf = shadow_directory.join(&name);
        // `git init` may have created a template entry of the same name; the real one replaces it.
        remove_entry(destination.as_path())?;
        share_entry(
            entry.path().as_path(),
            destination.as_path(),
            entry.file_type()?.is_dir(),
        )?;
    }
    return Ok(());
}

/// What:
///  Reports whether a regular per-worktree file exists.
/// Why:
///   A missing file is an answer, every other failure is an error.
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
    link_directory_entries(common_dir, shadow, |name| {
        return is_private_shadow_entry(name) || (mixed_info && name == "info");
    })?;
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
    match link_directory_entries(shared_info.as_path(), shadow_info.as_path(), |name| {
        return name == "sparse-checkout";
    }) {
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

/// Windows has no mode bits for this directory; the incumbent's ACL work is not ported.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_directory(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// Shadow link controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_links_tests.rs"]
mod tests;
