//! What:
//!  The per-invocation owner of candidate versions and candidate bytes.
//! Why:
//!  Policies read committed or staged content,
//!  never whatever is in the live
//!      worktree.
//!  A version costs one listing process,
//!  its bytes come from one
//!      long-lived object reader,
//!  and neither count grows with the number of files.
//!      After a fix or replay changed the index,
//!  `invalidate` retires every version.
//!  The listings a caller asks for and the Git commands behind them are
//!  `crate::candidate_store_listings`'s;
//!  this file holds the store,
//!  the one run every listing makes,
//!  the listing one commit keeps,
//!  the bytes a candidate holds and the generation that retires it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const store = new CandidateStore(git, prefix, overlay);
//! // const version = await store.version({ kind: 'staged-against-head' });
//! // const bytes = await store.bytes(version.candidates[0]);
//! ```

/// Import the layer's failure type and its closed list of causes.
use crate::candidate_error::{
    CandidateError,
    CandidateFailure,
};
/// The candidate modes and the validated object name.
use crate::candidate_object::{
    CandidateMode,
    ObjectId,
};
/// Import the long-lived object reader.
use crate::candidate_reader::{
    ObjectReader,
    start_object_reader,
};
/// The record a version is built from.
use crate::candidate_record::CandidateRecord;
/// The tree record a commit's listing holds.
use crate::candidate_tree::TreeRecord;
/// Import the version types and builder.
use crate::candidate_version::{
    Candidate,
    CandidateSource,
    CandidateVersion,
    build_version,
};
/// The captured-query runner and the two helpers of its output.
use crate::git_metadata::{
    MetadataOutput,
    read_stopped_text,
    run_metadata_git_without,
};
/// `OsString` is owned operating-system text of raw OS bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  `Rc<T>` is a shared,
///  read-only handle to one heap value,
///  counted so the value
///       lives until its last handle is gone (siblings:
///  `Box<T>`,
///  a single owner,
///  and
///       `Arc<T>`,
///  the thread-safe form).
/// Why:
///   Several policies hold the same version and the same bytes at once.
///  Nothing
///       here crosses threads,
///  so `Arc` would add cost for no benefit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Every object reference in TS already behaves like this.
/// ```
use std::rc::Rc;

/// What:
///  Everything one invocation needs to list and read candidates.
///       `Option<ObjectReader>` is "a running reader or nothing";
///  `u64` is an unsigned
///       64-bit counter (siblings `u32`,
///  `usize`).
/// Why:
///   The reader is started only when something needs it,
///  so an invocation that
///       reads no object starts no reader.
///  `u64` cannot run out within a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateStore { #reader?: ObjectReader; #generation = 0n; #versions: [CandidateSource, CandidateVersion][] = [] }
/// ```
pub struct CandidateStore {
    /// The real Git executable every listing and the reader start.
    real_git: PathBuf,
    /// The caller's arguments before the subcommand,
    ///  replayed so Git selects the same repository.
    global_prefix: Vec<OsString>,
    /// Added environment pairs,
    ///  such as a private index file or object directory.
    overlay: Vec<(OsString, OsString)>,
    /// The long-lived object reader,
    ///  once started.
    reader: Option<ObjectReader>,
    /// Counts invalidations;
    ///  candidates of an earlier generation are refused.
    ///  The listings this store answers read it too,
    ///  so a version they build carries the generation it was built in.
    pub(super) generation: u64,
    /// Versions already listed in this generation,
    ///  by source.
    pub(super) versions: Vec<(CandidateSource, Rc<CandidateVersion>)>,
    /// The last commit this store listed with its files,
    ///  and that listing.
    ///  One entry keeps the memory to one commit's listing,
    ///  and a commit's name fixes its tree,
    ///  so the listing of a commit never changes.
    listed: Option<(ObjectId, std::sync::Arc<Vec<TreeRecord>>)>,
    /// The commit every "against `HEAD`" listing compares with instead of the live `HEAD`:
    ///  `Some(Some(commit))` for a recorded base,
    /// `Some(None)` for a recorded unborn branch,
    /// `None` for live `HEAD`.
    baseline: Option<Option<ObjectId>>,
}

