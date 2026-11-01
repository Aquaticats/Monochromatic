//! What: Parse the NUL-delimited records of `git ls-tree -r -z --full-tree <commit>`: every
//!       file of one commit with its mode and object.
//! Why: The dependent-version policy compares the candidate state with `HEAD`: it reads
//!      each workspace manifest's `HEAD` bytes, and a direct fix may change a tracked file
//!      it did not select only when `HEAD` holds that file as the same ordinary blob.
//!      One listing answers both for every path. Pathnames stay the raw bytes Git printed.
//!
//! Git 2.56.0 (`builtin/ls-tree.c`, `show_tree_default`) prints, with `-r -z`:
//! `<mode> SP <type> SP <object> TAB <path> NUL`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const records = parseTreeRecords(stdout); const head = new Map(records.map((r) => [r.path, r]));
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Import the candidate modes, the validated object name and their parsers.
use super::candidate_object::{CandidateMode, ObjectId, mode_from_git, parse_object_id};

/// What: One file of a commit's tree. `#[derive(...)]` generates cloning, debug printing
///       and `==`.
/// Why:  A record holds what a comparison with an index entry needs: mode and object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TreeRecord = { mode: CandidateMode; object: ObjectId; path: Buffer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeRecord {
    /// The file's mode.
    pub mode: CandidateMode,
    /// The blob, or for a submodule the commit, the entry names.
    pub object: ObjectId,
    /// Repository-relative pathname, exactly as Git printed it.
    pub path: Vec<u8>,
}

/// What: Build the failure for one malformed tree record. `position` is the record's
///       zero-based place in the listing.
/// Why:  A record is named by its position, never by its pathname, which can itself be a
///       forbidden string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeFailure(failure: CandidateFailure, position: number, detail: string): CandidateError;
/// ```
fn tree_failure(failure: CandidateFailure, position: usize, detail: &str) -> CandidateError {
    return CandidateError::new(
        failure,
        format!(
            "cli-git could not read the HEAD commit's files: tree record {position} {detail}. \
             Inspect the entries with `git ls-tree -r HEAD`."
        )
        .as_str(),
    );
}

/// What: Whether one byte is NUL, the record terminator.
/// Why:  A named predicate for `split`, because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNul = (byte: number) => byte === 0;
/// ```
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What: Whether one byte is a space, the metadata field separator.
/// Why:  A named predicate for `split`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSpace = (byte: number) => byte === 0x20;
/// ```
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What: Whether one byte is a tab, which ends the metadata.
/// Why:  A named predicate for `position`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTab = (byte: number) => byte === 0x09;
/// ```
fn is_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What: The object type Git prints for an entry of `mode`.
/// Why:  A recursive listing names files only: a submodule is a commit, every other file
///       a blob. Any other pairing means the output is not such a listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const expectedType = (mode: CandidateMode) => mode === 'gitlink' ? 'commit' : 'blob';
/// ```
fn expected_type(mode: CandidateMode) -> &'static [u8] {
    match mode {
        CandidateMode::Gitlink => return b"commit",
        CandidateMode::Regular | CandidateMode::Executable | CandidateMode::Symlink => {
            return b"blob";
        }
    }
}

/// What: Parse one record from its zero-based position and its bytes, terminator removed.
/// Why:  The first tab ends the metadata; the pathname is every byte after it, tabs
///       included, and must not be empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseTreeRecord(position: number, record: Buffer): TreeRecord;
/// ```
fn parse_tree_record(position: usize, record: &[u8]) -> Result<TreeRecord, CandidateError> {
    let Some(tab) = record.iter().position(is_tab) else {
        return Err(tree_failure(
            CandidateFailure::ListingMalformed,
            position,
            "has no tab before its pathname",
        ));
    };
    // `&record[..tab]` borrows the bytes before the tab; `&record[tab + 1..]` those after it.
    let path: &[u8] = &record[tab + 1..];
    let fields: Vec<&[u8]> = record[..tab].split(is_space).collect();
    if fields.len() != 3 || path.is_empty() {
        return Err(tree_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not have three metadata fields and a pathname",
        ));
    }
    let Some(mode) = mode_from_git(fields[0]) else {
        return Err(tree_failure(
            CandidateFailure::UnsupportedMode,
            position,
            "has a file mode other than 100644, 100755, 120000 or 160000",
        ));
    };
    if fields[1] != expected_type(mode) {
        return Err(tree_failure(
            CandidateFailure::ListingMalformed,
            position,
            "has an object type that does not match its mode",
        ));
    }
    let Some(object) = parse_object_id(fields[2]) else {
        return Err(tree_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not carry a complete object name",
        ));
    };
    // `.to_vec()` copies the borrowed pathname into storage the record owns.
    return Ok(TreeRecord {
        mode,
        object,
        path: path.to_vec(),
    });
}

/// What: Parse a complete `git ls-tree -r -z` listing.
///       `Result<Vec<TreeRecord>, CandidateError>` is "every record, or the first failure".
/// Why:  A listing that does not end with NUL was cut short; a partial list would read a
///       file `HEAD` holds as new, so the whole listing is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseTreeRecords(output: Buffer): TreeRecord[];
/// ```
pub fn parse_tree_records(output: &[u8]) -> Result<Vec<TreeRecord>, CandidateError> {
    let mut records: Vec<TreeRecord> = Vec::new();
    if output.is_empty() {
        return Ok(records);
    }
    // `.strip_suffix(&[0_u8])` is `Some(rest)` when the output ends with a NUL byte.
    let Some(body) = output.strip_suffix(&[0_u8]) else {
        return Err(tree_failure(
            CandidateFailure::ListingMalformed,
            0,
            "is cut short: the listing does not end with a NUL byte",
        ));
    };
    // `.split(is_nul).enumerate()` yields each record with its zero-based position.
    for (position, record) in body.split(is_nul).enumerate() {
        records.push(parse_tree_record(position, record)?);
    }
    return Ok(records);
}

/// Parser controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_tree_tests.rs"]
mod tests;
