//! What:
//!  A scripted provider of repository facts for unit tests of checks,
//!  transforms and
//!       lifecycles.
//! Why:
//!  Those modules decide from facts;
//!  a script supplies each fact directly and logs
//!      which ones were asked for,
//!  so a test can prove a command asked for nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const facts = scriptedFacts(); facts.location = mainWorktree('/r', 'sub/');
//! ```

/// The facts interface and the fact types the script answers with.
use crate::candidate_error::{CandidateError, CandidateFailure};
/// Real preparation,
///  for scripted facts given a repository.
use crate::candidate_prediction::{CandidateRequest, PreparedCandidates, prepare_candidates};
use crate::config_schema::PolicyConfig;
use crate::policy_registry::{PolicyId, Severity};
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::RepositoryLocation;
use crate::rule_add_explicit::StagedSelection;
use crate::rule_commit_index::IndexVsHead;
use crate::rule_commit_sequencer::SequencerState;
use crate::rule_worktree_dirty::DirtyScope;
/// The image's real Git,
///  which prepares candidates for scripted facts.
use crate::test_support::REAL_GIT;
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
/// Branch names and global options are operating-system text.
use std::ffi::{OsStr, OsString};
/// Locations and repositories are paths.
use std::path::{Path, PathBuf};

/// A provider that answers from fields and logs every fact it was asked for.
#[derive(Clone, Debug)]
pub(crate) struct ScriptedFacts {
    /// The answer to `location`.
    pub(crate) location: Result<RepositoryLocation, String>,
    /// The answer to `index_vs_head`.
    pub(crate) index: Result<IndexVsHead, String>,
    /// The answer to `sequencer_state`.
    pub(crate) sequencer: Result<SequencerState, String>,
    /// The answer to `worktree_dirty`,
    /// whatever the scope.
    pub(crate) dirty: Result<bool, String>,
    /// The answer to `staged_selection`,
    /// whatever the pathspecs.
    pub(crate) selection: Result<StagedSelection, String>,
    /// The answer to `ignored_files_the_tree_tracks`.
    pub(crate) ignored_and_tracked: Result<Vec<String>, String>,
    /// The answer to `clean_would_delete`.
    pub(crate) clean_deletes: Result<bool, String>,
    /// The answer to `repository_holds_a_stash`.
    pub(crate) stash: Result<bool, String>,
    /// The answer to `remote_guess_creates_branch`.
    pub(crate) remote_guess: Result<bool, String>,
    /// Every fact asked for,
    ///  in call order;
    ///  the location is logged once,
    ///  and a remote
    /// guess with its target.
    pub(crate) asked: Vec<String>,
    /// The repository real Git prepares candidates in;
    ///  none makes preparation fail.
    pub(crate) candidates_repository: Option<PathBuf>,
    /// The invocation's own Git directory,
    ///  where a fix keeps its journals;
    ///  none is the answer of a provider that knows
    ///       neither the location nor the file system.
    pub(crate) git_dir: Option<PathBuf>,
}

impl RepositoryFacts for ScriptedFacts {
    fn location(&mut self) -> Result<RepositoryLocation, String> {
        // The real provider remembers the location, so a repeated question starts no
        // process; the script logs the question once for the same reason.
        let name: String = String::from("location");
        if !self.asked.contains(&name) {
            self.asked.push(name);
        }
        return self.location.clone();
    }

    fn index_vs_head(&mut self) -> Result<IndexVsHead, String> {
        self.asked.push(String::from("index"));
        return self.index.clone();
    }

    fn worktree_dirty(&mut self, scope: DirtyScope) -> Result<bool, String> {
        self.asked.push(format!("dirty:{scope:?}"));
        return self.dirty.clone();
    }

    fn staged_selection(&mut self, pathspecs: Vec<OsString>) -> Result<StagedSelection, String> {
        self.asked.push(format!(
            "selection:{}",
            pathspecs.join(OsStr::new(" ")).to_string_lossy()
        ));
        return self.selection.clone();
    }

    fn clean_would_delete(&mut self, arguments: Vec<OsString>) -> Result<bool, String> {
        self.asked.push(format!("clean-would-delete:{}", arguments.join(OsStr::new(" ")).to_string_lossy()));
        return self.clean_deletes.clone();
    }

    fn ignored_files_the_tree_tracks(&mut self, revision: Option<&OsStr>) -> Result<Vec<String>, String> {
        self.asked.push(format!(
            "ignored-and-tracked:{}",
            revision.unwrap_or(OsStr::new("HEAD")).to_string_lossy()
        ));
        return self.ignored_and_tracked.clone();
    }

