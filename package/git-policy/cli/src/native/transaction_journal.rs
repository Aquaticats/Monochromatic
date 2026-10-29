//! What:
//!  The schema-version-2 journal records of one commit transaction and their filenames;
//!       encoding and parsing live in `transaction_journal_encode.rs` and
//!       `transaction_journal_parse.rs`.
//! Why:
//!  State files are created exclusively and never rewritten,
//!  so a crash leaves the newest
//!      complete state readable;
//!  recovery in either wrapper reads the records the other wrote
//!      (`src/policy-engine/commit-transaction-journal-states.ts` and
//!      `commit-transaction-journal-parse.ts`).
//!  Every missing or mistyped field fails closed
//!      with the record and field named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const preparing = parsePreparingRecord(bytes); await writeJournalRecord({ directory, filename, record });
//! ```

/// The digest of an index snapshot.
use super::content_digest::ContentDigest;

/// Journal schema version of every per-transaction state record.
pub const JOURNAL_SCHEMA_VERSION: i64 = 2;
/// Invocation capture record filename.
pub const PREPARING_FILENAME: &str = "preparing.json";
/// Prepared commit record filename.
pub const PREPARED_FILENAME: &str = "prepared.json";
/// Landed commit record filename.
pub const REF_UPDATED_FILENAME: &str = "ref-updated.json";
/// Index installation completion marker filename.
pub const INDEX_INSTALLED_FILENAME: &str = "index-installed";

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
    /// `HEAD` is symbolic to this ref.
    Branch(String),
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
///  A file identity recorded without hashes:
///  device and inode as decimal text.
/// Why:
///   Recovery proves an artifact is the exact file written,
///  not one put in its place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileIdentity = { device: string; inode: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    /// Device identity.
    pub device: String,
    /// Inode identity.
    pub inode: String,
}

/// What:
///  The identity of the real `index.lock` a transaction created.
/// Why:
///   Recovery removes a real `index.lock` only when it is provably the transaction's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockIdentity = FileIdentity & { fsId: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockIdentity {
    /// Device and inode.
    pub file: FileIdentity,
    /// Filesystem identity containing the lock.
    pub fs_id: String,
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
    /// Repository-relative path, the exact bytes Git reported.
    pub path: Vec<u8>,
    /// `100644` or `100755`.
    pub git_mode: String,
    /// Blob at `HEAD`,
    ///  in the real index and in the worktree before the commit.
    pub original_oid: String,
    /// Blob the commit lands and the worktree receives.
    pub intended_oid: String,
}

/// What:
///  `preparing.json`:
///  invocation capture facts written before any Git mutation.
/// Why:
///   Recovery learns from it where every piece of the transaction lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparingRecord = { schemaVersion: 2; state: 'preparing'; transactionId; mode; base; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparingRecord {
    /// Transaction ID,
    ///  also the reflog nonce.
    pub transaction_id: String,
    /// Commit selection mode.
    pub mode: TransactionMode,
    /// Target commit at invocation.
    pub base: Base,
    /// Symbolic `HEAD` target at invocation.
    pub symbolic_head: SymbolicHead,
    /// Ref advanced by compare-and-swap.
    pub target_ref: String,
    /// Commit kind.
    pub conclusion: Conclusion,
    /// Canonical worktree root.
    pub repository_root: String,
    /// Owning worktree Git directory.
    pub git_dir: String,
    /// Common Git directory.
    pub common_dir: String,
    /// Real index path.
    pub real_index_path: String,
    /// Real object directory.
    pub object_directory: String,
    /// Ref storage backend.
    pub ref_format: RefFormat,
    /// Empty tree standing in for an unborn base.
    pub empty_tree_oid: String,
    /// Shadow repository path derived from the transaction ID.
    pub shadow_path: String,
    /// Pathspecs selected at invocation, as the exact bytes of the command line.
    pub selected_pathspecs: Vec<Vec<u8>>,
    /// ISO-8601 invocation start time.
    pub invoked_at: String,
}

