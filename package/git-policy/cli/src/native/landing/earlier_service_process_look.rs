//! What:
//!  The look at the running processes that tells whether a commit service of an earlier release may still be running for a
//!  worktree: `/proc/<pid>/cmdline` of every process of the same user, compared with the service's start arguments.
//! Why:
//!  A running service finishes its own batches, and two services must never settle one batch, so the adoption of an earlier
//!  journal (`earlier_service_journal_adopted.rs`) starts only when this look finds none. A look that cannot be made counts as
//!  "may be running": the batches are left, and the adoption is tried again at the next start. The look is bounded in time
//!  (`crate::wait_ceilings::PROCESS_LOOK_CEILING`), because reading another process's command line takes that process's
//!  address-space lock, which a process stuck in the kernel can hold for good
//!  (Linux `mm/memory.c`, `__access_remote_vm`: `mmap_read_lock_killable`, reached from `fs/proc/base.c`, `get_mm_cmdline`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mayRun: boolean = await withDeadline(ceiling, () => scan(procRoot, look)) ?? true
//! ```

use std::path::{Path, PathBuf};
use std::time::Duration;

/// The words that start the service mode after the program name, and the option that names the worktree's instance.
const SERVICE_MODE_WORDS: [&str; 2] = ["service", "run"];

/// The option that names the instance a service is for.
const INSTANCE_OPTION: &str = "--instance";

/// The option that names the folder a service keeps its journal in. Every unit of an earlier release passed it as a word of its
///  own, followed by the folder.
const JOURNAL_OPTION: &str = "--journal";

/// Most bytes of one process's command line that are read;
///  the options of the service are in the first few hundred bytes.
pub const COMMAND_LINE_BYTES: u64 = 65_536;

/// What a look needs to know to recognise a service of one worktree.
#[derive(Clone, Debug)]
pub struct ServiceLook {
    /// The worktree's instance name.
    pub instance: String,
    /// The folders an earlier release's service of the worktree kept its journal in, per instance and shared:
    ///  a process that was started with one of them as its journal is a service of the worktree.
    pub journals: Vec<PathBuf>,
    /// This process, which is never another service.
    pub own_pid: u32,
    /// The user whose processes are looked at.
    pub own_user: u32,
    /// How long the whole look may take before it counts as "may be running".
    pub ceiling: Duration,
}

/// Whether two neighbouring command line words are the service mode words.
fn is_mode_pair(pair: &[&[u8]]) -> bool {
    return pair[0] == SERVICE_MODE_WORDS[0].as_bytes() && pair[1] == SERVICE_MODE_WORDS[1].as_bytes();
}

/// Whether a byte ends a command line word.
fn is_word_end(byte: &u8) -> bool {
    return *byte == 0;
}

/// The bytes of a path, as a command line word spells it.
#[cfg(unix)]
fn path_word(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    return path.as_os_str().as_bytes().to_vec();
}

/// The bytes of a path, as a command line word spells it, for a platform whose paths are not bytes.
#[cfg(not(unix))]
fn path_word(path: &Path) -> Vec<u8> {
    return path.to_string_lossy().into_owned().into_bytes();
}

