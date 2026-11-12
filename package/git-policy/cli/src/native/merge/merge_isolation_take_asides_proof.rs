//! What:
//!  The proof that no program can write into a file whose last name is about to go.
//! Why:
//!  Removing a file's last name loses whatever a program still holding it writes afterwards,
//! and a scan of `/proc` cannot read the descriptors of a program of this account that is not dumpable,
//!  so the kernel's own answer is asked for where it offers leases
//!  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its first finding).
//!  A platform without leases has only the scan,
//! whose answer is the whole of what a link's removal needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = theProofThatNothingCanWriteInto(aside, scanned); // a read lease, or the scan's answer
//! ```

/// Every process this platform can read,
///  as one scan of the file's holders found them.
use crate::index_lock_holders::HolderEvidence;
/// The scan's reason for a process another account owns,
///  which a platform without leases acts through.
#[cfg(not(target_os = "linux"))]
use crate::index_lock_holders_accounts::OTHER_ACCOUNTS_PROCESS_REASON;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;

/// What:
///  The proof that nothing but this command can write into one file,
///  held for as long as the name of that file is being removed.
/// Why:
///   Removing a file's last name loses whatever a program still holding it writes afterwards,
/// and a scan of `/proc` cannot read the descriptors of a program of this account that is not dumpable,
///  so such a program could hold the file and write into an inode no name reaches
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its first finding).
/// A read lease is what the kernel refuses while any program has the file open for writing,
/// and one that is held keeps a later writer's open from reaching the file before the name goes,
///  so the kernel's answer is a proof the scan cannot give.
///  A file system that offers no leases answers nothing,
/// and the file is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProofNothingCanWrite = { lease?: Handle }; // dropped to let a later open through
/// ```
pub(crate) enum ProofNothingCanWrite {
    /// The open file whose read lease keeps a writer out until this is dropped.
    #[cfg(target_os = "linux")]
    KernelLease(std::fs::File),
    /// A scan that looked at every process this platform can read and found none holding the file,
    /// which is all a symbolic link's removal needs and all a platform without leases has.
    Scan,
}

impl ProofNothingCanWrite {
    /// What:
    ///  Remove the file this proof is about,
    ///  while the proof holds.
    /// Why:
    ///   The name goes while the lease keeps a writer out,
    /// and the lease is released only after the name is gone,
    ///  so no program can open the file between the proof and the removal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// remove(aside) { const outcome = rm(aside); close(this.lease); return outcome; }
    /// ```
    pub(crate) fn remove(self, aside: &Path) -> std::io::Result<()> {
        let outcome: std::io::Result<()> = std::fs::remove_file(aside);
        #[cfg(target_os = "linux")]
        if let Self::KernelLease(lease) = self {
            // Closing the descriptor releases the lease; the name is already gone.
            drop(lease);
        }
        return outcome;
    }
}

/// What:
///  Stop the kernel's lease-break notice from ending this command.
/// Why:
///   A program that opens a file this command holds a lease on makes the kernel send `SIGIO`,
/// whose default action is to terminate,
///  and this command asks for no asynchronous notification of its own:
/// the lease is taken to learn whether a writer holds the file and is released a step later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.once('SIGIO', () => {}); // the kernel's break notice says nothing this command waits for
/// ```
#[cfg(target_os = "linux")]
fn ignore_the_lease_break_notice() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(ignore_the_lease_break_notice_once);
}

/// What:
///  Ignore the kernel's lease-break notice,
///  which is the one step of `ignore_the_lease_break_notice`.
/// Why:
///  A once-guard takes a function rather than a closure that captures nothing;
/// the signal's disposition is the process's,
///  so one call is all it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignoreOnce = () => { const previous = ignoreSignal('SIGIO'); };
/// ```
#[cfg(target_os = "linux")]
fn ignore_the_lease_break_notice_once() {
    // SAFETY: `signal` with `SIG_IGN` for a signal this process never asks to receive, and the call is made once.
    let _previous = unsafe { libc::signal(libc::SIGIO, libc::SIG_IGN) };
}

/// What:
///  The proof that nothing can write into one aside,
///  from the kernel where it offers leases and from the scan where it does not.
/// Why:
///   The kernel's answer covers a writer this command cannot inspect,
/// and a platform without leases has only the scan:
/// a file whose scan found a holder,
///  or was cut at its bound,
/// or could not list the processes,
/// has no proof and is kept.
///  A symbolic link is not opened for writing by anyone,
/// so the scan's answer is the whole of what a link's removal needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theProofThatNothingCanWriteInto = (aside, scanned) => lstat(aside).isSymbolicLink() ? (scanProved(scanned) ? { kind: 'scan' } : undefined) : leaseOrUndefined(aside);
/// ```
pub(crate) fn the_proof_that_nothing_can_write_into(aside: &Path, scanned: &HolderEvidence) -> Option<ProofNothingCanWrite> {
    #[cfg(target_os = "linux")]
    {
        let _ = scanned;
        if matches!(std::fs::symlink_metadata(aside), Ok(found) if !found.file_type().is_file()) {
            return Some(ProofNothingCanWrite::Scan);
        }
        // A read lease, because the person may have made the file unwritable: it is refused while any program has the file open
        //  for writing, which is the only open that can change it, and it keeps a later writer out while the name goes.
        ignore_the_lease_break_notice();
        use std::os::fd::AsRawFd;
        let lease: std::fs::File = std::fs::File::open(aside).ok()?;
        // SAFETY: the descriptor is open for the call, and this command holds the only claim on it.
        let refused: libc::c_int = unsafe { libc::fcntl(lease.as_raw_fd(), libc::F_SETLEASE, libc::F_RDLCK) };
        if refused != 0 {
            return None;
        }
        return Some(ProofNothingCanWrite::KernelLease(lease));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(target_os = "linux"))]
    {
        let _ = aside;
        // A process another account owns cannot write into a file of this account's, and the scan of a platform without leases
        //  cannot see its descriptors: those reasons are the limit a removal acts through.
        let unruled_out: bool = scanned.partial.iter().any(|reason| !reason.starts_with(OTHER_ACCOUNTS_PROCESS_REASON));
        if scanned.holders.is_empty() && !unruled_out && !scanned.unresolved() {
            return Some(ProofNothingCanWrite::Scan);
        }
        return None;
    }
}
