//! What:
//!  The questions one abort form asks Git and the readings of their answers:
//!  the argument list that names each state file,
//!  the state those files describe,
//!  the two listings of paths a command would discard,
//!  and the paths an answer printed.
//! Why:
//!   A safety check must never read a failed or unexpected answer as "nothing to save",
//!  so every reading here answers `None` or an error instead of a quiet empty answer.
//!  The one question that asks Git runs them beside this file
//!  (`repository_facts_abort_discard.rs`),
//!  and each argument list is kept apart from the Git start so it can be checked without a repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const args = stateQueryArguments(prefix); const state = stateAnswer(output.stdout);
//! ```

use crate::git_metadata::MetadataOutput;
use crate::rule_abort_operation::{
    OPERATION_STATE_NAMES,
    OperationState,
    RESTORED_REVISION_NAMES,
    SEQUENCER_DIRECTORY,
};
use crate::rule_worktree_dirty::nul_separated_paths;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

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
    let mut records: Vec<(&'static str, PathBuf)> =
        Vec::with_capacity(RESTORED_REVISION_NAMES.len());
    for (index, path) in paths.iter().enumerate() {
        if index < OPERATION_STATE_NAMES.len() + 1 {
            present[index] = as_path(path).exists();
            continue;
        }
        let name: &'static str = RESTORED_REVISION_NAMES[index - (OPERATION_STATE_NAMES.len() + 1)];
        records.push((name, as_path(path).to_path_buf()));
    }
    return Ok(StateAnswer {
        state: state_from_presence(present),
        restored_paths: records,
    });
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
        return Path::new(std::ffi::OsStr::from_bytes(
            bytes.strip_suffix(b"/").unwrap_or(bytes),
        ));
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
    for token in [
        "--no-optional-locks",
        "diff",
        "--name-only",
        "-z",
        "AUTO_MERGE",
    ] {
        worktree.push(OsString::from(token));
    }
    let mut index: Vec<OsString> = global_prefix.to_vec();
    for token in [
        "--no-optional-locks",
        "diff",
        "--cached",
        "--name-only",
        "-z",
        "--diff-filter=ACDMRTB",
        "AUTO_MERGE",
    ] {
        index.push(OsString::from(token));
    }
    return [worktree, index];
}

/// What:
///  The two listings that name every uncommitted change of the tracked files,
///  for a state where Git recorded no result to compare with:
///  the worktree against the index,
///  and the index against `HEAD`.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///   Without a recorded result,
/// everything uncommitted is at risk;
///  the index side drops the unmerged entries (`--diff-filter=ACDMRTB` keeps every status but `U`),
///  because a path the operation is still stopped at is the operation's own state and not the person's work.
///  The worktree side names such a path too,
/// and the caller asks for the unresolved paths separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queries = [diff(['--name-only']), diff(['--cached', '--name-only', '--diff-filter=ACDMRTB'])];
/// ```
pub fn uncommitted_queries(global_prefix: &[OsString]) -> [Vec<OsString>; 2] {
    let mut worktree: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "diff", "--name-only", "-z"] {
        worktree.push(OsString::from(token));
    }
    let mut index: Vec<OsString> = global_prefix.to_vec();
    for token in [
        "--no-optional-locks",
        "diff",
        "--cached",
        "--name-only",
        "-z",
        "--diff-filter=ACDMRTB",
    ] {
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
        return Err(format!(
            "Git could not list the paths this command would discard: {detail}"
        ));
    }
    return Ok(nul_separated_paths(answer.stdout.as_slice()));
}

/// What:
///  The argument list of the query that names the paths the operation is still stopped at.
///  `&[OsString]` borrows the caller's global options.
/// Why:
///   `--diff-filter=U` selects exactly the unmerged entries
///  (measured with Git 2.56.0:
/// the worktree listing names such a path twice,
/// once with the status `U`
///  and once as a modification,
/// so no other filter separates them),
///  and this is the only reading that does.
///  Where no result is recorded they are named as conflicted rather than dropped,
///  because each may hold a resolution the person wrote and never staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...pre, '--no-optional-locks', 'diff', '--name-only', '-z', '--diff-filter=U'];
/// ```
pub fn unresolved_query(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in [
        "--no-optional-locks",
        "diff",
        "--name-only",
        "-z",
        "--diff-filter=U",
    ] {
        arguments.push(OsString::from(token));
    }
    return arguments;
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
pub fn both_listings(
    worktree: Vec<Vec<u8>>,
    index: Vec<Vec<u8>>,
) -> Vec<Vec<u8>> {
    let mut all: BTreeSet<Vec<u8>> = BTreeSet::<Vec<u8>>::new();
    for path in worktree.into_iter().chain(index) {
        all.insert(path);
    }
    return all.into_iter().collect();
}
