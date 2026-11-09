//! What:
//!  The removal of an empty `index.lock` that no process holds and no Git lock PID file stands behind,
//! by the one service that is waiting for it,
//! with the proof it acted on written to the service journal and told to the person.
//! Why:
//!  On 2026-11-07 four commits stopped after up to twenty minutes behind one empty `index.lock`
//! (no Git lock PID file,
//! no process with it open,
//! maker unknown) until an agent removed it by hand.
//! The owner's answer (2026-11-07,
//! "Service removes it"):
//! after a complete scan of every readable process finds no holder and there is no Git PID file,
//! the service removes the lock and lands the commit.
//! The limit stated with the question stands:
//! a process in another user namespace,
//! or in the same one but not readable,
//! cannot be seen,
//! so the removal can take a lock that is in fact held.
//! This release removes only an empty lock,
//! which is narrower than the answer and open to the owner's widening.
//! Measured on 2026-11-07 with Git 2.56.0 in a throwaway repository:
//! `git commit --all`,
//! `git commit --only <path>` and `git commit <path>` keep `index.lock` through their editor with the whole index written into it
//! (193 bytes there) and no process holding it open (`/proc/<pid>/fd` of every process named it in none),
//! and write no PID file unless `core.lockfilePid` is on;
//! a plain `git commit` of the staged index holds no lock then.
//! So a non-empty lock with no holder is what a person's editor-phase commit looks like,
//! and removing it would make that commit fail when the editor closes
//! (the end-to-end scenario `foreign-index-lock-unproven` is exactly that holder).
//! A Git holds the lock's descriptor from its creation until it has written the index into it,
//! so an empty lock that no readable process has open is one a Git left when it was killed between creating and writing.
//! A non-empty lock stays as before:
//! waited for to the ceiling,
//! then refused with the way to clear it.
//! The removal is one action with one owner:
//! the file is opened first and judged through that descriptor (its device,
//! inode and size are the identity the proof names),
//! the scan is of that identity,
//! the name is looked at once more just before the unlink,
//! and a name that no longer refers to the opened file is never removed,
//! so a lock another program placed in its place keeps its place.
//! What stays open is the instant between that last look and the unlink,
//! because no system call unlinks a name only if it still names a given file
//! (`service_index_reclaim.rs`,
//! `doc/planning/cli-git-one-reclaimer-per-index.md`).
//!  A merge whose wait met a Git lock PID file naming a number this process table does not have removes that
//!  very lock here too,
//!  on the same proof and with the one requirement on the PID file changed (`Disuse`,
//!  `remove_if_the_owner_is_gone`),
//!  so a merge repairs what a commit repairs
//!  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//!  finding 3).
//!  Only an empty lock is removed,
//!  so a live Git's editor-phase lock (the whole index written into it) is out of reach of both.
//! A lock that carries a marker of the service is not judged here:
//! `service_index_reclaim.rs` removes a dead service's lock and waits for a live one's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeIfUnheld(platform, index, scan) // open; empty and unmarked? no pid file? scan complete and empty; own descriptors == 1; same file; unlink
//! ```

use crate::index_lock_evidence::{LockMetadata, PidFile, host_pid_process, read_pid_file_evidence};
use crate::index_lock_holders::HolderEvidence;
use crate::recovery_files::lock_pid_path;
use crate::service_index_marker::marker_of_file;
use crate::service_index_placement::Platform;
use crate::service_index_lock::lock_path_of;
use crate::service_index_reclaim::{open_to_judge, same_file};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What the removal acted on:
/// the lock's identity as opened,
/// and how much of the machine the scan could not read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    /// The lock that was removed.
    pub lock: PathBuf,
    /// The evidence of disuse the removal acted on.
    pub disuse: Disuse,
    /// The process a Git lock PID file named,
    /// when the removal acted on one that this table does not have.
    pub gone_pid: Option<i64>,
    /// The device of the file that was opened.
    pub device: u64,
    /// The inode of the file that was opened.
    pub inode: u64,
    /// The size in bytes of the file that was opened.
    pub size: u64,
    /// How many processes the scan could not read (their descriptors were refused),
    /// which the scan counts as unseen,
    /// not as holders.
    pub unreadable: usize,
}

/// What came of a look at a lock nothing is known to hold.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unheld {
    /// The lock was removed on this proof.
    Removed(Proof),
    /// The name no longer refers to the file that was judged,
    /// or refers to nothing:
    /// the placement is worth another try.
    Changed,
    /// The lock is not proven unheld:
    /// it is left where it is and waited for as before.
    Kept,
}

