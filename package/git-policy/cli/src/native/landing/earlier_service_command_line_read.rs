//! What:
//!  What one process of the process table says about itself:
//!  the command line the look at an earlier commit service reads,
//! up to the largest line any program
//!  can carry,
//! and the report of a line that cannot be proved.
//! Why:
//!  The look is the whole protection against two services settling one batch
//! (`earlier_service_process_look.rs`),
//! and the
//!  only thing it goes by is the start arguments a process keeps.
//! A line that cannot be read is not "no service":
//!  it is a
//!  question left open,
//! so it comes back with the process that raised it and what the kernel answered,
//! which
//!  is what a person can act on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = commandLineOf(entry, pid, ownUser); // {words} | {skipped} | {unproven: {pid, program, reason}}
//! ```

#[cfg(unix)]
use crate::earlier_service_unsure_program::printable;
#[cfg(unix)]
use std::path::Path;

/// The largest number of bytes a program's arguments and environment may take together:
/// `_STK_LIM / 4 x 3` of Linux `fs/exec.c`,
/// `bprm_stack_limits`,
/// with `_STK_LIM` at 8 MiB.
#[cfg(unix)]
const KERNEL_ARGUMENT_SPACE_CEILING_BYTES: u64 = 6 * 1024 * 1024;

/// Most bytes of one process's command line that are looked at:
/// the largest a command line can be on Linux,
///  so a line that fills it is not one any program can carry.
///  One byte more is read,
/// to tell a command line that ends at the limit from one that goes on.
/// Why the limit is that size:
///   The kernel refuses to start a program whose arguments and environment together are more than
/// `_STK_LIM / 4 * 3`,
///  8 MiB / 4 x 3 = 6 MiB,
/// (`fs/exec.c`,
/// `bprm_stack_limits`),
/// and one argument is at most
///  32 pages,
/// 128 KiB,
/// (`MAX_ARG_STRLEN` in `include/uapi/linux/binfmts.h`),
///  so no command line is longer.
///  Measured on this host on 2026-11-10:
/// with the stack limit lifted (`ulimit -s unlimited`,
/// `getconf ARG_MAX` answering 6,291,456) a program started with 5 MiB of arguments and the kernel refused 6 MiB;
/// with the default 8 MiB stack limit the kernel refused 2 MiB (`getconf ARG_MAX` answering 2,097,152).
/// The limit this replaces was 65,536 bytes,
/// which ordinary programs of the user pass:
/// the longest command line on this host then was 44,174 bytes (a program started by another session,
/// read from `/proc/<pid>/cmdline`).
///  A shorter limit does not protect memory:
/// the buffer grows to what one process carries
///  (measured:
/// `take(6 MiB + 1).read_to_end` of a 12-byte line left the buffer at 44 bytes)
/// and is dropped
///  before the next one is read.
#[cfg(unix)]
pub const COMMAND_LINE_BYTES: u64 = KERNEL_ARGUMENT_SPACE_CEILING_BYTES;

/// A process whose command line the look could not prove.
/// Why:
///  Such a process cannot be ruled out as the earlier commit service of the worktree,
///  so the look holds;
/// the person can act on the hold only when it names the process and says what the kernel answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnreadableProcess {
    /// The process.
    pub pid: u32,
    /// The name the kernel keeps for its program,
    /// as `/proc/<pid>/comm` spells it (escaped when it holds a control character),
    /// or what to say when even that cannot be read.
    pub program: String,
    /// What the kernel answered,
    /// or what about the line itself proves nothing.
    pub reason: String,
}

/// What a look at one process found.
#[cfg(unix)]
pub enum CommandLine {
    /// The words of the command line,
    /// split at NUL bytes.
    Words(Vec<u8>),
    /// The process ended while it was looked at,
    /// or belongs to another user.
    Skipped,
    /// The command line cannot be read whole:
    /// any reason but the process having ended,
    /// or more bytes than any program can carry.
    Unproven(UnreadableProcess),
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
///  The name the kernel keeps for a process's program,
/// escaped so a program that set a name with a control character cannot carry it into a message.
/// Why:
///   A person blocked by a process has to recognise it,
/// and the name is all such a process shows
///  when its command line may not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = kernelNameOf(`${procRoot}/${pid}`); // escaped, or what to say when it cannot be read
/// ```
#[cfg(unix)]
fn kernel_name_of(process: &Path) -> String {
    const UNREADABLE: &str = "a program whose name could not be read";
    let Ok(bytes) = std::fs::read(process.join("comm")) else {
        return String::from(UNREADABLE);
    };
    let trimmed: &[u8] = bytes.strip_suffix(b"\n").unwrap_or(bytes.as_slice());
    if trimmed.is_empty() {
        return String::from(UNREADABLE);
    }
    return printable(trimmed);
}

/// What:
///  A command line that could not be proved,
/// naming the process and what the kernel answered.
/// Why:
///   Every branch of the read reports in the same shape,
/// so the caller cannot report one of them without the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unproven = (reason: string) => ({ unproven: { pid, program: kernelNameOf(folder), reason } });
/// ```
#[cfg(unix)]
fn unproven_of(entry: &std::fs::DirEntry, pid: u32, reason: String) -> CommandLine {
    return CommandLine::Unproven(UnreadableProcess { pid, program: kernel_name_of(entry.path().as_path()), reason });
}

/// What:
///  Reads the command line of one entry of the process table,
/// up to the limit and one byte more,
/// and names the process when it cannot be proved.
/// Why:
///   The limit keeps one process from filling memory;
/// the extra byte tells a line that ends at the limit from a line that
///  goes on,
/// whose later words were not looked at and may be the service's options.
///  The limit is larger than any command line the kernel allows,
/// so a line that fills it is no ordinary program's and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = read(`${pid}/cmdline`, limit + 1); return bytes.length > limit ? { unreadable: { pid, reason } } : bytes
/// ```
#[cfg(unix)]
pub fn command_line_of(entry: &std::fs::DirEntry, pid: u32, own_user: u32) -> CommandLine {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    // The owner of a process's folder is the user it runs as; another user's service is not this user's to wait for.
    //  The owner is read before the command line, so a host that hides other users' processes cannot make them "unreadable".
    match entry.metadata() {
        Ok(metadata) if metadata.uid() != own_user => return CommandLine::Skipped,
        Ok(_) => {}
        Err(error) if process_is_gone(&error) => return CommandLine::Skipped,
        Err(error) => return unproven_of(entry, pid, format!("the kernel would not describe the process ({error})")),
    }
    let file = match std::fs::File::open(entry.path().join("cmdline")) {
        Ok(file) => file,
        Err(error) if process_is_gone(&error) => return CommandLine::Skipped,
        Err(error) => return unproven_of(entry, pid, error.to_string()),
    };
    let mut bytes: Vec<u8> = Vec::new();
    match file.take(COMMAND_LINE_BYTES + 1).read_to_end(&mut bytes) {
        Ok(_) => {}
        Err(error) if process_is_gone(&error) => return CommandLine::Skipped,
        Err(error) => return unproven_of(entry, pid, error.to_string()),
    }
    if bytes.len() as u64 > COMMAND_LINE_BYTES {
        return unproven_of(entry, pid, format!("its command line is longer than the {COMMAND_LINE_BYTES} bytes the kernel allows any program to carry, so its later words were not looked at"));
    }
    return CommandLine::Words(bytes);
}
