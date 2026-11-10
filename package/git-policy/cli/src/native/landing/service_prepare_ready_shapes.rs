//! What:
//!  The shapes a service preparation hands back:
//! the scratch directory that removes itself,
//! what a preparation came to
//!  (conflict,
//! ended,
//! ready),
//! the inputs of the native Git run,
//! the `Ready` record,
//! and the record that builds a `Ready` for
//!  the ways out that share their parts.
//! Why:
//!  The preparation (`service_prepare.rs`),
//! the replay of captured changes and the commit loop all speak in these;
//! they sit in a
//!  file that uses none of those.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Prepared = Conflict | Ended | Ready; type Ready = { repository, reference, base, changes, events, inputs, scratch, ... }
//! ```

use crate::capture_invocation::InvocationCapture;
use crate::commit_transaction::TransactionResult;
use crate::lfs_pre_push::uses_lfs;
use crate::policy_events::PolicyEvent;
use crate::scratch_location::prepare_commit_scratch;
use crate::scratch_entry_removal::{Removal, SCRATCH_REMOVAL_CEILING, remove_within};
use crate::service_protocol::{Change, Cleanup};
use crate::transaction_journal::{AddedPath, Conclusion, TransactionMode};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
/// The commit's scratch directory:
/// the private indexes and Git's scratch repository,
/// and who removes it.
/// The resident service removes it when the connection that holds it closes,
/// by whatever means this process ends,
/// once the command has handed it over (`held_by_the_service`).
/// A command that never handed it over,
/// because the service refused it or because the preparation stopped where it was,
/// removes it itself as it goes out of scope,
/// bounded by `SCRATCH_REMOVAL_CEILING`,
/// and a tree that removal leaves behind is inside the repository scratch directory the sweep covers.
/// Why not here on the ordinary path:
/// a recursive removal after the commit landed waited on the person
/// (one `unlinkat` of 2.5 s in a shaped fixture,
/// `doc/planning/cli-git-commit-latency-under-many-agents.md`),
/// while nothing after it needed the bytes.
pub struct Scratch(
    /// Where the tree is.
    pub PathBuf,
    /// Whether the service holds it,
    /// so this process must leave it alone.
    pub bool,
    /// The renewal that keeps the tree from aging out while this command lives,
    /// and that stops when this value goes.
    pub Option<crate::service_scratch_renewal::ScratchRenewal>,
);

impl Scratch {
    /// What:
    ///  The commit's scratch directory,
    ///  made where the sweep of the repository scratch directory covers it,
    ///  and renewed while this command lives.
    /// Why:
    ///   A directory there is collected after a day even if this command is killed at any instruction and no service ever holds
    ///  it,
    ///  which a directory in the worktree's Git directory is not,
    ///  and the renewal is what tells a later command whether this one still runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const scratch = makeScratch(commonDir, gitDir, name);
    /// ```
    pub fn make(common_dir: &Path, git_dir: &Path, name: &str) -> std::io::Result<Self> {
        let path: PathBuf = prepare_commit_scratch(common_dir, git_dir, name, SCRATCH_REMOVAL_CEILING)?;
        let renewal: crate::service_scratch_renewal::ScratchRenewal = crate::service_scratch_renewal::ScratchRenewal::of_a_commit(path.as_path());
        return Ok(Self(path, false, Some(renewal)));
    }

    /// What:
    ///  Records whether the service holds the directory,
    ///  so this process must not remove a held one.
    /// Why:
    ///   The connection that carries the announcement is the service's claim on the directory,
    ///  and it removes the directory when that connection closes,
    ///  however this process ends.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scratch.heldByTheService(watch !== undefined);
    /// ```
    pub fn held_by_the_service(&mut self, held: bool) {
        self.1 = held;
    }

    /// What:
    ///  Takes a tree the service handed back to this command,
    ///  and starts its removal beside the command.
    /// Why:
    ///   A request the service answered without landing (a commit sent back to be prepared again,
    /// a branch that moved)
    ///  leaves its tree to this process,
    ///  which reads the change it captured from the tree's private index to make the commit again
    ///  (`service_server.rs`,
    ///  `remove_the_trees`).
    ///  The read is over when this is called,
    ///  and the removal is recursive and bounded by `SCRATCH_REMOVAL_CEILING`,
    ///  so it runs beside the command and never makes it wait.
    ///  A process killed before it calls this leaves the tree inside the repository scratch directory,
    ///  where the sweep of a later command removes it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scratch.releaseBesideTheCommand();
    /// ```
    pub fn release_beside_the_command(&mut self) {
        // The removal started here is this tree's only remover, so the drop must not start another.
        self.1 = true;
        crate::service_scratch_release_removal::start(self.0.clone());
    }
}

