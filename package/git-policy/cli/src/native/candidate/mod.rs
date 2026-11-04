//! What:
//!  Candidate content:
//!  what a commit or push would add,
//!  read from the index or objects,
//!  and the scanner run over it.
//! Why:
//!  Policies judge candidates,
//!  so the readers and the scanner adapter share a folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // candidate/index.ts: one `export * from './file'` per module below.
//! ```

/// The one failure type of the candidate layer and its closed list of causes.
pub mod candidate_error;

/// Validated Git object names and the file modes a candidate can carry.
pub mod candidate_object;

/// Pure framing of one `git cat-file --batch` reply.
pub mod candidate_batch;

/// Pure parsing of raw NUL-delimited changed-path records.
pub mod candidate_record;

/// The long-lived object reader that answers every blob read of an invocation.
pub mod candidate_reader;

/// Immutable candidate versions:
///  identity,
///  pathname,
///  mode,
///  change and object per path.
pub mod candidate_version;

/// Per-invocation listing,
///  lazy bytes and invalidation of candidate versions.
pub mod candidate_store;

/// The listings of one commit's tree,
///  of the parents of a set of commits and of a tree's blobs,
///  as a second block of operations of the candidate store.
pub mod candidate_store_commit_listings;

/// Pure parsing of `git ls-files --stage` records and the delta between two index states.
pub mod candidate_stage;

/// The files of a commit's tree with their modes and objects.
pub mod candidate_tree;

/// A private,
///  timestamp-preserving copy of the real index,
///  removed when dropped.
pub mod candidate_private_index;

/// Which candidate paths have their `text` Git attribute unset,
///  read from the state the candidates were listed from.
pub mod candidate_attribute;

/// How many paths one Git command of the candidate layer names at a time.
pub mod candidate_pathspec_chunk_size;

/// The Git runs of a prediction:
///  one captured command with its failures,
///  and the path of the real index.
pub mod candidate_prediction_git_runs;

/// What `git add` would stage,
///  and the worktree files a direct command selects.
pub mod candidate_prediction;

/// Which candidates hold content that a parent commit's tree already holds,
///  found by exact object name.
pub mod candidate_known_content;

/// The linked forbidden-strings scanner:
///  one load per invocation,
///  typed redacted findings.
pub mod scanner_adapter;

/// Rules-file precedence and which candidates the scanner is given.
pub mod scanner_selection;

/// One scan pass over a candidate version's exact bytes.
pub mod scanner_run;

/// Which engine failure code each candidate and scanner failure carries.
pub mod scanner_failure_code;

/// Process isolation and rule fixtures for scanner controls;
///  never in the release build.
#[cfg(test)]
pub(crate) mod scanner_test_support;

/// Loading many blobs through one `git cat-file --batch`.
pub mod blob_batch;
