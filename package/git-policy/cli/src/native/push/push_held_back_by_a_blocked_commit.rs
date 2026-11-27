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
//!  A commit whose gate is still running is the other side of that rule:
//! the queue holds its push until the gate answers,
//! because the answer may be a refusal,
//! and a push of the branch for any other commit sends every commit up to the tip.
//! The push therefore also stops before a commit whose recorded check has not decided and whose command still runs,
//! and the line says that it waits for those checks.
//!  The questions are asked twice over:
//! once by the service's git,
//! which pushes,
//! and once by a command's git context,
//! which only looks.
//! The answers feed one decision,
//! so both faces judge a verdict alike,
//! and the awaited face asks the same questions as a task of the runtime,
//! where a synchronous Git start is refused
//! (`push_held_back_by_a_blocked_commit_awaiting.rs`,
//! which is this file's decision with its questions awaited).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stops = whereThePushStops(history, branchRef, upstream, blockedVerdicts(workDir));
//! const source = stops.pushable ?? branchRef;
//! ```

/// The verdicts the post-commit gates kept.
use crate::work_item_holding::BlockedVerdict;

/// What:
///  The reason a verdict with no words of its own is named with.
/// Why:
///   A record an older release wrote holds the word `blocked` alone,
/// and the line must still say why the commit is
///  not published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNRECORDED = 'its post-commit check blocked the automatic backup';
/// ```
pub const UNRECORDED_REASON: &str = "its post-commit check blocked the automatic backup";

/// What:
///  The sentence a commit whose own post-commit check has not decided is named with.
/// Why:
///   The commit is not refused and nothing found it wanting:
/// the answer that may keep it local has simply not
///  been written yet,
/// and a line that borrowed the refusal's sentence would claim a check had blocked it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNDECIDED = 'its post-commit check has not decided whether it may be published';
/// ```
pub const UNDECIDED_REASON: &str =
    "its post-commit check has not decided whether it may be published";

/// What:
///  The words a person is given for one verdict.
/// Why:
///   A record with no words of its own is still a refusal,
/// and an empty clause would read as no reason at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedReason = (reason) => reason.trim() === '' ? UNRECORDED_REASON : reason.trim();
/// ```
pub fn named_reason(reason: &str) -> String {
    let named: &str = reason.trim();
    return match named.is_empty() {
        true => String::from(UNRECORDED_REASON),
        false => named.to_owned(),
    };
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
    fn holds(
        &self,
        of: &str,
        commit: &str,
    ) -> Option<bool>;
    /// The first parent of `commit`,
    /// and nothing when it has none.
    fn first_parent(
        &self,
        commit: &str,
    ) -> Option<String>;
    /// The newest commit among the ancestors of every one of `commits`,
    /// and nothing when Git could not name one.
    fn merge_base(
        &self,
        commits: &[String],
    ) -> Option<String>;
}

/// What:
///  What the automatic push may send of one branch.
/// Why:
///   `Nothing` keeps the push whole;
/// `Before` names the commits it may not publish and the commit it publishes instead,
/// so
///  the plan and the result are one decision;
/// `Undecided` names the commits whose own checks are still running,
/// which may not be published yet either;
/// `Unanswered` is the safe stop for a question Git could not answer,
/// because sending
///  nothing loses nothing while sending a commit the gate refused cannot be undone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhereThePushStops = { kind: 'nothing' } | { kind: 'before'; commits; pushable? } | { kind: 'undecided'; commits; pushable? } | { kind: 'unanswered'; commits };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhereThePushStops {
    /// No verdict holds this branch back.
    Nothing,
    /// Nothing past these commits is published.
    Before(Stopped),
    /// Nothing past these commits is published until their own post-commit checks decide.
    Undecided(Stopped),
    /// These commits may be on the branch's way out and Git could not say,
    /// so nothing is sent.
    Unanswered(Stopped),
}

