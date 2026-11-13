//! What:
//!  The permission bits of an entry a carry's write replaces,
//!  which Git records one bit of and a person sets all of,
//!  and the bits a write of the person's own entry carries back.
//! Why:
//!  A write of the merge's own content or of the person's bytes leaves the entry it replaces under a private name,
//! and that entry is the only place the person's read,
//!  write and special permission bits are kept:
//!  Git's index records the execute bit alone,
//! and the capture a merge keeps has to record the rest or the bits are gone when the path the write replaces stands
//!  empty:
//!  a `chmod 0600` on a tracked `0644` file is invisible to Git,
//! so a capture that kept only Git's mode gave a private file back as `0644`
//!  (a Kimi K3.1 Pro review of `1a98764d1f`,
//! its fifth finding;
//! a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its second finding).
//!  So the write takes every bit Git does not decide from the entry it replaces,
//! and only the execute bits from the plan,
//! and a write of the person's own entry takes every bit the capture read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keepThePermissionsOfTheEntryItReplaces(placed, replaced); // (replaced & 0o7666) | (placed & 0o111)
//! ```

use std::path::Path;

/// Every permission bit a person can set except the three the merge decides from Git's entry.
pub(crate) const PERMISSION_BITS_WITHOUT_EXECUTE: u32 = 0o7666;

/// Every permission bit a person can set on a regular file.
pub(crate) const EVERY_PERMISSION_BIT: u32 = 0o7777;

/// The bits of `st_mode` that say what kind of entry it describes.
const THE_KIND_OF_ENTRY_BITS: u32 = 0o170000;

/// The mode Git records for an executable file.
const EXECUTABLE_MODE: &str = "100755";

/// The kind `st_mode` carries for a regular file.
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
pub(crate) fn the_permission_bits_a_person_set(mode: u32) -> Option<u32> {
    if mode & THE_KIND_OF_ENTRY_BITS != A_REGULAR_FILE {
        return None;
    }
    return Some(mode & PERMISSION_BITS_WITHOUT_EXECUTE);
}

/// What:
///  Every permission bit of one entry,
///  as the file system shows it.
/// Why:
///  The capture keeps the bits of the entry it read,
/// and a write of that entry gives them back,
/// so the bits are read in one place and stored in one.
///  The three special bits go with them,
/// because Git records none of them either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePermissionBitsOf = (metadata) => metadata.mode & 0o7777;
/// ```
#[cfg(unix)]
pub(crate) fn the_permission_bits_of(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    return metadata.permissions().mode() & EVERY_PERMISSION_BIT;
}

/// What:
///  The permission bits Git's own checkout writes for one recorded mode.
/// Why:
///  Git records the executable bit and no other,
/// and a checkout of a plain entry writes `0644` where one of an executable entry writes `0755`
///  (the modes `merge_isolation_carry_place.rs` writes for the same entries).
/// A read,
/// write or special bit set differently is therefore one a checkout would not have written,
///  which is the only way to tell a mode the person set from the one Git gave them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theBitsACheckoutWrites = (mode) => mode === '100755' ? 0o755 : 0o644;
/// ```
pub(crate) fn the_bits_a_checkout_writes(mode: &str) -> u32 {
    if mode == EXECUTABLE_MODE {
        return 0o755;
    }
    return 0o644;
}

/// What:
///  The permission bits one mode string names.
/// Why:
///   A mode string is Git's kind digits and permission digits,
/// and a write of the person's own entry carries every bit they set in the permission digits:
///  `100644` names `0644` and `100600` names `0600`,
/// which is what keeps a private file private through the carry.
///  A string whose permission digits are zero names none,
/// and the plain file a checkout writes is what it takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePermissionBitsOfAMode = (mode) => (parseInt(mode, 8) & 0o7777) || theBitsACheckoutWrites(mode);
/// ```
pub(crate) fn the_permission_bits_of_a_mode(mode: &str) -> u32 {
    let recorded: u32 = u32::from_str_radix(mode, 8).unwrap_or(0);
    return match recorded & EVERY_PERMISSION_BIT {
        0 => the_bits_a_checkout_writes(mode),
        digits => digits,
    };
}

/// What:
///  Whether the entry a capture read carries permission bits Git's own checkout would not have written.
/// Why:
///  A mode the person set without touching the bytes is theirs as the bytes are,
/// and `git status` compares the executable bit alone,
/// so nothing else sees the `chmod 0600` they made.
///  Reading their bits against the ones a checkout writes is what makes the capture
///  keep them,
/// and it is what keeps their file at a path the merge deletes
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its second finding).
///  A checkout that wrote the file under another umask reads as theirs here,
/// which keeps the file rather than lose one.
///  A symbolic link's own mode is the platform's and says nothing of anyone's,
///  so a link never carries a mode a person set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carriesAModeAPersonSet = (file) => file.permissions !== undefined && kindOf(file.mode) === 'file' && file.permissions !== theBitsACheckoutWrites(file.mode);
/// ```
pub(crate) fn carries_a_mode_a_person_set(mode: &str, permissions: Option<u32>) -> bool {
    let Some(bits) = permissions else { return false };
    let Ok(recorded) = u32::from_str_radix(mode, 8) else { return false };
    if recorded & THE_KIND_OF_ENTRY_BITS != A_REGULAR_FILE {
        return false;
    }
    return bits & EVERY_PERMISSION_BIT != the_bits_a_checkout_writes(mode);
}

/// What:
///  The mode string a worktree write gives its file:
///  Git's kind and permission digits with every bit the person set in them.
/// Why:
///  The capture records the bits of the entry it read,
/// and a write of that entry back must give them again:
///  `100644` is Git's plain file and `100600` is the same file as the person made it private,
///  and a mode string is what the one placement step reads.
///  The executable bits come from the mode the plan decided,
/// because a layer's three-way choice of mode is what decides whether the file runs,
/// and every other bit is the person's.
///  A symbolic link and a submodule carry no bits of a person's,
/// so their mode is Git's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theModeWithThePersonsBits = (mode, permissions) => permissions === undefined ? mode : `${(parseInt(mode, 8) & 0o170111).toString(8)}${(permissions & 0o7666).toString(8)}`;
/// ```
pub(crate) fn the_mode_with_the_persons_bits(mode: &str, permissions: Option<u32>) -> String {
    let Some(bits) = permissions else { return String::from(mode) };
    let Ok(recorded) = u32::from_str_radix(mode, 8) else { return String::from(mode) };
    if recorded & THE_KIND_OF_ENTRY_BITS != A_REGULAR_FILE {
        return String::from(mode);
    }
    return format!("{:o}", (recorded & THE_KIND_OF_ENTRY_BITS) | (recorded & 0o111) | (bits & PERMISSION_BITS_WITHOUT_EXECUTE));
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
