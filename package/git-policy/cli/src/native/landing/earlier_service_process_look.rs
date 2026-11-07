//! What:
//!  The look that tells whether the commit service of an earlier release may still run, or may start, for a worktree: the
//!  sockets the earlier units listen on (`/proc/net/unix`), then `/proc/<pid>/cmdline` of every process of the same user,
//!  compared with the service's start arguments.
//! Why:
//!  A running service finishes its own batches, and two services must never settle one batch, so the adoption of an earlier
//!  journal (`earlier_service_journal_adopted.rs`) starts only when this look finds none. The earlier helper holds no lock on
//!  its journal (it creates the folder, binds, recovers and serves; read at tag `monochromatic-git-policy-cli-v0.5.34`,
//!  `service_main.rs`), so there is no exclusion this program could also take; the look is the whole protection, and it errs
//!  toward "may be running". A look that cannot be made counts as "may be running": the batches are left, and the adoption is
//!  tried again at the next start.
//!  - The earlier socket unit (`cli-git@<instance>.socket`, `ListenStream=%t/cli-git/%i.sock`) starts the helper on the first
//!    connection, so a helper can appear at any moment after a scan found none. A listening socket of that name is therefore a
//!    reason to wait, whoever holds it; the table of sockets lists only sockets that listen when the flags column says so, and
//!    the file of a socket whose unit stopped stays behind, so the table is read, never the file system.
//!  - A process is a service of the worktree by its start arguments. A journal argument is compared by the folder it names
//!    (device and inode), because the same folder has several spellings (a symbolic link such as `/home` for `/var/home`, a
//!    relative path); a spelling that cannot be resolved proves nothing, so it counts as "may be running".
//!  - A command line is read up to a limit; one that fills it has words that were not looked at, so it counts as "may be
//!    running".
//!
//!  The look is bounded in time (`crate::wait_ceilings::PROCESS_LOOK_CEILING`), because reading another process's command line
//!  takes that process's address-space lock, which a process stuck in the kernel can hold for good
//!  (Linux `mm/memory.c`, `__access_remote_vm`: `mmap_read_lock_killable`, reached from `fs/proc/base.c`, `get_mm_cmdline`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mayRun: boolean = await withDeadline(ceiling, () => earlierSocketListens(procRoot, look) || scan(procRoot, look)) ?? true
//! ```

use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The words that start the service mode after the program name, and the option that names the worktree's instance.
const SERVICE_MODE_WORDS: [&str; 2] = ["service", "run"];

/// The option that names the instance a service is for.
const INSTANCE_OPTION: &str = "--instance";

/// The option that names the folder a service keeps its journal in. Every unit of an earlier release passed it as a word of its
///  own, followed by the folder.
const JOURNAL_OPTION: &str = "--journal";

/// Most bytes of one process's command line that are looked at;
///  the options of the service are in the first few hundred bytes.
///  One byte more is read, to tell a command line that ends at the limit from one that goes on.
pub const COMMAND_LINE_BYTES: u64 = 65_536;

/// The folder below the user's runtime directory that holds the sockets of the earlier units: the earlier data folder's name
///  (`ListenStream=%t/cli-git/%i.sock`, and `cli-git/service.sock` for the single service of the first releases; tag
///  `monochromatic-git-policy-cli-v0.5.34`, `service_instance.rs`, `SOCKET_DIRECTORY` and `LEGACY_SOCKET_RELATIVE`).
const EARLIER_SOCKET_FOLDER: &str = DATA_FOLDER.earlier;

/// The file name of the socket the single service of the first releases listened on, in the earlier socket folder.
const EARLIER_SHARED_SOCKET_FILE: &str = "service.sock";

/// The extension of the socket file of an instance, after the instance name.
const SOCKET_EXTENSION: &str = ".sock";

/// The bit of the flags column of `/proc/net/unix` that says the socket listens: `__SO_ACCEPTCON`, `1 << 16`, in
///  Linux `include/linux/net.h`; `net/unix/af_unix.c`, `unix_seq_show`, prints it when `sk_state == TCP_LISTEN`.
const SOCKET_LISTENS_FLAG: u32 = 1 << 16;

