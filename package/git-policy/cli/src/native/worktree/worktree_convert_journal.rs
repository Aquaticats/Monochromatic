//! What:
//!  The journal of one conversion of a worktree into a subvolume.
//! Why:
//!  The journal is the only proof that the converting subvolume and the index lock are this command's, and it names the
//!  identity of the old root and of the subvolume it created, which is what every later phase checks before it reuses,
//!  exchanges, destroys or deletes anything. The journal is written through exclusively created files and never replaces
//!  a file it did not make. One command at a time owns a conversion through a numbered claim file
//!  (`single_owner_claim.rs`) taken before the journal is read, so no kernel lock is held.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = await takeClaim({ directory: admin, prefix: CLAIM_PREFIX }); // busy when another command owns it
//! const journal = await Journal.load(admin);
//! await journal.save(admin); // exclusive temporary, then rename
//! ```

use crate::fs_beneath::ObjectId;
use crate::subvolume_error::MigrateError;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

/// The name of the journal inside the worktree's administrative directory.
pub const JOURNAL_NAME: &str = "cli-git-convert";

/// The name of the kernel lock file that releases up to 0.5.20 left in the administrative directory; this release only probes for
///  it and removes it when nothing holds it.
pub const LOCK_NAME: &str = "cli-git-convert.lock";

/// The prefix of the numbered claim files that give one command at a time the conversion of a worktree.
pub const CLAIM_PREFIX: &str = "cli-git-convert.owner";

/// The most bytes of a journal or a small registration file that are read.
const SMALL_FILE_LIMIT: u64 = 1 << 20;

/// The first line of every journal.
const JOURNAL_HEADER: &str = "cli-git-convert v1";

/// What:
///  The phase a conversion has reached, in order.
/// Why:
///   A rerun reads it to skip the finished phases and to decide whether an abandon is still possible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'started' | 'creating' | 'synced' | 'exchanged' | 'refreshed' | 'removed';
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Phase {
    /// The journal exists; nothing else (Git's `index.lock` is taken only for the proving pass, and a release up to 0.5.20 took it
    ///  here, which a rerun accepts).
    Started,
    /// The converting subvolume is being created, or exists.
    Creating,
    /// The copy was made and proved equal to the original; provisional until the swap, because a writer may act after it.
    Synced,
    /// The path names the subvolume; the old tree sits at the converting name.
    Exchanged,
    /// The index stat fields describe the files of the subvolume.
    Refreshed,
    /// The old tree is gone; only the journal remains.
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
        return [Phase::Started, Phase::Creating, Phase::Synced, Phase::Exchanged, Phase::Refreshed, Phase::Removed]
            .into_iter()
            .find(|phase: &Phase| { return phase.name() == text; });
    }
}

/// What:
///  The journal of one conversion, kept in the worktree's administrative directory.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertJournal = { phase: Phase; token: string; path: string; oldIdentity: ObjectId; gitFile: Uint8Array; newIdentity?: ObjectId };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Journal {
    /// The phase reached.
    pub phase: Phase,
    /// The token the index lock holds.
    pub token: String,
    /// The worktree path as raw bytes.
    pub path: Vec<u8>,
    /// The identity of the old root: device, inode and, where the filesystem gives one, the birth time, which tells the recorded
    ///  directory from another that later got the same inode number.
    pub old_identity: ObjectId,
    /// The exact bytes of the `.git` file.
    pub git_file: Vec<u8>,
    /// The identity of the subvolume this command created, recorded right after the creation; absent until then, so a
    ///  directory at the converting name that the journal has no identity for is foreign.
    pub new_identity: Option<ObjectId>,
}

/// The lowercase hexadecimal form of some bytes.
fn hex(bytes: &[u8]) -> String {
    return bytes.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect();
}

/// The bytes a hexadecimal string names.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    return (0..text.len()).step_by(2).map(|at: usize| { return u8::from_str_radix(text.get(at..at + 2)?, 16).ok(); }).collect();
}

/// A name for a temporary file that no other file has: this process and the clock make it, and the creation is exclusive.
fn temporary_name() -> String {
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed: std::time::Duration| { return elapsed.as_nanos(); });
    return format!("{JOURNAL_NAME}.tmp.{}.{nanos}", std::process::id());
}

impl Journal {
    /// The journal text.
    fn encode(&self) -> String {
        return format!(
            "{JOURNAL_HEADER}\nphase={}\ntoken={}\npath={}\nold_identity={}\ngit_file={}\nnew_identity={}\n",
            self.phase.name(),
            self.token,
            hex(self.path.as_slice()),
            self.old_identity.encode(),
            hex(self.git_file.as_slice()),
            self.new_identity.map_or_else(|| { return String::from("-"); }, |identity: ObjectId| { return identity.encode(); })
        );
    }

