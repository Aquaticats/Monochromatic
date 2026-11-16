//! What:
//!  What the wrapper recorded beside a kept reference when it wrote the reference:
//!  when,
//!  which commit,
//!  which batch and form,
//!  and which worktree the commit was built in.
//! Why:
//!  A reference's own file says nothing about when it was written,
//!  and a kept reference is written with no reflog on purpose
//!  (`NO_REFLOG_SETTING`),
//!  so the age the pruning needs is nowhere in the reference itself.
//!  The worktree is worse:
//!  the reference outlives the worktree it was written for,
//!  so the moment the removal wrote the reference is the only moment the two are known together.
//!  One small file per kept reference answers both,
//!  and it goes with the reference when the pruning removes it,
//!  so the folder is exactly the kept references the repository holds.
//!  A reference written by a release that recorded nothing is dated from the first pass that listed it
//!  (`kept_reference_pass.rs`),
//!  so it is never pruned before it has been listed for the age limit,
//!  and its line says the worktree was not recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIfAbsent(recordPath(commonDir, 'refs/cli-git/kept/7'), { version: 1, reference, commit, batch: 7n, rescued: false, worktree, keptAtMs, why });
//! ```

use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// The folder inside the wrapper's data folder that holds one record per kept reference.
pub const RECORD_FOLDER: &str = "kept-references";

/// The record format version this release writes.
pub const VERSION: u64 = 1;

/// What the record of a reference written by an earlier release says it is.
pub const FIRST_SEEN: &str = "first seen by the pass that prunes kept references";

/// What the record of a reference a worktree removal wrote says it is.
pub const KEPT_FOR_A_REMOVAL: &str = "kept before a worktree that held the commit was removed";

/// What:
///  One kept reference,
/// as the wrapper wrote it down.
/// Why:
///   The pass needs the age and the listing needs the worktree,
///  and both are gone from the repository once the worktree is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptReferenceRecord = { version: 1; reference: string; commit: string; batch: bigint; rescued: boolean; worktree?: string; keptAtMs: number; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptReferenceRecord {
    /// The reference the record is about.
    pub reference: String,
    /// The commit that reference names when the record was written.
    pub commit: String,
    /// The batch the reference's name carries.
    pub batch: u64,
    /// Whether the reference is the name beside the batch's own.
    pub rescued: bool,
    /// The worktree the commit was built in,
    /// or nothing when the release that kept it recorded none.
    pub worktree: Option<String>,
    /// When the reference was written,
    /// in milliseconds since the Unix epoch.
    pub kept_at_ms: u128,
    /// Why it was written,
    /// in the wrapper's own words.
    pub why: String,
}

/// The file name of one kept reference's record:
/// the reference's own last name,
/// which the kept names already restrict to a number and the rescue suffix.
fn file_name_of(reference: &str) -> Option<String> {
    let last: &str = reference.rsplit('/').next()?;
    if last.is_empty() || last == "." || last == ".." || last.contains('\\') {
        return None;
    }
    return Some(format!("{last}.json"));
}

/// What:
///  The folder holding the record of every kept reference of a repository.
/// Why:
///   It sits in the wrapper's own data folder of the common Git directory,
///  beside the work records and the notices,
///  so every worktree of the repository writes the one folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordFolder = (commonDir) => join(commonDir, DATA_FOLDER.current, RECORD_FOLDER);
/// ```
pub fn record_folder(common_dir: &Path) -> PathBuf {
    return common_dir.join(DATA_FOLDER.current).join(RECORD_FOLDER);
}

/// The path of one kept reference's record.
fn record_path(common_dir: &Path, reference: &str) -> Option<PathBuf> {
    return Some(record_folder(common_dir).join(file_name_of(reference)?));
}

