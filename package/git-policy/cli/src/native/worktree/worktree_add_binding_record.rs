//! What:
//!  The record that carries an add's binding:
//! the binding itself,
//! the text it is written as,
//! what a reader makes of that text,
//! and the ignored-state copy the add still owes.
//! Why:
//!   A command killed between Git's registration and the copy leaves a worktree the next command must
//!  fill,
//! and the record is the only place the source and the selection survive the command that died,
//! so one writer spells every field and one reader takes them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = registeredRecordWith(binding, owed, child);
//! const recorded = decodeRecord(text); // { phase, destination } | undefined
//! ```

use crate::fs_beneath::ObjectId;
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// What an add bound itself to:
/// the two directories Git made and the pointer it wrote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    /// The destination,
    /// a canonical path.
    pub destination: PathBuf,
    /// The identity of the destination directory when Git returned.
    pub destination_id: ObjectId,
    /// The administrative directory,
    /// a canonical path directly below the repository's `worktrees`.
    pub admin: PathBuf,
    /// The identity of the administrative directory when Git returned.
    pub admin_id: ObjectId,
    /// The bytes of the `.git` file Git wrote in the destination.
    pub pointer: Vec<u8>,
}

/// What:
///  The value of the first line `<key>=<value>` of a record's text,
/// if any line has that key.
/// Why:
///   A binding record is a list of `key=value` lines,
/// and a field is looked up by key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').map((line) => line.startsWith(`${key}=`) ? line.slice(key.length + 1) : undefined).find((value) => value !== undefined)
/// ```
fn field_of<'a>(
    text: &'a str,
    key: &str,
) -> Option<&'a str> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(key)
            && let Some(value) = rest.strip_prefix('=')
        {
            return Some(value);
        }
    }
    return None;
}

/// Lower-case hexadecimal of bytes.
fn hex(bytes: &[u8]) -> String {
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(bytes);
}

/// The bytes of lower-case hexadecimal text.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    // The first pair that is not hexadecimal ends the decoding with nothing, as collecting into an `Option` did.
    let mut bytes: Vec<u8> = Vec::new();
    let mut start: usize = 0;
    while start < text.len() {
        let pair: &str = text.get(start..start + 2)?;
        bytes.push(u8::from_str_radix(pair, 16).ok()?);
        start += 2;
    }
    return Some(bytes);
}

/// The text of a path for a record:
/// hexadecimal,
/// so that a newline in a name cannot forge a field.
pub fn encode_path(path: &Path) -> String {
    return hex(path.as_os_str().as_bytes());
}

/// The path of a record's text.
pub fn decode_path(text: &str) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    return Some(PathBuf::from(std::ffi::OsString::from_vec(unhex(text)?)));
}

impl Binding {
    /// The record text of the binding.
    pub fn encode(&self) -> String {
        return format!(
            "destination={}\nphase=registered\ndestination_id={}\nadmin={}\nadmin_id={}\npointer={}\n",
            encode_path(self.destination.as_path()),
            self.destination_id.encode(),
            encode_path(self.admin.as_path()),
            self.admin_id.encode(),
            hex(self.pointer.as_slice()),
        );
    }
}

/// What:
///  The record text of an add that has been bound to the objects Git made,
/// keeping the ignored-state copy it still owes.
/// Why:
///   The binding is recorded as soon as Git returns,
/// which is before the copy runs,
/// so a record written with the binding alone loses the source and the selection a later command needs to make that copy;
/// the two lines therefore travel with every rewrite of the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = registeredRecordWith(binding, owed, child);
/// ```
pub fn registered_record_with(
    binding: &Binding,
    owed: Option<&OwedCopy>,
    child: Option<&crate::worktree_copy_owner::CopyOwner>,
) -> String {
    let mut text: String = binding.encode();
    if let Some(copy) = owed {
        text.push_str(format!("source={}\n", encode_path(copy.source.as_path())).as_str());
        text.push_str(format!("sourceSelection={}\n", copy.selection.word()).as_str());
    }
    if let Some(named) = child {
        text.push_str(format!("running={}\n", named.token()).as_str());
    }
    return text;
}

/// What a record says about an add.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Recorded {
    /// The add was planned and not bound:
    /// no object is known to be the add's.
    Planned(PathBuf),
    /// The add was bound to the objects Git made.
    Registered(Binding),
}

