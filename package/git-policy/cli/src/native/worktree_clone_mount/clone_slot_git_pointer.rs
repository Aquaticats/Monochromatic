//! What:
//!  The first observation's read of the slot's `.git` pointer:
//!  a bounded,
//! regular,
//! no-follow file holding one `gitdir:` line whose path
//!  names this checkout's `worktrees/<slot>` registration.
//! Why:
//!  The design requires the root-recorded underlay to "match the claimed
//!  registration slot and hold only the expected Git pointer",
//!  and to treat the pointer as untrusted:
//!  check only a bounded,
//! regular,
//! no-follow pointer file and never resolve a
//!  path from its contents as root
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The check is what tells a slot Git registered from a folder an account
//!  emptied or replaced,
//!  and it resolves nothing:
//!  the path's components are read as text and compared,
//!  so a pointer naming another checkout,
//!  another slot,
//!  or a `..` component is refused rather than followed.
//!  It is the first observation and not proof the slot has not changed since
//!  Git created it:
//!  the recorded identity is what later walks compare.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registration = readSlotGitPointer(slotFolder, '/home/user/Monochromatic', 'one'); // '/home/user/Monochromatic/.git/worktrees/one'
//! ```

use crate::fs_beneath::open_file_at;
use std::ffi::OsStr;
use std::io::Read;
use std::os::unix::fs::FileTypeExt;
use std::path::{
    Component,
    Path,
    PathBuf,
};

/// The pointer file's name inside a registered worktree slot.
pub const GIT_POINTER_NAME: &str = ".git";

/// The prefix every Git worktree pointer file begins with.
pub const GITDIR_PREFIX: &str = "gitdir: ";

/// The component Git puts above a worktree's own registration.
pub const WORKTREES_COMPONENT: &str = "worktrees";

/// The most bytes the pointer is read as,
/// which is far more than a registration path needs and keeps a surprise file
/// from filling memory.
pub const POINTER_BOUND: usize = 4_096;

/// What:
///  Why a slot holds no Git pointer this route may act on.
/// Why:
///  An absent pointer is a slot Git never registered,
///  a link or a named pipe is a file this route refuses to wait on or follow,
///  a pointer naming another place is a slot whose registration moved,
///  and a person fixes each somewhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitPointerRefusal = { kind: 'absent' | 'not-a-regular-file' | 'too-long' | 'not-a-pointer' | 'another-registration'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitPointerRefusal {
    /// The slot holds no `.git`.
    Absent {
        /// The slot's name.
        slot: String,
    },
    /// The `.git` entry could not be opened or read.
    Unreadable {
        /// The slot's name.
        slot: String,
        /// What the call answered.
        reason: String,
    },
    /// The `.git` entry is a link,
    /// a named pipe,
    /// a device or a folder rather than a regular file.
    NotARegularFile {
        /// The slot's name.
        slot: String,
        /// What the entry is instead.
        found: &'static str,
    },
    /// The pointer is longer than the route reads.
    TooLong {
        /// The slot's name.
        slot: String,
        /// The bound.
        bound: usize,
    },
    /// The file is no one `gitdir:` line with an absolute,
    /// plain path.
    NotAPointer {
        /// The slot's name.
        slot: String,
        /// What about it is not the expected shape.
        said: String,
    },
    /// The pointer names another registration than this slot's.
    AnotherRegistration {
        /// The slot's name.
        slot: String,
        /// The path the pointer names.
        points_at: PathBuf,
        /// The registration this checkout gives the slot.
        wanted: PathBuf,
    },
}

impl GitPointerRefusal {
    /// The refusal in words,
    /// naming the slot,
    /// the pointer and the way forward.
    pub fn message(&self) -> String {
        let cause: String = match self {
            GitPointerRefusal::Absent {
                slot,
            } => format!(
                "the slot `{slot}` holds no `.git`, so no Git registration says it is a worktree of this checkout"
            ),
            GitPointerRefusal::Unreadable {
                slot,
                reason,
            } => format!("the slot `{slot}` holds a `.git` that could not be read ({reason})"),
            GitPointerRefusal::NotARegularFile {
                slot,
                found,
            } => format!(
                "the slot `{slot}` holds a `.git` that is {found} rather than one regular file, and this route reads only a regular pointer file"
            ),
            GitPointerRefusal::TooLong {
                slot,
                bound,
            } => format!(
                "the slot `{slot}` holds a `.git` longer than the {bound} bytes this route reads"
            ),
            GitPointerRefusal::NotAPointer {
                slot,
                said,
            } => {
                format!("the slot `{slot}` holds a `.git` that is no Git worktree pointer: {said}")
            }
            GitPointerRefusal::AnotherRegistration {
                slot,
                points_at,
                wanted,
            } => format!(
                "the slot `{slot}` holds a `.git` naming `{}`, and this checkout registers the slot at `{}`",
                points_at.display(),
                wanted.display()
            ),
        };
        return format!(
            "{cause}. Nothing was mounted, attached or recorded for it, and the sweep's next trigger reads the slot again. Ways forward: remove the slot or let Git register it again with `git worktree add`, and never resolve the pointer's contents as root."
        );
    }
}

