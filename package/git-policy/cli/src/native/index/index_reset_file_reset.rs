//! What:
//!  The update itself:
//!  read the private index and the commit's tree,
//!  reset the names to the commit,
//!  record the worktree's stat data where a file proves it holds the commit's
//!  bytes,
//!  and write the file back as Git would.
//! Why:
//!  Everything the follow-up `git reset --no-refresh` needed is in the
//!  repository files the wrapper already reads in process,
//!  and any index form,
//!  configuration or hook the port does not reproduce returns nothing so the
//!  caller runs Git's own command,
//!  which is why every decline below is one `?` rather than a partial write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = resetFile({ git, repository, new, paths, built, times }); // Uint8Array[] | undefined
//! ```

/// The checksums the trailer is written with,
/// and the names this update resets.
use super::index_reset_file_names_and_checksums::{
    names_of,
    sha1_bytes,
    sha256_bytes,
};
/// The index file read into entries,
/// and the attribute directories its entries name.
use super::index_reset_file_reading::{
    indexed_attribute_directories,
    read_index,
};
/// The layout,
/// settings and objects the update reads around the index file.
use super::index_reset_file_surroundings::{
    Surroundings,
    surroundings,
};
/// The index file written as Git writes it,
/// and whether its checksum is written.
use crate::index_full_write::{
    Trailer,
    write_full_index,
};
/// The in-memory reset of the entries Git's own command performs.
use crate::index_reset::reset_paths;
/// The rename that installs the written file over the live index.
use crate::index_reset_publish::publish;
/// The trees a path-limited reset reads.
use crate::index_reset_tree::{
    root_tree,
    tree_entries_at,
};
/// Where a new entry's stat data comes from,
/// and the proof that answers with the worktree's.
use crate::index_reset_worktree_stat::LandingStat;
use crate::index_reset_worktree_stat_source::WorktreeStat;
/// The object hash the file is written in.
use crate::repo_read_layout::ObjectFormat;
/// The length of one object name in the repository's hash.
use crate::repo_read_object::name_length;
/// One entry of the commit's tree.
use crate::repo_read_tree::FlatEntry;
/// The index paths a caller hands the landing.
use crate::service_land_follow::IndexPaths;
/// The Git context the worktree's attributes are read in,
/// and the paths a caller resets.
use crate::transaction_git::GitContext;
/// The path the private copy is written to.
use std::path::Path;
/// The update,
/// with every decline as `None`,
/// and the names it reset.
fn reset_file(
    git: &GitContext,
    repository: &Path,
    new: &str,
    paths: IndexPaths<'_>,
    built: &Path,
    times: Option<(std::time::SystemTime, std::time::SystemTime)>,
) -> Option<Vec<Vec<u8>>> {
    let around: Surroundings = surroundings(git, repository)?;
    let (mut index, stamp) = read_index(built, around.layout.object_format)?;
    let new_tree: Vec<u8> = root_tree(&around.database, new, around.layout.object_format)?;
    let names: Vec<Vec<u8>> =
        names_of(&around.database, &around.layout, new_tree.as_slice(), paths)?;
    // An empty list resets nothing; `reset` with no path would reset the whole index, so the caller never asks it.
    if names.is_empty() {
        return Some(names);
    }
    let tree: Vec<FlatEntry> = tree_entries_at(
        &around.database,
        new_tree.as_slice(),
        names.as_slice(),
        around.layout.object_format,
    )?;
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
    let root: std::fs::File =
        crate::fs_beneath::open_root(around.layout.worktree_root.as_path()).ok()?;
    let environment: crate::worktree_clean_content_attribute_sources::ConversionEnvironment =
        crate::worktree_clean_content_attribute_sources::for_context(git);
    let target: crate::worktree_clean_tree_walk::ContentTarget<'_> =
        crate::worktree_clean_tree_walk::ContentTarget {
            root: around.layout.worktree_root.as_path(),
            git_dir: around.layout.git_dir.as_path(),
            common_dir: around.layout.common_dir.as_path(),
            environment: &environment,
            format: around.layout.object_format,
        };
    let hashed: std::cell::Cell<u64> = std::cell::Cell::new(0);
    let indexed: Vec<Vec<u8>> = indexed_attribute_directories(&index, tree.as_slice());
    let source: LandingStat<'_> =
        LandingStat::new(&root, &target, stamp, &hashed, indexed.as_slice());
    reset_paths(
        &mut index,
        names.as_slice(),
        tree.as_slice(),
        around.settings,
        stamp,
        WorktreeStat::Recorded(&source),
    )?;
    let format: ObjectFormat = around.layout.object_format;
    let hash: fn(&[u8]) -> Vec<u8> = if name_length(format) == 20 {
        sha1_bytes
    } else {
        sha256_bytes
    };
    // The file ends as the configuration asks: Git writes zeros for a skipped hash, and the differential controls compare
    // the bytes, so this update writes what Git's own `reset` would.
    let trailer: Trailer = if around.settings.skip_hash {
        Trailer::Skipped {
            checksum_length: name_length(format),
        }
    } else {
        Trailer::Hashed
    };
    let bytes: Vec<u8> = write_full_index(&index, &hash, trailer)?;
    // A copy that could not be published is a decline, and Git's own reset then runs.
    // The published copy takes the live index's times with its bytes, in the one flush the step makes
    // (`index_reset_publish.rs`, `publish_with`).
    if publish(
        built,
        bytes.as_slice(),
        around.settings.shared_repository,
        times,
    )
    .is_err()
    {
        return None;
    }
    return Some(names);
}

/// The update of `reset_file`,
/// as the guard calls it with the Git context,
/// the repository,
/// the commit,
/// the paths,
/// the copy and its times.
pub(super) fn reset_file_guarded(
    (git, repository, new, paths, built, times): (
        &GitContext,
        &Path,
        &str,
        IndexPaths<'_>,
        &Path,
        Option<(std::time::SystemTime, std::time::SystemTime)>,
    )
) -> Option<Vec<Vec<u8>>> {
    return reset_file(git, repository, new, paths, built, times);
}