/// Whether a command line word spells one of the earlier journal folders.
fn is_earlier_journal(journals: &[PathBuf], word: &[u8]) -> bool {
    for journal in journals {
        if path_word(journal.as_path()).as_slice() == word {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether one command line (words split at NUL bytes) is a service of the worktree.
/// Why:
///   Two shapes of start arguments exist. The units written before the service became a mode of the program started it as
///  `<program> --journal <state>/cli-git/service/<instance> --git <git> --instance <instance>`, and the first releases' single
///  service as `<program> --journal <state>/cli-git/service --git <git>`; neither has the mode words. The later units start
///  `<program> cli-git service run --journal <folder> --instance <instance>`. A process keeps the arguments it was started
///  with, so helpers of every shape can be alive. A process with the journal option followed by one of the earlier folders is a
///  service of the worktree; so is one with the mode words and the instance option followed by the instance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isServiceOf = (words: string[], look: ServiceLook) =>
///   words.some((w, i) => w === '--journal' && look.journals.includes(words[i + 1]))
///   || (hasPair(words, 'service', 'run') && hasPair(words, '--instance', look.instance))
/// ```
fn is_service_of(words: &[&[u8]], look: &ServiceLook) -> bool {
    let mode_in_a_row: bool = words.windows(SERVICE_MODE_WORDS.len()).any(is_mode_pair);
    let mut instance_named: bool = false;
    let mut journal_named: bool = false;
    for pair in words.windows(2) {
        if pair[0] == INSTANCE_OPTION.as_bytes() && pair[1] == look.instance.as_bytes() {
            instance_named = true;
        }
        if pair[0] == JOURNAL_OPTION.as_bytes() && is_earlier_journal(look.journals.as_slice(), pair[1]) {
            journal_named = true;
        }
    }
    return journal_named || (mode_in_a_row && instance_named);
}

/// Whether an error says the process ended while it was looked at: its entry is gone (`ENOENT`) or `read` found no process
///  (`ESRCH`). Any other error says nothing about the process.
#[cfg(unix)]
fn process_is_gone(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

/// What:
///  The scan itself: every process of the user other than this one, until one is a service of the worktree.
///  Whether a service may be running: it is, when a process is one, and when a process cannot be looked at for any reason but
///  having ended.
/// Why:
///   Only "the process ended" proves nothing is there; a descriptor shortage, an I/O error or a refused read leaves the
///  question open, and an open question must not start a second service on the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const pid of readdir(procRoot)) { try { if (isServiceOf(read(`${pid}/cmdline`))) return true } catch (e) { if (!gone(e)) return true } }
/// ```
#[cfg(unix)]
fn scan(look: &ServiceLook, proc_root: &Path) -> bool {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let Ok(listing) = std::fs::read_dir(proc_root) else {
        return true;
    };
    for listed in listing {
        let Ok(entry) = listed else {
            return true;
        };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if pid == look.own_pid {
            continue;
        }
        // The owner of a process's folder is the user it runs as; another user's service is not this user's to wait for.
        //  The owner is read before the command line, so a host that hides other users' processes cannot make them "unreadable".
        match entry.metadata() {
            Ok(metadata) if metadata.uid() != look.own_user => continue,
            Ok(_) => {}
            Err(error) if process_is_gone(&error) => continue,
            Err(_) => return true,
        }
        let file = match std::fs::File::open(entry.path().join("cmdline")) {
            Ok(file) => file,
            Err(error) if process_is_gone(&error) => continue,
            Err(_) => return true,
        };
        let mut bytes: Vec<u8> = Vec::new();
        match file.take(COMMAND_LINE_BYTES).read_to_end(&mut bytes) {
            Ok(_) => {}
            Err(error) if process_is_gone(&error) => continue,
            Err(_) => return true,
        }
        let words: Vec<&[u8]> = bytes.split(is_word_end).collect();
        if is_service_of(words.as_slice(), look) {
            return true;
        }
    }
    return false;
}

/// Without `/proc` and Unix users there is no look to make, so a service may be running.
#[cfg(not(unix))]
fn scan(_look: &ServiceLook, _proc_root: &Path) -> bool {
    return true;
}

/// The user this process runs as, whose processes are looked at.
#[cfg(unix)]
pub fn effective_user() -> u32 {
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    return unsafe { libc::geteuid() };
}

/// A platform without Unix users has no process owners to compare, so none is another's.
#[cfg(not(unix))]
pub fn effective_user() -> u32 {
    return 0;
}

/// What:
///  Whether a service of the worktree may be running: a process other than this one has the start arguments of one, or the
///  processes cannot be looked at within the look's ceiling.
/// Why:
///   The scan runs on a thread of its own and is waited for up to the ceiling, because one read of a command line can wait for
///  a process stuck in the kernel. A scan that does not answer in time counts as "may be running"; its thread is left behind (it
///  ends when the kernel lets the read go, and the service has no work for it), and the adoption is tried at the next start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serviceMayRun(look: ServiceLook, procRoot: string): Promise<boolean>;
/// ```
pub fn service_of_instance_may_run(look: ServiceLook, proc_root: &Path) -> bool {
    let ceiling: Duration = look.ceiling;
    let root: PathBuf = proc_root.to_path_buf();
    let (sender, receiver) = std::sync::mpsc::channel::<bool>();
    let started = std::thread::Builder::new().name(String::from("earlier-service-process-look")).spawn(move || {
        let _sent = sender.send(scan(&look, root.as_path()));
    });
    if let Err(error) = started {
        crate::service_error!("cannot start the look at the processes ({error}), so a service of the worktree may be running");
        return true;
    }
    return match receiver.recv_timeout(ceiling) {
        Ok(may_run) => may_run,
        Err(error) => {
            crate::service_error!("the look at the processes gave no answer ({error}), so a service of the worktree may be running");
            true
        }
    };
}
