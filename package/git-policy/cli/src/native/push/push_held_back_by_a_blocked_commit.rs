//! What:
//!  Where the automatic push of a branch must stop,
//! and the words a command says about it,
//! when a post-commit gate kept one of the branch's
//!  commits local.
//! Why:
//!  A landed commit whose post-commit policy blocks it exits `2` and stays local
//! (`package/git-policy/cli/README.md`).
//!  The automatic push sends the branch and not one commit
//! (`git push <remote> <branch>:<target>`), so without this module the next push
//!  of that branch published the very commit the gate refused.
//!  The push now stops before the blocked commits:
//! it publishes the merge base of their parents,
//! which is below every one of them,
//! and it
//!  publishes nothing when that base cannot be named.
//!   A person's own `git push` is unchanged:
//! the wrapper forwards it,
//! and only the automatic push of the
//!  queue stops.
//!   Every command answers the same questions,
//! so the line a person reads and the push that happened cannot disagree,
//! and neither
//!  can outlive the verdict:
//! once the blocked commit is off the branch's way out, or Git already holds it in the branch's
//!  remote, nothing is said and the push runs whole again.
//!  A commit whose command died before its gate ran has no verdict,
//! so it holds
//!  nothing back, as before: the next push of the branch carries it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stops = whereThePushStops(history, branchRef, upstream, blockedVerdicts(workDir));
//! const source = stops.pushable ?? branchRef;
//! ```

/// The Git of the queue and its error type.
use crate::auto_push_single_flight::{PushGit, run_query};
/// The unchanged questions a reconcile asks the same Git.
use crate::auto_push_reconcile_basics_and_limits::{argv, is_ancestor};
/// The verdicts the post-commit gates kept.
use crate::work_item_holding::BlockedVerdict;
/// The repository-wide questions a command asks with its own Git context.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};

/// What:
///  The questions about a repository's history that decide where a push stops.
/// Why:
///   The queue asks them of the Git it pushes with,
/// and a command asks them of the Git context it already holds,
/// while the
///  decision, the wording and the limits live here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BranchHistory { holds(of: string, commit: string): boolean | undefined; firstParent(commit: string): string | undefined; mergeBase(commits: string[]): string | undefined; upstreamOf(branch: string): string | undefined }
/// ```
pub trait BranchHistory {
    /// Whether the reference `of` names a commit that holds `commit` among its ancestors,
    /// and nothing when Git could not tell.
    fn holds(&self, of: &str, commit: &str) -> Option<bool>;
    /// The first parent of `commit`,
    /// and nothing when it has none.
    fn first_parent(&self, commit: &str) -> Option<String>;
    /// The newest commit among the ancestors of every one of `commits`.
    fn merge_base(&self, commits: &[String]) -> Option<String>;
    /// The remote-tracking ref a push of `branch_ref` updates,
    /// and nothing when the branch has no upstream.
    fn upstream_of(&self, branch_ref: &str) -> Option<String>;
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
/// type WhereThePushStops = { kind: 'nothing' } | { kind: 'before'; commits: Held[]; pushable?: string } | { kind: 'unanswered'; commits: Held[] };
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
/// type Stopped = { commits: { commit: string; reason: string }[]; pushable?: string };
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
///  The reason a verdict with no words of its own is named with.
/// Why:
///   A record an older release wrote holds the word `blocked` alone,
/// and the line must still say why the commit
///  is not published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNRECORDED = 'its post-commit check blocked the automatic backup';
/// ```
pub const UNRECORDED_REASON: &str = "its post-commit check blocked the automatic backup";

/// What:
///  The verdicts of this worktree that are about `branch_ref`.
/// Why:
///   A verdict an earlier release wrote holds no branch,
/// and the commit of such a verdict may be on any branch,
/// so it is
///  judged for every branch rather than dropped;
/// only a verdict that names another branch is another branch's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// verdicts.filter((v) => v.branchRef === '' || v.branchRef === branchRef)
/// ```
pub fn of_the_branch<'a>(verdicts: &'a [BlockedVerdict], branch_ref: &str) -> Vec<&'a BlockedVerdict> {
    return verdicts.iter().filter(|verdict| verdict.branch_ref.is_empty() || verdict.branch_ref == branch_ref).collect();
}

