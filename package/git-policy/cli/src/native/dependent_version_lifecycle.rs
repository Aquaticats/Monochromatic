//! What: The dependent-version planner's view of a prepared candidate state: every tracked
//!       file of the private index, its bytes with this invocation's corrections laid over
//!       them, and the `HEAD` commit's copy of a file.
//! Why: The planner reads workspace manifests, the registry configuration and source files
//!      wherever they are, not only the candidates; the installed wrapper's policy reads
//!      them as tracked files of the candidate state (`trackedFiles`, `bytes`, `headBytes`).
//!      Everything goes through the store the candidates were prepared in, so a direct fix's
//!      later pass reads what its earlier passes corrected.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const workspace = lifecycleWorkspace(store, corrected, candidatePaths); await planWorkspaceBumps(workspace);
//! ```

/// The layer's failure type, whose message is reported as the reason.
use super::candidate_error::CandidateError;
/// The modes and object names of index and tree entries.
use super::candidate_object::{CandidateMode, ObjectId};
/// The records a derived version is built from, and their change kinds.
use super::candidate_record::{CandidateChange, CandidateRecord};
/// The store the candidates were prepared in.
use super::candidate_store::CandidateStore;
/// The versions whose entries the store reads bytes for.
use super::candidate_version::{Candidate, CandidateVersion};
/// The planner's content seam and its failure.
use super::dependent_version_content::{
    ContentUnavailable, TrackedMode, TrackedPath, WorkspaceContent,
};
/// The corrected files of this invocation.
use super::direct_fix_install::InstallChange;
/// What: `BTreeMap<K, V>` is a map kept sorted by key.
/// Why:  The corrected files are kept that way by the content state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map<string, InstallChange>
/// ```
use std::collections::BTreeMap;
/// `OsString` is owned operating-system text: the listing's pathspec.
use std::ffi::OsString;
/// `Rc<T>` is a shared, read-only handle.
use std::rc::Rc;

/// The reason an unmerged entry stops the plan.
const UNMERGED_REASON: &str =
    "is an unmerged (conflicted) index entry; resolve the conflict and stage the result";

/// The reason a candidate the listing lacks stops the plan.
const UNLISTED_REASON: &str = "is a candidate that the listing of tracked files does not hold, so the listing cannot be trusted";

/// The reason a file that is not tracked cannot be read.
const UNTRACKED_REASON: &str = "is not a tracked file of the candidate state";

/// What: The candidate state as the planner reads it. `'a` names how long the borrows of
///       the store, the corrections and the candidate paths last.
/// Why:  Both listings are read once, when first needed, and kept as versions of the
///       store's current generation, whose bytes the store reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LifecycleWorkspace = { store: CandidateStore; corrected: Map<string, InstallChange>; candidates: string[] };
/// ```
pub struct LifecycleWorkspace<'a> {
    /// The store bound to the private index.
    store: &'a mut CandidateStore,
    /// The corrected files of this invocation, by pathname.
    corrected: &'a BTreeMap<Vec<u8>, InstallChange>,
    /// The paths of the lifecycle's candidates, which the tracked listing must hold.
    candidates: &'a [Vec<u8>],
    /// The private index's entries, once listed.
    index: Option<Rc<CandidateVersion>>,
    /// The `HEAD` commit's files, once listed.
    head: Option<Rc<CandidateVersion>>,
}

/// What: A view over a prepared store. `candidates` are the lifecycle's candidate paths
///       that are not deletions.
/// Why:  Nothing is listed until the planner asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lifecycleWorkspace(store, corrected, candidates): LifecycleWorkspace;
/// ```
pub fn lifecycle_workspace<'a>(
    store: &'a mut CandidateStore,
    corrected: &'a BTreeMap<Vec<u8>, InstallChange>,
    candidates: &'a [Vec<u8>],
) -> LifecycleWorkspace<'a> {
    return LifecycleWorkspace {
        store,
        corrected,
        candidates,
        index: None,
        head: None,
    };
}

/// What: The planner's failure for a layer failure, naming a path when one is concerned.
/// Why:  The layer's own message already says what to inspect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unavailable = (path, error) => ({ path, reason: error.message });
/// ```
fn unavailable(path: Option<&[u8]>, error: &CandidateError) -> ContentUnavailable {
    return ContentUnavailable {
        path: path.map(<[u8]>::to_vec),
        reason: error.message.clone(),
    };
}

