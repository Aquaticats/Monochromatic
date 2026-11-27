//! What:
//!  The words one unfilled-copy record holds:
//!  its suffix and the five fields of a record,
//!  the record a worktree a copy gave up on is named by,
//!  how those words are written and read back,
//!  and the name the record of one journal takes.
//! Why:
//!  The writer and the reader must agree on every field the reader acts on,
//! and a record the reader cannot parse proves nothing about the worktree it names,
//!  so the spelling lives with the two functions that write and read it
//!  (`worktree_copy_unfilled_record.rs`),
//!  which writes the record before an installation and removes it when one completes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = encodeUnfilled({ destination, destinationIdentity, source, reason, owner });
//! const copy = decodeUnfilled(text); // throws when its lines are not one this release writes
//! ```

use crate::path_bytes_in_records::{
    hex_of_bytes,
    hex_of_path,
    path_of_hex,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The suffix of a record that names a worktree left without part of its ignored state.
/// Why:
///   Journals end in `.json` and claims in `.stage`,
/// so a record of this kind is never read as one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNFILLED_SUFFIX = '.unfilled';
/// ```
pub(crate) const UNFILLED_SUFFIX: &str = ".unfilled";

/// What:
///  The field of the record that names its owner.
/// Why:
///   A record written by a running installation is that installation's,
/// and its owner is what tells a later command whether the
///  instrument may be settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWNER_FIELD = 'owner';
/// ```
const OWNER_FIELD: &str = "owner";

/// What:
///  The field of the record that names the worktree.
/// Why:
///   The bytes of the path are the one form that loses nothing,
/// and the record is read by commands that must act on exactly that worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DESTINATION_FIELD = 'destination';
/// ```
const DESTINATION_FIELD: &str = "destination";

/// What:
///  The field of the record that holds the worktree's identity.
/// Why:
///   A worktree removed and made again at the same path settles the record,
/// and only the object's identity tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDENTITY_FIELD = 'destinationIdentity';
/// ```
const IDENTITY_FIELD: &str = "destinationIdentity";

/// What:
///  The field of the record that names the worktree the ignored state came from.
/// Why:
///   The files the copy did not install are still in that source,
/// and the person needs its name to know what is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SOURCE_FIELD = 'source';
/// ```
const SOURCE_FIELD: &str = "source";

/// What:
///  The field of the record that holds what the installation ended with.
/// Why:
///   The command that ended the copy says why once;
/// the record keeps the reason for every later command that names the worktree again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REASON_FIELD = 'reason';
/// ```
const REASON_FIELD: &str = "reason";

/// What:
///  One record as written and as read back.
/// Why:
///   The writer and the reader must agree on every field the reader acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnfilledCopy = { destination: string; destinationIdentity: ObjectId; source: string; reason: string; owner?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnfilledCopy {
    /// The worktree that was left without part of its source's ignored state.
    pub destination: PathBuf,
    /// The identity of that worktree when the copy gave up.
    pub destination_identity: crate::fs_beneath::ObjectId,
    /// The worktree whose ignored state was being copied.
    pub source: PathBuf,
    /// What the installation ended with.
    pub reason: String,
    /// The owner that wrote the record,
    /// when the record names one.
    pub owner: Option<String>,
}

/// What:
///  The value of one `key=value` line of a record.
/// Why:
///   Every field of a record is one line,
/// and a field is looked up by its key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldOf = (text, key) => text.split('\n').find((line) => line.startsWith(`${key}=`))?.slice(key.length + 1);
/// ```
fn field_of<'a>(
    text: &'a str,
    key: &str,
) -> Option<&'a str> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        if let Some(value) = rest.strip_prefix('=') {
            return Some(value);
        }
    }
    return None;
}

/// What:
///  The bytes a hexadecimal field spells.
/// Why:
///   The reason holds whatever the operating system said,
/// which is not text that one line can hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytesOfHex = (text) => Uint8Array.from(text.match(/../g) ?? [], (pair) => parseInt(pair, 16));
/// ```
fn bytes_of_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || !text.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.as_bytes().chunks(2) {
        bytes.push(u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
    }
    return Some(bytes);
}

