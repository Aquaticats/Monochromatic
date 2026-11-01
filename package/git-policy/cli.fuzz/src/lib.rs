//! What:
//!  Generators and invariants for the native cli-git wrapper's pure boundaries.
//! Why:
//!  Fuzz targets and their generator controls call the same checks,
//!  so a property
//!      proven to be reached by the controls is the property the fuzzer explores.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export * from './arguments.ts'; export * from './configuration.ts';
//! ```
#![cfg(unix)]

/// Git token tables shared by the argument generator and its invariants.
mod argument_tables;

/// Byte-valued argument vectors and the global-argument and config-loading invariants.
pub mod arguments;

/// Generated configuration documents and the schema invariants.
pub mod configuration;

/// Wrapper-control token tables shared by the control generators and their invariants.
mod control_tables;

/// Control-bearing argument vectors and the invariants of removal and of the refusal frontier.
pub mod controls;

/// Generated `git cat-file --batch` replies and the reply-reading invariants.
pub mod batch;

/// Index listings,
///  staged deltas,
///  `rulesFile` values and final-newline bytes,
///  with their invariants.
pub mod content;

/// Generated owner records and `/proc/<pid>/stat` lines, with their parsers' invariants.
pub mod owners;

/// Generated Git PID file text and stat lines for the index-lock readers, with their invariants.
pub mod index_lock;

/// Generated journal, capture-order and sequence records, with their parsers' invariants.
pub mod journal;
/// Generated manifests and raw text through the dependent-version rewrite and specifier scan.
pub mod dependent_version;
/// Linter output, event paths and the one-rule configuration of the Markdown policy, with their invariants.
pub mod markdown;
