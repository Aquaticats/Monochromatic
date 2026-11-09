//! What:
//!  The right of one running command to work on one item (a worktree being converted,
//! a checkout being migrated),
//! held
//!  as a numbered claim file that exactly one command creates and that names its owner by process birth.
//! Why:
//!  Two commands that both pass the preflight would both write the journal,
//! share a token on a resume and exchange the
//!  directories back,
//! so one command must own the item for as long as it works.
//! The claim is lock-free:
//! a claim file is
//!  made whole in a private temporary file and published with one exclusive `link` under the next generation number,
//! so
//!  the kernel lets exactly one command create generation `n + 1`;
//! no lock is held,
//! no command ever waits,
//! and a command
//!  that finds a live owner stops with the owner's process id.
//! The newest generation is the claim.
//! A newer generation
//!  replaces an older one only when the older one records a released claim or an owner whose process no longer exists
//!  with that birth (the PID alone is not trusted,
//! because it is reused),
//! so a killed command leaves nothing to clean up
//!  by hand:
//! the next command takes the next generation.
//! The newest generation file is never removed,
//! because removing
//!  it would let a command that read an older generation create the same number again;
//! a released claim is rewritten in
//!  place by renaming a one-line record over it.
//! Older generations are removed by the command that took the newer one.
//!  An owner in another PID namespace cannot be probed;
//! it counts as alive until its claim file is older than
//!  `FOREIGN_OWNER_PATIENCE`,
//! as every owner record of the wrapper does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await takeClaim({ directory, prefix }); // { held: claim } or { busy: { pid, path } }
//! // ... work on the item ...
//! await claim.release();
//! ```

use crate::pid_namespace::{ForeignStanding, current_pid_namespace, foreign_owner_standing};
use crate::process_identity::{current_birth_identity, process_birth_identity};
use crate::subvolume_error::MigrateError;
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The first line of every claim file.
const CLAIM_HEADER: &str = "monochromatic-git-policy-cli owner-claim v1";

/// The line of a released claim.
const RELEASED_LINE: &str = "released";

/// The most rounds one `take_claim` call makes.
/// Each round after the first follows the progress of another command (it
///  created the generation this one wanted,
/// or a newer one),
/// so more rounds than this mean claimants keep arriving faster
///  than this command can look;
/// it then reports that instead of looping.
const MOST_ROUNDS: u32 = 64;

/// Distinguishes the temporary files of claims taken by one process at the same instant.
static TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What:
///  A claim this process holds:
/// the right to work on one item.
/// Why:
///   Dropping it releases the claim,
/// so every way out of the command,
/// an early error included,
/// gives the item back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerClaim = { path: string; generation: number; release(): Promise<void> };
/// ```
#[derive(Debug)]
pub struct OwnerClaim {
    /// The claim file this process created.
    path: PathBuf,
    /// The directory that holds the claim files of the item.
    directory: PathBuf,
    /// Whether the claim was released already.
    released: bool,
}

/// What:
///  The live owner that stopped a claim.
/// Why:
///   The refusal names the process and the file,
/// so the person can see which command runs and where its claim is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BusyOwner = { pid: number; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BusyOwner {
    /// The owner's process id.
    pub pid: i64,
    /// The claim file that records the owner.
    pub path: PathBuf,
}

/// What:
///  How a claim attempt ended.
/// Why:
///   A busy item is a normal answer (the other command is working),
/// not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimOutcome = { kind: 'held'; claim: OwnerClaim } | { kind: 'busy'; owner: BusyOwner };
/// ```
#[derive(Debug)]
pub enum ClaimOutcome {
    /// This process owns the item now.
    Held(OwnerClaim),
    /// A live process owns the item.
    Busy(BusyOwner),
}

/// What:
///  What a claim file says,
/// as [`decode`] reads it.
/// Why:
///   A claim decides whether a command may work on one item,
/// so the record it gives is the input of every liveness decision.
/// The record is public,
/// and so is [`decode`],
/// so the fuzz sidecar
///  (`package/git-policy/cli.fuzz/src/owner_claim_record.rs`) reads generated claim text through the same decoder a
///  command does,
/// instead of a copy of its rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimRecord {
    /// The owner gave the item back.
    Released,
    /// A process owns the item:
    /// its id,
    /// birth identity and PID namespace (`-` when unknown).
    Owned {
        /// The owner's process id,
        /// which a claim names no lower than one.
        pid: i64,
        /// The owner's birth identity,
        /// which a claim names non-empty.
        birth: String,
        /// The owner's PID namespace,
        /// absent when its own namespace is the reader's or cannot be read.
        namespace: Option<String>,
    },
}

