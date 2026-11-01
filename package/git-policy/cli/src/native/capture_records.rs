//! What:
//!  `captured.json` (a transaction's position in its worktree's capture order) and the
//!       landed-capture records,
//!  with recording after a landing and pruning.
//! Why:
//!  Replay decides a path both a prepared commit and an earlier-landed commit captured from
//!      this worktree by capture order;
//!  a landed record is kept while any published transaction
//!      may replay over it and pruned after (`src/policy-engine/commit-capture-order-journal.ts`,
//!      `commit-capture-order-records.ts`,
//!  `commit-capture-order-prune.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordLandedCaptureOrWarn({ gitDir, transactionDirectory, transactionId, landedOid });
//! await pruneLandedCaptures({ gitDir, registryRoot });
//! ```

/// The store's location,
///  files and failure.
use super::capture_store::{
    CaptureError, capture_store_path, ensure_capture_store, read_next_capture_sequence,
    read_store_file, read_worktree_id,
};
/// Diagnostics.
use super::diagnostic_log::{debug, warn};
/// Record readers and the ordered writer.
use super::json_record::{
    MAX_SAFE_INTEGER, ObjectWriter, parse_object, safe_integer, string_array,
};
/// Private exclusive files, written without a sync.
use super::private_storage::write_private_file_unsynced;
/// The journal schema version `captured.json` shares.
use super::transaction_journal::JOURNAL_SCHEMA_VERSION;
/// Registry listing and root creation.
use super::transaction_registry::{
    EntryKind, RegistryEntry, ensure_transaction_root, list_transaction_entries,
};
/// `Map`/`Value` are JSON objects and values.
use serde_json::{Map, Value};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Capture-order record filename inside a transaction directory.
pub const CAPTURED_FILENAME: &str = "captured.json";

/// Landed-record directory name inside the store.
pub const LANDED_DIRECTORY_NAME: &str = "landed";

/// Landed-record filename suffix.
pub const LANDED_RECORD_SUFFIX: &str = ".json";

/// Landed-record schema version.
pub const LANDED_SCHEMA_VERSION: i64 = 1;

/// What:
///  `captured.json`:
///  the capture stamp,
///  the next sequence read before the preparation base,
///       and the Latin-1 decoded paths the capture read from the worktree.
/// Why:
///   The record pins landed records against pruning and names the captured paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapturedRecord = { schemaVersion: 2; state: 'captured'; worktreeId: string; sequence: number; nextSequenceBeforeBase: number; worktreePaths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedRecord {
    /// Identity of the store that allocated the sequence number.
    pub worktree_id: String,
    /// Capture sequence number,
    ///  from 1.
    pub sequence: i64,
    /// Next sequence number read before the preparation base.
    pub next_sequence_before_base: i64,
    /// Paths whose bytes the capture read from the worktree,
    ///  Latin-1 decoded,
    ///  in Git order.
    pub worktree_paths: Vec<String>,
}

/// What:
///  One landed-capture record.
/// Why:
///   `LandedCaptureRecord`,
///  written right after a compare-and-swap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedCaptureRecord = { schemaVersion: 1; commit; transactionId; worktreeId; sequence; nextSequenceAfterLanding; worktreePaths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedCaptureRecord {
    /// Landed commit.
    pub commit: String,
    /// Transaction that landed it.
    pub transaction_id: String,
    /// Store identity.
    pub worktree_id: String,
    /// Capture sequence number.
    pub sequence: i64,
    /// Next capture sequence number read after the compare-and-swap.
    pub next_sequence_after_landing: i64,
    /// Paths the landing transaction captured from the worktree,
    ///  Latin-1 decoded.
    pub worktree_paths: Vec<String>,
}

/// What:
///  Parse a JSON object record,
///  or fail naming it.
/// Why:
///   `parseObject`:
///  arrays and scalars are not records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseObject({ text, name })
/// ```
fn record_object(text: &str, name: &str) -> Result<Map<String, Value>, CaptureError> {
    match parse_object(text) {
        Some(map) => return Ok(map),
        None => return Err(CaptureError(format!("{name} is not a JSON object."))),
    }
}