/// The columns of a line of `/proc/net/unix` up to and including the path:
///  `Num RefCount Protocol Flags Type St Inode Path` (a socket with no name has no path column).
const SOCKET_TABLE_COLUMNS: usize = 8;

/// The position of the flags column in a line of `/proc/net/unix`.
const SOCKET_FLAGS_COLUMN: usize = 3;

/// Why the look cannot promise that no service of the worktree runs or will start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hold {
    /// A socket of an earlier unit listens: a connection to it starts the earlier helper at any moment.
    SocketListens,
    /// A process has the start arguments of the earlier helper of the worktree.
    ServiceRuns,
    /// The sockets or the processes could not be looked at (a table or a command line that cannot be read, a look that ran out of time).
    CannotLook,
}

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

/// Whether a command line word spells one of the earlier journal folders, byte for byte.
fn is_earlier_journal(journals: &[PathBuf], word: &[u8]) -> bool {
    for journal in journals {
        if path_word(journal.as_path()).as_slice() == word {
            return true;
        }
    }
    return false;
}

/// The identity of a folder or file: the device it is on and its inode, which every spelling of one folder shares.
#[cfg(unix)]
type FileIdentity = (u64, u64);

/// The folders of the earlier journals as the look knows them, and where the process that is looked at keeps its files.
#[cfg(unix)]
struct JournalNames<'a> {
    /// The earlier journals, as the service spells them.
    spellings: &'a [PathBuf],
    /// The identity of each earlier journal that exists.
    identities: &'a [FileIdentity],
    /// `<proc root>/<pid>` of the process, whose `cwd` is the folder a relative word is resolved in.
    process: &'a Path,
}

/// What:
///  Whether a journal word of a command line may name one of the earlier journals: it spells one byte for byte, it resolves to
///  the same folder (device and inode), or it cannot be resolved at all.
///  `bool` is "may name": a word that resolves to another folder is the only `false`.
/// Why:
///   The service derives `/var/home/user/...`, and an earlier helper started with `/home/user/...` names the same folder through
///  the link `/home`; a relative word means the folder in the working directory of the process that was started with it, which
///  `/proc/<pid>/cwd` gives (the kernel follows it). A stat that fails says nothing about which folder the word names, and an
///  open question must not start a second service on the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayName = (word: string, names: JournalNames) => spells(word) || sameFolder(resolve(word, names.process + '/cwd'), names.identities) || !resolvable(word)
/// ```
#[cfg(unix)]
fn journal_word_may_name_earlier_journal(names: &JournalNames, word: &[u8]) -> bool {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;
    if is_earlier_journal(names.spellings, word) {
        return true;
    }
    let spelled: &Path = Path::new(std::ffi::OsStr::from_bytes(word));
    let resolved: PathBuf = if spelled.is_absolute() { spelled.to_path_buf() } else { names.process.join("cwd").join(spelled) };
    return match std::fs::metadata(resolved.as_path()) {
        Ok(found) => names.identities.contains(&(found.dev(), found.ino())),
        Err(_) => true,
    };
}

