//! What:
//!  The listings of one commit's tree,
//! of the parents of a set of commits,
//! and of a tree's blobs,
//! as a second block
//!  of operations of the candidate store.
//! Why:
//!  `candidate_store.rs` holds the store and its working listings and stays under the file-size limit;
//! these
//!  operations only ask the store to run Git and parse what it prints,
//! so they live in a file of their own that
//!  imports the store and that the store does not import.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // store.commitTreeRecords(commit); store.parentsOutside(commits); store.treeRecords(tree);
//! ```

/// Import the layer's failure type and its closed list of causes.
use crate::candidate_error::{CandidateError, CandidateFailure};
/// The validated object name and its parser.
use crate::candidate_object::{ObjectId, parse_object_id};
/// The size of a pathspec chunk,
/// shared with the prediction and the attribute query.
use crate::candidate_pathspec_chunk_size::PATHSPEC_CHUNK_SIZE;
/// The record a diff listing prints,
/// and its parser.
use crate::candidate_record::{CandidateRecord, parse_raw_records};
/// The store these operations belong to,
/// and the failure of a listing command.
use crate::candidate_store::{CandidateStore, listing_failure};
/// The record a tree listing prints,
/// and its parser.
use crate::candidate_tree::{TreeRecord, parse_tree_records};
/// A hash-based set,
/// for the membership tests of the parent listing.
use std::collections::HashSet;
/// A shared,
///  immutable listing the store keeps for the commit it listed.
use std::sync::Arc;
/// `OsString` is owned operating-system text of raw OS bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// The most commits one `git rev-list` receives.
///  `usize` is the type of counts;
/// 2,048 object names stay far below any argument limit.
///  Exempt from a citation under `LQM` (class:
/// chunk size):
/// it splits one command into several starts of the same
///  command,
/// so the result is the same and only the number of starts changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const COMMIT_CHUNK_SIZE: usize = PATHSPEC_CHUNK_SIZE;

impl CandidateStore {
    /// What:
    /// Every file of the `HEAD` commit with its mode and object;
    /// nothing when the
    ///       branch has no commit yet.
    /// Why:
    ///  A policy that compares the candidate state with `HEAD` reads `HEAD`'s copy of
    ///       a file without naming the path in an object request,
    /// and learns its mode.
    ///       `--full-tree` lists from the top level whatever directory Git runs in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async headTreeRecords(): Promise<TreeRecord[]>
    /// ```
    pub fn head_tree_records(&mut self) -> Result<Vec<TreeRecord>, CandidateError> {
        let Some(head) = self.baseline_commit()? else {
            return Ok(Vec::new());
        };
        return self.commit_tree_records(&head);
    }

    /// What:
    /// Every file of one commit with its mode and object.
    /// Why:
    ///  Policies that compare candidates with the parents of a commit read each parent's files
    ///       the way they read `HEAD`'s.
    /// `--full-tree` lists from the top level whatever directory
    ///       Git runs in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async commitTreeRecords(commit: ObjectId): Promise<TreeRecord[]>
    /// ```
    pub fn commit_tree_records(
        &mut self,
        commit: &ObjectId,
    ) -> Result<Vec<TreeRecord>, CandidateError> {
        // What:  The store keeps the last listing it made, keyed by the commit it listed.
        // Why:   A commit's name fixes its tree, so the listing of one commit never changes, and a policy that asks for the
        //        same commit twice must not start Git over every tree of a repository with tens of thousands of files again.
        //        The answer lives on the store, so no lock and no process-wide state stands between two readers; a second
        //        store lists the same commit again, which costs one process.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kept = this.keptListing(commit); if (kept) return [...kept];
        // ```
        if let Some(kept) = self.kept_listing(commit) {
            return Ok(kept.as_ref().clone());
        }
        let listed: Arc<Vec<TreeRecord>> = Arc::new(self.list_commit_tree(commit)?);
        self.keep_listing(commit, &listed);
        return Ok(listed.as_ref().clone());
    }

    /// The listing of one commit's files by Git,
    /// as `commit_tree_records` asks for it when it has no answer kept.
    fn list_commit_tree(&mut self, commit: &ObjectId) -> Result<Vec<TreeRecord>, CandidateError> {
        let output: Vec<u8> = self.run_listing(
            "ls-tree",
            &[
                OsString::from("ls-tree"),
                OsString::from("-r"),
                OsString::from("-z"),
                OsString::from("--full-tree"),
                OsString::from(commit.as_str()),
            ],
        )?;
        return parse_tree_records(output.as_slice());
    }

