//! What:
//!  The proof that one Git lock file is abandoned,
//!  and the one removal that acts on it.
//! Why:
//!  A lock whose holder is proven gone protects nothing,
//!  and the command that meets it must go on instead of waiting a minute and giving up.
//!  Two proofs decide,
//!  and the first is the one this pass prefers:
//!
//!  - the record a command of this repository wrote about the very lock its own Git took
//!    (`lock_bindings.rs`,
//!  `lock_watch.rs`):
//!    that command kept its process around its Git,
//!    saw the kernel report the lock created and then its PID file,
//!    pinned the very inode the lock had,
//!    and named that Git by the birth identity it read of its own child.
//!    So the birth times decide nothing on this path,
//!    which is what makes it act on this volume,
//!    where the two files one Git writes at once carry one stamp.
//!    It proves the record's lock,
//!    and it proves nothing about a lock made again at the same path (the pin holds the old inode),
//!    about a PID file that no longer names the recorded number,
//!    or about a Git the record does not name,
//!    and each of those keeps the lock.
//!    Only the commands whose Git this process waits for write such a record:
//!    a command that replaces itself with real Git keeps no process,
//!    so its lock keeps the second proof or stands.
//!  - the writer of the lock's PID file,
//!    where no record exists,
//!    never the number alone (`lock_holder_record_reads.rs`,
//! `proved_gone_writer`):
//!    the PID file must be born strictly after the lock,
//!    no live process may have the number it names,
//!    and every record of this repository naming that number must be a process of this boot,
//!    this PID namespace and this machine that has ended.
//!  What each way the three facts can disagree does here:
//!
//!  - a number used again:
//!    a live process has it,
//!    so the lock stays.
//!    Nothing tells the live Git that wrote the PID file from a later program that only has its number.
//!  - two records for one number:
//!    all of them must be gone.
//!    One record of another boot,
//!    namespace or machine keeps the lock.
//!  - a record written after the lock:
//!    it names a later process's birth,
//!    so the number is live or the record is another
//!    machine's,
//!    and the lock stays either way.
//!  - a lock made by a process that then became another program:
//!    the same number and the same birth,
//!    alive,
//!    so it stays.
//!  - a holder in another PID namespace or on another machine:
//!    its record judges unknown,
//!    so the lock stays.
//!  - the wrapper's own forwarded command:
//!    its `self` record carries the birth the exec'd Git keeps,
//!    so it stays.
//!  - a zombie,
//!    or a process that ended:
//!    it has no birth to compare,
//!    and it cannot have written anything since,
//!    so the record's judgement decides.
//!
//!  No time decides anything,
//!  so a correction of the clock moves no verdict.
//!  Four further facts must hold before the file goes,
//!  and each one is a way a live writer could still be inside it:
//!  no readable process has it open (a complete scan of this account),
//!  the name still refers to the very file that was opened and judged,
//!  the file system is one of this machine's (a file system another machine writes holds a Git whose
//!  numbers mean nothing here),
//!  and this process holds no descriptor on it but the one it is judging.
//!  Two commands that meet the same abandoned lock cannot both remove it and then both take it:
//!  each takes an exclusive `flock` of the lock file before its last look,
//!  so exactly one of them reaches the removal,
//!  and the one that does removes a file whose name it has just seen again and whose holder it has just proved gone;
//!  taking the lock afterwards is Git's own exclusive creation,
//!  so no second writer can take one that a first has taken.
//!  A lock carrying the commit service's marker is not judged by that proof:
//!  it is handed to the service's own reclaimer,
//!  which reads the marker's holder and takes its claim on the file
//!  (`service_index_reclaim.rs`,
//!  `StaleMark::HolderIsGone`).
//!  Nothing is restored after the removal,
//!  because there is nothing of the repository's to restore:
//!  every Git lock is a new file that Git fills and renames over its target,
//!  so a lock that never reached its rename holds only the dead holder's unfinished copy of the index,
//!  the reference list,
//!  `HEAD` or the shallow file,
//!  and the file that is really there is the one the repository had before.
//!
//! Every name the pass reads may have been made by another program,
//! and each read is opened so that a name of the wrong kind cannot hold it:
//!
//!  - the lock and its PID file:
//!    `symlink_metadata` for their identity and their times,
//!    which opens nothing and cannot wait;
//!  - the lock's bytes:
//!    `open_to_judge`,
//!    with `O_NOFOLLOW`,
//!    `O_NONBLOCK` and `O_CLOEXEC`,
//!    so a link is not followed and a pipe is never waited on;
//!  - the PID file's number:
//!    `plain_file::read_plain_file`,
//!    which refuses anything but a plain file
//!    and bounds what it reads (`stopped_git_locks.rs`,
//!    `pid_named_by`);
//!  - a record's bytes:
//!    `plain_file::read_plain_file`,
//!    the same way;
//!  - the marker inside a lock:
//!    the descriptor the lock was opened with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! clearOne({ lock, commonDir }); // { lock, pid, evidence } | undefined
//! ```

