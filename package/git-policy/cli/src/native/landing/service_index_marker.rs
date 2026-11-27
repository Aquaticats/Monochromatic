//! What:
//!  The marker the service writes into the `index.lock` it holds,
//! and the questions asked of a marker found there.
//! Why:
//!  A restart tells its own stale lock from a foreign one,
//! and a live holder from a dead one,
//! by the marker alone:
//! the batch,
//!  the holding process and that process's birth identity.
//! The module is split from `service_index_lock.rs`,
//! which re-exports the public items here under their old names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const marker = markerText(seq); const reclaimable = isReclaimable(lockPath);
//! ```

use crate::process_identity::ProcessIdentityError;
use std::io::Read;

/// The first bytes of a lock the service holds,
/// followed by the batch number,
/// the holding process and its birth identity.
pub const LOCK_MARKER: &str = "cli-git-service ";

/// What follows `LOCK_MARKER` in a marker of this release:
/// a word that no earlier release's reader reads as a batch number,
/// so
///  an earlier release that finds such a lock reads no marker in it,
/// and leaves it to a person as a lock it does not know.
/// The
///  stale-lock exclusion of this release (the claim chain) and the one of earlier releases (a `flock`) do not see each other,
/// and
///  this keeps each release's reclaimers to the locks written in its own format.
pub const MARKER_VERSION: &str = "v2 ";

/// The longest marker this reader looks at.
pub(crate) const MARKER_LIMIT: u64 = 512;

/// What a service marker names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Marked {
    /// The batch the lock was taken for.
    pub seq: u64,
    /// The holding process and its birth identity;
    /// none in a marker a release before this one wrote.
    pub holder: Option<(i64, String)>,
    /// Whether the marker is in the format of this release (`MARKER_VERSION`),
    /// as opposed to an earlier release's.
    pub current: bool,
}

/// What:
///  The marker this process writes for batch `seq`:
/// the batch,
/// the process and its birth identity,
/// one line.
/// Why:
///   A restart reclaims a marked lock only when that process is gone,
/// so a slow but live holder keeps its lock.
/// A marker without a birth identity reads as a dead instance's,
/// so a process that cannot name its own identity writes none:
/// it must not take the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git-service v2 ${seq} ${process.pid} ${birth}\n`
/// ```
///
/// # Errors
///
/// The identity error when this process's birth identity cannot be read.
pub fn marker_text(
    seq: u64,
    identity: &dyn Fn() -> Result<String, ProcessIdentityError>,
) -> Result<String, ProcessIdentityError> {
    let birth: String = identity()?;
    return Ok(format!(
        "{LOCK_MARKER}{MARKER_VERSION}{seq} {} {birth}\n",
        std::process::id()
    ));
}

/// What:
///  The batch and holder a lock's text names,
/// or nothing when it is not a service marker.
/// Why:
///   A marker without a holder is one a release before this one wrote;
/// one with a malformed holder is not the service's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseMarker(text) // { seq, holder?: [pid, birth] } | undefined
/// ```
pub fn parse_marker(text: &str) -> Option<Marked> {
    let after_prefix: &str = text.strip_prefix(LOCK_MARKER)?;
    let (rest, current): (&str, bool) = match after_prefix.strip_prefix(MARKER_VERSION) {
        Some(versioned) => (versioned, true),
        None => (after_prefix, false),
    };
    // A marker is a whole line: one cut anywhere before its newline is a marker that was not finished, whatever its fields look like.
    let line: &str = rest.strip_suffix('\n')?;
    let mut parts = line.splitn(3, ' ');
    let seq: u64 = parts.next()?.parse::<u64>().ok()?;
    let holder: Option<(i64, String)> = match (parts.next(), parts.next()) {
        (None, None) => None,
        (Some(pid), Some(birth)) if !birth.is_empty() => {
            Some((pid.parse::<i64>().ok()?, birth.to_string()))
        }
        _ => return None,
    };
    // A marker of this release always names its holder.
    if current && holder.is_none() {
        return None;
    }
    return Some(Marked {
        seq,
        holder,
        current,
    });
}

/// The marker of the already open `file`,
/// read from its start,
/// when it holds one.
pub fn marker_of_file(file: &std::fs::File) -> Option<Marked> {
    let mut text: String = String::new();
    // A shared reference reads from the file's own position, which a fresh descriptor has at the start.
    let mut reader: &std::fs::File = file;
    (&mut reader)
        .take(MARKER_LIMIT)
        .read_to_string(&mut text)
        .ok()?;
    return parse_marker(text.as_str());
}
