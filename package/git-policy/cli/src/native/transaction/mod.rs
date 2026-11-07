//! What:
//!  Durable transaction state:
//!  owner records,
//!  journal,
//!  phases and recovery.
//! Why:
//!  Every durable state shares one fail-closed recovery.
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

/// The request, reply and failure types of the Git runner and the text of a command line.
pub mod transaction_git_request_and_reply_types;

/// Which Git steps may be run again after a refused process and which may be ended at a ceiling.
pub mod transaction_git_step_rerun_and_end_rules;

/// How long a Git step is waited for and what happens at the bound.
pub mod transaction_git_step_bounds;

/// The Git steps a command ran, each with its name, start and duration, for the timing records.
pub mod transaction_step_durations;

/// No-follow file operations recovery and landing share.
pub mod recovery_files;

/// Test-only transaction phase markers, compiled into the release executable and armed only by an environment variable.
pub mod transaction_phase;