/// The lock's metadata and its identity without following a link.
use crate::index_lock_evidence::read_lock_metadata;
/// The proof that no program can write into a file,
/// from the kernel where it offers leases and from the scan elsewhere.
use crate::merge_isolation_take_asides_proof::{ProofNothingCanWrite, the_proof_that_nothing_can_write_into};
/// The scan of the processes that hold one file open,
/// for the platforms that offer no lease.
#[cfg(not(target_os = "linux"))]
use crate::index_lock_holders::{HolderEvidence, scan_platform_holders_bounded};
/// The one reason a removal may act through:
/// another account owns the process whose descriptors the kernel refused.
#[cfg(not(target_os = "linux"))]
use crate::index_lock_holders_accounts::OTHER_ACCOUNTS_PROCESS_REASON;
/// The proof a record of this repository gives about the writer of a PID file.
use crate::lock_holder_record_reads::proved_gone_writer;
/// How a lock's PID file is read and judged,
/// and the file systems a lock of this machine sits on.
use crate::stale_ref_lock_judge::{Judge, LOCAL_FILE_SYSTEMS};
/// The name a lock's PID file has.
use crate::stale_ref_lock_walk::pid_file_beside;
/// How the file system holding a lock is looked at.
use crate::fs_mount_options::read_mount_facts;
/// The commit service's marker,
/// whose locks belong to the landing's own reclaimer.
use crate::service_index_marker::marker_of_file;
/// The open that never waits,
/// the comparison of one file with the name it was reached by,
/// and the service's reclaimer.
use crate::service_index_reclaim::{Reclaimed, StaleMark, open_to_judge, reclaim_if, same_file};
/// The platform whose scans and placements this process uses.
use crate::service_index_placement::SYSTEM;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The instant after the abandoned lock was unlinked and before its PID file is,
///  where a control acts as a Git that takes the freed name.
pub(crate) const UNLINKED_THE_LOCK: &str = "abandoned-lock-unlinked";

/// How long a command that met another command's removal of the same lock waits for it to finish.
/// Exempt from a citation under `LQM` (class:
/// pacing of a removal):
/// the holder works for the microseconds between its last look and its unlink,
/// and nothing else in the repository ever takes this advisory lock.
const REMOVAL_CLAIM_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// The pause between two attempts at the removal's claim.
const REMOVAL_CLAIM_PAUSE: std::time::Duration = std::time::Duration::from_millis(2);

/// What:
///  One lock this pass removed,
///  and the evidence it acted on.
/// Why:
///  The caller tells the person one line per removal,
///  and the sweep of the records needs the numbers that stood beside a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Clearance = { lock: string; pid: number; evidence: 'pid-file' | 'service-marker' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clearance {
    /// The lock file that was removed.
    pub lock: PathBuf,
    /// The number the removal acted on:
    /// the PID file's,
    /// or the number the service's marker named.
    pub pid: i64,
    /// What named that number,
    ///  which the line for the person says.
    pub evidence: Evidence,
}

/// What:
///  The evidence a removal acted on,
///  in the terms the person reads.
/// Why:
///   A lock's PID file is one fact and the commit service's marker is another:
///  the line must say which of them named the holder,
///  or it describes a file that does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Evidence = 'pid-file' | 'service-marker';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evidence {
    /// The lock's own PID file,
    ///  written because `core.lockfilePid` is on.
    PidFile,
    /// The record a command of this repository wrote about the very lock its own Git took,
    ///  whose pin holds the file that stood here.
    Binding,
    /// The commit service's marker,
    ///  which the service itself wrote into the lock.
    ServiceMarker,
}

