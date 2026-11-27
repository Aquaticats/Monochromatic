//! What:
//!  Which roots and paths a stage leaves out:
//!  the registered worktrees nested strictly under the source,
//!  the stage container itself,
//!  and any path through a private stage or a snapshot the wrapper made ahead.
//! Why:
//!  A nested worktree holds another checkout's state and a private stage holds another copy's,
//! so neither is ever staged,
//!  and a snapshot the ignored state of one new worktree carried would be a second copy of the source inside a worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const exclusions = sourceExclusions({ sourceRoot, registeredRoots, stageContainer });
//! if (namesPrivateStage(repositoryPath) || liesWithinAny(path, exclusions)) skip(path);
//! ```

use crate::worktree_copy_selection::{
    STAGE_PREFIX,
    is_separator_byte,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The registered roots strictly nested under the source,
/// and the stage container,
/// as exclusions.
/// Why:
///   A nested worktree or the stage itself must never be walked as ignored state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sourceExclusions({ sourceRoot, registeredRoots, stageContainer })
/// ```
pub(super) fn source_exclusions(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
) -> Vec<PathBuf> {
    let mut exclusions: Vec<PathBuf> = registered_roots.to_vec();
    exclusions.push(stage_container.to_path_buf());
    return nested_under(source_root, exclusions.as_slice());
}

/// What:
///  The roots strictly inside the source root,
/// never the source root itself.
/// Why:
///   A registered worktree nested in the source is another checkout's state,
/// so the source's own walk must skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.filter((root) => root !== sourceRoot && isInside(sourceRoot, root))
/// ```
pub(super) fn nested_under(
    source_root: &Path,
    roots: &[PathBuf],
) -> Vec<PathBuf> {
    let mut nested: Vec<PathBuf> = Vec::new();
    for root in roots {
        if root.as_path() != source_root && root.starts_with(source_root) {
            nested.push(root.clone());
        }
    }
    return nested;
}

/// What:
///  Whether a path component begins the name of a private stage container or of a snapshot the wrapper made ahead.
/// Why:
///   A path that goes through one is part of another copy's stage,
/// or a whole copy of the source that no worktree owns yet,
/// and is
///  never staged:
/// a snapshot ahead that the ignored state of one new worktree carried would be a second copy of the
///  source inside a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// component.startsWith(STAGE_PREFIX) || component.startsWith(AHEAD_PREFIX)
/// ```
pub(super) fn starts_private_stage(component: &[u8]) -> bool {
    return component.starts_with(STAGE_PREFIX.as_bytes())
        || component.starts_with(crate::worktree_snapshot_ahead::AHEAD_PREFIX.as_bytes());
}

/// What:
///  Whether a path is at or below any of the roots.
/// Why:
///   Registered worktrees and the stage itself inside the source are never staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.some((root) => path.startsWith(root))
/// ```
pub(super) fn lies_within_any(
    path: &Path,
    roots: &[PathBuf],
) -> bool {
    for root in roots {
        if path.starts_with(root.as_path()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a repository path names a private stage at any component.
/// Why:
///   A private stage is never selected,
/// even when Git lists it as ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryPath.split('/').some((component) => component.startsWith(STAGE_PREFIX))
/// ```
pub(super) fn names_private_stage(repository_path: &[u8]) -> bool {
    return repository_path
        .split(is_separator_byte)
        .any(starts_private_stage);
}
