//! What:
//!  The proof that this command made one registration:
//! the file a command writes in a registration's administrative directory,
//! and whose it is.
//! Why:
//!   A `git worktree add` that Git refuses because the destination exists cannot tell from the path
//!  whether the registration there is its own,
//! so the first command to write the file proves it made the registration,
//! and the write is one exclusive create,
//! so of several commands exactly one succeeds and the rest read its token.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owner = claimRegistration(heldAdmin, token); // { mine: true } | { mine: false, owner }
//! ```

use std::ffi::OsStr;
use std::fs::File;

/// The file inside a registration's administrative directory that names the command which proved it made that
/// registration.
/// Why it is one file per registration and never removed:
///  the proof outlives the command that wrote it,
///  so a second `git worktree add` that finds the registration at its own
///  destination still learns it is another command's,
///  and the file goes away with the registration itself
///  (`git worktree remove` and `git worktree prune` delete the directory).
pub const REGISTRATION_OWNER_NAME: &str = "monochromatic-git-policy-cli-copy-owner";

/// What one registration says about which command made it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistrationOwner {
    /// This command proved the registration is its own,
    /// either because it wrote the file or because nobody had.
    Mine,
    /// Another command proved the registration is its own,
    /// and the token says which process it was.
    Another(String),
}

/// What:
///  Say that this command made the registration held open as `admin`,
/// and answer whose it is.
/// Why:
///   A `git worktree add` that Git refuses because the destination exists cannot tell
/// from the path whether the registration there
///  is its own:
/// another add that ran while its own Git ran leaves
/// a registration exactly where its own would be.
/// The first command to write this
///  file proves it made the registration.
/// The write is one exclusive create,
/// so of several commands exactly one succeeds and
///  the rest read its token;
/// nothing waits and no lock file exists.
/// The proof is published whole and flushed by nobody:
/// it guards a killed command,
/// whose writes stay in the page cache,
/// and native Git flushes nothing for the worktree add it serves,
/// while a flush of it is a commit of the whole btrfs filesystem on the add's waiting path
/// (`SPEC.md`,
/// "Local durability follows what the remote already holds").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = await claimRegistration(heldAdmin, token); // { mine: true } | { mine: false, owner }
/// ```
pub fn claim_registration(
    admin: &File,
    token: &str,
) -> std::io::Result<RegistrationOwner> {
    let name: &OsStr = OsStr::new(REGISTRATION_OWNER_NAME);
    // `match` on the exclusive create: the one command that made the file is the registration's owner.
    match crate::fs_beneath::create_file_at(admin, name, 0o600) {
        Ok(mut written) => {
            std::io::Write::write_all(&mut written, token.as_bytes())?;
            return Ok(RegistrationOwner::Mine);
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let mut read: File = crate::fs_beneath::open_file_at(admin, name)?;
    let mut recorded: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut read, &mut recorded)?;
    let owner: String = String::from_utf8_lossy(recorded.as_slice())
        .trim()
        .to_string();
    if owner == token {
        return Ok(RegistrationOwner::Mine);
    }
    return Ok(RegistrationOwner::Another(owner));
}
