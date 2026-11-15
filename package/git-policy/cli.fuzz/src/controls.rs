//! What:
//!  Argument vectors built around wrapper controls,
//!  and the invariants of removing
//!       them and of the refusal frontier behind them.
//! Why:
//!  A wrapper control must leave the arguments only where it is an option of its own,
//!      never as a message,
//!  a value or a path,
//!  and what it asked for must be recorded.
//!      Behind the removal,
//!  a commit,
//!  a publishing push,
//!  an unchecked `git add` and a
//!      worktree creation from a linked worktree must never be forwarded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkControlRemoval(controlArguments(data)); checkFrontier(controlArguments(data), data[0]);
//! ```

/// What:
///  `use` brings names from other files into this file;
///  `crate::` is this package and
///       `git_policy_cli::` is the wrapper under test.
/// Why:
///   The checks call the subject's public functions and compare with tables restated here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { stripWrapperControls } from 'git-policy-cli';
/// ```
use crate::control_tables::{CONTROL_TOKENS, TABLE_COMMANDS, VALUELESS_TOKENS};
use git_policy_cli::candidate_error::{CandidateError, CandidateFailure};
use git_policy_cli::candidate_prediction::{CandidateRequest, PreparedCandidates};
use git_policy_cli::command_commit::parse_commit_region;
/// The record of one tool cache directory the policy may exempt,
///  which the shipped checks take as their allowed worktree directories.
use git_policy_cli::effective_target::ToolCacheDirectory;
use git_policy_cli::global_arguments::{GlobalOutcome, global_layout};
/// The production conversion of raw bytes to an argument,
/// so the generators feed the code under test the same way.
use git_policy_cli::path_bytes::os_from_path_bytes as argument;
use git_policy_cli::policy_checks::{ShippedChecks, shipped_checks};
use git_policy_cli::policy_content::LifecycleContent;
use git_policy_cli::policy_registry::{POLICY_REGISTRY, PolicyId};
use git_policy_cli::repository_facts::RepositoryFacts;
/// What one abort form would discard besides the operation's own state.
use git_policy_cli::repository_facts_abort_discard::{AbortDiscard, DiscardBaseline};
use git_policy_cli::repository_location::RepositoryLocation;
/// The operation a guarded form ends, and the commit a skip writes back.
use git_policy_cli::rule_abort_operation::{
    AbortOperation, OperationEnd, SKIP_RESTORES, ended_operation,
};
/// The answer one `git add` selection question gives.
use git_policy_cli::rule_add_explicit::StagedSelection;
use git_policy_cli::rule_commit_index::IndexVsHead;
use git_policy_cli::rule_commit_sequencer::SequencerState;
/// The guarded form whose abort or skip the lifecycle judges.
use git_policy_cli::rule_linked_worktree::GuardedCommand;
use git_policy_cli::rule_worktree_dirty::DirtyScope;
use git_policy_cli::transaction_git::GitContext;
use git_policy_cli::worktree_identity::{WorktreeIdentity, common_directory};
use git_policy_cli::wrapped_command::{WrappedOutcome, run_wrapped_command};
use git_policy_cli::wrapper_controls::{
    ControlMeaning, Controls, control_meaning, is_escaped, no_controls,
};
use git_policy_cli::wrapper_invocation::{StrippedInvocation, strip_wrapper_controls};
/// What:
///  `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
///       Sibling the reader might expect:
///  `&str`/`String`,
///  which must be valid UTF-8.
/// Why:
///   Git arguments are arbitrary bytes and are compared without decoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::{OsStr, OsString};
/// `size_of` is the width of the mode's own type, in bytes.
use std::mem::size_of;
/// `PathBuf` is an owned filesystem path of raw bytes.
use std::path::{Path, PathBuf};

/// What:
///  The spelling of the `git commit` hatch,
///  restated here.
///  `&[u8]` is a borrowed byte
///       string baked into the program.
/// Why:
///   It is the only wrapper token outside the general control table,
///  and restating it
///       keeps the check independent of the subject's constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMMIT_HATCH = '--no-enforce-only';
/// ```
const COMMIT_HATCH: &[u8] = b"--no-enforce-only";

/// The longest generated argument list,
///  matching the other generators of this package.
pub const MAX_CONTROL_ARGUMENTS: usize = 48;

/// What:
///  The byte that ends the option part of a separated argument list.
/// Why:
///   One fixed value lets the fuzzer place the `--` separator wherever it likes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SEPARATOR_MARK = 0xff;
/// ```
pub const SEPARATOR_MARK: u8 = 0xff;