/// What:
///  Where the push of one branch stops,
/// decided from the answers a history gave.
/// Why:
///   The decision is one place,
/// so the queue's push and a command's notice cannot judge the same verdict differently,
/// and
///  the answer to a question Git could not answer is never read as "nothing is blocked".
///   The commit to publish is the merge base of the blocked commits' parents,
/// which is below every one of them and
///  publishes as much of the branch as no verdict forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whereThePushStops(history, branchRef, upstream, verdicts): WhereThePushStops;
/// ```
pub fn where_the_push_stops(history: &dyn BranchHistory, branch_ref: &str, upstream: Option<&str>, verdicts: &[BlockedVerdict]) -> WhereThePushStops {
    let mut held: Vec<Held> = Vec::new();
    let mut unsure: Vec<Held> = Vec::new();
    let mut parents: Vec<String> = Vec::new();
    // A commit with no parent cannot be published below: the branch has nothing before it to send.
    let mut no_parent: bool = false;
    for verdict in of_the_branch(verdicts, branch_ref) {
        let held_one: Held = Held { commit: verdict.commit.clone(), reason: named_reason(verdict.reason.as_str()) };
        match history.holds(branch_ref, verdict.commit.as_str()) {
            Some(true) => {}
            Some(false) => continue,
            None => {
                unsure.push(held_one);
                continue;
            }
        }
        match upstream.map(|of| history.holds(of, verdict.commit.as_str())) {
            None | Some(Some(false)) => {}
            // Git already holds the commit in the branch's remote, so no push of this branch publishes it.
            Some(Some(true)) => continue,
            Some(None) => {
                unsure.push(held_one);
                continue;
            }
        }
        match history.first_parent(verdict.commit.as_str()) {
            Some(parent) => parents.push(parent),
            None => no_parent = true,
        }
        held.push(held_one);
    }
    if held.is_empty() {
        return match unsure.is_empty() {
            true => WhereThePushStops::Nothing,
            false => WhereThePushStops::Unanswered(Stopped { commits: unsure, pushable: None }),
        };
    }
    // A blocked commit with no parent has nothing before it to publish, and a base Git cannot name is no base.
    let pushable: Option<String> = match no_parent {
        true => None,
        false => merge_base(history, parents.as_slice()),
    };
    return WhereThePushStops::Before(Stopped { commits: held, pushable });
}

/// What:
///  The base every one of these commits is above,
/// or nothing when there is none.
/// Why:
///   One parent is its own base and costs no Git process;
/// a merge of sides,
/// or several blocked commits,
/// is one `merge-base` question
///  whose answer is below all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeBase = (history, parents) => parents.length === 1 ? parents[0] : history.mergeBase(parents);
/// ```
fn merge_base(history: &dyn BranchHistory, parents: &[String]) -> Option<String> {
    match parents {
        [] => return None,
        [only] => return Some(only.clone()),
        several => return history.merge_base(several),
    }
}

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
///  The branch a line names,
/// as the person reads it,
/// or the reference itself when it is not a branch.
/// Why:
///   The owner's own wording names `main`,
/// and a detached reference has no short name to give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortName = (ref) => ref.startsWith('refs/heads/') ? ref.slice(11) : ref;
/// ```
pub fn branch_name(branch_ref: &str) -> &str {
    return branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
}

/// What:
///  The one line a command prints about a branch the automatic push stops short of.
/// Why:
///   The person must know which commit holds the branch back,
/// what its own check found,
/// and that nothing was lost:
///  the commit is local,
/// a person's own `git push` still sends it,
/// and the push resumes by itself once the commit is
///  off the branch's way out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (branchRef, stopped) => `${name(branchRef)} is held back by the commit ${stopped.commits[0].commit}, because ${reason}. …`;
/// ```
pub fn held_back_line(branch_ref: &str, stopped: &Stopped) -> String {
    let name: &str = branch_name(branch_ref);
    let [only] = stopped.commits.as_slice() else {
        let listed: Vec<String> = stopped.commits.iter().map(|held| format!("{} ({})", held.commit, held.reason)).collect();
        return format!(
            "{name} is held back by these commits, because their own post-commit checks kept them local: {}. The automatic push publishes nothing past them, a person's `git push` still sends them, and the push resumes once they are off {name}'s way out.",
            listed.join(", ")
        );
    };
    return format!(
        "{name} is held back by the commit {}, because {}. The automatic push stops before it, a person's `git push` still sends it, and the push resumes once it is off {name}'s way out.",
        only.commit, only.reason
    );
}

