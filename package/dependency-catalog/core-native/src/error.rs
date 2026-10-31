//! What:     The error types: catalog validation problems, sync problems, and the one `Error` enum the public
//!           functions return.
//! Why:      Both validators collect every problem in one pass so a contributor fixes the whole file at once; the
//!           messages match the TypeScript tool's text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // errors.ts
//! export class RustCatalogError extends Error { issues: RustCatalogIssue[] }
//! export class RustCatalogSyncError extends Error { issues: SyncIssue[] }
//! ```

use std::fmt;

/// What:     `pub struct CatalogIssue { ... }` is one validation problem in `rust-catalog.toml`.
/// Why:      Names where it is, what is wrong and how to fix it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RustCatalogIssue = { location: string; problem: string; remedy: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogIssue {
    /// Dotted path such as `catalog.tokio`.
    pub location: String,
    /// What is wrong.
    pub problem: String,
    /// How to fix it.
    pub remedy: String,
}

/// One problem that stops a sync, naming the file and location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncIssue {
    /// Manifest or catalog path, as passed in.
    pub file: String,
    /// Dotted location inside the file.
    pub location: String,
    /// What is wrong.
    pub problem: String,
    /// How to fix it.
    pub remedy: String,
}

/// What:     `pub enum Error { ... }` is every failure the public functions can return.
/// Why:      One type lets callers use `?` across parse, sync and check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failure = TomlEditError | RustCatalogError | RustCatalogSyncError;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The file is not valid TOML.
    Toml {
        /// File path for the message.
        path: String,
        /// The TOML parser's message.
        message: String,
    },
    /// `rust-catalog.toml` has entries or settings the catalog cannot use.
    Catalog {
        /// Catalog path for the message.
        source_path: String,
        /// Every problem found, in source order.
        issues: Vec<CatalogIssue>,
    },
    /// Manifests and the catalog cannot be brought in line without a human decision; nothing is changed.
    Sync {
        /// Every problem found, in discovery order.
        issues: Vec<SyncIssue>,
    },
}

impl fmt::Display for Error {
    /// What:     Prints the same multi-line diagnostic the TypeScript tool printed.
    /// Why:      Contributors and agents read these messages to fix the files.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// error.message
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Toml { path, message } => {
                return write!(formatter, "{path} is not valid TOML: {message}");
            }
            Error::Catalog { source_path, issues } => {
                write!(
                    formatter,
                    "{source_path} has {} problem(s) the Rust dependency catalog cannot accept:",
                    issues.len()
                )?;
                for issue in issues {
                    write!(formatter, "\n- {}: {}.\n  Fix: {}.", issue.location, issue.problem, issue.remedy)?;
                }
                return write!(formatter, "\nSee doc/decision/rust-dependency-catalog.md for the catalog format.");
            }
            Error::Sync { issues } => {
                write!(
                    formatter,
                    "The Rust dependency catalog sync found {} problem(s); nothing was written:",
                    issues.len()
                )?;
                for issue in issues {
                    write!(
                        formatter,
                        "\n- {} {}: {}.\n  Fix: {}.",
                        issue.file, issue.location, issue.problem, issue.remedy
                    )?;
                }
                return write!(formatter, "\nSee doc/decision/rust-dependency-catalog.md.");
            }
        }
    }
}

impl std::error::Error for Error {}
