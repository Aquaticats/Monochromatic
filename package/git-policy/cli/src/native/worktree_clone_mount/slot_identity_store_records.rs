//! What:
//!  One record file's grammar:
//!  the header line,
//!  the record kinds,
//!  a record's own encoding,
//!  and the reading of one two-line file into a record or the refusal of that file.
//! Why:
//!  A record decides which folder a clone may be attached to,
//!  so a file this module cannot read refuses rather than being skipped,
//!  and a slot record's name is checked as one plain folder name before it is written.
//!  The key a record carries is the name it is filed under,
//!  so a record copied,
//!  hard-linked or restored to another name refuses:
//!  one name means one identity for the folder it records.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseOneRecord(text, 'add-cache-layer'); // { kind, name, identity } or a refusal naming the line
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::slot_identity_store_refusals::StoreRefusal;
use super::slot_name::slot_name_refusal;
use std::path::Path;

/// The store's first line:
/// the file's own name and the version of its grammar,
/// so a later format cannot be read as this one.
/// Version 2 is the folder:
/// one file per slot,
/// and `.parent` for the parent folder.
pub const STORE_HEADER: &str =
    "# monochromatic-git-policy-cli worktree clone slot identities, version 2";

/// The name `.claude/worktrees`'s record is filed under,
/// the only name this layout reserves.
pub const PARENT_RECORD_NAME: &str = ".parent";

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

/// The longest slot name the store records:
/// what the kernel accepts as one name,
/// which the probe measured as 255 bytes created and 256 refused `ENAMETOOLONG`.
pub const SLOT_NAME_MAXIMUM_BYTES: u64 = 255;

/// How many lowercase hexadecimal digits a slot publication's staging name holds,
/// which is the sixteen random bytes it was made from.
const STAGING_NAME_DIGITS: usize = 32;

/// The prefix of the staging name the parent's replacing writer makes,
/// which is `.` joined to `.parent` and a separator.
const PARENT_STAGING_PREFIX: &str = "..parent.";

/// The suffix of that staging name.
const PARENT_STAGING_SUFFIX: &str = ".partial";

/// What:
///  One record file:
///  the parent folder's identity,
///  or one slot's.
/// Why:
///   The two kinds are read by different checks and told apart by the record,
///  so a caller looks up what it needs without knowing the file's layout.
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
    /// The record's own key,
    /// which is the name it is filed under.
    pub fn key(&self) -> &str {
        return match self {
            StoredIdentity::Parent(_identity) => PARENT_KEY,
            StoredIdentity::Slot {
                name,
                ..
            } => name.as_str(),
        };
    }

    /// The record as its one line,
    /// without the newline.
    pub fn encode(&self) -> String {
        return match self {
            StoredIdentity::Parent(identity) => format!(
                "{PARENT_KIND}{FIELD_SEPARATOR}{PARENT_KEY}{FIELD_SEPARATOR}{}",
                identity.encode()
            ),
            StoredIdentity::Slot {
                name,
                identity,
            } => format!(
                "{SLOT_KIND}{FIELD_SEPARATOR}{name}{FIELD_SEPARATOR}{}",
                identity.encode()
            ),
        };
    }
}

/// What:
///  The record a file filed under `filed_under` holds,
///  or the line that stopped it.
/// Why:
///   A record file is the version 2 header and exactly one record line,
///  and a file that is partly written,
///  holds a second record or holds a key another name belongs to refuses:
///  nothing is attached to a slot until the file reads whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseOneRecord(text, name); // throws on a line this repository did not write
/// ```
///
/// # Errors
/// A first line that is not the header,
/// a file with no record line,
/// a second record line,
/// a record that is not one this module wrote,
/// or a record whose key is not the name it is filed under.
pub(super) fn parse_one_record(
    path: &Path,
    text: &str,
    filed_under: &str,
) -> Result<StoredIdentity, StoreRefusal> {
    let mut header_seen: bool = false;
    let mut record: Option<StoredIdentity> = None;
    for (index, line) in text.lines().enumerate() {
        let number: usize = index + 1;
        if line.is_empty() {
            continue;
        }
        if !header_seen {
            if line != STORE_HEADER {
                return Err(StoreRefusal::Line {
                    path: path.to_path_buf(),
                    number,
                    reason: format!(
                        "the first line must be this file's own header, `{STORE_HEADER}`"
                    ),
                });
            }
            header_seen = true;
            continue;
        }
        if line.starts_with(COMMENT_PREFIX) {
            continue;
        }
        if record.is_some() {
            return Err(StoreRefusal::Line {
                path: path.to_path_buf(),
                number,
                reason: String::from(
                    "a record file holds exactly one record line, and this line holds a second",
                ),
            });
        }
        record = Some(parse_record_line(path, number, line)?);
    }
    if !header_seen {
        return Err(StoreRefusal::Line {
            path: path.to_path_buf(),
            number: 1,
            reason: format!("the file is empty: it must start with `{STORE_HEADER}`"),
        });
    }
    let Some(parsed) = record else {
        return Err(StoreRefusal::Line {
            path: path.to_path_buf(),
            number: 1,
            reason: String::from("the file holds a header and no record line"),
        });
    };
    check_filed_under(path, filed_under, &parsed)?;
    return Ok(parsed);
}