/// What:
///  The reason a record's bytes are not the text a reader names.
/// Why:
///   The reason holds whatever the operating system said,
/// and a reader must be able to tell it from a record it cannot parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notText = () => 'what it says the copy ended with is not text';
/// ```
fn not_text(_undecodable: std::string::FromUtf8Error) -> &'static str {
    return "what it says the copy ended with is not text";
}

/// What:
///  The record of one worktree a copy gave up on,
/// as the lines its reader parses.
/// Why:
///   A record the reader cannot parse proves nothing about the worktree,
/// so the reader keeps it and says so instead of acting on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${DESTINATION_FIELD}=${hexOfPath(destination)}\n${IDENTITY_FIELD}=${id}\n${SOURCE_FIELD}=${hexOfPath(source)}\n${REASON_FIELD}=${hexOfBytes(reason)}\nowner=${owner}\n`
/// ```
pub fn encode_unfilled(copy: &UnfilledCopy) -> String {
    let owner: String = copy.owner.clone().unwrap_or_default();
    return format!(
        "{DESTINATION_FIELD}={}\n{IDENTITY_FIELD}={}\n{SOURCE_FIELD}={}\n{REASON_FIELD}={}\n{OWNER_FIELD}={owner}\n",
        hex_of_path(copy.destination.as_path()),
        copy.destination_identity.encode(),
        hex_of_path(copy.source.as_path()),
        hex_of_bytes(copy.reason.as_bytes()),
    );
}

/// What:
///  The record a command wrote,
/// or the reason its lines are not one this release writes.
/// Why:
///   Recovery acts on a worktree this record names,
/// so a line that is not of this release's shape authorizes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decodeUnfilled = (text) => ({ destination: pathOfHex(fieldOf(text, 'destination')), ... });
/// ```
pub fn decode_unfilled(text: &str) -> Result<UnfilledCopy, String> {
    let destination: PathBuf =
        path_of_hex(field_of(text, DESTINATION_FIELD).ok_or("it names no worktree")?)
            .ok_or("its worktree is not a path")?;
    let identity: crate::fs_beneath::ObjectId = field_of(text, IDENTITY_FIELD)
        .and_then(crate::fs_beneath::ObjectId::decode)
        .ok_or("it holds no identity of the worktree")?;
    let source: PathBuf =
        path_of_hex(field_of(text, SOURCE_FIELD).ok_or("it names no source worktree")?)
            .ok_or("its source worktree is not a path")?;
    let said: Vec<u8> = bytes_of_hex(
        field_of(text, REASON_FIELD).ok_or("it does not say what the copy ended with")?,
    )
    .ok_or("what it says the copy ended with is not bytes")?;
    let reason: String = String::from_utf8(said).map_err(not_text)?;
    let owner: Option<String> = match field_of(text, OWNER_FIELD) {
        Some(token) if !token.is_empty() => Some(token.to_string()),
        _ => None,
    };
    return Ok(UnfilledCopy {
        destination,
        destination_identity: identity,
        source,
        reason,
        owner,
    });
}

/// What:
///  The record's path for one journal:
/// the journal's identifier and this suffix.
/// Why:
///   A resumed copy renames its journal to its own owner token,
/// so a record named after the journal's whole name would be left behind
///  by every resumed attempt and reported as an unfilled worktree that is being filled;
/// the identifier alone stays one name for one copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unfilledPathOf = (journalPath) => join(dirname(journalPath), `${basename(journalPath, '.json').split('.')[0]}${UNFILLED_SUFFIX}`);
/// ```
pub fn unfilled_path_of(journal: &Path) -> PathBuf {
    let name: String = match journal.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let stem: &str = name.split('.').next().unwrap_or_default();
    return journal.with_file_name(format!("{stem}{UNFILLED_SUFFIX}"));
}
