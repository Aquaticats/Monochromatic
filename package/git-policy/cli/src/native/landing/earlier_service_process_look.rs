//! What:
//!  The look that tells whether the commit service of an earlier release may still run,
//! or may start,
//! for a worktree:
//! the
//!  sockets the earlier units listen on (`/proc/net/unix`),
//! then `/proc/<pid>/cmdline` of every process of the same user,
//!  compared with the service's start arguments.
//! Why:
//!  A running service finishes its own batches,
//! and two services must never settle one batch,
//! so the adoption of an earlier
//!  journal (`earlier_service_journal_adopted.rs`) starts only when this look finds none.
//! The earlier helper holds no lock on
//!  its journal (it creates the folder,
//! binds,
//! recovers and serves;
//! read at tag `monochromatic-git-policy-cli-v0.5.34`,
//!  `service_main.rs`),
//! so there is no exclusion this program could also take;
//! the look is the whole protection,
//! and it errs
//!  toward "may be running".
//! A look that cannot be made counts as "may be running":
//! the batches are left,
//! and the adoption is
//!  tried again at the next start.
//!  - The earlier socket unit (`cli-git@<instance>.socket`,
//!    `ListenStream=%t/cli-git/%i.sock`) starts the helper on the first
//!    connection,
//!    so a helper can appear at any moment after a scan found none.
//!    A listening socket of that name is therefore a
//!    reason to wait,
//!    whoever holds it;
//!    the table of sockets lists only sockets that listen when the flags column says so,
//!    and
//!    the file of a socket whose unit stopped stays behind,
//!    so the table is read,
//!    never the file system.
//!  - A process is a service of the worktree by its start arguments.
//!    A journal argument is compared by the folder it names
//!    (device and inode),
//!    because the same folder has several spellings (a symbolic link such as `/home` for `/var/home`,
//!    a
//!    relative path).
//!    When that folder is not there,
//!    the word is compared by the path it spells,
//!    resolved as far as the file system
//!    knows it,
//!    because an earlier helper of the worktree shows its journal word before it makes the folder;
//!    a folder that is not there
//!    holds no batch of the worktree,
//!    so another program that takes a `--journal` option holds nothing back.
//!    A command line that also names this worktree's instance still holds:
//!    the earlier release's unit started
//!    a helper of one worktree with that instance,
//!    and a link in the journal spelling it was given may have been removed after that start,
//!    while the folder the helper writes is still there.
//!    A spelling that cannot be resolved that far proves nothing,
//!    so it counts as "may be running".
//!  - A command line is read up to a limit;
//!    one that fills it has words that were not looked at,
//!    so it counts as "may be
//!    running".
//!
//!  The look is bounded in time (`crate::wait_ceilings::PROCESS_LOOK_CEILING`),
//!  because reading another process's command line
//!  takes that process's address-space lock,
//!  which a process stuck in the kernel can hold for good
//!  (Linux `mm/memory.c`,
//!  `__access_remote_vm`:
//!  `mmap_read_lock_killable`,
//!  reached from `fs/proc/base.c`,
//!  `get_mm_cmdline`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mayRun: boolean = await withDeadline(ceiling, () => earlierSocketListens(procRoot, look) || scan(procRoot, look)) ?? true
//! ```
//!
//! The answers this look gives are declared in `earlier_service_process_look_answers.rs`,
//! the reading of one command line in `earlier_service_process_start_arguments.rs`,
//! and this file serves both.

/// The words of a process's own command line,
/// which the scan reads.
#[cfg(unix)]
use crate::earlier_service_command_line_read::{
    CommandLine,
    command_line_of,
};
/// The identity of each earlier journal,
/// which a scan that cannot read one stops at.
#[cfg(unix)]
use crate::earlier_service_look_paths::journal_identities;
/// What one process of a command line could not be proved to be,
/// how the scan splits that line into words,
/// and what that line says about its process.
#[cfg(unix)]
use crate::earlier_service_process_look_answers::ServiceFound;
#[cfg(unix)]
use crate::earlier_service_process_start_arguments::{
    JournalNames,
    is_service_of,
    is_word_end,
};
/// The words of a program the look could not rule out.
#[cfg(unix)]
use crate::earlier_service_unsure_program::{
    UnsureProgram,
    printable,
    program_of,
};
use crate::earlier_unix_sockets_that_listen::{
    ListeningSockets,
    earlier_socket_ending,
    listening_sockets_ending_with,
};
use crate::scan_awaited_under_a_ceiling::scan_under_ceiling_to_its_end;
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What:
///  The answers of the look,
/// declared beside this file and served by it,
/// so the paths its callers name stay the ones they named.
pub use crate::earlier_service_process_look_answers::{
    Hold,
    Holding,
    ServiceLook,
};

