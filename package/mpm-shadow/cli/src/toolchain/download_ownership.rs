//! What:
//! Who owns a version's downloads directory while an install writes into it,
//! and who may remove it.
//! Why:
//! An install downloads an archive into `<store>/downloads/<tool>/<version>/` and then unpacks it,
//! and a removal that looks at that directory and then deletes it takes the archive away from an install that started
//! in between.
//! A check and then a delete cannot be made to hold:
//! the answer changes between the two steps.
//! So the ownership is taken before the first write and given up after the last use,
//! and the remover takes the right to remove before it judges anything:
//! the install creates its owner file with `create_new` and then looks for the remover's claim,
//! and the remover creates its claim with `create_new` and then looks for a live owner.
//! Two creates are ordered in real time,
//! so at least one of the two sees the other and backs off,
//! and neither writes or deletes under the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // download-ownership.ts
//! export type DownloadsOwner = { ownerFile: string };
//! export type RemovalRight = { held: true; claimFile: string } | { held: false; taken: boolean };
//! export function takeOwnership(root: string, tool: string, version: string, bound: number): DownloadsOwner  // throws
//! export function takeRemovalRight(root: string, tool: string, version: string): RemovalRight  // throws
//! export function liveOwner(downloads: string): string | undefined
//! ```

/// Imports what this module uses from `std::fs`.
use std::fs::{self, OpenOptions};
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Duration`,
/// `Instant` from `std::time`.
use std::time::{Duration, Instant};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `downloads_of` from `super::store_versions`,
/// which names the directory these rights are about.
use super::store_versions::downloads_of;
/// Imports the process identity that tells this run's owner file from one a kill left.
use super::util::{ProcessIdentity, process_has_ended, this_process};

/// The first field of the file an install writes to say the downloads directory is its own.
const OWNER_PREFIX: &str = ".mpm-shadow-downloading.";

/// The last field of the name a remover claims one version's downloads under,
/// which sits beside the version's directory so it can be created before the directory is looked at.
pub(crate) const REMOVAL_SUFFIX: &str = ".download-removal";

/// How long an install waits for a removal claim to go before it gives up.
/// It is `OWNERSHIP_WAIT`, ten seconds, since the owner's decision of 2026-11-15;
/// Why:
/// A removal of one downloads directory takes milliseconds,
/// so a claim that is still there after two seconds belongs to a command that is not finishing;
/// the install then reports that it could not take the directory rather than writing under the remover.
/// What:
/// How long an install waits for a removal to finish with the archives before it refuses with the reason.
/// Why:
/// The owner's decision of 2026-11-15 (recorded in `doc/handover/cli-git-rust-implementation.md`) is ten seconds:
/// two was the earlier bound, and a removal that also waits on a process, or on a slow disk, was refused while it was
/// still working.
const OWNERSHIP_WAIT: Duration = Duration::from_secs(10);

/// The pause between two attempts at a removal claim.
const OWNERSHIP_POLL: Duration = Duration::from_millis(20);

/// How many times taking a removal claim clears a stale one and tries again.
/// Why:
/// Each attempt clears one dead holder;
/// more than a few at once means something is writing claims this call cannot
/// read,
/// and the answer then is no,
/// not an endless loop.
const CLAIM_ATTEMPTS: u32 = 8;

/// What:
/// An install's ownership of one version's downloads directory,
/// held until the value is dropped.
/// Why:
/// Dropping removes the owner file,
/// so an early return or a panic cannot leave an ownership file that keeps a later removal from taking a directory
/// nobody is using any more.
#[derive(Debug)]
pub struct DownloadsOwner {
    /// The owner file this call wrote;
    /// removed when the value is dropped.
    owner: PathBuf,
}

/// The methods of `DownloadsOwner`.
impl DownloadsOwner {
    /// What:
    /// Takes ownership of the downloads directory of `version` of `tool` before the first byte is written,
    /// creating the directory when it is not there.
    /// Why:
    /// The owner file says which process is writing there,
    /// and a remover that takes its claim first sees it and leaves the directory alone.
    /// A removal claim that is already there means another command is removing these archives:
    /// the install waits for it,
    /// up to `OWNERSHIP_WAIT`,
    /// and then refuses rather than writing under it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// export function takeOwnership(root: string, tool: string, version: string): DownloadsOwner  // throws
    /// ```
    pub fn take(root: &Path, tool: &str, version: &str) -> Result<DownloadsOwner, ToolchainError> {
        let directory: PathBuf = downloads_of(root, tool, version);
        let claim: PathBuf = removal_claim_of(root, tool, version);
        let maker: ProcessIdentity = this_process().unwrap_or(ProcessIdentity { pid: std::process::id(), started: 0 });
        let owner: PathBuf = directory.join(format!("{OWNER_PREFIX}{}.{}", maker.pid, maker.started));
        let give_up_at: Instant = Instant::now() + OWNERSHIP_WAIT;
        loop {
            if let Err(error) = fs::create_dir_all(&directory) {
                return Err(ToolchainError::failure(format!("cannot create {}: {error}", directory.display())));
            }
            match OpenOptions::new().write(true).create_new(true).open(&owner) {
                Ok(file) => drop(file),
                // This run's own owner file from an earlier attempt is the normal case of asking twice.
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(ToolchainError::failure(format!("cannot write the owner file {}: {error}", owner.display())));
                }
            }
            // What: the claim is looked for after the owner file was written.
            // Why: the remover writes its claim before it looks for an owner, so of the two creates at least one is
            // seen by the other, and whoever sees it backs off before touching a byte.
            if !claim_is_held(&claim) {
                return Ok(DownloadsOwner { owner });
            }
            let _ = fs::remove_file(&owner);
            if Instant::now() >= give_up_at {
                return Err(ToolchainError::failure(format!(
                    "another command is removing the archives of {version} right now ({}), so this install did not \
                     write into that directory; run the command again when it has finished",
                    claim.display()
                )));
            }
            std::thread::sleep(OWNERSHIP_POLL);
        }
    }
}

