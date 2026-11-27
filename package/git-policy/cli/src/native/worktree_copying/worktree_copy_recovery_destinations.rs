//! What:
//!  Which destinations one recovery pass may finish a copy into,
//!  and whether a destination is still a registered worktree of the repository.
//! Why:
//!  Finishing a copy stages or installs the source's ignored state,
//!  which costs what the tree costs,
//!  so only the worktree a copy fills may pay for it,
//!  while the discards and the keeps stay open to every command.
//!  A destination's registration and its own `.git` pointer together answer whether a copy
//!  may go on,
//!  and the passes that ask live above this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const destinations = { kind: 'only-worktree', root }; // or 'every' | 'no-worktree'
//! const registration = destinationRegistration({ commonDir, destinationRoot });
//! ```

use crate::plain_file::read_plain_file;
use crate::worktree_convert_inspection::GIT_FILE_LIMIT;
use crate::worktree_copy_registration_naming::{
    NamedRegistration,
    absent_path,
    registration_naming,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Which destinations one recovery pass may finish a copy into.
/// Why:
///   Finishing a copy stages or installs the source's ignored state,
/// which costs what the tree costs:
/// minutes for a large checkout.
/// Only the worktree a copy fills waits for it,
/// because that is the worktree that needs the files;
/// a command in another worktree never pays for it,
/// which is the owner's "Agents should never wait on each other".
/// The discards and the keeps stay open to every command:
/// they read a few records and remove the files of a transaction nothing may resume,
/// and a worktree that is gone has no command left to run in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryDestinations = 'every' | { onlyWorktree: string } | 'no-worktree';
/// ```
pub enum RecoveryDestinations<'a> {
    /// Every destination of the repository:
    /// the library's own sweep,
    /// which a command never holds.
    EveryDestination,
    /// Only the worktree the command runs in.
    OnlyThisWorktree(&'a Path),
    /// None,
    /// because the command runs in a shape that holds no worktree.
    NoWorktree,
}

/// What:
///  Whether this pass may finish the copy into `destination`.
/// Why:
///   The recorded destination is Git's canonical absolute path and so is the worktree the command
/// reports,
/// but a path a symbolic link crosses is still the same worktree,
/// so both are resolved before they are compared
/// and an answer that cannot be resolved is unmatched rather than assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finishes = (destinations, destination) => destinations === 'every' || samePath(destinations.onlyWorktree, destination);
/// ```
fn same_worktree(
    left: &Path,
    right: &Path,
) -> bool {
    if left == right {
        return true;
    }
    return match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left_canonical), Ok(right_canonical)) => left_canonical == right_canonical,
        _ => false,
    };
}

/// What:
///  Whether a recovery pass may finish a copy into `destination`,
/// asked before the copy's staging or installation.
/// Why:
///   One predicate answers for the journals and for the copy an add owed,
/// so both refuse another worktree's work the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finishesHere = (destinations, destination) => destinations === 'every' || (destinations.onlyWorktree !== undefined && sameWorktree(destinations.onlyWorktree, destination));
/// ```
pub fn finishes_destination(
    destinations: &RecoveryDestinations<'_>,
    destination: &Path,
) -> bool {
    return match destinations {
        RecoveryDestinations::EveryDestination => true,
        RecoveryDestinations::OnlyThisWorktree(root) => same_worktree(root, destination),
        RecoveryDestinations::NoWorktree => false,
    };
}

/// What:
///  Whether a journal's destination is still a linked worktree of the journal's repository.
/// Why:
///   A destination the repository registers nowhere can receive nothing,
/// so its journal is discarded.
/// The repository's registrations answer that question,
/// not the destination's `.git` file:
/// a command that unlinks,
/// truncates or rewrites that file leaves it missing,
/// cut short or naming something else for as long as it runs,
/// and the worktree stays registered throughout.
/// A registration that names the destination while its pointer does not is not an absence,
/// and neither is a registration or a destination that could not be read,
/// so the copy is kept and the person is told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestinationRegistration = 'registered' | 'unregistered' | 'unconfirmed' | 'unproven';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Registration {
    /// The destination is registered under the common directory,
    /// and its pointer names that registration.
    Registered,
    /// No registration under the common directory names the destination.
    Unregistered,
    /// A registration names the destination,
    /// but the destination's pointer does not name it,
    /// so whether the copy may go on cannot be told.
    Unconfirmed,
    /// The repository's registrations,
    /// or the destination's own path,
    /// could not be examined,
    /// so whether a registration names the destination cannot be told.
    Unproven,
}

