//! What:
//!  The two questions the service's push asks a branch before it sends anything:
//! whether the branch exists,
//! and which commit it names.
//! Why:
//!  Both start Git,
//!  which a task of the runtime may only do awaited,
//!  and the handler that asks them is at the package line limit,
//!  so the questions sit in a file of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const exists = await branchExists(git, item.branchRef); const tip = await branchTip(git, item.branchRef);
//! ```

use crate::auto_push_single_flight::PushGit;

/// What:
///  The same question,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The question starts Git,
/// and the synchronous query enters the runtime and is refused from there.
/// The arguments and the reading of the exit status are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await branchExistsOnTheRuntime(git, item.branch_ref);
/// ```
pub(crate) async fn branch_exists_awaiting(
    git: &PushGit,
    branch_ref: &str,
) -> Option<bool> {
    let asked = crate::auto_push_single_flight::run_query_awaiting(
        git,
        &crate::auto_push_reconcile_basics_and_limits::argv(&[
            "rev-parse",
            "--verify",
            "--quiet",
            branch_ref,
        ]),
    )
    .await
    .ok()?;
    return match asked.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    };
}

/// What:
///  The commit a branch reference names,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The question starts Git,
/// and the synchronous query enters the runtime and is refused from there.
/// A reference that names no commit is the answer "nothing",
/// which is what a branch that was deleted reads as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tip = await branchTipOnTheRuntime(git, item.branchRef);
/// ```
pub(crate) async fn branch_tip_awaiting(
    git: &PushGit,
    branch_ref: &str,
) -> Option<String> {
    let asked: String = format!("{branch_ref}^{{commit}}");
    let answer = crate::auto_push_single_flight::run_query_awaiting(
        git,
        &crate::auto_push_reconcile_basics_and_limits::argv(&[
            "rev-parse",
            "--verify",
            "--quiet",
            asked.as_str(),
        ]),
    )
    .await
    .ok()?;
    if answer.code != Some(0) {
        return None;
    }
    let named: String = String::from_utf8_lossy(answer.stdout.as_slice())
        .trim()
        .to_owned();
    return (!named.is_empty()).then_some(named);
}
