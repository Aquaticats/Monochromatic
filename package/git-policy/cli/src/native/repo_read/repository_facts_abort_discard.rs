//! What:
//!  What one abort form would discard besides the operation's own state,
//!  read from real Git's own listings.
//! Why:
//!  An operation that stopped with conflicts has made the worktree unclean by itself,
//!  and its abort exists to let that state go;
//!  what must not go is the person's own work,
//!  the conflict resolutions and edits made while the operation waited.
//!  Those are exactly the paths where the worktree or the index differs from
//!  the merge result Git recorded (`AUTO_MERGE`),
//!  and for the operations that record no result the whole uncommitted state,
//!  because a rebase,
//!  a pick and an `am` return with a hard reset that keeps nothing.
//!  Every answer is Git's own;
//!  nothing here guesses a path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const discard = await abortDiscard(facts, GuardedCommand.AbortMerge);
//! // discard.kind === 'waiting' && discard.paths // ['a.rs', 'notes.md']
//! ```

/// `use` brings the names this file needs from the sibling modules that own them.
use crate::git_metadata::MetadataOutput;
use crate::repository_facts::GitFacts;
use crate::rule_abort_operation::{
    AbortOperation, OPERATION_STATE_NAMES, RESTORED_REVISION_NAMES, RestoredRevision, SEQUENCER_DIRECTORY, OperationState, abort_is_live,
    aborted_operation, in_progress_operation, restored_revision,
};
use crate::rule_linked_worktree::GuardedCommand;
use crate::rule_worktree_dirty::nul_separated_paths;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

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
/// type AbortDiscard = { kind: 'nothing-waiting' } | { kind: 'waiting'; inProgress: AbortOperation; restored: string; paths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbortDiscard {
    /// The state this command's own operation writes is absent,
    ///  so Git refuses the command and discards nothing.
    NothingWaiting,
    /// The command's operation waits,
    ///  and these paths hold the work its abort would discard besides its own.
    Waiting {
        /// Which operation this command ends.
        in_progress: AbortOperation,
        /// The revision whose tree the abort writes back into the worktree,
        ///  as Git's own implementation of this abort finds it:
        ///  a name Git resolves,
        ///  or the commit read from the record Git wrote.
        restored: OsString,
        /// The listed paths the worktree no longer holds while the index does:
        ///  the abort resets the index too,
        ///  so their content would be discarded where no worktree file can be copied.
        index_only: Vec<Vec<u8>>,
        /// The repository-relative paths,
        ///  in a settled order and each once.
        paths: Vec<Vec<u8>>,
    },
}

/// What:
///  The argument list that names each state file and directory of an operation,
///  and each file an abort reads the commit it restores from.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///   `--git-path` resolves linked worktrees and `GIT_DIR`,
///  and `--path-format=absolute` makes each answer a path this process can open,
///  which a lexical join cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...pre, 'rev-parse', '--path-format=absolute', '--git-path', 'MERGE_HEAD', /* ... */];
/// ```
pub fn state_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("rev-parse"));
    arguments.push(OsString::from("--path-format=absolute"));
    for name in OPERATION_STATE_NAMES {
        arguments.push(OsString::from("--git-path"));
        arguments.push(OsString::from(name));
    }
    arguments.push(OsString::from("--git-path"));
    arguments.push(OsString::from(SEQUENCER_DIRECTORY));
    for name in RESTORED_REVISION_NAMES {
        arguments.push(OsString::from("--git-path"));
        arguments.push(OsString::from(name));
    }
    return arguments;
}

/// What:
///  The state a list of presence facts describes.
///  `[bool; 7]` is one presence fact per name `OPERATION_STATE_NAMES` asks about,
///  then the sequencer directory.
/// Why:
///   The reading is separated from the file system,
///  so the mapping from files to operations can be checked without a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function state_from_presence([mergeHead, cherryHead, revertHead, rebaseMerge, rebaseRebasing, rebaseApplying, sequencer]: boolean[]): OperationState;
/// ```
pub fn state_from_presence(present: [bool; 7]) -> OperationState {
    return OperationState {
        merge_head: present[0],
        cherry_pick_head: present[1],
        revert_head: present[2],
        rebase_merge: present[3],
        rebase_apply_rebasing: present[4],
        rebase_apply_applying: present[5],
        sequencer: present[6],
    };
}

/// What:
///  The state query's answers:
///  the presence facts `state_from_presence` reads,
///  and where Git records the commit each abort form restores.
///  `[bool; 7]` is one presence fact per name `OPERATION_STATE_NAMES` asks about,
///  then the sequencer directory.
/// Why:
///   The record paths have to come from the same one start of Git as the presence facts,
///  because a record can change between two starts,
///  and an abort must read the revision its own implementation will read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await facts.stateAnswer(); // { state, restoredPaths }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateAnswer {
    /// Which operation states are present.
    pub state: OperationState,
    /// Where each restored-revision record lives,
    ///  in `RESTORED_REVISION_NAMES` order.
    pub restored_paths: Vec<(&'static str, PathBuf)>,
}

