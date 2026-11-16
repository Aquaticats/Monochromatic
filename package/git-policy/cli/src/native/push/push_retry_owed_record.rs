//! What:
//!  The record a plan owes the person instead of another try:
//! what kind of record it is,
//! the words it carries,
//! how many looks are left that may write it,
//! and how a plan's file keeps it.
//! Why:
//!   A push whose end the storage refused leaves the plan holding the only copy of what the server said
//! (`service_work_drains_awaiting.rs`,
//! `write_the_record_it_owes`),
//! so the words and the bound on writing them travel in the plan itself
//! (`StoredPlan` in `push_retry_store.rs`).
//! A release that does not know this member of a plan's file tries the push once more instead,
//! which asks the server to say the same thing again,
//! so the plan's format version does not change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type OwedRecord = { kind: 'refused' | 'skipped'; words: string; looksLeft: number }
//! ```

use serde_json::{Map, Value};

/// What:
///  What a plan owes the person at its next look instead of another try:
/// the record of how its push ended,
/// which the storage refused when that end was decided.
/// Why:
///   The plan is the only durable record that a push waits for the person,
/// and the words of that record are the only copy of what the server said at its end.
/// Keeping both in one file means a look that finds the plan can write the record,
/// and a kill leaves the plan with its words rather than neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwedRecord = { kind: OwedRecordKind; words: string; looks_left: number }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwedRecord {
    /// The kind of record the plan owes.
    pub kind: OwedRecordKind,
    /// The words the record carries,
    /// cleaned and cut as the record itself keeps them.
    pub words: String,
    /// How many looks are left that may write it,
    /// so that a storage that keeps refusing it is given up on rather than asked for ever.
    pub looks_left: u32,
}

/// The records a push's end can owe the person.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwedRecordKind {
    /// A push Git refused,
    /// whose record says what the server said.
    Refused,
    /// A push the service did not make,
    /// whose record says why in the service's own words.
    Skipped,
}

/// Methods over the kind.
impl OwedRecordKind {
    /// The name the kind is kept under.
    fn kept_name(self) -> &'static str {
        return match self {
            Self::Refused => "refused",
            Self::Skipped => "skipped",
        };
    }
}

/// What:
///  The member a plan's file keeps the record under.
/// Why:
///   The record is a part of the plan and never a file of its own,
/// so a plan that owes one is the same file a try is,
/// and a newer plan of the branch replaces either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptMember(owed) // { kind: owed.kind, words: owed.words, looks: owed.looksLeft }
/// ```
pub fn kept_member(owed: &OwedRecord) -> Value {
    let mut kept: Map<String, Value> = Map::new();
    kept.insert(String::from("kind"), Value::from(owed.kind.kept_name()));
    kept.insert(String::from("words"), Value::from(owed.words.clone()));
    kept.insert(String::from("looks"), Value::from(owed.looks_left));
    return Value::Object(kept);
}

/// What:
///  The record a plan owes the person,
/// as a file keeps it,
/// or why the file names none it can be read as.
/// Why:
///   A file whose words or looks cannot be read schedules no look that could write them,
/// so a plan it holds is one this store did not write and `load` removes it rather than keeping a push nobody is told
/// about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// owedRecordOf(kept) // { kind, words, looksLeft }
/// ```
pub fn owed_record_of(kept: &Value) -> Result<OwedRecord, String> {
    let kind: OwedRecordKind = match kept.get("kind").and_then(Value::as_str) {
        Some("refused") => OwedRecordKind::Refused,
        Some("skipped") => OwedRecordKind::Skipped,
        Some(other) => return Err(format!("the plan owes a record of a kind this release does not know: {other}")),
        None => return Err(String::from("the plan owes a record and names no kind")),
    };
    let Some(words) = kept.get("words").and_then(Value::as_str) else {
        return Err(String::from("the plan owes a record and names no words"));
    };
    let Some(looks) = kept.get("looks").and_then(Value::as_u64) else {
        return Err(String::from("the plan owes a record and names no looks"));
    };
    let Ok(looks_left) = u32::try_from(looks) else {
        return Err(String::from("the plan owes a record and names no looks"));
    };
    return Ok(OwedRecord { kind, words: String::from(words), looks_left });
}