/// What:
///  `prepared.json`:
///  the verified prepared commit in the shadow repository.
/// Why:
///   Landing and recovery need the commit,
///  its tree and the worktree completions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedRecord = { schemaVersion: 2; state: 'prepared'; shadowPath; preparedOid; signed; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedRecord {
    /// Shadow repository holding the prepared commit.
    pub shadow_path: String,
    /// Prepared commit.
    pub prepared_oid: String,
    /// Whether the prepared commit carries a signature header.
    pub signed: bool,
    /// Intended tree the prepared commit was verified against.
    pub intended_tree_oid: String,
    /// Paths the commit carries,
    ///  including policy-added paths,
    ///  as the exact bytes Git reported.
    pub committed_paths: Vec<Vec<u8>>,
    /// Policy-added paths with original and intended blobs.
    pub added_paths: Vec<AddedPath>,
    /// Selected paths whose worktree copies receive settled bytes.
    pub selected_worktree_paths: Vec<AddedPath>,
}

/// What:
///  `index-lock-<n>.json`:
///  identity of the real `index.lock` attempt `n` created,
///       written right after creating it.
/// Why:
///   Recovery can prove ownership of the lock before any landing record exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLockRecord = { schemaVersion: 2; state: 'index-locked'; attempt: number; lock: LockIdentity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexLockRecord {
    /// Landing attempt number,
    ///  from 1.
    pub attempt: i64,
    /// Created lock identity.
    pub lock: LockIdentity,
}

/// What:
///  What a landing attempt installs:
///  a commit,
///  or a normalization that installs only index
///       and worktree bytes.
/// Why:
///   A normalization never moves the target,
///  so recovery checks it did not move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingOperation = 'commit' | 'normalize-only';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LandingOperation {
    /// The target advances to a new commit.
    Commit,
    /// Only the index and worktree change.
    NormalizeOnly,
}

/// What:
///  `landing-<n>.json`:
///  one landing attempt inside the critical section.
/// Why:
///   Recovery decides from it whether the attempt landed and completes or discards it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingRecord = { schemaVersion: 2; state: 'landing'; attempt; operation; expectedOld; newOid?; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandingRecord {
    /// Landing attempt number,
    ///  from 1.
    pub attempt: i64,
    /// Landing kind.
    pub operation: LandingOperation,
    /// Expected old target value for compare-and-swap.
    pub expected_old: Base,
    /// New target value;
    ///  absent for a normalization.
    pub new_oid: Option<String>,
    /// Tree the landing installs into the real index.
    pub landed_tree_oid: String,
    /// Exact pre-landing real index snapshot identity.
    pub pre_landing_index: FileIdentity,
    /// Exact post-index artifact identity.
    pub post_index: FileIdentity,
    /// Real `index.lock` identity.
    pub lock: LockIdentity,
    /// Migrated pack name;
    ///  absent for a normalization.
    pub pack_name: Option<String>,
    /// Policy-added path records.
    pub added_paths: Vec<AddedPath>,
    /// Selected worktree completion records.
    pub selected_worktree_paths: Vec<AddedPath>,
    /// Length and hash of the pre-landing snapshot as written;
    ///  absent in a record written before digests existed.
    pub pre_landing_digest: Option<ContentDigest>,
    /// Length and hash of the post-index as written;
    ///  absent in a record written before digests existed.
    pub post_digest: Option<ContentDigest>,
}

/// What:
///  The numbered artifact and record filenames of one landing attempt.
/// Why:
///   Every writer and reader names them the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// preLandingIndexFilename(1) // 'pre-landing-1.index'
/// ```
pub fn pre_landing_index_filename(attempt: i64) -> String {
    return format!("pre-landing-{attempt}.index");
}

/// What:
///  The post-index artifact filename of one attempt.
/// Why:
///   See `pre_landing_index_filename`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// postIndexFilename(1) // 'post-1.index'
/// ```
pub fn post_index_filename(attempt: i64) -> String {
    return format!("post-{attempt}.index");
}

/// What:
///  The landing record filename of one attempt.
/// Why:
///   See `pre_landing_index_filename`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landingRecordFilename(1) // 'landing-1.json'
/// ```
pub fn landing_record_filename(attempt: i64) -> String {
    return format!("landing-{attempt}.json");
}

/// What:
///  The index-lock record filename of one attempt.
/// Why:
///   See `pre_landing_index_filename`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// indexLockRecordFilename(1) // 'index-lock-1.json'
/// ```
pub fn index_lock_record_filename(attempt: i64) -> String {
    return format!("index-lock-{attempt}.json");
}
