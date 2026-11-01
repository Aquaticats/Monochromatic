//! What:
//!  Parse the NUL-delimited stage records of `git ls-files --stage -z`,
//!  find the paths
//!       one staging operation changed,
//!  and turn them into candidate records.
//! Why:
//!  `git add` is checked before Git runs it,
//!  by replaying it on a private copy of the
//!      index.
//!  The paths it would stage are exactly those whose index records differ
//!      between the copy before and after the replay.
//!  Comparing complete records,
//!  conflict
//!      stages included,
//!  finds a resolved conflict like any other change,
//!  and an unrelated
//!      conflict elsewhere in the index never stops the comparison.
//!  Pathnames stay the raw
//!      bytes Git printed.
//!
//! Git 2.56.0 (`builtin/ls-files.c`,
//!  `show_ce`) prints,
//!  with `--stage -z`:
//! `<mode> SP <object> SP <stage> TAB <path> NUL`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const before = parseStageRecords(stdoutBefore); const after = parseStageRecords(stdoutAfter);
//! // const paths = stagedDelta(before, after);
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Import the candidate modes,
///  the validated object name and their parsers.
use super::candidate_object::{CandidateMode, ObjectId, mode_from_git, parse_object_id};
/// Import the candidate record a version is built from,
///  and its change kinds.
use super::candidate_record::{CandidateChange, CandidateRecord};
/// What:
///  `HashMap<K, V>` is a key-to-value table and `HashSet<T>` a set of keys
///       (siblings:
///  `BTreeMap` and `BTreeSet`,
///  which keep keys sorted).
/// Why:
///   Each path of one state is found without scanning the other state's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byPath = new Map<string, StageRecord[]>(); const added = new Set<string>();
/// ```
use std::collections::{HashMap, HashSet};

/// What:
///  One index entry.
///  `u8` is an unsigned byte;
///  a stage is 0 for a merged entry and
///       1 to 3 for the sides of a conflict.
///  `#[derive(...)]` generates cloning,
///  debug
///       printing and `==`.
/// Why:
///   Two states are compared record by record,
///  so a record holds every field Git
///       printed for the entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageRecord = { mode: CandidateMode; object: ObjectId; stage: 0 | 1 | 2 | 3; path: Buffer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageRecord {
    /// The entry's file mode.
    pub mode: CandidateMode,
    /// The object the entry names.
    pub object: ObjectId,
    /// 0 for a merged entry;
    ///  1,
    ///  2 or 3 for a conflict stage.
    pub stage: u8,
    /// Repository-relative pathname,
    ///  exactly as Git printed it.
    pub path: Vec<u8>,
}

/// What:
///  One path a staging operation changed,
///  with what the index holds there afterwards.
///       `Vec<StageRecord>` is empty when the operation removed the path from the index.
/// Why:
///   A candidate is built from the state after the operation;
///  a removed path becomes
///       a deletion only when the baseline commit has it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChangedPath = { path: Buffer; after: StageRecord[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedPath {
    /// Repository-relative pathname.
    pub path: Vec<u8>,
    /// The path's records after the operation,
    ///  in stage order;
    ///  empty when it was removed.
    pub after: Vec<StageRecord>,
}

/// What:
///  Build the failure for one malformed stage record.
///  `position` is the record's
///       zero-based place in the listing;
///  `usize` is the type of list positions.
/// Why:
///   A record is named by its position,
///  never by its pathname,
///  which can itself be a
///       forbidden string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stageFailure(failure: CandidateFailure, position: number, detail: string): CandidateError;
/// ```
fn stage_failure(failure: CandidateFailure, position: usize, detail: &str) -> CandidateError {
    return CandidateError::new(
        failure,
        format!(
            "cli-git could not read the index: entry record {position} {detail}. \
             Inspect the entries with `git ls-files --stage`."
        )
        .as_str(),
    );
}

