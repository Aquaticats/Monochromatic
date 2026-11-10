//! What:
//!  The notice for a push whose branch the person deleted:
//! the refs that still hold its commit,
//! and the words that name them.
//! Why:
//!  A branch that is gone fails the same way on every try,
//! so the push is skipped once and the person is told where the commit lives;
//! the reading of the refs starts Git,
//! which a task of the runtime may only do awaited.
//!  The reading and the words sit in a file of their own,
//! because the handlers that use them are at the package's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = await branchIsGoneWords(git, item);
//! ```

use crate::auto_push_single_flight::PushGit;
use crate::work_item::WorkItem;

/// What:
///  The same words,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Reading the refs that hold the commit starts Git,
/// and the synchronous read is refused from inside the runtime.
/// The query and the words are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return goneWords(item, await refsHoldingOnTheRuntime(git, item));
/// ```
pub(crate) async fn branch_is_gone_words_awaiting(git: &PushGit, item: &WorkItem) -> String {
    return gone_words(item, refs_holding_awaiting(git, item).await);
}

/// What:
///  The same reading,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The read starts Git,
/// and only the awaited query may do that from a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refs = await git.for_each_ref(['--contains', item.oid, '--format=%(refname)']);
/// ```
async fn refs_holding_awaiting(git: &PushGit, item: &WorkItem) -> Option<Vec<String>> {
    let answer = match crate::auto_push_single_flight::run_query_awaiting(git, &crate::auto_push_reconcile_basics_and_limits::argv(&["for-each-ref", "--contains", item.oid.as_str(), "--format=%(refname)"])).await {
        Ok(answer) => answer,
        Err(error) => {
            crate::service_warn!("the refs that hold {} could not be read ({error}), so the notice does not say where the commit lives", item.oid);
            return None;
        }
    };
    return refs_of(answer, item);
}

/// What:
///  The refs one `for-each-ref` answer names,
/// or nothing when the query did not answer.
/// Why:
///   Both faces of the reading turn the answer into names here,
/// and a failed query is told once,
/// with the exit status Git gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answer.code === 0 ? answer.stdout.split('\n').filter(Boolean) : warn(...) ?? undefined
/// ```
fn refs_of(answer: crate::bounded_query::QueryOutput, item: &WorkItem) -> Option<Vec<String>> {
    if answer.code != Some(0) {
        crate::service_warn!("the refs that hold {} could not be read (git for-each-ref exited {}), so the notice does not say where the commit lives", item.oid, match answer.code { Some(code) => code.to_string(), None => String::from("on a signal") });
        return None;
    }
    let mut named: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(answer.stdout.as_slice()).lines() {
        let held: String = line.trim().to_owned();
        if !held.is_empty() {
            named.push(held);
        }
    }
    return Some(named);
}

/// What:
///  The words for a push whose branch the person deleted,
/// from the refs that still hold the commit.
/// Why:
///   Both faces of the notice word it here,
/// so the two cannot tell the person different things.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `... the branch ${branch} no longer exists ... ${whereItLives}`
/// ```
fn gone_words(item: &WorkItem, holding: Option<Vec<String>>) -> String {
    let branch: &str = item.branch_ref.strip_prefix("refs/heads/").unwrap_or(item.branch_ref.as_str());
    const NAMED_AT_MOST: usize = 5;
    let where_it_lives: String = match holding {
        None => format!(
            "Whether any ref of this repository still holds commit {} could not be told, because the refs could not be read.",
            item.oid
        ),
        Some(none) if none.is_empty() => format!(
            "Commit {} is on no ref of this repository any more, so only its object store keeps it and nothing was pushed for it.",
            item.oid
        ),
        Some(named) => {
            let shown: Vec<&str> = named.iter().take(NAMED_AT_MOST).map(String::as_str).collect();
            let more: usize = named.len().saturating_sub(shown.len());
            format!(
                "Commit {} is still on {}{}.",
                item.oid,
                shown.join(", "),
                if more == 0 { String::new() } else { format!(" and {more} more refs") }
            )
        }
    };
    return format!(
        "monochromatic-git-policy-cli: the branch {branch} no longer exists in this repository, so this push was not made: the branch was deleted. {where_it_lives}          Nothing is retried for it; a commit you want on a branch again can be pushed by hand after naming it with `git branch`."
    );
}
