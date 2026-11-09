//! What:
//!  The journal of one conversion of a worktree into a subvolume.
//! Why:
//!  The journal is the only proof that the converting subvolume and the index lock are this command's,
//! and it names the
//!  identity of the old root and of the subvolume it created,
//! which is what every later phase checks before it reuses,
//!  exchanges,
//! destroys or deletes anything.
//! The journal is written through exclusively created files and never replaces
//!  a file it did not make.
//! One command at a time owns a conversion through a numbered claim file
//!  (`single_owner_claim.rs`) taken before the journal is read,
//! so no kernel lock is held.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = await takeClaim({ directory: admin, prefix: CLAIM_PREFIX }); // busy when another command owns it
//! const journal = await Journal.load(admin);
//! await journal.save(admin); // exclusive temporary, then rename
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::ObjectId;
use crate::process_identity::process_birth_identity;
use crate::subvolume_error::MigrateError;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

/// The name of the journal inside the worktree's administrative directory.
pub const JOURNAL_NAME: &str = "cli-git-convert";

/// The name of the kernel lock file that releases up to 0.5.20 left in the administrative directory;
/// this release only probes for
///  it and removes it when nothing holds it.
pub const LOCK_NAME: &str = "cli-git-convert.lock";

/// The prefix of the numbered claim files that give one command at a time the conversion of a worktree.
pub const CLAIM_PREFIX: &str = "cli-git-convert.owner";

