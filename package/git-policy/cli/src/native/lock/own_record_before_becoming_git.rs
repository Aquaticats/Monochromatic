//! What:
//!  The line a Git the wrapper starts writes about itself between `fork` and `exec`,
//!  when it already knows its own number and can read its own birth,
//!  so that no instant exists at which a Git of the wrapper's can hold a lock and no record names it.
//! Why:
//!   A parent that records its child after starting it always has a window:
//!  the child can take `index.lock` and be killed inside it,
//!  and the wrapper can be killed inside it too,
//!  which leaves a lock nothing can ever explain and a commit that fails on `File exists`
//!  (a review of `3ac1254a38`,
//! findings 1 and 3 of the two rounds before it).
//!  A parent that holds the child before it runs cannot help either: `std::process::Command::spawn` waits on its
//!  own error pipe until the child reaches `execve`, so a child held in `pre_exec` never lets `spawn` return.
//!   What is left is this: the child writes its own line, in the parent's own record file, before `execve`.
//!  Everything here runs in the child between `fork` and `exec`,
//!  so it uses only the calls the standard lists as safe there:
//!  `mkdir`, `open`, `read`, `write`, `close` and `getpid`,
//!  with no allocation and no lock.
//!   The line names the process and its start tick,
//!  without the boot, the PID namespace and the machine:
//!  the pass judges a holder on a lock of one of this machine's own file systems,
//!  where a record of this machine is the only one that can explain it,
//!  and the parent adds the full identity beside it as soon as it can.
//!  A child that then fails to `exec` is a process that never took a lock,
//!  so its line is one a later pass finds gone and sweeps with the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const own = prepareOwnLine(); command.preExec(() => own.write()); // mkdir folder; open file; read /proc/self/stat; write line
//! ```

/// The bytes a record starts with, and the separator the line needs.
use crate::lock_holder_records::RECORD_HEADER;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Everything the child needs to write its line:
///  the folder and file as C strings the child can hand to `open`,
///  and the header a new file starts with.
/// Why:
///   The child may not allocate,
///  so every byte it writes is prepared here,
///  in the parent,
///  where a `CString` costs nothing later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnLineBeforeBecomingGit = { folder: Buffer; file: Buffer; header: Buffer };
/// ```
#[cfg(unix)]
pub struct OwnLineBeforeBecomingGit {
    /// The record folder, NUL-terminated for `mkdir`.
    folder: std::ffi::CString,
    /// The record file, NUL-terminated for `open`.
    file: std::ffi::CString,
    /// The header a new file starts with, newline included.
    header: Vec<u8>,
}

/// What:
///  Prepare the child's own-line writer for the record `path` in `folder`,
///  or nothing when either name is not text this platform can hand to `open`.
/// Why:
///   The two names come from the repository's own path and this process's identity,
///  and a name that cannot be a C string is one no child can open:
///  the parent then records the child itself and the caller keeps the slower shape it had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prepareOwnLine('/repo/.git/cli-git-lock-holders', '/repo/.git/cli-git-lock-holders/42.abc')
/// ```
#[cfg(unix)]
pub fn prepare_own_line(folder: &Path, path: &Path) -> Option<OwnLineBeforeBecomingGit> {
    use std::os::unix::ffi::OsStrExt as _;
    let folder_text: std::ffi::CString = std::ffi::CString::new(folder.as_os_str().as_bytes()).ok()?;
    let file_text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut header: Vec<u8> = RECORD_HEADER.as_bytes().to_vec();
    header.push(b'\n');
    return Some(OwnLineBeforeBecomingGit { folder: folder_text, file: file_text, header });
}

/// What:
///  The start tick of a process's record in `/proc/<pid>/stat` when the kernel keeps it,
///  read from bytes the caller already has.
/// Why:
///   The last `)` ends the command name,
///  which may hold spaces and brackets,
///  and the start tick is the twentieth field after it
///  (`process_identity.rs`,
/// `LINUX_START_FIELD_INDEX`).
///  A missing field or no digits is no birth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startTickOf('/proc/self/stat bytes') // '16474396', undefined when the record has no such field
/// ```
fn start_tick_of(stat: &[u8]) -> Option<u64> {
    let end: usize = stat.iter().rposition(|byte| { return *byte == b')'; })?;
    let mut index: usize = end + 1;
    // The state is the first field after `) `, so the start tick is nineteen spaces further on.
    let mut fields_seen: usize = 0;
    while index < stat.len() && fields_seen <= 19 {
        while index < stat.len() && stat[index] == b' ' {
            index += 1;
        }
        if index >= stat.len() {
            return None;
        }
        fields_seen += 1;
        if fields_seen == 20 {
            let began: usize = index;
            let mut value: u64 = 0;
            while index < stat.len() && stat[index].is_ascii_digit() {
                value = value.checked_mul(10)?.checked_add(u64::from(stat[index] - b'0'))?;
                index += 1;
            }
            return (index > began).then_some(value);
        }
        while index < stat.len() && stat[index] != b' ' {
            index += 1;
        }
    }
    return None;
}

