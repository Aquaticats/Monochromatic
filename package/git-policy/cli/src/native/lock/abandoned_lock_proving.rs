//! What:
//!  The proof that one Git lock file is abandoned,
//!  and the one removal that acts on it.
//! Why:
//!  A lock whose holder is proven gone protects nothing,
//!  and the command that meets it must go on instead of waiting a minute and giving up.
//!  The proof is the wrapper's own record of what it started (`lock_holder_record_reads.rs`):
//!  the lock's PID file names a number,
//!  a record of this repository names that number with a birth identity,
//!  and this boot, this process table and this machine have no process with that identity.
//!  No time decides anything,
//!  so a correction of the clock moves no verdict.
//!  Four further facts must hold before the file goes,
//!  and each one is a way a live writer could still be inside it:
//!  no readable process of this account has it open,
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
//!  (`service_index_reclaim.rs`, `StaleMark::HolderIsGone`),
//!  so a live service's lock is waited for and a killed one's is removed by its owner.
//!  Nothing is restored after the removal,
//!  because there is nothing of the repository's to restore:
//!  every Git lock is a new file that Git fills and renames over its target,
//!  so a lock that never reached its rename holds only the dead holder's unfinished copy of the index,
//!  the reference list, `HEAD` or the shallow file,
//!  and the file that is really there is the one the repository had before.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! clearOne({ lock, commonDir }); // { lock, pid } | undefined
//! ```

/// The lock's metadata and its identity without following a link.
use crate::index_lock_evidence::read_lock_metadata;
/// The scan of the processes that hold one file open, and what it could not read.
use crate::index_lock_holders::{HolderEvidence, scan_platform_holders_bounded};
/// The proof a record of this repository gives.
use crate::lock_holder_record_reads::{gone_holder_words, proven_gone};
/// How a lock's PID file is read and judged, and the file systems a lock of this machine sits on.
use crate::stale_ref_lock_judge::{Judge, LOCAL_FILE_SYSTEMS};
/// The name a lock's PID file has.
use crate::stale_ref_lock_walk::pid_file_beside;
/// How the file system holding a lock is looked at.
use crate::fs_mount_options::read_mount_facts;
/// The commit service's marker, whose locks belong to the landing's own reclaimer.
use crate::service_index_marker::marker_of_file;
/// The open that never waits, the comparison of one file with the name it was reached by, and the reason a scan could
///  not read a process of this account.
use crate::service_index_reclaim::{open_to_judge, reclaim_if, same_file, Reclaimed, StaleMark};
/// Counts of the descriptors of this process, the words a scan's partial reasons begin with, and the placement platform.
use crate::service_index_unheld_removal::{UNREADABLE_PROCESS_REASON, own_descriptors_of};
/// The platform whose scans and placements this process uses.
use crate::service_index_placement::SYSTEM;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  One lock this pass removed,
///  and the evidence it acted on.
/// Why:
///  The caller tells the person one line per removal,
///  and the sweep of the records needs the numbers that stood beside a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Clearance = { lock: string; pid: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clearance {
    /// The lock file that was removed.
    pub lock: PathBuf,
    /// The number the removal acted on:
    /// the PID file's, or the number the service's marker named.
    pub pid: i64,
}

/// What:
///  Remove the lock at `lock` when its holder is proven gone,
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
            Reclaimed::Removed => Some(Clearance { lock: lock.to_path_buf(), pid: marked.holder.map_or(0, |holder| { return holder.0; }) }),
            _ => None,
        };
    }
    let born: Option<std::time::SystemTime> = std::fs::symlink_metadata(lock).ok().and_then(|metadata| { return metadata.created().ok(); });
    // Only a PID file Git wrote for this very lock names its holder: one born before the lock belongs to an earlier holder.
    let crate::stale_ref_lock_found::PidFile::Names(pid) = Judge::pid_file_state(lock, born) else {
        return None;
    };
    // A record of this repository must name that number, and no process of this boot, table and machine may have its birth.
    proven_gone(common_dir, pid)?;
    if !nothing_holds_it_open(lock, judged.device, judged.inode) {
        return None;
    }
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
    if crate::stopped_git_locks::pid_named_by(pid_file_beside(lock).as_path()) != Some(i32::try_from(pid).ok()?) {
        return None;
    }
    std::fs::remove_file(lock).ok()?;
    // The PID file goes with the lock it belongs to: Git writes it beside the lock and removes both together.
    let _removed = std::fs::remove_file(pid_file_beside(lock));
    return Some(Clearance { lock: lock.to_path_buf(), pid });
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

