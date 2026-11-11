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
use crate::candidate_prediction::{CandidateRequest, PreparedCandidates, prepare_candidates};
/// The captured-query runner and its byte helpers.
use crate::git_metadata::{MetadataOutput, path_from_git_bytes, read_stopped_text, run_metadata_git};
use crate::repo_read_adapter::{location_of, sequencer_of};
use crate::git_system_config::with_real_git;
use crate::kept_names_in_the_git_directories::settle_kept_names;
use crate::repo_read_layout::{RepoLayout, read_layout};
/// The one query for the main checkout,
///  which a linked-worktree refusal names.
use crate::repo_read_main_worktree::{main_worktree_from_answer, main_worktree_query_arguments};
use crate::repository_location::{
    RepositoryLocation, location_query_arguments, parse_location_output,
};
use crate::rule_branch_worktree::{remote_guess_creates_branch, remote_guess_query_arguments};
use crate::rule_commit_index::{IndexVsHead, index_query_arguments, index_state_from_exit};
use crate::command_clean::clean_deleted_paths_from_answer;
/// The reading of a dry run written with one force,
///  which names the nested repositories that force keeps.
/// The answer one `git add` selection question gives.
use crate::rule_add_explicit::StagedSelection;
use crate::rule_worktree_dirty::{DirtyScope, dirty_from_output, dirty_query_arguments};
use crate::rule_repository_stash::{stash_from_output, stash_query_arguments};
use crate::rule_commit_sequencer::{
    SequencerFacts, SequencerState, sequencer_head_paths, sequencer_query_arguments,
    sequencer_state, sequencer_state_when_query_fails,
};
/// The transaction Git context.
use crate::transaction_git::GitContext;
/// The common directory of a location's identity.
use crate::worktree_identity::{common_directory, git_directory};
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
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

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
    fn staged_selection(&mut self, pathspecs: Vec<OsString>) -> Result<StagedSelection, String>;
    /// Whether the worktree holds uncommitted state of the given scope.
    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String>;
    /// The worktree's ignored files whose paths the given revision's tree also tracks,
    /// asked for a
    ///  forced checkout,
    ///  a forced switch and a hard reset.
    /// A revision Git cannot resolve falls back to `HEAD`,
    /// because every form that
    ///  names no tree writes `HEAD`'s content into the worktree;
    /// an unreadable `HEAD` is a failure,
    ///  never an empty answer.
    fn ignored_files_the_tree_tracks(&mut self, revision: Option<&OsStr>) -> Result<Vec<String>, String>;
    /// Whether the repository holds a stash entry.
    /// Asked for `git stash drop` and `git stash clear`,
    /// which delete entries of the one stash every worktree shares.
    fn repository_holds_a_stash(&mut self) -> Result<bool, String>;
    /// The paths the given `git clean --dry-run` argument list would delete,
    /// each as Git printed it,
    /// asked for a clean
    ///  that carries its own `-e`/`--exclude` patterns.
    fn clean_deleted_paths(&mut self, arguments: Vec<OsString>) -> Result<Vec<Vec<u8>>, String>;
    /// Whether a merge,
    ///  cherry-pick or revert awaits its concluding commit.
    fn sequencer_state(&mut self) -> Result<SequencerState, String>;
    /// The directory of the main checkout,
    ///  the worktree that owns the configuration file every worktree of the
    /// repository shares.
    /// Asked by a refusal in a linked worktree,
    ///  which must name where the setting belongs.
    fn main_worktree_root(&mut self) -> Result<PathBuf, String>;
    /// Whether `git switch <target>` or `git checkout <target>` would create a local
    /// branch from the one remote branch of that name.
    fn remote_guess_creates_branch(&mut self, target: &OsStr) -> Result<bool, String>;
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
///  The provider that asks real Git.
///  A `struct` is a record with named fields;
///       `Option<T>` is "a value or nothing";
///  `usize` is the unsigned integer of counts.
/// Why:
///   It owns copies of the executable path,
///  the caller's global options and the child
///       environment additions,
///  so it can be handed around without borrowing rules.
///  The
///       query count lets a test prove how many Git processes an invocation started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitFacts = { realGit: string; globalPrefix: string[]; overlay: [string, string][]; location?: Result; queries: number };
/// ```
#[derive(Clone, Debug)]
pub struct GitFacts {
    /// The selected real Git executable.
    real_git: PathBuf,
    /// The arguments before the subcommand,
    ///  without wrapper controls.
    pub(super) global_prefix: Vec<OsString>,
    /// Variables added to every Git child.
    overlay: Vec<(OsString, OsString)>,
    /// The remembered location answer,
    ///  once asked.
    location: Option<Result<RepositoryLocation, String>>,
    /// The in-process layout,
    /// read once:
    /// `Some(None)` means the reader declined and Git answers.
    layout: Option<Option<RepoLayout>>,
    /// How many Git processes this provider started.
    pub queries: usize,
}

/// What:
///  Build a provider that has asked nothing yet.
///  `&Path` and `&[...]` borrow the
///       caller's values;
///  the provider keeps its own copies.
/// Why:
///   Creating a provider costs no process;
///  only a rule that needs a fact pays for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitFacts(realGit: string, globalPrefix: string[], overlay: [string, string][]): GitFacts;
/// ```
pub fn git_facts(
    real_git: &Path,
    global_prefix: &[OsString],
    overlay: &[(OsString, OsString)],
) -> GitFacts {
    return GitFacts {
        // `.to_path_buf()` and `.to_vec()` copy the borrowed values into owned ones.
        real_git: real_git.to_path_buf(),
        global_prefix: global_prefix.to_vec(),
        overlay: overlay.to_vec(),
        // `None` is the "absent" case of `Option`: nothing has been asked.
        location: None,
        layout: None,
        queries: 0,
    };
}

