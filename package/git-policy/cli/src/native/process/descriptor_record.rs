//! What:
//!  One descriptor's own record (`/proc/<pid>/fdinfo/<n>`),
//!  read only when the descriptor's kind is known before the record is opened,
//!  and never past a bound.
//! Why:
//!  The kernel builds a whole record before the reading process receives its first byte:
//!  a record's `.show` callback fills one buffer and the reader's count does not shorten the work
//!  (`fs/seq_file.c`,
//!  `seq_read_iter`,
//!  which frees the buffer and doubles it until the record fits).
//!  So a record's size is decided by the process that holds the descriptor,
//!  and one read of an inotify descriptor's record costs one line and about 144 bytes for every watch it holds,
//!  allocated with `GFP_KERNEL_ACCOUNT` and therefore charged to the reading process's control group
//!  (`fs/notify/fdinfo.c`,
//!  `inotify_show_fdinfo`;
//!  issue 6287).
//!  The kind is read from the text of the descriptor's link,
//!  which is a different operation from walking that link:
//!  `readlink(2)` of `/proc/<pid>/fd/<n>` reaches `proc_pid_readlink` and `d_path`,
//!  which walks the dentry cache and asks the file's own file system nothing,
//!  while a walk of the same link revalidates the target's directory entry and can wait for ever on a stopped server
//!  (`doc/troubleshooting/fuse-descriptor-attributes.md`).
//!  A descriptor whose text is not a path cannot be a file of a file system,
//!  and a kernel anonymous inode (`anon_inode:`: inotify, fanotify, epoll and their kind) is not a file any scan here
//!  looks for,
//!  so neither is opened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = descriptorRecord({ process: '/proc/1234', fd: '3', targeted: 'real-file' });
//! ```

/// `OsStr` is a borrowed file name.
use std::ffi::OsStr;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// How much of a descriptor's record a scan takes:
/// four kibibytes,
/// which is far past the four short lines of a regular file's record
/// (`pos:`, `flags:`, `mnt_id:`, `ino:`)
/// and still a bound on what one scan holds in this process.
pub const RECORD_BYTES_AT_MOST: usize = 4096;

/// The text a kernel anonymous inode's link begins with.
/// Why:
///  `anon_inodefs` names every descriptor made by `anon_inode_getfile` this way,
///  and that is the family whose record grows with state another process decides
///  (`fs/notify/fdinfo.c`,
///  `inotify_show_fdinfo`).
const ANONYMOUS_INODE_PREFIX: &[u8] = b"anon_inode:";

/// The first byte of the text of a link to a file of a file system.
const PATH_PREFIX: &[u8] = b"/";

/// What:
///  The kind of target a scan looks for,
///  which decides which descriptors can be it at all.
/// Why:
///   A file is named by its device number and inode,
///  which no kernel anonymous inode and no pipe has,
///  and an anonymous file such as a pipe is named by its mount number and inode,
///  which is what a pipe's own record holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Targeted = 'real-file' | 'anonymous-file';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Targeted {
    /// A file of a file system,
    /// named by the device number and the inode the file has:
    /// only a descriptor whose link text is a path can be it.
    RealFile,
    /// An anonymous file,
    /// named by the mount number and the inode its record holds:
    /// a pipe or a socket is read,
    /// and a kernel anonymous inode is not.
    AnonymousFile,
}

/// What:
///  What one read of a descriptor's record came to.
/// Why:
///   A descriptor the scan cannot be looking for is answered without opening its record,
///  and everything that could not be read says which kind of refusal it was,
///  because a caller treats "the record could not be read" as a process it could not look at
///  and "the record is past the bound" as a file that may be held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Record = 'not-the-target' | 'gone' | { text: string } | 'beyond-bound' | { unreadable } | { kind-unknown }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Record {
    /// The descriptor cannot be the target, so its record was never opened.
    NotTheTarget,
    /// The descriptor or its process ended while the scan walked it,
    /// which is a race of every scan of another process's descriptors and no evidence about a file.
    Gone,
    /// The descriptor's record, at most `RECORD_BYTES_AT_MOST` bytes of it.
    Text(String),
    /// The record is longer than the bound,
    /// so none of it is used and the descriptor may be the target.
    BeyondBound,
    /// The record itself could not be read,
    /// named by the kind of refusal and by what the kernel said.
    Unreadable {
        /// What kind of refusal the record's read came to.
        kind: std::io::ErrorKind,
        /// What the kernel said.
        reason: String,
    },
    /// The text of the descriptor's link could not be read,
    /// so its kind is unknown and the descriptor may be the target.
    KindUnknown {
        /// What kind of refusal the link's read came to.
        kind: std::io::ErrorKind,
        /// What the kernel said.
        reason: String,
    },
}

/// What:
///  The record of the descriptor `fd` of the process whose table directory is `process`,
///  opened only when the text of its link says it can be the target.
/// Why:
///   The one read every scan of another process's descriptors goes through,
///  so the bound and the gate before the open cannot drift apart between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function descriptorRecord({ process, fd, targeted }) {
///   const text = readlink(`${process}/fd/${fd}`);
///   if (!opens(targeted, text)) return 'not-the-target';
///   return readAtMost(`${process}/fdinfo/${fd}`, 4096);
/// }
/// ```
#[cfg(unix)]
pub fn descriptor_record(process: &Path, fd: &OsStr, targeted: Targeted) -> Record {
    let text: Vec<u8> = match linked_text(process, fd) {
        Ok(found) => found,
        // The descriptor closed while the scan walked the directory, or the process exited: nothing of it is open, so it
        //  is not the file the scan looks for and the race is not evidence about the file.
        Err(error) if exited(&error) => return Record::Gone,
        // The kernel refuses this scan the descriptor at all, as it refuses its record: a process the scan could not look
        //  at is the case every caller tolerated before this gate, and it stays that case.
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return Record::Unreadable {
                kind: error.kind(),
                reason: format!("{error}"),
            };
        }
        // Anything else leaves the kind unknown: the text of a link that is a path can be longer than the kernel's own
        //  buffer, so a descriptor this scan cannot classify may be the file.
        Err(error) => {
            return Record::KindUnknown {
                kind: error.kind(),
                reason: format!("{error}"),
            };
        }
    };
    if !opens(targeted, text.as_slice()) {
        return Record::NotTheTarget;
    }
    return bounded_text(process.join("fdinfo").join(fd).as_path());
}