/// What:
///  Why one commit may not be published yet.
/// Why:
///   A refusal is durable and already decided;
/// a commit whose checks run is neither,
/// and the line the person reads
///  must not claim a refusal that has not happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Because = 'refused' | 'undecided';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Because {
    /// A post-commit check refused the commit.
    Refused,
    /// The commit's post-commit check has not decided yet.
    Undecided,
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
/// type Judged = { verdict: BlockedVerdict; because: Because; onTheBranch?: boolean; published?: boolean; firstParent?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Judged {
    /// The verdict being judged,
    /// or the record of a commit whose checks have not decided.
    pub verdict: BlockedVerdict,
    /// Whether the commit was refused or is still waiting for its checks.
    pub because: Because,
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
/// each with every answer still missing,
/// and the commits whose checks still run.
/// Why:
///   A verdict an earlier release wrote holds no branch,
/// and its commit may be on any branch,
/// so it is judged for
///  every branch rather than dropped;
/// a verdict that names another branch is another branch's.
/// The undecided commits come from the records of the branch being pushed,
/// so they are judged for it alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judgedFor = (verdicts, undecided, branchRef) => [...verdicts.filter((v) => !v.branchRef || v.branchRef === branchRef), ...undecided].map((verdict) => ({ verdict }));
/// ```
pub fn judged_for(
    verdicts: &[BlockedVerdict],
    undecided: &[String],
    branch_ref: &str,
) -> Vec<Judged> {
    let mut judged: Vec<Judged> = Vec::new();
    for verdict in verdicts {
        if verdict.branch_ref.is_empty() || verdict.branch_ref == branch_ref {
            judged.push(Judged {
                verdict: verdict.clone(),
                because: Because::Refused,
                on_the_branch: None,
                published: None,
                first_parent: None,
            });
        }
    }
    for commit in undecided {
        let waiting: BlockedVerdict = BlockedVerdict {
            branch_ref: branch_ref.to_owned(),
            commit: commit.clone(),
            reason: String::from(UNDECIDED_REASON),
        };
        judged.push(Judged {
            verdict: waiting,
            because: Because::Undecided,
            on_the_branch: None,
            published: None,
            first_parent: None,
        });
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
/// type Selection = { commits: Held[]; undecided: Held[]; unsure: Held[]; parents: string[]; noParent: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    /// The verdicts that hold the branch back.
    pub commits: Vec<Held>,
    /// The commits whose own checks have not decided.
    pub undecided: Vec<Held>,
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
    let mut selection: Selection = Selection {
        commits: Vec::new(),
        undecided: Vec::new(),
        unsure: Vec::new(),
        parents: Vec::new(),
        no_parent: false,
    };
    for one in judged {
        // A refusal's words are named the way every line names them; a running check's own sentence is already in its record.
        let held_one: Held = match one.because {
            Because::Refused => Held {
                commit: one.verdict.commit.clone(),
                reason: named_reason(one.verdict.reason.as_str()),
            },
            Because::Undecided => Held {
                commit: one.verdict.commit.clone(),
                reason: one.verdict.reason.clone(),
            },
        };
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
        match one.because {
            Because::Refused => selection.commits.push(held_one),
            Because::Undecided => selection.undecided.push(held_one),
        }
    }
    return selection;
}

/// What:
///  The decision the selection and the base it needs make.
/// Why:
///   The base is the newest commit below every commit that may not be published,
/// refused or still waiting for its checks,
/// and one parent is its own base,
/// so only several parents
///  cost a question and only its answer can be used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finish = (selection, merged) => selection.commits.length ? { kind: 'before', commits: selection.commits, pushable: base(selection, merged) } : selection.undecided.length ? { kind: 'undecided', commits: selection.undecided, pushable: base(selection, merged) } : selection.unsure.length ? { kind: 'unanswered', commits: selection.unsure } : { kind: 'nothing' };
/// ```
pub fn finish(
    selection: Selection,
    merged: Option<String>,
) -> WhereThePushStops {
    if selection.commits.is_empty() && selection.undecided.is_empty() {
        return match selection.unsure.is_empty() {
            true => WhereThePushStops::Nothing,
            false => WhereThePushStops::Unanswered(Stopped {
                commits: selection.unsure,
                pushable: None,
            }),
        };
    }
    let base: Option<String> = match selection.parents.as_slice() {
        [] => None,
        [only] => Some(only.clone()),
        _several => merged,
    };
    // A held-back commit with no parent has nothing before it to publish, so the push sends nothing.
    let pushable: Option<String> = match selection.no_parent {
        true => None,
        false => base,
    };
    if selection.commits.is_empty() {
        return WhereThePushStops::Undecided(Stopped {
            commits: selection.undecided,
            pushable,
        });
    }
    return WhereThePushStops::Before(Stopped {
        commits: selection.commits,
        pushable,
    });
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
/// function whereThePushStops(history, branchRef, upstream, verdicts, undecided): WhereThePushStops;
/// ```
pub fn where_the_push_stops(
    history: &dyn BranchHistory,
    branch_ref: &str,
    upstream: Option<&str>,
    verdicts: &[BlockedVerdict],
    undecided: &[String],
) -> WhereThePushStops {
    let mut judged: Vec<Judged> = judged_for(verdicts, undecided, branch_ref);
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

/// The decision's controls stay out of the release executable.
#[cfg(test)]
#[path = "push_held_back_by_a_blocked_commit_tests.rs"]
mod tests;