/// What:
///  The line a command prints when Git could not say whether a verdict is still on the branch's way out.
/// Why:
///   Sending nothing is the safe answer,
/// and the person is told why nothing was sent and that the remedy is their own push;
/// a
///  line that named a verdict as holding the branch back would claim more than Git answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unsureLine = (branchRef, stopped) => `${name(branchRef)} may be held back by the commit ${stopped.commits[0].commit}, and Git could not say whether it is still on the branch's way out, so the automatic push sends nothing; a person's \`git push\` still sends it.`;
/// ```
pub fn unanswered_line(branch_ref: &str, stopped: &Stopped) -> String {
    let name: &str = branch_name(branch_ref);
    let named: Vec<String> = stopped.commits.iter().map(|held| held.commit.clone()).collect();
    return format!(
        "{name} may be held back by {}, because {}, and Git could not say whether it is still on {name}'s way out; the automatic push sends nothing until it can, and a person's `git push` still sends the branch.",
        named.join(", "),
        named_reason(stopped.commits.first().map(|held| held.reason.as_str()).unwrap_or_default())
    );
}

/// The questions the queue asks the Git it pushes with.
pub struct QueueHistory<'a> {
    /// The Git of the queue.
    pub git: &'a PushGit,
    /// The remote-tracking ref the plan already resolved for the pushed branch,
    /// so the push asks it nothing twice.
    pub upstream: Option<String>,
}

/// What:
///  Whether a reference names a commit Git can read,
/// answered from its exit status alone.
/// Why:
///   `merge-base --is-ancestor` fails for a reference that does not exist,
/// and that failure is the answer "the reference holds
///  nothing" and not "Git could not tell".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// revParseQuietly(git, ref) // exit 0 means the reference names a commit
/// ```
fn queue_names_a_commit(git: &PushGit, reference: &str) -> Option<bool> {
    let asked: String = format!("{reference}^{{commit}}");
    return match run_query(git, &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()])) {
        Ok(output) => Some(output.code == Some(0)),
        Err(_unanswerable) => None,
    };
}

impl BranchHistory for QueueHistory<'_> {
    fn holds(&self, of: &str, commit: &str) -> Option<bool> {
        match is_ancestor(self.git, commit, of) {
            Ok(Some(answer)) => return Some(answer),
            Ok(None) | Err(_) => {}
        }
        if queue_names_a_commit(self.git, of) == Some(false) {
            // The reference names no commit at all, so it holds nothing.
            return Some(false);
        }
        // A commit this repository does not hold cannot be published by any push of it, so the reference does not hold it; both
        //  sides existing and Git still unable to relate them is the one question left unanswered.
        return match queue_names_a_commit(self.git, commit) {
            Some(false) => Some(false),
            Some(true) | None => None,
        };
    }

    fn first_parent(&self, commit: &str) -> Option<String> {
        let asked: String = format!("{commit}^1");
        let output = run_query(self.git, &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()])).ok()?;
        if output.code != Some(0) {
            return None;
        }
        let parent: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return (!parent.is_empty()).then_some(parent);
    }

    fn merge_base(&self, commits: &[String]) -> Option<String> {
        let mut arguments: Vec<&str> = vec!["merge-base", "--octopus"];
        for commit in commits {
            arguments.push(commit.as_str());
        }
        let output = run_query(self.git, &argv(arguments.as_slice())).ok()?;
        if output.code != Some(0) {
            return None;
        }
        let base: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return (!base.is_empty()).then_some(base);
    }

    fn upstream_of(&self, _branch_ref: &str) -> Option<String> {
        return self.upstream.clone();
    }
}

/// The same questions a command asks through the Git context it holds.
pub struct ContextHistory<'a> {
    /// The Git program and the environment every child receives.
    pub context: &'a GitContext,
    /// The directory the questions are asked in.
    pub cwd: &'a std::path::Path,
}

impl ContextHistory<'_> {
    /// One question asked of the command's own Git,
    /// and what it answered when Git ran at all.
    fn ask(&self, arguments: &[&str]) -> Option<GitOutput> {
        let request: GitRequest = GitRequest::new(self.cwd, arguments);
        return run_git(self.context, &request).ok();
    }

    /// Whether `reference` names a commit,
    /// and nothing when Git could not be asked.
    fn names_a_commit(&self, reference: &str) -> Option<bool> {
        let asked: String = format!("{reference}^{{commit}}");
        return match self.ask(&["rev-parse", "--verify", "--quiet", asked.as_str()]).and_then(|output| output.code) {
            Some(0) => Some(true),
            Some(_no_commit) => Some(false),
            None => None,
        };
    }
}

