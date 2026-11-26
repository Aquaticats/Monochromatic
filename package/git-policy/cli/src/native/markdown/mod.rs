//! What:
//!  The `markdown/autofix` policy and its in-process linter.
//! Why:
//!  The linter runs on a guarded thread and has its own configuration,
//!  so it stays apart from the other policies.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `markdown_exclude.rs`:
//!   the gitignore-syntax `exclude` patterns of `markdown/autofix`,
//!   matched as the linter matches them.
//! - `markdown_linter_config.rs`:
//!   the one-rule linter configuration document.
//! - `markdown_linter_guard.rs`:
//!   a job on a thread of its own with a deadline and panic containment.
//! - `markdown_linter_output.rs`:
//!   what one in-process linter run produced:
//!   the fixed source and the findings left,
//!   or why it is unusable.
//! - `policy_markdown.rs`:
//!   the `markdown/autofix` policy over candidates,
//!   behind its linter interface.
//! - `markdown_linter.rs`:
//!   the executable's linter for that policy:
//!   `monochromatic-lint` linked in-process,
//!   guarded by a deadline,
//!   once per candidate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // markdown/index.ts: one `export * from './file'` per module below.
//! ```

/// What one in-process linter run produced:
///  the fixed source and the findings left,
///  or why it is unusable.
pub mod markdown_linter_output;

/// The one-rule linter configuration document.
pub mod markdown_linter_config;

/// The gitignore-syntax `exclude` patterns of `markdown/autofix`,
///  matched as the linter matches them.
pub mod markdown_exclude;

/// The `markdown/autofix` policy over candidates,
///  behind its linter interface.
pub mod policy_markdown;

/// The executable's linter for that policy:
///  `monochromatic-lint` linked in-process,
///  guarded by a deadline,
///  once per candidate.
pub mod markdown_linter;

/// A job on a thread of its own with a deadline and panic containment.
pub mod markdown_linter_guard;