/// The text of a claim file.
fn encode(record: &ClaimRecord) -> String {
    return match record {
        ClaimRecord::Released => format!("{CLAIM_HEADER}\n{RELEASED_LINE}\n"),
        ClaimRecord::Owned { pid, birth, namespace } => format!("{CLAIM_HEADER}\npid={pid}\nbirth={birth}\nnamespace={}\n", namespace.as_deref().unwrap_or("-")),
    };
}

/// What:
///  The record a claim file's text holds,
/// or nothing when the text is not a claim of this command.
/// Why:
///   The two folder spellings,
/// the released line,
/// the three owner fields and every refusal between them are the whole of what
///  this command reads about an owner,
/// so a text that reads differently here is a liveness decision from what no command wrote.
/// Public so the fuzz sidecar reads generated claims through it.
pub fn decode(text: &str) -> Option<ClaimRecord> {
    let mut lines = text.lines();
    // The files are named as before the rename, so a claim of the earlier release is found, and read in its own spelling.
    let earlier_header: String = CLAIM_HEADER.replacen(DATA_FOLDER.current, DATA_FOLDER.earlier, 1);
    let first: &str = lines.next()?;
    if first != CLAIM_HEADER && first != earlier_header {
        return None;
    }
    let second: &str = lines.next()?;
    if second == RELEASED_LINE {
        return Some(ClaimRecord::Released);
    }
    let pid: i64 = second.strip_prefix("pid=")?.parse().ok()?;
    if pid <= 0 {
        return None;
    }
    let birth: String = lines.next()?.strip_prefix("birth=")?.to_string();
    let namespace: String = lines.next()?.strip_prefix("namespace=")?.to_string();
    if birth.is_empty() {
        return None;
    }
    return Some(ClaimRecord::Owned { pid, birth, namespace: if namespace == "-" { None } else { Some(namespace) } });
}

/// The generation a directory entry names under `prefix`,
/// or nothing for any other name.
fn generation_of(name: &OsStr, prefix: &OsStr) -> Option<u64> {
    let digits: &[u8] = name.as_bytes().strip_prefix(prefix.as_bytes())?.strip_prefix(b".")?;
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    return std::str::from_utf8(digits).ok()?.parse().ok();
}

/// Every generation of `prefix` in `directory`,
/// ascending.
fn generations(directory: &Path, prefix: &OsStr) -> Result<Vec<u64>, MigrateError> {
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) => return Err(MigrateError::io("list the claims in", directory, &error)),
    };
    let mut found: Vec<u64> = Vec::new();
    for entry in entries.flatten() {
        if let Some(generation) = generation_of(entry.file_name().as_os_str(), prefix) {
            found.push(generation);
        }
    }
    found.sort_unstable();
    return Ok(found);
}

/// The path of one generation.
fn path_of(directory: &Path, prefix: &OsStr, generation: u64) -> PathBuf {
    let mut name: OsString = prefix.to_os_string();
    name.push(format!(".{generation}"));
    return directory.join(name);
}

/// Write `record` into a new private file beside the claims,
/// synchronized,
/// and return its path.
fn write_private(directory: &Path, record: &ClaimRecord) -> std::io::Result<PathBuf> {
    let unique: u64 = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };
    let path: PathBuf = directory.join(format!(".cli-git-claim.tmp.{}.{nanos}.{unique}", std::process::id()));
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path)?;
    // The synchronization runs only when the write succeeded, as `and_then` did.
    let mut written: std::io::Result<()> = file.write_all(encode(record).as_bytes());
    if written.is_ok() {
        written = file.sync_all();
    }
    if let Err(error) = written {
        let _removed: std::io::Result<()> = std::fs::remove_file(&path);
        return Err(error);
    }
    return Ok(path);
}

/// Make the directory's new entry durable.
fn sync_directory(directory: &Path) -> std::io::Result<()> {
    let handle: std::fs::File = std::fs::File::open(directory)?;
    return handle.sync_all();
}

