//! What:
//!  What a name holds,
//! and whether it holds exactly what an install put there or found there.
//! Why:
//!  Every judgement of an install is about a file's bytes together with what it carries:
//!  its permission
//!       bits,
//!  its owner and its kind,
//!  read without following a symbolic link.
//!  Bytes alone would take a file the
//!       person narrowed for the fix's own,
//!  and a link whose target happens to hold the correction,
//!  so
//!       the comparison lives in one place and both the validation and the rollback use it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const state = lstatAndRead(path); const same = state.kind === 'file' && equal(state.bytes, bytes) && sameKept(state.kept, kept);
//! ```

/// The permission bits and owner a file carries.
use crate::direct_fix_install_writes::Kept;
/// The site a judgement is about.
use crate::direct_fix_install::Prepared;
/// The journal that holds what a swap displaced.
use crate::direct_fix_install_journal::content_of;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The situation of one prepared site,
///  read from a name.
/// Why:
///   A name can be gone,
///  no longer be a regular file,
///  or hold other bytes than the ones a caller expects;
///  each
///       answer is different for the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = 'absent' | 'not-a-file' | { bytes: Uint8Array; kept: Kept };
/// ```
pub(crate) enum SiteState {
    /// Nothing is at the name.
    Absent,
    /// Something is there,
    ///  and it is not a regular file.
    NotAFile,
    /// A regular file,
    ///  with its bytes and what it carries.
    File {
        /// The bytes it holds.
        bytes: Vec<u8>,
        /// Its permission bits and owner.
        kept: Kept,
    },
}

/// What:
///  Read what is at `path` now,
///  without following a symbolic link.
///  `&str` names the step that failed,
///  for the person's message.
/// Why:
///   Every judgement of a site is about the file itself,
///  never a link that leads elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = lstatAndRead(path); // 'absent' | 'not-a-file' | { bytes, kept }
/// ```
pub(crate) fn state_of(path: &Path, step: &str) -> Result<SiteState, String> {
    let described: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(SiteState::Absent),
        Err(error) => return Err(format!("{step} failed ({error})")),
    };
    if !described.file_type().is_file() {
        return Ok(SiteState::NotAFile);
    }
    let kept: Kept = Kept::of(&described);
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(read) => read,
        Err(error) => return Err(format!("{step} failed ({error})")),
    };
    return Ok(SiteState::File { bytes, kept });
}

/// What:
///  Whether the bytes a swap displaced are still the bytes the fix read,
///  and carry the mode
///       and owner the file had.
///  `Ok(false)` is "the file changed while the fix ran".
/// Why:
///   The comparison is made after the swap,
///  when the displaced file can no longer be overwritten by
///       the swap itself;
///  a changed permission or owner is the same kind of change as changed bytes,
///  and
///       installing over it would hand the file a mode the person did not choose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unchanged = (site) => { const s = stateOf(contentOf(site)); return s.kind === 'file' && equal(s.bytes, site.original) && s.kept.mode === site.kept.mode && s.kept.uid === site.kept.uid && s.kept.gid === site.kept.gid; };
/// ```
pub(crate) fn displaced_is_unchanged(site: &Prepared) -> Result<bool, String> {
    return holds_exactly(
        content_of(site.journal.as_path()).as_path(),
        &site.original,
        &site.kept,
        "reading what a corrected file displaced",
    );
}

/// What:
///  Whether the name holds exactly these bytes with these permission bits and this owner,
///  as a
///       regular file,
///  never following a symbolic link.
///  `&str` names the step that failed,
///  for the person's
///       message.
/// Why:
///  Bytes alone do not say whose bytes they are:
///  a file narrowed to `0600` was the person's act,
///  a
///       link whose target holds the bytes is not the file,
///  and a directory or a device at the name is not either.
///  A
///       judgement that reads through a link would take the person's link for the fix's own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsExactly = (path, bytes, kept, step) => { const s = lstatAndRead(path, step); return s.kind === 'file' && equal(s.bytes, bytes) && s.kept.mode === kept.mode && s.kept.uid === kept.uid && s.kept.gid === kept.gid; };
/// ```
pub(crate) fn holds_exactly(path: &Path, bytes: &[u8], kept: &Kept, step: &str) -> Result<bool, String> {
    let found: SiteState = state_of(path, step)?;
    let SiteState::File {
        bytes: held,
        kept: carried,
    } = found
    else {
        return Ok(false);
    };
    if carried.mode != kept.mode || carried.uid != kept.uid || carried.gid != kept.gid {
        return Ok(false);
    }
    return Ok(held.as_slice() == bytes);
}
