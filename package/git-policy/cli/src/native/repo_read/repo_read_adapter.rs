//! What:
//!  The shapes the wrapper's existing queries answer with, built from an in-process `RepoLayout`.
//! Why:
//!  Each caller keeps its own result type; the adapter is the one place that says how a layout becomes a
//!  `RepositoryLocation`, a `LandingLocation` or a sequencer state, so a caller tries the layout first and asks Git
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

/// What:
///  The location `git rev-parse` would report for this layout.
/// Why:
///   A Git directory equal to the common directory is the main worktree, any other is a linked one.
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
///  The landing location (Git directory, index) for this layout, or `None` for a layout whose
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
///   `MERGE_HEAD`, `CHERRY_PICK_HEAD` and `REVERT_HEAD` are per-worktree names; the state only needs whether each file exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sequencerState({ mergeHead: exists(gitPath('MERGE_HEAD')), cherryPickHead: ..., revertHead: ... })
/// ```
pub fn sequencer_of(layout: &RepoLayout) -> Option<SequencerState> {
    let exists = |name: &str| -> Option<bool> {
        let path = git_path(layout, name)?;
        return Some(std::fs::symlink_metadata(path).is_ok());
    };
    return Some(sequencer_state(SequencerFacts {
        merge_head: exists("MERGE_HEAD")?,
        cherry_pick_head: exists("CHERRY_PICK_HEAD")?,
        revert_head: exists("REVERT_HEAD")?,
    }));
}

/// What:
///  The seven-line report the invocation capture asks `rev-parse` for: top level, Git directory, common directory,
///  index, objects, transaction registry and object format, one per line.
/// Why:
///   The capture parses that text by line number, so an in-process answer is the same text and the parser stays one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [toplevel, gitDir, commonDir, gitPath('index'), gitPath('objects'), gitPath('cli-git-transactions'), objectFormat].join('\n')
/// ```
pub fn invocation_report(layout: &RepoLayout) -> Option<String> {
    let lines: [String; 7] = [
        layout.worktree_root.to_str()?.to_string(),
        layout.git_dir.to_str()?.to_string(),
        layout.common_dir.to_str()?.to_string(),
        git_path(layout, "index")?.to_str()?.to_string(),
        git_path(layout, "objects")?.to_str()?.to_string(),
        git_path(layout, "cli-git-transactions")?.to_str()?.to_string(),
        layout.object_format.name().to_string(),
    ];
    return Some(lines.join("\n"));
}
