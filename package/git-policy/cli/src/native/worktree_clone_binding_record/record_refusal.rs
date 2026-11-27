//! What:
//!  Why a byte sequence is not a record of this grammar,
//!  naming the line or the field that decided it,
//!  and the sentence a person reads.
//! Why:
//!  The project decided on 2026-11-22 that a record a later release cannot read is refused by name and waited on,
//!  so a refusal names the file,
//!  the line and the way forward,
//!  and no command acts on the record it refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { line?: number; message: string };
//! ```

/// What:
///  Why a byte sequence is not a record of this grammar,
///  naming the line or the field that decided it.
/// Why:
///  The project decided on 2026-11-22 that a record a later release cannot read is refused by name and waited on,
///  so every refusal carries the bytes' own words and the line they stood on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { line?: number; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRefusal {
    /// The bytes are longer than the record they claim to be.
    TooLarge {
        /// The bytes found.
        found: u64,
        /// The bound they passed.
        bound: u64,
    },
    /// One line is longer than the widest form its grammar allows.
    LineTooLarge {
        /// The line number,
        /// counting the header as one.
        line: usize,
        /// The bytes found.
        found: u64,
        /// The bound they passed.
        bound: u64,
    },
    /// The bytes are not UTF-8.
    NotUtf8 {
        /// The line whose bytes failed,
        /// counting the header as one.
        line: usize,
    },
    /// The first line is not this grammar's header.
    Header {
        /// The line as it was found.
        found: String,
    },
    /// A line does not hold exactly one `=`.
    NotAField {
        /// The line number.
        line: usize,
        /// The line as it was found,
        /// bounded to what a person can read.
        found: String,
    },
    /// A key a later grammar may write,
    /// or damage.
    UnknownKey {
        /// The line number.
        line: usize,
        /// The key as it was found.
        key: String,
    },
    /// A key out of the grammar's order,
    /// which also catches one written twice.
    KeyOutOfOrder {
        /// The line number.
        line: usize,
        /// The key as it was found.
        key: String,
        /// The key the grammar holds here.
        expected: String,
    },
    /// A value the field cannot hold.
    Value {
        /// The line number.
        line: usize,
        /// The field's key.
        key: String,
        /// Why the value is refused.
        reason: String,
    },
    /// A field every record of this phase must hold.
    Missing {
        /// The field's key.
        key: String,
        /// The phase the record names.
        phase: String,
    },
    /// A field written before the phase that proves it.
    AheadOfItsPhase {
        /// The field's key.
        key: String,
        /// The phase the record names.
        phase: String,
    },
    /// A record at `phase=settling` with no reason,
    /// or a reason beside another phase.
    SettleReason {
        /// Why the pair is refused.
        reason: String,
    },
    /// An entry count that disagrees with the entry lines.
    EntryCount {
        /// The count the file states.
        stated: u64,
        /// The lines the file holds.
        found: u64,
    },
    /// A kind that disagrees with the name the file was found under.
    Kind {
        /// The kind the file states.
        found: String,
        /// The kind its name holds.
        expected: String,
    },
}

impl RecordRefusal {
    /// The sentence a person reads,
    /// naming the line and the way forward.
    pub fn message(&self) -> String {
        return match self {
            RecordRefusal::TooLarge {
                found,
                bound,
            } => format!(
                "this record is {found} bytes long and the widest record its grammar allows is {bound} bytes, so it was not read. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::LineTooLarge {
                line,
                found,
                bound,
            } => format!(
                "line {line} of this record is {found} bytes long and the widest line its grammar allows is {bound} bytes, so it was not read. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::NotUtf8 {
                line,
            } => {
                let _ = line;
                format!(
                    "this record\x27s bytes are not UTF-8, so the record was not read. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
                )
            }
            RecordRefusal::Header {
                found,
            } => format!(
                "this record's first line is `{found}`, which is not the header of this release's grammar, so the record was not read as one. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::NotAField {
                line,
                found,
            } => format!(
                "line {line} of this record is `{found}`, which is not one `key=value` field, so the record was not read. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::UnknownKey {
                line,
                key,
            } => format!(
                "line {line} of this record holds the key `{key}`, which this release does not write, so the record was not read: an unknown key is either a later grammar or damage, and neither may be read as this one. Nothing was changed. Ways forward: read the file, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::KeyOutOfOrder {
                line,
                key,
                expected,
            } => format!(
                "line {line} of this record holds `{key}` where the grammar holds `{expected}`, so the record's fields are out of order or one is written twice. Nothing was changed. Ways forward: read the file, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::Value {
                line,
                key,
                reason,
            } => format!(
                "line {line} of this record holds a `{key}` value this grammar cannot read ({reason}), so the record was not read. Nothing was changed. Ways forward: read the file, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::Missing {
                key,
                phase,
            } => format!(
                "this record stands at phase `{phase}` and holds no `{key}` field, which every record of that phase must hold. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::AheadOfItsPhase {
                key,
                phase,
            } => format!(
                "this record stands at phase `{phase}` and holds a `{key}` field that phase does not prove. Nothing was changed. Ways forward: read the file and compare it with the record the add wrote, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::SettleReason {
                reason,
            } => format!(
                "this record's phase and its settle reason disagree ({reason}), so the record was not read. Nothing was changed. Ways forward: read the file, restore it from a copy, or delete it and run the command again."
            ),
            RecordRefusal::EntryCount {
                stated,
                found,
            } => format!(
                "this inventory states {stated} entries and holds {found} entry lines, so a truncated or extended file would be read as history. Nothing was changed. Ways forward: read the file, restore it from a copy, or delete it and record the inventory again."
            ),
            RecordRefusal::Kind {
                found,
                expected,
            } => format!(
                "this inventory states kind `{found}` and is filed under a name that holds `{expected}`. Nothing was changed. Ways forward: read the file, restore it under its own name, or delete it and record the inventory again."
            ),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "record_refusal_tests.rs"]
mod tests;