/// What:
///  Build the failure of a listing command that could not be started or exited unsuccessfully.
/// Why:
///   The message names the Git operation,
///  so the person can rerun it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listingFailure(failure: CandidateFailure, operation: string, detail: string): CandidateError;
/// ```
pub(crate) fn listing_failure(
    failure: CandidateFailure,
    operation: &str,
    detail: &str,
) -> CandidateError {
    return CandidateError::new(
        failure,
        format!(
            "monochromatic-git-policy-cli could not list candidate files: git {operation} {detail}"
        )
        .as_str(),
    );
}

/// What:
///  `impl CandidateStore { ... }` attaches the store's operations,
///  like class methods.
/// Why:
///   The store is the only place that starts Git for candidates,
///  which is what
///       keeps the process count bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateStore { version(source) {} bytes(candidate) {} invalidate() {} }
/// ```
impl CandidateStore {
    /// What:
    ///  Create a store without starting any process.
    ///       `&[OsString]` borrows the global arguments;
    ///  `&[(OsString, OsString)]` borrows
    ///       environment pairs;
    ///  `.to_vec()` and `.to_path_buf()` copy them into owned storage.
    /// Why:
    ///   The store outlives the caller's borrowed arguments,
    ///  and commands that
    ///       never ask for candidates pay nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(realGit: string, globalPrefix: string[], overlay: [string, string][])
    /// ```
    pub fn new(
        real_git: &Path,
        global_prefix: &[OsString],
        overlay: &[(OsString, OsString)],
    ) -> CandidateStore {
        return CandidateStore {
            real_git: real_git.to_path_buf(),
            global_prefix: global_prefix.to_vec(),
            overlay: overlay.to_vec(),
            // `None` is the "absent" variant: no reader yet.
            reader: None,
            generation: 0,
            versions: Vec::new(),
            listed: None,
            baseline: None,
        };
    }

    /// What:
    ///  Compare every "against `HEAD`" listing with a recorded base instead of the live `HEAD`.
    /// Why:
    ///   A commit transaction reads its candidates against the base it recorded at invocation,
    ///  because another landing may have moved `HEAD` since.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listChangedIndexPaths({ baseRevision }) // never live HEAD
    /// ```
    #[must_use]
    pub fn with_baseline(
        mut self,
        baseline: Option<ObjectId>,
    ) -> CandidateStore {
        self.baseline = Some(baseline);
        return self;
    }

    /// What:
    ///  The commit the "against `HEAD`" listings compare with:
    ///  the recorded baseline when one was set,
    /// otherwise the commit `HEAD` names now.
    /// Why:
    ///   One place decides the baseline,
    /// so the listings and the `HEAD` tree always agree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const baseline = recorded ?? await headCommit();
    /// ```
    pub(crate) fn baseline_commit(&mut self) -> Result<Option<ObjectId>, CandidateError> {
        if let Some(recorded) = &self.baseline {
            return Ok(recorded.clone());
        }
        return self.reader()?.head_commit();
    }

    /// What:
    ///  The running reader,
    ///  started on first use.
    ///  `&mut ObjectReader` lends it for changing.
    /// Why:
    ///   One reader serves every object read of the invocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// #reader(): ObjectReader { return (this.#reader ??= startObjectReader(...)); }
    /// ```
    fn reader(&mut self) -> Result<&mut ObjectReader, CandidateError> {
        if self.reader.is_none() {
            // A trailing `?` returns a start failure to our caller, or unwraps the reader.
            let started: ObjectReader = start_object_reader(
                self.real_git.as_path(),
                self.global_prefix.as_slice(),
                self.overlay.as_slice(),
            )?;
            // `Some(...)` is the "present" variant.
            self.reader = Some(started);
        }
        // `.as_mut()` lends the stored reader; `let Some(..) = .. else` unwraps it.
        let Some(reader) = self.reader.as_mut() else {
            // `Err(...)` is the failure variant. The reader was stored just above, so this is not expected.
            return Err(CandidateError::new(
                CandidateFailure::ReaderEnded,
                "monochromatic-git-policy-cli could not read a Git object: the object reader was not started.",
            ));
        };
        // `Ok(...)` is the success variant.
        return Ok(reader);
    }