/// What:
///  The record of one kept reference,
///  or nothing when the repository has none.
/// Why:
///   A record that is missing,
///  unreadable,
///  or of another version is no record:
///  the pass dates the reference from the first pass that lists it,
///  which is later than the truth and so prunes nothing early.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readRecord(commonDir, 'refs/cli-git/kept/7') // undefined when absent or damaged
/// ```
pub fn read_record(common_dir: &Path, reference: &str) -> Option<KeptReferenceRecord> {
    let path: PathBuf = record_path(common_dir, reference)?;
    let bytes: Vec<u8> = std::fs::read(path).ok()?;
    let text: &str = std::str::from_utf8(bytes.as_slice()).ok()?;
    let value: Value = serde_json::from_str(text.trim()).ok()?;
    let object: &Map<String, Value> = value.as_object()?;
    if object.get("version")?.as_u64()? != VERSION {
        return None;
    }
    let batch: u64 = object.get("batch")?.as_u64()?;
    let kept_at_ms: u128 = u128::from(object.get("kept_at_ms")?.as_u64()?);
    return Some(KeptReferenceRecord {
        reference: String::from(object.get("reference")?.as_str()?),
        commit: String::from(object.get("commit")?.as_str()?),
        batch,
        rescued: object.get("rescued")?.as_bool()?,
        worktree: object.get("worktree").and_then(Value::as_str).map(String::from),
        kept_at_ms,
        why: String::from(object.get("why")?.as_str()?),
    });
}

/// What:
///  Write the record of a kept reference,
///  unless the repository already holds one for it,
///  and answer the record the repository holds afterwards.
/// Why:
///   The record is written once and never replaced:
///  a second writer's later time would make the reference look younger and keep it longer,
///  and a record replaced while a pass reads it would make the pass and the listing disagree.
///  Two writers racing both write the same kind of line,
///  and the one that loses reads the one that won,
///  so every caller answers the record the repository holds.
///  A repository whose data folder cannot be written keeps no record,
///  and the pass dates the reference from its own listing,
///  which loses no commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = writeIfAbsent(commonDir, candidate) ?? readRecord(commonDir, candidate.reference);
/// ```
pub fn write_if_absent(common_dir: &Path, record: &KeptReferenceRecord) -> Option<KeptReferenceRecord> {
    if let Some(path) = record_path(common_dir, record.reference.as_str()) {
        if let Some(directory) = path.parent() {
            let _created = create_private_directory_all(directory);
        }
        let _written = write_private_file_unsynced(path.as_path(), line_of(record).as_bytes());
        return read_record(common_dir, record.reference.as_str());
    }
    return None;
}

/// What:
///  Remove the record of a kept reference that is no longer there.
/// Why:
///   A record whose reference is gone names nothing,
///  and the folder's size is what the pass bounds,
///  so it goes with the reference.
///  A record that cannot be removed is left;
///  the next pass removes it with its reference gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await unlink(recordPath(commonDir, reference)); } catch {}
/// ```
pub fn remove_record(common_dir: &Path, reference: &str) {
    let Some(path) = record_path(common_dir, reference) else {
        return;
    };
    let _removed = std::fs::remove_file(path);
}

/// What:
///  A path as the record keeps it,
///  which is text with a byte that is not text replaced.
/// Why:
///  A record is JSON,
///  and the path of a worktree the wrapper removed is the one thing here a person needs;
///  a path with an odd byte is still worth more than no path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathText = (path) => path.toString('utf8');
/// ```
fn path_text(path: &std::path::Path) -> String {
    return path.to_string_lossy().into_owned();
}

/// What:
///  Write the record of a kept reference a worktree removal has just written,
///  naming the worktree it kept the commit for.
/// Why:
///   The removal is the one moment the reference and the worktree are known together,
///  and the reference outlives the worktree by design,
///  so this is the only account of where a kept commit was built.
///  Nothing here may fail the removal:
///  a record that cannot be written leaves the reference dated from the first pass that lists it,
///  which loses no commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeIfAbsent(commonDir, { version: 1, reference, commit, batch, rescued, worktree, keptAtMs: Date.now(), why: KEPT_FOR_A_REMOVAL });
/// ```
pub fn record_a_removals_kept_reference(common_dir: &Path, reference: &str, commit: &str, worktree: &std::path::Path, now_ms: u128) {
    let Some((batch, rescued)) = crate::kept_reference_name::batch_and_rescued_of(reference) else {
        return;
    };
    let record: KeptReferenceRecord = KeptReferenceRecord {
        reference: String::from(reference),
        commit: String::from(commit),
        batch,
        rescued,
        worktree: Some(path_text(worktree)),
        kept_at_ms: now_ms,
        why: String::from(KEPT_FOR_A_REMOVAL),
    };
    let _recorded: Option<KeptReferenceRecord> = write_if_absent(common_dir, &record);
}

