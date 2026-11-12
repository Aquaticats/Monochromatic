//! What:
//!  The permission bits of an entry a carry's write replaces,
//!  which Git records one bit of and a person sets all of.
//! Why:
//!  A write of the merge's own content or of the person's bytes leaves the entry it replaces under a private name,
//! and that entry is the only place the person's read,
//!  write and special permission bits are kept:
//!  Git's index records the execute bit alone,
//! and the capture a merge keeps holds the same one.
//!  So the write takes every bit Git does not decide from the entry it replaces,
//! and only the execute bits from the plan,
//! which is what keeps a `chmod 0600` on a tracked `0644` file through a merge
//!  (a Kimi K3.1 Pro review of `1a98764d1f`,
//! its fifth finding).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keepThePermissionsOfTheEntryItReplaces(placed, replaced); // (replaced & 0o7666) | (placed & 0o111)
//! ```

use std::path::Path;

/// Every permission bit a person can set except the three the merge decides from Git's entry.
#[cfg(unix)]
pub(crate) const PERMISSION_BITS_WITHOUT_EXECUTE: u32 = 0o7666;

/// The bits of `st_mode` that say what kind of entry it describes.
#[cfg(unix)]
const THE_KIND_OF_ENTRY_BITS: u32 = 0o170000;

/// The kind `st_mode` carries for a regular file.
#[cfg(unix)]
const A_REGULAR_FILE: u32 = 0o100000;

/// What:
///  Every permission bit a person can set on the entry whose `st_mode` this is,
///  and nothing when that entry was not a regular file.
/// Why:
///  A regular file is the one entry whose mode is a person's:
///  they set its read,
/// write and special bits,
/// and Git records none of them but the execute bits.
/// A symbolic link's own
///  mode is the platform's `0777` on Linux and says nothing of the link,
/// so reading the bits of a link's
///  mode gives a set the person never made and wrote `0666` over the `0644` Git's entry implies when
///  a merge replaced a link with a file
///  (a Qwen review of `72013e1710`,
/// its first finding).
///  A directory's bits are its own and are never given to the file a merge puts where it stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePermissionBitsAPersonSet = (mode) => (mode & 0o170000) === 0o100000 ? mode & 0o7666 : undefined;
/// ```
#[cfg(unix)]
pub(crate) fn the_permission_bits_a_person_set(mode: u32) -> Option<u32> {
    if mode & THE_KIND_OF_ENTRY_BITS != A_REGULAR_FILE {
        return None;
    }
    return Some(mode & PERMISSION_BITS_WITHOUT_EXECUTE);
}


/// What:
///  Give the entry a write is about to place the permission bits of the entry it replaces,
///  with the executable bit the write decided.
/// Why:
///  Git records one permission bit of a file and no other,
/// and the capture the merge keeps holds the same bit and the bytes;
/// the read and write bits of each of the owner,
///  the group and everyone,
/// and the setuid,
/// setgid and sticky bits above them,
/// are the person's,
/// and a `chmod 0600` on a tracked `0644` file is invisible to Git and to every reading here.
/// Writing the mode
///  Git's entry implies would replace them,
/// so the bits of the entry that makes way for this write are kept and only the
///  executable bits come from the plan
/// (a Kimi K3.1 Pro review of `1a98764d1f`,
/// its fifth finding).
///  A symbolic link carries no permissions,
/// so an entry that is not a regular file gives nothing back and the write keeps what it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepThePermissionsOfWhatItReplaces(placed: string, replaced: string): void; // thePermissionBitsAPersonSet(replaced) | (placed & 0o111)
/// ```
pub(crate) fn keep_the_permissions_of_the_entry_it_replaces(placed: &Path, replaced: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let Ok(there) = std::fs::symlink_metadata(replaced) else {
            return Err(format!("{} could not be read to keep its permissions", replaced.display()));
        };
        let Ok(here) = std::fs::symlink_metadata(placed) else {
            return Err(format!("{} could not be read to give it the permissions of {}", placed.display(), replaced.display()));
        };
        if !here.file_type().is_file() {
            return Ok(());
        }
        let Some(there_bits) = the_permission_bits_a_person_set(there.permissions().mode()) else {
            return Ok(());
        };
        let bits: u32 = there_bits | (here.permissions().mode() & 0o111);
        return match std::fs::set_permissions(placed, std::fs::Permissions::from_mode(bits)) {
            Ok(()) => Ok(()),
            Err(error) => Err(format!("{} could not be given the permissions of {}: {error}", placed.display(), replaced.display())),
        };
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (placed, replaced);
        return Ok(());
    }
}