/// The folder below the user's runtime directory that holds the sockets of the earlier units:
/// the earlier data folder's name
///  (`ListenStream=%t/cli-git/%i.sock`,
/// and `cli-git/service.sock` for the single service of the first releases;
/// tag
///  `monochromatic-git-policy-cli-v0.5.34`,
/// `service_instance.rs`,
/// `SOCKET_DIRECTORY` and `LEGACY_SOCKET_RELATIVE`).
const EARLIER_SOCKET_FOLDER: &str = DATA_FOLDER.earlier;

/// The file name of the socket the single service of the first releases listened on,
/// in the earlier socket folder.
const EARLIER_SHARED_SOCKET_FILE: &str = "service.sock";

/// What:
///  Which socket of the earlier units listens,
/// if one does:
/// this worktree's own first,
/// then the single service's of the first
///  releases.
/// A table of sockets that cannot be read is told apart.
/// Why:
///   A connection to such a socket starts the earlier helper at once,
/// with no process to be seen before,
/// so no look at the
///  processes can prove that none will start.
/// The table lists the sockets that exist now and says which of them listen;
/// the
///  file of a socket whose unit stopped stays in the runtime directory,
/// and a connection to test it would start the helper.
///  The two sockets are told apart because stopping them takes different units,
/// and the shared one serves every worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listeningSocketsEndingWith(procRoot, [instanceSocket]) || listeningSocketsEndingWith(procRoot, [sharedSocket])
/// ```
#[cfg(unix)]
fn earlier_socket_that_listens(
    look: &ServiceLook,
    proc_root: &Path,
) -> Option<Hold> {
    let shared: Vec<u8> =
        format!("/{EARLIER_SOCKET_FOLDER}/{EARLIER_SHARED_SOCKET_FILE}").into_bytes();
    return match listening_sockets_ending_with(
        proc_root,
        &[earlier_socket_ending(look.instance.as_str())],
    ) {
        ListeningSockets::Listens => Some(Hold::InstanceSocketListens),
        ListeningSockets::Unreadable => Some(Hold::CannotLook),
        ListeningSockets::None => match listening_sockets_ending_with(proc_root, &[shared]) {
            ListeningSockets::Listens => Some(Hold::SharedSocketListens),
            ListeningSockets::Unreadable => Some(Hold::CannotLook),
            ListeningSockets::None => None,
        },
    };
}

/// What:
///  The scan itself:
/// a socket that listens where an earlier unit's socket does,
/// then every process of the user other than this
///  one,
/// until one is a service of the worktree.
///  Whether a service may be running or start:
/// it is,
/// when such a socket exists,
/// when a process is one,
/// and when a process cannot
///  be looked at for any reason but having ended.
/// Why:
///   Only "the process ended" proves nothing is there;
/// a descriptor shortage,
/// an I/O error or a refused read leaves the
///  question open,
/// and an open question must not start a second service on the journal.
/// The sockets come first:
/// once none
///  listens,
/// no helper can be started by a connection,
/// so the processes seen afterwards are all there can be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (earlierSocketMayListen(look, procRoot)) return true
/// for (const pid of readdir(procRoot)) { try { if (isServiceOf(read(`${pid}/cmdline`))) return true } catch (e) { if (!gone(e)) return true } }
/// ```
#[cfg(unix)]
fn scan(
    look: &ServiceLook,
    proc_root: &Path,
) -> Option<Holding> {
    if let Some(hold) = earlier_socket_that_listens(look, proc_root) {
        return Some(Holding::from(hold));
    }
    return scan_processes(look, proc_root);
}

/// What:
///  The processes half of `scan`:
/// every process of the user other than this one,
/// until one is a service of the worktree.
/// Why:
///   A caller that only needs to know whether an earlier program is running now (the fold of the earlier data names,
/// which a
///  socket that listens cannot write to) must not be held back by a socket whose unit stays active for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const pid of readdir(procRoot)) { if (isServiceOf(read(`${pid}/cmdline`))) return true }
/// ```
#[cfg(unix)]
fn scan_processes(
    look: &ServiceLook,
    proc_root: &Path,
) -> Option<Holding> {
    // A journal folder that cannot be looked at for a reason other than its absence leaves the question open.
    if journal_identities(look.journals.as_slice()).is_none() {
        return Some(Holding::from(Hold::CannotLook));
    }
    let Ok(listing) = std::fs::read_dir(proc_root) else {
        return Some(Holding::from(Hold::CannotLook));
    };
    for listed in listing {
        let Ok(entry) = listed else {
            return Some(Holding::from(Hold::CannotLook));
        };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if pid == look.own_pid {
            continue;
        }
        let bytes: Vec<u8> = match command_line_of(&entry, pid, look.own_user) {
            CommandLine::Words(bytes) => bytes,
            CommandLine::Skipped => continue,
            CommandLine::Unproven(unreadable) => {
                return Some(Holding {
                    because: Hold::CannotLook,
                    program: None,
                    unreadable: Some(unreadable),
                });
            }
        };
        let process: PathBuf = entry.path();
        let names: JournalNames = JournalNames {
            spellings: look.journals.as_slice(),
            process: process.as_path(),
        };
        // Every word ends with a NUL byte, the last one too: that last NUL ends a word, it does not start an empty one.
        let words: Vec<&[u8]> = bytes
            .strip_suffix(&[0])
            .unwrap_or(bytes.as_slice())
            .split(is_word_end)
            .collect();
        match is_service_of(words.as_slice(), look, &names) {
            Some(ServiceFound::Hold(hold)) => return Some(Holding::from(hold)),
            Some(ServiceFound::Unsure {
                word,
                relative,
                reason,
            }) => {
                let program: UnsureProgram = UnsureProgram {
                    pid,
                    program: program_of(words.first().copied()),
                    journal_word: printable(word.as_slice()),
                    journal_word_is_relative: relative,
                    reason,
                };
                return Some(Holding {
                    because: Hold::HelperUnsure,
                    program: Some(program),
                    unreadable: None,
                });
            }
            None => {}
        }
    }
    return None;
}

