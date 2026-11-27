//! What:
//!  The repository facts policies and fixed transforms need,
//!  behind one small
//!       interface,
//!  with the provider that asks real Git.
//! Why:
//!  The rule cores decide from arguments alone and name the fact they still need.
//!      This module fetches those facts lazily and at most once:
//!  an invocation whose rules
//!      need nothing starts no Git process,
//!  and several rules that need the repository's
//!      location share one query.
//!  Tests drive the rules with a scripted provider.
//!  The provider that asks real Git holds its own values and one query in
//!  `repository_facts_git_provider.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const facts = createGitFacts({ gitPath, globalPrefix }); await facts.location();
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Each fact has a pure core that builds its query and reads its answer;
///  this module
///       only runs the query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { indexQueryArguments, indexStateFromExit } from './rule_commit_index.ts';
/// ```
use crate::candidate_error::CandidateError;
/// Candidate preparation on a private index.
use crate::candidate_prediction::{
    CandidateRequest,
    PreparedCandidates,
    prepare_candidates,
};
use crate::command_clean::clean_deleted_paths_from_answer;
/// The captured-query runner and its byte helpers.
use crate::git_metadata::MetadataOutput;
use crate::kept_names_in_the_git_directories::settle_kept_names;
use crate::repo_read_adapter::{
    location_of,
    sequencer_of,
};
/// The one query for the main checkout,
///  which a linked-worktree refusal names.
use crate::repo_read_main_worktree::{
    main_worktree_from_answer,
    main_worktree_query_arguments,
};
pub use crate::repository_facts_git_provider::GitFacts;
/// The reading of a dry run written with one force,
///  which names the nested repositories that force keeps.
/// The answer one `git add` selection question gives.
use crate::repository_facts_git_provider::error_as_message;
pub use crate::repository_facts_git_provider::git_facts;
use crate::repository_location::{
    RepositoryLocation,
    location_query_arguments,
    parse_location_output,
};
use crate::rule_add_explicit::StagedSelection;
use crate::rule_branch_worktree::{
    remote_guess_creates_branch,
    remote_guess_query_arguments,
};
use crate::rule_commit_index::{
    IndexVsHead,
    index_query_arguments,
    index_state_from_exit,
};
use crate::rule_commit_sequencer::{
    SequencerFacts,
    SequencerState,
    sequencer_head_paths,
    sequencer_query_arguments,
    sequencer_state,
    sequencer_state_when_query_fails,
};
use crate::rule_repository_stash::{
    stash_from_output,
    stash_query_arguments,
};
use crate::rule_worktree_dirty::{
    DirtyScope,
    dirty_from_output,
    dirty_query_arguments,
};
use crate::sequencer_marker::head_file_exists;
/// The interface every repository-fact provider answers,
///  which lives in a file of its own so that the readings it names can use it in turn.
pub use crate::the_facts_a_repository_reads::RepositoryFacts;
/// The transaction Git context.
use crate::transaction_git::GitContext;
/// The common directory of a location's identity.
use crate::worktree_identity::{
    common_directory,
    git_directory,
};
/// What one abort form would discard,
/// and the forms that ask.
use crate::{
    repository_facts_abort_discard::AbortDiscard,
    rule_linked_worktree::GuardedCommand,
};
/// What:
///  `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
///  Sibling
///       the reader might expect:
///  `&str`/`String`,
///  which must be valid UTF-8.
/// Why:
///   Global options and branch names are passed to Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::{
    OsStr,
    OsString,
};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  `impl RepositoryFacts for GitFacts` provides the interface's methods for the
///       real-Git provider.
/// Why:
///   Each method is one query whose arguments and reading belong to a pure core.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GitFacts implements RepositoryFacts { /* ... */ }
/// ```
impl RepositoryFacts for GitFacts {
    fn location(&mut self) -> Result<RepositoryLocation, String> {
        // `if let Some(known) = &self.location` borrows the remembered answer when present.
        if let Some(known) = &self.location {
            // `.clone()` copies the remembered answer for the caller.
            return known.clone();
        }
        // A plain layout is read from the file system; only a layout the reader declines costs a Git process.
        let read: Option<RepositoryLocation> = self.layout().map(location_of);
        if let Some(found) = read {
            self.location = Some(Ok(found.clone()));
            return Ok(found);
        }
        let arguments: Vec<OsString> = location_query_arguments(self.global_prefix.as_slice());
        let answer: Result<RepositoryLocation, String> =
            match self.query(arguments, "the repository location") {
                // An unreadable answer becomes the message the caller reports.
                Ok(output) => parse_location_output(output.success, output.stdout.as_slice())
                    .map_err(error_as_message),
                Err(message) => Err(message),
            };
        // `Some(x)` is the "present" case of `Option`.
        // Git answered because the layout reader declined: the same settling as for a layout read from disk.
        if let Ok(found) = &answer
            && let Some(git_dir) = git_directory(&found.identity)
        {
            settle_kept_names(
                git_dir,
                common_directory(&found.identity).unwrap_or(git_dir),
            );
        }
        self.location = Some(answer.clone());
        return answer;
    }

    fn index_vs_head(&mut self) -> Result<IndexVsHead, String> {
        let arguments: Vec<OsString> = index_query_arguments(self.global_prefix.as_slice());
        // A trailing `?` returns the failure to our caller, or unwraps the output.
        let output: MetadataOutput = self.query(arguments, "the index state")?;
        return Ok(index_state_from_exit(output.code));
    }

