//! What:
//!  The entry a carry's write makes:
//!  a file created as Git's own checkout creates one,
//!  a symbolic link,
//!  and the permission bits a person set on their own entry.
//! Why:
//!  Every write of a carry,
//!  a clear,
//!  a putting back and a copy beside a path makes an entry the same way,
//!  and the mode those entries end with is the thing a merge must decide exactly as Git decides it:
//!  a file is created with the mode Git creates one with and the file system narrows it by the umask and any
//!  default ACL of the directory,
//!  so a merge writes `0644` under the usual mask and `0600` under a private one,
//!  where naming a finished mode would write `0644` under both
//!  (`SPEC.md`,
//!  "Carry edits forward").
//!  The placing of one file,
//!  which is where this is called from,
//!  is `merge_isolation_carry_place.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeEntry(named, bytes, mode, thePersonsBits); // symlink(target) for 120000, else create(...0666|0777) and chmod when the bits are the person's
//! ```

use std::path::Path;

/// The mode a symbolic link is recorded with.
pub(crate) const SYMBOLIC_LINK_MODE: &str = "120000";

/// The bits a checkout creates a plain file with,
/// before the file system narrows them.
pub(crate) const THE_BITS_A_CHECKOUT_CREATES_WITH: u32 = 0o666;

/// The bits a checkout creates an executable file with,
/// before the file system narrows them.
pub(crate) const THE_BITS_A_CHECKOUT_CREATES_EXECUTABLE_WITH: u32 = 0o777;

/// What:
///  Whether a mode string is one of Git's own for a regular file,
///  whose executable bit alone a person can set.
/// Why:
///  The executable bits are Git's to record,
///  so they are read from the plan's mode and never from the person's entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayRun = (mode) => (parseInt(mode, 8) & 0o111) !== 0;
/// ```
pub(crate) fn may_run(mode: &str) -> bool {
    return u32::from_str_radix(mode, 8).unwrap_or(0) & 0o111 != 0;
}

/// What:
///  Set the permission bits of a file a write just made,
///  with the failure the attempt made.
/// Why:
///   A file whose bits could not be set is not the entry the write promised,
/// so the attempt fails and is reported rather than
///  placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setMode(file, bits): Promise<void>; // chmod(file, bits)
/// ```
pub(crate) fn set_mode(file: &Path, bits: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return match std::fs::set_permissions(file, std::fs::Permissions::from_mode(bits)) {
            Ok(()) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (file, bits);
        return Ok(());
    }
}

/// What:
///  A file's mode is the one Git's own checkout gives it,
///  or the permission bits a person set on their own entry.
/// Why:
///  A path the merge writes is created as Git creates it:
///  `0666`,
///  or `0777` when its entry is executable,
///  with the file system narrowing that by the process's umask and any default ACL of the directory,
///  which is the one place those bits are decided,
///  so the write names no mode of its own there.
///  A path the merge writes nothing at and the person's own entry stands at keeps their bits,
///  which the entry it replaces does not hold:
///  Git records the executable bit alone,
///  so a `chmod 0600` on a tracked `0644` file is nowhere in the index and a write of Git's own mode
///  would widen it
///  (`SPEC.md`,
///  "Carry edits forward").
/// What:
///  Write one entry's bytes or its link target under a name beside its path,
///  with the mode Git records for it,
///  and the permission bits a person set on it when they have any.
/// Why:
///  The bytes of an entry are written where nothing else writes before they take a path,
/// so a kill leaves either
///  what was there or the whole entry;
/// a copy kept beside a path is written the same way,
/// and both go through
///  one step so a link is made in both cases and never followed.
///  A file is created with the mode Git's own checkout creates it with and the file system narrows it,
///  so a merge writes `0644` under the usual umask,
///  `0600` under a private one,
///  and whatever a default ACL of the directory allows,
///  exactly as Git's own `open` of the same entry does
///  (`SPEC.md`,
///  "Carry edits forward").
///  A file whose bytes are the person's own entry keeps every bit the capture read,
///  which is set after the creation because the entry they set them on is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeEntry(named, bytes, mode, bits): void; // symlink(target) for 120000, else create(...0666|0777) and chmod when bits are the person's
/// ```
pub(crate) fn write_entry(named: &Path, bytes: &[u8], mode: &str, the_persons_bits: Option<u32>) -> Result<(), String> {
    if mode == SYMBOLIC_LINK_MODE {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            return match std::os::unix::fs::symlink(std::ffi::OsStr::from_bytes(bytes), named) {
                Ok(()) => Ok(()),
                Err(error) => Err(error.to_string()),
            };
        }
        // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
        #[cfg(not(unix))]
        {
            let _ = (named, bytes);
            return Err(String::from("a symbolic link cannot be made on this platform"));
        }
    }
    if let Err(error) = create_file(named, bytes, the_persons_bits, may_run(mode)) {
        let _left = std::fs::remove_file(named);
        return Err(error.to_string());
    }
    let Some(bits) = the_persons_bits else { return Ok(()) };
    return set_mode(named, bits & EVERY_PERMISSION_BIT);
}

/// What:
///  Every permission bit of an entry.
/// Why:
///  A person's mode is the read,
/// write and execute bits of each of the owner,
/// the group and everyone,
///  and the setuid,
/// setgid and sticky bits above them,
/// which are the ones Git records none of.
pub(crate) const EVERY_PERMISSION_BIT: u32 = 0o7777;

/// What:
///  Create a file with the bytes and the mode Git's own checkout creates one with,
///  so the file system narrows that mode by the umask and any default ACL as it does for Git,
///  or with the person's own bits when the file holds their entry.
/// Why:
///  Git creates a plain entry with `0666` and an executable one with `0777` and lets the file system decide the rest
///  (`entry.c`);
/// naming the finished mode instead would write `0644` under a umask of `077`,
///  where Git writes `0600`.
///  A file that holds the person's own entry is created narrow and only widened to what they set,
///  so their `0600` is never readable by others,
///  not even between the write and the change of its bits.
///  The name is new,
///  so a file already standing there is never written through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createFile(named, bytes, thePersonsBits, executable): void; // open(O_CREAT|O_EXCL|O_WRONLY, thePersonsBits ?? (executable ? 0o777 : 0o666)), write, close
/// ```
pub(crate) fn create_file(named: &Path, bytes: &[u8], the_persons_bits: Option<u32>, executable: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt as _;
        let gits: u32 = if executable { THE_BITS_A_CHECKOUT_CREATES_EXECUTABLE_WITH } else { THE_BITS_A_CHECKOUT_CREATES_WITH };
        // A file that holds the person's own entry is created narrow and only widened to what they set, so their 0600 is never
        //  readable by others, not even between the write and the change of its bits.
        let mode: u32 = match the_persons_bits {
            Some(bits) => bits & EVERY_PERMISSION_BIT,
            None => gits,
        };
        let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(mode).open(named)?;
        file.write_all(bytes)?;
        return Ok(());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (the_persons_bits, executable);
        return std::fs::write(named, bytes);
    }
}