/// What:
///  Remove the record of every kept reference the repository no longer holds.
/// Why:
///  A reference a person removed by hand,
///  or one whose removal a crash cut between the delete and this file,
///  leaves a record that names nothing,
///  and the folder's size is what the references bound.
///  A record written a moment ago is kept:
///  another command may have written the reference it names after this pass listed the repository,
///  and its record is the only account of the worktree that reference was kept for.
///  That grace is the floor the size rule uses,
/// which is an hour by default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const file of await readdir(folder)) { const record = await read(file); if (!held.has(record.reference) && now - record.keptAtMs > grace) await unlink(file); }
/// ```
pub fn sweep_records_of_references_that_are_gone(common_dir: &Path, held: &[crate::kept_references_reading::OneKeptReference], now: u128, grace: std::time::Duration) {
    let Ok(listing) = std::fs::read_dir(record_folder(common_dir)) else {
        return;
    };
    for entry in listing.flatten() {
        let path: std::path::PathBuf = entry.path();
        let Ok(bytes) = std::fs::read(path.as_path()) else {
            continue;
        };
        let Ok(value) = serde_json::from_slice::<Value>(bytes.as_slice()) else {
            continue;
        };
        let Some(reference) = value.get("reference").and_then(Value::as_str) else {
            continue;
        };
        if names_one_of(held, reference) {
            continue;
        }
        let kept_at_ms: u128 = value.get("kept_at_ms").and_then(Value::as_u64).map(u128::from).unwrap_or(0);
        if now.saturating_sub(kept_at_ms) <= grace.as_millis() {
            continue;
        }
        let _removed = std::fs::remove_file(path.as_path());
    }
}

/// What:
///  Whether any kept reference the repository holds is the one a record's line names.
/// Why:
///  A record whose reference the repository no longer holds names nothing,
///  and telling that apart from one another command wrote a moment ago is what the grace below is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesOneOf = (held, reference) => held.some((one) => one.reference === reference);
/// ```
fn names_one_of(held: &[crate::kept_references_reading::OneKeptReference], reference: &str) -> bool {
    for one in held {
        if one.reference == reference {
            return true;
        }
    }
    return false;
}

/// What:
///  The worktree a record names,
///  as JSON,
///  or nothing when it names none.
/// Why:
///  A null says no release recorded where the commit was built,
///  which is different from a worktree the record knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktreeValue = (worktree) => worktree ?? null;
/// ```
fn worktree_value(worktree: Option<&str>) -> Value {
    return match worktree {
        Some(path) => Value::from(path),
        None => Value::Null,
    };
}

/// What:
///  One record as the one JSON line it is written as.
/// Why:
///   The fields are read back by this file alone,
///  so the line's shape is stated here once,
///  and a person who opens one reads it as the wrapper wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineOf = (record) => JSON.stringify(record) + '\n';
/// ```
pub fn line_of(record: &KeptReferenceRecord) -> String {
    let mut object: Map<String, Value> = Map::new();
    object.insert(String::from("version"), Value::from(VERSION));
    object.insert(String::from("reference"), Value::from(record.reference.as_str()));
    object.insert(String::from("commit"), Value::from(record.commit.as_str()));
    object.insert(String::from("batch"), Value::from(record.batch));
    object.insert(String::from("rescued"), Value::from(record.rescued));
    object.insert(String::from("worktree"), worktree_value(record.worktree.as_deref()));
    object.insert(String::from("kept_at_ms"), Value::from(u64::try_from(record.kept_at_ms).unwrap_or(u64::MAX)));
    object.insert(String::from("why"), Value::from(record.why.as_str()));
    return format!("{}\n", Value::Object(object));
}

/// Controls for the records stay out of the release executable.
#[cfg(test)]
#[path = "kept_reference_record_tests.rs"]
mod tests;
