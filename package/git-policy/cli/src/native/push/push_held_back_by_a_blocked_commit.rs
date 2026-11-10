//! What:
//!  Where the automatic push of a branch must stop,
//! and the words a command says about it,
//! when a post-commit gate kept one of the branch's commits local.
//! Why:
//!  A landed commit whose post-commit policy blocks it exits `2` and stays local
//! (`package/git-policy/cli/README.md`).
//!  The automatic push sends the branch and not one commit
//! (`git push <remote> <branch>:<target>`),
//! so without this module the next push of that branch published the very commit the gate refused.
//!  The push now stops before the blocked commits:
//! it publishes the merge base of their parents,
//! which is below every one of them,
//! and it publishes nothing when that base cannot be named.
//!  A person's own `git push` is unchanged:
//! the wrapper forwards it,
//! and only the automatic push of the queue stops.
//!  Every command answers the same questions,
//! so the line a person reads and the push that happened cannot disagree,
//! and neither can outlive the verdict:
//! once the blocked commit is off the branch's way out,
//! or Git already holds it in the branch's remote,
//! nothing is said and the push runs whole again.
//!  A commit whose command died before its gate ran has no verdict,
//! so it holds nothing back,
//! as before,
//! and the next push of the branch carries it.
//!  The questions are asked twice over:
//! once by the service's git,
//! which pushes,
//! and once by a command's git context,
//! which only looks.
//! The answers feed one decision,
//! so both faces judge a verdict alike,
//! and the awaited face asks the same questions as a task of the runtime,
//! where a synchronous Git start is refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stops = whereThePushStops(history, branchRef, upstream, blockedVerdicts(workDir));
//! const source = stops.pushable ?? branchRef;
//! ```

/// The Git of the queue,
/// whose questions the awaited face asks as a task of the runtime.
use crate::auto_push_single_flight::{PushGit, run_query_awaiting};
/// The unchanged ancestry question a reconcile asks the same Git.
use crate::auto_push_reconcile_basics_and_limits::{argv, is_ancestor_awaiting};
/// The words a verdict with none of its own is named with.
use crate::blocked_commit_lines::named_reason;
/// What a finished query of the queue's Git answered.
use crate::bounded_query::QueryOutput;
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
///  The questions about a repository's history that decide where a push stops.
/// Why:
///   The queue asks them of the Git it pushes with,
/// and a command asks them of the Git context it already holds,
/// while the decision,
/// the wording and the limits live here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BranchHistory { holds(of, commit); firstParent(commit); mergeBase(commits) }
/// ```
pub trait BranchHistory {
    /// Whether the reference `of` names a commit that holds `commit` among its ancestors,
    /// and nothing when Git could not tell.
    fn holds(&self, of: &str, commit: &str) -> Option<bool>;
    /// The first parent of `commit`,
    /// and nothing when it has none.
    fn first_parent(&self, commit: &str) -> Option<String>;
    /// The newest commit among the ancestors of every one of `commits`,
    /// and nothing when Git could not name one.
    fn merge_base(&self, commits: &[String]) -> Option<String>;
}

/// What:
///  What the automatic push may send of one branch.
/// Why:
///   `Nothing` keeps the push whole;
/// `Before` names the commits it may not publish and the commit it publishes instead,
/// so
///  the plan and the result are one decision;
/// `Unanswered` is the safe stop for a question Git could not answer,
/// because sending
///  nothing loses nothing while sending a commit the gate refused cannot be undone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhereThePushStops = { kind: 'nothing' } | { kind: 'before'; commits; pushable? } | { kind: 'unanswered'; commits };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhereThePushStops {
    /// No verdict holds this branch back.
    Nothing,
    /// Nothing past these commits is published.
    Before(Stopped),
    /// These commits may be on the branch's way out and Git could not say,
    /// so nothing is sent.
    Unanswered(Stopped),
}

/// What:
///  The commits a push stops before,
/// and the commit it publishes instead,
/// each with the words of its verdict.
/// Why:
///   The line a command prints and the refspec the push sends are read from one value,
/// so they cannot
///  name different commits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stopped = { commits: Held[]; pushable?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stopped {
    /// The commits the push may not publish,
    /// each with the words of its verdict.
    pub commits: Vec<Held>,
    /// The commit the push publishes,
    /// or nothing when it publishes nothing.
    pub pushable: Option<String>,
}