/// Without `/proc` and Unix users there is no look to make,
/// so a service may be running.
#[cfg(not(unix))]
fn scan(
    _look: &ServiceLook,
    _proc_root: &Path,
) -> Option<Holding> {
    return Some(Holding::from(Hold::CannotLook));
}

/// Without `/proc` and Unix users there is no look to make,
/// so a service may be running.
#[cfg(not(unix))]
fn scan_processes(
    _look: &ServiceLook,
    _proc_root: &Path,
) -> Option<Holding> {
    return Some(Holding::from(Hold::CannotLook));
}

/// The user this process runs as,
/// whose processes are looked at.
#[cfg(unix)]
pub fn effective_user() -> u32 {
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    return unsafe { libc::geteuid() };
}

/// A platform without Unix users has no process owners to compare,
/// so none is another's.
#[cfg(not(unix))]
pub fn effective_user() -> u32 {
    return 0;
}

/// What:
///  Why a service of the worktree may be running or may start,
/// or nothing when the look can promise neither:
/// a socket of an
///  earlier unit listens,
/// a process other than this one has the start arguments of one,
/// or the sockets or the processes cannot
///  be looked at within the look's ceiling.
/// Why:
///   The scan runs on a thread of its own and is waited for up to the ceiling,
/// because one read of a command line can wait for
///  a process stuck in the kernel.
/// A scan that does not answer in time counts as "cannot look";
/// its thread is left behind (it
///  ends when the kernel lets the read go),
/// and whoever asked is told to try again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function earlierServiceHold(look: ServiceLook, procRoot: string): Promise<Hold | undefined>;
/// ```
pub fn earlier_service_hold(
    look: ServiceLook,
    proc_root: &Path,
) -> Option<Holding> {
    return hold_within_ceiling(look, proc_root, scan);
}

/// What:
///  Why a service of the worktree is running now,
/// or nothing when no process has a service's start arguments:
/// the sockets of the
///  earlier units are not looked at.
/// Why:
///   A socket that listens starts a service only when a program connects to it;
/// for the fold of the earlier data names the
///  question is whether a program that writes them runs,
/// and the sockets of a worktree's earlier units stay listening for good.
///  The same ceiling applies,
/// and a look that cannot be made counts as a hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function earlierServiceProcessHold(look: ServiceLook, procRoot: string): Promise<Hold | undefined>;
/// ```
pub fn earlier_service_process_hold(
    look: ServiceLook,
    proc_root: &Path,
) -> Option<Holding> {
    return hold_within_ceiling(look, proc_root, scan_processes);
}

/// The answer of a look that could not be made:
/// a service of the worktree may be running.
fn cannot_look() -> Option<Holding> {
    return Some(Holding::from(Hold::CannotLook));
}

/// `earlier_service_hold` and `earlier_service_process_hold` with the scan they run on a job of the blocking pool,
/// awaited under the look's ceiling,
/// run to its end on the process's runtime.
fn hold_within_ceiling(
    look: ServiceLook,
    proc_root: &Path,
    scanner: fn(&ServiceLook, &Path) -> Option<Holding>,
) -> Option<Holding> {
    let ceiling: Duration = look.ceiling;
    return scan_under_ceiling_to_its_end(look, ceiling, proc_root, scanner, cannot_look);
}

/// Whether a service of the worktree may be running or may start.
pub fn service_of_instance_may_run(
    look: ServiceLook,
    proc_root: &Path,
) -> bool {
    return earlier_service_hold(look, proc_root).is_some();
}

/// The awaited look is controlled in the test module.
#[cfg(test)]
#[path = "earlier_service_process_look_tests.rs"]
mod tests;