    fn staged_selection(
        &mut self,
        pathspecs: Vec<OsString>,
    ) -> Result<StagedSelection, String> {
        return crate::repository_facts_staged_selection::staged_selection(self, pathspecs);
    }

    fn worktree_dirty(
        &mut self,
        scope: DirtyScope,
    ) -> Result<bool, String> {
        let arguments: Vec<OsString> = dirty_query_arguments(self.global_prefix.as_slice(), scope);
        let output: MetadataOutput = self.query(arguments, "the worktree status")?;
        return dirty_from_output(&output);
    }

    fn repository_holds_a_stash(&mut self) -> Result<bool, String> {
        let arguments: Vec<OsString> = stash_query_arguments(self.global_prefix.as_slice());
        let output: MetadataOutput = self.query(arguments, "the repository stash")?;
        return stash_from_output(&output);
    }

    fn clean_deleted_paths(
        &mut self,
        arguments: Vec<OsString>,
    ) -> Result<Vec<Vec<u8>>, String> {
        let output: MetadataOutput = self.query(arguments, "what the clean would delete")?;
        return clean_deleted_paths_from_answer(&output);
    }

    fn ignored_files_the_tree_tracks(
        &mut self,
        revision: Option<&OsStr>,
    ) -> Result<Vec<String>, String> {
        // The query and its reading live beside this provider, as the `git add` fact's do.
        return crate::repository_facts_ignored_files::ignored_files_the_tree_tracks(
            self, revision,
        );
    }

    fn untracked_files_the_tree_tracks(
        &mut self,
        revision: &OsStr,
    ) -> Result<Vec<String>, String> {
        return crate::repository_facts_untracked_files::untracked_files_the_tree_tracks(
            self, revision,
        );
    }

    fn sequencer_state(&mut self) -> Result<SequencerState, String> {
        if let Some(state) = self.layout().and_then(sequencer_of) {
            return Ok(state);
        }
        let arguments: Vec<OsString> = sequencer_query_arguments(self.global_prefix.as_slice());
        let output: MetadataOutput = self.query(arguments, "the Git directory")?;
        if !output.success {
            return Ok(sequencer_state_when_query_fails());
        }
        let paths: Vec<Vec<u8>> = match sequencer_head_paths(output.stdout.as_slice()) {
            Ok(found) => found,
            Err(error) => return Err(error.to_string()),
        };
        return Ok(sequencer_state(SequencerFacts {
            merge_head: head_file_exists(paths[0].as_slice()),
            cherry_pick_head: head_file_exists(paths[1].as_slice()),
            revert_head: head_file_exists(paths[2].as_slice()),
        }));
    }

    fn main_worktree_root(&mut self) -> Result<PathBuf, String> {
        // `--porcelain -z` prints the main worktree's path on the first line, whatever its bytes are.
        let arguments: Vec<OsString> = main_worktree_query_arguments(self.global_prefix.as_slice());
        let output: MetadataOutput = self.query(arguments, "the worktrees of the repository")?;
        return main_worktree_from_answer(&output);
    }

    fn abort_discard(
        &mut self,
        command: GuardedCommand,
    ) -> Result<AbortDiscard, String> {
        return crate::repository_facts_abort_discard::abort_discard(self, command);
    }

    fn remote_guess_creates_branch(
        &mut self,
        target: &OsStr,
    ) -> Result<bool, String> {
        let arguments: Vec<OsString> =
            remote_guess_query_arguments(self.global_prefix.as_slice(), target);
        let output: MetadataOutput = self.query(arguments, "branch names")?;
        if !output.success {
            return Ok(false);
        }
        // `.as_encoded_bytes()` lends the raw bytes of the branch name.
        return Ok(remote_guess_creates_branch(
            output.stdout.as_slice(),
            target.as_encoded_bytes(),
        ));
    }

    /// Prepare the candidates with this provider's Git,
    ///  prefix and environment.
    fn candidates(
        &mut self,
        request: &CandidateRequest,
    ) -> Result<PreparedCandidates, CandidateError> {
        // The same Git, prefix and environment as every other question, so the prediction
        // looks at the repository the command will change.
        return prepare_candidates(
            self.real_git.as_path(),
            self.global_prefix.as_slice(),
            self.overlay.as_slice(),
            request,
        );
    }

    /// What:
    ///  The location when it was already asked for,
    ///  without asking.
    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>> {
        return self.location.clone();
    }

    fn known_common_dir(&mut self) -> Option<PathBuf> {
        if let Some(Ok(found)) = &self.location {
            return common_directory(&found.identity).map(Path::to_path_buf);
        }
        // A plain layout is read from the file system; a layout the reader declines is not worth a Git process here.
        let found = self.layout()?;
        return Some(found.common_dir.clone());
    }

    fn known_git_dir(&mut self) -> Option<PathBuf> {
        if let Some(Ok(found)) = &self.location {
            return git_directory(&found.identity).map(Path::to_path_buf);
        }
        let found = self.layout()?;
        return Some(found.git_dir.clone());
    }

    fn transaction_context(&self) -> GitContext {
        return GitContext {
            real_git: self.real_git.clone(),
            overlay: self.overlay.clone(),
            global_prefix: self.global_prefix.clone(),
            // The repository is known once its location was asked; a context made before that has none.
            common_dir: match &self.location {
                Some(Ok(found)) => common_directory(&found.identity).map(Path::to_path_buf),
                Some(Err(_)) | None => None,
            },
        };
    }
}

/// Query-count and disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_facts_tests.rs"]
mod tests;
