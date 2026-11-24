//! What:
//!  The field lines of a slot's binding record,
//!  read whatever phase the record stands at,
//!  and the refusals a record owes before any sentence may be recomputed.
//! Why:
//!  The lock's check order takes the nonce from any well-formed record of the slot,
//!  even when the binding's own reader refuses it as foreign or ahead of the live facts
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  step 1 of "How doctor and removal tell the states apart",
//!  and its limit `L12`),
//!  so the scan here decides the header,
//!  a repeated key and a value its encoding refuses,
//!  and nothing about the add's phase.
//!  The keys the lock's proof needs are named once,
//!  so the reader and its controls spell them the same.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fields = readFieldLines(recordBytes); // { nonce, slot, slotIdentity, ... } or a refusal naming the line
//! ```

use super::reading_a_bounded_file::ARefusedReading;
use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, hexadecimal_as_bytes, path_from_bytes};
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::registration_home::binding_name;
use std::path::{Path, PathBuf};

/// The key whose value every lock sentence is recomputed from.
pub const NONCE_KEY: &str = "nonce";

/// The key of the slot's recorded whole path.
pub const SLOT_KEY: &str = "slot";

/// The key of the underlay folder's recorded identity.
pub const SLOT_IDENTITY_KEY: &str = "slotIdentity";

/// The key of the registration's recorded path.
pub const REGISTRATION_KEY: &str = "registration";

/// The key of the registration directory's recorded identity.
pub const REGISTRATION_IDENTITY_KEY: &str = "registrationIdentity";

/// The key of the clone's recorded dataset name.
pub const CLONE_DATASET_KEY: &str = "cloneDataset";

/// The key of the clone's recorded GUID.
pub const CLONE_GUID_KEY: &str = "cloneGuid";

/// What:
///  Why a slot's binding record gave no facts a lock can be judged by.
/// Why:
///  Each cause is a different sentence for a person,
///  and none of them may be read as permission to release a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'absent' } | { kind: 'refused' | 'another-account' | 'writable-by-others' | 'not-this-grammar' | 'missing-field' | 'malformed'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordedFactsRefusal {
    /// No record stands under the slot's name.
    Absent,
    /// The name holds what this route refuses to read.
    Refused(ARefusedReading),
    /// The record belongs to another account.
    AnotherAccount {
        /// The owner the file records.
        owner: u32,
        /// The account reading it.
        reader: u32,
    },
    /// Another account could rewrite the record.
    WritableByOthers {
        /// The mode the file records.
        mode: u32,
    },
    /// The bytes are not a record of this grammar.
    NotThisGrammar {
        /// The one-based line that broke the grammar.
        line: usize,
        /// What that line held.
        found: String,
    },
    /// A field the lock's proof needs is absent.
    MissingField {
        /// The key whose value is absent.
        key: &'static str,
    },
    /// A field holds a value its encoding refuses.
    Malformed {
        /// The key whose value it is.
        key: &'static str,
        /// The value as the record holds it.
        found: String,
    },
}

impl RecordedFactsRefusal {
    /// The words a person reads,
    /// naming the record and what is absent from it.
    pub fn words(&self, slot: &str, home: &Path) -> String {
        let record: String = home.join(binding_name(slot)).display().to_string();
        return match self {
            RecordedFactsRefusal::Absent => format!("`{record}` does not exist"),
            RecordedFactsRefusal::Refused(cause) => format!("`{record}` could not be read ({})", cause.words()),
            RecordedFactsRefusal::AnotherAccount { owner, reader } => format!("`{record}` belongs to account {owner} and this command runs as account {reader}"),
            RecordedFactsRefusal::WritableByOthers { mode } => format!("`{record}` has mode {mode:04o}, which lets another account write it"),
            RecordedFactsRefusal::NotThisGrammar { line, found } => format!("`{record}` line {line} holds `{found}`, which is not a comment-free field line of the binding's grammar"),
            RecordedFactsRefusal::MissingField { key } => format!("`{record}` holds no `{key}=` line"),
            RecordedFactsRefusal::Malformed { key, found } => format!("`{record}` holds `{key}={found}`, which that key's encoding refuses"),
        };
    }
}

