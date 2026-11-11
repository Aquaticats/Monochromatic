//! What:
//!  What recovery reads to decide whose checkout this is and whether a merge began:
//!  the branch `HEAD` names,
//!  the commit a ref names,
//!  and whether the index or the worktree holds the merge.
//! Why:
//!  These are the reads a person's command pays for on every merge record it meets,
//! and they are reads only;
//!  the decision they feed lives in `merge_isolation_recovery.rs`,
//! which keeps the file under the package's line
//!  limit.
//!
//!  Each read is one body waited on by the caller:
//! the recovery that a task awaits,
//!  because a synchronous Git start registers its step's pipes by entering the runtime,
//! which is refused from inside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checkedOut = await symbolicRef({ place, root }); const current = await resolveCommit({ place, root, reference });
//! ```

use crate::merge_isolation_changes::{Change, parse_raw_changes};
use crate::merge_isolation_recovery_steps::{Judged, RecoveryPlace, Standing};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::{run_git_awaiting, run_git_checked_awaiting};
use std::path::Path;

/// What:
///  Whether an index entry is the merge's own,
/// which is the only evidence that Git began to install it.
/// Why:
///   Named rather than written as a closure inside `any`,
/// and it says what the check means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheMergesEntry = (standing: Standing) => standing === 'new';
/// ```
pub fn holds_the_merges_entry(standing: &Standing) -> bool {
    return *standing == Standing::New;
}

/// What:
///  Whether the worktree holds the merge's content at this path,
/// which is the only evidence that Git began.
/// Why:
///   Named rather than written as a closure inside `any`,
/// and it says what the check means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheMerge = (judged: Judged) => judged.standing === 'new';
/// ```
pub fn holds_the_merge(judged: &Judged) -> bool {
    return judged.standing == Standing::New;
}

/// What:
///  The branch `HEAD` names in `root`,
/// or nothing when it names a commit itself.
/// Why:
///   The checkout a merge installs into is the one its branch is checked out in;
/// `symbolic-ref` says which branch that is,
/// and fails
///  on a detached `HEAD`,
/// which the record names with no target ref of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = await git(['symbolic-ref', '--quiet', 'HEAD']); return named.succeeded ? named.stdout.trim() : undefined;
/// ```
pub async fn symbolic_ref_awaiting(place: &RecoveryPlace, root: &Path) -> Result<Option<String>, RecoveryError> {
    let request: GitRequest = GitRequest::new(root, &["symbolic-ref", "--quiet", "HEAD"]);
    let output = match run_git_awaiting(&place.git, &request).await {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(format!("git symbolic-ref could not start: {error}"))),
    };
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// What:
///  The commit a ref names,
/// or nothing when it names none.
/// Why:
///   The branch value decides whether the merge landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCommit(reference: string): Promise<string | undefined>;
/// ```
pub async fn resolve_commit_awaiting(place: &RecoveryPlace, root: &Path, reference: &str) -> Result<Option<String>, RecoveryError> {
    let revision: String = format!("{reference}^{{commit}}");
    let request: GitRequest = GitRequest::new(root, &["rev-parse", "--verify", "--quiet", revision.as_str()]);
    let output = match run_git_awaiting(&place.git, &request).await {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(format!("git rev-parse could not start: {error}"))),
    };
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// What:
///  Whether the branch at `tip` does not already hold the merge's content at each of the merge's paths.
/// Why:
///   A path whose content the branch already has is one an independent commit could have made,
/// so its match in the index
///  or the worktree is no evidence that recovery installed anything;
/// evidence is a path the branch does not have and
///  the checkout holds the merge for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathsTheBranchDoesNotHold({ place, root, tip, changes, nextOid }): boolean[]; // diff-tree tip..nextOid
/// ```
pub async fn paths_the_branch_does_not_hold_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    tip: &str,
    next_oid: &str,
    changes: &[Change],
) -> Result<Vec<bool>, RecoveryError> {
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", tip, next_oid],
    );
    let output = match run_git_checked_awaiting(&place.git, &request).await {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(error.to_string())),
    };
    let differing: Vec<Change> = parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError)?;
    let mut found: Vec<bool> = Vec::with_capacity(changes.len());
    for change in changes {
        let mut lacks: bool = false;
        for moved in &differing {
            if moved.path == change.path {
                lacks = true;
                break;
            }
        }
        found.push(lacks);
    }
    return Ok(found);
}

/// What:
///  Whether every path that stands in the way of a saved merge holds a file whose bytes are neither side's,
///  which is
///  the person's own edit and nothing a merge wrote.
/// Why:
///   A merge saved behind a live `index.lock` never began,
/// and where the person's own edits stand in its paths it cannot
///  be installed without writing over bytes only they hold.
///  A directory in the way,
/// or a file whose bytes are one side's and whose mode differs,
/// is not that:
/// both are
///  read as they always were,
/// and the refusal that keeps the record stands.
///  So the answer decides between settling the record with its ref
/// named and refusing, and it is read from the same judgements the
///  decision already has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function thePathsHoldOnlyThePersonsOwnBytes({ place, root, changes, judgements }): Promise<boolean>; // every Foreign path a file whose file form neither side has
/// ```
pub async fn the_paths_hold_only_the_persons_own_bytes_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    changes: &[Change],
    judgements: &[Judged],
) -> bool {
    let mut found_one: bool = false;
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        if judged.standing != Standing::Foreign {
            continue;
        }
        // A directory or another entry has no bytes of its own to judge, and a mode of the person's over a side's bytes is not an
        //  edit this answer covers.
        let Some(content) = judged.content.as_deref() else { return false };
        for side in [change.old_oid.as_str(), change.new_oid.as_str()] {
            if crate::merge_isolation_take_asides::the_file_form_of_awaiting(&place.git, root, change.path.as_slice(), side)
                .await
                .as_deref()
                == Some(content)
            {
                return false;
            }
        }
        found_one = true;
    }
    return found_one;
}