/// What: The planner's mode for a candidate mode.
/// Why:  The two enumerations name the same four Git modes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trackedMode = (mode: CandidateMode) => mode === 'gitlink' ? 'submodule' : mode;
/// ```
pub fn tracked_mode(mode: CandidateMode) -> TrackedMode {
    match mode {
        CandidateMode::Regular => return TrackedMode::Regular,
        CandidateMode::Executable => return TrackedMode::Executable,
        CandidateMode::Symlink => return TrackedMode::Symlink,
        CandidateMode::Gitlink => return TrackedMode::Submodule,
    }
}

/// What: A record for a derived version: the entry's path, mode and object.
/// Why:  A derived version only carries what the store needs to read bytes; the change
///       kind is not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryRecord = (path, mode, object) => ({ path, mode, change: 'modified', object });
/// ```
fn entry_record(path: &[u8], mode: CandidateMode, object: &ObjectId) -> CandidateRecord {
    return CandidateRecord {
        path: path.to_vec(),
        mode,
        change: CandidateChange::Modified,
        object: Some(object.clone()),
    };
}

/// What: `impl LifecycleWorkspace<'_> { ... }` attaches the two listings and the lookups.
/// Why:  The planner, and a direct fix admitting a file it did not select, read both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LifecycleWorkspace { index() {} head() {} indexEntry(path) {} headEntry(path) {} }
/// ```
impl LifecycleWorkspace<'_> {
    /// What: The private index's stage-0 entries as a version, listed on first use.
    /// Why:  `:/` selects the whole index from any directory. An unmerged entry has no
    ///       single content, and a listing without one of the candidates is not the
    ///       candidate state, so either refuses the plan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async index(): Promise<CandidateVersion>;
    /// ```
    fn index(&mut self) -> Result<Rc<CandidateVersion>, ContentUnavailable> {
        if let Some(listed) = &self.index {
            return Ok(Rc::clone(listed));
        }
        let entries = match self.store.stage_records(&[OsString::from(":/")]) {
            Ok(found) => found,
            Err(error) => return Err(unavailable(None, &error)),
        };
        let mut records: Vec<CandidateRecord> = Vec::new();
        for entry in &entries {
            if entry.stage != 0 {
                return Err(ContentUnavailable {
                    path: Some(entry.path.clone()),
                    reason: String::from(UNMERGED_REASON),
                });
            }
            records.push(entry_record(&entry.path, entry.mode, &entry.object));
        }
        let version: Rc<CandidateVersion> = self.store.derived_version(records);
        for path in self.candidates {
            if version.candidate_at_path(path).is_none() {
                return Err(ContentUnavailable {
                    path: Some(path.clone()),
                    reason: String::from(UNLISTED_REASON),
                });
            }
        }
        self.index = Some(Rc::clone(&version));
        return Ok(version);
    }

    /// What: The `HEAD` commit's files as a version, listed on first use; empty before the
    ///       first commit.
    /// Why:  One listing answers every `HEAD` read of the invocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async head(): Promise<CandidateVersion>;
    /// ```
    fn head(&mut self) -> Result<Rc<CandidateVersion>, ContentUnavailable> {
        if let Some(listed) = &self.head {
            return Ok(Rc::clone(listed));
        }
        let entries = match self.store.head_tree_records() {
            Ok(found) => found,
            Err(error) => return Err(unavailable(None, &error)),
        };
        let mut records: Vec<CandidateRecord> = Vec::new();
        for entry in &entries {
            records.push(entry_record(&entry.path, entry.mode, &entry.object));
        }
        let version: Rc<CandidateVersion> = self.store.derived_version(records);
        self.head = Some(Rc::clone(&version));
        return Ok(version);
    }

    /// What: The mode and object of a path's private index entry, if it has one.
    /// Why:  A direct fix compares it with `HEAD` before changing a file it did not select.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async indexEntry(path): Promise<{ mode; object } | undefined>;
    /// ```
    pub fn index_entry(
        &mut self,
        path: &[u8],
    ) -> Result<Option<(CandidateMode, ObjectId)>, ContentUnavailable> {
        let version: Rc<CandidateVersion> = self.index()?;
        return Ok(entry_of(&version, path));
    }

    /// What: The mode and object of a path in `HEAD`, if `HEAD` has it.
    /// Why:  See `index_entry`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async headEntry(path): Promise<{ mode; object } | undefined>;
    /// ```
    pub fn head_entry(
        &mut self,
        path: &[u8],
    ) -> Result<Option<(CandidateMode, ObjectId)>, ContentUnavailable> {
        let version: Rc<CandidateVersion> = self.head()?;
        return Ok(entry_of(&version, path));
    }

    /// What: The bytes of a path's entry in `version` through the store.
    /// Why:  Index and `HEAD` reads share the one object reader.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async entryBytes(candidate): Promise<Uint8Array>;
    /// ```
    fn entry_bytes(&mut self, candidate: &Candidate) -> Result<Vec<u8>, ContentUnavailable> {
        match self.store.bytes(candidate) {
            Ok(read) => return Ok(read.to_vec()),
            Err(error) => return Err(unavailable(Some(candidate.path.as_slice()), &error)),
        }
    }
}