/// Drops this install's ownership:
/// the owner file goes,
/// and a removal may take the directory.
impl Drop for DownloadsOwner {
    /// Gives this install's ownership up:
    /// the owner file goes,
    /// and a removal may take the directory from the next moment.
    fn drop(&mut self) {
        // Best effort: an owner file that cannot be removed names a process that has ended, and a remover ignores it.
        let _ = fs::remove_file(&self.owner);
    }
}

/// What a removal found when it asked for the right to remove one version's downloads.
#[derive(Debug)]
pub enum RemovalRight {
    /// This call holds the right until the value is dropped.
    Held {
        /// The claim that holds it:
        /// dropping the claim is what gives the right up.
        claim: RemovalClaim,
    },
    /// Another command holds the right.
    Taken,
    /// There is no downloads directory at that name.
    Absent,
}

/// What:
/// The right to remove one version's downloads,
/// held until the value is dropped.
/// Why:
/// The claim is what an install looks for after it writes its owner file,
/// so no install starts writing into a directory this command is removing.
#[derive(Debug)]
pub struct RemovalClaim {
    /// The claim file this call wrote;
    /// removed when the value is dropped,
    /// which is what lets an install take the directory again.
    path: PathBuf,
}

/// Drops the removal claim:
/// an install may take the directory again.
impl Drop for RemovalClaim {
    /// Gives the right to remove up:
    /// an install that was waiting for it may take the directory from the next moment.
    fn drop(&mut self) {
        // Best effort: a claim that stays keeps installs out of one directory until the next pass looks at it.
        let _ = fs::remove_file(&self.path);
    }
}

/// The name a remover claims one version's downloads under,
/// beside the version's directory in the tool's downloads directory.
fn removal_claim_of(root: &Path, tool: &str, version: &str) -> PathBuf {
    let directory: PathBuf = downloads_of(root, tool, version);
    return directory.with_file_name(format!(".{version}{REMOVAL_SUFFIX}"));
}