    /// What:
    ///  Run one listing command and return its standard output.
    ///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
    /// Why:
    ///   Listings print raw pathname bytes.
    ///  A command that fails lists nothing
    ///       trustworthy,
    ///  so its output is discarded and Git's own message is reported.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #runListing(operation: string, args: string[]): Promise<Buffer>
    /// ```
    pub(crate) fn run_listing(
        &self,
        operation: &str,
        command: &[OsString],
    ) -> Result<Vec<u8>, CandidateError> {
        // `.clone()` copies the stored prefix; `.extend_from_slice(..)` appends the command after it.
        let mut arguments: Vec<OsString> = self.global_prefix.clone();
        arguments.extend_from_slice(command);
        // Every listing names a pathspec the wrapper wrote, so no pathspec mode the caller set may reach it:
        // literal mode would read it as a file name, case-insensitive matching would widen it, and two glob modes
        // at once make Git refuse the listing and fail the judgement that asked for it.
        // `match` on the run's `Result`: `Err(error)` means Git could not be started.
        let output: MetadataOutput = match run_metadata_git_without(
            self.real_git.as_path(),
            arguments.as_slice(),
            self.overlay.as_slice(),
            crate::conflict_side_modes::MODE_VARIABLES,
        ) {
            Ok(captured) => captured,
            Err(error) => {
                // A read the wrapper ended at its ceiling says what blocked it and what stayed as it was.
                if let Some(stopped) = read_stopped_text(operation, &error) {
                    return Err(CandidateError::new(
                        CandidateFailure::GitStopped,
                        stopped.as_str(),
                    ));
                }
                return Err(listing_failure(
                    CandidateFailure::GitNotStarted,
                    operation,
                    format!("could not be started: {error}.").as_str(),
                ));
            }
        };
        if !output.success {
            // `String::from_utf8_lossy` decodes Git's message for display, replacing invalid bytes.
            return Err(listing_failure(
                CandidateFailure::GitFailed,
                operation,
                format!(
                    "failed: {}",
                    String::from_utf8_lossy(output.stderr.as_slice()).trim()
                )
                .as_str(),
            ));
        }
        return Ok(output.stdout);
    }

    /// What:
    ///  The listing this store kept for `commit`,
    ///  when it kept one.
    /// Why:
    ///   A commit's name fixes its tree,
    ///  so a listing made by this store answers every later
    ///       request for the same commit without another Git process.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// keptListing(commit): TreeRecord[] | undefined
    /// ```
    pub(crate) fn kept_listing(
        &self,
        commit: &ObjectId,
    ) -> Option<std::sync::Arc<Vec<TreeRecord>>> {
        let (listed, records) = self.listed.as_ref()?;
        if listed != commit {
            return None;
        }
        return Some(std::sync::Arc::clone(records));
    }

    /// What:
    ///  Keep `records` as this store's listing of `commit`.
    /// Why:
    ///   The store that listed a commit answers the next request for it;
    ///  no lock and no process-wide
    ///       state is needed,
    ///  and the memory stays one commit's listing per store.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// keepListing(commit, records) { this.listed = { commit, records }; }
    /// ```
    pub(crate) fn keep_listing(
        &mut self,
        commit: &ObjectId,
        records: &std::sync::Arc<Vec<TreeRecord>>,
    ) {
        self.listed = Some((commit.clone(), std::sync::Arc::clone(records)));
    }