impl BranchHistory for ContextHistory<'_> {
    fn holds(&self, of: &str, commit: &str) -> Option<bool> {
        match self.ask(&["merge-base", "--is-ancestor", commit, of]).and_then(|output| output.code) {
            Some(0) => return Some(true),
            Some(1) => return Some(false),
            _unknown_or_unasked => {}
        }
        if self.names_a_commit(of) == Some(false) {
            // The reference names no commit at all, so it holds nothing.
            return Some(false);
        }
        // A commit this repository does not hold cannot be published by any push of it, so the reference does not hold it; both
        //  sides existing and Git still unable to relate them is the one question left unanswered.
        return match self.names_a_commit(commit) {
            Some(false) => Some(false),
            Some(true) | None => None,
        };
    }

    fn first_parent(&self, commit: &str) -> Option<String> {
        let asked: String = format!("{commit}^1");
        let output: GitOutput = self.ask(&["rev-parse", "--verify", "--quiet", asked.as_str()])?;
        if output.code != Some(0) {
            return None;
        }
        let parent: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return (!parent.is_empty()).then_some(parent);
    }

    fn merge_base(&self, commits: &[String]) -> Option<String> {
        let mut arguments: Vec<&str> = vec!["merge-base", "--octopus"];
        for commit in commits {
            arguments.push(commit.as_str());
        }
        let output: GitOutput = self.ask(arguments.as_slice())?;
        if output.code != Some(0) {
            return None;
        }
        let base: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return (!base.is_empty()).then_some(base);
    }

    fn upstream_of(&self, branch_ref: &str) -> Option<String> {
        let short: &str = branch_ref.strip_prefix("refs/heads/")?;
        let asked: String = format!("{short}@{{upstream}}");
        let output: GitOutput = self.ask(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", asked.as_str()])?;
        if output.code != Some(0) {
            return None;
        }
        let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
        return (!named.is_empty()).then_some(named);
    }
}

/// What:
///  The lines a command prints about the branches this worktree's verdicts hold back.
/// Why:
///   The record and the repository are the only places the state exists,
/// so the line cannot outlive the verdict and cannot
///  disagree with `git status`:
/// a commit Git already holds in the branch's remote says nothing,
/// and a line stops
///  once the commit is off the branch's way out.
/// One line per branch,
/// naming the commits that hold it back.
///   A branch whose verdict cannot be judged says the uncertainty instead,
/// because the automatic push of that branch
///  sends nothing until it can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldBackLines(workDir, history) // one line per branch that holds a verdict back
/// ```
pub fn held_back_lines(work_dir: &std::path::Path, history: &dyn BranchHistory) -> Vec<String> {
    let verdicts: Vec<BlockedVerdict> = crate::work_item_holding::blocked_verdicts(work_dir);
    if verdicts.is_empty() {
        return Vec::new();
    }
    let mut branches: Vec<String> = Vec::new();
    for verdict in &verdicts {
        if !verdict.branch_ref.is_empty() && !branches.contains(&verdict.branch_ref) {
            branches.push(verdict.branch_ref.clone());
        }
    }
    let mut lines: Vec<String> = Vec::new();
    for branch_ref in branches {
        // The branch's own upstream decides whether Git already holds a verdict's commit, so nothing is said about a commit the
        //  remote has.
        let upstream: Option<String> = history.upstream_of(branch_ref.as_str());
        match where_the_push_stops(history, branch_ref.as_str(), upstream.as_deref(), verdicts.as_slice()) {
            WhereThePushStops::Nothing => {}
            WhereThePushStops::Before(stopped) => lines.push(held_back_line(branch_ref.as_str(), &stopped)),
            WhereThePushStops::Unanswered(stopped) => lines.push(unanswered_line(branch_ref.as_str(), &stopped)),
        }
    }
    return lines;
}

/// The decision's controls stay out of the release executable.
#[cfg(test)]
#[path = "push_held_back_by_a_blocked_commit_tests.rs"]
mod tests;