/// What:
///  Say that real Git could not be started for a fact.
///  `&std::io::Error` borrows the
///       operating system's error.
/// Why:
///   The message names the executable and the fact,
///  so the reader knows what failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const message = `monochromatic-git-policy-cli could not start ${realGit} to read ${fact}: ${error}`;
/// ```
fn not_started(real_git: &Path, fact: &str, error: &std::io::Error) -> String {
    // `format!` builds owned text; `.display()` shows a path, replacing undecodable bytes.
    return format!(
        "monochromatic-git-policy-cli could not start {} to read {fact}: {error}",
        real_git.display()
    );
}

/// Private helpers of the real-Git provider.
impl GitFacts {
    /// What:
    ///  Run one captured Git query after the caller's global options and count it.
    ///       `&mut self` lends the provider for writing;
    ///  `Vec<OsString>` is the owned
    ///       argument list,
    ///  already starting with the global options.
    /// Why:
    ///   Every fact goes through here,
    ///  so the count cannot miss a process.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async query(args: string[], fact: string): Promise<MetadataOutput>;
    /// ```
    pub(super) fn query(&mut self, arguments: Vec<OsString>, fact: &str) -> Result<MetadataOutput, String> {
        self.queries += 1;
        // `match` unpacks the start result; `Ok`/`Err` are the success/failure variants.
        match run_metadata_git(
            self.real_git.as_path(),
            arguments.as_slice(),
            self.overlay.as_slice(),
        ) {
            Ok(output) => return Ok(output),
            // A read the wrapper ended at its ceiling says what blocked it and what stayed as it was.
            // `&error` borrows the error for the message.
            Err(error) => {
                return Err(match read_stopped_text(fact, &error) {
                    Some(stopped) => stopped,
                    None => not_started(self.real_git.as_path(), fact, &error),
                });
            }
        }
    }

    /// What:
    ///  The repository layout read from the file system,
    /// once per provider;
    ///  `None` when the reader declined and Git must answer.
    /// Why:
    ///   Location and sequencer questions share one read and start no Git process when the layout is plain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// layout(): RepoLayout | undefined; // memoized
    /// ```
    fn layout(&mut self) -> Option<&RepoLayout> {
        if self.layout.is_none() {
            self.layout = Some(read_layout(self.global_prefix.as_slice(), with_real_git(self.overlay.as_slice(), self.real_git.as_path()).as_slice()));
            // The first time the Git directories are known, a name an earlier release kept in them is moved to its current name.
            if let Some(Some(known)) = &self.layout {
                settle_kept_names(known.git_dir.as_path(), known.common_dir.as_path());
            }
        }
        return self.layout.as_ref().and_then(Option::as_ref);
    }

}

/// What:
///  The text one unreadable location answer is reported as.
///  `std::io::Error` is the operating system's own error.
/// Why:
///   Callers of the facts interface report failures as text,
///  and a named function keeps the conversion out of a closure the linter cannot keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errorAsMessage = (error: Error) => error.message;
/// ```
fn error_as_message(error: std::io::Error) -> String {
    return error.to_string();
}

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
            settle_kept_names(git_dir, common_directory(&found.identity).unwrap_or(git_dir));
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

    fn staged_selection(&mut self, pathspecs: Vec<OsString>) -> Result<StagedSelection, String> {
        return crate::repository_facts_staged_selection::staged_selection(self, pathspecs);
    }

    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String> {
        let arguments: Vec<OsString> = dirty_query_arguments(self.global_prefix.as_slice(), scope);
        let output: MetadataOutput = self.query(arguments, "the worktree status")?;
        return dirty_from_output(&output);
    }

    fn repository_holds_a_stash(&mut self) -> Result<bool, String> {
        let arguments: Vec<OsString> = stash_query_arguments(self.global_prefix.as_slice());
        let output: MetadataOutput = self.query(arguments, "the repository stash")?;
        return stash_from_output(&output);
    }

    fn clean_deleted_paths(&mut self, arguments: Vec<OsString>) -> Result<Vec<Vec<u8>>, String> {
        let output: MetadataOutput = self.query(arguments, "what the clean would delete")?;
        return clean_deleted_paths_from_answer(&output);
    }

    fn ignored_files_the_tree_tracks(&mut self, revision: Option<&OsStr>) -> Result<Vec<String>, String> {
        // The query and its reading live beside this provider, as the `git add` fact's do.
        return crate::repository_facts_ignored_files::ignored_files_the_tree_tracks(self, revision);
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
        let arguments: Vec<OsString> =
            main_worktree_query_arguments(self.global_prefix.as_slice());
        let output: MetadataOutput = self.query(arguments, "the worktrees of the repository")?;
        return main_worktree_from_answer(&output);
    }

    fn remote_guess_creates_branch(&mut self, target: &OsStr) -> Result<bool, String> {
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

/// What:
///  Whether the file at the path Git printed exists,
///  without following a final link.
///       `&[u8]` borrows the path bytes.
/// Why:
///   Git itself tests these head files with `lstat`,
///  so a dangling link counts as
///       present here too,
///  and the ref-storage reader already answers the same question one `Path` at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exists = (path: Buffer) => { const found = pathFromGitBytes(path); return found !== undefined && somethingIsAt(found); };
/// ```
fn head_file_exists(path: &[u8]) -> bool {
    // `match` unpacks "a path or nothing": empty bytes are not a path.
    return match path_from_git_bytes(path) {
        Some(found) => crate::sequencer_marker::something_is_at(found.as_path()),
        None => false,
    };
}

/// Query-count and disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_facts_tests.rs"]
mod tests;
