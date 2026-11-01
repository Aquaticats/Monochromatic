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
use super::candidate_error::CandidateError;
/// Candidate preparation on a private index.
use super::candidate_prediction::{CandidateRequest, PreparedCandidates, prepare_candidates};
/// The captured-query runner and its byte helpers.
use super::git_metadata::{MetadataOutput, path_from_git_bytes, run_metadata_git};
use super::repository_location::{
    RepositoryLocation, location_query_arguments, parse_location_output,
};
use super::rule_branch_worktree::{remote_guess_creates_branch, remote_guess_query_arguments};
use super::rule_commit_index::{IndexVsHead, index_query_arguments, index_state_from_exit};
use super::rule_worktree_dirty::{DirtyScope, dirty_from_output, dirty_query_arguments};
use super::rule_commit_sequencer::{
    SequencerFacts, SequencerState, sequencer_head_paths, sequencer_query_arguments,
    sequencer_state, sequencer_state_when_query_fails,
};
/// The transaction Git context.
use super::transaction_git::GitContext;
/// The common directory of a location's identity.
use super::worktree_identity::common_directory;
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
    /// Whether the worktree holds uncommitted state of the given scope.
    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String>;
    /// Whether a merge,
    ///  cherry-pick or revert awaits its concluding commit.
    fn sequencer_state(&mut self) -> Result<SequencerState, String>;
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
    global_prefix: Vec<OsString>,
    /// Variables added to every Git child.
    overlay: Vec<(OsString, OsString)>,
    /// The remembered location answer,
    ///  once asked.
    location: Option<Result<RepositoryLocation, String>>,
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
/// const message = `cli-git could not start ${realGit} to read ${fact}: ${error}`;
/// ```
fn not_started(real_git: &Path, fact: &str, error: &std::io::Error) -> String {
    // `format!` builds owned text; `.display()` shows a path, replacing undecodable bytes.
    return format!(
        "cli-git could not start {} to read {fact}: {error}",
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
    fn query(&mut self, arguments: Vec<OsString>, fact: &str) -> Result<MetadataOutput, String> {
        self.queries += 1;
        // `match` unpacks the start result; `Ok`/`Err` are the success/failure variants.
        match run_metadata_git(
            self.real_git.as_path(),
            arguments.as_slice(),
            self.overlay.as_slice(),
        ) {
            Ok(output) => return Ok(output),
            // `&error` borrows the error for the message.
            Err(error) => return Err(not_started(self.real_git.as_path(), fact, &error)),
        }
    }
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
        let arguments: Vec<OsString> = location_query_arguments(self.global_prefix.as_slice());
        let answer: Result<RepositoryLocation, String> =
            match self.query(arguments, "the repository location") {
                Ok(output) => read_location(&output),
                Err(message) => Err(message),
            };
        // `Some(x)` is the "present" case of `Option`.
        self.location = Some(answer.clone());
        return answer;
    }

    fn index_vs_head(&mut self) -> Result<IndexVsHead, String> {
        let arguments: Vec<OsString> = index_query_arguments(self.global_prefix.as_slice());
        // A trailing `?` returns the failure to our caller, or unwraps the output.
        let output: MetadataOutput = self.query(arguments, "the index state")?;
        return Ok(index_state_from_exit(output.code));
    }

    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String> {
        let arguments: Vec<OsString> = dirty_query_arguments(self.global_prefix.as_slice(), scope);
        let output: MetadataOutput = self.query(arguments, "the worktree status")?;
        return dirty_from_output(&output);
    }

    fn sequencer_state(&mut self) -> Result<SequencerState, String> {
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

    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>> {
        return self.location.clone();
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
///  Read the location answer,
///  turning an unreadable one into a message.
///       `&MetadataOutput` borrows the captured query result.
/// Why:
///   Callers of the facts interface report failures as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return parseLocationOutput(output.success, output.stdout); } catch (e) { throw String(e); }
/// ```
fn read_location(output: &MetadataOutput) -> Result<RepositoryLocation, String> {
    match parse_location_output(output.success, output.stdout.as_slice()) {
        Ok(location) => return Ok(location),
        // `.to_string()` renders the input/output error as owned text.
        Err(error) => return Err(error.to_string()),
    }
}

/// What:
///  Whether the file at the path Git printed exists,
///  without following a final link.
///       `&[u8]` borrows the path bytes.
/// Why:
///   Git itself tests these head files with `lstat`,
///  so a dangling link counts as
///       present here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exists = await lstat(path).then(() => true, () => false);
/// ```
fn head_file_exists(path: &[u8]) -> bool {
    // `match` unpacks "a path or nothing": empty bytes are not a path.
    match path_from_git_bytes(path) {
        // `.is_ok()` is true when the file's metadata could be read.
        Some(found) => return std::fs::symlink_metadata(found).is_ok(),
        None => return false,
    }
}

/// Query-count and disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_facts_tests.rs"]
mod tests;