/// What every reason of a scan begins with when one process's descriptors were refused (`index_lock_holders.rs`,
/// `scan_process`),
/// as opposed to a scan cut at its bound or unable to list the processes,
/// which say something else.
const UNREADABLE_PROCESS_REASON: &str = "PID ";

/// Whether a descriptor's link names a file a repository could hold,
/// rather than a pipe,
/// a socket,
/// an anonymous or memory file,
/// a device,
/// or a file of `/proc` or `/sys`.
/// The question is asked of the link's text,
/// which the kernel answers from its own memory.
fn names_a_file_of_a_repository(link: &Path) -> bool {
    let Ok(target) = std::fs::read_link(link) else {
        // A target that cannot be read may be a file that has been carried away:
        // the count is then unknown rather than zero.
        return true;
    };
    let text: String = target.to_string_lossy().into_owned();
    // The kernel writes a memory file's link as `/memfd:<name> (deleted)`, with the leading slash its other pseudo
    // names do not have, so the slash is dropped before they are matched: a test for `memfd:` alone skipped the file,
    // the count became unknown, and an abandoned lock stayed while a capture was open (cross-vendor review of
    // 2026-11-09, finding 4).
    let name: &str = text.strip_prefix('/').unwrap_or(text.as_str());
    for pseudo in ["pipe:[", "socket:[", "anon_inode:", "memfd:"] {
        if name.starts_with(pseudo) {
            return false;
        }
    }
    for directory in ["/proc/", "/sys/", "/dev/"] {
        if text.starts_with(directory) {
            return false;
        }
    }
    return true;
}

/// How many descriptors of this process (every thread's) name the file with this device and inode.
/// A process that cannot be counted is not proven to hold none,
/// so the count is then the largest there is.
/// Each descriptor is read through the one reader that refuses a file system whose attributes are not in the memory of
/// the kernel (`open_file_of_a_descriptor.rs`),
/// so a server that has stopped is never asked for anything:
/// a descriptor that cannot be read that way,
/// and whose link names a file a repository could hold,
/// makes the count unknown.
fn own_descriptors_of(device: u64, inode: u64) -> usize {
    let Ok(listing) = std::fs::read_dir("/proc/self/fd") else {
        return usize::MAX;
    };
    let mut count: usize = 0;
    for entry in listing.flatten() {
        match crate::open_file_of_a_descriptor::opened_file_of(Path::new("/proc/self"), entry.file_name().as_os_str()) {
            Ok(opened) if opened.device == device && opened.inode == inode => count += 1,
            Ok(_) => {}
            // A descriptor that ended while it was being counted names nothing any more.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => {
                if !names_a_file_of_a_repository(entry.path().as_path()) {
                    continue;
                }
                return usize::MAX;
            }
        }
    }
    return count;
}

/// What:
///  Which evidence of disuse a removal acts on.
/// Why:
///   The service removes a lock no PID file stands behind,
/// while a merge that waited on a PID file naming a number this table does not have removes that very lock;
/// both are the owner's,
/// from 2026-11-07 ("Service removes it") and from the answer for a held `index.lock`,
/// and only the requirement on the PID file differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Disuse = 'no-pid-file' | 'pid-file-of-a-gone-process';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disuse {
    /// No Git lock PID file exists at all.
    NoPidFile,
    /// The Git lock PID file names a number this process table does not have,
    /// which is what a Git that was killed leaves.
    PidFileOfAGoneProcess,
}

/// What:
///  The removal a merge makes: the empty `index.lock` of `index` whose PID file names a number this process table
///  does not have,
///  on the same proof as the service's removal,
///  saying what it acted on.
/// Why:
///   A Git killed under `core.lockfilePid` leaves exactly that,
/// and a merge that waited on it once repeated its wait and then refused with advice that could not work
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 3);
/// the commit service repaired what a merge refused.
/// A lock with content,
/// one carrying the service's marker,
/// a PID file naming a process this table has,
/// a holder the scan saw,
/// a scan that did not finish,
/// and a lock that is not a plain file all leave it where it is.
///  Only Linux can scan every process,
/// so elsewhere nothing is proven and nothing is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfTheOwnerIsGone(platform, index, scan) // 'removed' | 'changed' | 'kept'
/// ```
pub fn remove_if_the_owner_is_gone(platform: &Platform, index: &Path, scan: &dyn Fn(&Path, u64, u64) -> HolderEvidence) -> Unheld {
    return remove_when(platform, index, scan, Disuse::PidFileOfAGoneProcess);
}

