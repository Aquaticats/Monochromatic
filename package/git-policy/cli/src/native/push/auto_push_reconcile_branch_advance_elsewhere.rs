//! What:
//!  Which worktree holds the branch a push sent,
//!  and the move of that branch to the merge published for it
//!  where that worktree is not the one this command runs in.
//! Why:
//!  A branch is checked out in at most one worktree,
//! and only the worktree that holds it has files to write when it moves;
//! a branch no worktree holds moves as a ref and costs no file at all.
//!  The move here is the one of the worktree the command runs in
//!  (`auto_push_reconcile_branch_advance.rs`, which this file calls):
//! the same two-way index and file update,
//!  the same compare-and-swap of the ref,
//! and the same record that guards a killed move,
//!  run with every Git start made in the worktree that has to move.
//!  A pushed branch that happened to share the command's checkout tip used to be moved over a
//!  merge of another branch;
//! this file is where that is decided by the ref Git says was pushed and by nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const advanced = await advancePushedBranch(git, branchRef, local, mergeOid); // here | elsewhere | nowhere | not
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
/// The here-path's move and the words both paths share.
use crate::auto_push_reconcile_branch_advance::{
    BRANCH_MOVED, SHIPPED_MOVE, advance_local_branch_awaiting, fast_forward_recorded, refusal_of, status_differs,
};
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
use crate::diagnostic_log::warn;
use crate::inner_git_words::record_streams;
use std::path::{Path, PathBuf};

/// What:
///  Where the published merge left the branch the push sent,
/// and what a person may have to do about it.
/// Why:
///  The report says which of these happened,
/// so a person reading one line learns where their branch is now;
///  a reason and an advice travel together,
/// because a refusal that names no way forward is a dead end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Advanced = { kind: 'here'; kept: boolean } | { kind: 'elsewhere'; worktree: string; kept: boolean } | { kind: 'nowhere' } | { kind: 'not'; reason: string; advice: string };
/// ```
pub(crate) enum Advanced {
    /// The branch and this worktree, which had it checked out, moved to the merge;
    /// `kept` says uncommitted work was left exactly as it was.
    Here {
        /// Whether uncommitted work was left in place.
        kept: bool,
    },
    /// The branch and the worktree at this path, which had it checked out, moved to the merge.
    Elsewhere {
        /// The worktree that had the branch checked out.
        worktree: PathBuf,
        /// Whether that worktree's uncommitted work was left in place.
        kept: bool,
    },
    /// No worktree had the branch checked out, so only the branch moved and no file was written.
    Nowhere,
    /// The branch was not moved,
    /// and the reason and the way forward are here.
    Not {
        /// Why the branch is not at the merge.
        reason: String,
        /// What moves it there once nothing is in the way.
        advice: String,
    },
}

/// What:
///  How a person moves a branch that the reconcile left where it was.
/// Why:
///   The merge is published,
/// so the branch follows it by a fast-forward of the merge commit itself,
/// which needs no remote-tracking ref and cannot carry anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const advice = (mergeOid, where) => `move it with \`git merge --ff-only ${mergeOid}\` ${where} once nothing is in the way`;
/// ```
pub(crate) fn advice_to_move(merge_oid: &str, where_to: &str) -> String {
    return format!("move it with `git merge --ff-only {merge_oid}` {where_to} once nothing is in the way");
}

/// What:
///  The worktree that has `branch_ref` checked out,
/// or nothing when no worktree has it.
/// Why:
///   A branch is checked out in at most one worktree,
/// and only that worktree's checkout has to follow the branch when it moves;
///  a branch no worktree has needs no file written at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = worktreeHolding((await git(['worktree','list','--porcelain','-z'])).split('\0'), branchRef);
/// ```
async fn worktree_holding_awaiting(git: &PushGit, branch_ref: &str) -> Result<Option<PathBuf>, SingleFlightError> {
    let listed = run_query_awaiting(git, &argv(&["worktree", "list", "--porcelain", "-z"])).await?;
    if listed.code != Some(0) {
        return Ok(None);
    }
    return Ok(worktree_holding(&listed.stdout, branch_ref));
}

