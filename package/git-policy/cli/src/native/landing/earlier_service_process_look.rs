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

use crate::scan_awaited_under_a_ceiling::scan_under_ceiling_to_its_end;
use crate::earlier_service_unsure_program::UnsureProgram;
#[cfg(unix)]
use crate::earlier_service_unsure_program::{printable, program_of};
use crate::earlier_unix_sockets_that_listen::{ListeningSockets, earlier_socket_ending, listening_sockets_ending_with};
use crate::wrapper_names_old_and_new::{COMMAND_WORD, DATA_FOLDER};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The words that start the service mode after the program name,
/// and the option that names the worktree's instance.
const SERVICE_MODE_WORDS: [&str; 2] = ["service", "run"];

/// The option that names the instance a service is for.
const INSTANCE_OPTION: &str = "--instance";

/// The option that names the folder a service keeps its journal in.
/// Every unit of an earlier release passed it as a word of its
///  own,
/// followed by the folder.
const JOURNAL_OPTION: &str = "--journal";

/// Most bytes of one process's command line that are looked at;
///  the options of the service are in the first few hundred bytes.
///  One byte more is read,
/// to tell a command line that ends at the limit from one that goes on.
pub const COMMAND_LINE_BYTES: u64 = 65_536;

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

/// Why the look cannot promise that no service of the worktree runs or will start,
/// and which one it is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hold {
    /// The earlier release's socket of this worktree (`cli-git/<instance>.sock`) listens:
    /// a connection to it starts the helper.
    InstanceSocketListens,
    /// The socket of the single service of the first releases (`cli-git/service.sock`) listens;
    /// that service serves every worktree.
    SharedSocketListens,
    /// A process runs as the earlier helper of this worktree:
    /// its journal is this worktree's,
    /// or its mode words and instance are.
    InstanceHelperRuns,
    /// A process runs as the single service of the first releases:
    /// its journal is the shared one;
    /// it serves every worktree.
    SharedHelperRuns,
    /// A process has a journal option whose folder could not be resolved,
    /// so it may be an earlier helper of either kind.
    HelperUnsure,
    /// The sockets or the processes could not be looked at (a table or a command line that cannot be read,
    /// a look that ran out of time).
    CannotLook,
}

impl Hold {
    /// The reason in plain words,
    /// as a sentence part that follows "because".
    pub fn words(self) -> &'static str {
        return match self {
            Hold::InstanceSocketListens => "the socket of the earlier release's commit service for this worktree listens, and a connection to it starts that service at any moment",
            Hold::SharedSocketListens => "the socket of the single commit service of the first releases listens (it serves every worktree), and a connection to it starts that service at any moment",
            Hold::InstanceHelperRuns => "a program of the earlier release is running as the commit service of this worktree, and it finishes its own batches",
            Hold::SharedHelperRuns => "a program of the first releases is running as the single commit service of every worktree, and it finishes its own batches",
            Hold::HelperUnsure => "a running program has the earlier commit service's journal option with a folder this check could not resolve, so it may be that service",
            Hold::CannotLook => "the sockets and processes of the earlier helper could not be looked at in time, so it is not known whether one runs",
        };
    }
}

/// What a look found:
/// why the worktree's earlier service may be running or may start,
/// and,
/// when a program that cannot be ruled
///  out caused it,
/// which program that is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Holding {
    /// The reason,
    /// which tells the person what to stop.
    pub because: Hold,
    /// The program behind [`Hold::HelperUnsure`];
    /// nothing for every other reason.
    pub program: Option<UnsureProgram>,
}

impl From<Hold> for Holding {
    fn from(because: Hold) -> Self {
        return Holding { because, program: None };
    }
}

/// What the start arguments of one process say about it.
#[cfg(unix)]
enum ServiceFound {
    /// It is a service of the worktree,
    /// of the kind the reason names.
    Hold(Hold),
    /// A journal word could not be resolved,
    /// so it may be one:
    /// the word,
    /// whether it is relative,
    /// and the system's answer.
    Unsure { word: Vec<u8>, relative: bool, reason: String },
}

/// What a look needs to know to recognise a service of one worktree.
#[derive(Clone, Debug)]
pub struct ServiceLook {
    /// The worktree's instance name.
    pub instance: String,
    /// The folders an earlier release's service of the worktree kept its journal in:
    /// the instance's own first,
    /// the shared one
    ///  second.
    /// A process that was started with one of them as its journal is a service of the worktree,
    /// and which one it is
    ///  tells the person what to stop.
    pub journals: Vec<PathBuf>,
    /// This process,
    /// which is never another service.
    pub own_pid: u32,
    /// The user whose processes are looked at.
    pub own_user: u32,
    /// How long the whole look may take before it counts as "may be running".
    pub ceiling: Duration,
}