/// What:
///  One commit the push may not publish and the words of its verdict.
/// Why:
///   The pair is what the line and the log say:
/// the commit that holds the branch back,
/// and the check that refused it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Held = { commit: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Held {
    /// The commit the gate kept local.
    pub commit: String,
    /// What the gate said,
    /// or empty for a record an earlier release wrote.
    pub reason: String,
}

/// What:
///  One verdict with the answers the repository gave about it.
/// Why:
///   The questions are asked by two faces and the decision they feed is one place,
/// so a verdict cannot be judged
///  differently by the push that stops and the line that explains it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Judged = { verdict: BlockedVerdict; onTheBranch?: boolean; published?: boolean; firstParent?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Judged {
    /// The verdict being judged.
    pub verdict: BlockedVerdict,
    /// Whether the branch holds the commit;
    /// nothing when Git could not tell.
    pub on_the_branch: Option<bool>,
    /// Whether the commit is already on the branch's remote;
    /// nothing when Git could not tell.
    pub published: Option<bool>,
    /// The commit's first parent,
    /// when it was asked about.
    pub first_parent: Option<String>,
}

/// What:
///  The verdicts of one branch,
/// each with every answer still missing.
/// Why:
///   A verdict an earlier release wrote holds no branch,
/// and its commit may be on any branch,
/// so it is judged for
///  every branch rather than dropped;
/// a verdict that names another branch is another branch's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judgedFor = (verdicts, branchRef) => verdicts.filter((v) => !v.branchRef || v.branchRef === branchRef).map((verdict) => ({ verdict }));
/// ```
pub fn judged_for(verdicts: &[BlockedVerdict], branch_ref: &str) -> Vec<Judged> {
    let mut judged: Vec<Judged> = Vec::new();
    for verdict in verdicts {
        if verdict.branch_ref.is_empty() || verdict.branch_ref == branch_ref {
            judged.push(Judged { verdict: verdict.clone(), on_the_branch: None, published: None, first_parent: None });
        }
    }
    return judged;
}

/// What:
///  Whether the branch's remote has to be asked about this verdict.
/// Why:
///   Only a commit the branch holds can be published by a push of it,
/// and a commit the remote already holds publishes
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wantsPublished = (one) => one.onTheBranch === true;
/// ```
pub fn wants_the_published_answer(one: &Judged) -> bool {
    return one.on_the_branch == Some(true);
}

/// What:
///  Whether the commit's parent has to be asked about this verdict.
/// Why:
///   Only a commit on the branch and not on its remote holds the branch back,
/// and only such a commit contributes a
///  parent below which the push may publish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wantsParent = (one) => one.onTheBranch === true && one.published !== true;
/// ```
pub fn wants_the_parent_answer(one: &Judged) -> bool {
    return one.on_the_branch == Some(true) && one.published != Some(true);
}

/// What:
///  What the answers mean,
/// before the one question that depends on them.
/// Why:
///   Which verdicts hold the branch back is decided once for both faces,
/// and the base to publish is the answer to
///  a question about their parents,
/// so the decision is split where the questions are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selection = { commits: Held[]; unsure: Held[]; parents: string[]; noParent: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    /// The verdicts that hold the branch back.
    pub commits: Vec<Held>,
    /// The verdicts Git could not judge.
    pub unsure: Vec<Held>,
    /// The first parents of the commits that hold the branch back.
    pub parents: Vec<String>,
    /// Whether one of them has no parent,
    /// which leaves nothing below it to publish.
    pub no_parent: bool,
}

/// What:
///  The selection the answers make.
/// Why:
///   One place decides what "on the branch",
/// "already on the remote" and "cannot tell" mean together,
/// so a question Git could not answer is never read as "nothing is blocked".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const select = (judged) => judged.reduce(…);
/// ```
pub fn select(judged: &[Judged]) -> Selection {
    let mut selection: Selection = Selection { commits: Vec::new(), unsure: Vec::new(), parents: Vec::new(), no_parent: false };
    for one in judged {
        let held_one: Held = Held { commit: one.verdict.commit.clone(), reason: named_reason(one.verdict.reason.as_str()) };
        match one.on_the_branch {
            Some(false) => continue,
            Some(true) => {}
            None => {
                selection.unsure.push(held_one);
                continue;
            }
        }
        match one.published {
            // Git already holds the commit in the branch's remote, so no push of this branch publishes it.
            Some(true) => continue,
            Some(false) => {}
            None => {
                selection.unsure.push(held_one);
                continue;
            }
        }
        match &one.first_parent {
            Some(parent) => selection.parents.push(parent.clone()),
            None => selection.no_parent = true,
        }
        selection.commits.push(held_one);
    }
    return selection;
}