/// What:
///  Removes the empty `index.lock` of `index` when nothing proves a program uses it,
/// on the proof the owner named,
/// and says what it acted on.
/// Why:
///   The lock is judged through the descriptor it was opened with,
/// so the identity in the proof,
/// the file the scan looked for and the file removed are one file;
/// the name is looked at once more before the unlink.
/// A lock with content,
/// a lock with the service's marker,
/// a PID file of any kind,
/// a holder the scan saw,
/// a scan that did not finish,
/// a descriptor of this process on it,
/// or anything that is not a plain file leaves the lock where it is.
/// Only Linux can scan every process,
/// so elsewhere nothing is proven and nothing is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfUnheld(platform, index, scan) // 'removed' | 'changed' | 'kept'
/// ```
pub fn remove_if_unheld(platform: &Platform, index: &Path, scan: &dyn Fn(&Path, u64, u64) -> HolderEvidence) -> Unheld {
    return remove_when(platform, index, scan, Disuse::NoPidFile);
}

/// What:
///  `remove_if_unheld` with the evidence of disuse given.
/// Why:
///   One body judges and removes,
/// so the identity that was opened,
/// scanned and removed is the same in both callers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeWhen(platform, index, scan, disuse): Unheld;
/// ```
fn remove_when(platform: &Platform, index: &Path, scan: &dyn Fn(&Path, u64, u64) -> HolderEvidence, disuse: Disuse) -> Unheld {
    if !cfg!(target_os = "linux") {
        return Unheld::Kept;
    }
    let lock: PathBuf = lock_path_of(index);
    let file: std::fs::File = match open_to_judge(lock.as_path()) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Unheld::Changed,
        Err(_) => return Unheld::Kept,
    };
    let Ok(opened) = file.metadata() else {
        return Unheld::Kept;
    };
    // A lock with content is a Git's editor-phase or hook-phase lock as readily as a dead one's (see the module's measurement).
    if !opened.is_file() || opened.len() != 0 || marker_of_file(&file).is_some() {
        return Unheld::Kept;
    }
    let identity: LockMetadata = LockMetadata { device: opened.dev(), inode: opened.ino(), ctime_ms: opened.ctime() * 1000 + opened.ctime_nsec() / 1_000_000 };
    // A damaged PID file, a file naming a process this table has, and (for the service) any PID file at all are evidence
    //  this removal does not act on.
    let pid_file: PidFile = read_pid_file_evidence(lock_pid_path(index).as_path(), host_pid_process);
    let disuse_proved: bool = match disuse {
        Disuse::NoPidFile => pid_file == PidFile::Absent,
        // A number this table does not have may also be a live Git's of another PID namespace; the empty lock and the
        //  complete scan that finds no holder are the rest of the proof, and only an empty lock is reached here
        //  (cross-vendor review of 2026-11-09, findings 1 and 2).
        Disuse::PidFileOfAGoneProcess => matches!(&pid_file, PidFile::Owner(_, owner) if owner.gone()),
    };
    if !disuse_proved {
        return Unheld::Kept;
    }
    let scanned: HolderEvidence = scan(lock.as_path(), identity.device, identity.inode);
    if !scanned.holders.is_empty() {
        return Unheld::Kept;
    }
    // A descriptor whose record named the lock and that the kernel refused to copy is a process that may hold it:
    // the scan did not finish, so it proves nothing (cross-vendor review of 2026-11-09, finding 2).
    if !scanned.unresolvable.is_empty() {
        return Unheld::Kept;
    }
    let mut unreadable: usize = 0;
    for reason in &scanned.partial {
        if !reason.starts_with(UNREADABLE_PROCESS_REASON) {
            // A scan cut at its bound, or unable to list the processes, did not look at every readable process.
            return Unheld::Kept;
        }
        unreadable += 1;
    }
    // The scan leaves this process out, and this process may hold the lock (another landing placing one where no file is made whole).
    if own_descriptors_of(identity.device, identity.inode) != 1 {
        return Unheld::Kept;
    }
    // A control reaches here to act as another program would after the scan.
    crate::service_fault::reach("unheld-scanned");
    // A Git that writes the index into this lock while the scan runs and closes its descriptor is the editor-phase holder the
    // module measured, and the scan sees no holder of it then: the file is judged again here, through the descriptor it was
    // opened with and as late as the removal can be decided, so the emptiness the removal rests on is the file's at the moment
    // of the removal (cross-vendor review of 2026-11-09, finding 1).
    let Ok(filled) = file.metadata() else {
        return Unheld::Kept;
    };
    if !filled.is_file() || filled.len() != 0 || marker_of_file(&file).is_some() {
        return Unheld::Kept;
    }
    match same_file(lock.as_path(), &file) {
        Ok(true) => {}
        Ok(false) => return Unheld::Changed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Unheld::Changed,
        Err(error) => {
            crate::service_error!("the index lock {} that no process holds cannot be looked at ({error}), so it is not removed", lock.display());
            return Unheld::Kept;
        }
    }
    let gone_pid: Option<i64> = match (&pid_file, disuse) {
        (PidFile::Owner(pid, owner), Disuse::PidFileOfAGoneProcess) if owner.gone() => Some(*pid),
        _ => None,
    };
    let proof: Proof = Proof { lock: lock.clone(), disuse, gone_pid, device: identity.device, inode: identity.inode, size: opened.len(), unreadable };
    // The proof is journaled before the removal, so a removal that is then cut short still has its reason on record.
    crate::service_warn!("{}", journal_words(&proof));
    return match (platform.unlink)(lock.as_path()) {
        Ok(()) => Unheld::Removed(proof),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Unheld::Changed,
        Err(error) => {
            crate::service_error!("the index lock {} that no process holds cannot be removed ({error}); remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays", lock.display());
            Unheld::Kept
        }
    };
}

