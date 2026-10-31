//! What:
//!  The repository's forbidden-strings scanner,
//!  linked in and loaded once per invocation.
//! Why:
//!  The TypeScript wrapper wrote each candidate to a temporary file,
//!  started the
//!      scanner as a child process and parsed its standard error.
//!  Here candidate bytes
//!      go straight to the scanner library,
//!  and its own typed findings come back.
//!      Nothing is re-derived:
//!  rule loading,
//!  the rule cache,
//!  matching,
//!  redaction and
//!      the fail-closed findings all stay the scanner's.
//!
//! What the linking executable owes the scanner (`package/forbidden-strings/cli/README.md`,
//! "In-process candidate scans"):
//!  an unwinding panic strategy,
//!  which this file checks at
//! build time,
//!  and a panic hook that prints no payload,
//!  installed before the first scan.
//! The scanner catches its own loader and matcher panics;
//!  this module adds no catch of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scanner = CandidateScanner.load(rules, true); const scan = scanner.scan(candidate, bytes);
//! ```

/// Import the candidate whose pathname and identity each scan carries.
use super::candidate_version::Candidate;
/// Import the byte-preserving conversion from Git path bytes to a native path.
use super::git_metadata::path_from_git_bytes;
/// What:
///  The scanner library's public types:
///  its fixed-token cache diagnostic,
///  its
///       per-candidate result,
///  and the loaded rule set.
/// Why:
///   They are returned to callers unchanged,
///  so findings are never re-encoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type CacheWarning, type CandidateScan, Scanner } from 'forbidden-strings';
/// ```
use forbidden_strings::{CacheWarning, CandidateScan, RulesEntry, Scanner};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

// What: `const _: () = assert!(...)` is a check the compiler evaluates while building;
//       `cfg!(panic = "unwind")` is true only when panics unwind the stack.
// Why:  The scanner turns a matcher panic into a fail-closed finding by catching the
//       unwind. Under `panic = "abort"` there is nothing to catch and the process
//       would die mid-scan, so such a build is refused instead of shipped.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent: a build-time assertion about how thrown errors propagate.
// ```
const _: () = assert!(
    cfg!(panic = "unwind"),
    "monochromatic-git-policy-cli links the forbidden-strings scanner, whose fail-closed catch boundaries need panic = \"unwind\""
);

/// What:
///  Why a scan could not be set up or carried out.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   Both causes stop policy evaluation:
///  no rules means nothing was checked,
///  and a
///       pathname that cannot be handed to the scanner was not checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScannerFailure = 'rules-not-loaded' | 'pathname-unrepresentable';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScannerFailure {
    /// The scanner refused to load its rules;
    ///  no partial rule set exists.
    RulesNotLoaded,
    /// A candidate pathname cannot be expressed as a native path on this platform.
    PathnameUnrepresentable,
}

/// What:
///  One scanner failure:
///  its cause and a complete explanation.
///       `String` owns the text (sibling `&str` would borrow a temporary).
/// Why:
///   The explanation of a load failure is the scanner's own redacted message,
///       which names a path or an opaque rule index and never rule text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScannerError extends Error { failure: ScannerFailure; message: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannerError {
    /// Closed cause callers branch on.
    pub failure: ScannerFailure,
    /// Complete diagnostic.
    pub message: String,
}

/// What:
///  `impl std::fmt::Display for ScannerError` supplies Rust's "print me" interface.
/// Why:
///   The executable writes the message with ordinary formatting,
///  not field access.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return this.message; }
/// ```
impl std::fmt::Display for ScannerError {
    /// `&self` borrows this error read-only;
    ///  `&mut` lends the formatter for writing.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `.as_str()` lends the owned message as a borrowed view for the writer.
        return formatter.write_str(self.message.as_str());
    }
}

/// An empty `impl` marks the type as a standard error value.
impl std::error::Error for ScannerError {}

/// What:
///  Which rules files to load,
///  in which order,
///  and whether each one's absence is an error.
///       `Vec<RulesEntry>` is an owned list of the scanner library's own entries
///  (a path and an `optional` flag).
/// Why:
///   The standalone scanner tolerates a missing default rules file when the
///       built-in baseline is on,
///  and never a missing file that was named
///       explicitly unless its entry is optional.
///  The same facts are passed to the library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RulesSource = { entries: { path: string; optional: boolean }[]; explicit: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulesSource {
    /// The runtime rules files in the order they are read,
    ///  each path absolute;
    ///  one entry when `explicit` is false.
    pub entries: Vec<RulesEntry>,
    /// Whether the files were named by configuration or the environment variable instead of being the default location.
    pub explicit: bool,
}

