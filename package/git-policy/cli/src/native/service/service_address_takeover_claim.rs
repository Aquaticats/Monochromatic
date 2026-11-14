//! What:
//!  The right to replace the socket file at one worktree's address,
//!  held as a claim the kernel gives to one process at a time for the few system calls a takeover takes.
//! Why:
//!   A service that was killed leaves its socket file, and the next server takes that file over:
//! it asks the kernel whether anything still holds the address (a refusal of a connection is the proof),
//! unlinks the file and binds its own.
//!  Two servers that both meet the same stale file both ask, both are refused, and both unlink and bind:
//! the second unlink removes the first's *live* socket file, and two live services are left for one worktree,
//!  the second of them listening on a socket no path names any more
//! (the review of `76a08c9fac` reproduced it with the built binary).
//!  The three steps cannot be made one by position alone.
//! Nothing in the filesystem says "unlink this file only while it is still the inode I asked about":
//! `unlink` removes whatever the path names by then, and a rename of the path away is still a rename whatever the path
//!  names by then, so a second taker can always arrive between another taker's question and its answer.
//!  So the claim is the kernel's own, on a file beside the address:
//!  one process holds it at a time,
//! nothing is ever read from it, so it needs no owner text to be believed and has no window between being made and
//!  naming its owner,
//!  and the kernel gives it up when the holder exits or is killed at any instruction, so a claim never outlives its
//!  holder and never has to be taken over.
//!  The claim is taken without waiting:
//! a process that finds it held is one that another server is taking the address over at this instant,
//!  and it leaves the address alone rather than waiting for a turn.
//!  What it covers is the three system calls of a takeover, a probe, an unlink and a bind, and nothing else;
//! no command waits on it, and it is held for microseconds at the end of a service's start.
//!  The file exists only while a takeover does, and it is the claim's own holder that removes it:
//! the one command that created it, or the one that took a dead holder's over,
//! unlinks it by its inode as its takeover ends,
//! so an ordinary service's lifetime leaves no file at all.
//! A command killed inside the microseconds of a takeover leaves the file behind,
//! and the next takeover opens it, takes the claim the kernel has already given up, and removes it at its own end;
//! the sweep for a worktree that is gone removes one still there
//! (`service_files_of_a_stopped_instance.rs`, `forget_stopped_instance`).
//!  Removal is safe because it happens after the address is bound:
//! a taker that arrives once the file is gone finds the address accepting and never claims anything.
//!  The file a claim is taken on is asked for again once the lock is held:
//! a file that went between the open and the lock is a claim whose file no path names, and the command that opened it
//!  answers as it does for a claim another process holds
//! (`claim_the`; the review of `77f5d525e3`, finding 2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = tryClaim(`${address}.taking-over`); // { held } | { anotherProcessTakesOver } | throw
//! if (claim.held) { if (refuses(address)) { unlink(address); return listen(address) } }
//! ```

use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// The ending of the file the claim on one address is held on.
pub const CLAIM_ENDING: &str = ".taking-over";

/// The file one address's claim is held on, beside the address's own file.
pub fn path(address: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = address.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    name.push(CLAIM_ENDING);
    return address.with_file_name(name);
}

/// What:
///  The claim on one address, held for as long as this value lives,
///  and the file it is held on, which this value removes as it ends.
/// Why:
///   The kernel holds it for this descriptor and no other,
/// so the claim ends where this value does, and a process that is killed gives it up with everything else it had.
///  The file is unlinked by the inode this claim opened, never by its path alone:
/// a command that ended and another that started could otherwise unlink each other's claim file, and two claims on one
///  address are the defect the file exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddressTakeoverClaim = { release(): void }; // flock(LOCK_EX) on the claim file, then unlink while it is still ours
/// ```
#[derive(Debug)]
pub struct AddressTakeoverClaim {
    /// The file the kernel's claim is held on.
    file: std::fs::File,
    /// The path the claim file was opened at.
    path: PathBuf,
    /// The device and inode of the claim file this value opened.
    identity: (u64, u64),
}

