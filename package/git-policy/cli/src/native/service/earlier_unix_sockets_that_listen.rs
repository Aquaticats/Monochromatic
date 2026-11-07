//! What:
//!  Whether a socket of an earlier release's unit listens, read from the table of unix sockets (`/proc/net/unix`): the table's
//!  lines, the flag that says a socket listens, and the path a line ends with.
//! Why:
//!  A socket unit's file stays behind when the unit stops (measured with a throwaway socket unit), so the file says nothing
//!  about whether a connection would start the earlier helper; the table lists the sockets that exist now and flags the
//!  ones that listen (`__SO_ACCEPTCON`, `1 << 16`, Linux `include/linux/net.h`; `net/unix/af_unix.c`, `unix_seq_show`, prints it
//!  when `sk_state == TCP_LISTEN`). Provisioning reads it to know whether the earlier socket of a worktree still needs to be
//!  stopped, and the look that guards an adoption of the earlier journals reads it for the same reason. A table that cannot
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

/// The flags column of a line of `/proc/net/unix`, which the kernel prints in hexadecimal; `None` when it is not.
fn socket_flags_of(column: &[u8]) -> Option<u32> {
    let text: &str = std::str::from_utf8(column).ok()?;
    return u32::from_str_radix(text, 16).ok();
}

/// What:
///  Whether one line of `/proc/net/unix` is a socket that listens and whose path ends with one of `endings`.
///  A flags column that is not hexadecimal counts as listening.
/// Why:
///   A line the reader cannot understand must not hide a listener.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listensAt(line, endings)
/// ```
fn is_listening_socket_ending_with(line: &[u8], endings: &[Vec<u8>]) -> bool {
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

/// What:
///  Reads the table under `proc_root` and says whether a socket that listens ends with one of `endings`.
/// Why:
///   The table is read, never the file system, and nothing connects to a socket, because a connection to an earlier unit's
///  socket starts its helper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listeningSocketsEndingWith(procRoot, ['/cli-git/abc.sock'])
/// ```
pub fn listening_sockets_ending_with(proc_root: &Path, endings: &[Vec<u8>]) -> ListeningSockets {
    use std::io::BufRead;
    let Ok(table) = std::fs::File::open(proc_root.join("net").join("unix")) else {
        return ListeningSockets::Unreadable;
    };
    for read in std::io::BufReader::new(table).split(b'\n') {
        let Ok(line) = read else {
            return ListeningSockets::Unreadable;
        };
        if is_listening_socket_ending_with(line.as_slice(), endings) {
            return ListeningSockets::Listens;
        }
    }
    return ListeningSockets::None;
}

/// The path every socket of an earlier unit instance ends with: `/cli-git/<instance>.sock`.
pub fn earlier_socket_ending(instance: &str) -> Vec<u8> {
    return format!("/{}/{instance}.sock", SERVICE_UNIT.earlier).into_bytes();
}

/// Whether the earlier release's socket of an instance listens; a table that cannot be read counts as "yes".
pub fn earlier_socket_of_instance_listens(proc_root: &Path, instance: &str) -> bool {
    return listening_sockets_ending_with(proc_root, &[earlier_socket_ending(instance)]) != ListeningSockets::None;
}