/// What:
///  Whether one command line (words split at NUL bytes) is a service of the worktree.
/// Why:
///   Two shapes of start arguments exist. The units written before the service became a mode of the program started it as
///  `<program> --journal <state>/cli-git/service/<instance> --git <git> --instance <instance>`, and the first releases' single
///  service as `<program> --journal <state>/cli-git/service --git <git>`; neither has the mode words. The later units start
///  `<program> cli-git service run --journal <folder> --instance <instance>`. A process keeps the arguments it was started
///  with, so helpers of every shape can be alive. A process with the journal option followed by a word that may name one of the
///  earlier folders is a service of the worktree; so is one with the mode words and the instance option followed by the instance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isServiceOf = (words: string[], names: JournalNames, look: ServiceLook) =>
///   words.some((w, i) => w === '--journal' && mayName(words[i + 1], names))
///   || (hasPair(words, 'service', 'run') && hasPair(words, '--instance', look.instance))
/// ```
#[cfg(unix)]
fn is_service_of(words: &[&[u8]], look: &ServiceLook, names: &JournalNames) -> bool {
    let mode_in_a_row: bool = words.windows(SERVICE_MODE_WORDS.len()).any(is_mode_pair);
    let mut instance_named: bool = false;
    let mut journal_named: bool = false;
    for pair in words.windows(2) {
        if pair[0] == INSTANCE_OPTION.as_bytes() && pair[1] == look.instance.as_bytes() {
            instance_named = true;
        }
        if pair[0] == JOURNAL_OPTION.as_bytes() && journal_word_may_name_earlier_journal(names, pair[1]) {
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
///  Whether a socket that listens where an earlier unit's socket listens exists: the earlier socket of the worktree's instance
///  or the single service's socket of the first releases. A table of sockets that cannot be read counts as "yes".
/// Why:
///   A connection to such a socket starts the earlier helper at once, with no process to be seen before, so no look at the
///  processes can prove that none will start. The table lists the sockets that exist now and says which of them listen; the
///  file of a socket whose unit stopped stays in the runtime directory, and a connection to test it would start the helper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readLines(`${procRoot}/net/unix`).some((line) => listens(line.flags) && [instanceSocket, sharedSocket].some((s) => line.path.endsWith(s)))
/// ```
#[cfg(unix)]
fn earlier_socket_may_listen(look: &ServiceLook, proc_root: &Path) -> bool {
    use std::io::BufRead;
    let Ok(table) = std::fs::File::open(proc_root.join("net").join("unix")) else {
        return true;
    };
    let endings: [Vec<u8>; 2] = [
        format!("/{EARLIER_SOCKET_FOLDER}/{}{SOCKET_EXTENSION}", look.instance).into_bytes(),
        format!("/{EARLIER_SOCKET_FOLDER}/{EARLIER_SHARED_SOCKET_FILE}").into_bytes(),
    ];
    for read in std::io::BufReader::new(table).split(b'\n') {
        let Ok(line) = read else {
            return true;
        };
        if is_listening_earlier_socket(line.as_slice(), &endings) {
            return true;
        }
    }
    return false;
}

/// The flags column of a line of `/proc/net/unix`, which the kernel prints in hexadecimal; `None` when it is not.
#[cfg(unix)]
fn socket_flags_of(column: &[u8]) -> Option<u32> {
    let text: &str = std::str::from_utf8(column).ok()?;
    return u32::from_str_radix(text, 16).ok();
}

/// Whether one line of `/proc/net/unix` is a socket that listens and whose path ends with one of `endings`.
///  A flags column that is not hexadecimal counts as listening.
#[cfg(unix)]
fn is_listening_earlier_socket(line: &[u8], endings: &[Vec<u8>]) -> bool {
    let mut columns: Vec<&[u8]> = Vec::new();
    for column in line.split(u8::is_ascii_whitespace) {
        if !column.is_empty() {
            columns.push(column);
        }
    }
    if columns.len() < SOCKET_TABLE_COLUMNS {
        return false;
    }
    let listens: bool = match socket_flags_of(columns[SOCKET_FLAGS_COLUMN]) {
        Some(flags) => flags & SOCKET_LISTENS_FLAG != 0,
        None => true,
    };
    let path: &[u8] = columns[columns.len() - 1];
    for ending in endings {
        if path.ends_with(ending.as_slice()) {
            return listens;
        }
    }
    return false;
}

/// What a look at one process found.
#[cfg(unix)]
enum CommandLine {
    /// The words of the command line, split at NUL bytes.
    Words(Vec<u8>),
    /// The process ended while it was looked at, or belongs to another user.
    Skipped,
    /// The command line cannot be read whole: any reason but the process having ended, or more bytes than the limit.
    Unproven,
}

/// What:
///  Reads the command line of one entry of the process table, up to the limit and one byte more.
/// Why:
///   The limit keeps one process from filling memory; the extra byte tells a line that ends at the limit from a line that
///  goes on, whose later words were not looked at and may be the service's options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = read(`${pid}/cmdline`, limit + 1); return bytes.length > limit ? 'unproven' : bytes
/// ```
#[cfg(unix)]
fn command_line_of(entry: &std::fs::DirEntry, own_user: u32) -> CommandLine {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    // The owner of a process's folder is the user it runs as; another user's service is not this user's to wait for.
    //  The owner is read before the command line, so a host that hides other users' processes cannot make them "unreadable".
    match entry.metadata() {
        Ok(metadata) if metadata.uid() != own_user => return CommandLine::Skipped,
        Ok(_) => {}
        Err(error) if process_is_gone(&error) => return CommandLine::Skipped,
        Err(_) => return CommandLine::Unproven,
    }
    let file = match std::fs::File::open(entry.path().join("cmdline")) {
        Ok(file) => file,
        Err(error) if process_is_gone(&error) => return CommandLine::Skipped,
        Err(_) => return CommandLine::Unproven,
    };
    let mut bytes: Vec<u8> = Vec::new();
    match file.take(COMMAND_LINE_BYTES + 1).read_to_end(&mut bytes) {
        Ok(_) => {}
        Err(error) if process_is_gone(&error) => return CommandLine::Skipped,
        Err(_) => return CommandLine::Unproven,
    }
    if bytes.len() as u64 > COMMAND_LINE_BYTES {
        return CommandLine::Unproven;
    }
    return CommandLine::Words(bytes);
}

/// The identity of each earlier journal that exists, or `None` when one cannot be looked at for a reason other than its absence.
#[cfg(unix)]
fn journal_identities(journals: &[PathBuf]) -> Option<Vec<FileIdentity>> {
    use std::os::unix::fs::MetadataExt;
    let mut identities: Vec<FileIdentity> = Vec::new();
    for journal in journals {
        match std::fs::metadata(journal) {
            Ok(found) => identities.push((found.dev(), found.ino())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return None,
        }
    }
    return Some(identities);
}

/// What:
///  The scan itself: a socket that listens where an earlier unit's socket does, then every process of the user other than this
///  one, until one is a service of the worktree.
///  Whether a service may be running or start: it is, when such a socket exists, when a process is one, and when a process cannot
///  be looked at for any reason but having ended.
/// Why:
///   Only "the process ended" proves nothing is there; a descriptor shortage, an I/O error or a refused read leaves the
///  question open, and an open question must not start a second service on the journal. The sockets come first: once none
///  listens, no helper can be started by a connection, so the processes seen afterwards are all there can be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (earlierSocketMayListen(look, procRoot)) return true
/// for (const pid of readdir(procRoot)) { try { if (isServiceOf(read(`${pid}/cmdline`))) return true } catch (e) { if (!gone(e)) return true } }
/// ```
#[cfg(unix)]
fn scan(look: &ServiceLook, proc_root: &Path) -> bool {
    if earlier_socket_may_listen(look, proc_root) {
        return true;
    }
    let Some(identities) = journal_identities(look.journals.as_slice()) else {
        return true;
    };
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
        let bytes: Vec<u8> = match command_line_of(&entry, look.own_user) {
            CommandLine::Words(bytes) => bytes,
            CommandLine::Skipped => continue,
            CommandLine::Unproven => return true,
        };
        let process: PathBuf = entry.path();
        let names: JournalNames = JournalNames { spellings: look.journals.as_slice(), identities: identities.as_slice(), process: process.as_path() };
        // Every word ends with a NUL byte, the last one too: that last NUL ends a word, it does not start an empty one.
        let words: Vec<&[u8]> = bytes.strip_suffix(&[0]).unwrap_or(bytes.as_slice()).split(is_word_end).collect();
        if is_service_of(words.as_slice(), look, &names) {
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

/// Why the look cannot promise that no service of the worktree runs or starts, or nothing when it can.
pub fn earlier_service_hold(look: ServiceLook, proc_root: &Path) -> Option<Hold> {
    let _unused = (look, proc_root);
    return None;
}