/// What:
///  The bytes of the line this process writes about itself:
///  `child <pid> linux:<start tick>` and a newline,
///  made in a buffer the caller owns.
/// Why:
///   The child may not allocate,
///  so the number and the tick are written by hand into a fixed buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownLineBytes(42, 16474396) // 'child 42 linux:16474396\n'
/// ```
fn own_line_bytes(pid: u32, tick: u64, out: &mut [u8]) -> Option<usize> {
    let mut at: usize = 0;
    for byte in b"child " {
        *out.get_mut(at)? = *byte;
        at += 1;
    }
    at += write_decimal(pid as u64, out.get_mut(at..)?)?;
    for byte in b" linux:" {
        *out.get_mut(at)? = *byte;
        at += 1;
    }
    at += write_decimal(tick, out.get_mut(at..)?)?;
    *out.get_mut(at)? = b'\n';
    return Some(at + 1);
}

/// What:
///  Write `value` in decimal into `out`,
///  and answer how many bytes it took.
/// Why:
///   The child's buffer is written by hand,
///  and this is the one place a number is turned into bytes there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeDecimal(42, out) // 2
/// ```
fn write_decimal(value: u64, out: &mut [u8]) -> Option<usize> {
    let mut digits: [u8; 20] = [0; 20];
    let mut count: usize = 0;
    let mut left: u64 = value;
    loop {
        digits[count] = b'0' + u8::try_from(left % 10).ok()?;
        count += 1;
        left /= 10;
        if left == 0 {
            break;
        }
    }
    for index in 0..count {
        *out.get_mut(index)? = digits[count - 1 - index];
    }
    return Some(count);
}

#[cfg(unix)]
impl OwnLineBeforeBecomingGit {
    /// What:
    ///  Write this process's own line,
    ///  and let the child become Git whatever comes of it.
    /// Why:
    ///   A record that cannot be written must never fail the command:
    ///  the folder's path may hold a person's file,
    ///  and an error handed back here stopped the child before `exec`,
    ///  so a wrapped commit failed with `File exists` while plain Git committed the same change
    ///  (a review of `551084db3c`,
    /// finding 1).
    ///   The parent names the child as soon as the start returns,
    ///  which is the shape a command with no record keeps,
    ///  and what the missing line costs is said once by the parent's warning.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// void write(); // a failure leaves the record without this line, and Git runs
    /// ```
    pub fn write_without_failing(&self) {
        let _dropped: std::io::Result<()> = self.write();
    }

    /// What:
    ///  Write this process's own line,
    ///  making the folder and the file first when they are not there.
    ///  `std::io::Result<()>` is what stopped the child from becoming Git before this change;
    ///  the one caller now drops it.
    /// Why:
    ///   This runs in the child between `fork` and `exec`,
    ///  so it calls only `mkdir`, `open`, `read`, `write`, `close` and `getpid`,
    ///  and it touches no memory it did not bring with it.
    ///  An error here is dropped by `write_without_failing`,
    ///  so a Git that cannot be recorded still runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// mkdirSync(folder, { recursive: true }); const fd = openSync(file, 'a+'); writeSync(fd, headerWhenNew + line)
    /// ```
    pub fn write(&self) -> std::io::Result<()> {
        // SAFETY: `mkdir` takes a NUL-terminated path this value owns and a mode; it touches no other memory.
        let made: libc::c_int = unsafe { libc::mkdir(self.folder.as_ptr(), 0o700) };
        if made != 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(error);
            }
        }
        let mut stat: [u8; 1024] = [0; 1024];
        let read: usize = read_bounded(c"/proc/self/stat".as_ptr(), &mut stat)?;
        let Some(tick) = start_tick_of(stat.get(..read).unwrap_or(&[])) else {
            // A raw number and no heap: an allocating error in a forked child can wait on an inherited allocator lock.
            return Err(std::io::Error::from_raw_os_error(libc::EIO));
        };
        let mut line: [u8; 64] = [0; 64];
        // SAFETY: `getpid` takes nothing and returns this process's number.
        let pid: u32 = unsafe { libc::getpid() }.cast_unsigned();
        let Some(line_len) = own_line_bytes(pid, tick, &mut line) else {
            // A raw number and no heap, for the reason above.
            return Err(std::io::Error::from_raw_os_error(libc::EIO));
        };
        let filled_line: &[u8] = line.get(..line_len).unwrap_or(&[]);
        // The file is made once and appended to afterwards, so the header goes only with the creation.
        let created: libc::c_int = unsafe { libc::open(self.file.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC, 0o600) };
        if created >= 0 {
            let wrote_header: std::io::Result<()> = write_all(created, self.header.as_slice());
            let wrote_line: std::io::Result<()> = wrote_header.and_then(|()| { return write_all(created, filled_line); });
            close(created);
            return wrote_line;
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(error);
        }
        // What: The record file is opened with the one set of flags every plain-file open uses: no link is followed, a pipe
        //       cannot wait, and the opened thing is checked before a byte is written.
        // Why:  This open once followed a link and waited on a named pipe, so the child appended a record line to a person's
        //       file and a Git start hung before `exec`
        //       (a review of `58cd4d9f7c`,
        // findings 3 and 4).
        let appended: libc::c_int =
            unsafe { libc::open(self.file.as_ptr(), libc::O_WRONLY | libc::O_APPEND | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC) };
        if appended < 0 {
            return Err(std::io::Error::from_raw_os_error(errno()));
        }
        if !is_a_plain_file(appended) {
            close(appended);
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }
        let wrote: std::io::Result<()> = write_all(appended, filled_line);
        close(appended);
        return wrote;
    }
}