/// The prefix of the temporary file each journal write makes,
/// followed by the number of the process that made it and the clock reading.
pub const TEMPORARY_PREFIX: &str = "cli-git-convert.tmp.";

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeConvertJournal";

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
        for phase in [Phase::Started, Phase::Creating, Phase::Synced, Phase::Exchanged, Phase::Refreshed, Phase::Removed] {
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
    return MigrateError::Journal(String::from("the conversion journal has a shape this command does not know; nothing was changed"));
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
fn field_of(text: &str, name: &str) -> Option<String> {
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

/// What:
///  Make the directory's own entry durable by syncing a handle on it.
/// Why:
///   A rename or a link is only as durable as the directory that holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await (await open(admin)).sync()
/// ```
fn sync_directory_of(admin: &Path) -> std::io::Result<()> {
    return crate::subvolume_file_write_steps::sync_directory_to_disk(admin);
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

/// What:
///  An input or output outcome with its failure turned into a conversion error that names the action and the path.
/// Why:
///   Many steps of the conversion report the same way,
/// and a named function replaces a closure at each.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ioStep<T>(outcome: () => T, action: string, path: string): T { try { return outcome(); } catch (error) { throw migrateIo(action, path, error); } }
/// ```
pub fn io_step<T>(outcome: std::io::Result<T>, action: &str, path: &Path) -> Result<T, MigrateError> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(MigrateError::io(action, path, &error)),
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

/// A name for a temporary file that no other file has:
/// this process and the clock make it,
/// and the creation is exclusive.
fn temporary_name() -> String {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_before_epoch) => 0,
    };
    return format!("{TEMPORARY_PREFIX}{}.{nanos}", std::process::id());
}

/// What:
///  Whether one field of a temporary file's name is one or more ASCII digits and nothing else.
/// Why:
///  The process number and the clock reading are written as decimal text,
/// and a name with anything else there is no temporary file of this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isDecimalField(field: string): boolean;
/// ```
fn is_decimal_field(field: &str) -> bool {
    if field.is_empty() {
        return false;
    }
    for byte in field.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// What:
///  The process number in the name of a temporary journal file,
///  nothing for every other name.
/// Why:
///  Only the files a journal write makes carry it,
/// and the sweep may remove nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function makerOf(name: string): number | undefined;
/// ```
fn maker_of(name: &str) -> Option<u32> {
    // `?` returns `None` from this function for a name without the prefix.
    let rest: &str = name.strip_prefix(TEMPORARY_PREFIX)?;
    let (pid, nanos): (&str, &str) = rest.split_once('.')?;
    // Both fields are decimal digits and nothing else, as `temporary_name` writes them.
    if !is_decimal_field(pid) || !is_decimal_field(nanos) {
        return None;
    }
    return pid.parse().ok();
}

/// What:
///  Removes the temporary files of journal writes whose maker is proven gone,
///  and says how many it removed.
/// Why:
///  Each write makes `cli-git-convert.tmp.<pid>.<nanos>` and removes it only when its own rename or link fails,
///  so a conversion killed in between leaves one that nothing else cleans.
///  A file stays unless its maker is gone:
/// the name carries only a process number,
/// so a number that still runs,
///  a record this host refuses to read,
/// or a number a later process got keeps the file,
///  which costs one
///  file rather than someone else's write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sweepTemporaryFiles(admin: string): number;
/// ```
pub fn sweep_temporary_files(admin: &Path) -> usize {
    // Nothing to sweep when the directory cannot be read; the journal write that follows reports the real failure.
    let Ok(entries) = std::fs::read_dir(admin) else { return 0 };
    let mut removed: usize = 0;
    for entry in entries.flatten() {
        // A name that is not text under a home this command wrote is no temporary file of its own.
        let Ok(name) = entry.file_name().into_string() else { continue };
        let Some(pid) = maker_of(name.as_str()) else { continue };
        // `Ok(None)` is the one answer that proves no process of this number runs; anything else proves nothing.
        if !matches!(process_birth_identity(i64::from(pid)), Ok(None)) {
            continue;
        }
        if std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    return removed;
}

impl Journal {
    /// The journal text.
    fn encode(&self) -> String {
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
    fn decode(text: &str) -> Result<Journal, MigrateError> {
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
                device: field_of(text, "old_device").and_then(number_from_digits).ok_or_else(unknown_shape)?,
                inode: field_of(text, "old_inode").and_then(number_from_digits).ok_or_else(unknown_shape)?,
                birth: None,
            },
        };
        return Ok(Journal {
            phase: Phase::parse(field_of(text, "phase").ok_or_else(unknown_shape)?.as_str()).ok_or_else(unknown_shape)?,
            token: field_of(text, "token").ok_or_else(unknown_shape)?,
            path: unhex(field_of(text, "path").ok_or_else(unknown_shape)?.as_str()).ok_or_else(unknown_shape)?,
            old_identity,
            git_file: unhex(field_of(text, "git_file").ok_or_else(unknown_shape)?.as_str()).ok_or_else(unknown_shape)?,
            new_identity,
            // A journal written by an earlier release has no proof time; the removal then compares every file's contents.
            synced_ms: field_of(text, "synced_ms").and_then(number_from_digits),
        });
    }

    /// Write the journal into a file made for it alone:
    /// created exclusively,
    /// so nothing that is there is truncated or followed.
    fn write_temporary(&self, admin: &Path) -> std::io::Result<PathBuf> {
        // A conversion killed between its write and its rename leaves the temporary file behind; every write sweeps the
        // leftovers of its own worktree first, and only those whose maker is gone.
        let swept: usize = sweep_temporary_files(admin);
        if swept > 0 {
            debug(TAG, format!("removed {swept} temporary journal file(s) left by a conversion that is gone").as_str());
        }
        let temporary: PathBuf = admin.join(temporary_name());
        let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
        let written: std::io::Result<()> = crate::subvolume_file_write_steps::write_all_then_sync(&mut file, self.encode().as_bytes());
        if let Err(error) = written {
            let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
            return Err(error);
        }
        return Ok(temporary);
    }

    /// Replace the journal with this one:
    /// written beside it,
    /// synchronized,
    /// renamed into place.
    pub fn save(&self, admin: &Path) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        return io_step(self.write_and_publish(admin, target.as_path()), "write the conversion journal", target.as_path());
    }

    /// Write the journal to a temporary file,
    /// rename it over `target` and make the directory durable.
    fn write_and_publish(&self, admin: &Path, target: &Path) -> std::io::Result<()> {
        let temporary: PathBuf = self.write_temporary(admin)?;
        if let Err(error) = std::fs::rename(&temporary, target) {
            let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
            return Err(error);
        }
        return sync_directory_of(admin);
    }

    /// Make this the journal of a conversion that has none:
    /// it fails when any journal exists,
    /// so a journal another command
    ///  made is never replaced.
    pub fn create_new(&self, admin: &Path) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        return io_step(self.write_and_link(admin, target.as_path()), "create the conversion journal", target.as_path());
    }

    /// Write the journal to a temporary file,
    /// link it at `target` only if nothing is there and make the directory durable.
    fn write_and_link(&self, admin: &Path, target: &Path) -> std::io::Result<()> {
        let temporary: PathBuf = self.write_temporary(admin)?;
        let linked: std::io::Result<()> = std::fs::hard_link(&temporary, target);
        let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
        linked?;
        return sync_directory_of(admin);
    }

    /// The journal of an administrative directory,
    /// if one exists.
    pub fn load(admin: &Path) -> Result<Option<Journal>, MigrateError> {
        let path: PathBuf = admin.join(JOURNAL_NAME);
        match crate::plain_file::read_plain_file(path.as_path(), crate::plain_file::SMALL_FILE_LIMIT) {
            Ok(bytes) => return Journal::decode(String::from_utf8_lossy(bytes.as_slice()).as_ref()).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(MigrateError::io("read the conversion journal", path.as_path(), &error)),
        }
    }
}

