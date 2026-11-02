//! What:
//!  The binding of one `git worktree add` to the objects real Git made for it, and the removal of those objects, and only
//!  those, when the add did not complete: the destination directory and the administrative directory that registers it.
//! Why:
//!  A pathname recorded before Git ran proves nothing about who made the directory at it later: another process, real Git
//!  run by a person, or a reused path can put a valid worktree there. The wrapper therefore binds the add to the identities
//!  (device, inode, birth time) of the two directories and to the bytes of the `.git` pointer right after Git returned,
//!  writes that binding to the add's durable record, and deletes an object only when its identity is still the recorded one,
//!  through a quarantine rename, with the object held by handle. A record that was never bound (a kill before or during
//!  Git) deletes nothing. Everything uncertain is kept and reported.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bound = await bindRegistration(commonDir, destination); // throws when the pair is not proven
//! await claim.recordBinding(bound.binding);
//! const outcome = await removeBound(bound.binding); // 'removed' | 'kept' with the reason
//! ```

use super::fs_beneath::{Contents, ObjectId, Owned, identity_of, open_root, remove_owned_at};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What an add bound itself to: the two directories Git made and the pointer it wrote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    /// The destination, a canonical path.
    pub destination: PathBuf,
    /// The identity of the destination directory when Git returned.
    pub destination_id: ObjectId,
    /// The administrative directory, a canonical path directly below the repository's `worktrees`.
    pub admin: PathBuf,
    /// The identity of the administrative directory when Git returned.
    pub admin_id: ObjectId,
    /// The bytes of the `.git` file Git wrote in the destination.
    pub pointer: Vec<u8>,
}

/// A binding with the two directories held open.
#[derive(Debug)]
pub struct Bound {
    /// The binding, for the record.
    pub binding: Binding,
    /// The destination directory as it was when it was bound.
    pub destination: File,
    /// The administrative directory as it was when it was bound.
    pub admin: File,
}

/// Lower-case hexadecimal of bytes.
fn hex(bytes: &[u8]) -> String {
    return bytes.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect();
}

/// The bytes of lower-case hexadecimal text.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    return (0..text.len()).step_by(2).map(|start: usize| { return u8::from_str_radix(text.get(start..start + 2)?, 16).ok(); }).collect();
}

/// The text of a path for a record: hexadecimal, so that a newline in a name cannot forge a field.
pub fn encode_path(path: &Path) -> String {
    return hex(path.as_os_str().as_bytes());
}

/// The path of a record's text.
pub fn decode_path(text: &str) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    return Some(PathBuf::from(std::ffi::OsString::from_vec(unhex(text)?)));
}

impl Binding {
    /// The record text of the binding.
    pub fn encode(&self) -> String {
        return format!(
            "destination={}\nphase=registered\ndestination_id={}\nadmin={}\nadmin_id={}\npointer={}\n",
            encode_path(self.destination.as_path()),
            self.destination_id.encode(),
            encode_path(self.admin.as_path()),
            self.admin_id.encode(),
            hex(self.pointer.as_slice()),
        );
    }
}

/// What a record says about an add.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Recorded {
    /// The add was planned and not bound: no object is known to be the add's.
    Planned(PathBuf),
    /// The add was bound to the objects Git made.
    Registered(Binding),
}

/// What:
///  Read the text of an add's record.
/// Why:
///   A record that does not parse in full is treated as planned only when its destination field parses; anything else is
///  unreadable and authorizes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeRecord(text: string): Recorded | undefined;
/// ```
pub fn decode_record(text: &str) -> Option<Recorded> {
    let field = |key: &str| -> Option<&str> { return text.lines().find_map(|line: &str| { return line.strip_prefix(key).and_then(|rest: &str| { return rest.strip_prefix('='); }); }); };
    let destination: PathBuf = decode_path(field("destination")?)?;
    if field("phase")? == "planned" {
        return Some(Recorded::Planned(destination));
    }
    if field("phase")? != "registered" {
        return None;
    }
    return Some(Recorded::Registered(Binding {
        destination,
        destination_id: ObjectId::decode(field("destination_id")?)?,
        admin: decode_path(field("admin")?)?,
        admin_id: ObjectId::decode(field("admin_id")?)?,
        pointer: unhex(field("pointer")?)?,
    }));
}

/// The record text of a planned add.
pub fn planned_record(destination: &Path) -> String {
    return format!("destination={}\nphase=planned\n", encode_path(destination));
}

/// The identity of the object that the canonical path `path` names now, or nothing.
///
/// Why: a held handle keeps naming the directory it was opened on, whatever the path names later; this asks the path.
pub fn path_identity(path: &Path) -> Option<ObjectId> {
    let parent: File = open_root(path.parent()?).ok()?;
    return super::fs_beneath::identity_at(&parent, path.file_name()?).ok();
}