/// What:
///  Whether the service mode words start at position `at` of the command line and are not behind the current command word.
/// Why:
///   The service of this release starts as `<program> monochromatic-git-policy-cli service run ...`,
/// with the same mode words
///  and the same instance option as the earlier units that name the earlier command word,
/// and it is no earlier helper:
/// it
///  writes its own journal,
/// and it is the service a person usually has running when they ask for the earlier batches to be
///  settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEarlierModeAt = (words: string[], at: number) => words[at] === 'service' && words[at + 1] === 'run' && words[at - 1] !== 'monochromatic-git-policy-cli'
/// ```
fn is_earlier_mode_at(words: &[&[u8]], at: usize) -> bool {
    let Some(pair) = words.get(at..at + SERVICE_MODE_WORDS.len()) else {
        return false;
    };
    let behind_current_word: bool = at > 0 && words[at - 1] == COMMAND_WORD.current.as_bytes();
    return pair[0] == SERVICE_MODE_WORDS[0].as_bytes() && pair[1] == SERVICE_MODE_WORDS[1].as_bytes() && !behind_current_word;
}

/// Whether a byte ends a command line word.
fn is_word_end(byte: &u8) -> bool {
    return *byte == 0;
}

/// The bytes of a path,
/// as a command line word spells it.
#[cfg(unix)]
fn path_word(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    return path.as_os_str().as_bytes().to_vec();
}

/// The bytes of a path,
/// as a command line word spells it,
/// for a platform whose paths are not bytes.
#[cfg(not(unix))]
fn path_word(path: &Path) -> Vec<u8> {
    return path.to_string_lossy().into_owned().into_bytes();
}

/// The identity of a folder or file:
/// the device it is on and its inode,
/// which every spelling of one folder shares.
#[cfg(unix)]
type FileIdentity = (u64, u64);

/// The folders of the earlier journals as the look knows them,
/// and where the process that is looked at keeps its files.
#[cfg(unix)]
struct JournalNames<'a> {
    /// The earlier journals,
    /// as the service spells them.
    spellings: &'a [PathBuf],
    /// `<proc root>/<pid>` of the process,
    /// whose `cwd` is the folder a relative word is resolved in.
    process: &'a Path,
}

/// The hold a helper that runs on the journal spelled at `index` makes:
/// the shared journal is the second spelling.
#[cfg(unix)]
fn hold_of_journal_at(index: usize) -> Hold {
    return if index == 1 { Hold::SharedHelperRuns } else { Hold::InstanceHelperRuns };
}

