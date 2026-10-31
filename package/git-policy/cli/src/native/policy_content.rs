//! What:
//!  What one lifecycle offers the content policies,
//!  and the candidates it prepared
//!       for them,
//!  read once and shared by every content policy of the invocation.
//! Why:
//!  A forwarded command other than `git add` has no candidates,
//!  so its content
//!      policies find nothing.
//!  `git add` and the direct commands have candidates,
//!  which
//!      are prepared the first time a content policy reads them:
//!  an invocation whose
//!      content policies are all off or escaped starts no Git process for them.
//!  A failure
//!      to prepare is remembered,
//!  so it is reported once and never retried.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const version = await content.version(lifecycle, facts); for (const c of version.candidates) await content.bytes(c);
//! ```

/// Import the layer's failure type and its causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Import the lookup of content that a parent commit's tree already holds.
use super::candidate_known_content::{ContentBasis, KnownContent, find_known_content};
/// Import the candidate's mode,
///  which a correction keeps.
use super::candidate_object::CandidateMode;
/// Import what a lifecycle asks for and what preparing it yields.
use super::candidate_prediction::{CandidateRequest, PreparedCandidates};
/// Import the store the overlay reads through.
use super::candidate_store::CandidateStore;
/// Import the candidate and the immutable version type.
use super::candidate_version::{Candidate, CandidateVersion};
/// Import the change a converged fix installs,
///  which also records a corrected file.
use super::direct_fix_install::InstallChange;
/// Import the outcome a check returns.
use super::policy_engine::PolicyOutcome;
/// Import the facts interface that prepares candidates.
use super::repository_facts::RepositoryFacts;
/// Import the scanner adapter.
use super::scanner_adapter::CandidateScanner;
/// Import the code of each candidate failure.
use super::scanner_failure_code::candidate_failure_code;
/// Import the scan pass,
///  its byte reader and its failure.
use super::scanner_run::{CandidateBytes, ScanRunError, scan_version_skipping};
/// The scanner library's per-candidate result.
use forbidden_strings::CandidateScan;
/// What:
///  `BTreeMap<K, V>` is a map kept sorted by key (sibling `HashMap` is unordered).
/// Why:
///   Corrected files are keyed by pathname bytes,
///  and their sorted order is Git's
///       byte order,
///  the order a fix summary lists them in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map<string, Uint8Array>, iterated in sorted key order.
/// ```
use std::collections::{BTreeMap, HashSet};
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// `Rc<T>` is a shared,
///  read-only handle;
///  versions and bytes are shared,
///  not copied.
use std::rc::Rc;

/// What:
///  What the current lifecycle offers a content policy.
///  An `enum` is a closed set
///       of named alternatives;
///  `Requested` carries what to prepare.
///  `#[derive(...)]`
///       generates cloning,
///  debug printing and `==`.
/// Why:
///   "There are no files to check" and "there are files,
///  prepared on demand" must
///       never be confused:
///  the first is clean,
///  the second must be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LifecycleContent = { kind: 'none' } | { kind: 'requested'; request: CandidateRequest };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleContent {
    /// The lifecycle has no candidate content,
    ///  as for a forwarded command other than `git add`.
    None,
    /// The lifecycle has candidates;
    ///  they are prepared when a content policy first reads them.
    Requested(
        /// What to prepare.
        CandidateRequest,
    ),
}

/// What:
///  The candidates of one invocation once prepared,
///  or the remembered failure.
///       `Option<Result<..>>` is "not yet asked",
///  "prepared" or "failed".
/// Why:
///   Every content policy of the invocation reads the same version through one store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentState { #prepared?: PreparedCandidates | CandidateError }
/// ```
pub struct ContentState {
    /// The prepared candidates,
    ///  or the failure of the one attempt to prepare them.
    prepared: Option<Result<PreparedCandidates, CandidateError>>,
    /// Each corrected file by pathname:
    ///  its mode,
    ///  original bytes and corrected bytes.
    ///  Every other file keeps its object's bytes.
    corrected: BTreeMap<Vec<u8>, InstallChange>,
    /// The corrections policies proposed in the current pass,
    ///  in proposal order.
    proposals: Vec<Correction>,
}