impl Drop for Scratch {
    /// Removes the directory a command that never handed it over leaves,
    /// and waits for it no longer than a scratch removal may take.
    fn drop(&mut self) {
        if self.1 {
            return;
        }
        let _removed: Removal = remove_within(self.0.as_path(), SCRATCH_REMOVAL_CEILING);
    }
}

/// A tree the service answered it did not land,
///  released beside the command when the answer's arm ends.
/// Why:
///   The tree is this command's again,
///  and it is read while the arm runs,
///  so the release must wait for the arm to end and must not make the command wait for a recursive removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HandedBackTree { #path; [Symbol.dispose]() { startRemoval(#path); } }
/// ```
pub struct HandedBackTree(pub(crate) PathBuf);

impl Drop for HandedBackTree {
    fn drop(&mut self) {
        crate::service_scratch_release_removal::start(self.0.clone());
    }
}

/// What a preparation came to.
pub enum Prepared {
    /// The policies stopped the commit,
    /// or the preparation itself found the commit's ending (a staged deletion that is
    ///  still on disk,
    /// a normalization that removed every change).
    Ended(TransactionResult),
    /// The change is ready to land.
    Ready(Box<Ready>),
    /// A retry could not merge the captured change onto the branch;
    /// nothing was changed.
    Conflict(Box<ReplayConflict>),
}

/// What real Git needs from the preparation to build the commit in a scratch repository.
pub struct NativeInputs {
    /// The invocation capture the scratch repository is made from.
    pub capture: InvocationCapture,
    /// The arguments after the policies' fixed transforms.
    pub arguments: Vec<OsString>,
    /// The positional pathspecs of the command line,
    /// which the private index already holds.
    pub pathspecs: Vec<String>,
    /// Whether the line commits named paths or the index.
    pub mode: TransactionMode,
    /// Whether `-a` staged tracked changes into the private index.
    pub staged_all: bool,
    /// Whether the private index already holds a selection by `--include`,
    /// `--patch`,
    /// `--interactive`.
    pub selected_privately: bool,
    /// The private index the policies settled.
    pub commit_index: PathBuf,
}

/// Everything a landing needs,
/// from one preparation.
pub struct Ready {
    /// The worktree root.
    pub repository: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// The branch,
    /// or `HEAD` for a detached head.
    pub reference: String,
    /// The commit prepared against;
    /// the all-zero name for a branch that does not exist yet.
    pub base: String,
    /// The changes between the base and the private index,
    /// when the embedded route can carry them;
    /// nothing when a name is
    ///  not text,
    /// a mode is not an ordinary file,
    /// a symbolic link or an executable,
    /// or there are too many.
    pub changes: Option<Vec<Change>>,
    /// The policies' events,
    /// with the fix summary when a correction changed a file.
    pub events: Vec<PolicyEvent>,
    /// Whether the repository uses Git LFS.
    pub uses_lfs: bool,
    /// The commit's scratch directory,
    /// kept until the commit is landed because Git's scratch repository lives in it.
    pub scratch: Scratch,
    /// Whether only Git can build this commit:
    /// the configuration asks for something the service does not embed,
    /// or the
    ///  head,
    /// the branch or the state is not an ordinary one,
    /// or Git alone can say what the line means.
    pub needs_native: bool,
    /// What real Git needs to build the commit,
    /// or what the embedded route needs to make its tree.
    pub inputs: NativeInputs,
    /// The files a policy rewrote that the worktree receives once the commit has landed.
    pub worktree_records: Vec<AddedPath>,
    /// The conclusion state the landing removes,
    /// for the commit that concludes a merge,
    /// a cherry-pick or a revert.
    pub cleanup: Option<Cleanup>,
    /// Whether the person chose what to commit interactively,
    /// so that a rebuild keeps the private index holding that choice and asks no selection option again.
    pub interactive: bool,
    /// Whether a moved branch makes another build on the new tip meaningless (a conclusion).
    /// Why:
    ///   A conclusion's parents come from the state the worktree holds,
    ///  so the same build makes it on the tip as it is now;
    ///  a branch that did not exist is not one of these, because the change captured against it is a whole root tree
    ///  that the replay puts on the branch as it is now.
    pub fixed_parents: bool,
    /// When the worktree's bytes were captured (`service_replay::capture_stamp`);
    /// a retry keeps the first capture's stamp.
    pub captured_at: u64,
    /// The paths this commit commits that the unfinished merge or pick left in conflict.
    /// Their worktree bytes,
    /// conflict markers
    ///  included,
    /// are what is committed;
    /// the person is warned once the commit has landed (`conflicted_path_warning`).
    pub committed_conflicted: Vec<Vec<u8>>,
    /// The operation a commit of named paths leaves in progress:
    /// the merge,
    /// the cherry-pick or the revert whose state the commit does not
    ///  conclude.
    /// The person is told once the commit has landed.
    pub leaves_open: Option<Conclusion>,
    /// What the merge,
    /// cherry-pick or revert state named when this commit was prepared,
    /// so a replay made on a later tip concludes the same operation;
    /// nothing when the commit concluded none.
    /// Why:
    ///   A conclusion's parents come from that state,
    /// and another command that concluded or abandoned the operation leaves nothing to conclude,
    /// so the commit is refused rather than becoming an ordinary commit of the same tree
    /// (`service_replay.rs`,
    /// `Captured::conclusion`).
    pub conclusion_state: Option<String>,
}

