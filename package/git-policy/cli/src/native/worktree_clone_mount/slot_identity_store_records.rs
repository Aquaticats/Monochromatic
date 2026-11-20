//! What:
//!  One line of the slot identity store:
//!  the record kinds,
//!  a record's own encoding,
//!  and the reading of every line into a record or the refusal of that line.
//! Why:
//!  A record decides which folder a clone may be attached to,
//! so a line this module cannot read refuses the whole store rather than being skipped,
//! and a slot record's name is checked as one plain folder name before it is written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseRecord(line); // { kind, name, identity } or a refusal naming the line
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::slot_identity_store_refusals::StoreRefusal;
use std::collections::BTreeSet;
use std::path::Path;

/// The store's first line:
/// the file's own name and the version of its grammar,
/// so a later format cannot be read as this one.
pub(super) const STORE_HEADER: &str = "# monochromatic-git-policy-cli worktree clone slot identities, version 1";

/// The record kind of the `.claude/worktrees` parent folder.
const PARENT_KIND: &str = "parent";

/// The record kind of one clone slot.
const SLOT_KIND: &str = "slot";

/// The key a parent record carries,
/// because the parent folder has no slot name.
pub const PARENT_KEY: &str = "-";

/// The separator of one record's three fields.
const FIELD_SEPARATOR: char = ' ';

/// The first character of a line that says nothing about an identity.
const COMMENT_PREFIX: char = '#';

/// What:
///  One line of the store:
///  the parent folder's identity,
///  or one slot's.
/// Why:
///   The two kinds are read together and told apart by the record,
/// so a caller looks up what it needs without knowing the file's layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoredIdentity = { kind: 'parent'; identity } | { kind: 'slot'; name: string; identity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoredIdentity {
    /// The identity the setup run recorded for `.claude/worktrees`.
    Parent(CloneFolderIdentity),
    /// The identity recorded for one slot of the worktrees folder.
    Slot {
        /// The slot's name,
        /// one plain folder name.
        name: String,
        /// The identity.
        identity: CloneFolderIdentity,
    },
}

impl StoredIdentity {
    /// The record's own name for its key,
    /// used in a message about a duplicate.
    fn key(&self) -> &str {
        return match self {
            StoredIdentity::Parent(_identity) => PARENT_KEY,
            StoredIdentity::Slot { name, .. } => name.as_str(),
        };
    }

    /// The record as its store line,
    /// without the newline.
    pub fn encode(&self) -> String {
        return match self {
            StoredIdentity::Parent(identity) => format!("{PARENT_KIND}{FIELD_SEPARATOR}{PARENT_KEY}{FIELD_SEPARATOR}{}", identity.encode()),
            StoredIdentity::Slot { name, identity } => format!("{SLOT_KIND}{FIELD_SEPARATOR}{name}{FIELD_SEPARATOR}{}", identity.encode()),
        };
    }
}

/// The store's records from its text,
/// or the line that stopped it.
pub(super) fn parse_records(path: &Path, text: &str) -> Result<Vec<StoredIdentity>, StoreRefusal> {
    let mut records: Vec<StoredIdentity> = Vec::new();
    let mut keys: BTreeSet<String> = BTreeSet::new();
    let mut header_seen: bool = false;
    for (index, line) in text.lines().enumerate() {
        let number: usize = index + 1;
        if line.is_empty() {
            continue;
        }
        if !header_seen {
            if line != STORE_HEADER {
                return Err(StoreRefusal::Line { path: path.to_path_buf(), number, reason: format!("the first line must be the store's own header, `{STORE_HEADER}`") });
            }
            header_seen = true;
            continue;
        }
        if line.starts_with(COMMENT_PREFIX) {
            continue;
        }
        let record: StoredIdentity = parse_record(path, number, line)?;
        let key: String = record.key().to_string();
        if !keys.insert(key.clone()) {
            return Err(StoreRefusal::DuplicateKey { path: path.to_path_buf(), number, key });
        }
        records.push(record);
    }
    if !header_seen {
        return Err(StoreRefusal::Line { path: path.to_path_buf(), number: 1, reason: format!("the file is empty: it must start with `{STORE_HEADER}`") });
    }
    return Ok(records);
}

/// One record from one line.
fn parse_record(path: &Path, number: usize, line: &str) -> Result<StoredIdentity, StoreRefusal> {
    let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
    if fields.len() != 3 {
        return Err(StoreRefusal::Line { path: path.to_path_buf(), number, reason: format!("a record holds three space-separated fields, and this line holds {}", fields.len()) });
    }
    let identity: CloneFolderIdentity = CloneFolderIdentity::decode(fields[2]).ok_or_else(|| return StoreRefusal::Line { path: path.to_path_buf(), number, reason: format!("`{}` is not a recorded folder identity", fields[2]) })?;
    return match fields[0] {
        PARENT_KIND if fields[1] == PARENT_KEY => Ok(StoredIdentity::Parent(identity)),
        PARENT_KIND => Err(StoreRefusal::Line { path: path.to_path_buf(), number, reason: format!("a parent record's key is `{PARENT_KEY}`, and this line holds `{}`", fields[1]) }),
        SLOT_KIND => {
            if let Some(reason) = slot_name_refusal(fields[1]) {
                return Err(StoreRefusal::Line { path: path.to_path_buf(), number, reason: format!("`{}` is no slot name: {reason}", fields[1]) });
            }
            Ok(StoredIdentity::Slot { name: fields[1].to_string(), identity })
        }
        other => Err(StoreRefusal::Line { path: path.to_path_buf(), number, reason: format!("`{other}` is neither `{PARENT_KIND}` nor `{SLOT_KIND}`") }),
    };
}

/// What makes `name` no plain slot name,
/// or nothing when it is one.
pub(super) fn slot_name_refusal(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some(String::from("it is empty"));
    }
    if name == "." || name == ".." || name.contains('/') {
        return Some(String::from("it holds a path separator or names a folder above"));
    }
    if name.chars().any(char::is_whitespace) || name.chars().any(char::is_control) {
        return Some(String::from("it holds a space, a tab or another control character"));
    }
    return None;
}