/// What: The mode and object of the entry at `path` in `version`, if any.
/// Why:  Both lookups answer the same pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryOf = (version, path) => version.at(path) && { mode, object };
/// ```
fn entry_of(version: &CandidateVersion, path: &[u8]) -> Option<(CandidateMode, ObjectId)> {
    let candidate: &Candidate = version.candidate_at_path(path)?;
    let object: &ObjectId = candidate.object.as_ref()?;
    return Some((candidate.mode, object.clone()));
}

/// What: `impl WorkspaceContent for LifecycleWorkspace<'_>` is the planner's content seam.
/// Why:  The planner stays free of Git; this is the only place it meets the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LifecycleWorkspace implements WorkspaceContent {}
/// ```
impl WorkspaceContent for LifecycleWorkspace<'_> {
    /// What: Every tracked path of the candidate state with its mode, in index order.
    /// Why:  The planner selects manifests, the configuration and sources itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trackedPaths(): Promise<TrackedPath[]>;
    /// ```
    fn tracked_paths(&mut self) -> Result<Vec<TrackedPath>, ContentUnavailable> {
        let version: Rc<CandidateVersion> = self.index()?;
        let mut paths: Vec<TrackedPath> = Vec::new();
        for candidate in version.candidates() {
            paths.push(TrackedPath {
                path: candidate.path.clone(),
                mode: tracked_mode(candidate.mode),
            });
        }
        return Ok(paths);
    }

    /// What: A tracked file's candidate-state bytes, corrected ones first.
    /// Why:  A later pass of a direct fix reads what the earlier passes made of the file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// candidateBytes(path): Promise<Uint8Array>;
    /// ```
    fn candidate_bytes(&mut self, path: &[u8]) -> Result<Vec<u8>, ContentUnavailable> {
        if let Some(found) = self.corrected.get(path) {
            return Ok(found.replacement.to_vec());
        }
        let version: Rc<CandidateVersion> = self.index()?;
        let Some(candidate) = version.candidate_at_path(path) else {
            return Err(ContentUnavailable {
                path: Some(path.to_vec()),
                reason: String::from(UNTRACKED_REASON),
            });
        };
        return self.entry_bytes(candidate);
    }

    /// What: A file's bytes in `HEAD`, or nothing when `HEAD` lacks it.
    /// Why:  A manifest's version is raised when it differs from its `HEAD` version.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// baseBytes(path): Promise<Uint8Array | undefined>;
    /// ```
    fn base_bytes(&mut self, path: &[u8]) -> Result<Option<Vec<u8>>, ContentUnavailable> {
        let version: Rc<CandidateVersion> = self.head()?;
        let Some(candidate) = version.candidate_at_path(path) else {
            return Ok(None);
        };
        return Ok(Some(self.entry_bytes(candidate)?));
    }
}

/// Listing, overlay and `HEAD` controls against real Git stay out of the release executable.
#[cfg(test)]
#[path = "dependent_version_lifecycle_tests.rs"]
mod tests;