/// What:
///  Whether one byte is NUL,
///  the record terminator.
/// Why:
///   A named predicate for `split`,
///  because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNul = (byte: number) => byte === 0;
/// ```
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one byte is a space,
///  the metadata field separator.
/// Why:
///   A named predicate for `split`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSpace = (byte: number) => byte === 0x20;
/// ```
fn is_space(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What:
///  Whether one byte is a tab,
///  which ends the metadata.
/// Why:
///   A named predicate for `position`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTab = (byte: number) => byte === 0x09;
/// ```
fn is_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  The stage number of a stage field,
///  or nothing for any other text.
///       `Option<u8>` is "a number or nothing".
/// Why:
///   Git prints exactly one digit from 0 to 3;
///  anything else means the output is not
///       a stage listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stageNumber(text: Buffer): number | undefined;
/// ```
fn stage_number(text: &[u8]) -> Option<u8> {
    // `match` compares the whole field with each accepted spelling.
    match text {
        b"0" => return Some(0),
        b"1" => return Some(1),
        b"2" => return Some(2),
        b"3" => return Some(3),
        _ => return None,
    }
}

/// What:
///  Parse one record from its zero-based position and its bytes,
///  terminator removed.
/// Why:
///   The first tab ends the metadata;
///  the pathname is every byte after it,
///  tabs
///       included,
///  and must not be empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStageRecord(position: number, record: Buffer): StageRecord;
/// ```
fn parse_stage_record(position: usize, record: &[u8]) -> Result<StageRecord, CandidateError> {
    // What: `.iter().position(is_tab)` is the index of the first tab, if any.
    // Why:  Without a tab there is no pathname.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const tab = record.indexOf(0x09); if (tab === -1) throw malformed();
    // ```
    let Some(tab) = record.iter().position(is_tab) else {
        return Err(stage_failure(
            CandidateFailure::ListingMalformed,
            position,
            "has no tab before its pathname",
        ));
    };
    // `&record[..tab]` borrows the bytes before the tab; `&record[tab + 1..]` those after it.
    let path: &[u8] = &record[tab + 1..];
    let fields: Vec<&[u8]> = record[..tab].split(is_space).collect();
    if fields.len() != 3 || path.is_empty() {
        return Err(stage_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not have three metadata fields and a pathname",
        ));
    }
    let Some(mode) = mode_from_git(fields[0]) else {
        return Err(stage_failure(
            CandidateFailure::UnsupportedMode,
            position,
            "has a file mode other than 100644, 100755, 120000 or 160000",
        ));
    };
    let Some(object) = parse_object_id(fields[1]) else {
        return Err(stage_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not carry a complete object name",
        ));
    };
    let Some(stage) = stage_number(fields[2]) else {
        return Err(stage_failure(
            CandidateFailure::ListingMalformed,
            position,
            "does not carry a stage from 0 to 3",
        ));
    };
    // `.to_vec()` copies the borrowed pathname into storage the record owns.
    return Ok(StageRecord {
        mode,
        object,
        stage,
        path: path.to_vec(),
    });
}

/// What:
///  Parse a complete `git ls-files --stage -z` listing.
///       `Result<Vec<StageRecord>, CandidateError>` is "every record,
///  or the first failure".
/// Why:
///   A listing that does not end with NUL was cut short;
///  a partial list would hide
///       entries from the comparison,
///  so the whole listing is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStageRecords(output: Buffer): StageRecord[];
/// ```
pub fn parse_stage_records(output: &[u8]) -> Result<Vec<StageRecord>, CandidateError> {
    // `Vec::new()` is an empty growable list; `mut` lets the loop append to it.
    let mut records: Vec<StageRecord> = Vec::new();
    if output.is_empty() {
        return Ok(records);
    }
    // `.strip_suffix(&[0_u8])` is `Some(rest)` when the output ends with a NUL byte.
    let Some(body) = output.strip_suffix(&[0_u8]) else {
        return Err(stage_failure(
            CandidateFailure::ListingMalformed,
            0,
            "is cut short: the listing does not end with a NUL byte",
        ));
    };
    // `.split(is_nul).enumerate()` yields each record with its zero-based position.
    for (position, record) in body.split(is_nul).enumerate() {
        // A trailing `?` returns the failure to our caller, or unwraps the record.
        records.push(parse_stage_record(position, record)?);
    }
    return Ok(records);
}

/// What:
///  The records of one index state grouped by path,
///  in listing order.
///       `Vec<(&[u8], &[StageRecord])>` is a list of borrowed pathnames,
///  each with the
///       consecutive records Git listed for it.
/// Why:
///   Git lists every stage of a path together,
///  sorted,
///  so one pass over the slice
///       groups them without copying a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupByPath(records: StageRecord[]): [Buffer, StageRecord[]][];
/// ```
fn group_by_path(records: &[StageRecord]) -> Vec<(&[u8], &[StageRecord])> {
    let mut groups: Vec<(&[u8], &[StageRecord])> = Vec::new();
    // `start` is where the group being collected begins.
    let mut start: usize = 0;
    // `.windows(2)` visits each pair of neighbours; `.enumerate()` adds the left one's index.
    for (index, pair) in records.windows(2).enumerate() {
        if pair[0].path != pair[1].path {
            groups.push((records[start].path.as_slice(), &records[start..=index]));
            start = index + 1;
        }
    }
    // `.get(start)` is `Some(first)` when a last group remains after the final boundary.
    if let Some(first) = records.get(start) {
        groups.push((first.path.as_slice(), &records[start..]));
    }
    return groups;
}

