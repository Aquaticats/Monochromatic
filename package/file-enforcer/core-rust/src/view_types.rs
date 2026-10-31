//! What:
//!  The small value types the virtual view stores:
//! planned states and observed states.
//! Why:
//!  Kept apart so the view itself stays under the file length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Planned = { kind: 'file'; content: Uint8Array; mode?: number } | { kind: 'removed' };
//! ```

/// Imports `RealState`,
/// `sha256_hex` from `crate::real_fs`.
use crate::real_fs::{RealState, sha256_hex};

/// What:
///  A planned state of one path,
/// as an earlier rule left it.
/// Why:
///  The overlay stores these;
/// a removal hides the real file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Planned = { kind: 'file'; content: Uint8Array; mode?: number } | { kind: 'removed' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Planned {
    /// The path will hold these bytes (and this mode when one is set).
    File {
        /// Planned content.
        content: Vec<u8>,
        /// Planned permission bits,
        /// when the rule sets them.
        mode: Option<u32>,
    },
    /// The path will not exist.
    Removed,
}

/// What:
///  What the run observed about one real path,
/// compact enough to compare later.
/// Why:
///  Revalidation re-observes the path and compares;
/// content is kept as a digest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Observed = { kind: 'absent' } | { kind: 'file'; digest: string; mode: number } | { kind: 'directory' } | { kind: 'other' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Observed {
    /// Nothing there.
    Absent,
    /// A file with this content digest and mode.
    File {
        /// SHA-256 of the content.
        digest: String,
        /// Permission bits.
        mode: u32,
    },
    /// A directory,
    /// a link or a special file.
    NotAFile(
        /// The wrapped value.
        String,
    ),
}

/// What:
///  Reduce a real state to what revalidation compares.
/// Why:
///  A digest is enough to notice a changed file without keeping its bytes twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function observedOf(state: RealState): Observed
/// ```
pub fn observed_of(state: &RealState) -> Observed {
    match state {
        RealState::Absent => return Observed::Absent,
        RealState::File { content, mode } => return Observed::File { digest: sha256_hex(content), mode: *mode },
        RealState::Directory => return Observed::NotAFile("directory".to_string()),
        RealState::Symlink => return Observed::NotAFile("symlink".to_string()),
        RealState::Other => return Observed::NotAFile("other".to_string()),
    }
}
