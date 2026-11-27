//! What:
//!  The records an install hands between its steps:
//!  the correction it is given,
//!  the site it has prepared,
//!  and what it leaves for the person.
//! Why:
//!  The preparation, the swap and every judgement read the same three records,
//!  and their fields say what each step may trust:
//!  the bytes the fix read,
//!  the bytes to write,
//!  and the permission bits and owner the file had.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type InstallChange = { path: Uint8Array; mode: CandidateMode; original: Uint8Array; replacement: Uint8Array };
//! type Prepared = { destination: string; journal: string; original: Uint8Array; replacement: Uint8Array; kept: Kept };
//! type InstallOutcome = { notices: string[] };
//! ```

/// The mode a candidate carries,
///  which the installed wrapper's fix records beside each correction.
use crate::candidate_object::CandidateMode;
/// What the file being replaced had:
///  its permission bits and owner.
use crate::direct_fix_install_writes::Kept;
/// `PathBuf` is an owned filesystem path of raw bytes (sibling `&Path` borrows).
use std::path::PathBuf;
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;
/// What:
///  One file to correct:
///  its pathname,
///  mode,
///  the bytes the fix read and the bytes to
///       write.
///  `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   The read bytes prove nobody changed the file while the fix ran.
///  A replacement keeps the
///       destination's permission bits and owner (see `Kept`);
///  what it does not carry is everything else a
///       file can hold:
///  its extended attributes and access-control list,
///  its inode number and birth time,
///  and its
///       share of a hard link,
///  whose other name keeps the bytes it had.
///  A symbolic link at the path is refused rather
///       than replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallChange = { path: Uint8Array; mode: CandidateMode; original: Uint8Array; replacement: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallChange {
    /// The pathname,
    ///  relative to the repository's top level,
    ///  as Git's raw bytes.
    pub path: Vec<u8>,
    /// The file's mode;
    ///  executable files stay executable.
    pub mode: CandidateMode,
    /// The bytes the fix read;
    ///  the worktree file must still hold them.
    pub original: Rc<[u8]>,
    /// The corrected bytes.
    pub replacement: Rc<[u8]>,
}

/// What:
///  One correction,
///  ready to swap:
///  the file,
///  the journal that holds its bytes,
///  and what the file
///       was when the fix read it.
/// Why:
///   The swap and every later judgement need only these,
///  and nothing here changes the worktree until the swap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prepared = { destination: string; journal: string; original: Uint8Array; replacement: Uint8Array; kept: Kept };
/// ```
#[derive(Debug)]
pub struct Prepared {
    /// The worktree file.
    pub(crate) destination: PathBuf,
    /// The journal directory that holds the corrected bytes,
    ///  and then the bytes the swap displaced.
    pub(crate) journal: PathBuf,
    /// The bytes the fix read.
    pub(crate) original: Rc<[u8]>,
    /// The corrected bytes.
    pub(crate) replacement: Rc<[u8]>,
    /// The permission bits and owner the file had.
    pub(crate) kept: Kept,
}
/// What:
///  What an install leaves for the person to look at.
///  `Vec<String>` is one sentence per site,
///  empty when every journal was removed.
/// Why:
///   A site someone wrote to after its swap keeps its journal,
///  and the person must be told where
///       those bytes are without the message naming a candidate's pathname.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallOutcome = { notices: string[] };
/// ```
#[derive(Debug, Default, Eq, PartialEq)]
pub struct InstallOutcome {
    /// One sentence per journal that stays.
    pub notices: Vec<String>,
}