/// What:
///  Map each fuzz byte to one token of the control table.
///  `Vec<OsString>` is the
///       owned argument list.
/// Why:
///   Random bytes almost never spell a control;
///  drawing whole tokens lets the fuzzer
///       place controls,
///  hatches,
///  values and separators in every order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const controlArguments = (data: Uint8Array) => [...data].slice(0, 48).map(b => CONTROL_TOKENS[b % CONTROL_TOKENS.length]);
/// ```
pub fn control_arguments(data: &[u8]) -> Vec<OsString> {
    // `Vec::<OsString>::new()` is an empty owned list; `mut` allows pushing.
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    // `for byte in data` borrows each byte; `*byte` reads it.
    for byte in data {
        if result.len() == MAX_CONTROL_ARGUMENTS {
            break;
        }
        // `usize::from` widens the byte to an index; `%` wraps it into the table.
        result.push(argument(
            CONTROL_TOKENS[usize::from(*byte) % CONTROL_TOKENS.len()],
        ));
    }
    return result;
}

/// What:
///  Build `<command> <valueless options...> -- <any tokens...>` and say where the
///       `--` sits.
///  `Option<(Vec<OsString>, usize)>` is "the list and the separator's
///       index,
///  or nothing" for empty input.
/// Why:
///   When no token before `--` takes a separate value,
///  the `--` is certainly Git's
///       separator,
///  so everything from it on is paths and must survive untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function separatedArguments(data: Uint8Array): { args: string[]; separator: number } | undefined;
/// ```
pub fn separated_arguments(data: &[u8]) -> Option<(Vec<OsString>, usize)> {
    // `let Some((first, rest)) = ... else` splits off the first byte or returns for empty input.
    let Some((first, rest)) = data.split_first() else {
        // `None` is the "absent" case of `Option`.
        return None;
    };
    let mut result: Vec<OsString> = vec![argument(
        TABLE_COMMANDS[usize::from(*first) % TABLE_COMMANDS.len()],
    )];
    // `None` until the mark is met; then `Some(index of the separator)`.
    let mut separator: Option<usize> = None;
    for byte in rest {
        if result.len() == MAX_CONTROL_ARGUMENTS {
            break;
        }
        // `match` on "already separated or not".
        match separator {
            Some(_) => result.push(argument(
                CONTROL_TOKENS[usize::from(*byte) % CONTROL_TOKENS.len()],
            )),
            None => {
                if *byte == SEPARATOR_MARK {
                    // `Some(x)` is the "present" case.
                    separator = Some(result.len());
                    result.push(argument(b"--"));
                } else {
                    result.push(argument(
                        VALUELESS_TOKENS[usize::from(*byte) % VALUELESS_TOKENS.len()],
                    ));
                }
            }
        }
    }
    // An input without the mark gets its separator at the end.
    let at: usize = match separator {
        Some(found) => found,
        None => {
            result.push(argument(b"--"));
            result.len() - 1
        }
    };
    return Some((result, at));
}

/// What:
///  The tokens of `before` that are missing from `after`,
///  when `after` is `before`
///       with some tokens taken out.
///  `Option<Vec<OsString>>` is "the missing tokens,
///  or
///       nothing" when `after` is not such a list.
/// Why:
///   Removal may only delete whole tokens;
///  it may never reorder,
///  rewrite or add one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removedTokens(before: string[], after: string[]): string[] | undefined;
/// ```
pub fn removed_tokens(before: &[OsString], after: &[OsString]) -> Option<Vec<OsString>> {
    let mut removed: Vec<OsString> = Vec::<OsString>::new();
    // How many tokens of `after` have been matched so far.
    let mut matched: usize = 0;
    // `for token in before` borrows each original token in order.
    for token in before {
        if matched < after.len() && after[matched] == *token {
            matched += 1;
        } else {
            // `.clone()` copies the token into the owned result.
            removed.push(token.clone());
        }
    }
    if matched != after.len() {
        return None;
    }
    return Some(removed);
}

/// What:
///  Whether a token spells something the wrapper may remove.
///  `&OsStr` borrows it.
/// Why:
///   Only a general control or the commit hatch may ever leave the arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isWrapperToken = (token: string) => controlMeaning(token) !== undefined || token === '--no-enforce-only';
/// ```
fn is_wrapper_token(token: &OsStr) -> bool {
    // `.as_encoded_bytes()` lends the raw bytes of the token.
    let bytes: &[u8] = token.as_encoded_bytes();
    return control_meaning(bytes).is_some() || bytes == COMMIT_HATCH;
}

