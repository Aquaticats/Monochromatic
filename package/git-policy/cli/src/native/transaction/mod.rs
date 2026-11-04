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

/// No-follow file operations recovery and landing share.
pub mod recovery_files;

/// Test-only transaction phase markers, compiled into the release executable and armed only by an environment variable.
pub mod transaction_phase;
