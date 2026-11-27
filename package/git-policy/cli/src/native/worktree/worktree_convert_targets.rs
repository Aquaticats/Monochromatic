//! What:
//!  What one run of `git monochromatic-git-policy-cli convert-worktree-to-subvolume` covers,
//! and how the worktrees it covers are laid into lanes.
//! Why:
//!  A run that covers fewer worktrees than are registered says which ones it left out,
//! so no worktree is passed over without a word,
//! and the same worktree named twice is converted by one worker and found converted by the
//!  next instead of meeting its own claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { found, lanes } = findTheTargetsAndLanes(arguments, cwd);
//! ```

use crate::subvolume_command::main_checkout_root;
use crate::subvolume_index_files::{
    LinkedWorktreeScan,
    scan_linked_worktrees,
};
use crate::worktree_convert_arguments::ConvertArguments;
use std::collections::HashMap;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The worktrees a run covers,
/// and the registrations of `--all` it leaves out with why.
/// Why:
///   A run that covers fewer worktrees than are registered says which ones it left out,
/// so no worktree is passed over without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Targets = { covered: string[]; leftOut: string[] };
/// ```
pub(crate) struct Targets {
    /// The worktrees to convert,
    /// in input order.
    pub(crate) covered: Vec<PathBuf>,
    /// One sentence per registered worktree that is not covered.
    pub(crate) left_out: Vec<String>,
}

/// The worktrees a run covers:
/// every linked worktree inside the main checkout,
/// or every named path resolved from `cwd`.
fn targets(
    arguments: &ConvertArguments,
    cwd: &Path,
) -> Result<Targets, String> {
    if arguments.all {
        let root: PathBuf = main_checkout_root(cwd)?;
        let scan: LinkedWorktreeScan = scan_linked_worktrees(root.as_path())?;
        let mut covered: Vec<PathBuf> = Vec::new();
        for linked in scan.inside {
            covered.push(linked.worktree);
        }
        let mut left_out: Vec<String> = Vec::new();
        for left in scan.left_out {
            left_out.push(left.reason);
        }
        return Ok(Targets {
            covered,
            left_out,
        });
    }
    let mut named: Vec<PathBuf> = Vec::new();
    for word in &arguments.paths {
        named.push(cwd.join(word));
    }
    return Ok(Targets {
        covered: named,
        left_out: Vec::new(),
    });
}

/// What:
///  What a run found to convert and how its targets are grouped into lanes.
/// Why:
///  The two are found together,
/// in one job of the blocking pool
/// (`execute_run_awaiting`),
///  because both read the filesystem and one of them starts Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FoundTargets = { found: Targets | string; lanes: number[][] };
/// ```
pub(crate) struct FoundTargets {
    /// The targets,
    /// or the reason a run was refused before anything changed.
    pub(crate) found: Result<Targets, String>,
    /// The lanes of the targets,
    /// empty when the run was refused.
    pub(crate) lanes: Vec<Vec<usize>>,
}

/// What:
///  Finds what a run covers,
/// and groups its targets into lanes.
/// Why:
///  See `FoundTargets`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findTheTargetsAndLanes(arguments: ConvertArguments, cwd: string): FoundTargets;
/// ```
pub(crate) fn find_the_targets_and_lanes(
    arguments: &ConvertArguments,
    cwd: &Path,
) -> FoundTargets {
    let found: Targets = match targets(arguments, cwd) {
        Ok(covered) => covered,
        Err(reason) => {
            return FoundTargets {
                found: Err(reason),
                lanes: Vec::new(),
            };
        }
    };
    let lanes: Vec<Vec<usize>> = lanes_of(found.covered.as_slice());
    return FoundTargets {
        found: Ok(found),
        lanes,
    };
}

/// What:
///  The lanes of a run:
/// the targets that name one worktree share a lane,
/// so they run one after another,
/// and every other
///  target has a lane of its own.
/// Why:
///   The same worktree named twice (`wt wt`,
/// `wt ./wt`) is converted by the first and found converted by the second,
/// as in
///  a run of one after another;
/// run side by side,
/// the second would meet the claim of the first and be refused as busy with itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lanes = groupBy(list.keys(), (index) => realpath(list[index]) ?? list[index]);
/// ```
fn lanes_of(list: &[PathBuf]) -> Vec<Vec<usize>> {
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    let mut lane_of: HashMap<PathBuf, usize> = HashMap::new();
    for (index, path) in list.iter().enumerate() {
        // A path that cannot be resolved stands for itself, so it still gets a lane of its own.
        let real: PathBuf = match std::fs::canonicalize(path) {
            Ok(resolved) => resolved,
            Err(_unresolved) => path.clone(),
        };
        match lane_of.get(&real) {
            Some(&lane) => lanes[lane].push(index),
            None => {
                lane_of.insert(real, lanes.len());
                lanes.push(vec![index]);
            }
        }
    }
    return lanes;
}
