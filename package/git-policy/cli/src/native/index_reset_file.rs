//! What:
//!  The landing's index update without a Git process: read the private index file, reset the commit's paths to the commit
//!  as `index_reset.rs` does, write the file back, and say whether it did.
//! Why:
//!  The follow-up `git reset <commit> --pathspec-from-file=- --pathspec-file-nul` took a minute under the landing's index lock
//!  on a large checkout. Everything it needed is in the repository files the wrapper already reads in process: the index, the
//!  commit's trees and the configuration. Any index form, configuration or hook the port does not reproduce returns `None`
//!  and the caller runs Git's own command (`doc/planning/cli-git-main-commit-within-native-plus-five.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done: boolean = resetIndexFile({ git, repository, commit, paths, indexFile }); // false: run `git reset --no-refresh`
//! ```

use super::diagnostic_log::debug;
use super::index_full::{FullIndex, parse_full_index};
use super::index_full_write::write_full_index;
use super::index_reset::reset_paths;
use super::index_reset_publish::publish;
use super::service_index_lock::IndexLock;
use super::index_reset_tree::{changed_paths, root_tree, tree_entries_at};
use super::index_settings::{IndexSettings, read_settings};
use super::repo_read_config_list::list_config;
use super::repo_read_config_scan::ScopeEnvironment;
use super::repo_read_hook::{enabled, guarded};
use super::repo_read_layout::{ObjectFormat, RepoLayout, read_layout_with};
use super::repo_read_object::name_length;
use super::repo_read_odb::ObjectDatabase;
use super::repo_read_tree::FlatEntry;
use super::service_land_follow::{IndexPaths, PrivateCopy};
use super::service_protocol::is_zero_object;
use super::subvolume_sha::{sha1, sha256};
use super::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The most bytes of index file this reads: the bound the in-process query layer already puts on the same file (`repo_read_query.rs`, `Context::index`), not a new measurement.
const INDEX_LIMIT: u64 = 1 << 30;

/// The hook Git runs after it writes an index.
const POST_INDEX_HOOK: &str = "post-index-change";

/// The checksum of the format's object hash over `bytes`.
fn checksum(format: ObjectFormat, bytes: &[u8]) -> Vec<u8> {
    return if name_length(format) == 20 { sha1(bytes).to_vec() } else { sha256(bytes).to_vec() };
}

/// The names (top-relative, as bytes) the update resets.
fn names_of(database: &ObjectDatabase, layout: &RepoLayout, new_tree: &[u8], paths: IndexPaths<'_>) -> Option<Vec<Vec<u8>>> {
    return match paths {
        IndexPaths::Listed(listed) => Some(listed.iter().map(|path| { return path.as_bytes().to_vec(); }).collect()),
        IndexPaths::Named(named) => Some(named.to_vec()),
        IndexPaths::Derived { old } => {
            let old_tree: Option<Vec<u8>> = if is_zero_object(old) { None } else { Some(root_tree(database, old, layout.object_format)?) };
            changed_paths(database, old_tree.as_deref(), new_tree, layout.object_format)
        }
    };
}

/// Everything the update reads besides the index file: the layout, the settings and the object database.
struct Surroundings {
    /// Where the repository is.
    layout: RepoLayout,
    /// What the configuration makes of the extensions.
    settings: IndexSettings,
    /// The objects.
    database: ObjectDatabase,
}

/// Read the layout, the configuration and the objects, or decline.
fn surroundings(git: &GitContext, repository: &Path) -> Option<Surroundings> {
    let added: Vec<(OsString, OsString)> = super::git_system_config::with_real_git(git.overlay.as_slice(), git.real_git.as_path());
    if !enabled(added.as_slice()) {
        return None;
    }
    let mut effective: Vec<(OsString, OsString)> = std::env::vars_os().filter(|(name, _)| { return !added.iter().any(|(own, _)| { return own == name; }); }).collect();
    effective.extend(added);
    effective.retain(|(name, _)| { return name != "GIT_INDEX_FILE" && name != "GIT_OBJECT_DIRECTORY"; });
    // Git reads this variable to turn a monitor on; its effect depends on a daemon this update never asks.
    if effective.iter().any(|(name, value)| { return name == "GIT_TEST_FSMONITOR" && !value.is_empty(); }) {
        return None;
    }
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(effective.as_slice());
    let layout: RepoLayout = read_layout_with(repository, effective.as_slice(), &scope)?;
    let listed = list_config(layout.common_dir.as_path(), layout.git_dir.as_path(), &scope, effective.as_slice())?;
    let settings: IndexSettings = read_settings(listed.as_slice())?;
    if layout.common_dir.join("hooks").join(POST_INDEX_HOOK).symlink_metadata().is_ok() {
        return None;
    }
    let database: ObjectDatabase = ObjectDatabase::open(layout.common_dir.as_path(), None, layout.object_format)?;
    return Some(Surroundings { layout, settings, database });
}

