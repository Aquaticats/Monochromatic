//! What:
//!  The landing's index update without a Git process:
//! read the private index file,
//! reset the commit's paths to the commit
//!  as `index_reset.rs` does,
//! write the file back,
//! and say whether it did.
//! Why:
//!  The follow-up `git reset <commit> --pathspec-from-file=- --pathspec-file-nul` took a minute under the landing's index lock
//!  on a large checkout.
//! Everything it needed is in the repository files the wrapper already reads in process:
//! the index,
//! the
//!  commit's trees and the configuration.
//! Any index form,
//! configuration or hook the port does not reproduce returns `None`
//!  and the caller runs Git's own command (`doc/planning/cli-git-main-commit-within-native-plus-five.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done: boolean = resetIndexFile({ git, repository, commit, paths, indexFile }); // false: run `git reset --no-refresh`
//! ```

use crate::diagnostic_log::debug;
use crate::index_byte_search::has_nonzero_byte;
use crate::index_full::{FullIndex, parse_full_index};
use crate::index_full_write::{Trailer, write_full_index};
use crate::index_stat::IndexStamp;
use crate::index_reset::reset_paths;
use crate::index_reset_worktree_stat::LandingStat;
use crate::index_reset_worktree_stat_source::WorktreeStat;
use crate::index_reset_publish::publish;
use crate::service_index_lock::IndexLock;
use crate::index_reset_tree::{changed_paths, root_tree, tree_entries_at};
use crate::index_settings::{IndexSettings, read_settings};
use crate::repo_read_config_list::list_config;
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_hook::{enabled, guarded};
use crate::repo_read_layout::{ObjectFormat, RepoLayout, read_layout_with};
use crate::repo_read_object::name_length;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::FlatEntry;
use crate::service_land_follow::{IndexPaths, PrivateCopy};
use crate::service_protocol::is_zero_object;
use crate::subvolume_sha::{sha1, sha256};
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The most bytes of index file this reads:
/// the bound the in-process query layer already puts on the same file (`repo_read_query.rs`,
/// `Context::index`),
/// not a new measurement.
const INDEX_LIMIT: u64 = 1 << 30;

/// The hook Git runs after it writes an index.
const POST_INDEX_HOOK: &str = "post-index-change";

/// The checksum of the format's object hash over `bytes`.
fn checksum(format: ObjectFormat, bytes: &[u8]) -> Vec<u8> {
    return if name_length(format) == 20 { sha1_bytes(bytes) } else { sha256_bytes(bytes) };
}

/// The SHA-1 of `bytes` as a byte vector.
fn sha1_bytes(bytes: &[u8]) -> Vec<u8> {
    return sha1(bytes).to_vec();
}

/// The SHA-256 of `bytes` as a byte vector.
fn sha256_bytes(bytes: &[u8]) -> Vec<u8> {
    return sha256(bytes).to_vec();
}

/// The names (top-relative,
/// as bytes) the update resets.
fn names_of(database: &ObjectDatabase, layout: &RepoLayout, new_tree: &[u8], paths: IndexPaths<'_>) -> Option<Vec<Vec<u8>>> {
    return match paths {
        IndexPaths::Listed(listed) => {
            let mut names: Vec<Vec<u8>> = Vec::new();
            for path in listed {
                names.push(path.as_bytes().to_vec());
            }
            Some(names)
        }
        IndexPaths::Named(named) => Some(named.to_vec()),
        IndexPaths::Derived { old } => {
            let old_tree: Option<Vec<u8>> = if is_zero_object(old) { None } else { Some(root_tree(database, old, layout.object_format)?) };
            changed_paths(database, old_tree.as_deref(), new_tree, layout.object_format)
        }
    };
}

/// Everything the update reads besides the index file:
/// the layout,
/// the settings and the object database.
struct Surroundings {
    /// Where the repository is.
    layout: RepoLayout,
    /// What the configuration makes of the extensions.
    settings: IndexSettings,
    /// The objects.
    database: ObjectDatabase,
}

/// This process's environment with `added` laid over it,
/// minus the two variables that would point Git at other files.
fn effective_environment(added: Vec<(OsString, OsString)>) -> Vec<(OsString, OsString)> {
    let mut effective: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in std::env::vars_os() {
        // A variable the run adds replaces the one of this process.
        let mut replaced: bool = false;
        for (own, _) in &added {
            if *own == name {
                replaced = true;
                break;
            }
        }
        if !replaced {
            effective.push((name, value));
        }
    }
    effective.extend(added);
    let mut kept: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in effective {
        if name != "GIT_INDEX_FILE" && name != "GIT_OBJECT_DIRECTORY" {
            kept.push((name, value));
        }
    }
    return kept;
}

/// Whether the environment sets `GIT_TEST_FSMONITOR` to something,
/// which would turn a file system monitor on.
fn turns_monitor_on(effective: &[(OsString, OsString)]) -> bool {
    for (name, value) in effective {
        if name == "GIT_TEST_FSMONITOR" && !value.is_empty() {
            return true;
        }
    }
    return false;
}

