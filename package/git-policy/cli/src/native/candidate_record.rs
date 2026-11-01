//! What:
//!  Parse the NUL-delimited raw records of `git diff-index` and `git diff-tree`.
//! Why:
//!  One listing process names every changed path with its mode,
//!  object and change
//!      kind,
//!  so no per-path Git process is ever needed.
//!  Pathnames are kept as the raw
//!      bytes Git printed;
//!  they are never decoded.
//!
//! Git 2.56.0 (`Documentation/diff-format.adoc`,
//!  "RAW OUTPUT FORMAT") prints,
//!  with `-z`:
//! `:<old mode> SP <new mode> SP <old oid> SP <new oid> SP <status> NUL <path> NUL`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const records = parseRawRecords(stdout); // [{ path, mode, change, object }]
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Import the validated object name,
///  the candidate modes and their parsers.
use super::candidate_object::{CandidateMode, ObjectId, mode_from_git, parse_object_id};

/// What:
///  How a path differs from the baseline it was compared with.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   A deleted path has no content to read,
///  and policies may treat new files
///       differently from changed ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateChange = 'added' | 'modified' | 'deleted';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateChange {
    /// The baseline lacks the path.
    Added,
    /// Both sides hold the path with different content,
    ///  mode or type.
    Modified,
    /// The baseline holds the path and the candidate state does not.
    Deleted,
}

/// What:
///  One listed path.
///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
///       `Option<ObjectId>` is "an object name or nothing".
/// Why:
///   Git pathnames are arbitrary bytes on Unix and must round-trip exactly.
///  A
///       deleted path has no object on the candidate side,
///  which the type states
///       instead of carrying Git's all-zero placeholder name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateRecord = { path: Buffer; mode: CandidateMode; change: CandidateChange; object?: ObjectId };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateRecord {
    /// Repository-relative pathname with `/` separators,
    ///  exactly as Git printed it.
    pub path: Vec<u8>,
    /// Mode on the candidate side,
    ///  or the baseline's mode for a deleted path.
    pub mode: CandidateMode,
    /// Change against the baseline.
    pub change: CandidateChange,
    /// Object on the candidate side;
    ///  absent for a deleted path.
    pub object: Option<ObjectId>,
}

/// What:
///  Build a failure naming the record by position.
///       `usize` is the unsigned integer every index uses (siblings `u32`,
///  `u64`).
/// Why:
///   The position identifies the entry in the same listing a person can rerun,
///       without printing the pathname,
///  which may itself be a forbidden string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordFailure(failure: CandidateFailure, position: number, detail: string): CandidateError;
/// ```
fn record_failure(failure: CandidateFailure, position: usize, detail: &str) -> CandidateError {
    return CandidateError::new(
        failure,
        format!(
            "cli-git could not list candidate files: changed-path record {position} {detail}. \
             Inspect the entries with `git diff --cached --raw` or `git diff-tree --raw -r <commit>`."
        )
        .as_str(),
    );
}