/// What:
///  The directory a `.git` pointer names,
/// and nothing when the file is not a pointer at all.
/// Why:
///   Git strips every trailing line feed and carriage return from the file,
/// in any order:
/// `while (buf[len - 1] == '\n' || buf[len - 1] == '\r') len--;` in `read_gitfile_raw`,
/// Git's `setup.c`.
/// A pointer a Windows tool ended with a carriage return and a line feed therefore names the directory it holds;
/// a reader that strips only the line feed asks for a directory whose name ends in a carriage return,
/// and no worktree has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pointerDirectory = (file: Buffer) => file.subarray(0, 8).toString() === 'gitdir: ' ? file.subarray(8).toString().replace(/[\r\n]+$/, '') : undefined
/// ```
fn pointer_directory(recorded: &[u8]) -> Option<&[u8]> {
    let named: &[u8] = recorded.strip_prefix(b"gitdir: ")?;
    let mut end: usize = named.len();
    while end > 0 && (named[end - 1] == b'\n' || named[end - 1] == b'\r') {
        end -= 1;
    }
    return Some(&named[..end]);
}

/// What:
///  The administrative directory a destination's `.git` pointer names,
/// when the file holds a whole pointer that resolves.
/// Why:
///   The pointer is read as bytes,
/// as a plain file without blocking and without following a link,
/// and never past `GIT_FILE_LIMIT`,
/// because the pending check runs before every forwarded command and a named pipe at that name would stop it.
/// Its answer confirms a registration and never stands for one:
/// a command killed while it rewrites the file leaves it empty,
/// cut short or naming another directory,
/// and that says nothing about whether the worktree is registered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pointerAdmin = (destination) => tryResolve(readPlainFile(join(destination, '.git')))
/// ```
fn pointer_admin(destination: &Path) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let recorded: Vec<u8> =
        read_plain_file(destination.join(".git").as_path(), GIT_FILE_LIMIT).ok()?;
    let named: &[u8] = pointer_directory(recorded.as_slice())?;
    if named.is_empty() {
        return None;
    }
    // The pointer is bytes: Git writes the directory it names as it is, so a path that is not text must be read whole.
    return std::fs::canonicalize(destination.join(std::ffi::OsStr::from_bytes(named))).ok();
}

/// What:
///  Whether a journal's destination is still registered under the common directory.
/// Why:
///   The repository's own registrations answer this,
/// and the destination's `.git` file only confirms their answer:
/// a command that unlinks,
/// truncates or rewrites that file leaves it missing,
/// cut short or naming another directory while the worktree stays registered,
/// so reading it alone discarded copies of worktrees that were still there.
/// Only a proven absence of the registration answers `Unregistered`;
/// every other state of the pointer,
/// and every file that could not be read,
/// is an answer that proves nothing,
/// which keeps the copy and tells the person.
/// No answer here is a failure:
/// the pending check runs before every forwarded command,
/// so a file system that cannot be read must not make the command fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registration = registrationNaming(commonDir, destination);
/// if (registration === none) return 'unregistered';
/// if (registration === unknown) return 'unproven';
/// return pointerAdmin(destination) === registration.named ? 'registered' : 'unconfirmed';
/// ```
fn read_registration(
    common_dir: &Path,
    destination: &Path,
) -> Registration {
    // A destination that is gone, or is not a directory, holds nothing a copy could fill: that absence is proven.
    match std::fs::canonicalize(destination) {
        Ok(canonical) if canonical == destination => {}
        Ok(_) => return Registration::Unregistered,
        Err(error) if absent_path(&error) => return Registration::Unregistered,
        Err(_unreadable) => return Registration::Unproven,
    }
    return match registration_naming(common_dir, destination) {
        // No registration names the destination, so no copy is this destination's.
        NamedRegistration::None => Registration::Unregistered,
        // A registration names it: what the pointer says decides whether the copy may go on.
        NamedRegistration::Named(admin) => match pointer_admin(destination) {
            Some(named) if named == admin => Registration::Registered,
            _ => Registration::Unconfirmed,
        },
        // A `gitdir` that could not be read whole, or could not be resolved, proves nothing about the destination.
        NamedRegistration::Unknown => Registration::Unproven,
    };
}

/// What:
///  Whether a destination is still a linked worktree of the common directory.
/// Why:
///   Recovery decides whether to finish or discard a copy from this answer,
/// and every state of the destination's `.git` file and of the registrations reaches one of the four.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registration = destinationRegistration({ commonDir, destinationRoot });
/// ```
pub fn destination_registration(
    common_dir: &Path,
    destination: &Path,
) -> Registration {
    return read_registration(common_dir, destination);
}