/// What:
///  The failure for a malformed field.
/// Why:
///   Every field reader reports the same shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new CaptureOrderRecordError(`${name} has a malformed ${key} field.`)
/// ```
fn malformed(name: &str, key: &str) -> CaptureError {
    return CaptureError(format!("{name} has a malformed {key} field."));
}

/// What:
///  A required non-empty string field.
/// Why:
///   `stringField`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stringField({ value, key, name })
/// ```
fn text_field(map: &Map<String, Value>, key: &str, name: &str) -> Result<String, CaptureError> {
    match map.get(key) {
        Some(Value::String(text)) if !text.is_empty() => return Ok(text.clone()),
        _ => return Err(malformed(name, key)),
    }
}

/// What:
///  A required positive safe integer field.
/// Why:
///   `positiveField`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveField({ value, key, name })
/// ```
fn positive_field(map: &Map<String, Value>, key: &str, name: &str) -> Result<i64, CaptureError> {
    match map.get(key).and_then(safe_integer) {
        Some(number) if number >= 1 => return Ok(number),
        _ => return Err(malformed(name, key)),
    }
}

/// What:
///  A required array of non-empty strings.
/// Why:
///   `pathsField`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathsField({ value, key, name })
/// ```
fn paths_field(
    map: &Map<String, Value>,
    key: &str,
    name: &str,
) -> Result<Vec<String>, CaptureError> {
    let Some(Value::Array(items)) = map.get(key) else {
        return Err(malformed(name, key));
    };
    // `mut` allows collecting the paths one at a time.
    let mut paths: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::String(path) if !path.is_empty() => paths.push(path.clone()),
            _ => return Err(malformed(name, key)),
        }
    }
    return Ok(paths);
}

/// What:
///  Parse `captured.json`.
/// Why:
///   `parseCapturedRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCapturedRecord(text: string): CapturedRecord;
/// ```
pub fn parse_captured_record(text: &str) -> Result<CapturedRecord, CaptureError> {
    let map: Map<String, Value> = record_object(text, CAPTURED_FILENAME)?;
    if map.get("schemaVersion").and_then(safe_integer) != Some(JOURNAL_SCHEMA_VERSION)
        || map.get("state").and_then(Value::as_str) != Some("captured")
    {
        return Err(CaptureError(format!(
            "{CAPTURED_FILENAME} is not a schema-version-2 captured record."
        )));
    }
    return Ok(CapturedRecord {
        worktree_id: text_field(&map, "worktreeId", CAPTURED_FILENAME)?,
        sequence: positive_field(&map, "sequence", CAPTURED_FILENAME)?,
        next_sequence_before_base: positive_field(
            &map,
            "nextSequenceBeforeBase",
            CAPTURED_FILENAME,
        )?,
        worktree_paths: paths_field(&map, "worktreePaths", CAPTURED_FILENAME)?,
    });
}

/// What:
///  Encode `captured.json` in the incumbent's field order,
///  with a newline.
/// Why:
///   The capture writes exactly these bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify({ schemaVersion: 2, state: 'captured', worktreeId, sequence, nextSequenceBeforeBase, worktreePaths })}\n`
/// ```
pub fn encode_captured_record(record: &CapturedRecord) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", JOURNAL_SCHEMA_VERSION)
        .string("state", "captured")
        .string("worktreeId", record.worktree_id.as_str())
        .integer("sequence", record.sequence)
        .integer("nextSequenceBeforeBase", record.next_sequence_before_base)
        .raw(
            "worktreePaths",
            string_array(record.worktree_paths.as_slice()).as_str(),
        );
    return format!("{}\n", writer.finish());
}

