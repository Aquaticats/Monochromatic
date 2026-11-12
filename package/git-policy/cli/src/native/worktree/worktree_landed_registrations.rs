//! What:
//!  The linked worktrees a repository's common Git directory registers,
//!  with the branch each one has checked out,
//!  and the ones that were already registered at a moment the caller names.
//! Why:
//!   The registrations are the only list of worktrees a repository owns,
//!  and one file read per worktree names the branch that decides whether it is a candidate at all.
//!  This is the cheap half of the landed sweep:
//!  Git runs only for a branch a merge landed,
//!  never once per registered worktree.
//!  A sweep that reads the list after its merge commit exists would take a worktree another command
//!  added in between,
//!  which did not exist when the merge landed,
//!  so the command that runs the merge names the registrations it found before it started
//!  and the sweep removes only those.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registered = registrationsOf(commonDir); // throws the reason
//! const before = registeredBefore(commonDir); // the names that existed before the merge
//! ```

use std::collections::HashSet;
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

/// One registration of the repository,
/// with the branch its `HEAD` names when it names one.
/// Why:
///   A registration is the only list entry a repository owns,
/// and a detached `HEAD` is a state a worktree can have:
/// the sweep has no candidate there,
/// because nothing can land on a branch it does not have,
/// while a report about the worktree still has to name it as the worktree it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegisteredWorktree = { admin: string; worktree: string; branch?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegisteredWorktree {
    /// The administrative directory.
    pub(crate) admin: PathBuf,
    /// The worktree root the registration names.
    pub(crate) worktree: PathBuf,
    /// The branch its `HEAD` names,
    /// as a full ref name,
    /// or nothing for a detached `HEAD`.
    pub(crate) branch: Option<String>,
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
    let mut found: Vec<Registration> = Vec::new();
    for one in registered_worktrees(common_dir)? {
        let Some(branch) = one.branch else { continue };
        found.push(Registration { admin: one.admin, worktree: one.worktree, branch });
    }
    return Ok(found);
}

/// What:
///  The registration that names exactly `worktree` as its worktree,
///  whatever its `HEAD` names.
/// Why:
///   A report about a worktree that stands below another one names it and the command that removes it,
/// and a
///  detached `HEAD` is a worktree the repository registers like any other:
/// asking only the candidates
///  of a landed sweep made it an unnamed volume,
/// and the report then sent the person to delete its
///  files with the btrfs tool (a review finding,
/// F2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktree = registrationNaming(commonDir, path); // RegisteredWorktree | undefined
/// ```
///
/// # Errors
///
/// The words for a `worktrees` folder that cannot be listed.
pub(crate) fn registration_naming(common_dir: &Path, worktree: &Path) -> Result<Option<RegisteredWorktree>, String> {
    for one in registered_worktrees(common_dir)? {
        if one.worktree == worktree {
            return Ok(Some(one));
        }
    }
    return Ok(None);
}

/// What:
///  Every registration of the repository,
///  in one listing of its `worktrees` folder,
///  with the branch each one's `HEAD` names when it names one.
/// Why:
///   The sweep and the report ask the same files the same question,
/// and only what they keep differs:
/// one
///  needs a branch,
/// the other needs the registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registered = registeredWorktrees(commonDir); // throws the reason
/// ```
fn registered_worktrees(common_dir: &Path) -> Result<Vec<RegisteredWorktree>, String> {
    let folder: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", folder.display())),
    };
    let mut found: Vec<RegisteredWorktree> = Vec::new();
    for entry in listing.flatten() {
        let admin: PathBuf = entry.path();
        let Some(worktree) = registered_worktree(admin.as_path()) else {
            continue;
        };
        let branch: Option<String> = head_branch(admin.as_path());
        found.push(RegisteredWorktree { admin, worktree, branch });
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
    if !branch.starts_with("refs/heads/") {
        return None;
    }
    return Some(branch.to_owned());
}

/// The administrative directories a repository registered at one moment.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegisteredBefore {
    /// Their names,
    /// which is what a registration's directory is called and what a removal deletes.
    admins: HashSet<PathBuf>,
}

impl RegisteredBefore {
    /// Whether this registration was already there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// before.holds(registration) // boolean
    /// ```
    pub(crate) fn holds(&self, one: &Registration) -> bool {
        return self.admins.contains(&one.admin);
    }
}

/// What:
///  The names of the administrative directories the repository registers now,
///  for a sweep that runs after a merge that has not happened yet.
/// Why:
///   A merge that lands a branch removes the worktrees whose branches it landed,
///  and a worktree added while the merge ran,
///  or after it landed,
///  did not exist when the merge did,
///  so only the names read before it started may be removed.
///  This is one directory read and no file read:
///  the branch of each registration is read after the merge,
///  for the candidates alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// registeredBefore(commonDir) // { admins }, or the reason they cannot be listed
/// ```
///
/// # Errors
///
/// The words for a `worktrees` folder that cannot be listed.
pub(crate) fn registered_before(common_dir: &Path) -> Result<RegisteredBefore, String> {
    let folder: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(RegisteredBefore::default()),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", folder.display())),
    };
    let mut admins: HashSet<PathBuf> = HashSet::new();
    for entry in listing.flatten() {
        admins.insert(entry.path());
    }
    return Ok(RegisteredBefore { admins });
}