/// What:
///  Remove the lock at `lock` when the writer of its PID file is proved gone,
///  and report what was acted on.
/// Why:
///   One function decides,
///  so every command that meets a lock decides by one proof and one sequence,
///  and a control can drive the proof without a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearOne({ lock, commonDir }): Clearance | undefined;
/// ```
pub fn clear_one(lock: &Path, common_dir: &Path) -> Option<Clearance> {
    // Judged by its metadata alone: this process holds no descriptor on the file while the scan looks for a holder, so a
    //  second command's own look is not mistaken for a live writer holding the lock.
    let judged = read_lock_metadata(lock).ok()??;
    if !sits_on_this_machine(lock) {
        return None;
    }
    if let Some(marked) = marker_of(lock) {
        // The service's own reclaimer judges its marker, holds its claim on the file, and removes it only once the holder the
        //  marker names is gone.
        return match reclaim_if(&SYSTEM, lock, StaleMark::HolderIsGone) {
            Reclaimed::Removed => {
                let pid: i64 = match marked.holder {
                    Some((number, _identity)) => number,
                    None => 0,
                };
                Some(Clearance { lock: lock.to_path_buf(), pid, evidence: Evidence::ServiceMarker })
            }
            _ => None,
        };
    }
    // What: The record a command of this repository wrote about this very lock decides first, and the PID file's birth
    //       order decides only when there is none.
    // Why:  A command that kept its process around its Git watched that Git create the lock and wrote the record down with
    //       a pin on the very inode it took (`lock_bindings.rs`, `lock_watch.rs`), so no birth time is needed on that path:
    //       a file system of coarse times gives the lock and its PID file one stamp and proves nothing.
    let pid_file: PathBuf = pid_file_beside(lock);
    let (pid, evidence): (i64, Evidence) = match bound_holder(lock, common_dir, pid_file.as_path()) {
        Some(found) => found,
        None => {
            let born: Option<std::time::SystemTime> = born_of(lock);
            // Only a PID file Git wrote for this very lock names its holder: one born before the lock belongs to an
            // earlier holder.
            let crate::stale_ref_lock_found::PidFile::Names(found) = Judge::pid_file_state(lock, born) else {
                return None;
            };
            // The writer of this PID file must be proved gone. A number a live process has now is no proof of anything.
            proved_gone_writer(common_dir, found)?;
            (found, Evidence::PidFile)
        }
    };
    // Nothing may be able to write into the file when its name goes:
    //  the kernel's read lease is refused while any program has it open for writing,
    //  and the scan of a platform without leases is the whole answer there.
    let proof: ProofNothingCanWrite = nothing_can_write_into(lock, judged.device, judged.inode)?;
    let file: std::fs::File = open_to_judge(lock).ok()?;
    if !claim_the_removal(&file) {
        return None;
    }
    // The last looks before the removal: the name still refers to the very file that was judged, nothing wrote into it while
    //  the removal claimed it, and the PID file still names the number the record explained.
    let after: crate::index_lock_evidence::LockMetadata = read_lock_metadata(lock).ok()??;
    if after.device != judged.device || after.inode != judged.inode || after.ctime_ms != judged.ctime_ms {
        return None;
    }
    if !same_file(lock, &file).unwrap_or(false) || marker_of_file(&file).is_some() {
        return None;
    }
    if crate::stopped_git_locks::pid_named_by(pid_file.as_path()) != Some(i32::try_from(pid).ok()?) {
        return None;
    }
    let pid_file_judged: Option<crate::index_lock_evidence::LockMetadata> = read_lock_metadata(pid_file.as_path()).ok().flatten();
    std::fs::remove_file(lock).ok()?;
    // A control reaches here to act as a Git that takes the freed name at this instant.
    crate::service_fault::reach(UNLINKED_THE_LOCK);
    // The PID file goes with the lock it belongs to, and only when it is still the very file that was judged: a Git that took
    //  the freed name and wrote its own PID file keeps that file, because its device, inode or change time differ.
    if let Some(judged_pid_file) = pid_file_judged
        && matches!(read_lock_metadata(pid_file.as_path()), Ok(Some(still)) if still.device == judged_pid_file.device && still.inode == judged_pid_file.inode && still.ctime_ms == judged_pid_file.ctime_ms)
    {
        let _removed = std::fs::remove_file(pid_file.as_path());
    }
    // The record of this lock goes with it, and so does the pin that held its inode. The two are named by the file
    //  that was judged, not by what the path holds now.
    let recorded: crate::lock_bindings::Written = crate::lock_bindings::Written { lock: lock.to_path_buf(), file: (judged.device, judged.inode) };
    crate::lock_bindings::remove(common_dir, &recorded);
    // The lease, where this platform offers one, is released only now: both names are gone.
    drop(proof);
    return Some(Clearance { lock: lock.to_path_buf(), pid, evidence });
}