/// What:
/// Takes the right to remove one version's downloads,
/// or says who has it.
/// Why:
/// The claim is written before the directory is judged,
/// so a remover sees an install's owner file written before the claim,
/// and an install that starts after it sees the claim and waits;
/// that is what leaves no window between the check and the delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function takeRemovalRight(root: string, tool: string, version: string): RemovalRight  // throws
/// ```
pub fn take_removal_right(root: &Path, tool: &str, version: &str) -> Result<RemovalRight, ToolchainError> {
    let directory: PathBuf = downloads_of(root, tool, version);
    if !directory.is_dir() {
        return Ok(RemovalRight::Absent);
    }
    let claim: PathBuf = removal_claim_of(root, tool, version);
    let mut attempts: u32 = 0;
    loop {
        attempts += 1;
        match OpenOptions::new().write(true).create_new(true).open(&claim) {
            Ok(mut file) => {
                // What: the claim says which process holds it before it is handed out.
                // Why: a claim left by a kill must be provable stale by the next remover,
                // and a bare file name proves nothing; the number and start time are what a later process can check.
                let maker: ProcessIdentity = this_process().unwrap_or(ProcessIdentity { pid: std::process::id(), started: 0 });
                let _written = std::io::Write::write_all(&mut file, format!("{}.{}\n", maker.pid, maker.started).as_bytes());
                drop(file);
                return Ok(RemovalRight::Held { claim: RemovalClaim { path: claim } });
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(ToolchainError::failure(format!("cannot write the removal claim {}: {error}", claim.display())));
            }
        }
        let Some(reading): Option<ClaimReading> = read_claim(&claim) else {
            continue;
        };
        match &reading.maker {
            // A live command holds it: this one has nothing to do.
            Some(maker) if !process_has_ended(maker) => return Ok(RemovalRight::Taken),
            // What: a claim that does not say who holds it counts as held while it is young.
            // Why: the holder writes its number immediately after creating the file,
            // so a file read in that instant looks empty; a younger claim than the wait an install gives it is
            // therefore a writer in progress, and one older than that is a kill's leftover nobody finished.
            None if claim_is_young(&claim) => return Ok(RemovalRight::Taken),
            // The maker has ended, or the claim never got a readable maker and is old: a kill's leftover, which this
            // call takes for itself.
            _ => {
                if attempts > CLAIM_ATTEMPTS {
                    return Ok(RemovalRight::Taken);
                }
                // What: the leftover is taken by writing this process's identity into exactly the claim that was read.
                // Why: the review of this change showed a take that moved the claim aside and back: between the two
                // renames the name was free, so a third remover could take the right and the put back then overwrote
                // its claim, and two removers held one downloads directory. A claim is never moved now.
                if take_the_claim(&claim, &reading) {
                    return Ok(RemovalRight::Held { claim: RemovalClaim { path: claim } });
                }
            }
        }
    }
}

/// What:
/// Clears `version`'s removal claim when the claim's maker has ended, and answers whether it went.
/// Why:
/// A kill between the version's removal and the claim's unlink leaves a claim naming a dead maker beside a version that
/// is gone, and nothing else asks about that version again: the review of 82531265a7 left three further passes and saw
/// the claim stay, which also kept the tool's downloads directory from ever being removed.
pub(crate) fn clear_a_stale_claim(root: &Path, tool: &str, version: &str) -> bool {
    let claim: PathBuf = removal_claim_of(root, tool, version);
    let Some(reading): Option<ClaimReading> = read_claim(&claim) else {
        return false;
    };
    match &reading.maker {
        Some(maker) if !process_has_ended(maker) => return false,
        None if claim_is_young(&claim) => return false,
        _ => {}
    }
    return clear_the_stale_claim(&claim, &reading);
}

/// What:
/// Whether a live command holds the right to remove `version`'s archives now.
/// Why:
/// A removal moves the archives aside before it takes the version,
/// so between those two steps the only record of it is the downloads claim;
/// another pass must not finish the tombstone the removal is about to restore.
/// The claim names its maker,
/// so one a kill left answers no and its tombstone is finished by the pass.
pub(crate) fn removal_held(root: &Path, tool: &str, version: &str) -> bool {
    let claim: PathBuf = removal_claim_of(root, tool, version);
    if fs::symlink_metadata(&claim).is_err() {
        return false;
    }
    return claim_is_held(&claim);
}

/// What:
/// Whether a live command still holds the removal claim at `claim`,
/// clearing a claim whose maker has ended before answering.
/// Why:
/// A claim left by a kill names its maker,
/// and a later process can prove it gone:
/// answering "held" for ever would let one kill block every later install of that version,
/// and answering "free" for a live holder would put two commands on one directory.
/// A claim that cannot be read counts as held while it is young,
/// because its holder writes its number immediately
/// after creating the file.
fn claim_is_held(claim: &Path) -> bool {
    let Some(reading): Option<ClaimReading> = read_claim(claim) else {
        return false;
    };
    match &reading.maker {
        Some(maker) if !process_has_ended(maker) => return true,
        None if claim_is_young(claim) => return true,
        // The maker has ended, or the claim never got a readable maker and is old: a kill's leftover, which goes.
        _ => {}
    }
    // What: the leftover goes only as exactly the claim that was read, and only while this call holds its lock.
    // Why: a live claim another remover put there in the meantime is not this one's to remove.
    let _ = clear_the_stale_claim(claim, &reading);
    return false;
}

/// What:
/// One reading of a removal claim: what it says its maker is, and the file that said it.
/// Why:
/// The take below must remove exactly the claim whose text was judged,
/// so the numbers of the file read travel with the answer.
pub(crate) struct ClaimReading {
    /// Who the claim says holds it, when it says anything.
    maker: Option<ProcessIdentity>,
    /// The file that was read, for the take's comparison.
    metadata: fs::Metadata,
}