/// Whether the record's own key is the name its file is filed under.
fn check_filed_under(
    path: &Path,
    filed_under: &str,
    record: &StoredIdentity,
) -> Result<(), StoreRefusal> {
    let matches: bool = match record {
        StoredIdentity::Parent(_identity) => filed_under == PARENT_RECORD_NAME,
        StoredIdentity::Slot {
            name,
            ..
        } => name.as_str() == filed_under,
    };
    if matches {
        return Ok(());
    }
    return Err(StoreRefusal::RecordUnderAnotherName {
        path: path.to_path_buf(),
        filed_under: filed_under.to_string(),
        recorded: record.key().to_string(),
    });
}

/// One record from one line.
fn parse_record_line(
    path: &Path,
    number: usize,
    line: &str,
) -> Result<StoredIdentity, StoreRefusal> {
    let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
    if fields.len() != 3 {
        return Err(StoreRefusal::Line {
            path: path.to_path_buf(),
            number,
            reason: format!(
                "a record holds three space-separated fields, and this line holds {}",
                fields.len()
            ),
        });
    }
    let identity: CloneFolderIdentity =
        CloneFolderIdentity::decode(fields[2]).ok_or_else(|| {
            return StoreRefusal::Line {
                path: path.to_path_buf(),
                number,
                reason: format!("`{}` is not a recorded folder identity", fields[2]),
            };
        })?;
    return match fields[0] {
        PARENT_KIND if fields[1] == PARENT_KEY => Ok(StoredIdentity::Parent(identity)),
        PARENT_KIND => Err(StoreRefusal::Line {
            path: path.to_path_buf(),
            number,
            reason: format!(
                "a parent record's key is `{PARENT_KEY}`, and this line holds `{}`",
                fields[1]
            ),
        }),
        SLOT_KIND => {
            if let Some(reason) = slot_name_refusal(fields[1]) {
                return Err(StoreRefusal::Line {
                    path: path.to_path_buf(),
                    number,
                    reason: format!("`{}` is no slot name: {reason}", fields[1]),
                });
            }
            Ok(StoredIdentity::Slot {
                name: fields[1].to_string(),
                identity,
            })
        }
        other => Err(StoreRefusal::Line {
            path: path.to_path_buf(),
            number,
            reason: format!("`{other}` is neither `{PARENT_KIND}` nor `{SLOT_KIND}`"),
        }),
    };
}
/// Whether `name` is a dot name:
/// a parent record,
/// a staging entry,
/// or a name this layout does not know.
pub(super) fn is_a_dot_name(name: &str) -> bool {
    return name.starts_with('.');
}

/// What:
///  Whether `name` is one of the two staging forms this layout's writers make,
///  so a remover may remove it and every reader may ignore it.
/// Why:
///  A writer's staging name is not a record and is never read as one;
///  only the forms this repository makes are removed,
///  and a dot entry matching neither is left for an administrator,
///  because a third form means a writer this layout has not accounted for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const known = /^\.[0-9a-f]{32}$/.test(name) || (name.startsWith('..parent.') && name.endsWith('.partial'));
/// ```
pub fn is_known_staging_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('.') else {
        return false;
    };
    if rest.len() == STAGING_NAME_DIGITS
        && rest
            .chars()
            .all(|digit| return digit.is_ascii_digit() || ('a'..='f').contains(&digit))
    {
        return true;
    }
    let inner: Option<&str> = name
        .strip_prefix(PARENT_STAGING_PREFIX)
        .and_then(|after_the_prefix| return after_the_prefix.strip_suffix(PARENT_STAGING_SUFFIX));
    return inner.is_some_and(|random| return !random.is_empty());
}
