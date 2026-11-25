//! What:
//!  What the kernel says about the entry at the checkout path and about its
//!  parent:
//!  the entry's device,
//!  inode,
//!  owner and mode,
//!  whether it is a directory,
//!  whether it carries the immutable attribute,
//!  whether the filesystem reports that attribute at all,
//!  and whether the entry sits on its parent's device.
//! Why:
//!  the boot's verdict is a comparison of these numbers,
//!  and each one comes from `statx(2)` rather than from a path this process
//!  resolved again
//!  (`statx(2)`,
//!  `STATX_ATTR_IMMUTABLE` and `STATX_BASIC_STATS`).
//!  The parent's device is what tells an unmounted covered entry from a mounted
//!  checkout:
//!  a mount at the path gives the entry another filesystem's device,
//!  and the flag read there belongs to the dataset, not to the underlay the
//!  decision protects.
//!  The read opens nothing that can block and follows no link at the entry:
//!  `AT_SYMLINK_NOFOLLOW` makes a symlink at the path a file rather than a
//!  directory,
//!  which the verdict refuses by name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = new LiveEntryReader().read('/home/user/Monochromatic');
//! ```

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  One reading of the entry at the checkout path and of its parent.
/// Why:
///  every field is one number the verdict compares,
///  so a control states a shape without a filesystem and the release read
///  states the kernel's own answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryFacts = { device, inode, owner, mode, directory, immutable, attributesReported, sameDeviceAsParent };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryFacts {
    /// The entry's device number.
    pub device: u64,
    /// The entry's inode number.
    pub inode: u64,
    /// The account that owns the entry.
    pub owner: u32,
    /// The entry's permission and type bits,
    /// as `statx` reports them.
    pub mode: u32,
    /// Whether the entry is a directory.
    pub directory: bool,
    /// Whether the entry carries the kernel's immutable attribute.
    pub immutable: bool,
    /// Whether the filesystem reports the immutable attribute at all,
    /// which a filesystem without the attribute answers no to.
    pub attributes_reported: bool,
    /// Whether the entry is on the device its parent is on,
    /// which an unmounted covered entry is and a mount at the path is not.
    pub same_device_as_parent: bool,
}

/// What:
///  Why the entry could not be read.
/// Why:
///  a reader that answered a made-up shape would turn an unreadable path into a
///  verdict,
///  so each cause names the path and the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryReadRefusal = { kind: 'not-text' | 'call'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryReadRefusal {
    /// The path's bytes are not text or hold a NUL,
    /// so no `statx` call can name it.
    PathNotText {
        /// The path as given.
        path: PathBuf,
    },
    /// The entry has no parent,
    /// so nothing could compare the two devices.
    NoParent {
        /// The path as given.
        path: PathBuf,
    },
    /// The kernel refused the call.
    Call {
        /// Which read it was,
        /// `the entry` or `its parent`.
        what: &'static str,
        /// The path it read.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
}

impl EntryReadRefusal {
    /// The refusal in words,
    /// naming the path and what to inspect.
    pub fn message(&self) -> String {
        return match self {
            EntryReadRefusal::PathNotText { path } => format!(
                "the checkout path `{}` is not text or holds a NUL byte, so the kernel facts that decide the covered entry's protection cannot be read. Nothing was attached, mounted or changed. Ways forward: give the checkout path as UTF-8 text.",
                path.display()
            ),
            EntryReadRefusal::NoParent { path } => format!(
                "the checkout path `{}` has no parent, so the read cannot tell a covered entry from a mount at the path. Nothing was attached, mounted or changed. Ways forward: name the folder the checkout is mounted at, not a filesystem root.",
                path.display()
            ),
            EntryReadRefusal::Call { what, path, reason } => format!(
                "{what} of `{}` could not be read ({reason}), so whether the checkout's underlay is protected is unknown. Nothing was attached, mounted or changed. Ways forward: read the path by hand as root with `stat --format='%d %i %u %a %A'` and confirm it exists and is reachable; a path that is gone needs the migration or a restore before it is served.",
                path.display()
            ),
        };
    }
}

/// What:
///  One read of the entry at a checkout path.
/// Why:
///  the release read is a raw `statx(2)` call a control cannot make answer,
///  so a control supplies a shape and the verdict is proved on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface EntryReader { read(path): EntryFacts }
/// ```
pub trait EntryReader {
    /// The kernel's facts about the entry at this path and about its parent.
    ///
    /// # Errors
    /// The path is not text,
    /// has no parent,
    /// or the kernel refused a read.
    fn read(&self, path: &Path) -> Result<EntryFacts, EntryReadRefusal>;
}

/// The read the release boot makes.
///
/// Why:
///  it asks the kernel at the moment of the check, so a flag cleared by a
///  rollback or a mount since the last boot is seen.
pub struct LiveEntryReader;

impl EntryReader for LiveEntryReader {
    fn read(&self, path: &Path) -> Result<EntryFacts, EntryReadRefusal> {
        let parent: &Path = path.parent().filter(|one| return !one.as_os_str().is_empty()).ok_or_else(|| {
            return EntryReadRefusal::NoParent { path: path.to_path_buf() };
        })?;
        let entry: libc::statx = statx_one("the entry", path, libc::AT_SYMLINK_NOFOLLOW)?;
        let above: libc::statx = statx_one("its parent", parent, 0)?;
        let mode: u32 = u32::from(entry.stx_mode);
        let directory: bool = mode & libc::S_IFMT == libc::S_IFDIR;
        let attributes: u64 = u64::from(entry.stx_attributes);
        let immutable_bit: u64 = libc::STATX_ATTR_IMMUTABLE as u64;
        let attributes_reported: bool = u64::from(entry.stx_attributes_mask) & immutable_bit != 0;
        let immutable: bool = attributes & immutable_bit != 0;
        return Ok(EntryFacts {
            device: device_number(&entry),
            inode: entry.stx_ino,
            owner: entry.stx_uid,
            mode,
            directory,
            immutable,
            attributes_reported,
            same_device_as_parent: device_number(&entry) == device_number(&above),
        });
    }
}

/// The device number `statx` reports,
/// as the kernel's own major and minor pair.
///
/// Why:
///  the pair is what distinguishes a mount at the path from the underlay entry
///  on the parent's filesystem, and comparing the pair needs no `makedev`.
fn device_number(facts: &libc::statx) -> u64 {
    return (u64::from(facts.stx_dev_major) << 32) | u64::from(facts.stx_dev_minor);
}

/// One `statx(2)` call for the fields this module reads.
///
/// Why:
///  one call shape serves both the entry and its parent,
///  so the two answers are comparable field by field.
fn statx_one(what: &'static str, path: &Path, flags: libc::c_int) -> Result<libc::statx, EntryReadRefusal> {
    let text: CString = CString::new(path.as_os_str().as_bytes()).map_err(|_nul| return EntryReadRefusal::PathNotText { path: path.to_path_buf() })?;
    let mut facts: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: the path is a NUL-terminated string that outlives the call, the flags are the two the read names, and the
    // answer is written into `facts`, which this function owns.
    let answered: libc::c_int = unsafe { libc::statx(libc::AT_FDCWD, text.as_ptr(), flags, libc::STATX_BASIC_STATS, &mut facts) };
    if answered != 0 {
        return Err(EntryReadRefusal::Call { what, path: path.to_path_buf(), reason: std::io::Error::last_os_error().to_string() });
    }
    return Ok(facts);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "underlay_entry_read_tests.rs"]
mod tests;
