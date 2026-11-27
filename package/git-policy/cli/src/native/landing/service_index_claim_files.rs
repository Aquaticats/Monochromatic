//! What:
//!  What the file at one slot of the claim chain is:
//! absent,
//! a gone maker's,
//! a running maker's,
//! one that cannot be judged,
//! or one whose maker is on another machine.
//! Why:
//!  A claim is a file placed whole beside the stale lock,
//! and no link is followed and no pipe waited on when it is read,
//! so a stranger's file of that name cannot hang the reclaimer
//! (`service_index_claim.rs`).
//! What such a reading answers,
//! and the reading itself,
//! are one step of their own,
//! because the names of the claims and the chain they make are about another question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = lookAt(path) // 'free' | 'dead' | 'live' | { unjudged } | { elsewhere }
//! ```

use crate::service_index_identity::{
    holder_is_elsewhere,
    holder_is_gone,
    holder_is_running,
};
use crate::service_index_marker::{
    Marked,
    parse_marker,
};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// The longest claim this reader looks at,
/// as long as the longest marker it reads from a lock.
pub(super) const CLAIM_READ_LIMIT: u64 = 512;

/// What a claim file at a slot is.
#[derive(Debug, Eq, PartialEq)]
pub(super) enum Slot {
    /// No such file.
    Free,
    /// A claim whose maker is gone,
    /// or a file that holds no whole marker (a power loss left it,
    /// or a stranger placed it;
    /// a
    ///  running claimer's file is placed whole):
    /// passed over,
    /// never removed while the stale file stands.
    Dead,
    /// A claim whose maker is proven to run.
    Live,
    /// A claim whose maker cannot be judged here (another PID namespace,
    /// a `/proc` that does not agree),
    /// or a claim that cannot be
    ///  read at all (then without a marker):
    /// neither gone nor proven running,
    /// and not a contention that passes.
    Unjudged(Option<Marked>),
    /// A claim whose maker is on another machine,
    /// or on one this machine cannot be compared with.
    Elsewhere(Marked),
}

/// What:
///  What the file at `path` is as a claim:
/// absent,
/// a gone maker's,
/// a running maker's,
/// or one that cannot be judged.
/// Why:
///  The chain's next free slot is chosen by it,
/// and a claim that is neither gone nor proven running stops the reclaimer rather than passing over it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lookAt = (path) => readNoFollow(path) ?? 'free'
/// ```
pub(super) fn look_at(path: &Path) -> Slot {
    // No link is followed and no pipe waited on, so a stranger's file of this name cannot hang the reclaimer.
    let opened = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path);
    let file = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Slot::Free,
        Err(error) => {
            crate::service_error!(
                "the claim file {} of a reclaim cannot be opened ({error}), so its maker is not judged",
                path.display()
            );
            return Slot::Unjudged(None);
        }
    };
    let mut text: String = String::new();
    match file.take(CLAIM_READ_LIMIT).read_to_string(&mut text) {
        Ok(_) => {}
        // Bytes that are not text are no marker: the file is a stranger's.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => return Slot::Dead,
        Err(error) => {
            crate::service_error!(
                "the claim file {} of a reclaim cannot be read ({error}), so its maker is not judged",
                path.display()
            );
            return Slot::Unjudged(None);
        }
    }
    return match parse_marker(text.as_str()) {
        Some(marked) if holder_is_gone(marked.holder.as_ref()) => Slot::Dead,
        Some(marked) if holder_is_elsewhere(marked.holder.as_ref()) => Slot::Elsewhere(marked),
        Some(marked) if holder_is_running(marked.holder.as_ref()) => Slot::Live,
        Some(marked) => Slot::Unjudged(Some(marked)),
        None => Slot::Dead,
    };
}

/// What reading the claim file at `path` as text gave:
/// its text,
/// nothing because there is no such file,
/// or an error because it
///  could not be read (then whether it is this attempt's claim is not known).
pub(super) fn text_of(path: &Path) -> std::io::Result<Option<String>> {
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut text: String = String::new();
    return match file.take(CLAIM_READ_LIMIT).read_to_string(&mut text) {
        Ok(_) => Ok(Some(text)),
        // A file that is not text is not this attempt's claim, which is text.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Ok(None),
        Err(error) => Err(error),
    };
}