/// What:
///  Read a small file into `buffer` and answer how many bytes arrived.
/// Why:
///   The child reads `/proc/self/stat` into a buffer of its own,
///  and one read of a small record is all it needs;
///  a read that is interrupted is tried again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fd = openSync(path, 'r'); const n = readSync(fd, buffer); closeSync(fd)
/// ```
#[cfg(unix)]
fn read_bounded(path: *const libc::c_char, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: the path is NUL-terminated and owned by the caller, and the flags are the ones `open` takes.
    let descriptor: libc::c_int = unsafe { libc::open(path, libc::O_RDONLY | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut filled: usize = 0;
    while filled < buffer.len() {
        // SAFETY: the pointer is the buffer's own start offset by `filled`, inside its bounds.
        let got: libc::ssize_t = unsafe { libc::read(descriptor, buffer.as_mut_ptr().add(filled).cast(), buffer.len() - filled) };
        if got > 0 {
            filled += usize::try_from(got).unwrap_or(0);
            continue;
        }
        if got == 0 {
            break;
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            close(descriptor);
            return Err(error);
        }
    }
    close(descriptor);
    return Ok(filled);
}

/// What:
///  Write every byte of `bytes` to `descriptor`.
/// Why:
///   A short write is normal on a pipe and possible on a file,
///  and a half-written line would name nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (bytes.length) bytes = bytes.subarray(writeSync(fd, bytes))
/// ```
#[cfg(unix)]
fn write_all(descriptor: libc::c_int, bytes: &[u8]) -> std::io::Result<()> {
    let mut written: usize = 0;
    while written < bytes.len() {
        // SAFETY: the pointer is the slice's own start offset by `written`, inside its bounds.
        let count: libc::ssize_t = unsafe { libc::write(descriptor, bytes.as_ptr().add(written).cast(), bytes.len() - written) };
        if count > 0 {
            written += usize::try_from(count).unwrap_or(0);
            continue;
        }
        if count == 0 {
            return Err(std::io::Error::from_raw_os_error(libc::EIO));
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    return Ok(());
}

/// What:
///  Close a descriptor, ignoring the answer.
/// Why:
///   The line is already written by then,
///  and a close that fails has no place to be reported in a child about to `exec`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { closeSync(fd) } catch {}
/// ```
#[cfg(unix)]
fn close(descriptor: libc::c_int) {
    // SAFETY: the descriptor is one this call opened and closes once.
    let _closed: libc::c_int = unsafe { libc::close(descriptor) };
}

/// What:
///  The folder and file a child would write to for `common_dir`,
///  or nothing when this process's identity or those names cannot be prepared.
/// Why:
///   The caller needs the same two names the record writer uses,
///  so a child and its parent write one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownLineFor('/repo/.git') // the prepared writer, or undefined
/// ```
#[cfg(unix)]
pub fn own_line_for(common_dir: &Path) -> Option<OwnLineBeforeBecomingGit> {
    let folder: PathBuf = crate::lock_holder_records::folder_of(common_dir);
    let path: PathBuf = crate::lock_holder_records::this_process_record_path(common_dir)?;
    return prepare_own_line(folder.as_path(), path.as_path());
}

/// What:
///  The number of the last failure,
///  without building an error of it.
/// Why:
///   The child may not allocate,
///  and `std::io::Error::last_os_error` is an enum of a number that allocates nothing,
///  but a message built for it would;
///  this names the number plainly for the one place that reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errno = () => libc.errno()
/// ```
#[cfg(unix)]
fn errno() -> libc::c_int {
    // SAFETY: `__errno_location` returns this thread's own errno cell and reads no memory of its caller.
    return unsafe { *libc::__errno_location() };
}

/// What:
///  Whether an open descriptor is a plain file,
///  read from the descriptor and never from the name.
/// Why:
///   A name swapped for a pipe or a device between the open and the write cannot redirect the write once the type of
///  what was opened decides,
///  and the check allocates nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = fstatSync(fd); return stat.isFile()
/// ```
#[cfg(unix)]
fn is_a_plain_file(descriptor: libc::c_int) -> bool {
    // SAFETY: `fstat` writes into this stack value, which is exactly one `libc::stat`, and reads the descriptor's own record.
    let mut metadata: libc::stat = unsafe { std::mem::zeroed() };
    let asked: libc::c_int = unsafe { libc::fstat(descriptor, &raw mut metadata) };
    if asked != 0 {
        return false;
    }
    return metadata.st_mode & libc::S_IFMT == libc::S_IFREG;
}
