//! What:
//!  The facts a policy may ask a repository read for,
//!  and the two shapes the question about an abort form answers in.
//! Why:
//!  A policy names the interface and never the provider behind it,
//!  and the readings that answer one question of it name the same interface,
//!  so the interface lives in a file of its own that both use:
//!  in the file that defined it,
//!  the readings and the interface used each other,
//!  and a folder's files may not use each other in a loop (`AGENTS.md` rule `LYF`).
//!  The answer to the abort question is here for the same reason:
//!  the interface's own signature names it,
//!  so an interface file that reached back into the reading for it would close the loop again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface RepositoryFacts { location(): Promise<RepositoryLocation>; /* ... */ }
//! type AbortDiscard = { kind: 'nothing-waiting' } | { kind: 'waiting'; /* ... */ };
//! ```

/// Import the cause a candidate preparation may fail with.
use crate::candidate_error::CandidateError;
/// Import what a content policy reads and the candidates prepared for it.
use crate::candidate_prediction::{
    CandidateRequest,
    PreparedCandidates,
};
/// Import the location a policy asks for once.
use crate::repository_location::RepositoryLocation;
/// Import the operation an abort form ends and how it ends it.
use crate::rule_abort_operation::{
    AbortOperation,
    OperationEnd,
};
/// Import what one `git add` selects.
use crate::rule_add_explicit::StagedSelection;
/// Import whether the index differs from `HEAD`.
use crate::rule_commit_index::IndexVsHead;
/// Import the state of a merge,
///  a cherry-pick or a revert that waits for its concluding commit.
use crate::rule_commit_sequencer::SequencerState;
/// Import the command a rule guards before it runs.
use crate::rule_linked_worktree::GuardedCommand;
/// Import the scope of uncommitted state a question asks about.
use crate::rule_worktree_dirty::DirtyScope;
/// Import the transaction Git context the interface hands to recovery work.
use crate::transaction_git::GitContext;
/// Import how a location's identity names its common and its own Git directory.
use crate::worktree_identity::{
    common_directory,
    git_directory,
};
/// Import the borrowed and owned operating-system text of raw bytes.
use std::ffi::{
    OsStr,
    OsString,
};
/// Import the owned filesystem path a question answers with.
use std::path::PathBuf;

/// What:
///  The facts a rule may ask for.
///  A `trait` is a named set of methods a type promises
///       to provide,
///  like a TS `interface`;
///  `&mut self` lends the provider for writing,
///       because it remembers answers.
///  `Result<T, String>` is "a value,
///  or a message
///       saying why Git could not be asked or understood".
/// Why:
///   Two providers exist:
///  real Git for the executable,
///  a script for tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RepositoryFacts { location(): Promise<RepositoryLocation>; indexVsHead(): Promise<IndexVsHead>; /* ... */ }
/// ```
pub trait RepositoryFacts {
    /// Repository identity,
    ///  worktree top level and the path below it;
    ///  asked once.
    fn location(&mut self) -> Result<RepositoryLocation, String>;
    /// Whether the index differs from `HEAD`.
    fn index_vs_head(&mut self) -> Result<IndexVsHead, String>;
    /// How many paths the given positional pathspecs select,
    ///  and how many the whole worktree and the directory the command runs in give,
    ///  for the `git add` these pathspecs belong to.
    /// Asked only when a pathspec is no ordinary named path.
    fn staged_selection(
        &mut self,
        pathspecs: Vec<OsString>,
    ) -> Result<StagedSelection, String>;
    /// Whether the worktree holds uncommitted state of the given scope.
    fn worktree_dirty(
        &mut self,
        scope: DirtyScope,
    ) -> Result<bool, String>;
    /// The worktree's ignored files whose paths the given revision's tree also tracks,
    /// asked for a
    ///  forced checkout,
    ///  a forced switch and a hard reset.
    /// A revision Git cannot resolve falls back to `HEAD`,
    /// because every form that
    ///  names no tree writes `HEAD`'s content into the worktree;
    /// an unreadable `HEAD` is a failure,
    ///  never an empty answer.
    fn ignored_files_the_tree_tracks(
        &mut self,
        revision: Option<&OsStr>,
    ) -> Result<Vec<String>, String>;
    /// The worktree's untracked files whose paths the given revision's tree also tracks,
    /// asked before an abort restores that tree.
    /// A reset refuses an untracked file at one of those paths and loses nothing,
    ///  so the command does not run and no copy is owed.
    fn untracked_files_the_tree_tracks(
        &mut self,
        revision: &OsStr,
    ) -> Result<Vec<String>, String>;
    /// Whether the repository holds a stash entry.
    /// Asked for `git stash drop` and `git stash clear`,
    /// which delete entries of the one stash every worktree shares.
    fn repository_holds_a_stash(&mut self) -> Result<bool, String>;
    /// The paths the given `git clean --dry-run` argument list would delete,
    /// each as Git printed it,
    /// asked for a clean
    ///  that carries its own `-e`/`--exclude` patterns.
    fn clean_deleted_paths(
        &mut self,
        arguments: Vec<OsString>,
    ) -> Result<Vec<Vec<u8>>, String>;
    /// Whether a merge,
    ///  cherry-pick or revert awaits its concluding commit.
    fn sequencer_state(&mut self) -> Result<SequencerState, String>;
    /// The directory of the main checkout,
    ///  the worktree that owns the configuration file every worktree of the
    /// repository shares.
    /// Asked by a refusal in a linked worktree,
    ///  which must name where the setting belongs.
    fn main_worktree_root(&mut self) -> Result<PathBuf, String>;

