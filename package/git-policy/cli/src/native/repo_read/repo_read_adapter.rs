//! What:
//!  The shapes the wrapper's existing queries answer with,
//! built from an in-process `RepoLayout`.
//! Why:
//!  Each caller keeps its own result type;
//! the adapter is the one place that says how a layout becomes a
//!  `RepositoryLocation`,
//! a `LandingLocation` or a sequencer state,
//! so a caller tries the layout first and asks Git
//!  only when the layout declines.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const location = locationOf(layout); // same value `parseLocationOutput(rev-parse output)` gives
//! ```

use crate::landing_location::LandingLocation;
use crate::repo_read_layout::{RepoLayout, git_path};
use crate::repository_location::RepositoryLocation;
use crate::rule_commit_sequencer::{SequencerFacts, SequencerState, sequencer_state};
use crate::worktree_identity::WorktreeIdentity;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// What:
///  The location `git rev-parse` would report for this layout.
/// Why:
///   A Git directory equal to the common directory is the main worktree,
/// any other is a linked one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// layout.gitDir === layout.commonDir ? { kind: 'main', ...paths } : { kind: 'linked', ...paths }
/// ```
pub fn location_of(layout: &RepoLayout) -> RepositoryLocation {
    let identity: WorktreeIdentity = if layout.git_dir == layout.common_dir {
        WorktreeIdentity::MainWorktree {
            common_dir: layout.common_dir.clone(),
            git_dir: layout.git_dir.clone(),
            worktree_root: layout.worktree_root.clone(),
        }
    } else {
        WorktreeIdentity::LinkedWorktree {
            common_dir: layout.common_dir.clone(),
            git_dir: layout.git_dir.clone(),
            worktree_root: layout.worktree_root.clone(),
        }
    };
    return RepositoryLocation { identity, prefix: layout.prefix.clone() };
}

/// What:
///  The landing location (Git directory,
/// index) for this layout,
/// or `None` for a layout whose
///  per-worktree and shared names Git splits.
/// Why:
///   Only a worktree that owns its Git directory has one answer per name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { gitDir, indexPath: gitPath('index'), realIndexPath: join(gitDir, 'index') }
/// ```
pub fn landing_of(layout: &RepoLayout) -> Option<LandingLocation> {
    return Some(LandingLocation {
        git_dir: layout.git_dir.clone(),
        index_path: git_path(layout, "index")?,
        real_index_path: layout.git_dir.join("index"),
    });
}

/// What:
///  The sequencer state from the three head files of the layout's Git directory.
/// Why:
///   `MERGE_HEAD`,
/// `CHERRY_PICK_HEAD` and `REVERT_HEAD` are per-worktree names;
/// the state only needs whether each file exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sequencerState({ mergeHead: exists(gitPath('MERGE_HEAD')), cherryPickHead: ..., revertHead: ... })
/// ```
pub fn sequencer_of(layout: &RepoLayout) -> Option<SequencerState> {
    return Some(sequencer_state(SequencerFacts {
        merge_head: git_file_exists(layout, "MERGE_HEAD")?,
        cherry_pick_head: git_file_exists(layout, "CHERRY_PICK_HEAD")?,
        revert_head: git_file_exists(layout, "REVERT_HEAD")?,
    }));
}

/// Whether the per-worktree Git file `name` exists (a link counts),
/// or nothing when its path cannot be worked out.
fn git_file_exists(layout: &RepoLayout, name: &str) -> Option<bool> {
    let path = git_path(layout, name)?;
    return Some(std::fs::symlink_metadata(path).is_ok());
}

/// What:
///  The layout facts the invocation capture records,
///  read from a layout:
/// top level,
/// Git directory,
/// common directory,
///  index,
/// objects,
/// transaction registry,
///  and object format.
/// Why:
///   The capture stores paths and not a report split at line feeds.
///  A pathname may hold the line feed that separated two answers of one report,
///  and it may hold bytes that are not UTF-8,
///  so a text report cannot be taken apart into these paths again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InvocationReport = { topLevel: string; gitDir: string; commonDir: string; indexPath: string; objectDirectory: string; registryRoot: string; objectFormat: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationReport {
    /// The worktree's top level.
    pub top_level: PathBuf,
    /// The per-worktree Git directory.
    pub git_dir: PathBuf,
    /// The directory that holds the objects and the shared refs.
    pub common_dir: PathBuf,
    /// The index Git would write,
    ///  honouring `GIT_INDEX_FILE`.
    pub index_path: PathBuf,
    /// The object directory the shadow's alternates name.
    pub object_directory: PathBuf,
    /// The per-worktree transaction registry.
    pub registry_root: PathBuf,
    /// The object hash algorithm name `rev-parse --show-object-format` prints.
    pub object_format: String,
}

/// What:
///  The report of one layout,
///  or nothing when the reader cannot name its per-worktree paths.
/// Why:
///   Every path is the layout's own path,
///  kept as bytes (`path_bytes.rs`),
///  so a repository whose name is not UTF-8 is read where it is;
///  only a linked worktree,
///  whose names Git splits,
///  has no in-process answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ topLevel: layout.worktreeRoot, gitDir: layout.gitDir, ... , objectFormat: layout.objectFormat.name() })
/// ```
pub fn invocation_report(layout: &RepoLayout) -> Option<InvocationReport> {
    return Some(InvocationReport {
        top_level: layout.worktree_root.clone(),
        git_dir: layout.git_dir.clone(),
        common_dir: layout.common_dir.clone(),
        index_path: git_path(layout, "index")?,
        object_directory: git_path(layout, "objects")?,
        registry_root: git_path(layout, "cli-git-transactions")?,
        object_format: layout.object_format.name().to_string(),
    });
}

/// The report's own controls stay out of the release executable.
#[cfg(test)]
#[path = "repo_read_adapter_tests.rs"]
mod tests;
