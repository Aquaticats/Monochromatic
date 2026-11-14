//! What:
//!  Whether a socket of an earlier release's unit listens,
//!  and whether a socket of a given name listens,
//! read from the table of unix sockets (`/proc/net/unix`):
//! the table's
//!  lines,
//! the flag that says a socket listens,
//! the path a line ends with,
//!  and the whole name a line holds.
//! Why:
//!  A socket unit's file stays behind when the unit stops (measured with a throwaway socket unit),
//! so the file says nothing
//!  about whether a connection would start the earlier helper;
//! the table lists the sockets that exist now and flags the
//!  ones that listen (`__SO_ACCEPTCON`,
//! `1 << 16`,
//! Linux `include/linux/net.h`;
//! `net/unix/af_unix.c`,
//! `unix_seq_show`,
//! prints it
//!  when `sk_state == TCP_LISTEN`).
//! Provisioning reads it to know whether the earlier socket of a worktree still needs to be
//!  stopped,
//! and the look that guards an adoption of the earlier journals reads it for the same reason.
//! A service whose address is an abstract name has no file to look at,
//!  so provisioning and the route read the same table to see one
//!  (`service_address_of_a_worktree.rs`).
//! A table that cannot
//!  be read leaves the question open and is told apart from a table without a listener.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! earlierSocketOfInstanceListens(procRoot, instance) // 'listens' | 'none' | 'unreadable'
//! ```

use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::Path;

/// The bit of the flags column of `/proc/net/unix` that says the socket listens.
const SOCKET_LISTENS_FLAG: u32 = 1 << 16;

/// The columns of a line of `/proc/net/unix` up to and including the path:
///  `Num RefCount Protocol Flags Type St Inode Path` (a socket with no name has no path column).
const SOCKET_TABLE_COLUMNS: usize = 8;

/// The position of the flags column in a line of `/proc/net/unix`.
const SOCKET_FLAGS_COLUMN: usize = 3;

/// What the table of unix sockets says about the sockets that end with some paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListeningSockets {
    /// A socket that listens ends with one of the paths.
    Listens,
    /// The table was read and none does.
    None,
    /// The table or one of its lines could not be read.
    Unreadable,
}

/// The flags column of a line of `/proc/net/unix`,
/// which the kernel prints in hexadecimal;
/// `None` when it is not.
fn socket_flags_of(column: &[u8]) -> Option<u32> {
    let text: &str = std::str::from_utf8(column).ok()?;
    return u32::from_str_radix(text, 16).ok();
}

/// How a line's path column is held against what a question asks about.
#[derive(Clone, Copy, Debug)]
enum PathColumnMatch<'a> {
    /// The column ends with one of these,
    /// which is how a socket under a directory is found without spelling the directory.
    EndsWith(&'a [Vec<u8>]),
    /// The column is exactly one of these,
    /// which is how a name that stands alone is found,
    /// and never a name it ends.
    Exactly(&'a [Vec<u8>]),
}

/// What:
///  Whether one line of `/proc/net/unix` is a socket that listens whose path column is held the way `wanted` says.
///  A flags column that is not hexadecimal counts as listening.
/// Why:
///   A line the reader cannot understand must not hide a listener.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listensAt(line, wanted)
/// ```
fn is_listening_socket_matching(line: &[u8], wanted: PathColumnMatch<'_>) -> bool {
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
    if !listens {
        return false;
    }
    let path: &[u8] = columns[columns.len() - 1];
    return match wanted {
        PathColumnMatch::EndsWith(endings) => endings.iter().any(|ending| path.ends_with(ending.as_slice())),
        PathColumnMatch::Exactly(names) => names.iter().any(|name| path == name.as_slice()),
    };
}

/// What:
///  Reads the table under `proc_root` and says whether a socket that listens has a path column `wanted` describes.
/// Why:
///   The table is read,
/// never the file system,
/// and nothing connects to a socket,
/// because a connection to an earlier unit's
///  socket starts its helper,
/// and a connection to an abstract name starts a service on a commit's build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listeningSocketsWhere(procRoot, (path) => endings.some((ending) => path.endsWith(ending)))
/// ```
fn listening_sockets_where(proc_root: &Path, wanted: PathColumnMatch<'_>) -> ListeningSockets {
    use std::io::BufRead;
    let Ok(table) = std::fs::File::open(proc_root.join("net").join("unix")) else {
        return ListeningSockets::Unreadable;
    };
    for read in std::io::BufReader::new(table).split(b'\n') {
        let Ok(line) = read else {
            return ListeningSockets::Unreadable;
        };
        if is_listening_socket_matching(line.as_slice(), wanted) {
            return ListeningSockets::Listens;
        }
    }
    return ListeningSockets::None;
}

/// What:
///  Whether a socket that listens ends with one of `endings`.
/// Why:
///   The earlier release's socket is a file under a directory the caller does not spell,
/// so the name is matched by its ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listeningSocketsEndingWith(procRoot, ['/cli-git/abc.sock'])
/// ```
pub fn listening_sockets_ending_with(proc_root: &Path, endings: &[Vec<u8>]) -> ListeningSockets {
    return listening_sockets_where(proc_root, PathColumnMatch::EndsWith(endings));
}

/// What:
///  Whether a socket that listens holds exactly one of `names` in its path column.
/// Why:
///  An abstract socket's column is the whole name with an `@` before it,
///  as systemd prints one (`sockaddr_pretty`,
///  `src/basic/socket-util.c`),
///  and a name that is the ending of another name is another socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listeningSocketsExactly(procRoot, ['@monochromatic-git-policy-cli/1000/0123456789abcdef/abc'])
/// ```
pub fn listening_sockets_exactly(proc_root: &Path, names: &[Vec<u8>]) -> ListeningSockets {
    return listening_sockets_where(proc_root, PathColumnMatch::Exactly(names));
}

/// The path every socket of an earlier unit instance ends with:
/// `/cli-git/<instance>.sock`.
pub fn earlier_socket_ending(instance: &str) -> Vec<u8> {
    return format!("/{}/{instance}.sock", SERVICE_UNIT.earlier).into_bytes();
}

/// Whether the earlier release's socket of an instance listens;
/// a table that cannot be read counts as "yes".
pub fn earlier_socket_of_instance_listens(proc_root: &Path, instance: &str) -> bool {
    return listening_sockets_ending_with(proc_root, &[earlier_socket_ending(instance)]) != ListeningSockets::None;
}
