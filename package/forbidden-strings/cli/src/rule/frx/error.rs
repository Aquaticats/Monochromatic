//! Redacted load-error type for the forbidden-regex rule compiler.
//!
//! Every variant is safe to print on stdout or stderr:
//!  it carries an opaque rule
//! index (the rule's 0-based position in the compiled set,
//!  never a source line
//! number) and the engine's own static reason,
//!  but never the rule text.
//!  This is
//! the load-path half of the README's leak-safety guarantee (#217):
//!  a sensitive
//! rule body may live in a CI secret without its bytes reaching CI logs.

/// Imports the formatter pieces for the redacted `Display` rendering.
use std::fmt;

/// Imports the engine's compile-time error,
///  surfaced as the static reason.
use forbidden_regex::CompileError;

/// A rule-load failure with everything the rule text redacted out.
///
/// The compiler fails closed on the first offending rule and reports only its
/// opaque index plus a reason that echoes no pattern bytes.
///  `Debug` is derived
/// because no variant holds rule text,
///  so the derived form is leak-safe too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The source held no non-blank,
    ///  non-comment rule line.
    NoRules,
    /// A regex rule carried a flag letter other than the `m`/`x` no-ops.
    UnsupportedFlag {
        /// Opaque 0-based index of the offending rule in the compiled set.
        index: usize,
        /// First flag letter outside `{m, x}`;
        ///  a config letter,
        ///  never rule text.
        flag: char,
    },
    /// The engine rejected a rule at compile time (bad dialect,
    ///  empty-matchable,
    /// oversized repetition,
    ///  state-cap blowup).
    Compile {
        /// Opaque 0-based index of the offending rule in the compiled set.
        index: usize,
        /// Engine's static reason;
        ///  its `Display` echoes no pattern bytes.
        reason: CompileError,
    },
    /// A precompiled serialized `RegexSet` blob failed to decode or validate.
    Precompiled {
        /// Engine's static reason;
        ///  a codec/validation message,
        ///  never rule text.
        reason: CompileError,
    },
    /// A tail-format line's trimmed form led with `==>` without being exactly a
    /// strict `==> name <==` header,
    ///  so it is a near-header,
    ///  never absorbed
    /// silently into a section body;
    ///  genuine arrow-leading content uses the
    /// `[=]=> ` reshape in regex bodies.
    NearHeader {
        /// 1-based source line of the malformed header;
        ///  carries no name text.
        line: usize,
    },
    /// A tail-format file carried significant content before its first section
    /// header,
    ///  which the format forbids so no rule can escape a section.
    PreHeaderContent {
        /// 1-based source line of the offending pre-header content.
        line: usize,
    },
    /// A tail-format section header opened a body with no significant rule line,
    /// which is fail-closed rather than a silently dropped section.
    EmptySection {
        /// 1-based source line of the empty section's header.
        line: usize,
    },
    /// Two tail-format sections declared the same name,
    ///  which collides the rule
    /// identities the format exists to keep unique across the loaded input.
    DuplicateName {
        /// 1-based source line where the name was first declared.
        first_line: usize,
        /// 1-based source line of the colliding redeclaration.
        line: usize,
    },
}

/// Renders a `LoadError` as a redacted,
///  user-facing diagnostic.
impl fmt::Display for LoadError {
    /// Writes a one-line reason that never contains rule text.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Branch per variant; each interpolates only an opaque index, a config
        // flag letter, or the engine's own static reason.
        match self {
            LoadError::NoRules => {
                return write!(f, "no rules loaded")
            }
            LoadError::UnsupportedFlag { index, flag } => {
                return write!(
                    f,
                    "rule {index}: unsupported flag '{flag}'; only 'm' and 'x' are accepted as no-ops",
                )
            }
            LoadError::Compile { index, reason } => {
                return write!(f, "rule {index}: {reason}")
            }
            LoadError::Precompiled { reason } => {
                return write!(f, "precompiled ruleset failed to load: {reason}")
            }
            LoadError::NearHeader { line } => {
                return write!(
                    f,
                    "line {line}: line starts with '==>' but is not a strict '==> name <==' section header; reshape genuine content as '[=]=>'",
                )
            }
            LoadError::PreHeaderContent { line } => {
                return write!(f, "line {line}: content before the first section header")
            }
            LoadError::EmptySection { line } => {
                return write!(f, "line {line}: section header has no rule body")
            }
            LoadError::DuplicateName { first_line, line } => {
                return write!(
                    f,
                    "line {line}: duplicate section name first declared at line {first_line}",
                )
            }
        }
    }
}

/// Lets `LoadError` participate in the standard error ecosystem.
impl std::error::Error for LoadError {}

/// Registers the redaction and rendering unit tests (sidecar,
///  lint-exempt).
#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
