//! What:
//!  What a name holds,
//! and whether it holds exactly what an install put there or found there.
//! Why:
//!  Every judgement of an install is about a file's bytes together with what it carries:
//!  its permission
//!       bits,
//!  its owner,
//!  the extended attributes it holds outside its bytes,
//!  and its kind,
//!  read without
//!       following a symbolic link.
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

/// The permission bits,
///  owner and extended attributes a file carries.
use crate::direct_fix_install_writes::Kept;
/// The attributes themselves,
///  read from the file each judgement is about.
use crate::direct_fix_install_attributes::attributes_of;
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
        /// Its permission bits,
        ///  owner and extended attributes.
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
    // What: The attributes the file carries are read beside its description, because a nameless object has none and the
    //       attributes decide whose file it is.
    // Why:  A replacement that could not carry them would delete data no commit holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kept = { ...Kept.of(described), attributes: await attributesOf(path, step) };
    // ```
    let mut kept: Kept = Kept::of(&described);
    kept.attributes = attributes_of(path, step)?;
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
/// const unchanged = (site) => { const s = stateOf(contentOf(site)); return s.kind === 'file' && sameKept(s.kept, site.kept) && equal(s.bytes, site.original); };
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
///  Whether the name holds exactly these bytes with these permission bits,
///  this owner and these
///       extended attributes,
///  as a regular file,
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
///  A security
///       label is left out of the comparison:
///  the kernel and its policy assign it,
///  a fix carries one only where
///       the system allows it,
///  and it says so where it cannot (`Kept::same_as`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsExactly = (path, bytes, kept, step) => { const s = lstatAndRead(path, step); return s.kind === 'file' && sameKept(s.kept, kept) && equal(s.bytes, bytes); };
/// ```
pub(crate) fn holds_exactly(path: &Path, bytes: &[u8], kept: &Kept, step: &str) -> Result<bool, String> {
    let Some((held, carried)) = regular_file(path, step)? else {
        return Ok(false);
    };
    if !carried.same_as(kept) {
        return Ok(false);
    }
    return Ok(held.as_slice() == bytes);
}

/// What:
///  Whether the name holds a regular file whose digest is `digest`,
///  with these permission bits,
///       this owner and these extended attributes,
///  never following a symbolic link.
/// Why:
///   A record keeps a digest instead of the bytes,
///  and a digest alone would take a file the person saved
///       with the correction's own bytes - or narrowed to `0600` - for the fix's own.
///  The bits,
///  the
///       owner and the kind are part of whose file it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsDigest = (path, digest, kept, step) => { const f = regularFile(path, step); return f !== undefined && sameKept(kept, f.kept) && digestOf(f.bytes) === digest; };
/// ```
pub(crate) fn holds_digest(path: &Path, digest: &str, kept: &Kept, step: &str) -> Result<bool, String> {
    let Some((held, carried)) = regular_file(path, step)? else {
        return Ok(false);
    };
    if !carried.same_as(kept) {
        return Ok(false);
    }
    return Ok(crate::direct_fix_install_journal::digest_of(held.as_slice()) == digest);
}

/// What:
///  The bytes and the attributes of the regular file at `path`,
///  or nothing when the name holds no
///       regular file.
/// Why:
///   Every judgement of this route reads a name the same way:
///  the type first,
///  never through a link,
///  and
///       the bytes only once the name is known to be a regular file,
///  so a FIFO or a device at the name is never
///       opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const regularFile = (path, step) => { const s = lstatAndRead(path, step); return s.kind === 'file' ? s : undefined; };
/// ```
fn regular_file(path: &Path, step: &str) -> Result<Option<(Vec<u8>, Kept)>, String> {
    return match state_of(path, step)? {
        SiteState::File { bytes, kept } => Ok(Some((bytes, kept))),
        SiteState::Absent | SiteState::NotAFile => Ok(None),
    };
}