/// What:
/// Reads the claim at `claim` and what it says, or nothing when nothing is there.
/// Why:
/// A claim is created empty and given its maker's number immediately after,
/// so a reading can say nothing about its maker and still come from a file whose identity the take needs.
pub(crate) fn read_claim(claim: &Path) -> Option<ClaimReading> {
    let metadata: fs::Metadata = fs::symlink_metadata(claim).ok()?;
    let maker: Option<ProcessIdentity> = claim_holder(claim);
    return Some(ClaimReading { maker, metadata });
}

/// What:
/// The claim at `claim`, opened, locked and proved to be the file `read` described,
/// with what its text says now.
/// Why:
/// A claim is a name one command owns, and the name can be replaced between a reading and an act on it.
/// So the act is made only while this call holds the claim file's own lock and has re-read the name to be that same
/// file:
/// every taker of this file waits on the lock, so none of them can replace the name in between,
/// and a name that is not the file read is left alone.
/// The lock is held for the microseconds of one rewrite or one unlink and is never held for the length of a removal,
/// so it covers one claim file and nothing else.
struct LockedClaim {
    /// The claim file, locked by this call until the value is dropped.
    file: fs::File,
    /// What the file says now, read through the descriptor rather than the name.
    maker: Option<ProcessIdentity>,
}

/// What:
/// Opens the claim at `claim` for the act,
/// or answers nothing when the name is not the file `read` described, is not a regular file, or is locked by another
/// call.
/// Why:
/// The three checks together are what make the act safe:
/// the file opened must be the one judged (device and inode),
/// the name must still be that file after the lock is taken,
/// and only one call may be inside this window at a time.
fn lock_the_claim_it_read(claim: &Path, read: &ClaimReading) -> Option<LockedClaim> {
    use std::os::unix::fs::MetadataExt;

    let mut file: fs::File = OpenOptions::new().read(true).write(true).open(claim).ok()?;
    let opened: fs::Metadata = file.metadata().ok()?;
    // What: a link, a directory or a pipe where a claim belongs is not a claim.
    // Why: the act on a claim is a rewrite or an unlink of a regular file, and anything else there is another
    // command's or a person's.
    if !opened.is_file() || opened.dev() != read.metadata.dev() || opened.ino() != read.metadata.ino() {
        return None;
    }
    // What: the claim's own lock is taken without waiting.
    // Why: a call that cannot have it at once is asking about a claim another call is acting on, and its own attempt
    // is better spent re-reading the name.
    if file.try_lock().is_err() {
        return None;
    }
    // What: the name must still be the file that was opened.
    // Why: a taker that replaced the name did so before this lock was taken, and rewriting the name now would take
    // that taker's claim.
    let named: fs::Metadata = fs::symlink_metadata(claim).ok()?;
    if named.dev() != opened.dev() || named.ino() != opened.ino() {
        return None;
    }
    let maker: Option<ProcessIdentity> = holder_of_the_file(&mut file);
    return Some(LockedClaim { file, maker });
}

/// What:
/// Takes the claim at `claim` for this process by writing its identity into the file that was read,
/// or answers false when the name is no longer that file or a live command holds it.
/// Why:
/// The name never leaves its file, so there is no instant in which the name is free for a third remover,
/// which is what the review of this change found in a take that moved the claim aside and back.
/// The write is one truncate and one small write, inside the claim's own lock.
pub(crate) fn take_the_claim(claim: &Path, read: &ClaimReading) -> bool {
    let Some(mut locked): Option<LockedClaim> = lock_the_claim_it_read(claim, read) else {
        return false;
    };
    if locked.maker.as_ref().is_some_and(|maker| return !process_has_ended(maker)) {
        // A live command holds it: this call yields.
        return false;
    }
    let maker: ProcessIdentity = this_process().unwrap_or(ProcessIdentity { pid: std::process::id(), started: 0 });
    return write_identity(&mut locked.file, maker);
}

/// What:
/// Clears the stale claim at `claim`,
/// or answers false when the name is no longer the file that was read or a live command holds it.
/// Why:
/// This is the answer to "does a live removal hold this version's downloads": the stale leftover is removed so the
/// next pass can finish the tombstone beside it,
/// and it is removed by name only while this call holds the claim's lock and has proved the name is that file.
fn clear_the_stale_claim(claim: &Path, read: &ClaimReading) -> bool {
    let Some(locked): Option<LockedClaim> = lock_the_claim_it_read(claim, read) else {
        return false;
    };
    if locked.maker.as_ref().is_some_and(|maker| return !process_has_ended(maker)) {
        return false;
    }
    drop(locked);
    return fs::remove_file(claim).is_ok();
}