/// What:
///  The loaded scanner.
///  The field is private:
///  the only way to obtain a value is `load`.
/// Why:
///   Holding the loaded rules in one value is what makes "load once,
///  scan many"
///       a property of the type:
///  scanning borrows it and cannot reload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateScanner { readonly #scanner: Scanner }
/// ```
pub struct CandidateScanner {
    /// The scanner library's loaded runtime rules,
    ///  optional baseline and cache warnings.
    scanner: Scanner,
}

/// What:
///  `impl CandidateScanner { ... }` attaches the adapter's operations,
///  like class methods.
/// Why:
///   Loading is the only fallible,
///  expensive step;
///  scanning afterwards is cheap and repeatable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateScanner { static load(rules, builtinRules) {} scan(candidate, bytes) {} }
/// ```
impl CandidateScanner {
    /// What:
    ///  Load the runtime rules file and,
    ///  when asked,
    ///  the built-in baseline.
    ///       `&RulesSource` borrows the selection;
    ///  `Result<T, E>` is "a value or a failure".
    /// Why:
    ///   The scanner's loader validates rules,
    ///  reuses its compiled-rule cache and
    ///       catches its own panics.
    ///  Any failure leaves no scanner at all,
    ///  so nothing
    ///       can be scanned with half a rule set.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static load(rules: RulesSource, builtinRules: boolean): CandidateScanner
    /// ```
    pub fn load(
        rules: &RulesSource,
        builtin_rules: bool,
    ) -> Result<CandidateScanner, ScannerError> {
        // What: `match` on the loader's `Result`; `Err(error)` binds the scanner's failure,
        //       whose `{error}` text is already redacted.
        // Why:  The failure type belongs to a crate this one does not depend on by name,
        //       so it is turned into text here, at the one place it appears.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { return new CandidateScanner(Scanner.load(...)); } catch (error) { throw new ScannerError(...); }
        // ```
        let loaded = match (rules.explicit, rules.entries.as_slice()) {
            // The default location keeps the scanner's one-path contract: absent only under the baseline.
            (false, [only]) => Scanner::load(only.path.as_path(), builtin_rules, false),
            (_, entries) => Scanner::load_entries(entries, builtin_rules),
        };
        match loaded {
            // `Ok(...)` is the success variant.
            Ok(scanner) => return Ok(CandidateScanner { scanner }),
            // `Err(...)` is the failure variant.
            Err(error) => {
                return Err(ScannerError {
                    failure: ScannerFailure::RulesNotLoaded,
                    message: format!(
                        "cli-git could not load the forbidden-strings rules, so no file was scanned: {error}."
                    ),
                });
            }
        }
    }

    /// What:
    ///  The cache diagnostics of this load,
    ///  as the scanner's own fixed-token values.
    ///       `&[CacheWarning]` borrows the list read-only.
    /// Why:
    ///   A stale or missing rule cache is recovered by recompiling,
    ///  which is slow;
    ///       the caller reports that once per invocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get cacheWarnings(): readonly CacheWarning[]
    /// ```
    pub fn cache_warnings(&self) -> &[CacheWarning] {
        return self.scanner.cache_warnings();
    }

    /// What:
    ///  The optional rules files that were absent and skipped by this load,
    ///  in list order.
    ///       `&[PathBuf]` borrows the list read-only.
    /// Why:
    ///   The scanner library has no log subscriber inside cli-git,
    ///  so the caller reports the skipped files.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get skippedOptionalRules(): readonly string[]
    /// ```
    pub fn skipped_optional_rules(&self) -> &[PathBuf] {
        return self.scanner.skipped_optional_rules();
    }

    /// What:
    ///  Scan one candidate's pathname and exact bytes.
    ///       `&[u8]` borrows the bytes;
    ///  the result is the scanner's own `CandidateScan`.
    /// Why:
    ///   The scan's identity is the candidate's position in its version,
    ///  so a
    ///       finding is attributed without reading the pathname,
    ///  which may be masked.
    ///       Fail-closed findings (an engine error,
    ///  a line break in a pathname) are
    ///       returned like any other finding and are never filtered here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scan(candidate: Candidate, bytes: Uint8Array): CandidateScan
    /// ```
    pub fn scan(&self, candidate: &Candidate, bytes: &[u8]) -> Result<CandidateScan, ScannerError> {
        // `let Some(..) = .. else { .. }` unwraps the native path or leaves with a failure.
        let Some(path) = path_from_git_bytes(candidate.path.as_slice()) else {
            return Err(ScannerError {
                failure: ScannerFailure::PathnameUnrepresentable,
                message: format!(
                    "cli-git could not scan candidate {}: its pathname cannot be expressed as a native path on this platform.",
                    candidate.identity.index
                ),
            });
        };
        let logical: &Path = path.as_path();
        return Ok(self.scanner.scan(candidate.identity.index, logical, bytes));
    }
}

/// Scanner controls stay out of the release executable.
#[cfg(test)]
#[path = "scanner_adapter_tests.rs"]
mod tests;