/// What:
///  One proposed full-content correction:
///  the file,
///  the bytes it held when the policy
///       read it,
///  and the bytes it should hold.
///  `#[derive(...)]` generates cloning,
///  debug
///       printing and `==`.
/// Why:
///   A correction replaces a file's whole content,
///  so applying it needs no patch
///       format;
///  the bytes it replaces are kept so the original of a file is known
///       without reading it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Correction = { path: Uint8Array; mode: CandidateMode; before: Uint8Array; after: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Correction {
    /// The pathname,
    ///  relative to the repository's top level,
    ///  as Git's raw bytes.
    pub path: Vec<u8>,
    /// The file's mode,
    ///  which the correction keeps.
    pub mode: CandidateMode,
    /// The bytes the file held when the policy read it.
    pub before: Rc<[u8]>,
    /// The bytes the file should hold.
    pub after: Rc<[u8]>,
}

/// What:
///  The prepared store with the corrections laid over it,
///  as a scan's byte reader.
///       `'a` names how long both borrows last.
/// Why:
///   A later pass of a direct fix scans the corrected bytes,
///  not the objects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader: CandidateBytes = { bytes: (c) => corrected.get(c.path) ?? store.bytes(c) };
/// ```
struct OverlaidStore<'a> {
    /// The store bound to the private index.
    store: &'a mut CandidateStore,
    /// The corrected files,
    ///  by pathname.
    corrected: &'a BTreeMap<Vec<u8>, InstallChange>,
}

/// What:
///  `impl CandidateBytes for OverlaidStore<'_>` reads corrected bytes first.
/// Why:
///   The one byte rule every reader of a direct fix shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OverlaidStore implements CandidateBytes {}
/// ```
impl CandidateBytes for OverlaidStore<'_> {
    /// What:
    ///  The corrected bytes of the candidate's file,
    ///  or its object's bytes.
    /// Why:
    ///   See `overlaid_bytes`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bytes(candidate) { return overlaidBytes(this.store, this.corrected, candidate); }
    /// ```
    fn candidate_bytes(&mut self, candidate: &Candidate) -> Result<Rc<[u8]>, CandidateError> {
        return overlaid_bytes(self.store, self.corrected, candidate);
    }
}

/// What:
///  The corrected bytes of a candidate's file when a correction gave it some,
///  else
///       the bytes of its object through the store.
/// Why:
///   Every policy of a later pass must read what the earlier passes made of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overlaidBytes = (store, corrected, candidate) => corrected.get(candidate.path) ?? store.bytes(candidate);
/// ```
fn overlaid_bytes(
    store: &mut CandidateStore,
    corrected: &BTreeMap<Vec<u8>, InstallChange>,
    candidate: &Candidate,
) -> Result<Rc<[u8]>, CandidateError> {
    // `if let Some(found) = ...get(..)` runs only when the file was corrected.
    if let Some(found) = corrected.get(&candidate.path) {
        return Ok(Rc::clone(&found.replacement));
    }
    return store.bytes(candidate);
}

/// What:
///  The outcome of a content policy whose candidates could not be prepared or read.
///       `&CandidateError` borrows the failure;
///  its code follows its cause.
/// Why:
///   Every cause but a sequencing defect is unreadable content;
///  the message is the
///       layer's own,
///  which never contains a candidate's pathname.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const candidateFailure = (error: CandidateError): PolicyOutcome => ({ kind: 'failed', code: candidateFailureCode(error.failure), message: error.message });
/// ```
pub fn candidate_failure(error: &CandidateError) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code: candidate_failure_code(error.failure),
        // `.clone()` copies the message the remembered failure keeps.
        message: error.message.clone(),
        // `None`: the layer's failures name no candidate.
        path: None,
    };
}

/// What:
///  `impl ContentState { ... }` attaches the constructor and the two reads.
/// Why:
///   Preparation and reading stay behind one owner,
///  so no policy can read bytes of a
///       version it did not get from here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentState { version(lifecycle, facts) {} bytes(candidate) {} }
/// ```
impl ContentState {
    /// What:
    ///  A state that has prepared nothing.
    /// Why:
    ///   Preparation waits until a content policy reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor() {}
    /// ```
    pub fn new() -> ContentState {
        // `None` is the "absent" variant: nothing has been asked; nothing is corrected yet.
        return ContentState {
            prepared: None,
            corrected: BTreeMap::new(),
            proposals: Vec::new(),
        };
    }