/// Whether `found` is the object the journal recorded as `recorded`:
/// the same device and inode and,
/// when a birth time was
///  recorded,
/// the same birth time.
pub fn is_recorded_object(recorded: &ObjectId, found: &ObjectId) -> bool {
    return recorded.device == found.device && recorded.inode == found.inode && (recorded.birth.is_none() || recorded.birth == found.birth);
}

/// What:
///  Whether a command of an older release still holds the conversion's kernel lock on `admin`,
/// retiring the lock file when
///  none does.
/// Why:
///   Releases up to 0.5.20 serialized conversions with an advisory kernel lock on `cli-git-convert.lock` and left the file
///  in place.
/// This release serializes with a claim file instead and takes no kernel lock,
/// but an older command may still be
///  converting the worktree,
/// and two commands must never convert one worktree at once.
/// The probe asks without waiting and
///  gives the lock straight back;
/// a file nobody holds is removed,
/// so the leftover disappears after the upgrade.
/// A file that
///  is absent costs nothing,
/// and a file that is not a plain file is left where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heldByOlderRelease = await retireLegacyLock(admin); // false when absent, free (and now removed) or not a plain file
/// ```
pub fn legacy_lock_is_held(admin: &Path) -> Result<bool, MigrateError> {
    let path: PathBuf = admin.join(LOCK_NAME);
    let file: std::fs::File = match std::fs::OpenOptions::new().read(true).write(true).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC).open(&path) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(MigrateError::io("open the conversion lock an older release left", path.as_path(), &error)),
    };
    let plain: bool = io_step(file.metadata(), "examine the conversion lock an older release left", path.as_path())?.file_type().is_file();
    if !plain {
        return Ok(false);
    }
    // SAFETY: `flock` takes an open descriptor, which `file` keeps open for the call.
    let result: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::WouldBlock {
            return Ok(true);
        }
        return Err(MigrateError::io("probe the conversion lock an older release left", path.as_path(), &error));
    }
    // Removed while the probe still holds it, so no older command can take the name between the probe and the removal.
    return match std::fs::remove_file(&path) {
        Ok(()) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(MigrateError::io("remove the conversion lock an older release left", path.as_path(), &error)),
    };
}