/// Publish `record` as `target` without replacing anything:
/// `AlreadyExists` when the generation exists.
fn publish_new(directory: &Path, target: &Path, record: &ClaimRecord) -> std::io::Result<()> {
    let private: PathBuf = write_private(directory, record)?;
    let linked: std::io::Result<()> = std::fs::hard_link(&private, target);
    let _removed: std::io::Result<()> = std::fs::remove_file(&private);
    linked?;
    return sync_directory(directory);
}

/// The claim file's record,
/// or nothing when the file is gone,
/// or an error when it is not a claim of this command.
fn read_record(path: &Path) -> Result<Option<ClaimRecord>, MigrateError> {
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file(path, SMALL_RECORD_LIMIT) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(MigrateError::io("read the claim", path, &error)),
    };
    return match decode(String::from_utf8_lossy(bytes.as_slice()).as_ref()) {
        Some(record) => Ok(Some(record)),
        None => Err(MigrateError::Journal(format!(
            "{} is not a claim file this command wrote, so it cannot tell whether another command is working here; nothing was changed. Look at the file: if no monochromatic-git-policy-cli command is running on this item, remove it and run the command again",
            path.display()
        ))),
    };
}

/// The most bytes of a claim file that are read.
const SMALL_RECORD_LIMIT: u64 = 4096;

/// The process id of the owner a record names when that process is alive,
/// nothing when the claim is released or its owner is
///  gone,
/// and an error when that cannot be decided.
fn live_owner(record: &ClaimRecord, path: &Path) -> Result<Option<i64>, MigrateError> {
    let ClaimRecord::Owned { pid, birth, namespace } = record else { return Ok(None) };
    match foreign_owner_standing(namespace.as_deref(), path) {
        Some(ForeignStanding::Standing) => return Ok(Some(*pid)),
        // The claim of another namespace that its owner removed is a finished owner's: nothing is held, so nothing is taken
        //  from a live owner either (cross-vendor review of 2026-11-09, finding 3).
        Some(ForeignStanding::Finished) => return Ok(None),
        // Owner, 2026-11-04, "Refuse and name it (Recommended)": an old claim of another namespace is neither taken nor trusted.
        Some(ForeignStanding::Refused(refused)) => return Err(MigrateError::Journal(refused.describe(*pid))),
        None => {}
    }
    return match process_birth_identity(*pid) {
        Ok(Some(current)) => Ok((&current == birth).then_some(*pid)),
        Ok(None) => Ok(None),
        Err(error) => Err(MigrateError::Journal(format!(
            "{} names process {pid}, and whether it still runs cannot be decided ({error}); nothing was changed. If it does not, remove the file and run the command again",
            path.display()
        ))),
    };
}

/// This process's claim record.
fn own_record() -> Result<ClaimRecord, MigrateError> {
    let birth: String = match current_birth_identity() {
        Ok(identity) => identity,
        Err(error) => return Err(MigrateError::Journal(format!("this process cannot be identified for a claim ({error}); nothing was changed"))),
    };
    return Ok(ClaimRecord::Owned { pid: i64::from(std::process::id()), birth, namespace: current_pid_namespace() });
}

/// What:
///  Take the claim on the item whose claim files are `<prefix>.<generation>` in `directory`,
/// or report the live owner.
/// Why:
///   See the module description.
/// The call never waits:
/// it reads the newest generation,
/// creates the next one exclusively,
///  confirms that no newer one appeared,
/// and removes the older ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await takeClaim({ directory, prefix });
/// ```
pub fn take_claim(directory: &Path, prefix: &OsStr) -> Result<ClaimOutcome, MigrateError> {
    return take_claim_seamed(directory, prefix, &mut do_nothing);
}

/// The call `take_claim` makes between the listing and the creation:
/// none.
fn do_nothing() {}

