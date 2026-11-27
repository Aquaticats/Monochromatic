//! What:
//!  The move of the local branch after the merge was pushed:
//! the fast-forward,
//! the record that guards it,
//! the check that
//!  uncommitted work is not in the way,
//! and the words of a refusal.
//! Why:
//!  The branch follows the published merge only where nothing can be lost;
//! this file holds those checks apart from the
//!  retry loop in `auto_push_reconcile.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const advanced = await advanceLocalBranch(git, branchRef, local, mergeOid); // false | true | refusal text
//! ```

/// Import the step that moves the branch,
/// and the shipped one,
/// so a caller names them here as it always did.
pub(crate) use super::auto_push_reconcile_branch_move::{
    BranchMove,
    SHIPPED_MOVE,
};
/// Keep the step and the control's window reachable here,
/// where the controls that play them name them.
#[cfg(test)]
pub(crate) use super::auto_push_reconcile_branch_move::{
    between_look_and_write_in_a_control,
    move_the_named_branch,
};
/// Import the paths of a dirty worktree and of a fast-forward,
/// and their overlap.
use crate::auto_push_dirty_paths::{
    diff_paths,
    listed,
    overlapping_paths,
    status_paths,
};
/// Import the module's tag and the runner's argument builder.
use crate::auto_push_reconcile_basics_and_limits::{
    TAG,
    argv,
};
/// Import the probe's Git runner and its awaited queries.
use crate::auto_push_single_flight::{
    PushGit,
    SingleFlightError,
    run_query_awaiting,
};
/// Import the diagnostic log of the wrapper.
use crate::diagnostic_log::{
    debug,
    warn,
};
/// Import the clock and the identifier a record is written under.
use crate::iso_time::now_iso;
/// Import the record that says a fast-forward is about to happen,
/// and the steps that write and remove it.
use crate::merge_isolation_journal::{
    MergeIntent,
    remove_merge_intent,
    write_merge_intent,
};
/// Import the settling of a merge whose owner died,
/// and the place it reads Git in.
use crate::merge_isolation_recovery::{
    MergeRecoveryAction,
    RecoveryPlace,
    settle_failed_merge_awaiting,
};
use crate::random_id::random_uuid;
/// Import the Git context the settle is made with,
/// and the owner a record names.
use crate::transaction_git::GitContext;
use crate::transaction_owner::current_transaction_owner;
/// Import the borrowed and owned path types the record is written in.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether the status read after a fast-forward differs from the one read before it.
///  `&[u8]` borrows Git's output.
/// Why:
///   The uncommitted work must read exactly as it did;
/// a difference is the only sign that the move touched it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusDiffers(before: Uint8Array, after: Uint8Array): boolean;
/// ```
pub(crate) fn status_differs(
    before: &[u8],
    after: &[u8],
) -> bool {
    return before != after;
}

/// What:
///  Git's fast-forward to the published merge,
/// with an intent record written first and settled after,
/// so a branch that moved in
///  the instant between Git's checkout and its ref update,
/// or a service that died there,
/// leaves the checkout as it was or put
///  back to the moved head,
/// never half merged.
/// Why:
///   The owner's rule "Auto merging should never fail."
/// (`SPEC.md`,
/// "Merges and commits never fail"):
/// the person's own merge and
///  the reconcile can both move the same branch,
/// and the loser's checkout must not stay beside a branch that does not hold it.
///  The same record and settling serve the person's merge (`merge_isolation_recovery.rs`),
/// and a record whose owner died is
///  settled by the next wrapper command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await fastForwardRecorded({ git, branchRef, local, mergeOid }); // undefined when it moved
/// ```
pub(crate) async fn fast_forward_recorded(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
    move_branch: &dyn BranchMove,
) -> Result<Result<(), String>, SingleFlightError> {
    let git_dir: Option<PathBuf> = {
        let named = run_query_awaiting(git, &argv(&["rev-parse", "--absolute-git-dir"])).await?;
        // Git prints the directory's bytes and ends them with one line break that belongs to no path, so the record is
        //  written in the worktree a path that is not text names and not in a directory made from its text.
        let printed: &[u8] = named
            .stdout
            .strip_suffix(b"\n".as_slice())
            .unwrap_or(named.stdout.as_slice());
        if named.code == Some(0) {
            Some(PathBuf::from(crate::path_bytes::os_from_path_bytes(
                printed,
            )))
        } else {
            None
        }
    };
    let mut recorded: Option<(PathBuf, MergeIntent)> = None;
    if let Some(dir) = git_dir.as_deref() {
        recorded = record_merge_intent(dir, git, branch_ref, local, merge_oid);
    }
    let moved: Result<(), String> = move_branch
        .move_branch(git, branch_ref, local, merge_oid)
        .await?;
    let Some((record, intent)) = recorded else {
        // Without a record the fast-forward runs as it always did, and Git's refusal is the reason.
        return Ok(moved);
    };
    if moved.is_ok() {
        if let Err(error) = remove_merge_intent(record.as_path()) {
            debug(
                TAG,
                &format!("the merge record could not be removed: {error}"),
            );
        }
        return Ok(Ok(()));
    }
    let refusal: String = moved.err().unwrap_or_default();
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext {
            real_git: git.real_git.clone(),
            overlay: git.environment.clone(),
            global_prefix: Vec::new(),
            common_dir: git_dir.clone(),
        },
        cwd: git.cwd.clone(),
        common_dir: git_dir.clone().unwrap_or_default(),
        git_dir: git_dir.unwrap_or_default(),
    };
    // The settle reads Git, so it is the awaited face: a task can start no child through the synchronous one,
    //  which registers its step's pipes by entering the runtime.
    return Ok(
        match settle_failed_merge_awaiting(
            &place,
            record.as_path(),
            &intent,
            &mut std::io::stderr(),
        )
        .await
        {
            Ok(MergeRecoveryAction::Completed) => Ok(()),
            Ok(_) => Err(refusal),
            Err(error) => Err(format!("{refusal} ({})", error.error.0)),
        },
    );
}