/// What:
///  The paths whose records differ between two index states,
///  in the order the
///       installed wrapper reports them:
///  paths the first state held,
///  in its order,
///  then
///       paths only the second state holds,
///  in its order.
/// Why:
///   A path is changed when its complete record list differs,
///  stage by stage,
///  so a
///       resolved conflict counts and an untouched conflict does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stagedDelta(before: StageRecord[], after: StageRecord[]): ChangedPath[];
/// ```
pub fn staged_delta(before: &[StageRecord], after: &[StageRecord]) -> Vec<ChangedPath> {
    let before_groups: Vec<(&[u8], &[StageRecord])> = group_by_path(before);
    let after_groups: Vec<(&[u8], &[StageRecord])> = group_by_path(after);
    let mut before_by_path: HashMap<&[u8], &[StageRecord]> = HashMap::new();
    for (path, records) in &before_groups {
        // `*path` and `*records` copy the borrowed slices out of the pair.
        before_by_path.insert(*path, *records);
    }
    let mut after_by_path: HashMap<&[u8], &[StageRecord]> = HashMap::new();
    for (path, records) in &after_groups {
        after_by_path.insert(*path, *records);
    }
    let mut changed: Vec<ChangedPath> = Vec::new();
    for (path, records) in &before_groups {
        // `.get(path)` is the other state's records for this path, if it has the path;
        // `.copied()` turns the borrowed entry into the slice it holds.
        let later: Option<&[StageRecord]> = after_by_path.get(path).copied();
        if later != Some(*records) {
            changed.push(ChangedPath {
                path: path.to_vec(),
                // `.unwrap_or_default()` is the empty slice for a removed path.
                after: later.unwrap_or_default().to_vec(),
            });
        }
    }
    for (path, records) in &after_groups {
        if !before_by_path.contains_key(path) {
            changed.push(ChangedPath {
                path: path.to_vec(),
                after: records.to_vec(),
            });
        }
    }
    return changed;
}

/// What:
///  Turn changed paths into candidate records,
///  given the records `git diff-index`
///       printed for those paths against the baseline commit.
/// Why:
///   A path the index still holds is a candidate with its merged entry:
///  an addition
///       when the baseline lacks it,
///  otherwise a modification,
///  even when its content now
///       equals the baseline's.
///  A removed path is a deletion only when the baseline has
///       it;
///  a path that was never committed carries nothing to check and is dropped.
///       A path left unmerged has no single staged content and stops the prediction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deltaCandidates(changed: ChangedPath[], baseline: CandidateRecord[]): CandidateRecord[];
/// ```
pub fn delta_candidates(
    changed: &[ChangedPath],
    baseline: &[CandidateRecord],
) -> Result<Vec<CandidateRecord>, CandidateError> {
    let mut baseline_by_path: HashMap<&[u8], &CandidateRecord> = HashMap::new();
    for record in baseline {
        baseline_by_path.insert(record.path.as_slice(), record);
    }
    let mut candidates: Vec<CandidateRecord> = Vec::new();
    for (position, path) in changed.iter().enumerate() {
        let difference: Option<&CandidateRecord> =
            baseline_by_path.get(path.path.as_slice()).copied();
        // `match` on the records after the operation: none, exactly one merged entry, or anything else.
        match path.after.as_slice() {
            [] => {
                // `if let Some(record) = ...` keeps a deletion the baseline listed.
                if let Some(record) = difference
                    && record.change == CandidateChange::Deleted
                {
                    candidates.push(record.clone());
                }
            }
            [entry] if entry.stage == 0 => {
                let change: CandidateChange = match difference {
                    Some(record) if record.change == CandidateChange::Added => {
                        CandidateChange::Added
                    }
                    _ => CandidateChange::Modified,
                };
                candidates.push(CandidateRecord {
                    path: path.path.clone(),
                    mode: entry.mode,
                    change,
                    // `Some(...)` is the "present" variant: a staged entry always names an object.
                    object: Some(entry.object.clone()),
                });
            }
            _ => {
                return Err(stage_failure(
                    CandidateFailure::UnmergedPath,
                    position,
                    "is left unmerged (conflicted) by the staging operation; resolve the conflict and stage the result",
                ));
            }
        }
    }
    return Ok(candidates);
}

/// What:
///  Turn every entry of a scope into a candidate record,
///  given the records
///       `git diff-index` printed for the same scope against the baseline commit.
/// Why:
///   A direct check reads every selected file,
///  changed or not,
///  so every merged entry
///       is a candidate:
///  an addition when the baseline lacks it,
///  otherwise a modification.
///       A conflicted entry has no single content and stops the projection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scopeCandidates(entries: StageRecord[], baseline: CandidateRecord[]): CandidateRecord[];
/// ```
pub fn scope_candidates(
    entries: &[StageRecord],
    baseline: &[CandidateRecord],
) -> Result<Vec<CandidateRecord>, CandidateError> {
    let mut added: HashSet<&[u8]> = HashSet::new();
    for record in baseline {
        if record.change == CandidateChange::Added {
            added.insert(record.path.as_slice());
        }
    }
    let mut candidates: Vec<CandidateRecord> = Vec::new();
    for (position, entry) in entries.iter().enumerate() {
        if entry.stage != 0 {
            return Err(stage_failure(
                CandidateFailure::UnmergedPath,
                position,
                "is an unmerged (conflicted) index entry; resolve the conflict and stage the result",
            ));
        }
        let change: CandidateChange = if added.contains(&entry.path.as_slice()) {
            CandidateChange::Added
        } else {
            CandidateChange::Modified
        };
        candidates.push(CandidateRecord {
            path: entry.path.clone(),
            mode: entry.mode,
            change,
            object: Some(entry.object.clone()),
        });
    }
    return Ok(candidates);
}

/// Parser,
///  delta and classification controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_stage_tests.rs"]
mod tests;