    /// What:
    ///  The candidate version,
    ///  prepared on first use;
    ///  `Ok(None)` when the lifecycle
    ///       has no candidates.
    ///  `<F: RepositoryFacts>` accepts any facts provider;
    ///       `Err(error)` is the candidate layer's failure,
    ///  remembered for later calls.
    /// Why:
    ///   A direct command prepares its scope before any policy runs,
    ///  as the installed
    ///       wrapper does,
    ///  and reports a failure as a lifecycle failure of its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async prepare(lifecycle: LifecycleContent, facts: RepositoryFacts): Promise<CandidateVersion | undefined>; // throws CandidateError
    /// ```
    pub fn prepare<F: RepositoryFacts>(
        &mut self,
        lifecycle: &LifecycleContent,
        facts: &mut F,
    ) -> Result<Option<Rc<CandidateVersion>>, CandidateError> {
        let LifecycleContent::Requested(request) = lifecycle else {
            return Ok(None);
        };
        if self.prepared.is_none() {
            // `Some(...)` stores the one attempt's result, success or failure.
            self.prepared = Some(facts.candidates(request));
        }
        // `match` on the remembered attempt; `&self.prepared` borrows it read-only.
        match &self.prepared {
            Some(Ok(prepared)) => return Ok(Some(Rc::clone(&prepared.version))),
            // `.clone()` copies the remembered failure for this caller.
            Some(Err(error)) => return Err(error.clone()),
            None => return Err(not_prepared_error()),
        }
    }

    /// What:
    ///  The candidate version for a content policy:
    ///  as `prepare`,
    ///  with a failure
    ///       turned into the outcome the policy returns as it is.
    /// Why:
    ///   The first content policy that reads pays for the preparation;
    ///  later ones
    ///       share it,
    ///  and a failure is reported the same way to each.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async version(lifecycle: LifecycleContent, facts: RepositoryFacts): Promise<CandidateVersion | undefined>; // throws PolicyOutcome
    /// ```
    pub fn version<F: RepositoryFacts>(
        &mut self,
        lifecycle: &LifecycleContent,
        facts: &mut F,
    ) -> Result<Option<Rc<CandidateVersion>>, PolicyOutcome> {
        match self.prepare(lifecycle, facts) {
            Ok(found) => return Ok(found),
            Err(error) => return Err(candidate_failure(&error)),
        }
    }

    /// What:
    ///  The bytes of a candidate of the prepared version,
    ///  with corrections applied.
    /// Why:
    ///   Bytes come from the store bound to the private index,
    ///  never the live worktree,
    ///       unless a correction of this invocation replaced them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async bytes(candidate: Candidate): Promise<Uint8Array>;
    /// ```
    pub fn bytes(&mut self, candidate: &Candidate) -> Result<Rc<[u8]>, PolicyOutcome> {
        // `&mut self.prepared` lends the remembered attempt for reading through its store.
        match &mut self.prepared {
            Some(Ok(prepared)) => {
                match overlaid_bytes(&mut prepared.store, &self.corrected, candidate) {
                    Ok(read) => return Ok(read),
                    Err(error) => return Err(candidate_failure(&error)),
                }
            }
            Some(Err(_)) | None => return Err(candidate_failure(&not_prepared_error())),
        }
    }

    /// What:
    ///  The given candidate pathnames whose `text` Git attribute is unset,
    ///  read from the state the candidates were listed from.
    ///  `Err(outcome)` is the policy outcome of a lookup that failed.
    /// Why:
    ///   A file that declares `-text` keeps its exact bytes.
    ///  Only the paths a policy would change are asked about,
    ///  so a commit that needs no change starts no attribute query.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async textUnset(paths: Uint8Array[]): Promise<Set<string>>; // throws PolicyOutcome
    /// ```
    pub fn text_unset(&self, paths: &[&[u8]]) -> Result<HashSet<Vec<u8>>, PolicyOutcome> {
        // `&self.prepared` borrows the remembered attempt read-only.
        match &self.prepared {
            Some(Ok(prepared)) => match prepared.attributes.text_unset(paths) {
                Ok(unset) => return Ok(unset),
                Err(error) => return Err(candidate_failure(&error)),
            },
            Some(Err(_)) | None => return Err(candidate_failure(&not_prepared_error())),
        }
    }

    /// What:
    ///  The real index the prepared candidates were copied from,
    ///  once prepared.
    /// Why:
    ///   A direct fix proves that installing its corrections left that index unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get realIndexPath(): string | undefined;
    /// ```
    pub fn real_index(&self) -> Option<&Path> {
        // `match &self.prepared` borrows the remembered attempt read-only.
        match &self.prepared {
            Some(Ok(prepared)) => return Some(prepared.real_index.as_path()),
            Some(Err(_)) | None => return None,
        }
    }
}