/// What:
///  The decision the selection and the base it needs make.
/// Why:
///   The base is the newest commit below every blocked one,
/// and one parent is its own base,
/// so only several parents
///  cost a question and only its answer can be used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finish = (selection, merged) => selection.commits.length ? { kind: 'before', commits: selection.commits, pushable: base(selection, merged) } : selection.unsure.length ? { kind: 'unanswered', commits: selection.unsure } : { kind: 'nothing' };
/// ```
pub fn finish(selection: Selection, merged: Option<String>) -> WhereThePushStops {
    if selection.commits.is_empty() {
        return match selection.unsure.is_empty() {
            true => WhereThePushStops::Nothing,
            false => WhereThePushStops::Unanswered(Stopped { commits: selection.unsure, pushable: None }),
        };
    }
    let base: Option<String> = match selection.parents.as_slice() {
        [] => None,
        [only] => Some(only.clone()),
        _several => merged,
    };
    // A blocked commit with no parent has nothing before it to publish, so the push sends nothing.
    let pushable: Option<String> = match selection.no_parent {
        true => None,
        false => base,
    };
    return WhereThePushStops::Before(Stopped { commits: selection.commits, pushable });
}

/// What:
///  Where the push of one branch stops,
/// from the answers a synchronous Git gives.
/// Why:
///   The queue's push and a command's notice both decide here,
/// so the two cannot judge the same verdict differently,
/// and the
///  answer to a question Git could not answer is never read as "nothing is blocked".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whereThePushStops(history, branchRef, upstream, verdicts): WhereThePushStops;
/// ```
pub fn where_the_push_stops(history: &dyn BranchHistory, branch_ref: &str, upstream: Option<&str>, verdicts: &[BlockedVerdict]) -> WhereThePushStops {
    let mut judged: Vec<Judged> = judged_for(verdicts, branch_ref);
    for one in &mut judged {
        one.on_the_branch = history.holds(branch_ref, one.verdict.commit.as_str());
        if wants_the_published_answer(one) {
            // A branch with no upstream has published nothing of itself, so only an upstream that exists is asked about.
            one.published = match upstream {
                Some(of) => history.holds(of, one.verdict.commit.as_str()),
                None => Some(false),
            };
        }
        if wants_the_parent_answer(one) {
            one.first_parent = history.first_parent(one.verdict.commit.as_str());
        }
    }
    let selection: Selection = select(judged.as_slice());
    let merged: Option<String> = match selection.parents.len() {
        0 | 1 => None,
        _several => history.merge_base(selection.parents.as_slice()),
    };
    return finish(selection, merged);
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
async fn awaiting_holds(git: &PushGit, of: &str, commit: &str) -> Option<bool> {
    if let Ok(Some(answer)) = is_ancestor_awaiting(git, commit, of).await {
        return Some(answer);
    }
    let asked: String = format!("{of}^{{commit}}");
    let named = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()])).await.ok()?;
    if named.code != Some(0) {
        // The reference names no commit at all, so it holds nothing.
        return Some(false);
    }
    let held: String = format!("{commit}^{{commit}}");
    let named_commit = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "--quiet", held.as_str()])).await.ok()?;
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
/// async function whereThePushStopsAwaiting(git, branchRef, upstream, verdicts): Promise<WhereThePushStops>;
/// ```
pub async fn where_the_push_stops_awaiting(
    git: &PushGit,
    branch_ref: &str,
    upstream: Option<&str>,
    verdicts: &[BlockedVerdict],
) -> WhereThePushStops {
    let mut judged: Vec<Judged> = judged_for(verdicts, branch_ref);
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
            let answered = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()])).await.ok();
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
            let answer = run_query_awaiting(git, &argv(arguments.as_slice())).await.ok();
            match answer {
                Some(output) if output.code == Some(0) => {
                    let base: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
                    (!base.is_empty()).then_some(base)
                }
                _none => None,
            }
        }
    };
    return finish(selection, merged);
}

/// The decision's controls stay out of the release executable.
#[cfg(test)]
#[path = "push_held_back_by_a_blocked_commit_tests.rs"]
mod tests;