/// What:
///  A transaction's `captured.json`,
///  or nothing when it has not captured yet.
/// Why:
///   `readCapturedRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readCapturedRecord(directory: string): Promise<CapturedRecord | typeof CAPTURED_ABSENT>;
/// ```
pub fn read_captured_record(directory: &Path) -> Result<Option<CapturedRecord>, CaptureError> {
    match read_store_file(directory.join(CAPTURED_FILENAME).as_path())? {
        Some(text) => return Ok(Some(parse_captured_record(text.as_str())?)),
        None => return Ok(None),
    }
}

/// What:
///  The landed-record directory of a worktree.
/// Why:
///   `landedRecordDirectory`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landedRecordDirectory('/repo/.git') // '/repo/.git/cli-git-captures/landed'
/// ```
pub fn landed_record_directory(git_dir: &Path) -> PathBuf {
    return capture_store_path(git_dir).join(LANDED_DIRECTORY_NAME);
}

/// What:
///  Parse one landed record.
/// Why:
///   `parseLandedCaptureRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseLandedCaptureRecord({ text, name })
/// ```
pub fn parse_landed_capture_record(
    text: &str,
    name: &str,
) -> Result<LandedCaptureRecord, CaptureError> {
    let map: Map<String, Value> = record_object(text, name)?;
    if map.get("schemaVersion").and_then(safe_integer) != Some(LANDED_SCHEMA_VERSION) {
        return Err(CaptureError(format!(
            "{name} is not a schema-version-1 landed-capture record."
        )));
    }
    return Ok(LandedCaptureRecord {
        commit: text_field(&map, "commit", name)?,
        transaction_id: text_field(&map, "transactionId", name)?,
        worktree_id: text_field(&map, "worktreeId", name)?,
        sequence: positive_field(&map, "sequence", name)?,
        next_sequence_after_landing: positive_field(&map, "nextSequenceAfterLanding", name)?,
        worktree_paths: paths_field(&map, "worktreePaths", name)?,
    });
}

/// What:
///  Encode a landed record in the incumbent's field order,
///  with a newline.
/// Why:
///   `landedRecordOf` builds the object in this order before `JSON.stringify`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify(landedRecordOf(...))}\n`
/// ```
pub fn encode_landed_capture_record(record: &LandedCaptureRecord) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", LANDED_SCHEMA_VERSION)
        .string("commit", record.commit.as_str())
        .string("transactionId", record.transaction_id.as_str())
        .string("worktreeId", record.worktree_id.as_str())
        .integer("sequence", record.sequence)
        .integer(
            "nextSequenceAfterLanding",
            record.next_sequence_after_landing,
        )
        .raw(
            "worktreePaths",
            string_array(record.worktree_paths.as_slice()).as_str(),
        );
    return format!("{}\n", writer.finish());
}

/// What:
///  Record the capture of a commit that just landed,
///  or that recovery found landed;
///  returns
///       whether the transaction had a capture to record.
///  An existing record is kept.
/// Why:
///   `recordLandedCapture`:
///  only after the compare-and-swap succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recordLandedCapture({ gitDir, transactionDirectory, transactionId, landedOid }): Promise<boolean>;
/// ```
pub fn record_landed_capture(
    git_dir: &Path,
    transaction_directory: &Path,
    transaction_id: &str,
    landed_oid: &str,
) -> Result<bool, CaptureError> {
    let Some(captured) = read_captured_record(transaction_directory)? else {
        debug(
            "recordLandedCapture",
            format!(
                "transaction {transaction_id} captured without capture order; {landed_oid} gets no landed record"
            )
            .as_str(),
        );
        return Ok(false);
    };
    ensure_capture_store(git_dir)?;
    let directory: PathBuf = landed_record_directory(git_dir);
    if let Err(error) = ensure_transaction_root(directory.as_path()) {
        return Err(CaptureError(error.0));
    }
    let record: LandedCaptureRecord = LandedCaptureRecord {
        commit: String::from(landed_oid),
        transaction_id: String::from(transaction_id),
        worktree_id: captured.worktree_id,
        sequence: captured.sequence,
        next_sequence_after_landing: read_next_capture_sequence(git_dir)?,
        worktree_paths: captured.worktree_paths,
    };
    let path: PathBuf = directory.join(format!("{landed_oid}{LANDED_RECORD_SUFFIX}"));
    // Unsynced, file and directory: a torn or lost landed record completes in the tearing table at every phase.
    match write_private_file_unsynced(
        path.as_path(),
        encode_landed_capture_record(&record).as_bytes(),
    ) {
        Ok(()) => {
            debug(
                "recordLandedCapture",
                format!(
                    "recorded capture {} for landed {landed_oid}",
                    record.sequence
                )
                .as_str(),
            );
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            debug(
                "recordLandedCapture",
                format!("landed record of {landed_oid} already exists: {error}").as_str(),
            );
        }
        Err(error) => {
            return Err(CaptureError(format!(
                "writing {} failed: {error}",
                path.display()
            )));
        }
    }
    return Ok(true);
}

