//! What:
//!  The standards-review shadow run:
//!  its request,
//!  the patch it covers,
//!  its one HTTPS call and the result it appends.
//! Why:
//!  The review is optional and self-contained,
//!  so its modules sit together.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `standards_review_client_name_lookup_on_the_blocking_pool.rs`:
//!   the standards-review shadow run:
//!   the name lookup of the call,
//!   on the blocking pool.
//! - `standards_review_patch.rs`:
//!   the standards-review shadow run:
//!   a file's changed old-side ranges read from the patch.
//! - `standards_review_text.rs`:
//!   the standards-review shadow run:
//!   JavaScript text behaviors the log depends on (UTF-16 lengths,
//!   number text).
//! - `standards_review_input.rs`:
//!   the standards-review shadow run:
//!   which commits get an entry,
//!   and the capped model input.
//! - `standards_review_client.rs`:
//!   the standards-review shadow run:
//!   the in-process HTTPS call to the decisions gateway,
//!   with a total deadline.
//! - `standards_review_request.rs`:
//!   the standards-review shadow run:
//!   the Decisions API request body and the answer reading.
//! - `standards_review_entry.rs`:
//!   the standards-review shadow run:
//!   log entry rendering and appending.
//! - `standards_review_append.rs`:
//!   the standards-review shadow run:
//!   appending an entry to the shared log without a lock,
//!   once per commit,
//!   with repair.
//! - `standards_review_run.rs`:
//!   the standards-review shadow run,
//!   end to end,
//!   and its bounded runner.
//!   Nothing calls it yet.
//! - `standards_review_source.rs`:
//!   the standards-review shadow run:
//!   the two real-Git reads (patch and file texts) of a landed commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // standards_review/index.ts: one `export * from './file'` per module below.
//! ```

/// The standards-review shadow run:
/// JavaScript text behaviors the log depends on (UTF-16 lengths,
/// number text).
pub mod standards_review_text;

/// The standards-review shadow run:
/// a file's changed old-side ranges read from the patch.
pub mod standards_review_patch;

/// The standards-review shadow run:
/// which commits get an entry,
/// and the capped model input.
pub mod standards_review_input;

/// The standards-review shadow run:
/// the Decisions API request body and the answer reading.
pub mod standards_review_request;

/// The standards-review shadow run:
/// log entry rendering and appending.
pub mod standards_review_entry;

/// The standards-review shadow run:
/// the in-process HTTPS call to the decisions gateway,
/// with a total deadline.
pub mod standards_review_client;

/// The standards-review shadow run:
/// the name lookup of the call,
/// on the blocking pool.
pub mod standards_review_client_name_lookup_on_the_blocking_pool;

/// The standards-review shadow run:
/// the two real-Git reads (patch and file texts) of a landed commit.
pub mod standards_review_source;

/// The standards-review shadow run,
/// end to end,
/// and its bounded runner.
/// Nothing calls it yet.
pub mod standards_review_run;

/// A loopback HTTP server for the standards-review tests;
/// never in the release build.
#[cfg(test)]
pub(crate) mod standards_review_mock;

/// The standards-review shadow run:
/// appending an entry to the shared log without a lock,
/// once per commit,
/// with repair.
pub mod standards_review_append;