/// What:
///  `impl ContentState { ... }` continued:
///  corrections of a fixing lifecycle.
/// Why:
///   A policy proposes,
///  the fix loop applies after the pass,
///  and later passes read
///       the corrected bytes;
///  nothing reaches the worktree until the fix installs them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentState { propose(c) {} takeProposals() {} apply(cs) {} }
/// ```
impl ContentState {
    /// What:
    ///  Record one correction a policy proposes in the current pass.
    /// Why:
    ///   Corrections wait until the pass ends,
    ///  so every policy of the pass reads the
    ///       same bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// propose(correction: Correction): void;
    /// ```
    pub fn propose(&mut self, correction: Correction) {
        self.proposals.push(correction);
    }

    /// What:
    ///  Hand over the current pass's proposals and forget them.
    ///       `std::mem::take` moves the list out and leaves an empty one behind.
    /// Why:
    ///   Each pass starts with no proposals,
    ///  and a proposal is applied at most once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// takeProposals(): Correction[];
    /// ```
    pub fn take_proposals(&mut self) -> Vec<Correction> {
        return std::mem::take(&mut self.proposals);
    }

    /// What:
    ///  Apply corrections in order.
    ///  A file whose correction restores its original
    ///       bytes is no longer corrected.
    /// Why:
    ///   The corrected map then equals another state's map exactly when both states
    ///       hold the same bytes in every file,
    ///  which the fix loop's cycle check needs.
    ///  A
    ///       file's original is the bytes its first correction replaced;
    ///  a file restored
    ///       and corrected again replaces those same bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// apply(corrections: Correction[]): void;
    /// ```
    pub fn apply(&mut self, corrections: Vec<Correction>) {
        for correction in corrections {
            let original: Rc<[u8]> = match self.corrected.get(&correction.path) {
                Some(file) => Rc::clone(&file.original),
                None => correction.before,
            };
            if *original == *correction.after {
                self.corrected.remove(&correction.path);
            } else {
                self.corrected.insert(
                    correction.path.clone(),
                    InstallChange {
                        path: correction.path,
                        mode: correction.mode,
                        original,
                        replacement: correction.after,
                    },
                );
            }
        }
    }

    /// What:
    ///  A copy of every corrected file,
    ///  by pathname.
    ///  Copies share the bytes.
    /// Why:
    ///   The fix loop keeps one per changed state to recognize a repeated state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// correctedState(): Map<string, InstallChange>;
    /// ```
    pub fn corrected_state(&self) -> BTreeMap<Vec<u8>, InstallChange> {
        return self.corrected.clone();
    }

    /// What:
    ///  Every corrected file,
    ///  in Git's byte order of pathnames.
    /// Why:
    ///   These are the changes a converged fix installs,
    ///  and the order its summary
    ///       lists them in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// installChanges(): InstallChange[];
    /// ```
    pub fn install_changes(&self) -> Vec<InstallChange> {
        // `.values().cloned().collect()` copies each change in key order into a list.
        return self.corrected.values().cloned().collect();
    }
}

/// What:
///  `impl ContentState { ... }` continued:
///  the scan of every scannable candidate.
/// Why:
///   The scan pass reads bytes through the store the candidates were prepared in,
///       with this invocation's corrections laid over it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentState { scan(scanner, rulesPath) {} }
/// ```
impl ContentState {
    /// What:
    ///  Scan the prepared version through `scanner`.
    ///  `&[Vec<u8>]` is the rules
    ///       file's own pathname,
    ///  which is not scanned,
    ///  or nothing.
    /// Why:
    ///   `scan_version` is the candidate layer's one scan pass;
    ///  this lends it the store
    ///       and the version together.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async scan(scanner: CandidateScanner, rulesPath?: Buffer): Promise<CandidateScan[]>;
    /// ```
    pub fn scan(
        &mut self,
        scanner: &CandidateScanner,
        rules_paths: &[Vec<u8>],
    ) -> Result<Vec<CandidateScan>, ScanRunError> {
        return self.scan_skipping(scanner, rules_paths, &KnownContent::default());
    }

