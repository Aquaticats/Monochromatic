//! What:
//!  The per-branch `last-pushed` record that single-flight auto-push coordinates through.
//! Why:
//!  The incumbent (`src/auto-push-record.ts`) and this wrapper must read each other's records,
//!  because a commit waiting on a push made by the other wrapper decides from the same file.
//!  The record is a cache: an absent or unreadable record only costs an extra push,
//!  so only a read failure other than "absent" stops the caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! readLastPushedRecord(recordPath): Promise<LastPushedRecord | typeof LAST_PUSHED_ABSENT>;
//! writeLastPushedRecord({ recordPath, record }): Promise<void>;
//! ```

/// The safe-integer and string field readers, the object parser and the JSON string quoting.
use super::diagnostic_log::warn;
use super::json_record::{parse_object, quote, safe_integer_field, string_field};
/// The exclusive private file creation the incumbent does with mode `0600`.
use super::private_storage::write_exclusive_synced;
/// Unique candidate names, as the incumbent's `randomUUID`.
use super::random_id::random_uuid;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Record file name suffix after the encoded branch key.
pub const LAST_PUSHED_SUFFIX: &str = ".last-pushed.json";

/// Record schema version the incumbent writes and reads.
pub const LAST_PUSHED_SCHEMA_VERSION: i64 = 1;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// What:
///  Outcome of one recorded push attempt.
/// Why:
///   Only `pushed` and `failed` exist in the incumbent's record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LastPushedOutcome = 'pushed' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LastPushedOutcome {
    /// The push succeeded.
    Pushed,
    /// The push failed; the record keeps its output.
    Failed,
}

impl LastPushedOutcome {
    /// The spelling the record stores.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// outcome
    /// ```
    pub fn as_str(self) -> &'static str {
        return match self {
            Self::Pushed => "pushed",
            Self::Failed => "failed",
        };
    }
}

/// What:
///  The most recent push attempt for one branch.
/// Why:
///   `owner_token` tells a waiting commit whether a push finished after it first read the record;
///  `output` is the complete text of a failed push, which every joined commit surfaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LastPushedRecord = { schemaVersion: 1; tip; outcome; ownerToken; ownerPid; exitCode; output };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LastPushedRecord {
    /// Branch tip the attempt resolved before pushing; never empty.
    pub tip: String,
    /// Whether the push succeeded.
    pub outcome: LastPushedOutcome,
    /// Owner-lock token of the attempt; never empty.
    pub owner_token: String,
    /// Process that ran the attempt.
    pub owner_pid: i64,
    /// Push exit code, `0` on success.
    pub exit_code: i64,
    /// Complete push output of a failed attempt; empty on success.
    pub output: String,
}

/// What:
///  What reading a branch's record found.
/// Why:
///   Absence and an unreadable record both mean "no attempt"; they differ only in the diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// LastPushedRecord | typeof LAST_PUSHED_ABSENT
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LastPushedRead {
    /// A valid record.
    Record(LastPushedRecord),
    /// No record, or a record that does not validate.
    Absent,
}

/// What:
///  Parse and validate record text.
///       `Option<A>` is "a value or nothing".
/// Why:
///   Every field the incumbent checks is checked, so a malformed record never drives a join.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLastPushedRecord(text: string): LastPushedRecord; // throws TypeError
/// ```
pub fn parse_last_pushed_record(text: &str) -> Option<LastPushedRecord> {
    let object = parse_object(text)?;
    if safe_integer_field(&object, "schemaVersion") != Some(LAST_PUSHED_SCHEMA_VERSION) {
        return None;
    }
    let tip = non_empty_string(&object, "tip")?;
    let outcome = match string_field(&object, "outcome")? {
        "pushed" => LastPushedOutcome::Pushed,
        "failed" => LastPushedOutcome::Failed,
        _ => return None,
    };
    let owner_token = non_empty_string(&object, "ownerToken")?;
    let owner_pid = safe_integer_field(&object, "ownerPid")?;
    let exit_code = safe_integer_field(&object, "exitCode")?;
    let output = string_field(&object, "output")?;
    return Some(LastPushedRecord {
        tip,
        outcome,
        owner_token,
        owner_pid,
        exit_code,
        output: output.to_owned(),
    });
}

/// A string field that must not be empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typeof value.tip === 'string' && value.tip !== ''
/// ```
fn non_empty_string(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    let text = string_field(object, key)?;
    if text.is_empty() {
        return None;
    }
    return Some(text.to_owned());
}

/// What:
///  Read a branch's record, treating an absent or unreadable one as "no attempt".
/// Why:
///   A missing file is the normal first push;
///  any other read failure (permission denied, a directory in its place) must stop the caller,
///  and a record that fails validation is replaced by the next push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readLastPushedRecord(recordPath): Promise<LastPushedRecord | typeof LAST_PUSHED_ABSENT>;
/// ```
pub fn read_last_pushed_record(record_path: &Path) -> std::io::Result<LastPushedRead> {
    let bytes = match std::fs::read(record_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LastPushedRead::Absent);
        }
        Err(error) => return Err(error),
    };
    let text = String::from_utf8_lossy(&bytes);
    return match parse_last_pushed_record(&text) {
        Some(record) => Ok(LastPushedRead::Record(record)),
        None => {
            warn(
                TAG,
                &format!(
                    "ignoring unreadable last-pushed record {}; the next push replaces it",
                    record_path.display()
                ),
            );
            Ok(LastPushedRead::Absent)
        }
    };
}

/// What:
///  Serialize a record in the incumbent's `JSON.stringify` key order, with its trailing newline.
/// Why:
///   The incumbent and this wrapper write the same bytes for the same record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify(record)}\n`
/// ```
pub fn encode_last_pushed_record(record: &LastPushedRecord) -> String {
    return format!(
        "{{\"schemaVersion\":{LAST_PUSHED_SCHEMA_VERSION},\"tip\":{},\"outcome\":{},\"ownerToken\":{},\"ownerPid\":{},\"exitCode\":{},\"output\":{}}}\n",
        quote(&record.tip),
        quote(record.outcome.as_str()),
        quote(&record.owner_token),
        record.owner_pid,
        record.exit_code,
        quote(&record.output),
    );
}

/// What:
///  Publish a branch's record by rename, so readers never see a partial record.
/// Why:
///   The candidate is written exclusively with mode `0600` beside the record and renamed over it;
///  a failed publication removes the candidate and returns the error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeLastPushedRecord({ recordPath, record }): Promise<void>;
/// ```
pub fn write_last_pushed_record(
    record_path: &Path,
    record: &LastPushedRecord,
) -> std::io::Result<()> {
    let candidate_name = format!("{}.{}.pending", record_path.display(), random_uuid_or_io()?);
    let candidate = Path::new(&candidate_name);
    let bytes = encode_last_pushed_record(record);
    let published = write_exclusive_synced(candidate, bytes.as_bytes()).and_then(|()| {
        return std::fs::rename(candidate, record_path);
    });
    if let Err(error) = published {
        let _removed = std::fs::remove_file(candidate);
        return Err(error);
    }
    return Ok(());
}

/// A fresh unique suffix; a failed random source becomes an I/O error for the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// randomUUID()
/// ```
fn random_uuid_or_io() -> std::io::Result<String> {
    return random_uuid().map_err(|error| {
        return std::io::Error::other(error.0);
    });
}

/// Parsing, validation, publication and the candidate removal are controlled in the test module.
#[cfg(test)]
#[path = "auto_push_record_tests.rs"]
mod tests;
