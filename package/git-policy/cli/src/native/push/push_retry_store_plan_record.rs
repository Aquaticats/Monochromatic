//! What:
//!  The record a retry plan is kept as:
//!  its version,
//!  the two bounds a plan file is held to,
//!  and the JSON that reads back as the plan.
//! Why:
//!  The step that writes a plan and the step that reads it back agree here,
//!  and a file that is no plan this store wrote is refused rather than guessed at.
//!  The directory goes in as text and as the bytes of its name,
//!  so a plan is read back as the directory it was saved for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type StoredPlan = { gitDir: string; item: WorkItem; attempt: number };
//! const encoded = encode(plan); const decoded = decode(encoded);
//! ```

/// Import the reader and writer of a path's bytes in a record,
/// which is what a directory whose name is not text needs.
use crate::path_bytes_in_records as path_bytes;
/// Import the push item a plan schedules.
use crate::work_item::WorkItem;
/// Import the map and the value a plan's JSON record is built from.
use serde_json::{
    Map,
    Value,
};
/// Import the owned path type a plan names its Git directory with.
use std::path::PathBuf;

/// The record format version.
pub const PLAN_VERSION: u64 = 1;

/// The extension of a plan file.
pub(super) const PLAN_EXTENSION: &str = "plan";

/// The longest path this system allows,
/// which is what a plan names.
const LONGEST_PATH_BYTES: u64 = 4096;

/// The most bytes one plan's file may hold.
/// Why:
///   A plan is one JSON object naming a directory and a push item.
/// The directory goes in twice, as text and as the hex of its bytes,
/// the worktree once as text,
/// a backslash or a quote in a name doubles its bytes in the JSON,
/// and so does the hex,
/// so eight times the longest path holds every name the system allows;
/// a longer file is not one this store wrote,
/// and reading it whole would cost the service memory for nothing.
pub(super) const PLAN_FILE_LIMIT: u64 = 8 * LONGEST_PATH_BYTES;

/// One plan as it is kept on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredPlan {
    /// The Git directory whose service pushes.
    pub git_dir: PathBuf,
    /// The newest push item of the branch.
    pub item: WorkItem,
    /// The number of the try that is due.
    pub attempt: u32,
}

/// The text of a plan file.
pub(super) fn encode(plan: &StoredPlan) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(PLAN_VERSION));
    fields.insert(String::from("attempt"), Value::from(plan.attempt));
    // The directory goes in as plain text and as its bytes, so a plan is read back as the directory it was saved for.
    path_bytes::write_field(&mut fields, "gitDir", plan.git_dir.as_path());
    fields.insert(String::from("item"), Value::from(plan.item.encode()));
    return format!("{}\n", Value::Object(fields));
}

/// The plan a file's text holds,
/// or why it holds none.
pub(super) fn decode(text: &str) -> Result<StoredPlan, String> {
    let value: Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => return Err(format!("the plan is not JSON: {error}")),
    };
    if value.get("v").and_then(Value::as_u64) != Some(PLAN_VERSION) {
        return Err(String::from("the plan has an unsupported version"));
    }
    let Some(number) = value.get("attempt").and_then(Value::as_u64) else {
        return Err(String::from("the plan has no try number"));
    };
    let Ok(attempt) = u32::try_from(number) else {
        return Err(String::from("the plan has no try number"));
    };
    let Some(item_text) = value.get("item").and_then(Value::as_str) else {
        return Err(String::from("the plan holds no push item"));
    };
    let item: WorkItem = WorkItem::decode(item_text)?;
    return Ok(StoredPlan {
        git_dir: path_bytes::read_field(&value, "gitDir")?,
        item,
        attempt,
    });
}