/// What:
///  The holder a record of this repository's own command names for this very lock,
///  when that record still describes the file standing here and that holder is proved gone.
/// Why:
///  This is the proof a lock's PID file cannot give on a file system of coarse birth times:
///  the command watched its own Git create the lock and write the PID file after it,
///  pinned the very inode Git took,
///  and named the Git by the birth identity it read of its own child.
///  So each of these must hold,
///  and each one is a way the state is not the one the record describes:
///
///  - the pin holds the file standing at this path now,
///    or the lock was replaced,
///    renamed over its target,
///    or cleared and made again;
///  - the PID file standing beside it still names the number the command recorded;
///  - and that Git is gone,
///    by the birth identity the command read of its own child,
///    so a number a live process has now keeps the lock whatever the record says.
///
///  A record whose holder is not gone,
///  and every part of one that cannot be read,
///  leave the lock to the birth-order gate and the wait it has today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = bindingOf(commonDir, lock);
/// if (!bound || !pinnedIs(lock) || !pidFileIs(bound) || holderState(bound) !== 'gone') return undefined;
/// ```
fn bound_holder(lock: &Path, common_dir: &Path, pid_file: &Path) -> Option<(i64, Evidence)> {
    let bound: crate::lock_bindings::Binding = crate::lock_bindings::of(common_dir, lock)?;
    if crate::stopped_git_locks::pid_named_by(pid_file) != Some(i32::try_from(bound.pid).ok()?) {
        return None;
    }
    if crate::service_index_identity::holder_state(Some(&(bound.pid, bound.identity.clone()))) != crate::service_index_identity::HolderState::Gone {
        return None;
    }
    proved_gone_writer(common_dir, bound.pid)?;
    return Some((bound.pid, Evidence::Binding));
}

/// What:
///  The service's marker in the file at `lock`,
///  read and closed at once,
///  so the judgement holds no descriptor while it looks for a holder.
/// Why:
///  The marker is the one fact that needs the file's bytes before the scan,
///  and a descriptor of this process on the lock would make the scan refuse its own removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marked = markerOf(lock); // opened, read, closed
/// ```
fn marker_of(lock: &Path) -> Option<crate::service_index_marker::Marked> {
    let file: std::fs::File = open_to_judge(lock).ok()?;
    return marker_of_file(&file);
}

/// What:
///  When the file at `path` was created,
///  where this file system records that.
/// Why:
///  Git creates a lock's PID file after the lock itself,
///  so the two times are what tells this lock's holder from an earlier one's,
///  and a time this file system does not record,
///  or one both files share,
///  proves neither order
///  (`lock_birth_times.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(path).birthtimeMs
/// ```
fn born_of(path: &Path) -> Option<std::time::SystemTime> {
    return crate::lock_birth_times::birth_time_of(path);
}

/// What:
///  Whether the file system holding `lock` is one of this machine's.
/// Why:
///  A lock on a network or user-space file system can be written by a Git on another machine,
///  whose process numbers mean nothing here,
///  so nothing is cleared there and the lock is reported as it is today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const local = LOCAL_FILE_SYSTEMS.includes(readMountFacts(path).filesystemType);
/// ```
fn sits_on_this_machine(lock: &Path) -> bool {
    let Ok(Some(facts)) = read_mount_facts(lock) else {
        return false;
    };
    return LOCAL_FILE_SYSTEMS.contains(&facts.filesystem_type.as_str());
}

