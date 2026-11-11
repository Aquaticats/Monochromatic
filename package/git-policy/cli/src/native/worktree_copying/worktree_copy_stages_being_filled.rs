//! What:
//!  The stage containers a live command is filling that stand below a tree,
//!  read from the claims beside the journals.
//! Why:
//!  A claim records the container one command reserved and the owner filling it,
//! so a container met in a tree is that command's work while its owner runs,
//! and nobody's once it is dead.
//!  A conversion that would swap or delete a tree holding one has to know which of the two it met:
//! its phases stop while another command is still filling a container inside the worktree
//!  (`worktree_convert_phases.rs`).
//! An owner is judged by the rule the recovery sweep uses (`token_owner_is_dead`),
//! so a container this module reports as filled is one no sweep would remove either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const filled = await filledStagesBelow(commonDir, worktree, converting); // [{ container, owner }]
//! ```

use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_journal::journal_root;
use crate::worktree_copy_owner::{CopyOwner, token_owner_is_dead};
use crate::worktree_copy_stage_claim::claim_names;
use crate::worktree_copy_stage_claim_reaping::read_claim;
use std::path::{Path, PathBuf};

/// What:
///  One stage container a live command is filling.
/// Why:
///   The refusal a conversion says and the control of it both need where the container stands
/// and which process fills it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FilledStage = { container: string; owner: number | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilledStage {
    /// Where the container stands below the tree the caller asked about.
    pub container: PathBuf,
    /// The process filling it,
    /// when the claim names one.
    pub owner: Option<i64>,
}

/// What:
///  Every stage container whose claim names an owner that still runs.
/// Why:
///   The journal root holds one claim per stage a command reserved;
/// a claim whose owner is dead is an abandoned stage the recovery sweep settles with its own proof,
///  so it is not another command's work any more.
/// A claim this release cannot read is an error rather than a container ruled out,
/// because the claim is the only record of what it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function filledStages(commonDir: string): Promise<FilledStage[]>;
/// ```
fn filled_stages(common_dir: &Path) -> Result<Vec<FilledStage>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let mut stages: Vec<FilledStage> = Vec::new();
    for name in claim_names(root.as_path())? {
        let claim: PathBuf = root.join(name.as_str());
        let Some((container, owner)) = read_claim(claim.as_path())? else { continue };
        if token_owner_is_dead(owner.as_deref(), claim.as_path())? {
            continue;
        }
        let filling: Option<i64> = owner.as_deref().and_then(CopyOwner::from_token).map(|found: CopyOwner| return found.pid);
        stages.push(FilledStage { container, owner: filling });
    }
    return Ok(stages);
}

/// What:
///  The stage containers a live command is filling that stand below `standing_below`,
///  with the process filling each,
///  where the claims name them below `named_below`.
/// Why:
///   A conversion moves a worktree to its converting name with one exchange,
/// so a container below the worktree is named one way before that exchange and another after it,
/// and a claim read afterwards still uses the name from before.
/// A claim names its container as the command that made it spelled the path,
/// which may go through a link the worktree is reached by,
/// so both sides are resolved before they are compared and a spelling alone never hides a container.
/// A claim whose container is gone is nobody's:
/// its command removed the container while the claim outlived it for a moment,
/// and a folder no claim names is not one a sweep removes either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await filledStagesBelow(commonDir, worktree, converting); // [{ container: join(converting, relative), owner }]
/// ```
pub fn filled_stages_below(common_dir: &Path, named_below: &Path, standing_below: &Path) -> Result<Vec<FilledStage>, WorktreeCopyError> {
    let stages: Vec<FilledStage> = filled_stages(common_dir)?;
    // The worktree's own name is resolved once, so a claim's resolved place and it are compared the same way.
    let named: PathBuf = match std::fs::canonicalize(named_below) {
        Ok(resolved) => resolved,
        Err(_unresolved) => named_below.to_path_buf(),
    };
    let mut found: Vec<FilledStage> = Vec::new();
    for stage in stages {
        let Some(relative) = below(stage.container.as_path(), named.as_path()) else { continue };
        let here: PathBuf = standing_below.join(relative);
        if std::fs::symlink_metadata(here.as_path()).is_err() {
            continue;
        }
        found.push(FilledStage { container: here, owner: stage.owner });
    }
    return Ok(found);
}

/// What:
///  Where `path` stands below `named`,
///  resolving the longest part of the path that still exists.
/// Why:
///   A claim names its container with the path its own command spelled,
/// which may go through a link,
/// and the folder it names is gone once a conversion has exchanged the worktree with its stage,
/// so a resolution that needed the whole path would find nothing and rule the live container out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// below('/w/inside/.cli-x', '/w') // 'inside/.cli-x'
/// ```
fn below(path: &Path, named: &Path) -> Option<PathBuf> {
    let mut missing: Vec<std::ffi::OsString> = Vec::new();
    let mut probe: PathBuf = path.to_path_buf();
    let resolved: PathBuf = loop {
        if let Ok(found) = std::fs::canonicalize(probe.as_path()) {
            break found;
        }
        let name: std::ffi::OsString = probe.file_name().map(std::ffi::OsStr::to_os_string)?;
        missing.push(name);
        probe = probe.parent().map(Path::to_path_buf)?;
    };
    let mut absolute: PathBuf = resolved;
    for name in missing.iter().rev() {
        absolute.push(name.as_os_str());
    }
    return absolute.strip_prefix(named).ok().map(Path::to_path_buf);
}

/// Stage-use controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stages_being_filled_tests.rs"]
mod tests;
