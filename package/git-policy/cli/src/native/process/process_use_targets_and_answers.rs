//! What:
//!  What a scan of this host's processes is asked for,
//!  and what it answers with:
//!  the processes found using one directory or one file,
//!  and the reasons it could not see everything.
//! Why:
//!  The walk of the table (`process_use.rs`) and the callers that judge a record
//!  share one shape,
//!  so a caller reads a scan's answer without knowing how the table was walked.
//!  `rust/max-lines` splits a file by role,
//!  and these are the shapes a caller names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type UseTarget = { inside: string } | { file: { device: bigint; inode: bigint } };
//! type UseScan = { users: ProcessUser[]; unreadable: string[]; unresolved: string[]; unavailable?: string };
//! ```

use std::path::Path;
/// The root of the process table on Linux.
pub const PROC_ROOT: &str = "/proc";

/// How long one scan of this host's processes may take.
///
/// Measured on 2026-11-07 on this host at load average 37 to 52:
/// one walk of every process's open descriptors (list each `/proc/<pid>/fd` and `stat` each link) over 1,000 processes
/// and 54,500 descriptors took 313,
/// 360 and 398 ms,
/// and the commit service's scan carries the same bound for the same measurement (`SERVICE_SCAN_BUDGET`).
/// A scan that reaches it returns what it found and says that it did not finish,
/// which every caller reads as a table it may not act on:
/// two seconds is the smallest whole number of seconds that is at least five times the longest walk.
/// Why a whole table and not one descriptor:
///  a read of a descriptor's record cannot be interrupted once it has begun,
///  so what bounds a single read is the gate before it (`descriptor_record.rs`) and what bounds the scan is this.
pub const PROCESS_SCAN_CEILING: std::time::Duration = std::time::Duration::from_secs(2);

/// What:
///  What a scan looks for.
/// Why:
///   A directory is matched by path because a record inside it may be opened under any name;
///  a file is matched by device and inode because a lock has more than one name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UseTarget = { inside: string } | { file: { device: bigint; inode: bigint } };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UseTarget<'a> {
    /// A working directory or open descriptor whose resolved path lies under this canonical directory.
    Inside(&'a Path),
    /// An open descriptor on this file,
    /// named by the device number and the inode the file has.
    File {
        /// The device number of the file system that holds the file (`st_dev`).
        device: u64,
        /// Inode number of the file on that device.
        inode: u64,
    },
    /// An open descriptor on a file reached through this mount,
    /// named as `/proc/<pid>/fdinfo` names it.
    /// Why:
    ///   A pipe and another anonymous file have no device a descriptor could be read for through the gate that
    ///   keeps a stalled server out (`open_file_of_a_descriptor.rs`),
    ///   and the mount number and the inode in the descriptor's own record name the open file without resolving it.
    Mounted {
        /// The mount number the kernel gives the mount the file is reached through.
        mount: u64,
        /// The file's inode on that mount.
        inode: u64,
    },
}

/// One process found using the target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessUser {
    /// The process number.
    pub pid: i64,
    /// The command name when readable.
    pub command: Option<String>,
}

/// What:
///  The processes found using a target and the reasons the scan could not see everything.
/// Why:
///   A caller that cannot rely on a scan with no process table refuses when `unavailable` is set;
///  `unreadable` lists single processes the scan could not see into,
/// which a caller may or may not tolerate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UseScan = { users: ProcessUser[]; unreadable: string[]; unavailable?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UseScan {
    /// Processes using the target.
    pub users: Vec<ProcessUser>,
    /// Reasons some process of the scanning account could not be read at all.
    pub unreadable: Vec<String>,
    /// Reasons a descriptor whose record names the target could not be identified,
    /// because the kernel refused to copy it into this process:
    /// the process may hold the file,
    /// which is what a scan that did not finish proves nothing means.
    pub unresolved: Vec<String>,
    /// Why the process table itself could not be listed,
    /// so that no process was scanned.
    pub unavailable: Option<String>,
}

/// What:
///  Why a descriptor could not be identified.
/// Why:
///   Something of the process could not be read at all (its descriptor's record,
/// or its working directory),
///  which a caller may tolerate as it always did;
///  a record that named the file whose file cannot then be identified is a file that may be held,
///  which a caller must not read as "not the file".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unread = { kind: 'process'; reason: string } | { kind: 'file'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unread {
    /// Something of the process could not be read.
    Process(String),
    /// The record was read and named the file,
    /// and the file itself could not be identified.
    File(String),
}