    /// What:
    /// The commit the "against `HEAD`" listings compare with:
    /// the recorded base or the commit
    ///       `HEAD` names now;
    /// nothing when the branch has no commit yet.
    /// Why:
    ///  A policy that skips content the baseline already holds names the commit in its log
    ///       record and lists its files through `commit_tree_records`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async baselineReference(): Promise<ObjectId | undefined>
    /// ```
    pub fn baseline_reference(&mut self) -> Result<Option<ObjectId>, CandidateError> {
        return self.baseline_commit();
    }

    /// What:
    /// A version of the current generation built from records the caller derived.
    ///       `Vec<CandidateRecord>` is taken by value:
    /// the records move into the version.
    /// Why:
    ///  A predicted staging operation has no single listing command;
    /// its candidates
    ///       come from comparing index states.
    /// The version is not remembered by source,
    ///       because no source names it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// derivedVersion(records: CandidateRecord[]): CandidateVersion
    /// ```
    pub fn derived_version(
        &mut self,
        records: Vec<CandidateRecord>,
    ) -> Rc<CandidateVersion> {
        return Rc::new(build_version(self.generation, records));
    }

    /// What:
    ///  The exact bytes a candidate holds in its version.
    ///       `Rc<[u8]>` is a shared read-only byte list.
    /// Why:
    ///   Bytes come from the object the listing named,
    ///  so an edit to the worktree
    ///       file after staging is invisible here.
    ///  A deleted path has no bytes.
    ///  A
    ///       gitlink has no blob in this repository,
    ///  so its bytes are the submodule
    ///       commit's name,
    ///  as in the TypeScript wrapper.
    ///  A candidate of a retired
    ///       version is refused instead of answered with bytes that may be outdated.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async bytes(candidate: Candidate): Promise<Uint8Array>
    /// ```
    pub fn bytes(
        &mut self,
        candidate: &Candidate,
    ) -> Result<Rc<[u8]>, CandidateError> {
        if candidate.identity.generation != self.generation {
            return Err(CandidateError::new(
                CandidateFailure::StaleCandidate,
                "monochromatic-git-policy-cli refused to read a candidate from a version that was invalidated by a fix or replay; \
                 list the version again and use its candidates.",
            ));
        }
        let Some(object) = candidate.object.as_ref() else {
            // `Rc::from(Vec::new())` is a shared empty byte list.
            return Ok(Rc::from(Vec::new()));
        };
        if candidate.mode == CandidateMode::Gitlink {
            // `.as_bytes()` views the name's text as bytes; `Rc::from(..)` copies them into shared storage.
            return Ok(Rc::from(object.as_str().as_bytes()));
        }
        return self.reader()?.blob(object);
    }

    /// What:
    ///  Retire every version listed so far.
    ///  `+= 1` advances the generation.
    /// Why:
    ///   A fix or replay rewrites index entries,
    ///  so earlier listings no longer
    ///       describe what would be committed.
    ///  Blob bytes stay remembered in the
    ///       reader:
    ///  an object name always denotes the same bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// invalidate(): void { this.#versions = []; this.#generation += 1n; }
    /// ```
    pub fn invalidate(&mut self) {
        self.versions.clear();
        self.generation += 1;
    }
}

/// Repository controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_store_tests.rs"]
mod tests;

/// Committed versions,
///  invalidation and listing failures.
#[cfg(test)]
#[path = "candidate_store_lifecycle_tests.rs"]
mod lifecycle_tests;

/// The process-count bound and its per-file positive control.
#[cfg(test)]
#[path = "candidate_process_count_tests.rs"]
mod process_count_tests;

/// Unborn,
///  SHA-256 and private-index listings.
#[cfg(test)]
#[path = "candidate_store_baseline_tests.rs"]
mod baseline_tests;

/// Listings that must be refused.
#[cfg(test)]
#[path = "candidate_store_failure_tests.rs"]
mod failure_tests;

/// The `HEAD` tree listing.
#[cfg(test)]
#[path = "candidate_store_tree_tests.rs"]
mod tree_tests;