/// What:
///  The registration path this checkout gives one slot.
/// Why:
///  The expected pointer is a value this route builds,
///  so the compare is against the checkout's own Git directory rather than
///  against whatever the slot says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wanted = `${checkout}/.git/worktrees/${slot}`;
/// ```
pub fn the_registration_of(
    checkout: &Path,
    slot: &str,
) -> PathBuf {
    return checkout
        .join(GIT_POINTER_NAME)
        .join(WORKTREES_COMPONENT)
        .join(slot);
}

/// What:
///  Whether one pointer path is a plain absolute path with no component this
///  route resolves.
/// Why:
///   A `..` or a `.` component,
///  a device or a pipe spelled as a path,
///  and a relative path are all ways a pointer could steer root,
///  and none of them is what Git writes for a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plain = components.every((c) => c === 'rootdir' || isNormal(c));
/// ```
fn pointer_path_refusal(points_at: &Path) -> Option<String> {
    if !points_at.is_absolute() {
        return Some(String::from("its path is not absolute"));
    }
    for component in points_at.components() {
        let plain: bool = matches!(component, Component::RootDir | Component::Normal(_));
        if !plain {
            return Some(format!(
                "its path holds `{}`, which this route does not resolve",
                component.as_os_str().to_string_lossy()
            ));
        }
    }
    return None;
}

/// What:
///  The path one pointer text names,
///  read as text and resolved nowhere.
/// Why:
///  The file's whole content is untrusted,
///  so this accepts exactly one line,
///  one known prefix and one plain absolute path,
///  and answers what about it differs otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = parsePointer('gitdir: /home/user/Monochromatic/.git/worktrees/one\n'); // a PathBuf or a reason
/// ```
fn parse_pointer(text: &str) -> Result<PathBuf, String> {
    let line: &str = match text.strip_suffix('\n') {
        Some(line) => line,
        None => return Err(String::from("it has no final newline")),
    };
    if line.contains('\n') {
        return Err(String::from("it holds more than one line"));
    }
    let Some(named) = line.strip_prefix(GITDIR_PREFIX) else {
        return Err(format!(
            "its first line does not begin with `{GITDIR_PREFIX}`"
        ));
    };
    if named.is_empty() {
        return Err(String::from("it names no path"));
    }
    let points_at: PathBuf = PathBuf::from(named);
    if let Some(reason) = pointer_path_refusal(points_at.as_path()) {
        return Err(reason);
    }
    return Ok(points_at);
}

/// What:
///  Read one slot's Git pointer and return the registration it names.
/// Why:
///  The first observation must check what the slot holds before it records the
///  folder's identity,
///  because an identity pinned for a folder that never held a registration
///  would authorize an attach at a slot Git does not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registration = readTheSlotGitPointer(slotFolder, checkout, 'one');
/// ```
///
/// # Errors
/// The pointer is absent,
/// is not one regular readable file,
/// is longer than the bound,
/// is not one `gitdir:` line,
/// or names a registration other than this checkout's slot.
pub fn read_the_slot_git_pointer(
    slot_folder: &std::fs::File,
    checkout: &Path,
    slot: &str,
) -> Result<PathBuf, GitPointerRefusal> {
    let name: &OsStr = OsStr::new(GIT_POINTER_NAME);
    let mut file: std::fs::File = match open_file_at(slot_folder, name) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(GitPointerRefusal::Absent {
                slot: String::from(slot),
            });
        }
        Err(error) => {
            return Err(GitPointerRefusal::Unreadable {
                slot: String::from(slot),
                reason: error.to_string(),
            });
        }
    };
    let metadata: std::fs::Metadata = file.metadata().map_err(|error| {
        return GitPointerRefusal::Unreadable {
            slot: String::from(slot),
            reason: error.to_string(),
        };
    })?;
    let kind: std::fs::FileType = metadata.file_type();
    if !kind.is_file() {
        let found: &'static str = if kind.is_dir() {
            "a folder"
        } else if kind.is_symlink() {
            "a link"
        } else if kind.is_fifo() {
            "a named pipe"
        } else {
            "no regular file"
        };
        return Err(GitPointerRefusal::NotARegularFile {
            slot: String::from(slot),
            found,
        });
    }
    let mut bytes: Vec<u8> = Vec::new();
    let read: usize = file
        .by_ref()
        .take((POINTER_BOUND + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return GitPointerRefusal::Unreadable {
                slot: String::from(slot),
                reason: error.to_string(),
            };
        })?;
    if read > POINTER_BOUND {
        return Err(GitPointerRefusal::TooLong {
            slot: String::from(slot),
            bound: POINTER_BOUND,
        });
    }
    let text: String = String::from_utf8(bytes).map_err(|_error| {
        return GitPointerRefusal::NotAPointer {
            slot: String::from(slot),
            said: String::from("it is not UTF-8 text"),
        };
    })?;
    let points_at: PathBuf = parse_pointer(text.as_str()).map_err(|said| {
        return GitPointerRefusal::NotAPointer {
            slot: String::from(slot),
            said,
        };
    })?;
    let wanted: PathBuf = the_registration_of(checkout, slot);
    if points_at != wanted {
        return Err(GitPointerRefusal::AnotherRegistration {
            slot: String::from(slot),
            points_at,
            wanted,
        });
    }
    return Ok(points_at);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_slot_git_pointer_tests.rs"]
mod tests;