/// What:
///  What every `Ready` of one preparation shares:
/// the pathspecs,
/// the selection mode,
/// the private index,
/// the branch and base the
///  preparation announced,
/// the cleanup of the conclusion state and the stamps.
/// Why:
///  The preparation makes a `Ready` on two of its ways out;
/// one record builds both,
/// so the two are the same by construction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shape = { pathspecs, mode, stagedAll, selectedPrivately, commitIndex, reference, announcedBase, cleanup, interactive, fixedParents, capturedAt };
/// const ready = byGit(shape, taken, events, changes, records, args, scratch);
/// ```
pub(crate) struct ReadyShape {
    /// The positional pathspecs as text.
    pub(crate) pathspecs: Vec<String>,
    /// Whether the line commits named paths or the index.
    pub(crate) mode: TransactionMode,
    /// Whether `-a` staged tracked changes into the private index.
    pub(crate) staged_all: bool,
    /// Whether the private index already holds a selection.
    pub(crate) selected_privately: bool,
    /// The private index the commit is built from.
    pub(crate) commit_index: PathBuf,
    /// The branch reference the preparation announced.
    pub(crate) reference: String,
    /// The base the preparation announced.
    pub(crate) announced_base: String,
    /// The conclusion state to remove after the landing.
    pub(crate) cleanup: Option<Cleanup>,
    /// Whether an interactive selection was made.
    pub(crate) interactive: bool,
    /// Whether the parents of the commit are fixed.
    pub(crate) fixed_parents: bool,
    /// The stamp of the capture.
    pub(crate) captured_at: u64,
    /// What the merge,
    /// cherry-pick or revert state named,
    /// nothing when the preparation concluded none.
    pub(crate) conclusion_state: Option<String>,
}

/// Builders of a `Ready` from a shape.
impl ReadyShape {
    /// The inputs of the native Git run for an invocation capture and the arguments after the policies' transforms.
    pub(crate) fn native_inputs(&self, arguments: Vec<OsString>, captured: InvocationCapture) -> NativeInputs {
        return NativeInputs {
            arguments,
            pathspecs: self.pathspecs.clone(),
            mode: self.mode,
            staged_all: self.staged_all,
            selected_privately: self.selected_privately,
            commit_index: self.commit_index.clone(),
            capture: captured,
        };
    }

    /// A preparation that Git builds:
    /// the `Ready` that needs the native run.
    pub(crate) fn by_git(&self, taken: InvocationCapture, events: Vec<PolicyEvent>, changes: Option<Vec<Change>>, records: Vec<AddedPath>, arguments: Vec<OsString>, scratch_dir: Scratch) -> Box<Ready> {
        return Box::new(Ready {
            repository: taken.layout.repository_root.clone(),
            common_dir: taken.layout.common_dir.clone(),
            reference: self.reference.clone(),
            base: self.announced_base.clone(),
            changes,
            events,
            uses_lfs: taken.layout.config_snapshot.as_ref().is_none_or(uses_lfs),
            scratch: scratch_dir,
            needs_native: true,
            inputs: self.native_inputs(arguments, taken),
            worktree_records: records,
            cleanup: self.cleanup.clone(),
            interactive: self.interactive,
            fixed_parents: self.fixed_parents,
            captured_at: self.captured_at,
            committed_conflicted: Vec::new(),
            leaves_open: None,
            conclusion_state: self.conclusion_state.clone(),
        });
    }
}

/// The first commit on the branch since the preparation base that changed one of the conflicting paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedFirst {
    /// Its abbreviated name,
    /// as Git abbreviates it in this repository.
    pub short: String,
    /// Its subject.
    pub subject: String,
}

/// A retry that could not combine the captured change with the branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayConflict {
    /// The conflicting paths in Git's byte order,
    /// as the exact bytes Git reported.
    pub paths: Vec<Vec<u8>>,
    /// The commit that landed first on them,
    /// when it could be found.
    pub landed: Option<LandedFirst>,
    /// The Git directory of the worktree,
    /// where a saved message is kept.
    pub git_dir: PathBuf,
}

/// The scratch directory and its two controls stay out of the release executable.
#[cfg(test)]
#[path = "service_prepare_ready_shapes_tests.rs"]
mod tests;
