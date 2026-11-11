//! What:
//!  What one look at one process of a push found,
//!  and the identities a look at a tree's sockets works with.
//! Why:
//!  `push_progress.rs` decides when a push is waiting and when it has stalled;
//! these are the pieces of that decision that outgrew it:
//! the two signs one look finds,
//! and the process identities a holder is remembered by.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Signs = { moved: boolean; busy: boolean }; const inodesOf = (member) => [member.pid, socketInodesOf(member.pid)];
//! ```

/// The processes below the push,
/// and the identity of each.
use crate::process_tree::TreeMember;
/// The sockets a process holds.
use crate::tcp_sockets::socket_inodes_of;

/// What one look at one process found:
/// the bytes it moved since the look before,
/// and whether it used processor time or has a task at work.
/// Why:
///   The signs are kept apart because a process that holds the connection to the remote may be polling it,
/// and a poll is not work the remote sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Signs = { moved: boolean; busy: boolean };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Signs {
    /// Bytes read or written since the look before.
    pub moved: bool,
    /// Processor time used since the look before,
    /// or a task able to run or waiting for the disk now.
    pub busy: bool,
}

/// The sockets one process of the tree holds,
/// with its number.
pub fn inodes_of(member: &TreeMember) -> (i32, Vec<u64>) {
    return (member.pid, socket_inodes_of(member.pid));
}

/// The start time the kernel recorded for one process of the tree,
/// which tells a holder from a later process the kernel gave its number.
pub fn start_time_of(members: &[TreeMember], pid: i32) -> u64 {
    for member in members {
        if member.pid == pid {
            return member.started;
        }
    }
    return 0;
}

/// Whether a process of the tree holds no socket at all.
pub fn holds_no_socket(held: &(i32, Vec<u64>)) -> bool {
    return held.1.is_empty();
}