/// What:
///  The state query's answers as the state and the record paths,
///  or why the answer is not one path per name.
///  `&[u8]` borrows the query's standard output.
/// Why:
///   A query that printed another number of lines answered a different question,
///  and a guard must not read that as "no operation waits".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = stateAnswer(output.stdout.split('\n')); // throws
/// ```
pub fn state_answer(answer: &[u8]) -> Result<StateAnswer, String> {
    let names: usize = OPERATION_STATE_NAMES.len() + 1 + RESTORED_REVISION_NAMES.len();
    let mut paths: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(names);
    for line in answer.split(is_line_feed) {
        if line.is_empty() {
            continue;
        }
        paths.push(line);
    }
    if paths.len() != names {
        return Err(String::from(
            "could not read the operation state: git rev-parse --git-path did not answer one path per state file.",
        ));
    }
    let mut present: [bool; 7] = [false; 7];
    let mut records: Vec<(&'static str, PathBuf)> = Vec::with_capacity(RESTORED_REVISION_NAMES.len());
    for (index, path) in paths.iter().enumerate() {
        if index < OPERATION_STATE_NAMES.len() + 1 {
            present[index] = as_path(path).exists();
            continue;
        }
        let name: &'static str = RESTORED_REVISION_NAMES[index - (OPERATION_STATE_NAMES.len() + 1)];
        records.push((name, as_path(path).to_path_buf()));
    }
    return Ok(StateAnswer { state: state_from_presence(present), restored_paths: records });
}

/// Whether a byte of the state query's answer ends a path.
/// Why:
///   The answer is one path per line,
///  and a named predicate reads as the one test it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeed = (byte: number): boolean => byte === 0x0a;
/// ```
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  A byte path as the process reads it.
/// Why:
///   A Git directory can hold any byte but NUL,
///  and the presence probe must open exactly what Git printed,
///  without a trailing directory separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asPath = (bytes: Buffer): string => Buffer.from(bytes).toString();
/// ```
fn as_path(bytes: &[u8]) -> &Path {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return Path::new(std::ffi::OsStr::from_bytes(bytes.strip_suffix(b"/").unwrap_or(bytes)));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return Path::new(std::str::from_utf8(bytes).unwrap_or_default());
    }
}

/// What:
///  The two listings that name the paths where the worktree and the index differ from
///  Git's recorded merge result,
///  which is the person's own work during a merge.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///   `AUTO_MERGE` holds the merge result,
///  conflict markers included,
///  so a path whose worktree file matches it was not touched since the merge,
///  and only a path that differs is work the abort would discard.
///  The index side drops the unmerged entries (`--diff-filter=ACDMRTB` keeps every status but `U`),
///  because an untouched conflicted path has stages rather than an entry and is not the person's doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queries = [diffWorktree('AUTO_MERGE'), diffIndex('AUTO_MERGE', 'ACDMRTB')];
/// ```
pub fn merge_result_queries(global_prefix: &[OsString]) -> [Vec<OsString>; 2] {
    let mut worktree: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "diff", "--name-only", "-z", "AUTO_MERGE"] {
        worktree.push(OsString::from(token));
    }
    let mut index: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "diff", "--cached", "--name-only", "-z", "--diff-filter=ACDMRTB", "AUTO_MERGE"] {
        index.push(OsString::from(token));
    }
    return [worktree, index];
}

/// What:
///  The two listings that name every uncommitted change of the tracked files:
///  the worktree against the index,
///  and the index against `HEAD`.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///   A rebase,
///  a cherry-pick,
///  a revert and an `am` return with a hard reset that keeps no uncommitted change at all,
///  and none of them records the result it staged,
///  so everything uncommitted is at risk and all of it is saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queries = [diff(['--name-only']), diff(['--cached', '--name-only'])];
/// ```
pub fn uncommitted_queries(global_prefix: &[OsString]) -> [Vec<OsString>; 2] {
    let mut worktree: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "diff", "--name-only", "-z"] {
        worktree.push(OsString::from(token));
    }
    let mut index: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "diff", "--cached", "--name-only", "-z"] {
        index.push(OsString::from(token));
    }
    return [worktree, index];
}