/// What:
///  The worktree a `worktree list --porcelain -z` listing says has `branch_ref` checked out.
/// Why:
///   The listing is the only place Git answers that question for every worktree,
/// and `-z` keeps a path that holds bytes which are not text whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeHolding(fields, branchRef) { let path; for (const field of fields) { if (field.startsWith('worktree ')) path = field.slice(9); else if (field === `branch ${branchRef}`) return path; } return undefined; }
/// ```
pub(crate) fn worktree_holding(listing: &[u8], branch_ref: &str) -> Option<PathBuf> {
    let mut current: Option<PathBuf> = None;
    for field in listing.split(|byte| return *byte == 0) {
        if field.is_empty() {
            continue;
        }
        if let Some(path) = field.strip_prefix(b"worktree ".as_slice()) {
            current = Some(PathBuf::from(crate::path_bytes::os_from_path_bytes(path)));
            continue;
        }
        let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(field);
        if let Some(named) = text.strip_prefix("branch ")
            && named == branch_ref
        {
            return current;
        }
    }
    return None;
}

/// Whether this worktree is the one at `place`,
/// as far as the file system says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameDirectory = (left, right) => fs.realpathSync(left) === fs.realpathSync(right);
/// ```
fn same_directory(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    return match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(here), Ok(there)) => here == there,
        _ => false,
    };
}

/// What:
///  Whether this command runs in the worktree at `place`.
/// Why:
///   The two moves are not the same:
/// the one here asks this checkout and writes its files,
///  and the one elsewhere runs every question and every write in the other worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isHere = sameDirectory(await git(['rev-parse','--show-toplevel']), place);
/// ```
async fn is_this_worktree(git: &PushGit, place: &Path) -> Result<bool, SingleFlightError> {
    let top = run_query_awaiting(git, &argv(&["rev-parse", "--show-toplevel"])).await?;
    if top.code != Some(0) {
        return Ok(false);
    }
    let printed: &[u8] = top.stdout.strip_suffix(b"\n".as_slice()).unwrap_or(top.stdout.as_slice());
    let here: PathBuf = PathBuf::from(crate::path_bytes::os_from_path_bytes(printed));
    return Ok(same_directory(here.as_path(), place));
}

/// What:
///  Whether an environment variable would keep a Git started for another worktree from reading that worktree's
///  repository, its index or its files.
/// Why:
///   The caller's environment may name a Git directory,
/// a worktree or an index,
///  and every start of a move in another worktree must read the one it was asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesTheRepository = (name) => ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR'].includes(name);
/// ```
fn names_the_repository(name: &std::ffi::OsStr) -> bool {
    return matches!(
        name.to_str(),
        Some("GIT_DIR" | "GIT_WORK_TREE" | "GIT_INDEX_FILE" | "GIT_COMMON_DIR")
    );
}

/// What:
///  The same Git context, run in another worktree.
/// Why:
///   The move of a branch checked out elsewhere writes that worktree's index and files,
/// so every start of it has to run there,
///  and the caller's own repository variables would otherwise keep it in the repository this command was started in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const there = (git, directory) => ({ ...git, cwd: directory, environment: git.environment.filter(([name]) => !namesTheRepository(name)) });
/// ```
fn git_at(git: &PushGit, directory: &Path) -> PushGit {
    let environment: Vec<(std::ffi::OsString, std::ffi::OsString)> = git
        .environment
        .iter()
        .filter(|(name, _)| return !names_the_repository(name.as_os_str()))
        .cloned()
        .collect();
    return PushGit {
        real_git: git.real_git.clone(),
        cwd: directory.to_path_buf(),
        environment,
        lfs: git.lfs,
    };
}

