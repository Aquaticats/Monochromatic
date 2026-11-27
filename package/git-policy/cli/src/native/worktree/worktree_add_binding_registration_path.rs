//! What:
//!  The places a registration's `gitdir` file and a destination's `.git` pointer name,
//! resolved by their text,
//! and which object a path names now.
//! Why:
//!   Git writes the destination's `.git` path absolute or relative to the registration's directory,
//! and the pair is the same either way,
//! so the resolution reads the text and refuses what climbs out of the root or reaches the
//!  registration through a link.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const target = pointerTarget(destination, pointerBytes); // throws a reason
//! const same = recordedDestination(admin, recordedGitdirLine) === join(destination, '.git');
//! ```

use crate::fs_beneath::{
    ObjectId,
    open_file_at,
    open_root,
};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// The identity of the object that the canonical path `path` names now,
/// or nothing.
///
/// Why:
/// a held handle keeps naming the directory it was opened on,
/// whatever the path names later;
/// this asks the path.
pub fn path_identity(path: &Path) -> Option<ObjectId> {
    let parent: File = open_root(path.parent()?).ok()?;
    return crate::fs_beneath::identity_at(&parent, path.file_name()?).ok();
}

/// The message for a pointer whose target reaches the registration through a link or by a non-canonical path.
pub(super) fn not_canonical(
    shown: &str,
    target: &Path,
    why: &str,
) -> String {
    return format!(
        "the pointer in {shown}/.git names {}, which cannot be resolved without a symbolic link to the registration ({why}); the add is left as Git made it: make the worktree with a canonical path, or replace the link in the pointer with the real directory",
        target.display()
    );
}

/// What:
///  The place a registration's `gitdir` file names,
/// absolute whether Git wrote it absolute or relative to the registration.
/// Why:
///   Git writes the destination's `.git` path relative to the registration's directory under `--relative-paths` and
///  `worktree.useRelativePaths`;
/// the pair is the same either way,
/// and a copy must be made for either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolve(adminDirectory, recordedGitdirLine) === join(destination, '.git');
/// ```
pub(super) fn recorded_destination(
    admin: &Path,
    recorded: &[u8],
) -> Result<PathBuf, String> {
    return match resolve_lexically(admin, recorded) {
        Ok(resolved) => Ok(resolved),
        Err(()) => Err(format!(
            "the registration {} names a place outside the file system root",
            admin.display()
        )),
    };
}

/// What:
///  `named` joined onto `base` and cleaned of `.` and `..` by the text alone,
/// or an error when it climbs out of the root.
/// Why:
///   Both bases are canonical directories opened without a link,
/// so the text decides,
/// as Git's own resolution does;
/// a link
///  in the result is refused where the result is opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.resolve(base, named);
/// ```
pub(crate) fn resolve_lexically(
    base: &Path,
    named: &[u8],
) -> Result<PathBuf, ()> {
    let mut resolved: PathBuf = PathBuf::new();
    for part in base.join(OsStr::from_bytes(named)).components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !resolved.pop() {
                    return Err(());
                }
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    return Ok(resolved);
}

/// What:
///  The registration directory a `gitdir:` pointer names,
/// resolved lexically against the destination.
/// Why:
///   The destination is a canonical path,
/// so `.` and `..` in a relative pointer resolve by their text;
/// a link anywhere in the
///  result is then caught by opening it as a canonical root,
/// and no alias is ever followed to find the registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const target = pointerTarget(destination, pointerBytes); // throws a reason
/// ```
pub(crate) fn pointer_target(
    destination: &Path,
    pointer: &[u8],
) -> Result<PathBuf, String> {
    let shown: String = destination.display().to_string();
    // Git reads the rest of the file after the prefix and drops only the newlines at its end (`read_gitfile_gently`,
    //  `setup.c`), so a path holding a newline is whole and a pointer written with CRLF names the same place.
    let line: &[u8] = without_trailing_newlines(pointer);
    let Some(named) = line.strip_prefix(b"gitdir: ") else {
        return Err(format!("{shown}/.git is not a gitdir pointer"));
    };
    return match resolve_lexically(destination, named) {
        Ok(resolved) => Ok(resolved),
        Err(()) => Err(format!(
            "the pointer in {shown}/.git climbs out of the file system root"
        )),
    };
}

/// What:
///  The bytes of a pointer without the newlines that end it.
/// Why:
///   Git drops every trailing `\n` and `\r` (`read_gitfile_gently`,
/// `setup.c`) and keeps the rest,
/// so a path that
///  holds a newline stays whole and a pointer written with CRLF names the same place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (bytes.at(-1) === 10 || bytes.at(-1) === 13) bytes = bytes.subarray(0, -1);
/// ```
fn without_trailing_newlines(bytes: &[u8]) -> &[u8] {
    let mut end: usize = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    return &bytes[..end];
}

/// The bytes of the file `name` of the held directory,
/// read through the handle and not through a path.
pub(super) fn read_through(
    directory: &File,
    name: &OsStr,
) -> std::io::Result<Vec<u8>> {
    let mut file: File = open_file_at(directory, name)?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes)?;
    return Ok(bytes);
}
