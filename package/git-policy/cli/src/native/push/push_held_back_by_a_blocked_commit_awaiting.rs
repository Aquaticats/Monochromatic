//! What:
//!  Where the push of one branch stops when its questions go to a Git awaited on the runtime:
//! the synchronous face's own decision,
//! asked and answered with waits that are tasks of the runtime.
//! Why:
//!  The awaited push is a task of the runtime,
//! where a synchronous Git start is refused,
//! so its questions are awaited ones,
//!  while the decision they feed,
//! the verdicts they are asked about and the words that come out are
//!  the synchronous face's (`push_held_back_by_a_blocked_commit.rs`),
//! which is what keeps the two faces from judging one verdict differently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stops = await whereThePushStopsAwaiting(git, branchRef, upstream, verdicts, undecided);
//! ```

/// The unchanged ancestry question a reconcile asks the same Git,
/// awaited here.
use crate::auto_push_reconcile_basics_and_limits::{
    argv,
    is_ancestor_awaiting,
};
/// The Git of the queue,
/// whose questions this face asks as a task of the runtime.
use crate::auto_push_single_flight::{
    PushGit,
    run_query_awaiting,
};
/// What a finished query of the queue's Git answered.
use crate::bounded_query::QueryOutput;
/// The decision both faces read,
/// its parts and the questions that fill them.
use crate::push_held_back_by_a_blocked_commit::{
    Judged,
    Selection,
    WhereThePushStops,
    finish,
    judged_for,
    select,
    wants_the_parent_answer,
    wants_the_published_answer,
};
/// The verdicts the post-commit gates kept.
use crate::work_item_holding::BlockedVerdict;

/// What:
///  A finished query's output,
/// for the callers that read it after they judged its status.
/// Why:
///   A closure that captures nothing has a named function's shape,
/// and the package's linter asks for the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stdoutOf = (output) => output.stdout;
/// ```
fn stdout_of(output: QueryOutput) -> String {
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// What:
///  The exit status of a query,
/// and nothing when it was not run at all.
/// Why:
///   The status and the answer are read from one value,
/// and the caller needs both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statusAwaiting = (output) => output.code;
/// ```
fn status_awaiting(output: &QueryOutput) -> Option<i32> {
    return output.code;
}

/// What:
///  Whether the reference `of` names a commit that holds `commit`,
/// awaited on the runtime.
/// Why:
///   Only a question Git could not put,
/// or one it read a side as unknown,
/// leaves the answer open;
/// a reference or
///  a commit this repository does not name is the answer "it holds nothing".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async holds(of, commit) { const answer = await isAncestor(commit, of); if (answer !== undefined) return answer; if (await names(of) === false) return false; return (await names(commit)) === false ? false : undefined; }
/// ```
async fn awaiting_holds(
    git: &PushGit,
    of: &str,
    commit: &str,
) -> Option<bool> {
    if let Ok(Some(answer)) = is_ancestor_awaiting(git, commit, of).await {
        return Some(answer);
    }
    let asked: String = format!("{of}^{{commit}}");
    let named = run_query_awaiting(
        git,
        &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()]),
    )
    .await
    .ok()?;
    if named.code != Some(0) {
        // The reference names no commit at all, so it holds nothing.
        return Some(false);
    }
    let held: String = format!("{commit}^{{commit}}");
    let named_commit = run_query_awaiting(
        git,
        &argv(&["rev-parse", "--verify", "--quiet", held.as_str()]),
    )
    .await
    .ok()?;
    return match named_commit.code == Some(0) {
        false => Some(false),
        true => None,
    };
}

/// What:
///  Where the push of one branch stops,
/// from the answers a Git awaited on the runtime gives.
/// Why:
///   The awaited push is the synchronous one with its waits on the runtime,
/// so its questions,
/// its decision and its
///  wording are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function whereThePushStopsAwaiting(git, branchRef, upstream, verdicts, undecided): Promise<WhereThePushStops>;
/// ```
pub async fn where_the_push_stops_awaiting(
    git: &PushGit,
    branch_ref: &str,
    upstream: Option<&str>,
    verdicts: &[BlockedVerdict],
    undecided: &[String],
) -> WhereThePushStops {
    let mut judged: Vec<Judged> = judged_for(verdicts, undecided, branch_ref);
    for one in &mut judged {
        one.on_the_branch = awaiting_holds(git, branch_ref, one.verdict.commit.as_str()).await;
        if wants_the_published_answer(one) {
            one.published = match upstream {
                Some(of) => awaiting_holds(git, of, one.verdict.commit.as_str()).await,
                None => Some(false),
            };
        }
        if wants_the_parent_answer(one) {
            let asked: String = format!("{}^1", one.verdict.commit);
            let answered = run_query_awaiting(
                git,
                &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()]),
            )
            .await
            .ok();
            one.first_parent = match answered.as_ref().map(status_awaiting) {
                Some(Some(0)) => {
                    let parent: String = answered.map_or_else(String::new, stdout_of);
                    let trimmed: String = parent.trim().to_owned();
                    (!trimmed.is_empty()).then_some(trimmed)
                }
                _none_or_unasked => None,
            };
        }
    }
    let selection: Selection = select(judged.as_slice());
    let merged: Option<String> = match selection.parents.len() {
        0 | 1 => None,
        _several => {
            let mut arguments: Vec<&str> = vec!["merge-base", "--octopus"];
            for parent in &selection.parents {
                arguments.push(parent.as_str());
            }
            let answer = run_query_awaiting(git, &argv(arguments.as_slice()))
                .await
                .ok();
            match answer {
                Some(output) if output.code == Some(0) => {
                    let base: String = String::from_utf8_lossy(output.stdout.as_slice())
                        .trim()
                        .to_owned();
                    (!base.is_empty()).then_some(base)
                }
                _none => None,
            }
        }
    };
    return finish(selection, merged);
}
