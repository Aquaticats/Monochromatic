//! What:
//!  The refusals of an isolated merge, as the finding events a caller reads, each naming the paths involved and
//!  every valid way forward.
//! Why:
//!  A merge that cannot proceed changes nothing, and the person (or the agent) who ran it must learn which files
//!  stand in the way and what to do next without reading the wrapper's source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! renderPolicyEvents(1, [{ kind: 'core-finding', coreId: 'merge-isolation', code: 'staged-overlap', message }]);
//! ```

/// The path list of an overlap report: a few names, then a count.
use super::auto_push_dirty_paths::listed;
/// The finding event and its rendering.
use super::policy_events::{PolicyEvent, render_policy_events};

/// The finding family of every refusal here.
pub const CORE_ID: &str = "merge-isolation";

/// Code of the refusal when the merge changes a path another session holds changes in.
pub const OVERLAP_CODE: &str = "paths-in-use";

/// Code of the refusal when the merge conflicts textually.
pub const CONFLICT_CODE: &str = "merge-conflict";

/// Code of the refusal when Git refused the last step.
pub const APPLY_REFUSED_CODE: &str = "not-applied";

/// What:
///  Render one finding as a JSONL line.
/// Why:
///   Every refusal is a core finding of the merge-isolation family; the message carries the ways forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderPolicyEvents(1, [finding])
/// ```
fn render(code: &'static str, message: String) -> String {
    return render_policy_events(
        1,
        &[PolicyEvent::CoreFinding {
            core_id: CORE_ID,
            code,
            message,
        }],
    );
}

/// What:
///  The refusal for a merge that changes paths which the checkout holds staged or modified.
/// Why:
///   Those paths belong to another session's work; writing the merge over them would lose it, and Git itself refuses
///  the same overlap.
///  Every way out is named, including the one that needs no cooperation: a clean linked worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overlapFinding({ reference, paths }): string;
/// ```
pub fn overlap_finding(reference: &str, paths: &[Vec<u8>]) -> String {
    return render(
        OVERLAP_CODE,
        format!(
            "git merge {reference} was not run: the merge changes {}, which this checkout holds staged or modified \
             (probably another session's work in progress). Writing the merge there would overwrite that work, so \
             nothing was changed. Valid ways forward: wait until the session that owns those files has committed or \
             restored them, then merge again; or merge in a clean linked worktree \
             (`git worktree add -b merge-work ../merge-work <branch>`, then `git merge {reference}` there, and bring \
             the result back with `git merge --ff-only merge-work` here once those paths are free); or, when the \
             files are yours, commit them (`git commit -- <path>`) or set them aside (`git stash push -- <path>`) and \
             run the merge again.",
            listed(paths)
        ),
    );
}

/// What:
///  The refusal for a merge that conflicts textually.
/// Why:
///   A conflict needs a worktree to resolve in, and this checkout's worktree and index hold other work; resolving
///  there would mix the two.
///  The merge was computed from objects alone, so nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictFinding({ reference, paths }): string;
/// ```
pub fn conflict_finding(reference: &str, paths: &[String]) -> String {
    return render(
        CONFLICT_CODE,
        format!(
            "git merge {reference} was not run: merging it into the current branch conflicts in {}. Conflicts have \
             to be resolved in a worktree, and this checkout holds staged changes that are not yours, so nothing was \
             changed. Valid ways forward: merge in a clean linked worktree (`git worktree add -b merge-work \
             ../merge-work <this branch>`, then `git merge {reference}` there, resolve, commit, and bring the result \
             back with `git merge --ff-only merge-work` here); or resolve in a clean checkout of the repository; or \
             rebase or adjust {reference} so that it no longer conflicts.",
            paths.join(", ")
        ),
    );
}

/// What:
///  The refusal for a merge commit that was made but that Git refused to apply to the worktree.
/// Why:
///   The last step is Git's own, and it can still refuse (an untracked file in the way, a path changed after the
///  check); the branch, the index and the worktree are then as they were, and the commit stays unreachable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notAppliedFinding({ reference, detail }): string;
/// ```
pub fn not_applied_finding(reference: &str, detail: &str) -> String {
    return render(
        APPLY_REFUSED_CODE,
        format!(
            "git merge {reference} was not applied: Git refused to move the checkout to the merge ({detail}). The \
             branch, the index and the worktree are as they were. Valid ways forward: remove or move the files Git \
             names and merge again; or merge in a clean linked worktree and bring the result back with \
             `git merge --ff-only`."
        ),
    );
}

/// Finding text and the paths it names.
#[cfg(test)]
#[path = "merge_isolation_report_tests.rs"]
mod tests;
