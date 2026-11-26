//! What:
//!  What a drain does with the plans of a repository the file system proves
//!  gone,
//!  and with those of a repository it cannot reach:
//!  the plans end,
//!  one journal line says which path lost them,
//!  and the pushes of a directory whose drive is not mounted keep their
//!  pauses.
//! Why:
//!  A repository that was deleted cannot be pushed again,
//!  and a plan kept for it is a try that can never run
//!  (`doc/planning/cli-git-unreachable-repository-ends-its-plans.md`),
//!  while a directory that cannot be reached runs nothing and is tried again
//!  when it is back;
//!  both answer with the drain's own value,
//!  so the loop reads one shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = await endThePlansOfAGoneRepositoryAwaiting(gitDir, retrying, unreachable, gone);
//! const kept = await keepPlansWhileUnreachable(gitDir, retrying, unreachable);
//! ```

use super::service_work_drained_and_replanning::{
    Drained,
    Replanning,
};
use super::service_work_planned_retry::{
    PlannedRetry,
    planned_retry,
};
use crate::push_retry::Retrying;
use crate::service_instance::GitDirUnreachable;
use crate::unreachable_git_dir_proven_gone::Gone;
use std::path::Path;

/// What:
///  What a drain does for a Git directory the filesystem proves is gone:
/// its push plans end,
/// and one journal line says which path
///  lost them and what the proof looked at.
/// Why:
///   A repository that was deleted cannot be pushed again,
/// and its work records went with it,
/// so a plan kept for it
///  is a try that can never run:
/// on 2026-11-10 two deleted throwaway repositories were retried every 600 s for hours,
/// at
///  try 48 and climbing
/// (`doc/planning/cli-git-unreachable-repository-ends-its-plans.md`).
/// The person's own notice lives inside the Git
///  directory,
/// which is gone,
/// so the journal is the only place that can say what became of the backup,
/// and the line names the
///  branch and the commit with the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = prove(gitDir, mounts); if (gone) { log(goneLine(gitDir, gone, branches)); for (const due of dueOf(gitDir)) await forget(due); }
/// ```
pub(crate) async fn end_the_plans_of_a_gone_repository_awaiting(
    git_dir: &Path,
    retrying: &Retrying,
    unreachable: &GitDirUnreachable,
    gone: &Gone,
) -> Drained {
    let mut replanning: Replanning = Replanning {
        first: retrying.first,
        changes: Vec::new(),
        plans: retrying.plans.clone(),
    };
    let mut named: Vec<String> = Vec::new();
    for due in &retrying.due {
        if due.git_dir != git_dir {
            continue;
        }
        named.push(format!("{} of {}", due.item.branch_ref, due.item.oid));
        replanning.forget_when_unreachable(git_dir, due).await;
    }
    // Nothing of this drain ran, so the caller has nothing to wait for; the file of each ended plan goes here while its
    //  repository is still missing, and the change drops the branch from the list the loop reads. A repository that is
    //  back by the time the change lands keeps its file, and the change puts its plan back (`PushRetries::apply`).
    if !named.is_empty() {
        crate::service_warn!(
            "{} ({unreachable})",
            crate::unreachable_git_dir_proven_gone::gone_line(git_dir, gone, named.as_slice())
        );
    }
    // What: Every value held against a push record of that repository is given up with its plans.
    // Why:  A repository the filesystem proves gone has no push that can run, and a value of a record nothing will run keeps a
    //  command line that can hold a password (`service_push_programs_held_in_memory`).
    crate::service_push_programs_held_in_memory::forget_every(git_dir);
    return Drained {
        wait: None,
        changes: replanning.changes,
    };
}

/// What:
///  What a drain does for a Git directory that cannot be reached (a link to a drive that is not mounted):
/// nothing runs,
/// its work stays where it is for the next command,
/// and every push that was due keeps its plan with the next pause,
/// doubled the way a failed try doubles it.
/// Why:
///  The push is not given up on and never runs on another path;
/// the plan is asked again for the pause that follows,
/// so the push is tried when the directory is back.
/// Without a first pause (a drain that keeps no plan) nothing is planned.
/// The person's failure record is inside the directory and is not touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const due of retrying.due) if (due.gitDir === gitDir) await plan(gitDir, due.item, due.attempt + 1, pauseBefore(due.attempt + 1))
/// ```
pub(crate) async fn keep_plans_while_unreachable(
    git_dir: &Path,
    retrying: &Retrying,
    unreachable: &GitDirUnreachable,
) -> Drained {
    crate::service_warn!(
        "{unreachable}; its work stays where it is for the next command, nothing runs, and its push plans keep their pauses"
    );
    let mut replanning: Replanning = Replanning {
        first: retrying.first,
        changes: Vec::new(),
        plans: retrying.plans.clone(),
    };
    let Some(first) = retrying.first else {
        return Drained {
            wait: None,
            changes: replanning.changes,
        };
    };
    for due in &retrying.due {
        if due.git_dir == git_dir {
            let cause: String = format!(
                "the repository at {} is not reachable now (is its drive mounted?)",
                git_dir.display()
            );
            let _said: PlannedRetry = planned_retry(
                git_dir,
                &mut replanning,
                first,
                &due.item,
                due.attempt + 1,
                cause.as_str(),
            )
            .await;
        }
    }
    return Drained {
        wait: None,
        changes: replanning.changes,
    };
}
