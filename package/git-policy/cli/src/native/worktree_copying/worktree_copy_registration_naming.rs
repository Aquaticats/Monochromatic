//! What:
//!  Which of a repository's registrations names one destination.
//! Why:
//!  A linked worktree is registered by its administrative directory under `<common>/worktrees`:
//! the `gitdir` file of that directory names the worktree's `.git` file,
//! and it does so whatever a command is doing to that `.git` file at the time.
//!  Recovery decided a copy was abandoned by reading the destination's `.git` pointer,
//! and a command that unlinks,
//! truncates or rewrites that file leaves it missing,
//! cut short or naming something else while the worktree stays registered;
//! the registrations answer the same question without reading the pointer at all.
//!  Nothing here waits or fails:
//! a `gitdir` that cannot be read,
//! or holds nothing that could be read,
//! is an answer that proves nothing rather than an absence,
//! and a directory that cannot be listed is reported as unknown rather than as an absence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const naming = registrationNaming(commonDir, destination); // { named } | none | unknown
//! ```

use crate::fs_beneath::ObjectId;
use crate::plain_file::SMALL_FILE_LIMIT;
use crate::plain_file::read_plain_file;
use crate::worktree_add_binding::resolve_lexically;
use std::path::{Path, PathBuf};

/// What:
///  The registration that names a destination,
/// or why none does.
/// Why:
///  `read_registration` discards a pending copy only on a proven absence,
/// so the answer that proves nothing is kept apart from the answer that proves there is no registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamedRegistration = { named: string } | 'none' | 'unknown';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedRegistration {
    /// The administrative directory whose `gitdir` file names this destination.
    Named(PathBuf),
    /// No registration names it.
    None,
    /// The registrations cannot all be read,
    /// so whether one names it is unknown.
    Unknown,
}

/// What:
///  What one registration's `gitdir` file says about a destination.
/// Why:
///  Only a `gitdir` that was read whole and names another place proves that this registration names nothing;
/// a file a command is rewriting is empty,
/// gone or unreadable for as long as it runs,
/// and it proves nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Naming = 'names' | 'names-nothing' | 'unproven';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Naming {
    /// The registration's `gitdir` names the destination.
    Names,
    /// The registration names a worktree that is gone or is another place.
    NamesNothing,
    /// The registration's `gitdir` was not read whole,
    /// or could not be resolved,
    /// so it proves nothing.
    Unproven,
}

/// What:
///  Which registration under `<common>/worktrees` names `destination`.
/// Why:
///   The destination's own `.git` file is not evidence:
/// a command that rewrites it leaves it empty,
/// half written or naming another directory,
/// and the registration stays.
/// A registration that was read whole and names no worktree,
/// or one whose worktree is gone,
/// is one Git would prune,
/// so it names nothing,
/// while a `gitdir` that is gone,
/// empty or unreadable proves nothing.
///  The destination is compared by the one canonical spelling both exist under,
/// because a registration written before the repository moved behind a link names the worktree all the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const admin of readdir(join(commonDir, 'worktrees'))) if (namesDestination(admin, destination)) return { named: admin }
/// ```
pub fn registration_naming(common_dir: &Path, destination: &Path) -> NamedRegistration {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(worktrees.as_path()) {
        Ok(listing) => listing,
        // No `worktrees` directory is a repository with no linked worktree at all.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return NamedRegistration::None,
        // A repository whose registrations cannot be listed proves nothing, so a copy is kept rather than discarded.
        Err(_unreadable) => return NamedRegistration::Unknown,
    };
    let mut unproven: bool = false;
    for listed in listing {
        let admin: PathBuf = match listed {
            Ok(entry) => entry.path(),
            Err(_unreadable) => {
                unproven = true;
                continue;
            }
        };
        match read_plain_file(admin.join("gitdir").as_path(), SMALL_FILE_LIMIT) {
            Ok(recorded) => match naming_by(admin.as_path(), recorded.as_slice(), destination) {
                Naming::Names => return NamedRegistration::Named(admin),
                Naming::NamesNothing => {}
                Naming::Unproven => unproven = true,
            },
            // A `gitdir` a command is rewriting is gone or unreadable for as long as it runs, so it proves nothing.
            Err(_unreadable) => unproven = true,
        }
    }
    return if unproven { NamedRegistration::Unknown } else { NamedRegistration::None };
}

/// What:
///  Which worktree of this repository is one object,
///  or why none is.
/// Why:
///   A record that captured a worktree's object still names it after `git worktree move` renamed the directory,
/// and the registration whose `gitdir` names that object gives the path the worktree stands at now.
///   The answer that proves nothing is kept apart from the answer that proves there is no such worktree:
/// a `gitdir` is unreadable for a moment while a command rewrites it,
/// and a caller that settles a record on it would forget a worktree that still holds less than its source ignores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegisteredObject = { root: string } | 'none' | 'unknown';
/// ```
pub(crate) enum RegisteredObject {
    /// The root of the worktree this object is.
    Root(PathBuf),
    /// No registration holds it.
    None,
    /// The registrations cannot all be read,
    /// so whether one holds it is unknown.
    Unknown,
}

