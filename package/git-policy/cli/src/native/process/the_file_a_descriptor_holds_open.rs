//! What:
//!  The device number and the inode number of the file an open descriptor of
//!  a process refers to.
//! Why:
//!  Two readers answer with it and a third reads it back:
//!  this process's own `statx` on `/proc/<pid>/fd/<n>` (`open_file_of_a_descriptor`),
//!  the same read made by a child when the walk could reach a server that stopped
//!  (`open_file_of_a_descriptor_in_a_child`),
//!  and the judgement that compares it with a lock's own numbers
//!  (`descriptor_of_a_process_that_may_hold_the_file`).
//!  It sits in a file of its own,
//!  before all three,
//!  so neither reader names the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type OpenedFile = { device: bigint; inode: bigint };
//! ```

/// What:
///  The device number and the inode number of a file a descriptor holds open.
/// Why:
///  A lock is matched to the descriptor that holds it by these two numbers,
///  which the file system answers from the memory of the kernel for a mount it serves locally.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const opened: OpenedFile = { device: 2049n, inode: 123456n };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenedFile {
    /// The device number of the file system that holds the file (`st_dev`).
    /// The siblings a reader might expect are the inode below it and the mount id
    /// (`mnt_id`),
    /// which names a mount rather than a device.
    pub device: u64,
    /// The inode number of the file on that device.
    /// The sibling a reader might expect is the file's generation number,
    /// which the kernel reports only in its own record of a directory entry.
    pub inode: u64,
}
