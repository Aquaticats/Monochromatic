//! What:
//!  The linked worktrees a repository's common Git directory registers,
//!  with the branch each one has checked out.
//! Why:
//!   The registrations are the only list of worktrees a repository owns,
//!  and one file read per worktree names the branch that decides whether it is a candidate at all.
//!  This is the cheap half of the landed sweep:
//!  Git runs only for a branch a merge landed,
//!  never once per registered worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registered = registrationsOf(commonDir); // throws the reason
//! ```

use std::path::{Path, PathBuf};

/// One registration of the repository,
/// as the sweep needs it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Registration {
    /// The administrative directory.
    pub(crate) admin: PathBuf,
    /// The worktree root the registration names.
    pub(crate) worktree: PathBuf,
    /// The branch its `HEAD` names,
    /// as a full ref name.
    pub(crate) branch: String,
}

/// What:
///  Every linked worktree the repository registers,
///  with the branch each one has checked out.
/// Why:
///   The registrations are the only list of worktrees this repository owns,
///  and each one's `HEAD` names its branch without a Git start:
///  a file read decides which candidates exist,
///  and Git runs only for a branch that landed.
///  A worktree on a detached `HEAD` names no branch,
///  so it is no candidate:
///  nothing can land on a branch it does not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registered = registrationsOf(commonDir); // throws the reason
/// ```
pub(crate) fn registrations_of(common_dir: &Path) -> Result<Vec<Registration>, String> {
    let folder: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", folder.display())),
    };
    let mut found: Vec<Registration> = Vec::new();
    for entry in listing.flatten() {
        let admin: PathBuf = entry.path();
        let Some(worktree) = registered_worktree(admin.as_path()) else {
            continue;
        };
        let Some(branch) = head_branch(admin.as_path()) else {
            continue;
        };
        found.push(Registration { admin, worktree, branch });
    }
    return Ok(found);
}

/// The worktree root a registration's `gitdir` file names,
/// when that root is still a directory.
fn registered_worktree(admin: &Path) -> Option<PathBuf> {
    let text: Vec<u8> = std::fs::read(admin.join("gitdir")).ok()?;
    let recorded: PathBuf = PathBuf::from(String::from_utf8_lossy(text.trim_ascii_end()).into_owned());
    let pointer: PathBuf = if recorded.is_absolute() { recorded } else { admin.join(recorded) };
    let root: PathBuf = std::fs::canonicalize(pointer.parent()?).ok()?;
    return root.is_dir().then_some(root);
}

/// The branch the registration's `HEAD` names,
/// as a full ref name,
/// or nothing for a detached `HEAD`.
fn head_branch(admin: &Path) -> Option<String> {
    let text: String = std::fs::read_to_string(admin.join("HEAD")).ok()?;
    let line: &str = text.trim_end();
    let branch: &str = line.strip_prefix("ref: ")?;
    return branch.starts_with("refs/heads/").then(|| { return branch.to_owned() });
}
