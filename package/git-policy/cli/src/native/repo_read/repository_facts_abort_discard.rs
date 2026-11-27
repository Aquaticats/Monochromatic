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
//!  The two listings are kept apart rather than merged,
//!  because a refusal has to say which of the two holds the work it would discard:
//!  a staged resolution and an edit in the worktree are two different things,
//!  and one erased index entry is a third.
//!  The argument lists of the questions this file asks and the readings of their answers are
//!  `repository_facts_abort_discard_query_arguments_and_readings.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const discard = await abortDiscard(facts, GuardedCommand.AbortMerge);
//! // discard.kind === 'waiting' && discard.paths // ['a.rs', 'notes.md']
//! ```

/// `use` brings the names this file needs from the sibling modules that own them.
use crate::git_metadata::MetadataOutput;
use crate::repository_facts::GitFacts;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::StateAnswer;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::both_listings;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::listed_paths;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::merge_result_queries;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::state_answer;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::state_from_presence;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::state_query_arguments;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::uncommitted_queries;
pub use crate::repository_facts_abort_discard_query_arguments_and_readings::unresolved_query;
use crate::rule_abort_operation::{
    AbortOperation,
    OperationEnd,
    OperationState,
    RestoredRevision,
    SKIP_RESTORES,
    abort_is_live,
    ended_operation,
    in_progress_operation,
    restored_revision,
};
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::PathBuf;

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

/// What:
///  The paths the operation is still stopped at,
///  or why Git could not name them.
/// Why:
///   The reading that compares each side with the other cannot tell an untouched conflicted path
///  from an edit,
/// and no more can be proved about it without a recorded result;
///  a listing that failed is a failure and never an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function unresolvedPaths(facts: GitFacts): Promise<Buffer[]>; // throws
/// ```
fn unresolved_paths(facts: &mut GitFacts) -> Result<Vec<Vec<u8>>, String> {
    let answer: MetadataOutput = facts.query(
        unresolved_query(facts.global_prefix.as_slice()),
        "the paths the operation still stands at",
    )?;
    return listed_paths(&answer);
}

/// What:
///  Whether the result Git computed for the conflict it stopped at (`AUTO_MERGE`) can be read.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   Every operation that stops at a conflict with the merge machinery records that result,
///  not only a merge:
///  a rebase,
/// a pick and an `am` leave `AUTO_MERGE` behind too (measured with Git 2.56.0),
///  and a path that still holds it was never touched by the person.
///  A repository without the ref (an older Git)
///  has no recorded result to compare with,
///  so the readings fall back to comparing each side with the other.
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
///  that is not a regular file,
///  or that does not hold a commit id,
///  is a failure and never a quiet fall back to `HEAD`,
///  because the refusal it earns must not read as clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recordedCommit(answer: StateAnswer, record: 'sequencer/head'): Promise<string>; // throws
/// ```
fn recorded_commit(
    answer: &StateAnswer,
    record: &OsStr,
) -> Result<OsString, String> {
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
    let text: String = crate::repository_facts_abort_discard_record::read_the_record(path)?;
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
pub(super) fn abort_discard(
    facts: &mut GitFacts,
    command: GuardedCommand,
) -> Result<AbortDiscard, String> {
    let Some((aborted, ends)): Option<(AbortOperation, OperationEnd)> = ended_operation(command)
    else {
        return Ok(AbortDiscard::NothingWaiting);
    };
    let state_output: MetadataOutput = facts.query(
        state_query_arguments(facts.global_prefix.as_slice()),
        "the operation state",
    )?;
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
    // A skip writes back the tree it stopped at, which is `HEAD`; an abort writes back the record its own
    //  implementation reads (`restored_revision`).
    let restored: OsString = match ends {
        OperationEnd::Skip => OsString::from(SKIP_RESTORES),
        OperationEnd::Abort => match restored_revision(aborted, state) {
            RestoredRevision::Named(name) => name.to_os_string(),
            RestoredRevision::StateFile(record) => recorded_commit(&answer, record)?,
        },
    };
    // Git records the result it computed for the conflict it stopped at, whichever operation stopped there,
    //  so the person's own work is what differs from that result;
    //  where no result is recorded there is no baseline that tells Git's own conflict text from a resolution,
    //  and every uncommitted change is at risk.
    let recorded: bool = merge_result_readable(facts)?;
    let (queries, baseline): ([Vec<OsString>; 2], DiscardBaseline) = match recorded {
        true => (
            merge_result_queries(facts.global_prefix.as_slice()),
            DiscardBaseline::RecordedResult,
        ),
        false => (
            uncommitted_queries(facts.global_prefix.as_slice()),
            DiscardBaseline::OperationStart,
        ),
    };
    let first: MetadataOutput =
        facts.query(queries[0].clone(), "the paths this command would discard")?;
    let second: MetadataOutput = facts.query(
        queries[1].clone(),
        "the staged paths this command would discard",
    )?;
    let mut worktree_paths: Vec<Vec<u8>> = listed_paths(&first)?;
    let index_paths: Vec<Vec<u8>> = listed_paths(&second)?;
    // Without a recorded result, a path the operation is still stopped at holds either the conflict text Git wrote
    //  there or a resolution the person wrote over it, and nothing the wrapper can read tells the two apart:
    //  a checked-in strategy's text names its own temporary files (the `resolve` strategy does: measured, its markers
    //  read `.merge_file_1xHykn` and the name is fresh each time), a custom merge driver writes whatever it likes,
    //  and an older Git records no result at all. So those paths are kept too, and named as ones the wrapper cannot
    //  judge, which refuses the command rather than risk a resolution that exists nowhere else.
    let conflicted: Vec<Vec<u8>> = match recorded {
        true => Vec::<Vec<u8>>::new(),
        false => unresolved_paths(facts)?,
    };
    // A path the worktree no longer holds is no work of the person's: the reset writes the tree back over it.
    //  The answer is read from the filesystem, after the location Git already named.
    let top: Option<PathBuf> =
        crate::repository_facts_abort_discard_worktree_entries::worktree_top(facts);
    let mut held: Vec<Vec<u8>> = Vec::<Vec<u8>>::with_capacity(worktree_paths.len());
    for path in worktree_paths {
        if crate::repository_facts_abort_discard_worktree_entries::holds_an_entry(
            top.as_deref(),
            path.as_slice(),
        ) {
            held.push(path);
        }
    }
    worktree_paths = held;
    // The content a whole index entry holds is work the command would discard;
    //  a path whose index entry was erased holds none, and the reset writes the tree back over it,
    //  so it is no reason to refuse the command.
    let staged: Vec<Vec<u8>> =
        crate::repository_facts_staged_entries::staged_entries(facts, index_paths.as_slice())?;
    return Ok(AbortDiscard::Waiting {
        in_progress: in_progress_operation(aborted, state),
        ends,
        restored,
        baseline,
        worktree: both_listings(worktree_paths, Vec::new()),
        staged,
        conflicted,
    });
}

/// The queries,
/// their readings and the paths they name.
#[cfg(test)]
#[path = "repository_facts_abort_discard_tests.rs"]
mod tests;