/// What:
///  The same on a platform whose `/proc` names no descriptor.
/// Why:
///   Nothing is guessed at where the kernel's own table is not there,
///  and the caller treats the process as one it could not look into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform !== 'linux') throw new Error('no /proc');;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn descriptor_record(_process: &Path, _fd: &OsStr, _targeted: Targeted) -> Record {
    return Record::Unreadable {
        kind: std::io::ErrorKind::Unsupported,
        reason: String::from("this platform has no `/proc`, so a descriptor's record is not read"),
    };
}

/// What:
///  The text of a descriptor's link,
///  read without walking it.
/// Why:
///   `/proc/<pid>/fd/<n>` is a magic link:
///  `readlink` of it copies the name `d_path` builds from the descriptor the kernel already holds,
///  and a walk of it would revalidate the target's directory entry on the target's own file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readlink(`${process}/fd/${fd}`); // 'anon_inode:inotify', 'pipe:[42]' or '/some/path'
/// ```
#[cfg(unix)]
fn linked_text(process: &Path, fd: &OsStr) -> std::io::Result<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt as _;
    let link: PathBuf = process.join("fd").join(fd);
    let target: PathBuf = std::fs::read_link(link.as_path())?;
    return Ok(target.as_os_str().as_bytes().to_vec());
}

/// What:
///  Whether an error means the descriptor or its process ended while the scan walked it.
/// Why:
///   `ENOENT` and `ESRCH` are expected races of a scan of another process's descriptors,
///  never a descriptor that may be open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['ENOENT', 'ESRCH'].includes(error.code)
/// ```
#[cfg(unix)]
fn exited(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

/// What:
///  Whether a descriptor whose link text is `text` can be the target at all.
/// Why:
///   This is the whole gate:
///  a record is opened only for a descriptor this answers `true` for,
///  so the size of a record another process decides is never built for the reading process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const opens = (targeted, text) => !text.startsWith('anon_inode:') && (targeted === 'anonymous-file' || text.startsWith('/'));
/// ```
#[cfg(unix)]
fn opens(targeted: Targeted, text: &[u8]) -> bool {
    if text.starts_with(ANONYMOUS_INODE_PREFIX) {
        return false;
    }
    return match targeted {
        Targeted::RealFile => text.starts_with(PATH_PREFIX),
        Targeted::AnonymousFile => true,
    };
}

/// What:
///  At most `RECORD_BYTES_AT_MOST` bytes of the file at `record`,
///  and the fact that there were more when there were.
/// Why:
///   A read of one byte past the bound is what tells a record that fits from one that does not,
///  and the text of a record that does not fit is not used at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = readAtMost(record, 4096 + 1);
/// return text.length > 4096 ? 'beyond-bound' : { text };
/// ```
#[cfg(unix)]
fn bounded_text(record: &Path) -> Record {
    use std::io::Read as _;
    let file: std::fs::File = match std::fs::File::open(record) {
        Ok(found) => found,
        Err(error) => {
            return Record::Unreadable {
                kind: error.kind(),
                reason: format!("{error}"),
            };
        }
    };
    let mut text: String = String::new();
    if let Err(error) = file.take(RECORD_BYTES_AT_MOST as u64 + 1).read_to_string(&mut text) {
        return Record::Unreadable {
            kind: error.kind(),
            reason: format!("{error}"),
        };
    }
    if text.len() > RECORD_BYTES_AT_MOST {
        return Record::BeyondBound;
    }
    return Record::Text(text);
}

/// What:
///  The number the record's `ino:` line names,
///  when it names one.
/// Why:
///   The last `ino:` line wins,
///  which is what the parse of a record has always been here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').filter(l => l.startsWith('ino:')).at(-1)?.slice(4).trim() as number
/// ```
pub fn inode_in(text: &str) -> Option<u64> {
    return number_after(text, "ino:");
}

/// What:
///  The number the record's `mnt_id:` line names,
///  when it names one.
/// Why:
///   The mount number and the inode together name an anonymous file,
///  which is what a pipe is found by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').filter(l => l.startsWith('mnt_id:')).at(-1)?.slice(7).trim() as number
/// ```
pub fn mount_id_in(text: &str) -> Option<u64> {
    return number_after(text, "mnt_id:");
}

/// What:
///  The number of the last line of `text` that begins with `prefix`.
/// Why:
///   Every line of a record is `<key>:\t<value>`,
///  and a record this module reads holds one `ino:` line and one `mnt_id:` line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const last = text.split('\n').filter(l => l.startsWith(prefix)).at(-1);
/// return last === undefined ? undefined : Number(last.slice(prefix.length).trim());
/// ```
fn number_after(text: &str, prefix: &str) -> Option<u64> {
    let mut found: Option<u64> = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix(prefix) {
            found = value.trim().parse::<u64>().ok();
        }
    }
    return found;
}

/// Record-read controls stay out of the release executable.
#[cfg(test)]
#[path = "descriptor_record_tests.rs"]
mod tests;
