//! What:
//!  Small shared building blocks:
//!  private files,
//!  scratch locations,
//!  JSON,
//!  time,
//!  random ids,
//!  paths and bounded parallelism.
//! Why:
//!  Subsystems depend on these and they depend on no subsystem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // foundation/index.ts: one `export * from './file'` per module below.
//! ```

/// Private,
///  crash-safe file primitives shared by every durable record.
pub mod private_storage;

/// Where short-lived private files live:
///  the per-user scratch directory and the repository scratch directory,
///  never the system temporary directory.
pub mod scratch_location;

/// Compact JSON records read and written with the incumbent's `JSON` semantics.
pub mod json_record;

/// Random identifiers in the layout of the incumbent's `randomUUID`.
pub mod random_id;

/// Text operations with the exact semantics of JavaScript's built-ins.
pub mod js_text;

/// Line-oriented debug and warning diagnostics on standard error.
pub mod diagnostic_log;

/// UTC timestamps in the exact form of JavaScript's `toISOString`.
pub mod iso_time;

/// Lexical path normalization.
pub mod lexical_path;

/// How many workers independent items may use together, from the CPUs allowed, the memory free and a measured ceiling.
pub mod parallel_budget;

/// Independent items on a bounded number of threads, reported in input order with no lock.
pub mod parallel_runner;