    /// What:
    ///  `scan`,
    ///  except that the candidates in `known` are given to the scanner without bytes.
    ///       `&KnownContent` lists them by position in the prepared version.
    /// Why:
    ///   A candidate whose content a parent already holds has its pathname checked and its
    ///       bytes left unread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async scanSkipping(scanner: CandidateScanner, rulesPath: Buffer | undefined, known: KnownContent): Promise<CandidateScan[]>;
    /// ```
    pub fn scan_skipping(
        &mut self,
        scanner: &CandidateScanner,
        rules_paths: &[Vec<u8>],
        known: &KnownContent,
    ) -> Result<Vec<CandidateScan>, ScanRunError> {
        match &mut self.prepared {
            // The store is lent for writing and the version for reading: two separate fields.
            Some(Ok(prepared)) => {
                let mut reader: OverlaidStore<'_> = OverlaidStore {
                    store: &mut prepared.store,
                    corrected: &self.corrected,
                };
                return scan_version_skipping(
                    scanner,
                    &mut reader,
                    &prepared.version,
                    rules_paths,
                    known,
                );
            }
            Some(Err(_)) | None => return Err(ScanRunError::Candidate(not_prepared_error())),
        }
    }

    /// What:
    ///  The candidates of the prepared version whose content a parent commit's tree already holds,
    ///       as this lifecycle's request defines the parents.
    ///  `&[Vec<u8>]` is the rules files' own pathnames.
    /// Why:
    ///   A file this invocation corrected holds new bytes whatever its object is,
    ///  so it is never known.
    ///       Nothing is listed before preparation or after a failed one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async knownContent(request: CandidateRequest, rulesPath?: Buffer): Promise<KnownContent>; // throws CandidateError
    /// ```
    pub fn known_content(
        &mut self,
        request: &CandidateRequest,
        rules_paths: &[Vec<u8>],
    ) -> Result<KnownContent, CandidateError> {
        match &mut self.prepared {
            Some(Ok(prepared)) => {
                return find_known_content(
                    &mut prepared.store,
                    &prepared.version,
                    &ContentBasis::of(request),
                    rules_paths,
                    &self.corrected,
                );
            }
            Some(Err(_)) | None => return Err(not_prepared_error()),
        }
    }
}

/// What: `impl ContentState { ... }` continued: the whole candidate state, for a policy
///       that reads files beyond the candidates.
/// Why:  The dependent-version policy lists every tracked file of the private index and
///       reads files that are not candidates, through the same store and the same
///       corrections every other content policy reads through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentState { storeWithCorrections() {} }
/// ```
impl ContentState {
    /// What: The store the candidates were prepared in, with this invocation's corrected
    ///       files; nothing before preparation or after a failed one. The two borrows are
    ///       of separate fields, so both can be held at once.
    /// Why:  A later pass of a direct fix must read what the earlier passes made of every
    ///       file, candidate or not.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// storeWithCorrections(): [CandidateStore, Map<string, InstallChange>] | undefined;
    /// ```
    pub fn store_with_corrections(
        &mut self,
    ) -> Option<(&mut CandidateStore, &BTreeMap<Vec<u8>, InstallChange>)> {
        match &mut self.prepared {
            Some(Ok(prepared)) => return Some((&mut prepared.store, &self.corrected)),
            Some(Err(_)) | None => return None,
        }
    }
}

/// What: The failure of a read made before any version was prepared.
/// Why:  Only a defect in a calling policy can cause it. It is reported as a stale
///       candidate, whose code is `policy-incomplete`: the policy's own sequencing
///       failed, and the read must still not look clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notPreparedError = () => new CandidateError('stale-candidate', '...');
/// ```
fn not_prepared_error() -> CandidateError {
    return CandidateError::new(
        CandidateFailure::StaleCandidate,
        "cli-git read candidate content before the candidates were prepared; this is a defect in cli-git.",
    );
}

/// What:
///  `impl Default for ContentState` lets `ContentState::default()` build the empty state.
/// Why:
///   Clippy asks a type with a `new()` without arguments to provide `Default` too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// static default(): ContentState { return new ContentState(); }
/// ```
impl Default for ContentState {
    /// What:
    ///  The state that has prepared nothing.
    ///  Why:
    ///  same as `new`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static default(): ContentState
    /// ```
    fn default() -> ContentState {
        return ContentState::new();
    }
}

/// Preparation,
///  sharing and failure controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_content_tests.rs"]
mod tests;