/// What:
/// Writes this process's identity into an already-open claim file,
/// emptying it first, and answers whether the write landed.
/// Why:
/// A claim that says which process holds it is what a later process checks,
/// and the identity is two numbers and a newline, so the file is truncated and written in place.
fn write_identity(file: &mut fs::File, maker: ProcessIdentity) -> bool {
    use std::io::{Seek, SeekFrom, Write};

    if file.set_len(0).is_err() {
        return false;
    }
    if file.seek(SeekFrom::Start(0)).is_err() {
        return false;
    }
    if file.write_all(format!("{}.{}\n", maker.pid, maker.started).as_bytes()).is_err() {
        return false;
    }
    return file.sync_data().is_ok();
}

/// What:
/// The process an open claim file says holds it,
/// read through the descriptor,
/// or `None` when it says nothing this call can read.
/// Why:
/// Reading the name again would judge a file this call has not locked;
/// the descriptor is the file the lock is on.
fn holder_of_the_file(file: &mut fs::File) -> Option<ProcessIdentity> {
    use std::io::{Read, Seek, SeekFrom};

    if !file.metadata().ok()?.is_file() {
        return None;
    }
    if file.seek(SeekFrom::Start(0)).is_err() {
        return None;
    }
    let mut text: String = String::new();
    if file.take(64).read_to_string(&mut text).is_err() {
        return None;
    }
    return parse_identity(&text);
}

/// What:
/// The process a claim's text names,
/// or `None` when the text does not name one.
/// Why:
/// A claim is created empty and given its holder's number immediately after,
/// so an empty or partial text is a writer in progress, not a claim without a holder.
fn parse_identity(text: &str) -> Option<ProcessIdentity> {
    let mut fields = text.trim().splitn(2, '.');
    let pid: u32 = fields.next()?.parse().ok()?;
    let started: u64 = fields.next()?.trim().parse().ok()?;
    return Some(ProcessIdentity { pid, started });
}

/// Whether the removal claim was written less than `OWNERSHIP_WAIT` ago.
/// Why:
/// A claim is created empty and then given its holder's number;
/// a claim read in between says nothing,
/// and the age is what tells that in-progress write from a claim a kill left
/// before it could be given a holder at all.
fn claim_is_young(claim: &Path) -> bool {
    let Ok(metadata) = fs::symlink_metadata(claim) else {
        return false;
    };
    let Ok(written) = metadata.modified() else {
        return false;
    };
    return std::time::SystemTime::now().duration_since(written).unwrap_or(Duration::ZERO) < OWNERSHIP_WAIT;
}

/// What:
/// The process a removal claim names as its holder,
/// or `None` when it names one this call cannot read.
/// Why:
/// A claim whose holder has ended is a kill's leftover and must not block the next remover for good;
/// the number and start time together are what tells a live holder from a later process with the same number.
fn claim_holder(claim: &Path) -> Option<ProcessIdentity> {
    let text: String = fs::read_to_string(claim).ok()?;
    let mut fields = text.trim().splitn(2, '.');
    let pid: u32 = fields.next()?.parse().ok()?;
    let started: u64 = fields.next()?.parse().ok()?;
    return Some(ProcessIdentity { pid, started });
}

/// What:
/// Why an install is writing in this downloads directory now,
/// or `None` when no owner file of a live process is there.
/// Why:
/// The remover has the right to remove by the time it asks,
/// so an owner file it finds is one whose install started before that right was taken;
/// a live one means the install is still using the archives,
/// and a dead one is what a killed install leaves,
/// which keeps nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function liveOwner(downloads: string): string | undefined
/// ```
pub fn live_owner(downloads: &Path) -> Option<String> {
    let Ok(entries) = fs::read_dir(downloads) else {
        return None;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Some(rest) = name.strip_prefix(OWNER_PREFIX) else {
            continue;
        };
        let mut fields = rest.splitn(2, '.');
        let Ok(pid) = fields.next().unwrap_or_default().parse::<u32>() else {
            continue;
        };
        let Ok(started) = fields.next().unwrap_or_default().parse::<u64>() else {
            continue;
        };
        let maker: ProcessIdentity = ProcessIdentity { pid, started };
        if !process_has_ended(&maker) {
            return Some(format!(
                "process {} started at {} is unpacking them",
                maker.pid, maker.started
            ));
        }
    }
    return None;
}
