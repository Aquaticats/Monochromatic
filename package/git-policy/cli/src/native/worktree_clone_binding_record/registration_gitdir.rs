//! What:
//!  The path a registration's own `gitdir` file names,
//!  as the folder that owns the record reads it.
//! Why:
//!  A record names the registration that belongs to its slot,
//!  and the file below that registration is what tells a complete
//!  registration from the half-made one Git's own killed add leaves
//!  (`worktree_clone_add_route::add_registration_reads::read_the_gitdir`
//!  reads the same file for the add's own proof).
//!  The read is bounded and takes the type from the open handle,
//!  so a name another process put at the file waits for nothing,
//!  and a file nobody may read is answered as unread rather than as absent:
//!  a registration that stands with a link this account cannot see proves
//!  nothing about the link.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theGitdirARegistrationNames(path); // { kind: 'named', path } | { kind: 'no-file' } | { kind: 'unreadable', why }
//! ```

use crate::private_storage::ReadRefusal;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};

/// The most bytes one registration's `gitdir` file may hold:
/// the kernel's own path bound and the newline Git writes after it.
///
/// The add's own reader takes the same bound for the same reason
/// (`add_registration_reads::RECORD_LIMIT`),
/// and a path longer than the kernel's bound is one no registration can name.
pub(crate) const GITDIR_RECORD_LIMIT: u64 = 8 + 4096 + 2;

/// What:
///  What one registration's `gitdir` file answered.
/// Why:
///  "No such file",
///  "a link this account may not read" and "a path" are three
///  answers a report acts on differently,
///  and one `Option` would fold the last two into the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ARegistrationsGitdir = { kind: 'named'; path } | { kind: 'no-file' } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ARegistrationsGitdir {
    /// The file names this path,
    /// with a relative spelling resolved against the registration's folder.
    Named(PathBuf),
    /// Nothing stands at the file's name,
    /// which is what a kill inside Git's own `worktree add` leaves.
    NoFile,
    /// The file stands and could not be read,
    /// in the record reader's own words.
    Unreadable {
        /// Why the file was not read.
        why: String,
    },
}

/// What:
///  The path one registration's `gitdir` file names.
/// Why:
///  Git writes an absolute back-link and a person's hand-written one may be
///  relative to the registration,
///  so the relative spelling is resolved here once rather than judged as damage
///  by every reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theGitdirARegistrationNames(registration);
/// ```
pub(crate) fn the_gitdir_a_registration_names(registration: &Path) -> ARegistrationsGitdir {
    let file: PathBuf = registration.join("gitdir");
    let bytes: Vec<u8> =
        match crate::private_storage::read_regular_file(file.as_path(), GITDIR_RECORD_LIMIT) {
            Ok(found) => found,
            Err(ReadRefusal::Missing) => return ARegistrationsGitdir::NoFile,
            Err(refusal) => {
                return ARegistrationsGitdir::Unreadable {
                    why: format!("{} cannot be read ({refusal:?})", file.display()),
                };
            }
        };
    let line: &[u8] = bytes.strip_suffix(b"\n").unwrap_or(bytes.as_slice());
    let named: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(line));
    return ARegistrationsGitdir::Named(if named.is_absolute() {
        named
    } else {
        registration.join(named)
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_gitdir_tests.rs"]
mod tests;
