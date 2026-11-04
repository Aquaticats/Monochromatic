//! What:
//!  The binding of one `git worktree add` to the objects real Git made for it (the destination directory and the
//!  administrative directory that registers it), the record that carries the binding, and the check that the binding still
//!  holds.
//! Why:
//!  The wrapper completes an add in place, through the bound objects, and never deletes or unregisters a worktree: a path
//!  can be changed by another process between any check and any act, so no pathname or identity recorded earlier proves the
//!  right to remove something later. The binding is what a fast rung and the fallback checkout verify before they write into
//!  the worktree, and what the report of an interrupted add names. It is captured right after Git returned, and an add
//!  whose pair cannot be proven is left exactly as Git made it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bound = await bindRegistration(commonDir, destination); // throws when the pair is not proven
//! await claim.recordBinding(bound.binding);
//! await verifyAssociation(bound); // throws when the objects are no longer where and what they were
//! ```

use crate::fs_beneath::{ObjectId, identity_of, open_file_at, open_root};
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
    return crate::fs_beneath::identity_at(&parent, path.file_name()?).ok();
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
    let pointer: Vec<u8> = read_through(&held, OsStr::new(".git")).map_err(|error| { return format!("{shown}/.git is unreadable ({error})"); })?;
    let pointer_path: PathBuf = destination.join(".git");
    let admin_path: PathBuf = pointer_target(destination, pointer.as_slice())?;
    // The target is opened as a canonical root first: a pointer through a link is refused here, with its own message.
    let admin: File = open_root(admin_path.as_path()).map_err(|error| { return not_canonical(shown.as_str(), admin_path.as_path(), error.to_string().as_str()); })?;
    let worktrees: PathBuf = std::fs::canonicalize(common_dir.join("worktrees")).map_err(|error| { return format!("the worktrees of the repository are unreadable ({error})"); })?;
    if admin_path.parent() != Some(worktrees.as_path()) {
        return Err(format!("{shown} is not registered in this repository"));
    }
    let recorded: Vec<u8> = read_through(&admin, OsStr::new("gitdir")).map_err(|error| { return format!("the registration of {shown} is unreadable ({error})"); })?;
    let recorded_line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
    if recorded_destination(admin_path.as_path(), recorded_line)?.as_path() != pointer_path.as_path() {
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

/// The message for a pointer whose target reaches the registration through a link or by a non-canonical path.
fn not_canonical(shown: &str, target: &Path, why: &str) -> String {
    return format!("the pointer in {shown}/.git names {}, which cannot be resolved without a symbolic link to the registration ({why}); the add is left as Git made it: make the worktree with a canonical path, or replace the link in the pointer with the real directory", target.display());
}

/// What:
///  The place a registration's `gitdir` file names, absolute whether Git wrote it absolute or relative to the registration.
/// Why:
///   Git writes the destination's `.git` path relative to the registration's directory under `--relative-paths` and
///  `worktree.useRelativePaths`; the pair is the same either way, and a copy must be made for either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolve(adminDirectory, recordedGitdirLine) === join(destination, '.git');
/// ```
fn recorded_destination(admin: &Path, recorded: &[u8]) -> Result<PathBuf, String> {
    return resolve_lexically(admin, recorded).map_err(|()| { return format!("the registration {} names a place outside the file system root", admin.display()); });
}

/// What:
///  `named` joined onto `base` and cleaned of `.` and `..` by the text alone, or an error when it climbs out of the root.
/// Why:
///   Both bases are canonical directories opened without a link, so the text decides, as Git's own resolution does; a link
///  in the result is refused where the result is opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.resolve(base, named);
/// ```
fn resolve_lexically(base: &Path, named: &[u8]) -> Result<PathBuf, ()> {
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
///  The registration directory a `gitdir:` pointer names, resolved lexically against the destination.
/// Why:
///   The destination is a canonical path, so `.` and `..` in a relative pointer resolve by their text; a link anywhere in the
///  result is then caught by opening it as a canonical root, and no alias is ever followed to find the registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const target = pointerTarget(destination, pointerBytes); // throws a reason
/// ```
fn pointer_target(destination: &Path, pointer: &[u8]) -> Result<PathBuf, String> {
    let shown: String = destination.display().to_string();
    let line: &[u8] = pointer.strip_suffix(b"\n").unwrap_or(pointer);
    let named: &[u8] = line.strip_prefix(b"gitdir: ").ok_or_else(|| { return format!("{shown}/.git is not a gitdir pointer"); })?;
    return resolve_lexically(destination, named).map_err(|()| { return format!("the pointer in {shown}/.git climbs out of the file system root"); });
}

/// The bytes of the file `name` of the held directory, read through the handle and not through a path.
fn read_through(directory: &File, name: &OsStr) -> std::io::Result<Vec<u8>> {
    let mut file: File = open_file_at(directory, name)?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes)?;
    return Ok(bytes);
}

/// What:
///  Check that the objects an add was bound to are still where and what they were: the destination and the registration
///  still have the bound identities at their paths, the pointer still has the bound bytes, and the registration still
///  names the pointer.
/// Why:
///   A write into the worktree, or a checkout by Git, must go to the exact worktree the add made; if another process moved
///  or replaced either object, the step stops and the add is reported, with nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await verifyAssociation(bound); // throws the first difference
/// ```
pub fn verify_association(bound: &Bound) -> Result<(), String> {
    let binding: &Binding = &bound.binding;
    let shown: String = binding.destination.display().to_string();
    if path_identity(binding.destination.as_path()) != Some(binding.destination_id) {
        return Err(format!("{shown} is no longer the directory Git made for this add"));
    }
    if path_identity(binding.admin.as_path()) != Some(binding.admin_id) {
        return Err(format!("the registration {} is no longer the one Git made for this add", binding.admin.display()));
    }
    let pointer: Vec<u8> = read_through(&bound.destination, OsStr::new(".git")).map_err(|error| { return format!("{shown}/.git is unreadable ({error})"); })?;
    if pointer != binding.pointer {
        return Err(format!("{shown}/.git is no longer the pointer Git wrote"));
    }
    // What Git resolves the unchanged pointer to now: the target is opened without a link and must be the held registration.
    let target: PathBuf = pointer_target(binding.destination.as_path(), pointer.as_slice())?;
    let resolved: File = open_root(target.as_path()).map_err(|error| { return not_canonical(shown.as_str(), target.as_path(), error.to_string().as_str()); })?;
    if identity_of(&resolved).ok() != Some(binding.admin_id) || identity_of(&bound.admin).ok() != Some(binding.admin_id) {
        return Err(format!("the pointer in {shown}/.git now resolves to another registration than the one Git made for this add"));
    }
    let recorded: Vec<u8> = read_through(&bound.admin, OsStr::new("gitdir")).map_err(|error| { return format!("the registration of {shown} is unreadable ({error})"); })?;
    let line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
    if recorded_destination(binding.admin.as_path(), line)?.as_path() != binding.destination.join(".git").as_path() {
        return Err(format!("the registration {} no longer names {shown}", binding.admin.display()));
    }
    return Ok(());
}

/// Binding controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_binding_tests.rs"]
mod tests;
