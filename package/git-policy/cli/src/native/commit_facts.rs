//! What:
//!  The repository facts of a commit transaction:
//!  every rule fact is read from the real repository, and the candidate files are read from the private index
//!  and the shadow object store against the recorded base.
//! Why:
//!  The incumbent's rules (`commit-only`, `commit-index`, `commit-sequencer`) ask real Git about the real index and
//!  the real sequencer state, while the content policies read the private commit index that the transaction corrects.
//!  Pointing every fact at the private index would answer the rules about a state the user never staged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = { ...realRepositoryFacts, ...createPrivateIndexFacts({ indexPath, paths, baseRevision, objectDirectory }) };
//! ```

/// Errors of the candidate layer.
use super::candidate_error::CandidateError;
/// The prediction entry point and the request kinds.
use super::candidate_prediction::{CandidateRequest, PreparedCandidates, prepare_candidates};
/// Rule facts.
use super::repository_facts::RepositoryFacts;
use super::repository_location::RepositoryLocation;
use super::rule_commit_index::IndexVsHead;
use super::rule_commit_sequencer::SequencerState;
use super::rule_worktree_dirty::DirtyScope;
use super::transaction_git::GitContext;
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// The variable that names the index a Git child reads.
const INDEX_FILE_VARIABLE: &str = "GIT_INDEX_FILE";
/// The variable that names the object store a Git child writes to.
const OBJECT_DIRECTORY_VARIABLE: &str = "GIT_OBJECT_DIRECTORY";

/// What:
///  Rule facts from the wrapped real repository and candidate reads from a private index.
/// Why:
///   The private index changes from one replay to the next, so the facts keep it as a replaceable value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitFacts = { rules: RepositoryFacts; privateIndex: { indexPath: string; objectDirectory: string } };
/// ```
pub struct PrivateIndexFacts<F: RepositoryFacts> {
    /// The facts every rule reads.
    inner: F,
    /// `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY` of the candidate reads.
    private: Vec<(OsString, OsString)>,
}

impl<F: RepositoryFacts> PrivateIndexFacts<F> {
    /// What:
    ///  Wrap rule facts with the private index and the object store the candidates are read from.
    /// Why:
    ///   The candidate prediction copies the named index, so reading never changes the transaction's index.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// createPrivateIndexFacts({ indexPath, objectDirectory });
    /// ```
    pub fn new(inner: F, index: &Path, object_directory: &Path) -> Self {
        return PrivateIndexFacts {
            inner,
            private: private_entries(index, object_directory),
        };
    }

    /// What:
    ///  Read the candidates from another private index from now on.
    /// Why:
    ///   A replay validates its own merged tree in `replay-<n>/commit.index`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// facts = createPrivateIndexFacts({ indexPath: replayIndexPath, objectDirectory });
    /// ```
    pub fn use_index(&mut self, index: &Path, object_directory: &Path) {
        self.private = private_entries(index, object_directory);
    }
}

/// What:
///  The two child variables that select a private index and its object store.
/// Why:
///   Both candidate reads and corrections must name the same pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ GIT_INDEX_FILE: indexPath, GIT_OBJECT_DIRECTORY: objectDirectory })
/// ```
fn private_entries(index: &Path, object_directory: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from(INDEX_FILE_VARIABLE), index.as_os_str().to_os_string()),
        (
            OsString::from(OBJECT_DIRECTORY_VARIABLE),
            object_directory.as_os_str().to_os_string(),
        ),
    ];
}

impl<F: RepositoryFacts> RepositoryFacts for PrivateIndexFacts<F> {
    fn location(&mut self) -> Result<RepositoryLocation, String> {
        return self.inner.location();
    }

    fn index_vs_head(&mut self) -> Result<IndexVsHead, String> {
        return self.inner.index_vs_head();
    }

    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String> {
        return self.inner.worktree_dirty(scope);
    }

    fn sequencer_state(&mut self) -> Result<SequencerState, String> {
        return self.inner.sequencer_state();
    }

    fn remote_guess_creates_branch(&mut self, target: &OsStr) -> Result<bool, String> {
        return self.inner.remote_guess_creates_branch(target);
    }

    fn candidates(&mut self, request: &CandidateRequest) -> Result<PreparedCandidates, CandidateError> {
        let context: GitContext = self.inner.transaction_context();
        let mut overlay: Vec<(OsString, OsString)> = context.overlay.clone();
        overlay.extend(self.private.iter().cloned());
        return prepare_candidates(
            context.real_git.as_path(),
            context.global_prefix.as_slice(),
            overlay.as_slice(),
            request,
        );
    }

    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>> {
        return self.inner.asked_location();
    }

    fn transaction_context(&self) -> GitContext {
        return self.inner.transaction_context();
    }
}

/// A borrowed provider answers like the provider it borrows.
impl<F: RepositoryFacts> RepositoryFacts for &mut F {
    fn location(&mut self) -> Result<RepositoryLocation, String> {
        return (**self).location();
    }

    fn index_vs_head(&mut self) -> Result<IndexVsHead, String> {
        return (**self).index_vs_head();
    }

    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String> {
        return (**self).worktree_dirty(scope);
    }

    fn sequencer_state(&mut self) -> Result<SequencerState, String> {
        return (**self).sequencer_state();
    }

    fn remote_guess_creates_branch(&mut self, target: &OsStr) -> Result<bool, String> {
        return (**self).remote_guess_creates_branch(target);
    }

    fn candidates(&mut self, request: &CandidateRequest) -> Result<PreparedCandidates, CandidateError> {
        return (**self).candidates(request);
    }

    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>> {
        return (**self).asked_location();
    }

    fn transaction_context(&self) -> GitContext {
        return (**self).transaction_context();
    }
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_facts_tests.rs"]
mod tests;
