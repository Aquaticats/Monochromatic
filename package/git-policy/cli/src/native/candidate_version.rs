//! What:
//!  An immutable candidate version:
//!  the paths one Git state changes,
//!  each with its identity,
//!  mode and object.
//! Why:
//!  Every policy of one pass must judge the same files.
//!  A version is listed once and
//!      then only read,
//!  so path metadata is shared instead of re-queried,
//!  and a later
//!      fix or replay produces a new version instead of altering this one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CandidateVersion = { readonly candidates: readonly Candidate[] };
//! ```

/// Import the validated object name and the candidate modes.
use super::candidate_object::{CandidateMode, ObjectId};
/// Import the parsed listing records and their change kinds.
use super::candidate_record::{CandidateChange, CandidateRecord};
/// What:
///  `HashMap<K, V>` is a key-to-value table (siblings:
///  `BTreeMap`,
///  which keeps keys
///       sorted,
///  and `Vec` of pairs).
/// Why:
///   Candidates are found by pathname without scanning the whole list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byPath = new Map<string, number>();
/// ```
use std::collections::HashMap;

/// What:
///  Which Git state a version lists,
///  and what it is compared with.
///       `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   A commit being prepared is judged by what is staged;
///  a landed commit by what
///       it changed.
///  Naming the comparison makes the version reproducible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateSource = { kind: 'staged-against-head' } | { kind: 'staged-against-commit'; commit: ObjectId } | { kind: 'committed'; commit: ObjectId };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateSource {
    /// The index compared with `HEAD`;
    ///  every index entry when no commit exists yet.
    StagedAgainstHead,
    /// The index compared with one named commit,
    ///  such as a transaction's recorded base.
    StagedAgainstCommit(ObjectId),
    /// One commit compared with each of its parents;
    ///  every entry of a root commit.
    Committed(ObjectId),
}

/// What:
///  The identity of one candidate within one invocation.
///       `u64` is an unsigned 64-bit counter (siblings `u32`,
///  `usize`);
///  `usize` is the
///       unsigned integer every index uses.
/// Why:
///   Identity must not be derived from the pathname,
///  which the scanner may mask.
///       The generation tells a candidate of an invalidated version from a current
///       one;
///  `u64` cannot run out within a process.
///  The index is the candidate's
///       position in its version,
///  and `usize` is what indexing and the scanner take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateIdentity = { generation: bigint; index: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateIdentity {
    /// Counts invalidations:
    ///  candidates of an earlier generation are stale.
    pub generation: u64,
    /// Zero-based position in the version's candidate list.
    pub index: usize,
}

/// What:
///  One changed path of a version.
///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
///       `Option<ObjectId>` is "an object name or nothing".
/// Why:
///   Git pathnames are arbitrary bytes on Unix and are kept exactly.
///  A deleted
///       path has no object,
///  which the type states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Candidate = { identity: CandidateIdentity; path: Buffer; mode: CandidateMode; change: CandidateChange; object?: ObjectId };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// Invocation-local identity,
    ///  independent of the pathname.
    pub identity: CandidateIdentity,
    /// Repository-relative pathname with `/` separators,
    ///  exactly as Git stores it.
    pub path: Vec<u8>,
    /// Mode in the candidate state,
    ///  or the baseline's mode for a deleted path.
    pub mode: CandidateMode,
    /// Change against the baseline.
    pub change: CandidateChange,
    /// Object in the candidate state;
    ///  absent for a deleted path.
    pub object: Option<ObjectId>,
}

/// What:
///  The candidates of one listed state.
///  Fields are private:
///  a version is built
///       once by `build_version` and never changed.
/// Why:
///   Immutability is what lets several policies share one listing safely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateVersion { readonly #candidates: readonly Candidate[]; readonly #byPath: ReadonlyMap<string, number> }
/// ```
#[derive(Debug)]
pub struct CandidateVersion {
    /// Candidates in listing order.
    candidates: Vec<Candidate>,
    /// Position of each pathname's candidate.
    by_path: HashMap<Vec<u8>, usize>,
}

/// What:
///  Build a version from listing records,
///  keeping the first record of each pathname.
///       `Vec<CandidateRecord>` is taken by value:
///  the records' bytes move into the
///       version without being copied.
/// Why:
///   A merge commit lists one comparison per parent,
///  so a pathname can appear more
///       than once;
///  the first appearance decides,
///  as it did in the TypeScript wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildVersion(generation: bigint, records: CandidateRecord[]): CandidateVersion;
/// ```
pub fn build_version(generation: u64, records: Vec<CandidateRecord>) -> CandidateVersion {
    // `Vec::new()` and `HashMap::new()` are empty containers; `mut` lets the loop fill them.
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut by_path: HashMap<Vec<u8>, usize> = HashMap::new();
    for record in records {
        // `.contains_key(&record.path)` lends the pathname for the lookup.
        if by_path.contains_key(&record.path) {
            continue;
        }
        let index: usize = candidates.len();
        // `.clone()` copies the pathname so the table owns its key.
        by_path.insert(record.path.clone(), index);
        candidates.push(Candidate {
            identity: CandidateIdentity { generation, index },
            path: record.path,
            mode: record.mode,
            change: record.change,
            object: record.object,
        });
    }
    return CandidateVersion {
        candidates,
        by_path,
    };
}

/// What:
///  `impl CandidateVersion { ... }` attaches read-only accessors,
///  like class getters.
/// Why:
///   Callers can read a version and cannot change it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateVersion { get candidates() {} candidateAtPath(path) {} }
/// ```
impl CandidateVersion {
    /// What:
    ///  Every candidate,
    ///  in listing order.
    ///  `&[Candidate]` borrows the list read-only.
    /// Why:
    ///   A candidate's `identity.index` is its position in this list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get candidates(): readonly Candidate[]
    /// ```
    pub fn candidates(&self) -> &[Candidate] {
        return self.candidates.as_slice();
    }

    /// What:
    ///  The candidate at an exact pathname,
    ///  if this version changes it.
    ///       `Option<&Candidate>` is "a borrowed candidate or nothing".
    /// Why:
    ///   A policy that cares about specific files (a manifest,
    ///  a rules file) asks by
    ///       name instead of scanning the list or asking Git again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// candidateAtPath(path: Buffer): Candidate | undefined
    /// ```
    pub fn candidate_at_path(&self, path: &[u8]) -> Option<&Candidate> {
        // What: `.get(path)` is `Option<&usize>`; a trailing `?` returns `None` when absent;
        //       `*` reads the position through the borrow.
        // Why:  The table stores positions, so the candidate itself is fetched from the list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const index = byPath.get(path); return index === undefined ? undefined : candidates[index];
        // ```
        let index: usize = *self.by_path.get(path)?;
        return self.candidates.get(index);
    }
}

/// Version controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_version_tests.rs"]
mod tests;