/// What:
///  Record a landed capture as `record_landed_capture` does,
///  but report a failure as a
///       warning instead of returning it.
/// Why:
///   `recordLandedCaptureOrWarn`:
///  the commit already landed;
///  a missing record only sends its
///       paths through subsumption in later replays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recordLandedCaptureOrWarn(options): Promise<boolean>;
/// ```
pub fn record_landed_capture_or_warn(
    git_dir: &Path,
    transaction_directory: &Path,
    transaction_id: &str,
    landed_oid: &str,
) -> bool {
    match record_landed_capture(git_dir, transaction_directory, transaction_id, landed_oid) {
        Ok(recorded) => return recorded,
        Err(error) => {
            warn(
                "recordLandedCaptureOrWarn",
                format!(
                    "could not record the capture of landed {landed_oid}; later replays over it use subsumption: {error}"
                )
                .as_str(),
            );
            return false;
        }
    }
}

/// What:
///  The landed-record filenames,
///  or none when the directory is absent.
/// Why:
///   `listLandedRecordNames`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function listLandedRecordNames(gitDir: string): Promise<readonly string[]>;
/// ```
pub fn list_landed_record_names(git_dir: &Path) -> Result<Vec<String>, CaptureError> {
    let directory: PathBuf = landed_record_directory(git_dir);
    let listing: std::fs::ReadDir = match std::fs::read_dir(&directory) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            debug(
                "listLandedRecordNames",
                format!("no landed-capture records in {}", git_dir.display()).as_str(),
            );
            return Ok(Vec::new());
        }
        Err(error) => {
            return Err(CaptureError(format!(
                "listing {} failed: {error}",
                directory.display()
            )));
        }
    };
    // `mut` allows collecting the names one at a time.
    let mut names: Vec<String> = Vec::new();
    for item in listing {
        match item {
            Ok(entry) => names.push(entry.file_name().to_string_lossy().into_owned()),
            Err(error) => {
                return Err(CaptureError(format!(
                    "listing {} failed: {error}",
                    directory.display()
                )));
            }
        }
    }
    return Ok(names);
}

/// What:
///  The smallest next sequence number a published transaction of this store generation
///       read before its base;
///  `None` keeps every record (a published transaction has not
///       captured yet);
///  `Some(i64::MAX)` stands for the incumbent's `Infinity`.
/// Why:
///   `pinningBound`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pinningBound({ registryRoot, worktreeId }): Promise<number | typeof KEEP_ALL>;
/// ```
fn pinning_bound(
    registry_root: &Path,
    worktree_id: Option<&str>,
) -> Result<Option<i64>, CaptureError> {
    let entries: Vec<RegistryEntry> = match list_transaction_entries(registry_root) {
        Ok(found) => found,
        Err(error) => return Err(CaptureError(error.0)),
    };
    // `mut` allows narrowing the bound transaction by transaction.
    let mut bound: i64 = i64::MAX;
    for entry in entries {
        if entry.kind != EntryKind::Transaction {
            continue;
        }
        let Some(captured) = read_captured_record(entry.path.as_path())? else {
            return Ok(None);
        };
        if Some(captured.worktree_id.as_str()) == worktree_id {
            bound = bound.min(captured.next_sequence_before_base);
        }
    }
    return Ok(Some(bound));
}

