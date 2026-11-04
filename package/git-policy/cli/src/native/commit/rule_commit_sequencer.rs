//! What:
//!  The pure core of the "is a merge,
//!  cherry-pick or revert awaiting its commit" check.
//! Why:
//!  The caller runs real Git and probes the filesystem;
//!  this module says which query to
//!      run,
//!  reads its output exactly,
//!  and decides from three presence facts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sequencerState({ mergeHead, cherryPickHead, revertHead }): 'in-progress' | 'none'
//! ```

/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Whether an operation is awaiting its concluding commit.
/// Why:
///   While one is,
///  Git forbids partial commits,
///  so a pathless `git commit` is the
///       documented conclusion and the commit-only transform lets it through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SequencerState = 'in-progress' | 'none';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequencerState {
    /// A merge,
    ///  cherry-pick or revert head file exists.
    InProgress,
    /// No head file exists,
    ///  or Git could not answer;
    ///  normal enforcement applies.
    NotInProgress,
}

/// Git-directory files whose presence marks an operation awaiting its concluding commit.
pub const SEQUENCER_HEAD_FILES: &[&str] = &["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"];

/// What:
///  Existence of each head file,
///  as the caller observed it.
///  `bool` is true or false.
/// Why:
///   These three facts are everything the decision needs from the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SequencerFacts = { mergeHead: boolean; cherryPickHead: boolean; revertHead: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequencerFacts {
    /// `MERGE_HEAD` exists.
    pub merge_head: bool,
    /// `CHERRY_PICK_HEAD` exists.
    pub cherry_pick_head: bool,
    /// `REVERT_HEAD` exists.
    pub revert_head: bool,
}

/// What:
///  Why the path query's output could not be read.
/// Why:
///   Output that is not exactly three paths in one directory is reported,
///  never guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SequencerOutputError extends Error {}
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequencerOutputError;

/// `impl std::fmt::Display` supplies Rust's "print me" interface for the error.
impl std::fmt::Display for SequencerOutputError {
    /// `&self` borrows the error read-only;
    ///  `&mut` lends the formatter for writing.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(
            "git rev-parse --git-path did not print the three sequencer head paths of one Git directory",
        );
    }
}

/// An empty `impl` marks the type as a standard error value for generic handling.
impl std::error::Error for SequencerOutputError {}

/// What:
///  Build the real-Git argument list:
///  the caller's global options,
///  then one
///       `--git-path` per head file.
///  `Vec<OsString>` is the owned result.
/// Why:
///   `--git-path` resolves linked worktrees and `GIT_DIR`,
///  which a lexical join cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...pre, 'rev-parse', '--path-format=absolute', '--git-path', 'MERGE_HEAD', /* ... */];
/// ```
pub fn sequencer_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into a list this function owns.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("rev-parse"));
    arguments.push(OsString::from("--path-format=absolute"));
    // `for name in SEQUENCER_HEAD_FILES` borrows each file name in order.
    for name in SEQUENCER_HEAD_FILES {
        arguments.push(OsString::from("--git-path"));
        arguments.push(OsString::from(name));
    }
    return arguments;
}

/// What:
///  Read the query output as three paths without splitting on newlines.
///       `Result<Vec<Vec<u8>>, _>` is "three byte paths,
///  or a refusal".
/// Why:
///   A Git directory path may contain a newline.
///  The three files share one directory,
///       so its length follows from the output length and the result is checked byte for byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directoryLength = (stdout.length - fixedLength) / 3; // then verify every byte
/// ```
pub fn sequencer_head_paths(stdout: &[u8]) -> Result<Vec<Vec<u8>>, SequencerOutputError> {
    // Bytes that are not the directory: each name plus its trailing newline.
    let mut fixed: usize = 0;
    for name in SEQUENCER_HEAD_FILES {
        fixed += name.len() + 1;
    }
    if stdout.len() < fixed {
        // `Err(x)` is the failure case of `Result`.
        return Err(SequencerOutputError);
    }
    // `.is_multiple_of(n)` is true when dividing by `n` leaves no remainder.
    if !(stdout.len() - fixed).is_multiple_of(SEQUENCER_HEAD_FILES.len()) {
        return Err(SequencerOutputError);
    }
    // The shared directory prefix, including its trailing separator.
    let directory_length: usize = (stdout.len() - fixed) / SEQUENCER_HEAD_FILES.len();
    // `&stdout[..n]` borrows the first `n` bytes.
    let directory: &[u8] = &stdout[..directory_length];
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    let mut offset: usize = 0;
    for name in SEQUENCER_HEAD_FILES {
        // `.to_vec()` copies the directory bytes so the path can grow.
        let mut path: Vec<u8> = directory.to_vec();
        path.extend_from_slice(name.as_bytes());
        let end: usize = offset + path.len();
        if &stdout[offset..end] != path.as_slice() || stdout[end] != b'\n' {
            return Err(SequencerOutputError);
        }
        offset = end + 1;
        paths.push(path);
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(paths);
}

/// What:
///  Decide from the three presence facts.
/// Why:
///   Any one head file means Git is waiting for the concluding commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return present.includes(true) ? 'in-progress' : 'none';
/// ```
pub fn sequencer_state(facts: SequencerFacts) -> SequencerState {
    if facts.merge_head || facts.cherry_pick_head || facts.revert_head {
        return SequencerState::InProgress;
    }
    return SequencerState::NotInProgress;
}

/// What:
///  The state to use when the path query itself failed.
/// Why:
///   The incumbent falls back to normal enforcement and lets real Git report its error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// catch (error) { if (error instanceof SubprocessError) return 'none'; throw error; }
/// ```
pub fn sequencer_state_when_query_fails() -> SequencerState {
    return SequencerState::NotInProgress;
}

/// Output parsing,
///  the decision and real-Git controls of the query.
#[cfg(test)]
#[path = "rule_commit_sequencer_tests.rs"]
mod tests;