/// `take_claim` with a call between the listing of the generations and the creation of the next one,
/// where a control may let
///  another command act.
pub fn take_claim_seamed(directory: &Path, prefix: &OsStr, after_listing: &mut dyn FnMut()) -> Result<ClaimOutcome, MigrateError> {
    let record: ClaimRecord = own_record()?;
    for _round in 0..MOST_ROUNDS {
        let existing: Vec<u64> = generations(directory, prefix)?;
        after_listing();
        let target: u64 = match existing.last() {
            None => 1,
            Some(newest) => {
                let newest_path: PathBuf = path_of(directory, prefix, *newest);
                let Some(found) = read_record(newest_path.as_path())? else { continue };
                if let Some(pid) = live_owner(&found, newest_path.as_path())? {
                    return Ok(ClaimOutcome::Busy(BusyOwner { pid, path: newest_path }));
                }
                newest + 1
            }
        };
        let path: PathBuf = path_of(directory, prefix, target);
        match publish_new(directory, path.as_path(), &record) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(MigrateError::io("create the claim", path.as_path(), &error)),
        }
        // A command that read an older generation may have published one below the newest after the others were removed;
        // a newer generation than this one means this claim lost.
        if let Some(newest) = generations(directory, prefix)?.last()
            && *newest > target
        {
            let _withdrawn: std::io::Result<()> = std::fs::remove_file(&path);
            continue;
        }
        for older in &existing {
            if *older < target {
                let _removed: std::io::Result<()> = std::fs::remove_file(path_of(directory, prefix, *older));
            }
        }
        return Ok(ClaimOutcome::Held(OwnerClaim { path, directory: directory.to_path_buf(), released: false }));
    }
    return Err(MigrateError::Journal(format!("other commands kept taking the claim in {} while this one looked; nothing was changed, run the command again", directory.display())));
}

/// What:
///  The live owner of the item whose claim files are `<prefix>.<generation>` in `directory`,
/// if any;
/// nothing is created.
/// Why:
///   A command that works on a different item inside the same checkout (a migration of the checkout,
/// a conversion of a worktree
///  in it) must not start while the other runs,
/// and reading the claim is enough to know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = await liveOwnerOf({ directory, prefix }); // undefined when nobody owns the item
/// ```
pub fn live_owner_of(directory: &Path, prefix: &OsStr) -> Result<Option<BusyOwner>, MigrateError> {
    if !directory.is_dir() {
        return Ok(None);
    }
    let existing: Vec<u64> = generations(directory, prefix)?;
    let Some(newest) = existing.last() else { return Ok(None) };
    let path: PathBuf = path_of(directory, prefix, *newest);
    let Some(found) = read_record(path.as_path())? else { return Ok(None) };
    return match live_owner(&found, path.as_path())? {
        Some(pid) => Ok(Some(BusyOwner { pid, path })),
        None => Ok(None),
    };
}

/// Replace the claim file at `path` by a released record through a new private file and a rename,
/// then make it durable.
fn rewrite_as_released(directory: &Path, path: &Path) -> std::io::Result<()> {
    let private: PathBuf = write_private(directory, &ClaimRecord::Released)?;
    if let Err(error) = std::fs::rename(&private, path) {
        let _removed: std::io::Result<()> = std::fs::remove_file(&private);
        return Err(error);
    }
    return sync_directory(directory);
}

impl OwnerClaim {
    /// Give the item back:
    /// the claim file is rewritten,
    /// atomically,
    /// as a released claim,
    /// so the next command takes the next
    ///  generation without judging a process.
    pub fn release(&mut self) -> Result<(), MigrateError> {
        if self.released {
            return Ok(());
        }
        let written: std::io::Result<()> = rewrite_as_released(self.directory.as_path(), self.path.as_path());
        self.released = true;
        return match written {
            Ok(()) => Ok(()),
            Err(error) => Err(MigrateError::io("release the claim", self.path.as_path(), &error)),
        };
    }
}

impl Drop for OwnerClaim {
    fn drop(&mut self) {
        // A release that failed leaves a claim that names this process; it stops counting as alive when the process ends.
        let _released: Result<(), MigrateError> = self.release();
    }
}

/// Plant a claim file as another process would have left it.
#[cfg(test)]
pub(crate) fn plant_claim_for_tests(directory: &Path, prefix: &OsStr, generation: u64, pid: i64, birth: &str) {
    let record: ClaimRecord = ClaimRecord::Owned { pid, birth: birth.to_string(), namespace: current_pid_namespace() };
    std::fs::write(path_of(directory, prefix, generation), encode(&record)).expect("planted claim");
}

/// A process id that names no process:
/// the first number the kernel never hands out.
#[cfg(test)]
pub(crate) fn unused_pid_for_tests() -> i64 {
    let text: String = std::fs::read_to_string("/proc/sys/kernel/pid_max").expect("pid_max");
    return text.trim().parse().expect("a number");
}

/// The claim controls stay out of the release executable.
#[cfg(test)]
#[path = "single_owner_claim_tests.rs"]
mod tests;