impl AddressTakeoverClaim {
    /// The claim on `file`, which is the file at `path`, with the identity that says the path is still that file.
    fn of(file: std::fs::File, path: PathBuf) -> std::io::Result<Self> {
        let metadata: std::fs::Metadata = file.metadata()?;
        return Ok(Self { file, path, identity: (metadata.dev(), metadata.ino()) });
    }
}

impl Drop for AddressTakeoverClaim {
    fn drop(&mut self) {
        // What: The claim file goes while the claim is still held, and only while the path still names the file this claim
        //       opened.
        // Why:  A file left behind is a file the wrapper leaves in a runtime directory it served for good (the review of
        //       `55f236a0a8`, finding 3). The inode comparison is what keeps two claims from being made on one address: a
        //       path that names another file is another claim's, and this one removes nothing.
        //       Unlinking after the address is bound is what makes the removal harmless: a taker that arrives later finds
        //       the address accepting.
        if let Ok(now) = std::fs::symlink_metadata(self.path.as_path())
            && (now.dev(), now.ino()) == self.identity
        {
            let _removed: std::io::Result<()> = std::fs::remove_file(self.path.as_path());
        }
        // Lock kept: the kernel's own claim on one worktree's address, held for the three system calls of a takeover.
        // Why:  nothing in the filesystem can say "unlink this file only while it is still the inode I asked about", and two
        //       servers that both unlinked and bound left one of them unreachable (the review of `76a08c9fac`, finding 3).
        //       The kernel's claim has no owner text and no stale state, is taken without waiting, and covers a probe, an
        //       unlink and a bind, which is the smallest unit a takeover has.
        // SAFETY: `flock` takes one descriptor of this claim's file and one number, and touches no memory. Closing the file
        // would give the claim up too; asking first is what says when it ended.
        let _released: libc::c_int = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

/// What:
///  What a claim attempt came to.
/// Why:
///   A claim another process holds is a normal answer (that process is taking the address over right now),
/// not a failure,
///  and it is never a reason to wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimOutcome = { kind: 'held'; claim } | { kind: 'anotherProcessTakesOver' };
/// ```
#[derive(Debug)]
pub enum Claim {
    /// This process may take the address over.
    Held(AddressTakeoverClaim),
    /// Another process either holds the claim now or took the address over and gave the claim back between this
    /// process's look and its own; either way this process must not unlink what is there.
    AnotherProcessTakesOver,
}

/// What:
///  Claims the takeover of one address,
///  or says that another process has it.
/// Why:
///   See the module description.
/// A file that cannot be opened or claimed is an error rather than a claim:
/// the caller leaves the address as it is,
///  which is the state a takeover that could not be proven leaves it in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim(address) // flock(claimFile(address), LOCK_EX | LOCK_NB)
/// ```
pub(crate) fn claim(address: &Path) -> std::io::Result<Claim> {
    return claim_seamed(address, &mut do_nothing);
}

/// The call a claim makes between opening the claim file and taking the kernel's lock on it:
/// none.
fn do_nothing() {}

/// `claim` with a call between the open and the lock, where a control may let another command open the same file, let its
/// holder unlink it, and put another file at the path.
pub(crate) fn claim_seamed(address: &Path, after_the_open: &mut dyn FnMut()) -> std::io::Result<Claim> {
    let claim_path: PathBuf = path(address);
    // What: Two rounds, and each one ends in a claim or in another process's claim.
    // Why:  This command creates the file when no takeover is in flight, and the round after a file that went between the
    //       two steps is the file another command created and removed in the meantime.
    for _round in 0..2 {
        match std::fs::OpenOptions::new().read(true).write(true).create_new(true).mode(0o600).open(claim_path.as_path()) {
            // What: A file this command created is claimed in the same step, so no second process can slip in between.
            // Why:  A file that exists is not a claim: the claim is the kernel's lock on it, and a file created without one
            //       is a file another process may take (`claim_the`).
            Ok(file) => return claim_the(file, claim_path, after_the_open),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // What: A file that is there is opened and claimed, or it is another process's claim.
                // Why:  A file a killed taker left holds no claim: the kernel gave that up when the process ended, so this
                //       command takes it and removes it at the end of its own takeover.
                let Ok(file) = std::fs::OpenOptions::new().read(true).write(true).open(claim_path.as_path()) else { continue };
                return claim_the(file, claim_path, after_the_open);
            }
            Err(refused) => return Err(refused),
        }
    }
    return Ok(Claim::AnotherProcessTakesOver);
}

/// What:
///  Takes the kernel's exclusive claim on an open claim file,
///  without waiting,
///  and only while the path still names the file this call opened.
/// Why:
///   A claim another process holds is a normal answer, not a failure: it is taking this address over at this instant.
///  The file this call opened can stop being the claim's file between the open and the lock:
/// a holder that is ending unlinks the file by its inode, a command between the two steps creates another at the path, and
///  this call's lock lands on a file no path names any more.
/// Two takers would then hold a claim each, one of them on a dead file, and the claim would no longer be one process's
///  for one address (the review of `77f5d525e3`, finding 2).
///  So the lock is taken and then the path is asked again: a path that names another file is another command's claim,
/// and this call releases its lock and answers as it does for a claim it cannot have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// flock(file, LOCK_EX | LOCK_NB) && sameFile(path, file) ? held : anotherProcessTakesOver
/// ```
fn claim_the(file: std::fs::File, path: PathBuf, after_the_open: &mut dyn FnMut()) -> std::io::Result<Claim> {
    after_the_open();
    // SAFETY: `flock` takes a descriptor this process owns and one number, and touches no memory.
    let taken: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if taken != 0 {
        let refused: std::io::Error = std::io::Error::last_os_error();
        return match refused.raw_os_error() {
            Some(code) if code == libc::EWOULDBLOCK => Ok(Claim::AnotherProcessTakesOver),
            _ => Err(refused),
        };
    }
    // What: The path must still name the file this call locked, and a path that names anything else is another claim.
    // Why:  The lock alone says which file this process holds, never that the file is the address's claim: the file is
    //       unlinked by inode when the holder it was opened beside ends, and the next command creates a new one.
    let opened: (u64, u64) = {
        let metadata: std::fs::Metadata = file.metadata()?;
        (metadata.dev(), metadata.ino())
    };
    let named_now: Option<(u64, u64)> = std::fs::symlink_metadata(path.as_path()).ok().map(|metadata| { return (metadata.dev(), metadata.ino()); });
    if named_now != Some(opened) {
        // The lock goes with the file, which this call does not keep: the claim it took is not the address's.
        return Ok(Claim::AnotherProcessTakesOver);
    }
    return Ok(Claim::Held(AddressTakeoverClaim::of(file, path)?));
}

/// What:
///  Removes a claim file a killed taker left, while taking that claim,
///  and leaves one another process is holding.
/// Why:
///  A service that is ending has unlinked its own address, so nothing of its worktree claims anything any more, and a file
/// a killed command left there is a file the wrapper leaves in a runtime directory it served for good
///  (the review of `55f236a0a8`, finding 3).
/// Taking the claim before removing the file is what keeps the removal from making two claims of one address:
/// a taker that arrives while this holds it is told another process has the claim, and a taker that arrives once it is
///  gone binds the address (or finds its own stale file) without ever claiming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeLeftoverClaim(address) // flock(LOCK_EX|LOCK_NB): unlink by inode when taken, otherwise leave it
/// ```
pub fn remove_a_leftover_claim(address: &Path) -> bool {
    let claim_path: PathBuf = path(address);
    // A file that is not there is nothing to remove.
    let Ok(file) = std::fs::OpenOptions::new().read(true).write(true).open(claim_path.as_path()) else { return false };
    // SAFETY: `flock` takes a descriptor this process owns and one number, and touches no memory.
    let taken: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if taken != 0 {
        // Another process is taking the address over at this instant, and its file is its own to remove.
        return false;
    }
    // What: The file goes only while the path still names the file this call opened.
    // Why:  A path that names another file is another claim's, and removing it would let a second claim be made.
    let Ok(opened) = file.metadata() else { return false };
    let Ok(now) = std::fs::symlink_metadata(claim_path.as_path()) else { return false };
    if (now.dev(), now.ino()) != (opened.dev(), opened.ino()) {
        return false;
    }
    return std::fs::remove_file(claim_path.as_path()).is_ok();
}

/// Controls of the claim stay out of the release executable.
#[cfg(test)]
#[path = "service_address_takeover_claim_tests.rs"]
mod tests;
