//! What:
//!  The one read that lets a fast rung step aside before it claims an add's record or copies a file:
//! which paths the tree of the commit the add names changes,
//! and whether one of them is a path the conversion fence must have Git write.
//! Why:
//!  A rung installs its index with the stat data of every path Git must write zeroed
//! (`doc/planning/cli-git-clone-rung-conversion-fence.md`),
//! and `git read-tree -m -u` verifies the entries of the paths it is about to change,
//!  so a zeroed entry the target tree changes makes it refuse the whole tree difference.
//! The rung then falls back to Git's checkout after its copies,
//!  which is correct and pays the copy twice:
//! a 10,002-file fixture measured 508 ms against `main`'s 378 ms for one such add.
//! A commit the add names is the only way a target tree can differ from the source's `HEAD`,
//!  and the fence's own attributes already say which paths it hands to Git,
//!  so the rung can answer before the claim and before the first copy,
//!  and a skipped add is Git's own checkout and nothing else.
//! The read is one `git diff-tree --name-only` between the source's frozen commit and the commit the person named:
//! names only,
//!  no blobs,
//!  no worktree.
//! It is the same set `git read-tree -m -u` is about to change,
//!  because a source on either rung holds no staged change
//!  (a source with changes is snapshotted and restored,
//!  never tree-differenced),
//!  so its index is the source's tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reason = skipReasonForTheTargetTree(source, context, plan, firstWrittenChange); // undefined: the rung runs
//! ```

use crate::diagnostic_log::debug;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::{OsStr, OsString};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadderTargetTree";

/// What:
///  The sentence that ends the rung when the tree the add names changes a path the fence must have Git write,
///  or nothing when the rung may run.
/// Why:
///   The answer is taken before the add's record is claimed and before anything is copied,
/// so a skipped add pays Git's own checkout and one name-only read,
///  and nothing of the rung's.
/// A commit the add names is the only shape whose target tree can differ;
/// an add that names none starts at the source's `HEAD`,
///  so it costs no read at all and this function answers nothing.
/// A read that fails decides nothing:
///  the rung then runs as it did and its fallback still keeps every byte guarantee.
/// `first` is the caller's own answer to which of the changed paths the fence hands to Git,
///  because each rung knows its own path set:
/// the snapshot rung has the whole fence before its snapshot,
///  and the clone rung has the attributes half before its copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = (source, context, plan, first) => {
///   const named = plan.target; if (!named) return undefined;
///   const changed = changedPaths(source, context, named); if (!changed) return undefined;
///   const path = first(changed); return path && `the tree of ${named} differs ... at ${path} ...`;
/// };
/// ```
pub fn skip_reason_for_the_target_tree(source: &SnapshotSource, context: &GitContext, plan: &AddPlan, first: &dyn Fn(&[Vec<u8>]) -> Option<Vec<u8>>) -> Option<String> {
    let named: &OsStr = plan.target.as_deref()?;
    let changed: Vec<Vec<u8>> = match changed_paths(context, source, named) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("the tree of {} cannot be read ({reason}); the rung runs as it did", String::from_utf8_lossy(named.as_encoded_bytes())).as_str());
            return None;
        }
    };
    let path: Vec<u8> = first(changed.as_slice())?;
    return Some(format!(
        "the tree of {} differs from the source's index at {}, which this rung must leave for Git to write, so this rung declines and Git's checkout writes the worktree",
        String::from_utf8_lossy(named.as_encoded_bytes()),
        String::from_utf8_lossy(path.as_slice())
    ));
}

/// What:
///  Every path the tree of `target` changes against the source's `HEAD`,
/// as one `git diff-tree -r -z --name-only` reports them.
/// Why:
///   The names are the whole answer,
/// so no blob and no worktree is read,
///  and a rename is two names rather than a pair (`--no-renames`),
///  which is what the tree update changes.
/// A target that does not resolve is Git's own error later;
/// this read answers it as a reason nothing is decided from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = (context, source, target) => git(context, source.root, ['diff-tree', '-r', '-z', '--name-only', '--no-renames', '--end-of-options', source.clean.headCommit, `${target}^{commit}`]).split('\0');
/// ```
fn changed_paths(context: &GitContext, source: &SnapshotSource, target: &OsStr) -> Result<Vec<Vec<u8>>, String> {
    let mut peeled: OsString = target.to_os_string();
    peeled.push("^{commit}");
    let mut arguments: Vec<OsString> = ["diff-tree", "-r", "-z", "--name-only", "--no-renames", "--end-of-options"].iter().map(|word: &&str| return OsString::from(word)).collect();
    arguments.push(OsString::from(source.clean.head_commit.as_str()));
    arguments.push(peeled);
    let request: GitRequest = GitRequest { cwd: source.root.clone(), arguments, without_prefix: true, ..GitRequest::default() };
    let output: GitOutput = run_git(context, &request).map_err(|error| return format!("a Git run of the target tree could not start ({error})"))?;
    if output.code != Some(0) {
        return Err(format!("`git diff-tree` ended with {:?} ({})", output.code, String::from_utf8_lossy(output.stderr.as_slice()).trim()));
    }
    let mut changed: Vec<Vec<u8>> = Vec::new();
    for path in output.stdout.split(|byte| return *byte == 0) {
        if path.is_empty() {
            continue;
        }
        changed.push(path.to_vec());
    }
    return Ok(changed);
}
