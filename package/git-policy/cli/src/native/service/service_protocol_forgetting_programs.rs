//! What:
//!  Reads the message that names the push records which can no longer run,
//!  so the service gives up the programs and transport settings it holds against them.
//! Why:
//!  The values live only in the service's memory (`service_push_programs_held_in_memory`),
//! and a record a committing wrapper
//!  settles or removes leaves nothing else for the service to learn the ids by.
//! The reader trusts nothing from a socket:
//! the directory must be absolute and hold no NUL,
//! and every id must be one the queue could have written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const forgettingPrograms = (value) => ({ gitDir: absolutePath(value.gitDir), records: value.records.map(anId) });
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::service_protocol::ClientMessage;
use crate::service_protocol_messages_and_name_checks::MAX_FORGOTTEN_RECORDS;
use serde_json::Value;
use std::path::PathBuf;

/// What:
///  The message a line asks for,
///  or the words the line is refused with.
/// Why:
///   Every field is checked here,
/// so the connection handler only remembers what the queue itself could have written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgettingPrograms(value) // { type: 'forgetPrograms', gitDir, records } | throw
/// ```
pub(crate) fn forgetting_programs(value: &Value) -> Result<ClientMessage, String> {
    let git_dir: PathBuf = path_bytes::read_field(value, "gitDir")?;
    if !git_dir.is_absolute() || git_dir.to_string_lossy().contains('\0') {
        return Err(String::from("forget request is not acceptable"));
    }
    let named: &Vec<Value> = value
        .get("records")
        .and_then(Value::as_array)
        .ok_or("records is missing")?;
    if named.len() > MAX_FORGOTTEN_RECORDS {
        return Err(String::from("number of records is not acceptable"));
    }
    let mut records: Vec<String> = Vec::new();
    for entry in named {
        let Some(id) = entry.as_str() else {
            return Err(String::from(
                "a forget request names a record that is not text",
            ));
        };
        if !crate::work_item_ids_and_names::is_valid_id(id) {
            return Err(String::from(
                "a forget request names a record that is no work record",
            ));
        }
        records.push(id.to_string());
    }
    return Ok(ClientMessage::ForgetPrograms {
        git_dir,
        records,
    });
}