/// What:
///  The line the service journal keeps for a removal:
/// the lock,
/// its identity as opened,
/// and what the scan and the PID file said.
/// Why:
///   The removal can take a lock that was held by a program the scan cannot see;
/// the journal must say what it acted on so that a later reader can tell which case it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `removing ${lock} (device ${device}, inode ${inode}, ${size} bytes): ...`
/// ```
pub fn journal_words(proof: &Proof) -> String {
    return format!(
        "removing the empty index lock {} (device {}, inode {}, {} bytes): it carries no marker of the service, {} and the scan of every process \
         this service could read found no process with it open (unreadable processes: {}); the owner chose this removal on 2026-11-07 (\"Service removes it\")",
        proof.lock.display(),
        proof.device,
        proof.inode,
        proof.size,
        disuse_words(proof),
        proof.unreadable
    );
}

/// What:
///  The clause that names the evidence of disuse a removal acted on,
/// for the journal.
/// Why:
///   The two removals act on different evidence,
/// and a journal line that named the wrong one would hide which policy removed a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// disuse === 'no-pid-file' ? 'there is no Git lock PID file beside it,' : 'its Git lock PID file names PID N, which this table does not have,';
/// ```
fn disuse_words(proof: &Proof) -> String {
    return match (proof.disuse, proof.gone_pid) {
        (Disuse::NoPidFile, _) => String::from("there is no Git lock PID file beside it,"),
        (Disuse::PidFileOfAGoneProcess, Some(pid)) => format!("its Git lock PID file names PID {pid}, which this process table does not have,"),
        (Disuse::PidFileOfAGoneProcess, None) => String::from("its Git lock PID file names a process this process table does not have,"),
    };
}

/// What:
///  The line the person is told when a lock was removed:
/// what was removed,
/// why it was judged unheld,
/// what the judgement cannot see,
/// and what to do if that was the case.
/// Why:
///   A removal of a file the person may own must be said in plain words at the moment it happens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: removed the index lock ${lock}, ...`
/// ```
pub fn told_words(proof: &Proof) -> String {
    return format!(
        "monochromatic-git-policy-cli: removed the empty index lock {}, because no process this service could read had it open and {}, \
         so nothing proved a program was still using it; the command goes on. The scan cannot see a process in another user namespace or one it may not read; \
         if a Git there was starting a command in this repository, run its command again.\n",
        proof.lock.display(),
        told_disuse_words(proof)
    );
}

/// What:
///  The clause that tells the person which evidence of disuse removed the lock.
/// Why:
///   The person must be able to tell a lock no PID file stood behind from one a Git left,
/// because the second can be removed while a Git of another PID namespace still holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// disuse === 'no-pid-file' ? 'no Git lock PID file stood beside it' : `its Git lock PID file named PID ${pid}, which this table does not have`;
/// ```
fn told_disuse_words(proof: &Proof) -> String {
    return match (proof.disuse, proof.gone_pid) {
        (Disuse::NoPidFile, _) => String::from("no Git lock PID file stood beside it"),
        (Disuse::PidFileOfAGoneProcess, Some(pid)) => format!("its Git lock PID file named PID {pid}, which this process table does not have"),
        (Disuse::PidFileOfAGoneProcess, None) => String::from("its Git lock PID file named a process this table does not have"),
    };
}

/// The pieces of the proof are controlled in the test module.
#[cfg(test)]
#[path = "service_index_unheld_removal_tests.rs"]
mod tests;