    /// The journal a text describes.
    fn decode(text: &str) -> Result<Journal, MigrateError> {
        let bad = || { return MigrateError::Journal(String::from("the conversion journal has a shape this command does not know; nothing was changed")); };
        let mut lines = text.lines();
        if lines.next() != Some(JOURNAL_HEADER) {
            return Err(bad());
        }
        let field = |name: &str| -> Option<String> { return text.lines().find_map(|line: &str| { return line.strip_prefix(name).and_then(|rest: &str| { return rest.strip_prefix('='); }).map(String::from); }); };
        let new_identity: Option<ObjectId> = match field("new_identity").as_deref() {
            None | Some("-") => None,
            Some(encoded) => Some(ObjectId::decode(encoded).ok_or_else(bad)?),
        };
        // A journal written before the birth time was recorded names the old root by device and inode alone.
        let old_identity: ObjectId = match field("old_identity") {
            Some(encoded) => ObjectId::decode(encoded.as_str()).ok_or_else(bad)?,
            None => ObjectId {
                device: field("old_device").and_then(|digits: String| { return digits.parse().ok(); }).ok_or_else(bad)?,
                inode: field("old_inode").and_then(|digits: String| { return digits.parse().ok(); }).ok_or_else(bad)?,
                birth: None,
            },
        };
        return Ok(Journal {
            phase: Phase::parse(field("phase").ok_or_else(bad)?.as_str()).ok_or_else(bad)?,
            token: field("token").ok_or_else(bad)?,
            path: unhex(field("path").ok_or_else(bad)?.as_str()).ok_or_else(bad)?,
            old_identity,
            git_file: unhex(field("git_file").ok_or_else(bad)?.as_str()).ok_or_else(bad)?,
            new_identity,
        });
    }

    /// Write the journal into a file made for it alone: created exclusively, so nothing that is there is truncated or followed.
    fn write_temporary(&self, admin: &Path) -> std::io::Result<PathBuf> {
        let temporary: PathBuf = admin.join(temporary_name());
        let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
        let written: std::io::Result<()> = file.write_all(self.encode().as_bytes()).and_then(|()| { return file.sync_all(); });
        if let Err(error) = written {
            let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
            return Err(error);
        }
        return Ok(temporary);
    }

    /// Replace the journal with this one: written beside it, synchronized, renamed into place.
    pub fn save(&self, admin: &Path) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        let written: std::io::Result<()> = (|| {
            let temporary: PathBuf = self.write_temporary(admin)?;
            if let Err(error) = std::fs::rename(&temporary, &target) {
                let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
                return Err(error);
            }
            return std::fs::File::open(admin).and_then(|directory: std::fs::File| { return directory.sync_all(); });
        })();
        return written.map_err(|error: std::io::Error| { return MigrateError::io("write the conversion journal", target.as_path(), &error); });
    }

    /// Make this the journal of a conversion that has none: it fails when any journal exists, so a journal another command
    ///  made is never replaced.
    pub fn create_new(&self, admin: &Path) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        let written: std::io::Result<()> = (|| {
            let temporary: PathBuf = self.write_temporary(admin)?;
            let linked: std::io::Result<()> = std::fs::hard_link(&temporary, &target);
            let _removed: std::io::Result<()> = std::fs::remove_file(&temporary);
            linked?;
            return std::fs::File::open(admin).and_then(|directory: std::fs::File| { return directory.sync_all(); });
        })();
        return written.map_err(|error: std::io::Error| { return MigrateError::io("create the conversion journal", target.as_path(), &error); });
    }

    /// The journal of an administrative directory, if one exists.
    pub fn load(admin: &Path) -> Result<Option<Journal>, MigrateError> {
        let path: PathBuf = admin.join(JOURNAL_NAME);
        match read_plain_file(path.as_path(), SMALL_FILE_LIMIT) {
            Ok(bytes) => return Journal::decode(String::from_utf8_lossy(bytes.as_slice()).as_ref()).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(MigrateError::io("read the conversion journal", path.as_path(), &error)),
        }
    }
}

/// What:
///  The bytes of the plain file at `path`, opened without blocking and without following a link, refused when it is anything
///  else or larger than `limit`.
/// Why:
///   Opening a named pipe for reading waits for a writer, and a command that waits while it owns the conversion blocks every
///  other command that wants it; the type is read from the open handle, and the read is bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPlainFile(path: string, limit: number): Buffer; // throws
/// ```
pub fn read_plain_file(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "it is not a plain file (a named pipe, a device or a directory)"));
    }
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is larger than this command reads"));
    }
    return Ok(bytes);
}

/// The text of the plain file at `path`, read like `read_plain_file`.
pub fn read_plain_text(path: &Path) -> std::io::Result<String> {
    return String::from_utf8(read_plain_file(path, SMALL_FILE_LIMIT)?).map_err(|_error| { return std::io::Error::new(std::io::ErrorKind::InvalidData, "it is not text"); });
}

/// Whether `found` is the object the journal recorded as `recorded`: the same device and inode and, when a birth time was
///  recorded, the same birth time.
pub fn is_recorded_object(recorded: &ObjectId, found: &ObjectId) -> bool {
    return recorded.device == found.device && recorded.inode == found.inode && (recorded.birth.is_none() || recorded.birth == found.birth);
}

/// What:
///  Whether a command of an older release still holds the conversion's kernel lock on `admin`, retiring the lock file when
///  none does.
/// Why:
///   Releases up to 0.5.20 serialized conversions with an advisory kernel lock on `cli-git-convert.lock` and left the file
///  in place. This release serializes with a claim file instead and takes no kernel lock, but an older command may still be
///  converting the worktree, and two commands must never convert one worktree at once. The probe asks without waiting and
///  gives the lock straight back; a file nobody holds is removed, so the leftover disappears after the upgrade. A file that
///  is absent costs nothing, and a file that is not a plain file is left where it is.
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
    let plain: bool = file.metadata().map_err(|error: std::io::Error| { return MigrateError::io("examine the conversion lock an older release left", path.as_path(), &error); })?.file_type().is_file();
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