/// What:
///  The values the lock's proof reads,
///  each absent until its line arrives,
///  and a second line for one key refused as damage.
/// Why:
///  A record holds each key once,
///  so a repeated key is not a later grammar but a record two writers touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fields = { nonce: undefined, slot: undefined, ... }; fields[key] = value; // throws on a repeat
/// ```
#[derive(Default)]
pub(super) struct Fields {
    pub(super) nonce: Option<String>,
    pub(super) slot: Option<String>,
    pub(super) slot_identity: Option<String>,
    pub(super) registration: Option<String>,
    pub(super) registration_identity: Option<String>,
    pub(super) clone_dataset: Option<String>,
    pub(super) clone_guid: Option<String>,
}

impl Fields {
    /// Take one line's value under its key.
    fn set(&mut self, key: &str, value: &str, line: usize) -> Result<(), RecordedFactsRefusal> {
        let place: &mut Option<String> = match key {
            NONCE_KEY => &mut self.nonce,
            SLOT_KEY => &mut self.slot,
            SLOT_IDENTITY_KEY => &mut self.slot_identity,
            REGISTRATION_KEY => &mut self.registration,
            REGISTRATION_IDENTITY_KEY => &mut self.registration_identity,
            CLONE_DATASET_KEY => &mut self.clone_dataset,
            CLONE_GUID_KEY => &mut self.clone_guid,
            _unknown => return Ok(()),
        };
        if place.is_some() {
            return Err(RecordedFactsRefusal::NotThisGrammar { line, found: String::from(key) });
        }
        *place = Some(String::from(value));
        return Ok(());
    }
}

/// The value one key must hold,
/// or the refusal naming it.
pub(super) fn required<'a>(value: Option<&'a String>, key: &'static str) -> Result<&'a String, RecordedFactsRefusal> {
    return value.ok_or_else(|| return RecordedFactsRefusal::MissingField { key });
}

/// A path a record writes as lowercase hexadecimal,
/// with a value the encoding refuses named by its key.
pub(super) fn path_field(value: Option<&String>, key: &'static str) -> Result<Option<PathBuf>, RecordedFactsRefusal> {
    let Some(digits) = value else { return Ok(None) };
    let Some(bytes) = hexadecimal_as_bytes(digits.as_str()) else {
        return Err(RecordedFactsRefusal::Malformed { key, found: digits.clone() });
    };
    return match path_from_bytes(bytes.as_slice()) {
        Some(path) => Ok(Some(path)),
        None => Err(RecordedFactsRefusal::Malformed { key, found: digits.clone() }),
    };
}

/// A folder identity a record writes,
/// with a value the encoding refuses named by its key.
pub(super) fn identity_field(value: Option<&String>, key: &'static str) -> Result<Option<FolderIdentity>, RecordedFactsRefusal> {
    let Some(text) = value else { return Ok(None) };
    return match FolderIdentity::decode(text.as_str()) {
        Some(identity) => Ok(Some(identity)),
        None => Err(RecordedFactsRefusal::Malformed { key, found: text.clone() }),
    };
}

/// Each line's key and value,
/// in the order the record holds them,
/// with a line that is no field refused by its number.
pub(super) fn fields_of(text: &str) -> Result<Fields, RecordedFactsRefusal> {
    let mut fields: Fields = Fields::default();
    let mut lines: std::str::Lines<'_> = text.lines();
    let header: &str = lines.next().unwrap_or("");
    if header != BINDING_HEADER {
        return Err(RecordedFactsRefusal::NotThisGrammar { line: 1, found: String::from(header) });
    }
    let mut number: usize = 1;
    for line in lines {
        number += 1;
        if line.is_empty() {
            continue;
        }
        let Some((key, value)): Option<(&str, &str)> = line.split_once('=') else {
            return Err(RecordedFactsRefusal::NotThisGrammar { line: number, found: String::from(line) });
        };
        fields.set(key, value, number)?;
    }
    return Ok(fields);
}
