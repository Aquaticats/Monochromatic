//! What:
//!  The shape of one conversion's journal,
//!  and the text it is written as.
//! Why:
//!  The journal is the only proof that the converting subvolume and the index lock are this command's,
//!  and it names the identity of the old root and of the subvolume it created,
//!  which is what every later phase checks before it reuses,
//!  exchanges,
//!  destroys or deletes anything.
//!  The text is a list of `name=value` lines in no fixed order,
//!  so a field is looked up by name and a journal of a shape this build does not know is one refusal,
//!  with nothing changed.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_convert_journal_writes.rs` writes the text into a file and reads it back,
//!  and `worktree_convert_journal.rs` holds the claim a conversion takes and the kernel lock an older
//!  release left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = journal.encode();
//! const journal = Journal.decode(text); // a refusal when the shape is unknown
//! ```

use crate::fs_beneath::ObjectId;
use crate::subvolume_error::MigrateError;

/// The name of the journal inside the worktree's administrative directory.
pub const JOURNAL_NAME: &str = "cli-git-convert";

/// The first line of every journal.
const JOURNAL_HEADER: &str = "cli-git-convert v1";

/// What:
///  The phase a conversion has reached,
/// in order.
/// Why:
///   A rerun reads it to skip the finished phases and to decide whether an abandon is still possible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'started' | 'creating' | 'synced' | 'exchanged' | 'refreshed' | 'removed';
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Phase {
    /// The journal exists;
    /// nothing else (Git's `index.lock` is taken only for the proving pass,
    /// and a release up to 0.5.20 took it
    ///  here,
    /// which a rerun accepts).
    Started,
    /// The converting subvolume is being created,
    /// or exists.
    Creating,
    /// The copy was made and proved equal to the original;
    /// provisional until the swap,
    /// because a writer may act after it.
    Synced,
    /// The path names the subvolume;
    /// the old tree sits at the converting name.
    Exchanged,
    /// The index stat fields describe the files of the subvolume.
    Refreshed,
    /// The old tree is gone;
    /// only the journal remains.
    Removed,
}

impl Phase {
    /// The name written to the journal.
    fn name(self) -> &'static str {
        return match self {
            Phase::Started => "started",
            Phase::Creating => "creating",
            Phase::Synced => "synced",
            Phase::Exchanged => "exchanged",
            Phase::Refreshed => "refreshed",
            Phase::Removed => "removed",
        };
    }

    /// The phase a journal line names.
    fn parse(text: &str) -> Option<Phase> {
        for phase in [
            Phase::Started,
            Phase::Creating,
            Phase::Synced,
            Phase::Exchanged,
            Phase::Refreshed,
            Phase::Removed,
        ] {
            if phase.name() == text {
                return Some(phase);
            }
        }
        return None;
    }
}

/// What:
///  The journal of one conversion,
/// kept in the worktree's administrative directory.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertJournal = { phase: Phase; token: string; path: string; oldIdentity: ObjectId; gitFile: Uint8Array; newIdentity?: ObjectId; syncedMs?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Journal {
    /// The phase reached.
    pub phase: Phase,
    /// The token the index lock holds.
    pub token: String,
    /// The worktree path as raw bytes.
    pub path: Vec<u8>,
    /// The identity of the old root:
    /// device,
    /// inode and,
    /// where the filesystem gives one,
    /// the birth time,
    /// which tells the recorded
    ///  directory from another that later got the same inode number.
    pub old_identity: ObjectId,
    /// The exact bytes of the `.git` file.
    pub git_file: Vec<u8>,
    /// The identity of the subvolume this command created,
    /// recorded right after the creation;
    /// absent until then,
    /// so a
    ///  directory at the converting name that the journal has no identity for is foreign.
    pub new_identity: Option<ObjectId>,
    /// When the proving pass that made the copy equal to the original finished,
    /// in milliseconds after the epoch;
    /// absent in
    ///  a journal written by an earlier release.
    /// The removal compares the contents of a file whose change time is at or
    ///  after this instant,
    /// because no other compared field can see such a change.
    pub synced_ms: Option<i64>,
}

/// What:
///  The error of a conversion journal whose text has a shape this command does not know.
/// Why:
///   Every field that is missing or unreadable is the same refusal,
/// and nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new JournalError('the conversion journal has a shape this command does not know; nothing was changed')
/// ```
fn unknown_shape() -> MigrateError {
    return MigrateError::Journal(String::from(
        "the conversion journal has a shape this command does not know; nothing was changed",
    ));
}

