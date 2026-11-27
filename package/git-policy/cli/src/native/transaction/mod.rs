//! What:
//!  Durable transaction state:
//!  owner records,
//!  journal,
//!  phases and recovery.
//! Why:
//!  Every durable state shares one fail-closed recovery.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `git_step_arguments.rs`:
//!   the arguments of a Git step that ran long,
//!   named in its record with every word the person typed left out.
//! - `recovery_error.rs`:
//!   the fail-closed failure of reading or recovering durable transaction state.
//! - `recovery_files.rs`:
//!   no-follow file operations recovery and landing share.
//! - `transaction_git_request_and_reply_types.rs`:
//!   the request,
//!   reply and failure types of the Git runner and the text of a command line.
//! - `transaction_git_step_words.rs`:
//!   the subcommand of a Git step,
//!   the words after it,
//!   and their split into options,
//!   names and paths.
//! - `transaction_git_step_rerun_and_end_rules.rs`:
//!   which Git steps may be run again after a refused process and which may be ended at a ceiling.
//! - `transaction_step_durations.rs`:
//!   the Git steps a command ran,
//!   each with its name,
//!   start and duration,
//!   for the timing records.
//! - `transaction_git_step_bounds.rs`:
//!   how long a Git step is waited for and what happens at the bound.
//! - `transaction_git_step_about_to_start.rs`:
//!   one Git step of a transaction at the moment before it starts,
//!   the child command and input file it is started from,
//!   and what the wrapper answers once it ended.
//! - `transaction_git.rs`:
//!   running real Git for transaction,
//!   recovery and landing work.
//! - `transaction_journal.rs`:
//!   the schema-version-2 journal records of one commit transaction and their filenames.
//! - `transaction_owner.rs`:
//!   the owner record of one commit-transaction directory and its liveness rule.
//! - `transaction_phase.rs`:
//!   test-only transaction phase markers,
//!   compiled into the release executable and armed only by an environment variable.
//! - `transaction_git_awaiting.rs`:
//!   running real Git for a caller that is itself a task of the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // transaction/index.ts: one `export * from './file'` per module below.
//! ```

/// The fail-closed failure of reading or recovering durable transaction state.
pub mod recovery_error;

/// The owner record of one commit-transaction directory and its liveness rule.
pub mod transaction_owner;

/// The schema-version-2 journal records of one commit transaction and their filenames.
pub mod transaction_journal;

/// Running real Git for transaction,
///  recovery and landing work.
pub mod transaction_git;

/// Running real Git for a caller that is itself a task of the runtime.
pub mod transaction_git_awaiting;

/// One Git step of a transaction at the moment before it starts,
/// and what the wrapper answers once it ended.
pub mod transaction_git_step_about_to_start;

/// The request,
/// reply and failure types of the Git runner and the text of a command line.
pub mod transaction_git_request_and_reply_types;

/// The subcommand of a Git step,
/// the words after it,
/// and their split into options,
/// names and paths.
pub mod transaction_git_step_words;

/// Which Git steps may be run again after a refused process and which may be ended at a ceiling.
pub mod transaction_git_step_rerun_and_end_rules;

/// How long a Git step is waited for and what happens at the bound.
pub mod transaction_git_step_bounds;

/// The Git steps a command ran,
/// each with its name,
/// start and duration,
/// for the timing records.
pub mod transaction_step_durations;

/// The arguments of a Git step that ran long,
/// named in its record with every word the person typed left out.
pub mod git_step_arguments;

/// No-follow file operations recovery and landing share.
pub mod recovery_files;

/// Test-only transaction phase markers,
/// compiled into the release executable and armed only by an environment variable.
pub mod transaction_phase;