/// How long a command that met another command's removal of the same lock waits for it to finish.
/// Exempt from a citation under `LQM` (class: pacing of a removal):
/// the holder works for the microseconds between its last look and its unlink,
/// and nothing else in the repository ever takes this advisory lock.
const REMOVAL_CLAIM_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// The pause between two attempts at the removal's claim.
const REMOVAL_CLAIM_PAUSE: std::time::Duration = std::time::Duration::from_millis(2);

/// What:
///  Whether this process now holds the exclusive `flock` of the lock file,
///  which is what keeps two commands from removing the same lock.
/// Why:
///  An advisory lock covers the smallest unit that matters here, the one lock file,
///  and it is held for the moment between the last look and the removal.
///  The kernel gives it to one holder;
///  a second command waits for that holder, bounded, instead of running Git into the refusal it was about to remove.
///  When the holder is done the name is gone, so the second command's own last look refuses the removal and it goes on with a
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
///  Whether every readable process of this account was looked at and none has the lock open.
/// Why:
///  A process holding the file open is a live writer whatever the record says,
///  a descriptor the kernel refused to copy is a process that may hold it,
///  and a scan cut at its own bound did not look at every process.
///  A process of another account cannot write into this account's file,
///  so the reason the scan gives for one is not a holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scan = scanPlatformHolders(lock, device, inode);
/// !scan.holders.length && !scan.unresolvable.length && scan.partial.every((why) => why.startsWith('PID '))
/// ```
fn nothing_holds_it_open(lock: &Path, device: u64, inode: u64) -> bool {
    let scanned: HolderEvidence = scan_platform_holders_bounded(lock, device, inode);
    if !scanned.holders.is_empty() || !scanned.unresolvable.is_empty() {
        return false;
    }
    for reason in &scanned.partial {
        if !reason.starts_with(UNREADABLE_PROCESS_REASON) {
            return false;
        }
    }
    // The scan leaves this process out, and this process holds no descriptor on the lock at all: it judges by metadata, so a
    //  second command's own look at the same lock cannot make the first refuse a removal both are entitled to make.
    return own_descriptors_of(device, inode) == 0;
}

/// What:
///  The line the person reads when a lock of a dead holder was cleared.
/// Why:
///  One line says which lock went,
///  what proved its holder gone,
///  and what became of the half-written file,
///  so the person needs no other command to understand what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: cleared ${lock}, whose holder is gone ...`
/// ```
pub fn clearance_line(clearance: &Clearance) -> String {
    return format!(
        "monochromatic-git-policy-cli: cleared {}, which a Git that no longer runs left: its PID file names {}, and this repository's own record of the processes its commands started names that same process. A Git fills a lock and renames it over its target, so the file it never renamed held only that Git's unfinished copy of the index, the reference list, HEAD or the shallow file, and the repository has the file it had before.\n",
        clearance.lock.display(),
        gone_holder_words(clearance.pid)
    );
}
/// What:
///  The line for a lock that stands and cannot be proven abandoned,
///  when the PID file beside it names a number no process of this table has
///  and no record of this repository's commands explains it.
/// Why:
///  That is the state a killed command leaves and a record the wrapper did not write,
///  so the person meets a lock that cannot release itself and needs the lock's name,
///  what is known about its holder,
///  and the one action that clears it.
///  A live holder, whose number a process has, is not told about:
///  its own command will report what it does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unexplainedDeadHolderLine(lock, commonDir) // '...names PID 42, which no process...' | undefined
/// ```
pub fn unexplained_dead_holder_line(lock: &Path, common_dir: &Path) -> Option<String> {
    let born: Option<std::time::SystemTime> = std::fs::symlink_metadata(lock).ok().and_then(|metadata| { return metadata.created().ok(); });
    let crate::stale_ref_lock_found::PidFile::Names(pid) = Judge::pid_file_state(lock, born) else {
        return None;
    };
    if !matches!(crate::process_start::resolve_process_start(pid), crate::process_start::ProcessStart::Missing) {
        return None;
    }
    if crate::lock_holder_record_reads::recorded_identity(common_dir, pid).is_some() {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli: {}, whose PID file names {pid}, which no process in this process table has, and no record of the processes this repository's own commands started names that number, so nothing here proves the holder gone and nothing was removed. A Git in another PID namespace, or one on another machine that shares this repository, writes a number this table has never had. If no program is using this repository, the lock was left by a program that was killed or has ended, and removing the file {} lets the command run.\n",
        lock.display(),
        lock.display()
    ));
}