    fn repository_holds_a_stash(&mut self) -> Result<bool, String> {
        self.asked.push(String::from("stash"));
        return self.stash.clone();
    }

    fn sequencer_state(&mut self) -> Result<SequencerState, String> {
        self.asked.push(String::from("sequencer"));
        return self.sequencer.clone();
    }

    fn remote_guess_creates_branch(&mut self, target: &OsStr) -> Result<bool, String> {
        self.asked
            .push(format!("remote-guess:{}", target.to_string_lossy()));
        return self.remote_guess.clone();
    }

    /// Log the question,
    ///  then prepare with real Git in the scripted repository,
    ///  or fail.
    /// Answer with the scripted Git directory,
    ///  so a fix's journals land where a sweep looks for them.
    fn known_git_dir(&mut self) -> Option<PathBuf> {
        return self.git_dir.clone();
    }

    fn candidates(
        &mut self,
        request: &CandidateRequest,
    ) -> Result<PreparedCandidates, CandidateError> {
        self.asked.push(String::from("candidates"));
        // `match` on the scripted repository: real Git prepares there, or nothing can be prepared.
        match &self.candidates_repository {
            Some(directory) => {
                return prepare_candidates(
                    Path::new(REAL_GIT),
                    &[OsString::from("-C"), directory.as_os_str().to_os_string()],
                    &[],
                    request,
                );
            }
            None => {
                return Err(CandidateError::new(
                    CandidateFailure::GitNotStarted,
                    "the scripted facts prepare no candidates",
                ));
            }
        }
    }

    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>> {
        if self.asked.contains(&String::from("location")) {
            return Some(self.location.clone());
        }
        return None;
    }

    fn transaction_context(&self) -> GitContext {
        return GitContext {
            real_git: PathBuf::from(REAL_GIT),
            overlay: Vec::new(),
            global_prefix: Vec::new(),
            common_dir: None,
        };
    }
}

/// The location of a command run in the main worktree rooted at `root`,
///  `prefix` below its top level.
pub(crate) fn main_worktree(root: &str, prefix: &str) -> RepositoryLocation {
    return RepositoryLocation {
        identity: WorktreeIdentity::MainWorktree {
            common_dir: PathBuf::from(format!("{root}/.git")),
            git_dir: PathBuf::from(format!("{root}/.git")),
            worktree_root: PathBuf::from(root),
        },
        prefix: prefix.as_bytes().to_vec(),
    };
}

/// The location of a command run at the top level of a linked worktree at `root` of the repository at `main`.
pub(crate) fn linked_worktree(main: &str, root: &str) -> RepositoryLocation {
    return RepositoryLocation {
        identity: WorktreeIdentity::LinkedWorktree {
            common_dir: PathBuf::from(format!("{main}/.git")),
            git_dir: PathBuf::from(format!("{main}/.git/worktrees/linked")),
            worktree_root: PathBuf::from(root),
        },
        prefix: Vec::<u8>::new(),
    };
}

/// The location of a command run where there is no worktree.
pub(crate) fn outside_worktree() -> RepositoryLocation {
    return RepositoryLocation {
        identity: WorktreeIdentity::OutsideWorktree,
        prefix: Vec::<u8>::new(),
    };
}

/// A script for the top level of the main worktree at `/r`:
///  clean index,
///  no operation in progress,
///  no remote guess.
pub(crate) fn scripted_facts() -> ScriptedFacts {
    return ScriptedFacts {
        location: Ok(main_worktree("/r", "")),
        index: Ok(IndexVsHead::Matches),
        sequencer: Ok(SequencerState::NotInProgress),
        dirty: Ok(false),
        selection: Ok(StagedSelection {
            selected: 0,
            repository: 1,
            directory: 1,
        }),
        stash: Ok(false),
        remote_guess: Ok(false),
        ignored_and_tracked: Ok(Vec::<String>::new()),
        clean_deletes: Ok(false),
        asked: Vec::<String>::new(),
        candidates_repository: None,
        git_dir: None,
    };
}

/// A copy of `config` with one policy's severity replaced.
pub(crate) fn with_severity(
    config: &PolicyConfig,
    policy: PolicyId,
    severity: Severity,
) -> PolicyConfig {
    let mut changed: PolicyConfig = config.clone();
    for setting in &mut changed.settings {
        if setting.id == policy {
            setting.severity = severity;
        }
    }
    return changed;
}