/// What:
///  The path a journal word names,
/// with the longest part of it that exists resolved and the names below that part as the word spells them.
/// Why:
///   A journal folder that the process has not made yet has no device and inode to compare,
///  and the same folder is spelled
///  differently by the process that runs this command and by the unit that started an earlier helper
///  (`/home/user/...` through the link `/home`,
///  `/var/home/user/...`).
/// The longest existing part is what the
///  file system can answer for,
/// so resolving it compares the two spellings,
/// and a name below it that does not exist yet is the
///  same name in both.
/// A word that reaches past a name the file system will not look through resolves to nothing,
/// and a
///  word that resolves to nothing proves nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolvedSoFar = (word: string): string => { let at = word; const below: string[] = []; for (;;) { try { return join(realpathSync(at), ...below) } catch (e) { if (e.code !== 'ENOENT') throw e; below.unshift(basename(at)); at = dirname(at) } } }
/// ```
#[cfg(unix)]
fn resolved_as_far_as_the_file_system_knows(word: &Path) -> std::io::Result<PathBuf> {
    let mut below: Vec<&std::ffi::OsStr> = Vec::new();
    let mut at: &Path = word;
    loop {
        match std::fs::canonicalize(at) {
            Ok(known) => {
                let mut path: PathBuf = known;
                for name in below.iter().rev() {
                    path.push(*name);
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // A path whose last name is `..` has no name to carry down, and where it points is not known here.
                let (Some(name), Some(parent)) = (at.file_name(), at.parent()) else {
                    return Err(error);
                };
                below.push(name);
                at = parent;
            }
            Err(error) => return Err(error),
        }
    }
}

/// What:
///  Which earlier journal a journal word of a command line names:
/// the instance's own (the first of the look's journals),
/// the
///  shared one (the second),
/// or one that cannot be resolved;
/// nothing when it names another folder.
/// Why:
///   The word is compared as it is spelled,
/// then by the folder it resolves to (device and inode):
/// the service derives
///  `/var/home/user/...`,
/// and an earlier helper started with `/home/user/...` names the same folder through the link `/home`;
/// a
///  relative word means the folder in the working directory of the process that was started with it,
/// which `/proc/<pid>/cwd`
///  gives (the kernel follows it).
/// A stat that fails for a reason other than the folder being absent says nothing about which folder the word names,
///  and neither does a relative word whose working directory cannot be read:
/// an open question must
///  not start a second service on the journal.
/// An absolute word whose folder is not there is the one case that is settled without the folder:
/// it is
///  compared by the path it spells,
/// and a folder that is not there holds no batch of this worktree,
/// so a
///  program that names one settles nothing here.
/// Which journal it is decides what the person is told to stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalNamed = (word: string, names: JournalNames) => spelled(word) ?? sameFolderAs(resolve(word, names.process + '/cwd')) ?? (absent(word) ? samePathAs(resolvedSoFar(word)) : undefined) ?? (resolvable(word) ? undefined : unsure(word, whyNot(word)))
/// ```
#[cfg(unix)]
fn journal_word_names(names: &JournalNames, word: &[u8]) -> Option<ServiceFound> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;
    for (index, journal) in names.spellings.iter().enumerate() {
        if path_word(journal.as_path()).as_slice() == word {
            return Some(ServiceFound::Hold(hold_of_journal_at(index)));
        }
    }
    let spelled: &Path = Path::new(std::ffi::OsStr::from_bytes(word));
    let absolute: bool = spelled.is_absolute();
    let resolved: PathBuf = if absolute { spelled.to_path_buf() } else { names.process.join("cwd").join(spelled) };
    let found = match std::fs::metadata(resolved.as_path()) {
        Ok(found) => found,
        // An absolute word whose folder is not there names a folder that holds no batch now,
        // and a starting earlier helper of the worktree is in that state until it makes its folder:
        // the path it spells says whether it is one of this worktree's journals.
        Err(error) if absolute && error.kind() == std::io::ErrorKind::NotFound => {
            let named: PathBuf = match resolved_as_far_as_the_file_system_knows(spelled) {
                Ok(named) => named,
                Err(unknown) => return Some(ServiceFound::Unsure { word: word.to_vec(), relative: false, reason: unknown.to_string() }),
            };
            for (index, journal) in names.spellings.iter().enumerate() {
                if journal.is_absolute() && matches!(resolved_as_far_as_the_file_system_knows(journal), Ok(known) if known == named) {
                    return Some(ServiceFound::Hold(hold_of_journal_at(index)));
                }
            }
            return None;
        }
        Err(error) => return Some(ServiceFound::Unsure { word: word.to_vec(), relative: !absolute, reason: error.to_string() }),
    };
    for (index, journal) in names.spellings.iter().enumerate() {
        if matches!(std::fs::metadata(journal), Ok(known) if (known.dev(), known.ino()) == (found.dev(), found.ino())) {
            return Some(ServiceFound::Hold(hold_of_journal_at(index)));
        }
    }
    return None;
}