/// The index of the file, its modification time in whole seconds, or `None` when the checksum or the layout is not plain.
fn read_index(file: &Path, format: ObjectFormat) -> Option<(FullIndex, u32)> {
    let bytes: Vec<u8> = super::private_storage::read_regular_file(file, INDEX_LIMIT).ok()?;
    let hash_length: usize = name_length(format);
    let (body, stored) = bytes.split_at(bytes.len().checked_sub(hash_length)?);
    // Git refuses a file whose checksum is wrong; an all-zero checksum is what a skipped hash looks like.
    if stored != checksum(format, body).as_slice() && stored.iter().any(|byte| { return *byte != 0; }) {
        return None;
    }
    let modified: std::time::SystemTime = std::fs::metadata(file).ok()?.modified().ok()?;
    let seconds: u64 = modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    return Some((parse_full_index(bytes.as_slice(), hash_length)?, seconds as u32));
}

/// The update, with every decline as `None`.
fn reset_file(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, built: &Path) -> Option<()> {
    let around: Surroundings = surroundings(git, repository)?;
    let (mut index, timestamp) = read_index(built, around.layout.object_format)?;
    let new_tree: Vec<u8> = root_tree(&around.database, new, around.layout.object_format)?;
    let names: Vec<Vec<u8>> = names_of(&around.database, &around.layout, new_tree.as_slice(), paths)?;
    // An empty list resets nothing; `reset` with no path would reset the whole index, so the caller never asks it.
    if names.is_empty() {
        return Some(());
    }
    let tree: Vec<FlatEntry> = tree_entries_at(&around.database, new_tree.as_slice(), names.as_slice(), around.layout.object_format)?;
    reset_paths(&mut index, names.as_slice(), tree.as_slice(), around.settings, timestamp)?;
    let format: ObjectFormat = around.layout.object_format;
    let bytes: Vec<u8> = write_full_index(&index, &|data: &[u8]| { return checksum(format, data); })?;
    return publish(built, bytes.as_slice(), around.settings.shared_repository).ok();
}

/// What:
///  Reset the index entries of `paths` in the landing's private copy to the commit `new` without starting Git; whether it did.
/// Why:
///   The private index is the copy the landing made while it holds the live index lock: the function takes a borrow of that
///  lock, so only the holder can call it, and the copy, a file the landing created exclusively, so it works on no other file. A `false` leaves the file as it was (the new file replaces it only by rename
///  after it is complete), so the caller can run `git reset --no-refresh` on the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetIndexFile({ git, repository, commit, paths, indexFile }): boolean
/// ```
pub fn reset_index_file(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, lock: &IndexLock, copy: &PrivateCopy) -> bool {
    // The copy is installed by a rename onto the live index, so it must be beside it.
    if copy.directory().parent() != lock.index().parent() {
        return false;
    }
    let built: PathBuf = copy.path().to_path_buf();
    let done: bool = guarded(|| { return reset_file(git, repository, new, paths, built.as_path()); }).is_some();
    // `publish` flushes the new bytes before it renames them onto the copy, and nothing writes to the copy after it returns.
    copy.set_flushed(done);
    debug("indexResetFile", format!("in-process reset of {}: {}", built.display(), if done { "done" } else { "declined, Git answers" }).as_str());
    return done;
}

/// Fixtures for the differential controls; never part of the release executable.
#[cfg(test)]
#[path = "index_reset_support.rs"]
pub(crate) mod support;

/// Differential controls against `git reset`; never part of the release executable.
#[cfg(test)]
#[path = "index_reset_file_tests.rs"]
mod tests;

/// Controls for the forms the update leaves to Git; never part of the release executable.
#[cfg(test)]
#[path = "index_reset_fallback_tests.rs"]
mod fallback_tests;
