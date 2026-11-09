//! What:
//!  What a look at the destination's own pointer said:
//! Git's pointer is there,
//! it is not there,
//! or the
//!  directory could not be examined,
//! and why.
//! Why:
//!  Recovery decides from that look whether a planned add left anything,
//! and the lookup answers three
//!  things,
//! not two:
//! an answer that could not be read is not an answer of absence,
//! and only absence says the
//!  add left nothing to finish.
//! A refusal read as absence deleted the record and the claim of a worktree that is still
//!  registered,
//! and no later command named it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { lstatSync(join(destination, '.git')); return 'present'; } catch (e) { return e.code === 'ENOENT' ? 'absent' : { unknown: e.message }; }
//! ```

use super::POINTER;
use std::path::Path;

/// What:
///  Whether the destination holds Git's pointer,
/// and when that cannot be read at all.
/// Why:
///   The lookup answers three things,
/// not two:
/// the pointer is there,
/// it is not,
/// or the directory could not be
///  examined.
/// An answer that could not be read is not an answer of absence,
/// and only absence says the add left
///  nothing to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { lstatSync(join(destination, '.git')); return 'present'; } catch (e) { return e.code === 'ENOENT' ? 'absent' : { unknown: e.message }; }
/// ```
pub(super) fn pointer_presence(destination: &Path) -> PointerPresence {
    return match std::fs::symlink_metadata(destination.join(POINTER)) {
        Ok(_found) => PointerPresence::Present,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => PointerPresence::Absent,
        Err(error) => PointerPresence::Unknown(error.to_string()),
    };
}

/// What:
///  What a lookup of the destination's pointer said.
/// Why:
///   The three answers are kept apart so no caller reads a refusal as absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PointerPresence = 'present' | 'absent' | { unknown: string };
/// ```
pub(super) enum PointerPresence {
    /// Git's pointer is there.
    Present,
    /// The destination holds no pointer.
    Absent,
    /// The destination could not be examined,
    /// and the reason why.
    Unknown(String),
}