/// Named predicate for splitting output into tokens;
///  `&u8` borrows one byte.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Named predicate for splitting record metadata into fields.
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What:
///  Turn one metadata token and its path token into a record.
///       `Result<T, E>` is "a value or a failure".
/// Why:
///   Only the statuses a listing without rename detection can print are
///       interpreted:
///  `A`,
///  `M`,
///  `T` (type change) and `D`.
///  `U` is a conflicted index
///       entry,
///  which has no single content to check.
///  Anything else is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRecord(position: number, metadata: Buffer, path: Buffer): CandidateRecord;
/// ```
fn parse_record(
    position: usize,
    metadata: &[u8],
    path: &[u8],
) -> Result<CandidateRecord, CandidateError> {
    // What: `.strip_prefix(b":")` is `Some(rest)` when the token starts with a colon;
    //       `let Some(..) = .. else { .. }` unwraps it or leaves with a failure.
    // Why:  Every raw record starts with a colon; its absence means the tokens are out of step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!metadata.startsWith(':')) throw malformed();
    // ```
    let Some(fields_text) = metadata.strip_prefix(b":") else {
        // `Err(...)` is the failure variant.
        return Err(record_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not start with a colon",
        ));
    };
    // `.split(is_space)` cuts at every space; `.collect()` gathers the borrowed pieces.
    let fields: Vec<&[u8]> = fields_text.split(is_space).collect();
    if fields.len() != 5 || path.is_empty() {
        return Err(record_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not have five metadata fields and a pathname",
        ));
    }
    let status: &[u8] = fields[4];
    if status == b"U" {
        return Err(record_failure(
            CandidateFailure::UnmergedPath,
            position,
            "is an unmerged (conflicted) index entry; resolve the conflict and stage the result",
        ));
    }
    let deleted: bool = status == b"D";
    if !deleted && status != b"A" && status != b"M" && status != b"T" {
        return Err(record_failure(
            CandidateFailure::UnsupportedStatus,
            position,
            "has a change status other than A, M, T or D",
        ));
    }
    // A deleted path is described by its baseline side: fields 0 and 2. Every other status uses the new side.
    let mode_text: &[u8] = if deleted { fields[0] } else { fields[1] };
    let Some(mode) = mode_from_git(mode_text) else {
        return Err(record_failure(
            CandidateFailure::UnsupportedMode,
            position,
            "has a file mode other than 100644, 100755, 120000 or 160000",
        ));
    };
    // Both names are validated even though a deleted path keeps neither: a bad one means the output is not raw records.
    let (Some(_old_object), Some(new_object)) =
        (parse_object_id(fields[2]), parse_object_id(fields[3]))
    else {
        return Err(record_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not carry two complete object names",
        ));
    };
    // `None` is the "absent" variant; `Some(...)` is the "present" variant.
    let object: Option<ObjectId> = if deleted { None } else { Some(new_object) };
    // `Ok(...)` is the success variant; `.to_vec()` copies the borrowed path bytes into an owned list.
    return Ok(CandidateRecord {
        path: path.to_vec(),
        mode,
        change: change_of(status),
        object,
    });
}

/// Map an already accepted status letter to its change kind:
///  `A` adds,
///  `D` deletes,
///  `M` and `T` modify.
fn change_of(status: &[u8]) -> CandidateChange {
    if status == b"A" {
        return CandidateChange::Added;
    }
    if status == b"D" {
        return CandidateChange::Deleted;
    }
    return CandidateChange::Modified;
}

/// What:
///  Parse complete raw `-z` output into records,
///  in Git's order.
///       `&[u8]` borrows the output;
///  the records own copies of what they keep.
/// Why:
///   Each record is two NUL-terminated tokens.
///  Output that does not end with a NUL,
///       or that holds an odd number of tokens,
///  was cut short or is not raw output,
///       and is refused whole:
///  a partial list would leave paths unchecked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRawRecords(output: Buffer): CandidateRecord[];
/// ```
pub fn parse_raw_records(output: &[u8]) -> Result<Vec<CandidateRecord>, CandidateError> {
    // `Vec::new()` is an empty growable list; `mut` lets the loop append to it.
    let mut records: Vec<CandidateRecord> = Vec::new();
    if output.is_empty() {
        return Ok(records);
    }
    // What: `.strip_suffix(&[0_u8])` is `Some(rest)` when the output ends with a NUL byte
    //       (`0_u8` is the number zero typed as one byte).
    // Why:  Removing the final terminator leaves tokens separated by single NULs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (output.at(-1) !== 0) throw malformed();
    // ```
    let Some(body) = output.strip_suffix(&[0_u8]) else {
        return Err(record_failure(
            CandidateFailure::ListingMalformed,
            0,
            "is cut short: the listing does not end with a NUL byte",
        ));
    };
    let tokens: Vec<&[u8]> = body.split(is_nul).collect();
    // What: `.as_chunks::<2>()` splits the tokens into pairs (arrays of exactly two) plus
    //       whatever is left over when the count is odd; `::<2>` is the pair size.
    // Why:  A leftover token is a record without its pathname.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (tokens.length % 2 !== 0) throw malformed();
    // ```
    let (pairs, leftover): (&[[&[u8]; 2]], &[&[u8]]) = tokens.as_chunks::<2>();
    if !leftover.is_empty() {
        return Err(record_failure(
            CandidateFailure::ListingMalformed,
            pairs.len(),
            "has no pathname token",
        ));
    }
    // `.iter().enumerate()` visits each pair with its zero-based position.
    for (position, pair) in pairs.iter().enumerate() {
        // A trailing `?` returns the failure to our caller, or unwraps the record.
        records.push(parse_record(position, pair[0], pair[1])?);
    }
    return Ok(records);
}

/// Record-format controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_record_tests.rs"]
mod tests;
