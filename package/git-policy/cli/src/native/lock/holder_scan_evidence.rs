//! What:
//!  What one scan of a lock's holders reports:
//!  the mechanism it read,
//!  the processes holding the file open,
//!  and the reasons it could not see a process or identify a descriptor.
//! Why:
//!  A removal that acts on a scan has to tell a clean scan from one that saw nothing:
//!  a descriptor whose record names the lock and which the kernel would not copy out may be a live holder,
//!  so it is kept apart from a process the scan simply could not look into.
//!  One record shape for every mechanism
//!  (`/proc/<pid>/fd`,
//!  `lsof`
//!  and the Restart Manager)
//!  keeps that judgement in one place.
//!  The reader of a descriptor's open file is a seam a control hands its own to,
//!  because a control's `/proc` is a directory of links
//!  and its numbers are not processes the kernel copies a descriptor for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LockHolderEvidence = { method: 'proc-fd' | 'lsof' | 'restart-manager' | 'unsupported'; holders: LockHolderProcess[]; partial: string[]; unresolvable: string[] };
//! ```

/// `OsStr` is a borrowed name of one entry in a process's descriptor folder.
use std::ffi::OsStr;
/// `io` holds the result of reading a descriptor's open file.
use std::io;
/// `Path` is a borrowed filesystem path,
/// and not the owned `PathBuf`.
use std::path::Path;

/// How a descriptor's open file is read:
/// the kernel's copy of the descriptor in a judgement,
/// or a control's own read of a link for a table the kernel does not know.
#[cfg(unix)]
pub type OpenedFileReader =
    fn(&Path, &OsStr) -> io::Result<crate::open_file_of_a_descriptor::OpenedFile>;

/// What:
///  One process holding the lock open.
/// Why:
///   `LockHolderProcess`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockHolderProcess = { pid: number; command?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Holder {
    /// Holder process ID.
    pub pid: i64,
    /// Holder command name when readable.
    pub command: Option<String>,
}

/// What:
///  The scan mechanism,
///  its holders,
///  and what it could not see.
/// Why:
///   `LockHolderEvidence`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockHolderEvidence = { method: 'proc-fd' | 'lsof' | 'restart-manager' | 'unsupported'; holders; partial };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderEvidence {
    /// Scan mechanism name.
    pub method: &'static str,
    /// Processes holding the lock open.
    pub holders: Vec<Holder>,
    /// Reasons the scan could not see everything.
    pub partial: Vec<String>,
    /// Reasons a descriptor whose record names the lock could not be identified,
    /// because the kernel refused to copy it into this process:
    /// a removal that acts on a scan with one of these is acting on a scan that proved nothing.
    pub unresolvable: Vec<String>,
}

impl HolderEvidence {
    /// Whether the scan reported a descriptor whose record named the lock and which it could not identify,
    /// which is a holder that may be alive.
    pub fn unresolved(&self) -> bool {
        return !self.unresolvable.is_empty();
    }
}