/// What:
///  Which worktree of this repository is the object `identity` names.
/// Why:
///   Git renames a worktree's directory when it moves it and keeps the object,
/// so the identity a record captured still names that worktree while its recorded path no longer does;
/// the registration's `gitdir` gives the path the worktree stands at now,
/// which is the path every line and command about it must name.
///   A registration that cannot be read proves nothing:
/// the answer says so rather than naming no worktree,
/// and no record is settled on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registeredObject = (commonDir, identity) => readdir(join(commonDir, 'worktrees')).map(registrationRoot).find((root) => identityAt(root) === identity);
/// ```
pub(crate) fn registered_object(common_dir: &Path, identity: ObjectId) -> RegisteredObject {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(worktrees.as_path()) {
        Ok(listing) => listing,
        // No `worktrees` directory is a repository with no linked worktree, so no registration holds any object.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return RegisteredObject::None,
        // A repository whose registrations cannot be listed proves nothing, so no record is settled on it.
        Err(_unreadable) => return RegisteredObject::Unknown,
    };
    let mut unproven: bool = false;
    for listed in listing {
        let admin: PathBuf = match listed {
            Ok(entry) => entry.path(),
            Err(_unreadable) => {
                unproven = true;
                continue;
            }
        };
        match registration_root(admin.as_path()) {
            Registered::Root(root) if crate::worktree_add_binding::path_identity(root.as_path()) == Some(identity) => return RegisteredObject::Root(root),
            Registered::Root(_another_object) => {}
            Registered::NoRoot => {}
            Registered::Unproven => unproven = true,
        }
    }
    return if unproven { RegisteredObject::Unknown } else { RegisteredObject::None };
}

/// What:
///  The worktree root one registration's `gitdir` file names,
///  or why it names none.
/// Why:
///  `gitdir` holds the path of the worktree's `.git` file,
/// absolute or relative to the administrative directory,
/// and the directory that holds it is the worktree;
/// a file that is gone,
/// empty or unreadable proves nothing about the worktree it describes,
/// while one that names a worktree which is gone names none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registrationRoot = (admin) => { const line = read('gitdir'); if (!line) return 'unproven'; const resolved = resolveLexically(admin, line.trimEnd()); return canonicalize(dirname(resolved)) ?? 'no-root'; };
/// ```
fn registration_root(admin: &Path) -> Registered {
    let Ok(recorded) = read_plain_file(admin.join("gitdir").as_path(), SMALL_FILE_LIMIT) else {
        return Registered::Unproven;
    };
    let line: &[u8] = recorded_line(recorded.as_slice());
    if line.is_empty() {
        return Registered::Unproven;
    }
    let Ok(marker) = resolve_lexically(admin, line) else {
        return Registered::NoRoot;
    };
    let Some(named) = marker.parent() else {
        return Registered::NoRoot;
    };
    return match std::fs::canonicalize(named) {
        Ok(canonical) => Registered::Root(canonical),
        // A recorded worktree that is not there is one Git would prune, so it names no worktree of this repository;
        // a path that cannot be resolved for another reason proves nothing.
        Err(error) if absent_path(&error) => Registered::NoRoot,
        Err(_unresolved) => Registered::Unproven,
    };
}

/// What:
///  The worktree root one registration names,
///  or why it names none.
/// Why:
///  A registration whose recorded worktree is gone names nothing a caller could name,
/// while one that cannot be read proves nothing and is answered apart from the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Registered = { root: string } | 'no-root' | 'unproven';
/// ```
enum Registered {
    /// The directory the registration names.
    Root(PathBuf),
    /// The registration names no worktree:
    /// it records no path,
    /// or the worktree it records is gone.
    NoRoot,
    /// The registration cannot be read,
    /// so it proves nothing.
    Unproven,
}

/// What:
///  The path bytes a registration's `gitdir` file holds,
/// with Git's one line feed removed.
/// Why:
///   Git ends the file with one line feed and writes the path as it is (`write_file` in its `worktree.c`),
/// and trimming every space around it would rename a worktree whose path begins or ends with one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordedLine = (recorded) => recorded.endsWith('\n') ? recorded.slice(0, -1) : recorded;
/// ```
fn recorded_line(recorded: &[u8]) -> &[u8] {
    return recorded.strip_suffix(b"\n").unwrap_or(recorded);
}

/// What:
///  What one registration's `gitdir` says about `destination`.
/// Why:
///  `gitdir` holds the path of the worktree's `.git` file,
/// absolute or relative to the administrative directory,
/// and its last component is always `.git`;
/// the directory that holds it is the worktree.
/// A registration whose recorded worktree is gone names nothing this copy could fill,
/// while one that could not be resolved for another reason proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = read('gitdir').replace(/\n$/, ''); const marker = resolveLexically(admin, line); return namingOf(canonicalize(dirname(marker)), destination)
/// ```
fn naming_by(admin: &Path, recorded: &[u8], destination: &Path) -> Naming {
    let line: &[u8] = recorded_line(recorded);
    if line.is_empty() {
        return Naming::Unproven;
    }
    let marker: PathBuf = match resolve_lexically(admin, line) {
        Ok(resolved) => resolved,
        Err(()) => return Naming::NamesNothing,
    };
    let Some(named) = marker.parent() else {
        return Naming::NamesNothing;
    };
    return match std::fs::canonicalize(named) {
        Ok(canonical) if canonical.as_path() == destination => Naming::Names,
        Ok(_another_place) => Naming::NamesNothing,
        // A recorded worktree that is not there is one Git would prune, and it can name no copy this fills;
        // a path that cannot be resolved for another reason (no search permission, a symbolic link loop) proves nothing.
        Err(error) if absent_path(&error) => Naming::NamesNothing,
        Err(_unresolved) => Naming::Unproven,
    };
}

/// What:
///  Whether an error says the path,
/// or a part of it,
/// is not there.
/// Why:
///   A path that is absent answers a question about a registration,
/// while a path that cannot be read for another reason is a fault to report and never an absence.
/// What counts as an absence is decided in this one place,
/// for the recorded path and for the destination alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absentPath = (error) => error.code === 'ENOENT' || error.code === 'ENOTDIR'
/// ```
pub(crate) fn absent_path(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ENOTDIR);
}

/// Registration naming controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_registration_naming_tests.rs"]
mod tests;