/// Whether the earlier release's mode words start at any place of `words`.
#[cfg(unix)]
fn has_earlier_mode(words: &[&[u8]]) -> bool {
    for at in 0..words.len() {
        if is_earlier_mode_at(words, at) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether one command line (words split at NUL bytes) is a service of the worktree.
/// Why:
///   Two shapes of start arguments exist.
/// The units written before the service became a mode of the program started it as
///  `<program> --journal <state>/cli-git/service/<instance> --git <git> --instance <instance>`,
/// and the first releases' single
///  service as `<program> --journal <state>/cli-git/service --git <git>`;
/// neither has the mode words.
/// The later units start
///  `<program> cli-git service run --journal <folder> --instance <instance>`.
/// A process keeps the arguments it was started
///  with,
/// so helpers of every shape can be alive.
/// A process with the journal option followed by a word that may name one of the
///  earlier folders is a service of the worktree;
/// so is one with the mode words and the instance option followed by the instance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isServiceOf = (words: string[], names: JournalNames, look: ServiceLook) =>
///   words.some((w, i) => w === '--journal' && mayName(words[i + 1], names))
///   || (hasPair(words, 'service', 'run') && hasPair(words, '--instance', look.instance))
/// ```
#[cfg(unix)]
fn is_service_of(words: &[&[u8]], look: &ServiceLook, names: &JournalNames) -> Option<ServiceFound> {
    let mode_in_a_row: bool = has_earlier_mode(words);
    let mut instance_named: bool = false;
    let mut journal_named: Option<ServiceFound> = None;
    for pair in words.windows(2) {
        if pair[0] == INSTANCE_OPTION.as_bytes() && pair[1] == look.instance.as_bytes() {
            instance_named = true;
        }
        if pair[0] == JOURNAL_OPTION.as_bytes()
            && journal_named.is_none()
            && let Some(named) = journal_word_names(names, pair[1])
        {
            journal_named = Some(named);
        }
    }
    if journal_named.is_some() {
        return journal_named;
    }
    return if mode_in_a_row && instance_named { Some(ServiceFound::Hold(Hold::InstanceHelperRuns)) } else { None };
}

/// Whether an error says the process ended while it was looked at:
/// its entry is gone (`ENOENT`) or `read` found no process
///  (`ESRCH`).
/// Any other error says nothing about the process.
#[cfg(unix)]
fn process_is_gone(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

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
fn earlier_socket_that_listens(look: &ServiceLook, proc_root: &Path) -> Option<Hold> {
    let shared: Vec<u8> = format!("/{EARLIER_SOCKET_FOLDER}/{EARLIER_SHARED_SOCKET_FILE}").into_bytes();
    return match listening_sockets_ending_with(proc_root, &[earlier_socket_ending(look.instance.as_str())]) {
        ListeningSockets::Listens => Some(Hold::InstanceSocketListens),
        ListeningSockets::Unreadable => Some(Hold::CannotLook),
        ListeningSockets::None => match listening_sockets_ending_with(proc_root, &[shared]) {
            ListeningSockets::Listens => Some(Hold::SharedSocketListens),
            ListeningSockets::Unreadable => Some(Hold::CannotLook),
            ListeningSockets::None => None,
        },
    };
}

/// What a look at one process found.
#[cfg(unix)]
enum CommandLine {
    /// The words of the command line,
    /// split at NUL bytes.
    Words(Vec<u8>),
    /// The process ended while it was looked at,
    /// or belongs to another user.
    Skipped,
    /// The command line cannot be read whole:
    /// any reason but the process having ended,
    /// or more bytes than the limit.
    Unproven,
}

/// What:
///  Reads the command line of one entry of the process table,
/// up to the limit and one byte more.
/// Why:
///   The limit keeps one process from filling memory;
/// the extra byte tells a line that ends at the limit from a line that
///  goes on,
/// whose later words were not looked at and may be the service's options.
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

/// The identity of each earlier journal that exists,
/// or `None` when one cannot be looked at for a reason other than its absence.
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
fn scan(look: &ServiceLook, proc_root: &Path) -> Option<Holding> {
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
fn scan_processes(look: &ServiceLook, proc_root: &Path) -> Option<Holding> {
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
        let bytes: Vec<u8> = match command_line_of(&entry, look.own_user) {
            CommandLine::Words(bytes) => bytes,
            CommandLine::Skipped => continue,
            CommandLine::Unproven => return Some(Holding::from(Hold::CannotLook)),
        };
        let process: PathBuf = entry.path();
        let names: JournalNames = JournalNames { spellings: look.journals.as_slice(), process: process.as_path() };
        // Every word ends with a NUL byte, the last one too: that last NUL ends a word, it does not start an empty one.
        let words: Vec<&[u8]> = bytes.strip_suffix(&[0]).unwrap_or(bytes.as_slice()).split(is_word_end).collect();
        match is_service_of(words.as_slice(), look, &names) {
            Some(ServiceFound::Hold(hold)) => return Some(Holding::from(hold)),
            Some(ServiceFound::Unsure { word, relative, reason }) => {
                let program: UnsureProgram = UnsureProgram {
                    pid,
                    program: program_of(words.first().copied()),
                    journal_word: printable(word.as_slice()),
                    journal_word_is_relative: relative,
                    reason,
                };
                return Some(Holding { because: Hold::HelperUnsure, program: Some(program) });
            }
            None => {}
        }
    }
    return None;
}

/// Without `/proc` and Unix users there is no look to make,
/// so a service may be running.
#[cfg(not(unix))]
fn scan(_look: &ServiceLook, _proc_root: &Path) -> Option<Holding> {
    return Some(Holding::from(Hold::CannotLook));
}

/// Without `/proc` and Unix users there is no look to make,
/// so a service may be running.
#[cfg(not(unix))]
fn scan_processes(_look: &ServiceLook, _proc_root: &Path) -> Option<Holding> {
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
pub fn earlier_service_hold(look: ServiceLook, proc_root: &Path) -> Option<Holding> {
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
pub fn earlier_service_process_hold(look: ServiceLook, proc_root: &Path) -> Option<Holding> {
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
fn hold_within_ceiling(look: ServiceLook, proc_root: &Path, scanner: fn(&ServiceLook, &Path) -> Option<Holding>) -> Option<Holding> {
    let ceiling: Duration = look.ceiling;
    return scan_under_ceiling_to_its_end(look, ceiling, proc_root, scanner, cannot_look);
}

/// Whether a service of the worktree may be running or may start.
pub fn service_of_instance_may_run(look: ServiceLook, proc_root: &Path) -> bool {
    return earlier_service_hold(look, proc_root).is_some();
}

/// The awaited look is controlled in the test module.
#[cfg(test)]
#[path = "earlier_service_process_look_tests.rs"]
mod tests;