/// What:
///  Read the text of an add's record.
/// Why:
///   A record that does not parse in full is treated as planned only when its destination field parses;
/// anything else is
///  unreadable and authorizes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeRecord(text: string): Recorded | undefined;
/// ```
pub fn decode_record(text: &str) -> Option<Recorded> {
    let destination: PathBuf = decode_path(field_of(text, "destination")?)?;
    if field_of(text, "phase")? == "planned" {
        return Some(Recorded::Planned(destination));
    }
    if field_of(text, "phase")? != "registered" {
        return None;
    }
    return Some(Recorded::Registered(Binding {
        destination,
        destination_id: ObjectId::decode(field_of(text, "destination_id")?)?,
        admin: decode_path(field_of(text, "admin")?)?,
        admin_id: ObjectId::decode(field_of(text, "admin_id")?)?,
        pointer: unhex(field_of(text, "pointer")?)?,
    }));
}

/// What:
///  Which of the source's ignored paths the add's copy takes.
/// Why:
///   A main checkout hands over its ignored state under its leave-out rule,
/// and a linked worktree under the full listing;
///  a later command that makes the copy must know which of the two the add meant,
///  and that decision comes from the invoking identity,
/// which is gone by then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwedSelection = 'ignored' | 'main-checkout';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwedSelection {
    /// Every path Git's standard exclusion stack ignores,
    /// plus the ignored special entries.
    Ignored,
    /// The same listing,
    /// walked under the main checkout's leave-out rule.
    MainCheckout,
}

impl OwedSelection {
    /// The word this selection is written as in an add's record.
    fn word(self) -> &'static str {
        return match self {
            OwedSelection::Ignored => "ignored",
            OwedSelection::MainCheckout => "main-checkout",
        };
    }

    /// The selection a record's word names,
    /// or nothing when the word is not one this release writes.
    fn of_word(word: &str) -> Option<OwedSelection> {
        return match word {
            "ignored" => Some(OwedSelection::Ignored),
            "main-checkout" => Some(OwedSelection::MainCheckout),
            _ => None,
        };
    }
}

/// What:
///  The ignored-state copy one add owes:
/// the worktree it takes the state from,
/// and which of that worktree's ignored paths it takes.
/// Why:
///   A command killed after real Git registered its worktree and before the copy finished leaves a worktree that must be
/// filled,
/// and the next command can only fill it from the source the add was invoked in;
/// the record is where that source survives the command that died.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwedCopy = { source: string; selection: OwedSelection };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwedCopy {
    /// The worktree whose ignored state the add copies.
    pub source: PathBuf,
    /// Which of that worktree's ignored paths the copy takes.
    pub selection: OwedSelection,
}

/// The record text of a planned add.
pub fn planned_record(destination: &Path) -> String {
    return planned_record_with(destination, None, None);
}

/// What:
///  The record text of a planned add,
/// with the Git process it started and the ignored-state copy it owes when each is known.
/// Why:
///   Both facts must survive a kill,
/// and the record is rewritten when Git starts,
/// so one writer spells every field a reader takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plannedRecordWith({ destination, owed, child })
/// ```
pub fn planned_record_with(
    destination: &Path,
    owed: Option<&OwedCopy>,
    child: Option<&crate::worktree_copy_owner::CopyOwner>,
) -> String {
    let mut text: String = format!("destination={}\nphase=planned\n", encode_path(destination));
    if let Some(named) = child {
        text.push_str(format!("running={}\n", named.token()).as_str());
    }
    if let Some(copy) = owed {
        text.push_str(format!("source={}\n", encode_path(copy.source.as_path())).as_str());
        text.push_str(format!("sourceSelection={}\n", copy.selection.word()).as_str());
    }
    return text;
}

/// What:
///  The ignored-state copy a planned record says the add owes,
/// or nothing when it names none.
/// Why:
///   A record written before this field existed promises no copy a later command could make,
/// so it settles exactly as it
///  did;
/// only a record that names a source is finished from one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owed = owedCopyOf(text); // { source, selection } | undefined
/// ```
pub fn owed_copy_of(text: &str) -> Option<OwedCopy> {
    return Some(OwedCopy {
        source: decode_path(field_of(text, "source")?)?,
        selection: OwedSelection::of_word(field_of(text, "sourceSelection")?)?,
    });
}

/// What:
///  The Git process a planned record names,
/// or nothing when the record names none.
/// Why:
///   A record is written before Git starts,
/// and the command that starts Git writes the name once it has one;
/// a command killed between
///  the two names no process,
/// so the record's age is the only bound a reader has then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = CopyOwner.fromToken(fieldOf(text, 'running') ?? '');
/// ```
pub fn running_child_of(text: &str) -> Option<crate::worktree_copy_owner::CopyOwner> {
    return crate::worktree_copy_owner::CopyOwner::from_token(field_of(text, "running")?);
}