/// What:
///  Whether any token of a list has this meaning as a general control.
/// Why:
///   Every effect the removal records must have a removed token that asked for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = (tokens: string[], meaning: ControlMeaning) => tokens.some(t => controlMeaning(t) === meaning);
/// ```
fn any_asks(tokens: &[OsString], meaning: ControlMeaning) -> bool {
    for token in tokens {
        if control_meaning(token.as_encoded_bytes()) == Some(meaning) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether any token of a list is exactly these bytes.
/// Why:
///   Several frontier rules ask "did the caller write this token anywhere".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const has = (tokens: string[], bytes: string) => tokens.includes(bytes);
/// ```
fn has_token(tokens: &[OsString], bytes: &[u8]) -> bool {
    for token in tokens {
        if token.as_encoded_bytes() == bytes {
            return true;
        }
    }
    return false;
}

/// What:
///  Check every invariant of control removal for one argument list.
///  Panics,
///  which is
///       how a fuzz target reports a failure,
///  when one does not hold.
/// Why:
///   The properties are stated without Git's option tables,
///  so a mistake in a table
///       cannot hide behind the same mistake in the check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkControlRemoval(args: string[]): void; // throws on a broken invariant
/// ```
pub fn check_control_removal(arguments: &[OsString]) {
    // `.to_vec()` copies the input to prove afterwards that it was not changed.
    let snapshot: Vec<OsString> = arguments.to_vec();
    let stripped: StrippedInvocation = strip_wrapper_controls(arguments);
    assert_eq!(arguments, snapshot.as_slice(), "removal changed its input");
    assert_eq!(
        strip_wrapper_controls(arguments),
        stripped,
        "removal is not repeatable"
    );
    assert_eq!(
        global_layout(stripped.arguments.as_slice()),
        stripped.layout,
        "the reported layout is not the layout of the result"
    );
    // `let Some(removed) = ... else { panic!(...) }` unwraps the missing tokens or fails.
    let Some(removed) = removed_tokens(arguments, stripped.arguments.as_slice()) else {
        panic!(
            "removal reordered, rewrote or added a token: {arguments:?} -> {:?}",
            stripped.arguments
        );
    };
    for token in &removed {
        assert!(
            is_wrapper_token(token.as_os_str()),
            "a token that is not a wrapper control was removed: {token:?} from {arguments:?}"
        );
    }
    // Every recorded effect has a removed token that asked for it, and the other way round.
    let controls: &Controls = &stripped.controls;
    assert_eq!(
        controls.keep_going,
        any_asks(removed.as_slice(), ControlMeaning::KeepGoing),
        "keep-going: {arguments:?}"
    );
    assert_eq!(
        controls.skip_worktree_copy,
        any_asks(removed.as_slice(), ControlMeaning::SkipWorktreeCopy),
        "worktree-copy opt-out: {arguments:?}"
    );
    assert_eq!(
        controls.commit_only_escaped,
        has_token(removed.as_slice(), COMMIT_HATCH),
        "commit hatch: {arguments:?}"
    );
    for descriptor in POLICY_REGISTRY {
        assert_eq!(
            is_escaped(controls, descriptor.id),
            any_asks(removed.as_slice(), ControlMeaning::Escape(descriptor.id)),
            "escape of {}: {arguments:?}",
            descriptor.name
        );
    }
    if removed.is_empty() {
        assert_eq!(
            *controls,
            no_controls(),
            "an effect without a removed token"
        );
    }
    // Removing again finds nothing: what is left is values, paths and Git's own tokens.
    let again: StrippedInvocation = strip_wrapper_controls(stripped.arguments.as_slice());
    assert_eq!(
        again.arguments, stripped.arguments,
        "a second removal took more: {arguments:?}"
    );
    assert_eq!(
        again.controls,
        no_controls(),
        "a second removal found a control: {arguments:?}"
    );
}

/// What:
///  Check that everything from the `--` at `separator` on survives removal.
/// Why:
///   After Git's separator every token is a path,
///  whatever it spells.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkSeparator(args: string[], separator: number): void;
/// ```
pub fn check_separator(arguments: &[OsString], separator: usize) {
    let stripped: StrippedInvocation = strip_wrapper_controls(arguments);
    // `&list[n..]` borrows the tokens from index `n` on.
    let tail: &[OsString] = &arguments[separator..];
    assert!(
        stripped.arguments.ends_with(tail),
        "a token after the separator was removed: {arguments:?} -> {:?}",
        stripped.arguments
    );
}

/// What:
///  A facts provider with fixed answers.
///  A `struct` is a record with named fields.
/// Why:
///   The frontier is a decision from arguments and location;
///  fixed answers let the
///       fuzzer drive it without Git or a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FixedFacts = { location: RepositoryLocation; index: IndexVsHead; dirty: boolean; sequencer: SequencerState; remoteGuess: boolean; ignoredAndTracked: string[]; untrackedAndTracked: string[]; stash: boolean; cleanDeletesOnlyByItsPatterns: boolean; operationWaits: boolean; worktreeWork: boolean; stagedWork: boolean; selection: StagedSelection };
/// ```
#[derive(Clone, Debug)]
pub struct FixedFacts {
    /// The answer to `location`.
    pub location: RepositoryLocation,
    /// The answer to `index_vs_head`.
    pub index: IndexVsHead,
    /// The answer to `worktree_dirty`,
    /// for either scope.
    pub dirty: bool,
    /// The answer to `sequencer_state`.
    pub sequencer: SequencerState,
    /// The answer to `remote_guess_creates_branch`.
    pub remote_guess: bool,
    /// The ignored files the tree a forced form writes also tracks;
    ///  empty unless the mode asks for them.
    pub ignored_and_tracked: Vec<String>,
    /// The untracked files the tree an abort restores also tracks;
    ///  empty unless the mode asks for them.
    pub untracked_and_tracked: Vec<String>,
    /// The answer to `repository_holds_a_stash`.
    pub stash: bool,
    /// Whether the dry run of a clean as typed names a path only its own `-e`/`--exclude` patterns call ignored,
    ///  and the same clean without them names nothing.
    pub clean_deletes: bool,
    /// The answer to `staged_selection`,
    ///  for whichever pathspecs were asked about.
    pub selection: StagedSelection,
    /// Whether the operation a guarded abort or skip form ends waits;
    ///  when it does not, `abort_discard` answers that Git refuses the form itself.
    pub operation_waits: bool,
    /// Whether the person's own work stands in the worktree an abort would write over.
    pub worktree_work: bool,
    /// Whether the person's own work stands in the index an abort would reset.
    pub staged_work: bool,
}

/// What:
///  `impl RepositoryFacts for FixedFacts` provides the subject's facts interface.
///       `Result<T, String>` is "a value or an error text";
///  `Ok(x)` is the success case.
/// Why:
///   The lifecycle under test asks for facts only through this interface.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FixedFacts implements RepositoryFacts { /* returns its fields */ }
/// ```
impl RepositoryFacts for FixedFacts {
    fn location(&mut self) -> Result<RepositoryLocation, String> {
        // `.clone()` copies the stored answer for the caller.
        return Ok(self.location.clone());
    }

    fn index_vs_head(&mut self) -> Result<IndexVsHead, String> {
        return Ok(self.index);
    }

    fn staged_selection(&mut self, _pathspecs: Vec<OsString>) -> Result<StagedSelection, String> {
        return Ok(self.selection.clone());
    }

    fn worktree_dirty(&mut self, _scope: DirtyScope) -> Result<bool, String> {
        return Ok(self.dirty);
    }

    /// What:
    ///  Answer with the ignored files the tree a forced form writes also tracks,
    ///  which that form overwrites.
    /// Why:
    ///   A non-empty answer is what makes a forced checkout reject in a linked worktree,
    ///  and the mode byte chooses it so the fuzzer reaches that rejection.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ignoredFilesTheTreeTracks(revision) { return this.ignoredAndTracked; }
    /// ```
    fn ignored_files_the_tree_tracks(&mut self, _revision: Option<&OsStr>) -> Result<Vec<String>, String> {
        return Ok(self.ignored_and_tracked.clone());
    }

    /// What:
    ///  Answer with the untracked files the tree an abort or skip restores also tracks,
    ///  whose paths its reset refuses to write over.
    /// Why:
    ///   A non-empty answer is what makes an abort or a skip reject in a linked worktree,
    ///  and the mode chooses it so the fuzzer reaches that rejection.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// untrackedFilesTheTreeTracks(revision) { return this.untrackedAndTracked; }
    /// ```
    fn untracked_files_the_tree_tracks(&mut self, _revision: &OsStr) -> Result<Vec<String>, String> {
        return Ok(self.untracked_and_tracked.clone());
    }

    /// The answer to `repository_holds_a_stash`,
    ///  which decides whether `stash drop` and `stash clear` reject.
    fn repository_holds_a_stash(&mut self) -> Result<bool, String> {
        return Ok(self.stash);
    }

    /// What:
    ///  Answer the dry run of the clean as typed with the one path its own patterns add,
    ///  and the dry run of the same clean without them with nothing.
    /// Why:
    ///   That difference is what decides whether a clean carrying its own exclusions rejects;
    ///  a path both answers name is one the worktree itself ignores,
    ///  which no longer refuses anything.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cleanDeletedPaths(arguments) {
    ///   return arguments.some((one) => one.startsWith('-e') || one.startsWith('--exclude')) && this.cleanDeletes
    ///     ? [Buffer.from('Would remove notes.txt')] : [];
    /// }
    /// ```
    fn clean_deleted_paths(&mut self, arguments: Vec<OsString>) -> Result<Vec<Vec<u8>>, String> {
        // Only the answer that carries a pattern names anything; the same clean without one deletes what the
        //  worktree itself ignores, which is nothing this fake repository holds.
        let carries_pattern: bool = arguments.iter().any(|one| {
            let bytes: &[u8] = one.as_encoded_bytes();
            return bytes.starts_with(b"-e") || bytes.starts_with(b"--exclude");
        });
        if carries_pattern && self.clean_deletes {
            return Ok(vec![b"Would remove notes.txt".to_vec()]);
        }
        return Ok(Vec::<Vec<u8>>::new());
    }

    fn sequencer_state(&mut self) -> Result<SequencerState, String> {
        return Ok(self.sequencer);
    }

    /// What:
    ///  Answer with the directory whose `.git` is the repository's common directory:
    ///  the main checkout of the linked worktree, or the worktree itself.
    /// Why:
    ///   A refusal of a shared remote setting names the checkout that owns the file,
    ///  and the fake location puts that file at `<main>/.git/config`,
    ///  so the directory holding it is the main checkout.
    ///  No repository has no such directory,
    ///  which is the failure the real provider gives when Git cannot name one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// mainWorktreeRoot() {
    ///   const common = commonDirectory(this.location.identity);
    ///   return common === undefined ? { error: '...' } : path.dirname(common);
    /// }
    /// ```
    fn main_worktree_root(&mut self) -> Result<PathBuf, String> {
        // `.parent()` is the directory whose `.git` the common directory is; a worktree's own root stands there.
        return match common_directory(&self.location.identity).and_then(Path::parent) {
            Some(root) => Ok(root.to_path_buf()),
            None => Err(String::from(
                "the fuzz facts hold no repository, so no main checkout exists to name",
            )),
        };
    }

    /// What:
    ///  Answer what the abort or skip form would discard besides the operation's own state:
    ///  the operation the command's own word ends when the mode says one waits,
    ///  and the work the mode puts in the worktree or the index.
    /// Why:
    ///   The guard reads this before it decides anything about an abort,
    ///  so the three answers it can give are each a mode:
    ///  no operation of this kind waits (Git refuses the form and discards nothing),
    ///  the operation waits and holds only its own state (the command runs as Git runs it),
    ///  or the person's own work stands where the reset writes (the command is refused).
    ///  The fake holds no recorded conflict result of its own,
    ///  so the listings compare against the result Git recorded,
    ///  which is the case no path of them can be confused with a resolution.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// abortDiscard(command) {
    ///   const ended = endedOperation(command);
    ///   if (ended === undefined || !this.operationWaits) return { kind: 'nothing-waiting' };
    ///   return { kind: 'waiting', inProgress: ended[0], ends: ended[1], restored: SKIP_RESTORES,
    ///     baseline: 'recorded-result',
    ///     worktree: this.worktreeWork ? ['notes.md'] : [], staged: this.stagedWork ? ['staged.rs'] : [],
    ///     conflicted: [] };
    /// }
    /// ```
    fn abort_discard(&mut self, command: GuardedCommand) -> Result<AbortDiscard, String> {
        // The operation a form ends is the command's own reading, so the answer names the state the guard judges.
        let Some((aborted, ends)): Option<(AbortOperation, OperationEnd)> = ended_operation(command) else {
            return Ok(AbortDiscard::NothingWaiting);
        };
        if !self.operation_waits {
            return Ok(AbortDiscard::NothingWaiting);
        }
        return Ok(AbortDiscard::Waiting {
            in_progress: aborted,
            ends,
            // A skip writes back the tree it stopped at; the fake repository holds that one commit, so an abort's
            //  reset writes the same tree back.
            restored: OsString::from(SKIP_RESTORES),
            // Git recorded the result it computed for the conflict it stopped at, so a path still holding it is
            //  Git's own text rather than a resolution the wrapper cannot judge.
            baseline: DiscardBaseline::RecordedResult,
            // `vec![...]` is an owned list holding the one path the mode puts there.
            worktree: if self.worktree_work {
                vec![b"notes.md".to_vec()]
            } else {
                Vec::<Vec<u8>>::new()
            },
            staged: if self.staged_work {
                vec![b"staged.rs".to_vec()]
            } else {
                Vec::<Vec<u8>>::new()
            },
            conflicted: Vec::<Vec<u8>>::new(),
        });
    }

    fn remote_guess_creates_branch(&mut self, _target: &OsStr) -> Result<bool, String> {
        return Ok(self.remote_guess);
    }

    /// What:
    ///  Refuse to prepare candidates,
    ///  as Git that cannot start would.
    /// Why:
    ///   The fuzz lifecycle starts no Git,
    ///  so a content policy can never read a
    ///       candidate here,
    ///  and must then stop the command rather than pass it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// candidates(request) { throw new CandidateError('git-not-started', '...'); }
    /// ```
    fn candidates(
        &mut self,
        _request: &CandidateRequest,
    ) -> Result<PreparedCandidates, CandidateError> {
        return Err(CandidateError::new(
            CandidateFailure::GitNotStarted,
            "the fuzz facts start no Git, so no candidate can be prepared",
        ));
    }

    /// What:
    /// The fixed location,
    /// as already known.
    /// Why:
    ///  Recovery before a read-only command then probes a location that does not exist,
    ///       which finds no transaction and starts no Git.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// askedLocation() { return this.location; }
    /// ```
    fn asked_location(&self) -> Option<Result<RepositoryLocation, String>> {
        return Some(Ok(self.location.clone()));
    }

    /// What:
    /// A transaction context naming a Git that does not exist,
    ///       with the common directory of the fixed location when it has one.
    /// Why:
    ///  Any Git the lifecycle tried to start would fail instead of touching a repository,
    ///       and a location without a repository has no common directory,
    /// as in the real facts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// transactionContext() {
    ///   return { gitPath: '/nonexistent/git', overlay: [], globalPrefix: [], commonDir: commonDirectory(this.location.identity) };
    /// }
    /// ```
    fn transaction_context(&self) -> GitContext {
        return GitContext {
            real_git: PathBuf::from("/nonexistent/git"),
            overlay: Vec::new(),
            global_prefix: Vec::new(),
            // `.map(Path::to_path_buf)` copies the borrowed directory into the owned value the context keeps.
            common_dir: common_directory(&self.location.identity).map(Path::to_path_buf),
        };
    }
}

/// What:
///  The answers one frontier run gives its repository, as bits of one unsigned integer.
/// Why:
///   The fuzzer chooses them and the seeds carry them,
///  so both need one width to read and write:
///  `MODE_BYTES` is it,
///  and widening the answers is a change here and in `fuzz_targets/wrapper_controls.rs`,
///  which the seed writer reads and the seed control decodes with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mode = number; // little endian, MODE_BYTES wide
/// ```
pub type Mode = u16;

/// The bytes an input spends on its mode, which is the width of `Mode` itself.
pub const MODE_BYTES: usize = size_of::<Mode>();

/// What:
///  The bit of a mode that selects the linked worktree;
///  its absence is no repository at all.
/// Why:
///   One bit of the fuzz input chooses between the two locations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINKED_WORKTREE = 0x001;
/// ```
pub const LINKED_WORKTREE: Mode = 1 << 0;

/// The bit of a mode that makes the index differ from `HEAD`.
pub const INDEX_DIFFERS: Mode = 1 << 1;

/// The bit of a mode that makes an operation wait for its conflicts.
pub const SEQUENCER_WAITS: Mode = 1 << 2;

/// The bit of a mode that makes the target of a `checkout` or `switch` a branch of one remote.
pub const REMOTE_GUESS: Mode = 1 << 3;

/// The bit of a mode whose worktree holds uncommitted state.
pub const WORKTREE_DIRTY: Mode = 1 << 4;

/// The bit of a mode whose repository holds an ignored file at a path a restored tree tracks.
pub const IGNORED_AND_TRACKED: Mode = 1 << 5;

/// The bit of a mode whose repository holds a stash entry.
pub const STASH_HELD: Mode = 1 << 6;

/// The bit of a mode whose pattern-only `clean` dry run names a path.
pub const CLEAN_DELETES: Mode = 1 << 7;

/// The bit of a mode whose repository holds an untracked file at a path a restored tree tracks.
pub const UNTRACKED_AND_TRACKED: Mode = 1 << 8;

/// What:
///  The bit of a mode in which the operation a guarded abort or skip form ends waits.
/// Why:
///   Without it the form's own state is absent,
///  which is Git's to refuse and discards nothing.
pub const OPERATION_WAITS: Mode = 1 << 9;

/// The bit of a mode in which the person's own work stands in the worktree an abort writes over.
pub const WORKTREE_WORK: Mode = 1 << 10;

/// The bit of a mode in which the person's own work stands in the index an abort resets.
pub const STAGED_WORK: Mode = 1 << 11;

/// What:
///  Whether `mode` selects the linked worktree.
///  `bool` is true or false.
/// Why:
///   One bit of the fuzz input chooses between the two locations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLinked = (mode: number) => (mode & LINKED_WORKTREE) !== 0;
/// ```
pub fn mode_is_linked(mode: Mode) -> bool {
    return mode & LINKED_WORKTREE != 0;
}

/// What:
///  Fixed facts chosen by two bytes:
///  a linked worktree or no repository,
///  and the
///       eleven other answers from the remaining bits,
///  one bit each (`LINKED_WORKTREE` through `STAGED_WORK`).
/// Why:
///   Neither location exists on disk,
///  so no configuration and no leftover state is
///       found and the defaults apply:
///  the built-in content policy is on.
///  One bit per answer lets the fuzzer choose every answer on its own,
///  so a rule that reads two of them is seen with each of their four combinations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixedFacts(mode: number): FixedFacts;
/// ```
pub fn fixed_facts(mode: Mode) -> FixedFacts {
    let identity: WorktreeIdentity = if mode_is_linked(mode) {
        WorktreeIdentity::LinkedWorktree {
            common_dir: PathBuf::from("/nonexistent-cli-git-fuzz/main/.git"),
            git_dir: PathBuf::from("/nonexistent-cli-git-fuzz/main/.git/worktrees/linked"),
            worktree_root: PathBuf::from("/nonexistent-cli-git-fuzz/linked"),
        }
    } else {
        WorktreeIdentity::OutsideWorktree
    };
    return FixedFacts {
        location: RepositoryLocation {
            identity,
            // `Vec::<u8>::new()` is an empty owned byte list: the command runs at the top level.
            prefix: Vec::<u8>::new(),
        },
        index: if mode & INDEX_DIFFERS != 0 {
            IndexVsHead::Differs
        } else {
            IndexVsHead::Matches
        },
        sequencer: if mode & SEQUENCER_WAITS != 0 {
            SequencerState::InProgress
        } else {
            SequencerState::NotInProgress
        },
        dirty: mode & WORKTREE_DIRTY != 0,
        remote_guess: mode & REMOTE_GUESS != 0,
        // `vec![...]` is an owned list holding the one path a forced form would overwrite.
        ignored_and_tracked: if mode & IGNORED_AND_TRACKED != 0 {
            vec![String::from("ignored/kept.env")]
        } else {
            Vec::<String>::new()
        },
        // The same shape for the untracked file an abort's reset refuses to write over.
        untracked_and_tracked: if mode & UNTRACKED_AND_TRACKED != 0 {
            vec![String::from("victim.txt")]
        } else {
            Vec::<String>::new()
        },
        stash: mode & STASH_HELD != 0,
        clean_deletes: mode & CLEAN_DELETES != 0,
        operation_waits: mode & OPERATION_WAITS != 0,
        worktree_work: mode & WORKTREE_WORK != 0,
        staged_work: mode & STAGED_WORK != 0,
        // The fake repository holds one path, and the whole worktree and the directory the command runs in
        //  both give it, so a reach-only spelling of `git add` covers a set here as it does in a real one.
        selection: StagedSelection {
            selected: 1,
            repository: 1,
            directory: 1,
        },
    };
}

/// What:
///  The starts of the `git commit` long options that make it a dry run,
///  and the
///       letter of the short one.
///  `&[&[u8]]` is a borrowed list of byte strings.
/// Why:
///   Git 2.56.0 `builtin/commit.c` turns `--dry-run` on by itself for every status
///       format option:
///  `--short`,
///  `--porcelain`,
///  `--long` and `-z`/`--null`.
///  Each start is
///       the shortest abbreviation Git accepts for that option (`--l` for `--long`),
///  except `--po`,
///       which Git itself refuses as ambiguous and which is kept as a looser start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMMIT_DRY_RUN_STARTS = ['--dr', '--sh', '--po', '--l', '--nu'];
/// ```
const COMMIT_DRY_RUN_STARTS: &[&[u8]] = &[b"--dr", b"--sh", b"--po", b"--l", b"--nu"];

/// The short option of `git commit` that implies a dry run:
///  `-z`.
const COMMIT_DRY_RUN_LETTER: u8 = b'z';

/// The start of the one `git push` long option that makes it a dry run.
const PUSH_DRY_RUN_STARTS: &[&[u8]] = &[b"--dr"];

/// The short option of `git push` for a dry run:
///  `-n`.
const PUSH_DRY_RUN_LETTER: u8 = b'n';

/// What:
///  Whether some token could make Git treat a command as a dry run:
///  a long option
///       with one of the given starts,
///  or a short cluster containing the given letter.
/// Why:
///   This is a deliberately loose restatement that does not use the subject's tables.
///       A forwarded commit or publishing push without any such token is certainly real.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayBeDryRun = (region: string[], starts: string[], letter: string) =>
///   region.some(t => starts.some(s => t.startsWith(s)) || (/^-[^-]/.test(t) && t.includes(letter)));
/// ```
fn may_be_dry_run(region: &[OsString], starts: &[&[u8]], letter: u8) -> bool {
    for token in region {
        let bytes: &[u8] = token.as_encoded_bytes();
        for start in starts {
            if bytes.starts_with(start) {
                return true;
            }
        }
        if bytes.starts_with(b"-") && !bytes.starts_with(b"--") && bytes.contains(&letter) {
            return true;
        }
    }
    return false;
}

/// What:
///  How the lifecycle ended for one checked argument list.
///  An `enum` is a closed set
///       of named alternatives.
///  `#[derive(...)]` asks the compiler to generate copying,
///       debug printing and `==`.
/// Why:
///   The generator controls count these,
///  so a frontier rule that is never reached
///       cannot pass unnoticed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FrontierSeen = 'forwarded' | 'rejected' | 'refused';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierSeen {
    /// Git would run.
    Forwarded,
    /// A policy or fixed transform stopped the command with exit status 1.
    Rejected,
    /// The command stopped with exit status 2.
    Refused,
}

/// What:
///  Check the refusal frontier for one argument list at the location and with the repository answers `mode`
///       selects,
///  and say how the lifecycle ended.
/// Why:
///   Whatever the arguments,
///  a forwarded command must not be a real commit,
///  a
///       publishing push with a policy on,
///  or an unchecked `git add`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkFrontier(args: string[], mode: number): FrontierSeen;
/// ```
pub fn check_frontier(arguments: &[OsString], mode: Mode) -> FrontierSeen {
    let stripped: StrippedInvocation = strip_wrapper_controls(arguments);
    // `mut` lets the lifecycle state what content policies can read.
    let mut checks: ShippedChecks<FixedFacts> = shipped_checks(
        fixed_facts(mode),
        stripped.arguments.clone(),
        LifecycleContent::None,
        Vec::<ToolCacheDirectory>::new(),
    );
    // `&[]` is an empty environment: no lease is inherited.
    let outcome: WrappedOutcome = run_wrapped_command(&stripped, &[], &mut checks);
    // `match` picks by variant and binds the fields each ending carries.
    let forwarded: Vec<OsString> = match outcome {
        WrappedOutcome::Exit { code, stderr } => {
            assert!(code == 1 || code == 2, "a stop exits 1 or 2, got {code}");
            // A real commit writes its own events and diagnostics while it runs, so its ending carries no text.
            let commit_wrote_itself: bool = stderr.is_empty()
                && stripped.layout.outcome == GlobalOutcome::Command
                && stripped.arguments[stripped.layout.prefix_len].as_encoded_bytes() == b"commit";
            assert!(
                stderr.ends_with('\n') || commit_wrote_itself,
                "a stop explains itself: {arguments:?}"
            );
            if code == 1 {
                return FrontierSeen::Rejected;
            }
            return FrontierSeen::Refused;
        }
        WrappedOutcome::Forward {
            arguments: sent, ..
        } => sent,
    };
    if stripped.layout.outcome != GlobalOutcome::Command {
        assert_eq!(
            forwarded, stripped.arguments,
            "Git's own answer is forwarded as written"
        );
        return FrontierSeen::Forwarded;
    }
    let word: &[u8] = stripped.arguments[stripped.layout.prefix_len].as_encoded_bytes();
    let region: &[OsString] = &stripped.arguments[stripped.layout.prefix_len + 1..];
    if word == b"commit" {
        // A region that Git's option table refuses is forwarded so that Git reports its own refusal, and commits nothing.
        assert!(
            may_be_dry_run(region, COMMIT_DRY_RUN_STARTS, COMMIT_DRY_RUN_LETTER)
                || parse_commit_region(region, &[]).is_err(),
            "a real commit was forwarded: {arguments:?}"
        );
    }
    if word == b"push" {
        assert!(
            may_be_dry_run(region, PUSH_DRY_RUN_STARTS, PUSH_DRY_RUN_LETTER)
                || is_escaped(&stripped.controls, PolicyId::FinalNewline),
            "a publishing push was forwarded past the manual-push gate: {arguments:?}"
        );
    }
    if !mode_is_linked(mode) {
        return FrontierSeen::Forwarded;
    }
    if word == b"add" {
        assert!(
            is_escaped(&stripped.controls, PolicyId::FinalNewline),
            "git add was forwarded while a content policy could not read it: {arguments:?}"
        );
    }
    return FrontierSeen::Forwarded;
}

/// Generator reach and fixed hard cases stay out of the fuzz targets.
#[cfg(test)]
#[path = "controls_tests.rs"]
mod tests;