/// What:
///  Move the branch and the worktree that has it checked out,
/// where that worktree is not this one,
/// or say why it was not moved.
/// Why:
///   The same checks the current worktree gets apply there:
/// the worktree still has the branch at the tip the push sent,
///  and none of its uncommitted work lies in a file the move writes.
/// The move is the two-way update and the compare-and-swap the current worktree's move is,
/// run in that
///  worktree,
/// so a worktree that its own command checks out another branch in keeps that branch's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceInAnotherWorktree(there, place, branchRef, local, mergeOid): Promise<Advanced>;
/// ```
async fn advance_in_another_worktree(
    there: &PushGit,
    place: &Path,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Advanced, SingleFlightError> {
    let named: String = format!("the worktree at {}", place.display());
    let where_to: String = format!("in {named}");
    let checked_out = run_query_awaiting(there, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0) || String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Advanced::Not {
            reason: format!("{named} checked out another branch while the merge was published"),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let head = run_query_awaiting(there, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if head.code != Some(0) || String::from_utf8_lossy(&head.stdout).trim() != local {
        return Ok(Advanced::Not {
            reason: String::from(BRANCH_MOVED),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let status_arguments = argv(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ]);
    let status = run_query_awaiting(there, &status_arguments).await?;
    if status.code != Some(0) {
        return Ok(Advanced::Not {
            reason: format!("the status of {named} could not be read"),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query_awaiting(
            there,
            &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
        )
        .await?;
        if incoming.code != Some(0) {
            return Ok(Advanced::Not {
                reason: format!("the files the merge changes in {named} could not be read"),
                advice: advice_to_move(merge_oid, where_to.as_str()),
            });
        }
        let overlap = overlapping_paths(&dirty, &diff_paths(&incoming.stdout));
        if !overlap.is_empty() {
            return Ok(Advanced::Not {
                reason: format!(
                    "{named} has uncommitted changes to files the merge changes: {}",
                    listed(&overlap)
                ),
                advice: advice_to_move(merge_oid, where_to.as_str()),
            });
        }
    }
    if let Err(reason) = fast_forward_recorded(there, branch_ref, local, merge_oid, &SHIPPED_MOVE).await? {
        return Ok(Advanced::Not {
            reason,
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    if kept {
        // That worktree's uncommitted work must read exactly as it did; the move is done, so a difference is reported.
        let after = run_query_awaiting(there, &status_arguments).await?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                &format!("the status of {named} differs after the fast-forward; compare it with the status before the merge"),
            );
        }
    }
    return Ok(Advanced::Elsewhere {
        worktree: place.to_path_buf(),
        kept,
    });
}

/// What:
///  Move a branch no worktree has checked out,
/// which writes no file,
/// or say why it was not moved.
/// Why:
///   A branch that is checked out nowhere has only its ref to move,
/// and the compare-and-swap of `update-ref` refuses a branch that
///  moved since the push without touching it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function moveTheRefOnly(git, branchRef, local, mergeOid): Promise<Advanced>;
/// ```
async fn move_the_ref_only_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Advanced, SingleFlightError> {
    let where_to: &str = "in a worktree that has it checked out";
    let tip = run_query_awaiting(git, &argv(&["rev-parse", "--verify", format!("{branch_ref}^{{commit}}").as_str()])).await?;
    if tip.code != Some(0) || String::from_utf8_lossy(&tip.stdout).trim() != local {
        return Ok(Advanced::Not {
            reason: String::from(BRANCH_MOVED),
            advice: advice_to_move(merge_oid, where_to),
        });
    }
    let message: String = format!("merge: fast-forward {branch_ref} to the published reconcile merge");
    let moved = run_query_awaiting(
        git,
        &argv(&["update-ref", "-m", message.as_str(), branch_ref, merge_oid, local]),
    )
    .await?;
    if moved.code != Some(0) {
        return Ok(Advanced::Not {
            reason: refusal_of(&moved.stderr),
            advice: advice_to_move(merge_oid, where_to),
        });
    }
    record_streams(&["update-ref", branch_ref, merge_oid, local], &moved.stdout, &moved.stderr);
    return Ok(Advanced::Nowhere);
}

/// What:
///  Bring the branch the push sent to the published merge,
/// in whichever worktree has it checked out,
/// and never another branch.
/// Why:
///   The branch left behind its own remote makes the next commit rejected again,
/// and the person would have to
///  merge by hand;
/// the merge contains the pushed tip,
/// so the move is a pure fast-forward.
///   Only the branch the push sent is moved:
/// the command's own checkout,
/// the branch it had checked out
///  and every other branch are left exactly where they were,
/// which is what the wrong name of a typed push broke.
///   A branch checked out in another worktree moves there under that worktree's own rules:
/// it follows only while it still
///  has the branch at the pushed tip and none of its uncommitted work lies in a file the move writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advancePushedBranch(git, branchRef, local, mergeOid): Promise<Advanced>;
/// ```
pub(crate) async fn advance_pushed_branch_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Advanced, SingleFlightError> {
    let Some(place) = worktree_holding_awaiting(git, branch_ref).await? else {
        return move_the_ref_only_awaiting(git, branch_ref, local, merge_oid).await;
    };
    if is_this_worktree(git, place.as_path()).await? {
        return match advance_local_branch_awaiting(git, branch_ref, local, merge_oid).await? {
            Ok(kept) => Ok(Advanced::Here { kept }),
            Err(reason) => Ok(Advanced::Not {
                reason,
                advice: advice_to_move(merge_oid, "in this worktree"),
            }),
        };
    }
    let there: PushGit = git_at(git, place.as_path());
    return advance_in_another_worktree(&there, place.as_path(), branch_ref, local, merge_oid).await;
}
