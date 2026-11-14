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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = tryClaim(`${address}.taking-over`); // { held } | { anotherProcessTakesOver } | throw
//! if (claim.held) { if (refuses(address)) { unlink(address); return listen(address) } }
//! ```

use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
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
///  The claim on one address, held for as long as this value lives.
/// Why:
///   The kernel holds it for this descriptor and no other,
/// so the claim ends where this value does, and a process that is killed gives it up with everything else it had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddressTakeoverClaim = { release(): void }; // flock(LOCK_EX) on the claim file
/// ```
#[derive(Debug)]
pub struct AddressTakeoverClaim {
    /// The file the kernel's claim is held on.
    file: std::fs::File,
}

impl Drop for AddressTakeoverClaim {
    fn drop(&mut self) {
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
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(path(address).as_path())?;
    // SAFETY: `flock` takes a descriptor this process owns and one number, and touches no memory.
    let taken: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if taken == 0 {
        return Ok(Claim::Held(AddressTakeoverClaim { file }));
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    return match error.raw_os_error() {
        Some(code) if code == libc::EWOULDBLOCK => Ok(Claim::AnotherProcessTakesOver),
        _ => Err(error),
    };
}

/// Controls of the claim stay out of the release executable.
#[cfg(test)]
#[path = "service_address_takeover_claim_tests.rs"]
mod tests;
