//! What:
//!  The questions about a repository's history that decide where the automatic push stops,
//! in the two faces that ask them:
//! a synchronous Git,
//! and a command's own Git context,
//! and the lines a command prints from the answers.
//! Why:
//!  The queue asks its questions of the Git it pushes with,
//! and a command asks its own of the context it holds,
//! while the decision
//!  and the wording live beside them in one place,
//! so the push that stops and the line that explains it cannot disagree.
//!  A reference or a commit this repository does not name is the answer "it holds nothing":
//! only a question Git
//!  could not put,
//! or one it read a side as unknown,
//! leaves the answer open.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const history = { git }; const lines = heldBackLines(gitDir, context, cwd);
//! ```

/// The Git of the queue and the error type its questions answer in.
use crate::auto_push_single_flight::{PushGit, run_query};
/// The unchanged ancestry question a reconcile asks the same Git.
use crate::auto_push_reconcile_basics_and_limits::{argv, is_ancestor};
/// The decision the answers feed,
/// and the lines a command prints from it.
use crate::blocked_commit_lines::{held_back_line, unanswered_line};
use crate::push_held_back_by_a_blocked_commit::{BranchHistory, WhereThePushStops, where_the_push_stops};
/// The repository-wide questions a command asks with its own Git context.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// The verdicts the post-commit gates kept.
use crate::work_item_holding::BlockedVerdict;

/// The questions the queue asks the synchronous Git it pushes with.
pub struct QueueHistory<'a> {
    /// The Git of the queue.
    pub git: &'a PushGit,
}

/// What:
///  Whether a reference names a commit Git can read,
/// and nothing when Git could not be asked.
/// Why:
///   `merge-base --is-ancestor` fails for a reference that does not exist,
/// and that failure is the answer "the reference
///  holds nothing" rather than "Git could not tell".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// revParseQuietly(git, ref) // true when the reference names a commit
/// ```
fn queue_names_a_commit(git: &PushGit, reference: &str) -> Option<bool> {
    let asked: String = format!("{reference}^{{commit}}");
    let output = run_query(git, &argv(&["rev-parse", "--verify", "--quiet", asked.as_str()])).ok()?;
    return Some(output.code == Some(0));
}

impl BranchHistory for QueueHistory<'_> {
    fn holds(&self, of: &str, commit: &str) -> Option<bool> {
        // A Git that could not put the question, or one that read a side as unknown, leaves the answer to the questions below.
        if let Ok(Some(answer)) = is_ancestor(self.git, commit, of) {
            return Some(answer);
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
}

/// The same questions a command asks through the Git context it holds.
pub struct ContextHistory<'a> {
    /// The Git program and the environment every child receives.
    pub context: &'a GitContext,
    /// The directory the questions are asked in.
    pub cwd: &'a std::path::Path,
}

/// What:
///  The exit status of a question Git answered,
/// and nothing when it did not run at all.
/// Why:
///   The answer and the failure to ask it are one value to the questions below,
/// and naming the step keeps the
///  closures the package's linter refuses out of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statusOf = (output) => output?.code;
/// ```
fn status_of(output: GitOutput) -> Option<i32> {
    return output.code;
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
        return match self.ask(&["rev-parse", "--verify", "--quiet", asked.as_str()]).and_then(status_of) {
            Some(0) => Some(true),
            Some(_no_commit) => Some(false),
            None => None,
        };
    }
}

impl BranchHistory for ContextHistory<'_> {
    fn holds(&self, of: &str, commit: &str) -> Option<bool> {
        match self.ask(&["merge-base", "--is-ancestor", commit, of]).and_then(status_of) {
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
}

/// What:
///  The remote-tracking reference a branch's pushes update,
/// asked of a command's own Git.
/// Why:
///   `@{upstream}` is Git's own answer,
/// so a branch whose upstream was never set is not guessed at from the settings,
/// and a commit Git already holds there publishes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const upstreamOf = (history, branchRef) => history.ask(['rev-parse', '--abbrev-ref', '--symbolic-full-name', `${short(branchRef)}@{upstream}`]) || undefined;
/// ```
pub fn upstream_of_a_branch(history: &ContextHistory<'_>, branch_ref: &str) -> Option<String> {
    let short: &str = branch_ref.strip_prefix("refs/heads/")?;
    let asked: String = format!("{short}@{{upstream}}");
    let output: GitOutput = history.ask(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", asked.as_str()])?;
    if output.code != Some(0) {
        return None;
    }
    let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return (!named.is_empty()).then_some(named);
}

/// What:
///  The lines a command prints about the branches this worktree's verdicts hold back.
/// Why:
///   The record and the repository are the only places the state exists,
/// so the line cannot outlive the verdict and cannot
///  disagree with `git status`:
/// a commit Git already holds in the branch's remote is not named,
/// and a line stops once the
///  commit is off the branch's way out.
/// One line per branch,
/// naming the commits that hold it back.
///   A branch whose verdict cannot be judged says the uncertainty instead,
/// because the automatic push of that branch
///  sends nothing until it can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldBackLines(gitDir, process.cwd()) // one line per branch that holds a verdict back
/// ```
pub fn held_back_lines(git_dir: &std::path::Path, history: &ContextHistory<'_>) -> Vec<String> {
    let verdicts: Vec<BlockedVerdict> = crate::work_item_holding::blocked_verdicts(crate::service_work::work_directory(git_dir).as_path());
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
        let upstream: Option<String> = upstream_of_a_branch(history, branch_ref.as_str());
        // A commit whose own checks still run is named by the held line of its record, which every command already prints, so
        //  nothing here is said about it: two lines for one wait would have to agree with each other as well as with the record.
        match where_the_push_stops(history, branch_ref.as_str(), upstream.as_deref(), verdicts.as_slice(), &[]) {
            WhereThePushStops::Nothing => {}
            WhereThePushStops::Before(stopped) => lines.push(held_back_line(branch_ref.as_str(), &stopped)),
            WhereThePushStops::Undecided(_waiting) => {}
            WhereThePushStops::Unanswered(stopped) => lines.push(unanswered_line(branch_ref.as_str(), &stopped)),
        }
    }
    return lines;
}
