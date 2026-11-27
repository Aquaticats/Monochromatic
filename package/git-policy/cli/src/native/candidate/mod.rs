//! What:
//!  Candidate content:
//!  what a commit or push would add,
//!  read from the index or objects,
//!  and the scanner run over it.
//! Why:
//!  Policies judge candidates,
//!  so the readers and the scanner adapter share a folder.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `blob_batch.rs`:
//!   loading many blobs through one `git cat-file --batch`.
//! - `candidate_error.rs`:
//!   the one failure type of the candidate layer and its closed list of causes.
//! - `candidate_object.rs`:
//!   validated Git object names and the file modes a candidate can carry.
//! - `candidate_batch.rs`:
//!   pure framing of one `git cat-file --batch` reply.
//! - `candidate_pathspec_chunk_size.rs`:
//!   how many paths one Git command of the candidate layer names at a time.
//! - `candidate_attribute.rs`:
//!   which candidate paths have their `text` Git attribute unset,
//!   read from the state the candidates were listed from.
//! - `candidate_prediction_git_runs.rs`:
//!   the Git runs of a prediction:
//!   one captured command with its failures,
//!   and the path of the real index.
//! - `candidate_private_index.rs`:
//!   a private,
//!   timestamp-preserving copy of the real index,
//!   removed when dropped.
//! - `candidate_reader.rs`:
//!   the long-lived object reader that answers every blob read of an invocation.
//! - `candidate_record.rs`:
//!   pure parsing of raw NUL-delimited changed-path records.
//! - `candidate_stage.rs`:
//!   pure parsing of `git ls-files --stage` records and the delta between two index states.
//! - `candidate_tree.rs`:
//!   the files of a commit's tree with their modes and objects.
//! - `candidate_version.rs`:
//!   immutable candidate versions:
//!   identity,
//!   pathname,
//!   mode,
//!   change and object per path.
//! - `candidate_store.rs`:
//!   per-invocation listing,
//!   lazy bytes and invalidation of candidate versions.
//! - `candidate_prediction_listings.rs`:
//!   the listings one prediction makes:
//!   the changed paths `git add` would stage,
//!   the selected worktree files and the staged index entries.
//! - `candidate_prediction.rs`:
//!   what `git add` would stage,
//!   and the worktree files a direct command selects.
//! - `candidate_store_commit_listings.rs`:
//!   the listings of one commit's tree,
//!   of the parents of a set of commits and of a tree's blobs,
//!   as a second block of operations of the candidate store.
//! - `scanner_adapter.rs`:
//!   the linked forbidden-strings scanner:
//!   one load per invocation,
//!   typed redacted findings.
//! - `scanner_selection.rs`:
//!   rules-file precedence and which candidates the scanner is given.
//! - `candidate_known_content.rs`:
//!   which candidates hold content that a parent commit's tree already holds,
//!   found by exact object name.
//! - `scanner_run.rs`:
//!   one scan pass over a candidate version's exact bytes.
//! - `scanner_failure_code.rs`:
//!   which engine failure code each candidate and scanner failure carries.
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

/// The listings one prediction makes:
///  the changed paths,
///  the selected worktree files and the staged index entries.
pub mod candidate_prediction_listings;

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
