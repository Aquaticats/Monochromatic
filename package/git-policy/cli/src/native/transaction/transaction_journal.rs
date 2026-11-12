//! What:
//!  The values a commit's preparation and the service's landing share about the repository state they work on:
//! the base,
//!  the symbolic head,
//! the private index mode,
//! the sequencer conclusion,
//! the ref storage format and a policy-added path.
//! Why:
//!  The records the earlier transaction wrote into its registry (preparing,
//! prepared,
//! ref-updated,
//! landing and index-lock
//!  records) are gone with the transaction;
//! only the types the service route still passes between its steps remain.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Base = { kind: 'unborn' } | { kind: 'commit'; oid: string };
//! ```


/// What:
///  The target at invocation,
///  or the expected old target of a landing.
/// Why:
///   An unborn branch has no commit;
///  compare-and-swap then expects the all-zero OID.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparationBase = { kind: 'unborn' } | { kind: 'commit'; oid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Base {
    /// The target names no commit yet.
    Unborn,
    /// The target names this commit.
    Commit(String),
}

/// What:
///  The symbolic `HEAD` target at invocation.
/// Why:
///   A branch switch between preparation and landing fails the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SymbolicHeadTarget = { kind: 'branch'; ref: string } | { kind: 'detached' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolicHead {
    /// `HEAD` is symbolic to this ref,
    /// by the bytes Git printed for its name.
    Branch(crate::ref_name::RefName),
    /// `HEAD` is detached.
    Detached,
}

/// What:
///  How the commit selects its content.
/// Why:
///   Explicit-path commits build the tree from the base plus selected paths;
///  index
///       commits use the captured index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionMode = 'explicit-path' | 'index';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionMode {
    /// `git commit -- <paths>` with commit-only semantics.
    ExplicitPath,
    /// A commit of the index.
    Index,
}

/// What:
///  The commit kind that decides whether a moved target fails instead of replaying.
/// Why:
///   An amend or a conclusion cannot replay onto a moved branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConclusionKind = 'none' | 'amend' | 'merge' | 'cherry-pick' | 'revert';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Conclusion {
    /// An ordinary commit.
    None,
    /// `--amend`.
    Amend,
    /// The conclusion of a merge.
    Merge,
    /// The conclusion of a cherry-pick.
    CherryPick,
    /// The conclusion of a revert.
    Revert,
}

/// What:
///  The ref storage backend of the real repository.
/// Why:
///   The shadow ref snapshot and the conclusion pseudorefs differ by backend.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefStorageFormat = 'files' | 'reftable';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefFormat {
    /// Loose refs and `packed-refs`.
    Files,
    /// The reftable backend.
    Reftable,
}


/// What:
///  One policy-added path,
///  or one selected path whose worktree copy receives settled
///       bytes:
///  its mode,
///  the blob before the commit and the blob the commit lands.
/// Why:
///   After landing,
///  the worktree copy is replaced only while it still holds the original.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddedPathRecord = { path: string; gitMode: '100644' | '100755'; originalOid: string; intendedOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddedPath {
    /// Repository-relative path,
    /// the exact bytes Git reported.
    pub path: Vec<u8>,
    /// `100644` or `100755`.
    pub git_mode: String,
    /// Blob at `HEAD`,
    ///  in the real index and in the worktree before the commit.
    pub original_oid: String,
    /// Blob the commit lands and the worktree receives.
    pub intended_oid: String,
}