/// What:
///  Fast-forward the pushed branch and this worktree to the published merge,
/// or the reason that was not done.
///  The success value says whether uncommitted work was left in place.
/// Why:
///   A branch left behind the remote makes the next commit rejected again,
/// and the person at the keyboard would have
///  to merge by hand;
/// the merge contains the branch tip,
/// so the move is a pure fast-forward.
///   It is attempted only where nothing can be lost or surprised:
/// the worktree has the branch checked out at the tip
///  that was pushed,
/// and no uncommitted change (tracked file,
/// staged entry,
/// or either name of a staged rename)
///  lies in a file the move writes.
///   Another session's work in progress therefore never holds the branch behind the remote,
/// while a move that
///  would write over such a file is left for the person.
///   The status read takes no optional lock,
/// and `git merge --ff-only` refuses,
/// without changing anything,
/// a move that
///  would overwrite an untracked file or a change made after the status read.
///   Every refusal is a reason for the report,
/// never a failure:
/// the merge is already published.
///
/// Why a control still calls it:
///   The loop fast-forwards through the awaited face (`advance_local_branch_awaiting`),
/// and the control that compares the two faces asks both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceLocalBranch({ git, branchRef, local, mergeOid }): Promise<{ reason?: string; keptChanges?: boolean }>;
/// ```
#[cfg(test)]
pub(crate) fn advance_local_branch(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<bool, String>, SingleFlightError> {
    return match crate::lazy_runtime::run_on_runtime(advance_local_branch_awaiting(
        git, branch_ref, local, merge_oid,
    )) {
        Ok(moved) => moved,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(
            refusal.to_string(),
        ))),
    };
}

/// What:
///  The same fast-forward,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every question the checks ask and every write the move makes is a start of Git,
/// and the synchronous face enters the runtime for each of them and is refused from there.
/// The checks,
/// the window and the record are the synchronous face's,
/// so the two cannot decide differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = await advanceLocalBranchOnTheRuntime({ git, branchRef, local, mergeOid });
/// ```
pub(crate) async fn advance_local_branch_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<bool, String>, SingleFlightError> {
    return advance_local_branch_moving(git, branch_ref, local, merge_oid, &SHIPPED_MOVE).await;
}

/// `advance_local_branch` with the step that does the move given,
/// so a control can play a `git switch` in the window between the checks and it.
pub(crate) async fn advance_local_branch_moving(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
    move_branch: &dyn BranchMove,
) -> Result<Result<bool, String>, SingleFlightError> {
    let checked_out = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0) {
        return Ok(Err(String::from("HEAD is detached")));
    }
    if String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Err(String::from(
            "this worktree has another branch checked out",
        )));
    }
    let head = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if String::from_utf8_lossy(&head.stdout).trim() != local {
        return Ok(Err(String::from(
            "the branch moved after the commit landed",
        )));
    }
    let status_arguments = argv(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ]);
    let status = run_query_awaiting(git, &status_arguments).await?;
    if status.code != Some(0) {
        return Ok(Err(String::from("the worktree status could not be read")));
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query_awaiting(
            git,
            &argv(&[
                "diff",
                "--name-only",
                "-z",
                "--no-renames",
                local,
                merge_oid,
            ]),
        )
        .await?;
        if incoming.code != Some(0) {
            return Ok(Err(String::from(
                "the files the merge changes could not be read",
            )));
        }
        let overlap = overlapping_paths(&dirty, &diff_paths(&incoming.stdout));
        if !overlap.is_empty() {
            return Ok(Err(format!(
                "the worktree has uncommitted changes to files the merge changes: {}",
                listed(&overlap)
            )));
        }
    }
    if let Err(reason) =
        fast_forward_recorded(git, branch_ref, local, merge_oid, move_branch).await?
    {
        return Ok(Err(reason));
    }
    if kept {
        // The uncommitted work must read exactly as it did; the move is already done, so a difference is reported.
        let after = run_query_awaiting(git, &status_arguments).await?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                "the worktree status differs after the fast-forward; compare it with the status before the commit",
            );
        }
    }
    return Ok(Ok(kept));
}

/// Write the record that says a fast-forward of the branch is about to happen,
/// or nothing when the record cannot be made.
pub(crate) fn record_merge_intent(
    dir: &Path,
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Option<(PathBuf, MergeIntent)> {
    let owner = current_transaction_owner(random_uuid().ok()?.as_str(), now_iso().as_str()).ok()?;
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: git.cwd.to_path_buf(),
        target_ref: Some(branch_ref.to_owned()),
        old_oid: local.to_owned(),
        new_oid: merge_oid.to_owned(),
        side_ref: None,
        carried: None,
    };
    let record: PathBuf = write_merge_intent(dir, &intent).ok()?;
    return Some((record, intent));
}