/// What:
///  Whether one landed-record file can go:
///  not a record,
///  unreadable,
///  of another store
///       generation,
///  or recorded a next number below the bound.
/// Why:
///   `prunable`:
///  a malformed record is reported and removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prunable({ directory, name, worktreeId, bound }): Promise<boolean>;
/// ```
fn prunable(directory: &Path, name: &str, worktree_id: Option<&str>, bound: i64) -> bool {
    if !name.ends_with(LANDED_RECORD_SUFFIX) {
        return true;
    }
    let path: PathBuf = directory.join(name);
    let name_text: String = path.display().to_string();
    let parsed: Result<LandedCaptureRecord, CaptureError> = match read_store_file(path.as_path()) {
        Ok(Some(text)) => parse_landed_capture_record(text.as_str(), name_text.as_str()),
        Ok(None) => Err(CaptureError(format!("{name_text} vanished"))),
        Err(error) => Err(error),
    };
    match parsed {
        Ok(record) => {
            return Some(record.worktree_id.as_str()) != worktree_id
                || record.next_sequence_after_landing < bound;
        }
        Err(error) => {
            warn(
                "pruneLandedCaptures",
                format!("removing unreadable landed-capture record {name_text}: {error}").as_str(),
            );
            return true;
        }
    }
}

/// What:
///  Remove every landed record no published transaction can replay over;
///  returns how many.
///       Never fails:
///  a failure is reported and the records stay for the next pruning.
/// Why:
///   `pruneLandedCaptures` runs after every transaction removes its directory and after
///       startup recovery,
///  so the last transaction leaves no record behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pruneLandedCaptures({ gitDir, registryRoot }): Promise<number>;
/// ```
pub fn prune_landed_captures(git_dir: &Path, registry_root: &Path) -> usize {
    match prune_or_fail(git_dir, registry_root) {
        Ok(count) => return count,
        Err(error) => {
            warn(
                "pruneLandedCaptures",
                format!(
                    "could not prune landed-capture records in {}; they stay for the next pruning: {error}",
                    git_dir.display()
                )
                .as_str(),
            );
            return 0;
        }
    }
}

/// What:
///  The pruning steps,
///  with failures returned.
/// Why:
///   `prune_landed_captures` turns any failure into a warning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of pruneLandedCaptures inside its try
/// ```
fn prune_or_fail(git_dir: &Path, registry_root: &Path) -> Result<usize, CaptureError> {
    let names: Vec<String> = list_landed_record_names(git_dir)?;
    if names.is_empty() {
        return Ok(0);
    }
    let worktree_id: Option<String> = read_worktree_id(git_dir)?;
    let Some(bound) = pinning_bound(registry_root, worktree_id.as_deref())? else {
        debug(
            "pruneLandedCaptures",
            "a published transaction has not captured yet; keeping every landed-capture record",
        );
        return Ok(0);
    };
    let directory: PathBuf = landed_record_directory(git_dir);
    // `mut` allows counting removals.
    let mut removed: usize = 0;
    for name in &names {
        if !prunable(
            directory.as_path(),
            name.as_str(),
            worktree_id.as_deref(),
            bound,
        ) {
            continue;
        }
        let path: PathBuf = directory.join(name);
        match std::fs::remove_file(&path) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => removed += 1,
            Err(error) => {
                return Err(CaptureError(format!(
                    "removing {} failed: {error}",
                    path.display()
                )));
            }
        }
    }
    debug(
        "pruneLandedCaptures",
        format!("pruned {removed} of {} landed-capture records", names.len()).as_str(),
    );
    return Ok(removed);
}

/// The incumbent's `Infinity` bound cannot be exceeded by a safe integer.
const _: () = assert!(MAX_SAFE_INTEGER < i64::MAX);

/// Record controls stay out of the release executable.
#[cfg(test)]
#[path = "capture_records_tests.rs"]
mod tests;