/// What:
///  Bind an add to the directories Git just made for it, after proving that they belong together and to this repository.
/// Why:
///   The pointer must name an administrative directory directly below this repository's `worktrees`, and that directory's
///  `gitdir` must name the pointer back; only then are the identities of both directories and the pointer bytes captured,
///  with both directories held open. A pointer edited to name another worktree's administrative directory fails the
///  back-link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = await bindRegistration(commonDir, destination); // throws a reason
/// ```
pub fn bind_registration(common_dir: &Path, destination: &Path) -> Result<Bound, String> {
    let shown: String = destination.display().to_string();
    let held: File = open_root(destination).map_err(|error| { return format!("{shown} is not a real directory ({error})"); })?;
    let pointer_path: PathBuf = destination.join(".git");
    let pointer: Vec<u8> = std::fs::read(pointer_path.as_path()).map_err(|error| { return format!("{shown}/.git is unreadable ({error})"); })?;
    let line: &[u8] = pointer.strip_suffix(b"\n").unwrap_or(pointer.as_slice());
    let named: &[u8] = line.strip_prefix(b"gitdir: ").ok_or_else(|| { return format!("{shown}/.git is not a gitdir pointer"); })?;
    let admin_path: PathBuf = std::fs::canonicalize(destination.join(OsStr::from_bytes(named))).map_err(|error| { return format!("the administrative directory of {shown} is unreadable ({error})"); })?;
    let worktrees: PathBuf = std::fs::canonicalize(common_dir.join("worktrees")).map_err(|error| { return format!("the worktrees of the repository are unreadable ({error})"); })?;
    if admin_path.parent() != Some(worktrees.as_path()) {
        return Err(format!("{shown} is not registered in this repository"));
    }
    let admin: File = open_root(admin_path.as_path()).map_err(|error| { return format!("the registration of {shown} cannot be opened ({error})"); })?;
    let recorded: Vec<u8> = std::fs::read(admin_path.join("gitdir")).map_err(|error| { return format!("the registration of {shown} is unreadable ({error})"); })?;
    let recorded_line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
    if recorded_line != pointer_path.as_os_str().as_bytes() {
        return Err(format!("the registration {} does not name {shown}", admin_path.display()));
    }
    let binding: Binding = Binding {
        destination: destination.to_path_buf(),
        destination_id: identity_of(&held).map_err(|error| { return format!("{shown} cannot be examined ({error})"); })?,
        admin: admin_path,
        admin_id: identity_of(&admin).map_err(|error| { return format!("the registration cannot be examined ({error})"); })?,
        pointer,
    };
    return Ok(Bound { binding, destination: held, admin });
}

/// What a removal of a bound add did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Removal {
    /// Both objects are gone, or were gone already.
    Removed,
    /// Something was kept, for this reason; the add's record must stay when the reason is a failure.
    Kept {
        /// Why something stayed.
        reason: String,
        /// Whether the objects are no longer the add's (nothing is pending) rather than the removal having failed.
        replaced: bool,
    },
}

/// Remove the directory `path` if it is the object with identity `expected`, through its canonical parent.
fn remove_directory_owned(path: &Path, expected: ObjectId) -> Result<Owned, (String, bool)> {
    let parent: &Path = path.parent().ok_or_else(|| { return (format!("{} has no parent", path.display()), false); })?;
    let name: &OsStr = path.file_name().ok_or_else(|| { return (format!("{} has no name", path.display()), false); })?;
    let held: File = match open_root(parent) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Owned::Absent),
        Err(error) => return Err((format!("{} cannot be opened ({error})", parent.display()), false)),
    };
    return remove_owned_at(&held, name, expected, Contents::Everything).map_err(|error| { return (format!("{} cannot be removed ({error})", path.display()), false); });
}

/// What:
///  Remove the destination and the registration an add was bound to, each only if it is still the bound object.
/// Why:
///   Destination first, registration last, as Git's own junk removal does, and the registration only once the destination
///  is gone. A path that now holds another object is not the
///  add's and stays, with a reason that says so; a removal that failed leaves what it could not remove and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await removeBound(binding);
/// ```
pub fn remove_bound(binding: &Binding) -> Removal {
    let mut kept: Vec<String> = Vec::new();
    let mut replaced: bool = true;
    for (index, (path, id)) in [(binding.destination.as_path(), binding.destination_id), (binding.admin.as_path(), binding.admin_id)].into_iter().enumerate() {
        // The registration serves whatever stands at the destination, so it goes only after the destination did.
        if index == 1 && !kept.is_empty() {
            kept.push(format!("{} stays because the destination was not removed", path.display()));
            continue;
        }
        match remove_directory_owned(path, id) {
            Ok(Owned::Removed | Owned::Absent) => {}
            Ok(Owned::Kept(reason)) => {
                // A mismatch means the object is not the add's; a failure to empty or remove it is a pending removal.
                replaced &= reason.contains("is not the object that was made");
                kept.push(reason);
            }
            Err((reason, _unused)) => {
                replaced = false;
                kept.push(reason);
            }
        }
    }
    return if kept.is_empty() { Removal::Removed } else { Removal::Kept { reason: kept.join("; "), replaced } };
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_rollback_tests.rs"]
mod tests;