/// What:
///  The value of the first line `<name>=<value>` of a journal's text,
/// if any line has that name.
/// Why:
///   The journal is a list of `name=value` lines in no fixed order,
/// and a field is looked up by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').map((line) => line.startsWith(`${name}=`) ? line.slice(name.length + 1) : undefined).find((value) => value !== undefined)
/// ```
fn field_of(
    text: &str,
    name: &str,
) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(name)
            && let Some(value) = rest.strip_prefix('=')
        {
            return Some(String::from(value));
        }
    }
    return None;
}

/// What:
///  A field's digits read as a number,
/// or nothing when they are not one.
/// Why:
///   The old device and inode of a journal written before identities carried a birth time are plain decimal fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number.parseInt(digits, 10)
/// ```
fn number_from_digits<Number: std::str::FromStr>(digits: String) -> Option<Number> {
    return digits.parse().ok();
}

/// The clock reading the journal records:
/// milliseconds after the epoch,
/// and nothing before it.
pub fn now_milliseconds() -> i64 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX),
        Err(_before_epoch) => 0,
    };
}

/// The lowercase hexadecimal form of some bytes.
fn hex(bytes: &[u8]) -> String {
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(bytes);
}

/// The bytes a hexadecimal string names.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    // The first pair that is not hexadecimal ends the decoding with nothing, as collecting into an `Option` did.
    let mut bytes: Vec<u8> = Vec::new();
    let mut at: usize = 0;
    while at < text.len() {
        let pair: &str = text.get(at..at + 2)?;
        bytes.push(u8::from_str_radix(pair, 16).ok()?);
        at += 2;
    }
    return Some(bytes);
}

impl Journal {
    /// The journal text.
    pub(crate) fn encode(&self) -> String {
        return format!(
            "{JOURNAL_HEADER}\nphase={}\ntoken={}\npath={}\nold_identity={}\ngit_file={}\nnew_identity={}\nsynced_ms={}\n",
            self.phase.name(),
            self.token,
            hex(self.path.as_slice()),
            self.old_identity.encode(),
            hex(self.git_file.as_slice()),
            match self.new_identity {
                Some(identity) => identity.encode(),
                None => String::from("-"),
            },
            match self.synced_ms {
                Some(milliseconds) => milliseconds.to_string(),
                None => String::from("-"),
            }
        );
    }

    /// The journal a text describes.
    pub(crate) fn decode(text: &str) -> Result<Journal, MigrateError> {
        let mut lines = text.lines();
        if lines.next() != Some(JOURNAL_HEADER) {
            return Err(unknown_shape());
        }
        let new_identity: Option<ObjectId> = match field_of(text, "new_identity").as_deref() {
            None | Some("-") => None,
            Some(encoded) => Some(ObjectId::decode(encoded).ok_or_else(unknown_shape)?),
        };
        // A journal written before the birth time was recorded names the old root by device and inode alone.
        let old_identity: ObjectId = match field_of(text, "old_identity") {
            Some(encoded) => ObjectId::decode(encoded.as_str()).ok_or_else(unknown_shape)?,
            None => ObjectId {
                device: field_of(text, "old_device")
                    .and_then(number_from_digits)
                    .ok_or_else(unknown_shape)?,
                inode: field_of(text, "old_inode")
                    .and_then(number_from_digits)
                    .ok_or_else(unknown_shape)?,
                birth: None,
            },
        };
        return Ok(Journal {
            phase: Phase::parse(field_of(text, "phase").ok_or_else(unknown_shape)?.as_str())
                .ok_or_else(unknown_shape)?,
            token: field_of(text, "token").ok_or_else(unknown_shape)?,
            path: unhex(field_of(text, "path").ok_or_else(unknown_shape)?.as_str())
                .ok_or_else(unknown_shape)?,
            old_identity,
            git_file: unhex(
                field_of(text, "git_file")
                    .ok_or_else(unknown_shape)?
                    .as_str(),
            )
            .ok_or_else(unknown_shape)?,
            new_identity,
            // A journal written by an earlier release has no proof time; the removal then compares every file's contents.
            synced_ms: field_of(text, "synced_ms").and_then(number_from_digits),
        });
    }
}

/// Whether `found` is the object the journal recorded as `recorded`:
/// the same device and inode and,
/// when a birth time was
///  recorded,
/// the same birth time.
pub fn is_recorded_object(
    recorded: &ObjectId,
    found: &ObjectId,
) -> bool {
    return recorded.device == found.device
        && recorded.inode == found.inode
        && (recorded.birth.is_none() || recorded.birth == found.birth);
}
