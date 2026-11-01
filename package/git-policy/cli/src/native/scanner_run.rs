//! What:
//!  Scan every eligible candidate of one version through the loaded scanner.
//! Why:
//!  This is the whole forbidden-strings check of one policy pass:
//!  candidate bytes
//!      come from the candidate store (staged or committed,
//!  never the worktree),
//!  go to
//!      the scanner in memory,
//!  and the scanner's own results come back.
//!  No temporary
//!      file is written and no process is started per file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scans = await scanVersion(scanner, store, version, rulesPath);
//! ```

/// Import the candidate layer's failure,
///  store and version.
use super::candidate_error::CandidateError;
use super::candidate_store::CandidateStore;
use super::candidate_version::{Candidate, CandidateVersion};
/// Import the candidates whose content a parent already holds.
use super::candidate_known_content::KnownContent;
/// Import the loaded scanner and its failure.
use super::scanner_adapter::{CandidateScanner, ScannerError};
/// Import the eligibility decision.
use super::scanner_selection::is_scannable;
/// The scanner library's per-candidate result,
///  returned unchanged.
use forbidden_strings::CandidateScan;
/// `Rc<T>` is a shared,
///  read-only handle to one heap value (siblings `Box<T>`,
///  `Arc<T>`).
use std::rc::Rc;

/// What:
///  Why a scan pass stopped:
///  the candidate layer failed,
///  or the scanner did.
///       Each variant carries the original typed failure.
/// Why:
///   Either way some file was not checked,
///  so the pass has no result.
///  Keeping the
///       original failure lets the caller report its own cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanRunError = { kind: 'candidate'; error: CandidateError } | { kind: 'scanner'; error: ScannerError };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanRunError {
    /// Candidate bytes could not be read.
    Candidate(CandidateError),
    /// The scanner could not be given a candidate.
    Scanner(ScannerError),
}

/// What:
///  `impl std::fmt::Display for ScanRunError` supplies Rust's "print me" interface.
/// Why:
///   The message is the wrapped failure's own,
///  unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return this.error.message; }
/// ```
impl std::fmt::Display for ScanRunError {
    /// `&self` borrows this error read-only;
    ///  `&mut` lends the formatter for writing.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `match self` reads the variant; each arm prints the message of the failure it carries.
        match self {
            ScanRunError::Candidate(error) => return formatter.write_str(error.message.as_str()),
            ScanRunError::Scanner(error) => return formatter.write_str(error.message.as_str()),
        }
    }
}

/// An empty `impl` marks the type as a standard error value.
impl std::error::Error for ScanRunError {}

/// What:
///  Where a scan pass reads candidate bytes.
///  A `trait` is an interface.
/// Why:
///   The candidate store reads the version's objects;
///  a direct fix reads the same
///       objects with its in-memory corrections laid over them,
///  and its later passes
///       must scan the corrected bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CandidateBytes { bytes(candidate: Candidate): Promise<Uint8Array> }
/// ```
pub trait CandidateBytes {
    /// What:
    ///  The bytes the candidate holds for this reader.
    ///  Why:
    ///  the scan's only input.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bytes(candidate: Candidate): Promise<Uint8Array>; // throws CandidateError
    /// ```
    fn candidate_bytes(&mut self, candidate: &Candidate) -> Result<Rc<[u8]>, CandidateError>;
}

/// What:
///  `impl CandidateBytes for CandidateStore` reads through the store itself.
/// Why:
///   Without corrections the store's objects are the bytes to scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateStore implements CandidateBytes {}
/// ```
impl CandidateBytes for CandidateStore {
    /// What:
    ///  The store's bytes for the candidate.
    ///  Why:
    ///  `CandidateStore::bytes` is the read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bytes(candidate) { return this.read(candidate); }
    /// ```
    fn candidate_bytes(&mut self, candidate: &Candidate) -> Result<Rc<[u8]>, CandidateError> {
        return self.bytes(candidate);
    }
}

/// What:
///  Scan each eligible candidate and return the scanner's result for every one scanned.
///       `&mut dyn CandidateBytes` lends any byte reader for changing (it may read objects);
///       `&[Vec<u8>]` is "the rules files' candidate pathnames, none when no file is tracked";
///       `Result<T, E>` is "a value or a failure".
/// Why:
///   Results are returned for clean candidates too,
///  so the caller can tell
///       "scanned and clean" from "not scanned".
///  The first failure ends the pass with
///       no partial result,
///  because a partial result would look like a clean pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanVersion(scanner, store, version, rulesPath?): Promise<CandidateScan[]>;
/// ```
pub fn scan_version(
    scanner: &CandidateScanner,
    store: &mut dyn CandidateBytes,
    version: &CandidateVersion,
    rules_paths: &[Vec<u8>],
) -> Result<Vec<CandidateScan>, ScanRunError> {
    return scan_version_skipping(scanner, store, version, rules_paths, &KnownContent::default());
}

/// What:
///  `scan_version`,
///  except that a candidate whose content a parent already holds is given to the scanner without
///       bytes.
///       `&KnownContent` lists those candidates by position.
/// Why:
///   The scanner reads the pathname of every candidate it is given,
///  so a known candidate still has its
///       new pathname checked,
///  and its bytes are neither read from the store nor scanned.
///  The result list
///       keeps one entry per eligible candidate,
///  so "scanned" and "skipped" differ only in the bytes behind
///       the entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanVersionSkipping(scanner, store, version, rulesPath, known): Promise<CandidateScan[]>;
/// ```
pub fn scan_version_skipping(
    scanner: &CandidateScanner,
    store: &mut dyn CandidateBytes,
    version: &CandidateVersion,
    rules_paths: &[Vec<u8>],
    known: &KnownContent,
) -> Result<Vec<CandidateScan>, ScanRunError> {
    // `Vec::new()` is an empty growable list; `mut` lets the loop append to it.
    let mut scans: Vec<CandidateScan> = Vec::new();
    for candidate in version.candidates() {
        if !is_scannable(candidate, rules_paths) {
            continue;
        }
        // `.is_some()` is true when the candidate's content is already in a parent's tree.
        if known.entry(candidate.identity.index).is_some() {
            // An empty byte slice leaves the scanner only the pathname to check.
            match scanner.scan(candidate, &[]) {
                Ok(scan) => scans.push(scan),
                Err(error) => return Err(ScanRunError::Scanner(error)),
            }
            continue;
        }
        // `match` on the read's `Result`: `Ok(read)` unwraps the bytes, `Err(error)` ends the pass.
        let bytes: Rc<[u8]> = match store.candidate_bytes(candidate) {
            Ok(read) => read,
            // `Err(...)` is the failure variant.
            Err(error) => return Err(ScanRunError::Candidate(error)),
        };
        // `&bytes` lends the shared bytes as a plain byte slice.
        match scanner.scan(candidate, &bytes) {
            Ok(scan) => scans.push(scan),
            Err(error) => return Err(ScanRunError::Scanner(error)),
        }
    }
    // `Ok(...)` is the success variant.
    return Ok(scans);
}

#[cfg(test)]
#[path = "scanner_run_tests.rs"]
mod tests;

/// Which candidates a pass skips,
///  and how a pass ends on failure.
#[cfg(test)]
#[path = "scanner_run_selection_tests.rs"]
mod selection_tests;