    /// What:
    /// The parents of these commits that are not among the commits themselves,
    /// each once,
    /// in
    ///       the order of the commits and then of their parents.
    /// Why:
    ///  A pushed commit's parent that is also pushed is new to the remote,
    /// so only the parents
    ///       outside the given commits count as already published.
    /// One `rev-list` answers 2,048
    ///       commits,
    /// so the process count does not grow with the commits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async parentsOutside(commits: ObjectId[]): Promise<ObjectId[]>
    /// ```
    pub fn parents_outside(
        &mut self,
        commits: &[ObjectId],
    ) -> Result<Vec<ObjectId>, CandidateError> {
        // Membership tests stay constant-time however many commits a push carries.
        let given: HashSet<&ObjectId> = commits.iter().collect();
        let mut seen: HashSet<ObjectId> = HashSet::new();
        let mut outside: Vec<ObjectId> = Vec::new();
        for chunk in commits.chunks(COMMIT_CHUNK_SIZE) {
            let mut command: Vec<OsString> = vec![
                OsString::from("rev-list"),
                OsString::from("--no-walk=unsorted"),
                OsString::from("--parents"),
            ];
            for commit in chunk {
                command.push(OsString::from(commit.as_str()));
            }
            let output: Vec<u8> = self.run_listing("rev-list", command.as_slice())?;
            for line in output.split(is_line_feed_byte) {
                // The first name of a line is the commit itself; the rest are its parents.
                for name in line.split(is_space_byte).skip(1) {
                    let Some(parent) = parse_object_id(name) else {
                        return Err(listing_failure(
                            CandidateFailure::ListingMalformed,
                            "rev-list",
                            "printed a parent that is not an object name.",
                        ));
                    };
                    if !given.contains(&parent) && seen.insert(parent.clone()) {
                        outside.push(parent);
                    }
                }
            }
        }
        return Ok(outside);
    }

    /// What:
    ///  The records of one commit's own delta,
    /// each against every parent (or the empty tree for a root commit).
    /// Why:
    ///   A pushed commit's changes are the content a manual-push scan reads;
    /// the listing is the one the
    ///  landed-delta source already uses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// committedRecords(commit: ObjectId): CandidateRecord[]
    /// ```
    pub fn committed_records(
        &mut self,
        commit: &ObjectId,
    ) -> Result<Vec<CandidateRecord>, CandidateError> {
        let output: Vec<u8> = self.list_committed(commit)?;
        return parse_raw_records(output.as_slice());
    }

    /// What:
    ///  Every blob entry of one tree,
    /// as records that add each path.
    /// Why:
    ///   A push with no known published base scans the pushed tip's whole tree.
    ///   Submodule entries are not listed:
    /// their content is a commit name,
    /// which no content policy reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// treeRecords(tree: ObjectId): CandidateRecord[] // ls-tree -r, every blob as added
    /// ```
    pub fn tree_records(
        &mut self,
        tree: &ObjectId,
    ) -> Result<Vec<CandidateRecord>, CandidateError> {
        let listing: Vec<u8> = self.run_listing(
            "ls-tree",
            &[
                OsString::from("ls-tree"),
                OsString::from("--full-tree"),
                OsString::from("-r"),
                OsString::from("-z"),
                OsString::from(tree.as_str()),
            ],
        )?;
        let raw: Vec<u8> = added_raw_from_ls_tree(listing.as_slice())?;
        return parse_raw_records(raw.as_slice());
    }
}

/// What:
///  Whether one byte is the line feed that ends a line of `git rev-list` output.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeedByte = (byte: number) => byte === '\n'.charCodeAt(0);
/// ```
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one byte is the space that separates the names on a line of `git rev-list --parents` output.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSpaceByte = (byte: number) => byte === ' '.charCodeAt(0);
/// ```
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What:
///  Whether one byte is the NUL byte that ends each record of `git ls-tree -z` output.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one byte is the tab that separates the metadata of a `git ls-tree` record from its path.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTabByte = (byte: number) => byte === '\t'.charCodeAt(0);
/// ```
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  Turn NUL-separated `ls-tree -r -z` records into raw diff records that add each blob.
/// Why:
///   The record parser is the one the diff listings use,
/// so tree entries follow the same path and mode rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addedRawFromLsTree(output: Uint8Array): Uint8Array;
/// ```
fn added_raw_from_ls_tree(output: &[u8]) -> Result<Vec<u8>, CandidateError> {
    let mut raw: Vec<u8> = Vec::new();
    for record in output.split(is_nul_byte) {
        if record.is_empty() {
            continue;
        }
        let Some(tab) = record.iter().position(is_tab_byte) else {
            return Err(CandidateError::new(
                crate::candidate_error::CandidateFailure::ListingMalformed,
                "monochromatic-git-policy-cli could not list the pushed tree: an entry lacks its path separator.",
            ));
        };
        // The mode, type and object name are ASCII; the path stays the exact bytes Git printed.
        let metadata = String::from_utf8_lossy(&record[..tab]);
        let path: &[u8] = &record[tab + 1..];
        let fields: Vec<&str> = metadata.split(' ').collect();
        if fields.len() != 3 {
            return Err(CandidateError::new(
                crate::candidate_error::CandidateFailure::ListingMalformed,
                "monochromatic-git-policy-cli could not list the pushed tree: an entry lacks its mode, type or object name.",
            ));
        }
        if fields[1] != "blob" {
            continue;
        }
        let zeros = "0".repeat(fields[2].len());
        raw.extend_from_slice(format!(":000000 {} {zeros} {} A", fields[0], fields[2]).as_bytes());
        raw.push(0);
        raw.extend_from_slice(path);
        raw.push(0);
    }
    return Ok(raw);
}