/// What:
///  The paths a `--name-only -z` listing printed,
///  or why Git could not give one.
/// Why:
///   A listing that failed says nothing,
///  and a safety check must never read a failure as "nothing to save".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listedPaths(answer: MetadataOutput): Buffer[]; // throws
/// ```
pub fn listed_paths(answer: &MetadataOutput) -> Result<Vec<Vec<u8>>, String> {
    if !answer.success {
        let detail: String = String::from_utf8_lossy(&answer.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not list the paths this command would discard: {detail}"));
    }
    return Ok(nul_separated_paths(answer.stdout.as_slice()));
}

/// What:
///  The two listings' paths together,
///  each once and in a settled order.
/// Why:
///   A path can differ in both the index and the worktree,
///  and a set keeps the save from writing it twice while the order stays the same between runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const all = new Set([...worktree, ...index]);
/// ```
pub fn both_listings(worktree: Vec<Vec<u8>>, index: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut all: BTreeSet<Vec<u8>> = BTreeSet::<Vec<u8>>::new();
    for path in worktree.into_iter().chain(index) {
        all.insert(path);
    }
    return all.into_iter().collect();
}

/// What:
///  Whether the merge result `AUTO_MERGE` can be read at all.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A repository without the ref (an older Git,
///  a merge the wrapper finished itself)
///  has no recorded result to compare with,
///  so the abort falls back to saving every uncommitted change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeResultReadable(facts: GitFacts): Promise<boolean>;
/// ```
fn merge_result_readable(facts: &mut GitFacts) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = facts.global_prefix.to_vec();
    for token in ["rev-parse", "--verify", "--quiet", "AUTO_MERGE"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput = facts.query(arguments, "the merge result")?;
    return Ok(output.success);
}

/// What:
///  The commit Git recorded for the tree an abort writes back,
///  read from the file its own implementation reads.
/// Why:
///   A rebase records its original head in `rebase-merge/orig-head`
///  or `rebase-apply/orig-head`,
///  and a pick sequence in `sequencer/head`;
///  a record that cannot be read,
///  or that does not hold a commit id,
///  is a failure and never a quiet fall back to `HEAD`,
///  because the ignored-file refusal it earns must not read as clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recordedCommit(answer: StateAnswer, record: 'sequencer/head'): Promise<string>; // throws
/// ```
fn recorded_commit(answer: &StateAnswer, record: &OsStr) -> Result<OsString, String> {
    let mut place: Option<&PathBuf> = None;
    for (name, path) in &answer.restored_paths {
        if OsStr::new(name) == record {
            place = Some(path);
            break;
        }
    }
    let Some(path) = place else {
        return Err(format!(
            "could not read the commit this abort restores: the wrapper asked Git for no record named {}.",
            record.to_string_lossy()
        ));
    };
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(read) => read,
        Err(error) => {
            return Err(format!(
                "could not read the commit this abort restores: Git records it in {} and the file could not be read ({error}).",
                path.display()
            ));
        }
    };
    let text: String = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return Err(format!(
                "could not read the commit this abort restores: Git records it in {} and the file is not text ({error}).",
                path.display()
            ));
        }
    };
    let commit: &str = text.trim();
    if commit.is_empty() || !commit.bytes().all(is_hex_digit) {
        return Err(format!(
            "could not read the commit this abort restores: Git records it in {} and the file holds no commit id.",
            path.display()
        ));
    }
    return Ok(OsString::from(commit));
}

/// Whether one byte of a record is a hexadecimal digit,
///  which every commit id's bytes are.
fn is_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// What:
///  Which paths the abort of `command` would discard besides the operation's own state,
///  and the revision whose tree it writes back.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   This is the one question the guard asks before it saves anything:
///  the command's own operation absent means Git refuses it,
///  a waiting operation means these paths are the person's own work,
///  and the revision is the tree the abort writes back over the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function abortDiscard(facts: GitFacts, command: GuardedCommand): Promise<AbortDiscard>;
/// ```
pub(super) fn abort_discard(facts: &mut GitFacts, command: GuardedCommand) -> Result<AbortDiscard, String> {
    let Some(aborted): Option<AbortOperation> = aborted_operation(command) else {
        return Ok(AbortDiscard::NothingWaiting);
    };
    let state_output: MetadataOutput = facts.query(state_query_arguments(facts.global_prefix.as_slice()), "the operation state")?;
    if !state_output.success {
        let detail: String = String::from_utf8_lossy(&state_output.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("could not read the operation state: {detail}"));
    }
    let answer: StateAnswer = state_answer(state_output.stdout.as_slice())?;
    let state: OperationState = answer.state;
    if !abort_is_live(aborted, state) {
        // This command's own state is absent: Git refuses it, and it discards nothing.
        return Ok(AbortDiscard::NothingWaiting);
    }
    let restored: OsString = match restored_revision(aborted, state) {
        RestoredRevision::Named(name) => name.to_os_string(),
        RestoredRevision::StateFile(record) => recorded_commit(&answer, record)?,
    };
    let queries: [Vec<OsString>; 2] = match aborted {
        AbortOperation::Merge if merge_result_readable(facts)? => merge_result_queries(facts.global_prefix.as_slice()),
        _ => uncommitted_queries(facts.global_prefix.as_slice()),
    };
    let first: MetadataOutput = facts.query(queries[0].clone(), "the paths this command would discard")?;
    let second: MetadataOutput = facts.query(queries[1].clone(), "the staged paths this command would discard")?;
    let paths: Vec<Vec<u8>> = both_listings(listed_paths(&first)?, listed_paths(&second)?);
    // A path the worktree lost while the index kept its content is work this abort would discard,
    //  and no worktree file can be copied for it: the guard refuses the command instead.
    let index_only: Vec<Vec<u8>> = crate::repository_facts_index_only_paths::index_only_paths(facts, paths.as_slice())?;
    return Ok(AbortDiscard::Waiting { in_progress: in_progress_operation(aborted, state), restored, index_only, paths });
}

/// The queries,
/// their readings and the paths they name.
#[cfg(test)]
#[path = "repository_facts_abort_discard_tests.rs"]
mod tests;