/// Read the layout,
/// the configuration and the objects,
/// or decline.
fn surroundings(git: &GitContext, repository: &Path) -> Option<Surroundings> {
    let added: Vec<(OsString, OsString)> = crate::git_system_config::with_real_git(git.overlay.as_slice(), git.real_git.as_path());
    if !enabled(added.as_slice()) {
        return None;
    }
    let effective: Vec<(OsString, OsString)> = effective_environment(added);
    // Git reads this variable to turn a monitor on; its effect depends on a daemon this update never asks.
    if turns_monitor_on(effective.as_slice()) {
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

/// The index of the file,
/// its modification time to the nanosecond,
/// or `None` when the checksum or the layout is not plain.
fn read_index(file: &Path, format: ObjectFormat) -> Option<(FullIndex, IndexStamp)> {
    let bytes: Vec<u8> = crate::private_storage::read_regular_file(file, INDEX_LIMIT).ok()?;
    let hash_length: usize = name_length(format);
    let (body, stored) = bytes.split_at(bytes.len().checked_sub(hash_length)?);
    // Git refuses a file whose checksum is wrong; an all-zero checksum is what a skipped hash looks like.
    if stored != checksum(format, body).as_slice() && has_nonzero_byte(stored) {
        return None;
    }
    let modified: std::time::SystemTime = std::fs::metadata(file).ok()?.modified().ok()?;
    let age: std::time::Duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    let stamp: IndexStamp = IndexStamp { seconds: age.as_secs() as u32, nanoseconds: age.subsec_nanos() };
    return Some((parse_full_index(bytes.as_slice(), hash_length)?, stamp));
}

/// What:
///  The directory one path is the `.gitattributes` of,
/// or nothing when it is another path.
/// Why:
///   The directories whose attributes the index can supply are named by the `.gitattributes` entries themselves,
/// at the root or below one directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributeDirectory = (path) => path === '.gitattributes' ? '' : path.endsWith('/.gitattributes') ? path.slice(0, -'/'.length - '.gitattributes'.length) : undefined;
/// ```
fn attribute_directory(path: &[u8]) -> Option<Vec<u8>> {
    if path == b".gitattributes" {
        return Some(Vec::new());
    }
    let suffix: &[u8] = b"/.gitattributes";
    return path.strip_suffix(suffix).map(<[u8]>::to_vec);
}

/// What:
///  The directories whose `.gitattributes` the index this update writes holds,
/// in path order.
/// Why:
///   Git reads the index's copy of an attributes file where the worktree's copy is missing
/// (`attr.c`,
/// measured on Git 2.56.0:
/// `git check-attr` still answers from the index after the worktree's `.gitattributes` is removed),
/// and a
///  sparse checkout leaves it outside its cone the same way,
/// so the proof must know which directories could supply one through the index.
///  The entries before the reset and the tree's entries are both taken,
/// so a path this reset installs is counted too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directories = [...index.entries, ...tree].map((entry) => entry.path === '.gitattributes' ? '' : entry.path.replace(/\/\.gitattributes$/, '')).filter(defined);
/// ```
fn indexed_attribute_directories(index: &FullIndex, tree: &[FlatEntry]) -> Vec<Vec<u8>> {
    let mut directories: Vec<Vec<u8>> = Vec::new();
    for entry in index.entries.iter() {
        if entry.stage() == 0
            && let Some(directory) = attribute_directory(entry.path.as_slice())
        {
            directories.push(directory);
        }
    }
    for entry in tree {
        if let Some(directory) = attribute_directory(entry.path.as_slice()) {
            directories.push(directory);
        }
    }
    directories.sort();
    directories.dedup();
    return directories;
}

/// The update,
/// with every decline as `None`,
/// and the names it reset.
fn reset_file(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, built: &Path) -> Option<Vec<Vec<u8>>> {
    let around: Surroundings = surroundings(git, repository)?;
    let (mut index, stamp) = read_index(built, around.layout.object_format)?;
    let new_tree: Vec<u8> = root_tree(&around.database, new, around.layout.object_format)?;
    let names: Vec<Vec<u8>> = names_of(&around.database, &around.layout, new_tree.as_slice(), paths)?;
    // An empty list resets nothing; `reset` with no path would reset the whole index, so the caller never asks it.
    if names.is_empty() {
        return Some(names);
    }
    let tree: Vec<FlatEntry> = tree_entries_at(&around.database, new_tree.as_slice(), names.as_slice(), around.layout.object_format)?;
    // What:
    //  Where the stat data of each entry the reset makes comes from:
    //  the worktree's own, for a path whose file provably holds the bytes the entry records.
    // Why:
    //  Native `git commit` leaves the worktree's stat data on the entries it writes,
    // and the all-zero value a reset writes makes the
    //   next commit hash every path the previous commit landed
    //  (`index_reset_worktree_stat.rs`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = new LandingStat({ worktreeRoot, format, stamp }); resetPaths(index, names, tree, settings, stamp, { recorded: source });
    // ```
    // What:
    //  The worktree held open and the attribute files Git would apply,
    //  which decide what stat data the reset may record.
    // Why:
    //  The proof reads the worktree below a held root,
    //  so a directory swapped for a link on the way is refused instead of read through,
    // and it refuses every path a
    //   conversion could change,
    // which it learns from the same attributes files the worktree's clean check reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const root = openRoot(layout.worktreeRoot); const environment = forContext(git);
    // const target = { root, gitDir: layout.gitDir, commonDir: layout.commonDir, environment, format };
    // const source = new LandingStat(root, target, stamp, new Cell(0));
    // ```
    let root: std::fs::File = crate::fs_beneath::open_root(around.layout.worktree_root.as_path()).ok()?;
    let environment: crate::worktree_clean_content_attribute_sources::ConversionEnvironment = crate::worktree_clean_content_attribute_sources::for_context(git);
    let target: crate::worktree_clean_tree_walk::ContentTarget<'_> = crate::worktree_clean_tree_walk::ContentTarget {
        root: around.layout.worktree_root.as_path(),
        git_dir: around.layout.git_dir.as_path(),
        common_dir: around.layout.common_dir.as_path(),
        environment: &environment,
        format: around.layout.object_format,
    };
    let hashed: std::cell::Cell<u64> = std::cell::Cell::new(0);
    let indexed: Vec<Vec<u8>> = indexed_attribute_directories(&index, tree.as_slice());
    let source: LandingStat<'_> = LandingStat::new(&root, &target, stamp, &hashed, indexed.as_slice());
    reset_paths(&mut index, names.as_slice(), tree.as_slice(), around.settings, stamp, WorktreeStat::Recorded(&source))?;
    let format: ObjectFormat = around.layout.object_format;
    let hash: fn(&[u8]) -> Vec<u8> = if name_length(format) == 20 { sha1_bytes } else { sha256_bytes };
    // The file ends as the configuration asks: Git writes zeros for a skipped hash, and the differential controls compare
    // the bytes, so this update writes what Git's own `reset` would.
    let trailer: Trailer = if around.settings.skip_hash { Trailer::Skipped { checksum_length: name_length(format) } } else { Trailer::Hashed };
    let bytes: Vec<u8> = write_full_index(&index, &hash, trailer)?;
    // A copy that could not be published is a decline, and Git's own reset then runs.
    if publish(built, bytes.as_slice(), around.settings.shared_repository).is_err() {
        return None;
    }
    return Some(names);
}

/// The update of `reset_file`,
/// as the guard calls it with the Git context,
/// the repository,
/// the commit,
/// the paths and the copy.
fn reset_file_guarded((git, repository, new, paths, built): (&GitContext, &Path, &str, IndexPaths<'_>, &Path)) -> Option<Vec<Vec<u8>>> {
    return reset_file(git, repository, new, paths, built);
}

/// What:
///  Reset the index entries of `paths` in the landing's private copy to the commit `new` without starting Git;
/// the names it reset,
/// or nothing when it declined and Git has to.
/// Why:
///   The private index is the copy the landing made while it holds the live index lock:
/// the function takes a borrow of that
///  lock,
/// so only the holder can call it,
/// and the copy,
/// a file the landing created exclusively,
/// so it works on no other file.
/// A `false` leaves the file as it was (the new file replaces it only by rename
///  after it is complete),
/// so the caller can run `git reset --no-refresh` on the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetIndexFile({ git, repository, commit, paths, indexFile }): names | undefined
/// ```
pub fn reset_index_file(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, lock: &IndexLock, copy: &PrivateCopy) -> Option<Vec<Vec<u8>>> {
    // The copy is installed by a rename onto the live index, so it must be beside it.
    if copy.directory().parent() != lock.index().parent() {
        return None;
    }
    let built: PathBuf = copy.path().to_path_buf();
    let done: Option<Vec<Vec<u8>>> = guarded((git, repository, new, paths, built.as_path()), reset_file_guarded);
    // `publish` flushes the new bytes before it renames them onto the copy, and nothing writes to the copy after it returns.
    copy.set_flushed(done.is_some());
    debug("indexResetFile", format!("in-process reset of {}: {}", built.display(), if done.is_some() { "done" } else { "declined, Git answers" }).as_str());
    return done;
}

/// Fixtures for the differential controls;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_support.rs"]
pub(crate) mod support;

/// Differential controls against `git reset`;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_file_tests.rs"]
mod tests;

/// Controls for the forms the update leaves to Git;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_fallback_tests.rs"]
mod fallback_tests;

/// Controls for the stat data the update records on the entries it writes;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_recorded_stat_tests.rs"]
mod recorded_stat_tests;