    /// What the abort form `command` would discard besides the state of the operation it undoes,
    /// and whether that operation waits at all.
    /// Asked before an abort form runs,
    /// so the person's own work is saved first and the operation's own state is let go.
    fn abort_discard(
        &mut self,
        command: GuardedCommand,
    ) -> Result<AbortDiscard, String>;
    /// Whether `git switch <target>` or `git checkout <target>` would create a local
    /// branch from the one remote branch of that name.
    fn remote_guess_creates_branch(
        &mut self,
        target: &OsStr,
    ) -> Result<bool, String>;
    /// The candidates a content policy reads for this request,
    ///  prepared on a private
    /// index;
    ///  a failure carries the candidate layer's cause.
    fn candidates(
        &mut self,
        request: &CandidateRequest,
    ) -> Result<PreparedCandidates, CandidateError>;
    /// The location when it was already asked for,
    ///  without asking.
    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>>;
    /// The repository's common Git directory when it is known without starting Git:
    /// from the location already asked,
    ///  or from the file system.
    /// A provider that has neither answers nothing.
    fn known_common_dir(&mut self) -> Option<PathBuf> {
        let asked: Option<Result<RepositoryLocation, String>> = self.asked_location();
        if let Some(Ok(found)) = asked
            && let Some(directory) = common_directory(&found.identity)
        {
            return Some(directory.to_path_buf());
        }
        return None;
    }
    /// The invocation's own Git directory (the worktree's,
    /// which holds its work records and notices) when it is known without
    ///  starting Git:
    /// from the location already asked,
    /// or from the file system.
    /// A provider that has neither answers nothing.
    fn known_git_dir(&mut self) -> Option<PathBuf> {
        let asked: Option<Result<RepositoryLocation, String>> = self.asked_location();
        if let Some(Ok(found)) = asked
            && let Some(directory) = git_directory(&found.identity)
        {
            return Some(directory.to_path_buf());
        }
        return None;
    }
    /// Real Git,
    ///  the child environment and the invocation's global options,
    ///  for transaction
    /// and recovery work.
    fn transaction_context(&self) -> GitContext;
}

/// What:
///  What an abort form would discard,
///  from the shape of the repository alone.
/// Why:
///   The guard needs three different answers from this:
///  no state of this operation is there (Git refuses the command itself),
///  the operation this command ends waits and these paths hold the person's work,
///  or the commit whose tree the abort writes back is recorded somewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AbortDiscard = { kind: 'nothing-waiting' } | { kind: 'waiting'; inProgress: AbortOperation; restored: string; worktree: string[]; staged: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbortDiscard {
    /// The state this command's own operation writes is absent,
    ///  so Git refuses the command and discards nothing.
    NothingWaiting,
    /// The command's operation waits,
    ///  and these paths hold the work its command would discard besides its own.
    Waiting {
        /// Which operation this command ends.
        in_progress: AbortOperation,
        /// Whether the command ends the operation or goes on with the next commit or patch.
        ends: OperationEnd,
        /// The revision whose tree the abort writes back into the worktree,
        ///  as Git's own implementation of this abort finds it:
        ///  a name Git resolves,
        ///  or the commit read from the record Git wrote.
        restored: OsString,
        /// What the two listings below compare each side with:
        ///  the result Git recorded for a merge,
        ///  or the state the operation started from.
        baseline: DiscardBaseline,
        /// The repository-relative paths where the worktree differs from that baseline,
        ///  in a settled order and each once.
        worktree: Vec<Vec<u8>>,
        /// The listed paths whose index holds a whole entry (stage 0),
        ///  in a settled order and each once:
        ///  that is content the command's reset drops from the index itself.
        ///  A path whose index entry was erased is not among them and is no reason to refuse,
        ///  because the tree the reset writes back holds that path and its content:
        ///  an abort undoes the deletion and loses nothing of the person's.
        staged: Vec<Vec<u8>>,
        /// The paths the operation is still stopped at,
        ///  named by the worktree listing,
        ///  when Git recorded no result for the conflict.
        ///  Nothing tells the conflict text Git wrote there from a resolution the person wrote over it,
        ///  so each of them is kept as the person's and named as one the wrapper cannot judge.
        conflicted: Vec<Vec<u8>>,
    },
}

/// What:
///  What the two listings of a waiting command compare the worktree and the index with.
/// Why:
///   Git records the result it computed for the conflict it stopped at (`AUTO_MERGE`),
///  for a merge and for the rebase,
///  pick and `am` that use the same merge machinery (measured with Git 2.56.0),
///  so a path that still holds that result was never touched by the person.
///  A path whose worktree file still holds the recorded result and whose index entry differs
///  is content that stands only in the index.
///  Where Git recorded no result (an older Git,
/// a state it does not write for),
///  the listings compare each side with the other,
///  and a staged path's content is in the worktree as well.
///  A refusal that named the wrong one would tell the person the wrong place to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DiscardBaseline = 'recorded-result' | 'operation-start';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscardBaseline {
    /// `AUTO_MERGE` holds the result Git computed for the conflict it stopped at,
    ///  and both listings are read against it.
    RecordedResult,
    /// No such result is recorded:
    ///  the worktree is read against the index,
    ///  and the index against `HEAD`.
    OperationStart,
}