/// What:
///  Whether this process now holds the exclusive `flock` of the lock file,
///  which is what keeps two commands from removing the same lock.
/// Why:
///  An advisory lock covers the smallest unit that matters here,
/// the one lock file,
///  and it is held for the moment between the last look and the removal.
///  The kernel gives it to one holder;
///  a second command waits for that holder,
/// bounded,
/// instead of running Git into the refusal it was about to remove.
///  When the holder is done the name is gone,
/// so the second command's own last look refuses the removal and it goes on with a
///  clear repository.
///  A file system that has no `flock` gives an answer that is not a lock,
///  which leaves the file alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// flock(fd, LOCK_EX | LOCK_NB) === 0 // retried for at most REMOVAL_CLAIM_WAIT
/// ```
fn claim_the_removal(file: &std::fs::File) -> bool {
    use std::os::fd::AsRawFd as _;
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        // SAFETY: `flock` takes a descriptor of a file this function was given and a flag word, and touches no memory of this
        //  process. The descriptor stays open for as long as the file value lives, which is past the removal.
        let taken: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if taken == 0 {
            return true;
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        // Only a lock another command holds is worth waiting for; a file system without `flock` answers something else.
        if error.raw_os_error() != Some(libc::EWOULDBLOCK) || began.elapsed() >= REMOVAL_CLAIM_WAIT {
            return false;
        }
        std::thread::sleep(REMOVAL_CLAIM_PAUSE);
    }
}

/// What:
///  Whether a program holds the lock open for writing now,
///  which is what keeps a lock whose records prove its writer gone from being removed.
/// Why:
///  The person's line must say what actually stands in the way:
///  a writer the kernel refuses this command's lease is a live program at work,
///  and its own command ends by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aWriterHoldsItOpen(lock) // the lease was refused
/// ```
pub(crate) fn a_writer_holds_it_open(lock: &Path) -> bool {
    let Ok(Some(judged)) = read_lock_metadata(lock) else {
        return false;
    };
    return nothing_can_write_into(lock, judged.device, judged.inode).is_none();
}

/// What:
///  The proof that nothing can write into the lock while its name is removed:
///  the kernel's own answer where it offers leases,
///  and the scan of every readable process where it does not.
/// Why:
///  A scan of `/proc` cannot read the descriptors of a program of this account that is not dumpable,
///  so a program that holds the lock open for writing could write into an inode no name reaches
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its first finding).
///  A read lease is refused while any program has the file open for writing,
///  whatever account owns it and whether or not it is dumpable,
///  and holding it keeps a later writer out until the name is gone
///  (`merge_isolation_take_asides_proof.rs`,
/// the same proof the merge's asides are removed under).
///  A platform without leases has only the scan:
///  a file whose scan found a holder,
///  was cut at its bound,
///  or could not list the processes has no proof and is kept,
///  and another account's refusal is the one reason a removal acts through there,
///  because such a process cannot write into a file of this account's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proof = leaseOrScan(lock, scanned); // undefined keeps the lock
/// ```
fn nothing_can_write_into(lock: &Path, device: u64, inode: u64) -> Option<ProofNothingCanWrite> {
    #[cfg(target_os = "linux")]
    {
        let _ = (device, inode);
        return the_proof_that_nothing_can_write_into(lock, &empty_evidence());
    }
    #[cfg(not(target_os = "linux"))]
    {
        let scanned: HolderEvidence = scan_platform_holders_bounded(lock, device, inode);
        if !scanned.holders.is_empty() || !scanned.unresolvable.is_empty() {
            return None;
        }
        for reason in &scanned.partial {
            if !reason.starts_with(OTHER_ACCOUNTS_PROCESS_REASON) {
                return None;
            }
        }
        return the_proof_that_nothing_can_write_into(lock, &scanned);
    }
}

/// What:
///  A scan's answer with nothing in it,
///  which the kernel's own proof needs and never reads.
/// Why:
///  The lease is the whole proof where it exists,
///  and saying so here is cheaper than a scan of the process table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const empty = { method: 'lease', holders: [], partial: [], unresolvable: [] };
/// ```
#[cfg(target_os = "linux")]
fn empty_evidence() -> crate::index_lock_holders::HolderEvidence {
    return crate::index_lock_holders::HolderEvidence { method: "lease", holders: Vec::new(), partial: Vec::new(), unresolvable: Vec::new() };
}
